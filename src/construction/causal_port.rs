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
    pub fn factual_event_count(&self) -> usize {
        usize::try_from(self.next_id).unwrap_or(usize::MAX)
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
        if self.pending || self.poisoned || self.causal_memory.is_none() {
            return Err(Error::Busy);
        }
        // A canceled prospective trial keeps its original risk debt. Never
        // splice an in-flight evidence window into an apparently fresh trial.
        if self.shadow.is_some() {
            self.cancel_shadow();
        }
        let half = self.records.len() / 2;
        let remove = self
            .records
            .iter()
            .enumerate()
            .skip(half)
            .find(|(_, r)| r.transition.restart)
            .map_or(half, |(i, _)| i);
        if remove == 0 {
            return Err(Error::Capacity);
        }
        let memory = self.causal_memory.as_ref().ok_or(Error::NoEvidence)?;
        let mut seed = memory.seed_state.clone();
        let mut meter = Meter::new(10_000_000);
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
        let previous =
            if let (Some(program), Some((hash, prior))) = (&self.previous, &memory.previous_seed) {
                if *hash != program.fingerprint() {
                    return Err(Error::Stale);
                }
                Some((
                    *hash,
                    replay_causal_window(
                        program,
                        &self.records[..remove]
                            .iter()
                            .map(|r| r.transition.clone())
                            .collect::<Vec<_>>(),
                        &self.library,
                        &mut meter,
                        prior.clone(),
                    )?
                    .1,
                ))
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
        self.records.drain(..remove);
        self.construction_work = self.construction_work.saturating_add(meter.spent);
        Ok(())
    }
    pub(crate) fn carrier_causal_proposal(
        &self,
        offers: &[super::autonomy::ActionOffer],
        goal: Option<&[Option<i64>]>,
        budget: u64,
        horizon: usize,
    ) -> (Option<super::causal_memory::CausalProposal>, u64) {
        self.causal_memory.as_ref().map_or((None, 0), |m| {
            m.propose(
                &self.current,
                offers,
                goal,
                self.config.planning_depth.min(horizon),
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
        l.construct().unwrap();
        assert!(
            l.shadow.is_some(),
            "existing synthesis must still create a prospective trial"
        );
        for id in 100..145 {
            let t = l.prepare(0).unwrap();
            l.commit(t, vec![5], id + 1, 1).unwrap();
            let bytes = l.checkpoint().unwrap();
            l = Learner::from_checkpoint(&bytes).unwrap();
        }
        assert!(l.promotions.iter().any(|p| p.accepted));
        assert_eq!(
            l.active.state,
            vec![152],
            "unchanged VM registers retain their absolute physical history"
        );
        l.rollback().unwrap();
        assert_eq!(l.active.state, vec![152]);
        assert_eq!(l.factual_event_count(), 145);
        assert_eq!(l.causal_report().unwrap().factual_transitions, 145);
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
