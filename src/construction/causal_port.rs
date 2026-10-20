// Learner-owned bridge: no additional actuator authority or host data labels.
impl Learner {
    pub fn enable_causal_memory(
        &mut self,
        config: super::causal_memory::CausalConfig,
    ) -> Result<()> {
        config.validate()?;
        if !self.self_directed
            || self.pending
            || self.poisoned
            || self.frozen_at.is_some()
            || self.next_id != 0
            || self.causal_memory.is_some()
        {
            return Err(Error::Busy);
        }
        self.causal_memory = Some(super::causal_memory::CausalMemory::new(
            config,
            self.active.program.fingerprint(),
            self.active.program.initial_state(),
        )?);
        Ok(())
    }
    pub fn causal_report(&self) -> Option<super::causal_memory::CausalReport> {
        self.causal_memory.as_ref().map(|m| m.report())
    }
    pub fn causal_memory_enabled(&self) -> bool {
        self.causal_memory.is_some()
    }
    pub fn action_count(&self) -> usize {
        self.config.actions
    }
    pub fn configuration(&self) -> &Config {
        &self.config
    }
    pub fn autonomy_policy(&self) -> Option<&super::autonomy::AutonomyPolicy> {
        self.autonomy.as_ref()
    }
    pub fn factual_event_count(&self) -> usize {
        usize::try_from(self.next_id).unwrap_or(usize::MAX)
    }
    pub(crate) fn carrier_swap_planning_depth(&mut self, depth: usize) -> usize {
        std::mem::replace(&mut self.config.planning_depth, depth)
    }
    pub fn causal_action_beliefs(
        &self,
        offers: &[super::autonomy::ActionOffer],
        goal: Option<&[Option<i64>]>,
        work: u64,
    ) -> Result<(Vec<super::causal_memory::ActionBelief>, u64)> {
        if offers.len() != self.config.actions || goal.is_some_and(|g| g.len() != self.config.width)
        {
            return Err(Error::InvalidData);
        }
        self.causal_memory
            .as_ref()
            .ok_or(Error::NoEvidence)?
            .beliefs(&self.current, offers, goal, work)
    }
    fn compact_causal_journal(&mut self) -> Result<()> {
        let half = self.records.len() / 2;
        let remove = self
            .records
            .iter()
            .enumerate()
            .skip(half)
            .find(|(_, r)| r.transition.restart)
            .map_or(half, |(i, _)| i);
        self.compact_causal_prefix(remove)
    }
    // Upper bound on the encoded journal, including optional forecasts and
    // extreme numbers. The configured record capacity is still an upper cap.
    fn causal_journal_capacity(&self) -> usize {
        self.config
            .journal_capacity
            .min(4 * 1024 * 1024 / (1024 + 160 * self.config.width))
    }
    fn ensure_legacy_journal_capacity(&mut self) -> Result<()> {
        if self
            .journal_encoding
            .is_none_or(|(rows, _)| rows != self.records.len())
        {
            self.journal_encoding = Some((
                self.records.len(),
                serde_json::to_vec(&self.records)
                    .map_err(|_| Error::InvalidData)?
                    .len(),
            ));
        }
        let reserve = 1024 + 160 * self.config.width;
        if self.journal_encoding.unwrap().1.saturating_add(reserve) > 3 * 1024 * 1024 {
            return Err(Error::Capacity);
        }
        Ok(())
    }
    pub(crate) fn carrier_take_causal_credits(&mut self) -> Vec<([f32; 5], f64)> {
        self.causal_memory
            .as_mut()
            .map_or_else(Vec::new, |m| m.take_credits())
    }
    fn compact_causal_prefix(&mut self, remove: usize) -> Result<()> {
        if self.pending || self.poisoned || self.causal_memory.is_none() {
            return Err(Error::Busy);
        }
        // A canceled prospective trial keeps its original risk debt. Never
        // splice an in-flight evidence window into an apparently fresh trial.
        if self.shadow.is_some() {
            self.cancel_shadow();
        }
        if remove == 0 || remove > self.records.len() {
            return Err(Error::Capacity);
        }
        let memory = self.causal_memory.as_ref().ok_or(Error::NoEvidence)?;
        let mut seed = memory.seed_state.clone();
        let mut meter = Meter::new(causal_replay_work(remove, 1));
        let replay = (|| -> Result<()> {
            for row in &self.records[..remove] {
                let t = &row.transition;
                if t.restart {
                    seed = self.active.program.initial_state()
                }
                seed = self
                    .active
                    .program
                    .step(&t.pre, t.action, &seed, &self.library, &mut meter)?
                    .1;
            }
            Ok(())
        })();
        if let Err(error) = replay {
            self.construction_work = self.construction_work.saturating_add(meter.spent);
            return Err(error);
        }
        let mut prior_meter = Meter::new(causal_replay_work(remove, 1));
        let mut prior_error = None;
        let previous =
            if let (Some(program), Some((hash, prior))) = (&self.previous, &memory.previous_seed) {
                if *hash != program.fingerprint() {
                    return Err(Error::Stale);
                }
                match replay_causal_window(
                    program,
                    &self.records[..remove]
                        .iter()
                        .map(|r| r.transition.clone())
                        .collect::<Vec<_>>(),
                    &self.library,
                    &mut prior_meter,
                    prior.clone(),
                ) {
                    Ok((_, state)) => Some((*hash, state)),
                    Err(error) => {
                        prior_error = Some(error);
                        None
                    }
                }
            } else {
                None
            };
        let cost = self.records[..remove]
            .iter()
            .fold(0u64, |n, r| n.saturating_add(r.external_cost));
        let digest = fingerprint(
            &serde_json::to_vec(&(memory.journal_digest, &self.records[..remove]))
                .map_err(|_| Error::InvalidData)?,
        );
        self.causal_memory.as_mut().unwrap().compacted(
            remove,
            cost,
            self.active.program.fingerprint(),
            seed,
            digest,
        );
        self.causal_memory.as_mut().unwrap().previous_seed = previous;
        if let Some(error) = prior_error {
            self.previous = None;
            self.last_error = Some(format!(
                "rollback unavailable after causal compaction: {error}"
            ));
        }
        self.records.drain(..remove);
        self.construction_work = self
            .construction_work
            .saturating_add(meter.spent)
            .saturating_add(prior_meter.spent);
        Ok(())
    }
    pub(crate) fn carrier_causal_proposal(
        &self,
        offers: &[super::autonomy::ActionOffer],
        goal: Option<&[Option<i64>]>,
        budget: u64,
        horizon: usize,
        remaining_cost: u64,
    ) -> (Option<super::causal_memory::CausalProposal>, u64) {
        self.causal_memory.as_ref().map_or((None, 0), |m| {
            m.propose_with_cost_limit(
                &self.current,
                offers,
                goal,
                self.config.planning_depth.min(horizon),
                remaining_cost,
                budget,
            )
        })
    }
    pub(crate) fn carrier_causal_feedback(
        &mut self,
        goal: Option<&[Option<i64>]>,
        post: &[i64],
        cost: u64,
        experiment: Option<[f32; 5]>,
        native_choice: bool,
    ) -> Vec<([f32; 5], f64)> {
        let Some(memory) = &mut self.causal_memory else {
            return Vec::new();
        };
        if !native_choice {
            memory.cancel_experiment_credit();
        }
        memory.feedback(goal, post, cost, experiment, self.frozen_at.is_none());
        memory.take_credits()
    }
}

// Each real VM execution already has a 100,000-unit ceiling. Replaying a
// bounded journal needs that allowance PER row and per executable, rather
// than one fixed lifetime-independent meter. Include replay bookkeeping.
fn causal_replay_work(rows: usize, passes: u64) -> u64 {
    (rows as u64)
        .saturating_mul(passes)
        .saturating_mul(100_001)
        .saturating_add(1_000_000)
}

fn replay_causal_window(
    program: &Program,
    data: &[Transition],
    library: &Library,
    meter: &mut Meter,
    mut state: Vec<i64>,
) -> Result<(Vec<Vec<i64>>, Vec<i64>)> {
    let mut predictions = Vec::with_capacity(data.len());
    for t in data {
        meter.charge(1)?;
        if t.restart {
            state = program.initial_state()
        }
        let (p, s) = program.step(&t.pre, t.action, &state, library, meter)?;
        predictions.push(p);
        state = s;
    }
    Ok((predictions, state))
}

fn causal_candidate_seed(
    memory: &super::causal_memory::CausalMemory,
    incumbent: &Program,
    candidate: &Program,
) -> Vec<i64> {
    let mut seed = candidate.initial_state();
    if candidate.cells.starts_with(&incumbent.cells) {
        seed[..incumbent.cells.len()].clone_from_slice(&memory.seed_state);
    }
    seed
}

fn calibrate_causal_window(
    program: &Program,
    data: &[Transition],
    library: &Library,
    meter: &mut Meter,
    seed: Vec<i64>,
) -> Result<(Vec<i64>, Vec<f64>)> {
    let (predictions, state) = replay_causal_window(program, data, library, meter, seed)?;
    let mut scale = vec![0.0; program.outputs.len()];
    for (p, t) in predictions.iter().zip(data) {
        for (j, (&pred, &actual)) in p.iter().zip(&t.post).enumerate() {
            scale[j] += (i128::from(pred) - i128::from(actual)).unsigned_abs() as f64;
        }
    }
    for s in &mut scale {
        *s = (*s / data.len().max(1) as f64).clamp(0.25, 1e9);
    }
    Ok((state, scale))
}

#[cfg(test)]
mod causal_checkpoint_tests {
    use super::*;
    #[test]
    fn expensive_valid_state_programs_compact_restore_and_rollback() {
        let mut l = Learner::new(
            Config {
                journal_capacity: 512,
                search_every: 100_000,
                ..Default::default()
            },
            vec![1],
        )
        .unwrap();
        l.enable_autonomy(super::super::autonomy::AutonomyPolicy::default())
            .unwrap();
        l.manage_from_carrier().unwrap();
        l.enable_self_directed_control().unwrap();
        let mut body = Expr::Arg(0);
        for _ in 0..7 {
            body = Expr::bin(Op::Add, body, Expr::integer(0));
        }
        l.active.program.cells.push(super::super::vm::StateCell {
            initial: 7,
            update: Expr::Fold {
                count: Box::new(Expr::integer(4096)),
                seed: Box::new(Expr::State(0)),
                body: Box::new(body),
            },
        });
        l.active.state = vec![7];
        l.enable_causal_memory(super::super::causal_memory::CausalConfig::default())
            .unwrap();
        let mut previous = l.active.program.clone();
        previous.cells[0].initial = 3;
        l.causal_memory.as_mut().unwrap().previous_seed = Some((previous.fingerprint(), vec![3]));
        l.previous = Some(previous);
        for id in 0..513 {
            let token = l.prepare(0).unwrap();
            l.commit(token, vec![1], id + 1, 1).unwrap();
        }
        assert!(l.causal_report().unwrap().compactions > 0);
        assert!(l.construction_work > 10_000_000);
        let mut l = Learner::from_checkpoint(&l.checkpoint().unwrap()).unwrap();
        assert_eq!(l.active.state, vec![7]);
        l.rollback().unwrap();
        assert_eq!(l.active.state, vec![3]);
        let token = l.prepare(0).unwrap();
        l.commit(token, vec![1], 514, 1).unwrap();
        assert_eq!(l.factual_event_count(), 514);
    }
    #[test]
    fn an_inexecutable_old_rollback_model_does_not_stop_current_learning() {
        let mut l = Learner::new(
            Config {
                journal_capacity: 32,
                search_every: 100_000,
                ..Default::default()
            },
            vec![1],
        )
        .unwrap();
        l.enable_autonomy(super::super::autonomy::AutonomyPolicy::default())
            .unwrap();
        l.manage_from_carrier().unwrap();
        l.enable_self_directed_control().unwrap();
        l.enable_causal_memory(super::super::causal_memory::CausalConfig::default())
            .unwrap();
        let fold = |body: Expr| Expr::Fold {
            count: Box::new(Expr::integer(4096)),
            seed: Box::new(Expr::integer(0)),
            body: Box::new(body),
        };
        let previous = Program {
            cells: vec![],
            outputs: vec![Expr::If(
                Box::new(Expr::bin(Op::Eq, Expr::Input(0), Expr::integer(0))),
                Box::new(Expr::integer(0)),
                Box::new(fold(fold(Expr::Arg(0)))),
            )],
        };
        previous.validate(1, &l.library).unwrap();
        l.causal_memory.as_mut().unwrap().previous_seed = Some((previous.fingerprint(), vec![]));
        l.previous = Some(previous);
        for id in 0..34 {
            let token = l.prepare(0).unwrap();
            l.commit(token, vec![1], id + 1, 1).unwrap();
        }
        assert!(l.previous.is_none());
        assert!(l
            .last_error
            .as_deref()
            .unwrap()
            .contains("rollback unavailable"));
        assert_eq!(
            Learner::from_checkpoint(&l.checkpoint().unwrap())
                .unwrap()
                .factual_event_count(),
            34
        );
    }
    #[test]
    fn legacy_large_capacity_stops_before_losing_checkpointability() {
        let observation = vec![i64::MIN; 32];
        let mut learner = Learner::new(
            Config {
                width: 32,
                journal_capacity: 65_536,
                search_every: 100_000,
                ..Default::default()
            },
            observation.clone(),
        )
        .unwrap();
        for id in 0..4097 {
            let token = match learner.prepare(0) {
                Ok(token) => token,
                Err(Error::Capacity) => break,
                Err(error) => panic!("unexpected failure: {error}"),
            };
            learner
                .commit(token, observation.clone(), id + 1, 1)
                .unwrap();
        }
        assert!(learner.records.len() > 100 && learner.records.len() <= 4096);
        assert!(matches!(learner.prepare(0), Err(Error::Capacity)));
        let bytes = learner.checkpoint().unwrap();
        assert!(bytes.len() < 4 * 1024 * 1024);
        let mut restored = Learner::from_checkpoint(&bytes).unwrap();
        assert_eq!(
            restored.checkpoint().unwrap(),
            bytes,
            "encoding cache is not persistent state"
        );
        assert!(matches!(restored.prepare(0), Err(Error::Capacity)));
    }
    #[test]
    fn wide_large_capacity_journal_remains_checkpointable_and_reports_lifetime_debt() {
        let observation = (0..32)
            .map(|i| if i % 2 == 0 { i64::MIN } else { i64::MAX })
            .collect::<Vec<_>>();
        let mut l = Learner::new(
            Config {
                width: 32,
                journal_capacity: 65_536,
                search_every: 100_000,
                ..Default::default()
            },
            observation.clone(),
        )
        .unwrap();
        l.enable_autonomy(super::super::autonomy::AutonomyPolicy::default())
            .unwrap();
        l.manage_from_carrier().unwrap();
        l.enable_self_directed_control().unwrap();
        l.enable_causal_memory(super::super::causal_memory::CausalConfig::default())
            .unwrap();
        for id in 0..8000 {
            let token = l.prepare(id as usize % 2).unwrap();
            l.commit(token, observation.clone(), id + 1, 3).unwrap();
            if id % 500 == 499 {
                let bytes = l.checkpoint().unwrap();
                assert!(bytes.len() < 16 * 1024 * 1024);
                l = Learner::from_checkpoint(&bytes).unwrap();
                let report = l.autonomy_report().unwrap();
                assert_eq!(report.events as u64, id + 1);
                assert_eq!(report.external_cost, (id + 1) * 3);
                assert_eq!(report.remaining_cost, 1_000_000 - (id + 1) * 3);
                assert_eq!(l.report().events as u64, id + 1);
            }
        }
        assert!(l.causal_report().unwrap().compactions > 0);
        assert!(l.records.len() < l.config.journal_capacity);
    }
    #[test]
    fn stateful_vm_anchor_survives_compaction_and_a_real_restart() {
        let cfg = Config {
            journal_capacity: 32,
            search_every: 10000,
            ..Default::default()
        };
        let mut l = Learner::new(cfg, vec![1]).unwrap();
        l.enable_autonomy(super::super::autonomy::AutonomyPolicy::default())
            .unwrap();
        l.manage_from_carrier().unwrap();
        l.enable_self_directed_control().unwrap();
        l.active.program.cells.push(super::super::vm::StateCell {
            initial: 7,
            update: Expr::bin(Op::Add, Expr::State(0), Expr::Lit(Value::Int(1))),
        });
        l.active.state = l.active.program.initial_state();
        l.enable_causal_memory(super::super::causal_memory::CausalConfig::default())
            .unwrap();
        for id in 0..113 {
            let token = l.prepare(0).unwrap();
            l.commit(token, vec![1], id + 1, 1).unwrap();
        }
        assert_eq!(l.active.state, vec![120]);
        assert!(l.records.len() < 32);
        let bytes = l.checkpoint().unwrap();
        let restored = Learner::from_checkpoint(&bytes).unwrap();
        assert_eq!(restored.active.state, l.active.state);
        assert_eq!(restored.checkpoint().unwrap(), bytes);
        l.active.state[0] = 0;
        assert!(
            Learner::from_checkpoint(&l.checkpoint().unwrap()).is_err(),
            "rehashing must not hide broken retained VM replay"
        );
        let mut restored = restored;
        restored.record_external_restart(vec![1]).unwrap();
        let bytes = restored.checkpoint().unwrap();
        assert_eq!(
            Learner::from_checkpoint(&bytes).unwrap().active.state,
            vec![7]
        );
    }
    #[test]
    fn a_prospective_model_trial_and_rollback_keep_register_meanings_after_compaction() {
        let cfg = Config {
            journal_capacity: 64,
            search_every: 10000,
            ..Default::default()
        };
        let mut l = Learner::new(cfg, vec![5]).unwrap();
        l.enable_autonomy(super::super::autonomy::AutonomyPolicy::default())
            .unwrap();
        l.manage_from_carrier().unwrap();
        l.enable_self_directed_control().unwrap();
        l.active.program.cells.push(super::super::vm::StateCell {
            initial: 7,
            update: Expr::bin(Op::Add, Expr::State(0), Expr::Lit(Value::Int(1))),
        });
        l.active.program.outputs[0] = Expr::State(0);
        l.active.state = vec![7];
        l.enable_causal_memory(super::super::causal_memory::CausalConfig::default())
            .unwrap();
        for id in 0..100 {
            let t = l.prepare(0).unwrap();
            l.commit(t, vec![5], id + 1, 1).unwrap();
        }
        assert!(!l.records[0].transition.restart);
        assert_eq!(validate_data(&l.data(), 1, 2), Err(Error::InvalidData));
        assert_eq!(
            replay(
                &l.active.program,
                &l.data(),
                &l.library,
                &mut Meter::new(100_000)
            ),
            Err(Error::InvalidData)
        );
        let before = l.active.state.clone();
        assert_eq!(l.construct(), Err(Error::NoEvidence));
        assert_eq!(l.active.state, before, "construction cannot invent a reset");
        assert!(l.shadow.is_none());
        // Only a real, host-observed episode boundary gives every existing
        // synthesis and recursive audit lane valid initial register history.
        l.record_external_restart(vec![5]).unwrap();
        for id in 100..108 {
            let t = l.prepare(0).unwrap();
            l.commit(t, vec![5], id + 1, 1).unwrap();
        }
        l.construct().unwrap();
        assert!(l.records[0].transition.restart);
        assert!(
            l.shadow.is_some(),
            "existing synthesis must still create a prospective trial"
        );
        for id in 108..153 {
            let t = l.prepare(0).unwrap();
            l.commit(t, vec![5], id + 1, 1).unwrap();
            let bytes = l.checkpoint().unwrap();
            l = Learner::from_checkpoint(&bytes).unwrap();
        }
        assert!(l.promotions.iter().any(|p| p.accepted));
        assert_eq!(
            l.active.state,
            vec![60],
            "unchanged VM registers retain their absolute physical history"
        );
        l.rollback().unwrap();
        assert_eq!(l.active.state, vec![60]);
        assert_eq!(l.factual_event_count(), 153);
        assert_eq!(l.causal_report().unwrap().factual_transitions, 153);
        let bytes = l.checkpoint().unwrap();
        assert_eq!(
            Learner::from_checkpoint(&bytes)
                .unwrap()
                .checkpoint()
                .unwrap(),
            bytes
        );
    }
}
