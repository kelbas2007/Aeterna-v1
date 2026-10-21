// Included in scientific_runtime with unified_runtime. This coordinator owns
// no actuator and creates no labels: every row comes from a committed POST.
use crate::construction::search_policy::{
    AdmissionSuite, EvolutionConfig, Example as RExample, ProducerCase, SynthesisTask,
    ValidationCase,
};
use std::collections::{BTreeMap, BTreeSet};

const DEVELOPMENT_FACTS: usize = 512;
const DEVELOPMENT_EPISODE: usize = 256;

#[derive(Debug, Clone, USerialize, UDeserialize)]
struct DevelopmentFact {
    event: usize,
    namespace: i64,
    input: Vec<i64>,
    fields: [f32; 5],
    eligible: bool,
    reward: f64,
    outcome: i64,
}

#[derive(Debug, Clone, USerialize, UDeserialize)]
struct LiveEpisode {
    episode: u64,
    first_event: usize,
    last_event: usize,
    namespace: i64,
    candidate: bool,
    reward: f64,
    cost: u64,
    actions: usize,
    changed_readouts: usize,
}

#[derive(Debug, Clone, USerialize, UDeserialize)]
struct LiveRevisionTrial {
    receipt: usize,
    program: MExpr,
    incumbent: Option<MExpr>,
    after_episode: u64,
    required_domains: Vec<i64>,
    episodes: Vec<LiveEpisode>,
    // None until sufficient real future behavior exists. No caller verdict.
    qualified: Option<bool>,
    revoked: bool,
    interrupted: bool,
}

impl LiveRevisionTrial {
    fn verdict(&self) -> Option<bool> {
        if self.interrupted {
            return Some(false);
        }
        let arms = |candidate: bool, namespace: Option<i64>| {
            let rows: Vec<_> = self
                .episodes
                .iter()
                .filter(|r| r.candidate == candidate && namespace.is_none_or(|n| n == r.namespace))
                .collect();
            (
                rows.len(),
                rows.iter().map(|r| r.reward).sum::<f64>(),
                rows.iter().map(|r| r.cost).sum::<u64>(),
            )
        };
        let (n0, r0, c0) = arms(false, None);
        let (n1, r1, c1) = arms(true, None);
        let covered = self
            .required_domains
            .iter()
            .all(|&n| arms(false, Some(n)).0 >= 4 && arms(true, Some(n)).0 >= 4);
        if n0 < 12 || n1 < 12 || !covered {
            return (self.episodes.len() >= 64).then_some(false);
        }
        let non_regression = self.required_domains.iter().all(|&n| {
            let (a, x, _) = arms(false, Some(n));
            let (b, y, _) = arms(true, Some(n));
            y / b as f64 + 1e-9 >= x / a as f64
        });
        Some(
            r1 / n1 as f64 >= r0 / n0 as f64 + 0.05
                && non_regression
                && c1 as f64 / n1 as f64 <= 1.10 * c0 as f64 / n0 as f64
                && self
                    .episodes
                    .iter()
                    .any(|r| r.candidate && r.changed_readouts > 0),
        )
    }
    // Balanced randomized blocks within each observed interface. Assignment is
    // fixed before its first action, without access to the eventual outcome.
    fn assignment(&self, namespace: i64) -> bool {
        let count = self
            .episodes
            .iter()
            .filter(|r| r.namespace == namespace)
            .count();
        let bit = uword(
            self.after_episode
                ^ (namespace as u64).rotate_left(17)
                ^ (count as u64 / 2).wrapping_mul(0x9e3779b97f4a7c15),
        ) & 1
            != 0;
        bit ^ (count % 2 != 0)
    }
}

#[derive(Debug, USerialize, UDeserialize)]
struct OrganismDevelopment {
    recursive: bool,
    facts: Vec<DevelopmentFact>,
    pending: Vec<DevelopmentFact>,
    used_inputs: BTreeSet<u64>,
    domains: BTreeMap<i64, u64>,
    episodes: u64,
    next_search_event: usize,
    search_attempts: u64,
    search_pairs: BTreeMap<String, u8>,
    evaluator_attempts: BTreeMap<i64, u32>,
    search_errors: u64,
    last_search_error: Option<String>,
    aborted_episodes: u64,
    pending_cost: u64,
    pending_actions: usize,
    pending_reward: f64,
    pending_changed: usize,
    pending_first_event: usize,
    pending_arm: Option<bool>,
    trials: Vec<LiveRevisionTrial>,
    readout: Option<MExpr>,
    seen_rollbacks: u32,
}

#[derive(Debug, Clone, USerialize, UDeserialize)]
pub struct OrganismDevelopmentReport {
    pub episodes: u64,
    pub namespaces: Vec<i64>,
    pub retained_factual_examples: usize,
    pub recursive_enabled: bool,
    pub recursive_attempts: u64,
    pub recursive_generations: u32,
    pub recursive_work: u64,
    pub recursive_errors: u64,
    pub last_recursive_error: Option<String>,
    pub active_search_policy: u64,
    pub evaluator_searches: u64,
    pub evaluator_search_policy: Option<u64>,
    pub evaluator_attempts_per_namespace: BTreeMap<i64, u32>,
    pub recursive_attempts_per_pair: BTreeMap<String, u8>,
    pub behavioral_trials: usize,
    pub behavioral_admissions: usize,
    pub behavioral_rejections: usize,
    pub behavioral_episodes: usize,
    pub candidate_actions: usize,
    pub candidate_changed_readouts: usize,
    pub admitted_readout: Option<String>,
    pub abandoned_episodes: u64,
}

impl OrganismDevelopment {
    fn new(recursive: bool) -> Self {
        Self {
            recursive,
            facts: Vec::new(),
            pending: Vec::new(),
            used_inputs: BTreeSet::new(),
            domains: BTreeMap::new(),
            episodes: 0,
            next_search_event: 0,
            search_attempts: 0,
            search_pairs: BTreeMap::new(),
            evaluator_attempts: BTreeMap::new(),
            search_errors: 0,
            last_search_error: None,
            aborted_episodes: 0,
            pending_cost: 0,
            pending_actions: 0,
            pending_reward: 0.0,
            pending_changed: 0,
            pending_first_event: 0,
            pending_arm: None,
            trials: Vec::new(),
            readout: None,
            seen_rollbacks: 0,
        }
    }
    fn open_trial(&self) -> Option<&LiveRevisionTrial> {
        self.trials
            .last()
            .filter(|t| t.qualified.is_none() && !t.revoked)
    }
    fn observe(
        &mut self,
        event: usize,
        pre: &[i64],
        action: usize,
        fields: [f32; 5],
        reward: f64,
        cost: u64,
        eligible: bool,
    ) {
        if self.pending_actions == 0 {
            self.pending_first_event = event;
        }
        self.pending_actions += 1;
        self.pending_cost = self.pending_cost.saturating_add(cost);
        self.pending_reward += reward;
        // Namespace routes experience, never supplying a hidden task ID to RSP2.
        // Missing sensory values remain distinguishable from measured zero.
        let mut input: Vec<_> = pre[2..]
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                if pre[1] & (1_i64 << i) == 0 {
                    -1_000_000
                } else {
                    v.clamp(-999_999, 999_999)
                }
            })
            .collect();
        input.push(action as i64);
        if self.pending.len() == DEVELOPMENT_EPISODE {
            self.pending.remove(0);
        }
        self.pending.push(DevelopmentFact {
            event,
            namespace: pre[0],
            input,
            fields,
            eligible,
            reward,
            outcome: 0,
        });
    }
    fn reset_episode(&mut self) {
        self.pending.clear();
        self.pending_cost = 0;
        self.pending_actions = 0;
        self.pending_reward = 0.0;
        self.pending_changed = 0;
        self.pending_first_event = 0;
        self.pending_arm = None;
    }
    fn abandon(&mut self) {
        if self.pending_actions > 0 {
            self.aborted_episodes += 1;
            // An interrupted canary cannot select away inconvenient outcomes.
            if self.pending_arm.is_some() {
                if let Some(trial) = self
                    .trials
                    .last_mut()
                    .filter(|t| t.qualified.is_none() && !t.revoked)
                {
                    trial.interrupted = true;
                    trial.qualified = Some(false);
                }
            }
        }
        self.reset_episode();
    }
    fn freeze(&mut self) {
        // Freezing is not an environment terminal observation. Preserve the
        // committed world facts, but do not label an unfinished goal a failure.
        self.abandon();
        if let Some(trial) = self
            .trials
            .last_mut()
            .filter(|t| t.qualified.is_none() && !t.revoked)
        {
            trial.interrupted = true;
            trial.qualified = Some(false);
        }
    }
    fn adjust(&self, revision: &MetaRevision, fields: &mut [f32; 5], live: bool) -> bool {
        if revision.readout_ablated {
            return false;
        }
        let before = fields[0];
        let code = if let Some(trial) = self.open_trial().filter(|_| live) {
            if self.pending_arm == Some(true) {
                Some(&trial.program)
            } else {
                trial.incumbent.as_ref()
            }
        } else {
            self.readout.as_ref()
        };
        // Revoked predictive evidence can never keep controlling the organism.
        if self.seen_rollbacks != revision.rollbacks {
            return false;
        }
        if let Some(code) = code {
            if let Some(value) = MetaRevision::score(Some(code), &MetaRevision::features(*fields)) {
                fields[0] = value as f32 / 100.0;
            }
        }
        (fields[0] - before).abs() > 0.00001
    }
    fn curriculum(&self) -> Option<(Vec<SynthesisTask>, AdmissionSuite, BTreeSet<u64>, String)> {
        // Distinct measured inputs in every role; an exam input never returns
        // to proposal training. Require two actually experienced interfaces.
        let mut pools = BTreeMap::<i64, Vec<&DevelopmentFact>>::new();
        let mut unique = BTreeSet::new();
        for fact in &self.facts {
            let key = uhash(&serde_json::to_vec(&fact.input).ok()?);
            if !self.used_inputs.contains(&key) && unique.insert(key) {
                pools.entry(fact.namespace).or_default().push(fact);
            }
        }
        let enough: Vec<_> = pools.iter().filter(|(_, r)| r.len() >= 58).collect();
        let mut pairs = Vec::new();
        for (i, &a) in enough.iter().enumerate() {
            for &b in &enough[i + 1..] {
                let name = format!("{}:{}", a.0, b.0);
                if self.search_pairs.get(&name).copied().unwrap_or(0) < 2 {
                    let recent = a.1.last()?.event.max(b.1.last()?.event);
                    pairs.push((recent, a, b, name));
                }
            }
        }
        pairs.sort_by_key(|p| std::cmp::Reverse(p.0));
        let (_, a, b, pair) = pairs.into_iter().next()?;
        let make = |rows: &[&DevelopmentFact]| SynthesisTask {
            library: MLibrary::default(),
            examples: rows
                .iter()
                .map(|r| RExample {
                    input: r.input.clone(),
                    output: r.outcome,
                })
                .collect(),
        };
        let case = |rows: &[&DevelopmentFact]| ValidationCase {
            support: make(&rows[..3]),
            probes: make(&rows[3..5]),
        };
        let mut consumed = BTreeSet::new();
        for row in a.1[..58].iter().chain(&b.1[..58]) {
            consumed.insert(uhash(&serde_json::to_vec(&row.input).ok()?));
        }
        let training = vec![make(&a.1[..4]), make(&b.1[..4])];
        let fresh = a.1[4..44].chunks(5).map(case).collect();
        let transfer = b.1[4..44].chunks(5).map(case).collect();
        let producers = [a.1, b.1]
            .into_iter()
            .map(|rows| ProducerCase {
                training: vec![make(&rows[44..48])],
                examination: rows[48..58].chunks(5).map(case).collect(),
            })
            .collect();
        Some((
            training,
            AdmissionSuite {
                fresh,
                transfer,
                producers,
            },
            consumed,
            pair,
        ))
    }
    fn evolve(&mut self, learner: &mut ULearner) -> UResult<u64> {
        let event = learner.factual_event_count();
        if !self.recursive
            || event < self.next_search_event
            || self.search_attempts >= 8
            || self.used_inputs.len() >= 4096
        {
            return Ok(0);
        }
        // Insufficient / uniformly unsuccessful experience is an explicit wait,
        // not a fabricated curriculum or a successful self-improvement count.
        if !self.facts.iter().any(|r| r.outcome >= 90) || !self.facts.iter().any(|r| r.outcome == 0)
        {
            return Ok(0);
        }
        let Some((training, gate, consumed, pair)) = self.curriculum() else {
            return Ok(0);
        };
        if self.used_inputs.len() + consumed.len() > 4096 {
            return Ok(0);
        }
        self.next_search_event = event.saturating_add(256);
        self.used_inputs.extend(consumed);
        self.search_attempts += 1;
        *self.search_pairs.entry(pair).or_default() += 1;
        let before = learner.report().construction_work;
        let result = learner.improve_recursive_search(&training, &gate, uword(self.episodes));
        if let Err(error) = result {
            self.search_errors += 1;
            self.last_search_error = Some(format!("{error:?}"));
            if !matches!(
                error,
                UError::Budget | UError::Capacity | UError::NoEvidence | UError::Busy
            ) {
                return Err(error);
            }
        } else {
            self.last_search_error = None;
        }
        Ok(learner.report().construction_work.saturating_sub(before))
    }
    fn validate(&self, learner: &ULearner, revision: &MetaRevision, frozen: bool) -> UResult<()> {
        let valid = |r: &DevelopmentFact| {
            r.event > 0
                && r.event <= learner.factual_event_count()
                && (0..=262143).contains(&r.namespace)
                && r.input.len() == 31
                && r.input.iter().all(|v| v.abs_diff(0) <= 1_000_000)
                && (0..16).contains(&r.input[30])
                && r.fields.iter().all(|v| v.is_finite())
                && r.reward.is_finite()
                && (-1.0..=1.0).contains(&r.reward)
                && (0..=100).contains(&r.outcome)
        };
        let rsp = learner.recursive_search().ok_or(UError::InvalidData)?;
        let known_policy = |id| {
            id == MSearchPolicy::default().fingerprint()
                || rsp
                    .audits()
                    .iter()
                    .any(|a| a.accepted && a.candidate.fingerprint() == id)
        };
        if self.facts.len() > DEVELOPMENT_FACTS
            || self.pending.len() > DEVELOPMENT_EPISODE
            || self.used_inputs.len() > 4096
            || self.domains.len() > 8
            || self.search_attempts > 8
            || self.search_errors > self.search_attempts
            || self.search_pairs.values().any(|n| *n > 2)
            || self
                .search_pairs
                .values()
                .map(|n| u64::from(*n))
                .sum::<u64>()
                != self.search_attempts
            || self.evaluator_attempts.values().any(|n| *n > 3)
            || self.evaluator_attempts.values().sum::<u32>() != revision.attempts
            || self
                .evaluator_attempts
                .keys()
                .any(|n| !self.domains.contains_key(n))
            || self.trials.len() > 3
            || self.facts.iter().chain(&self.pending).any(|r| !valid(r))
            || self.pending.len() > self.pending_actions
            || !self.pending_reward.is_finite()
            || self.pending_reward.abs() > self.pending_actions as f64
            || self.pending_changed > self.pending_actions
            || (frozen
                && (self.pending_actions != 0
                    || !self.pending.is_empty()
                    || self.pending_arm.is_some()
                    || self.open_trial().is_some()))
            || self.episodes != revision.episodes
            || self.seen_rollbacks != revision.rollbacks
            || learner.recursive_search().is_none()
            || !learner.external_reward_enabled()
            || !revision.behavior_guard_enabled
            || (!self.recursive && self.search_attempts > 0)
            || self.search_attempts != rsp.audits().len() as u64 + self.search_errors
            || revision
                .shared_search_policy
                .is_some_and(|id| !known_policy(id))
        {
            return Err(UError::InvalidData);
        }
        let mut readout = None;
        for (i, trial) in self.trials.iter().enumerate() {
            let receipt = revision
                .receipts
                .get(trial.receipt)
                .ok_or(UError::InvalidData)?;
            if receipt.program != trial.program
                || trial.incumbent != readout
                || trial.after_episode < receipt.future.iter().map(|r| r.episode).max().unwrap_or(0)
                || trial.required_domains.is_empty()
                || trial.required_domains.len() > 8
                || trial.episodes.len() > 64
                || trial.qualified != trial.verdict()
                || (trial.interrupted && self.aborted_episodes == 0 && !frozen)
                || (trial.revoked && !receipt.revoked)
                || (trial.qualified.is_none() && !trial.revoked && i + 1 != self.trials.len())
            {
                return Err(UError::InvalidData);
            }
            let mut replay = trial.clone();
            replay.episodes.clear();
            for row in &trial.episodes {
                if row.episode <= trial.after_episode
                    || row.episode > self.episodes
                    || row.first_event == 0
                    || row.last_event < row.first_event
                    || row.last_event - row.first_event + 1 != row.actions
                    || row.last_event > learner.factual_event_count()
                    || row.actions == 0
                    || row.changed_readouts > row.actions
                    || !row.reward.is_finite()
                    || !(-1.0..=1.0).contains(&row.reward)
                    || row.candidate != replay.assignment(row.namespace)
                    || replay.episodes.last().is_some_and(|p| {
                        p.episode >= row.episode || p.last_event >= row.first_event
                    })
                {
                    return Err(UError::InvalidData);
                }
                replay.episodes.push(row.clone());
            }
            if trial.qualified == Some(true) {
                readout = Some(trial.program.clone());
            }
        }
        if self.seen_rollbacks > 0 {
            readout = None;
        }
        if self.readout != readout {
            return Err(UError::InvalidData);
        }
        Ok(())
    }
}

impl UnifiedConstruction {
    fn finish_development_episode(&mut self) -> UResult<()> {
        if self.freeze_controller {
            return Ok(());
        }
        let Some(dev) = self.development.as_mut() else {
            return Ok(());
        };
        if dev.pending_actions == 0 {
            return Ok(());
        }
        let revision = self.self_revision.as_mut().ok_or(UError::InvalidData)?;
        dev.episodes += 1;
        let namespace = dev.pending.first().ok_or(UError::InvalidData)?.namespace;
        *dev.domains.entry(namespace).or_default() += 1;
        if let Some(arm) = dev.pending_arm {
            let row = LiveEpisode {
                episode: dev.episodes,
                first_event: dev.pending_first_event,
                last_event: self.current.learner.factual_event_count(),
                namespace,
                candidate: arm,
                reward: dev.pending_reward.clamp(-1.0, 1.0),
                cost: dev.pending_cost,
                actions: dev.pending_actions,
                changed_readouts: dev.pending_changed,
            };
            if let Some(trial) = dev
                .trials
                .last_mut()
                .filter(|t| t.qualified.is_none() && !t.revoked)
            {
                trial.episodes.push(row);
                trial.qualified = trial.verdict();
                if trial.qualified == Some(true) {
                    dev.readout = Some(trial.program.clone());
                }
            }
        }
        let mut discounted = 0.0;
        for row in dev.pending.iter_mut().rev() {
            discounted = (row.reward + 0.95 * discounted).clamp(-1.0, 1.0);
            row.outcome = (100.0 * discounted.max(0.0)).round() as i64;
        }
        let completed = std::mem::take(&mut dev.pending);
        dev.facts.extend(completed.iter().cloned());
        while dev.facts.len() > DEVELOPMENT_FACTS {
            let mut counts = BTreeMap::new();
            for row in &dev.facts {
                *counts.entry(row.namespace).or_insert(0usize) += 1;
            }
            let namespace = *counts.iter().max_by_key(|(_, n)| *n).unwrap().0;
            let index = dev
                .facts
                .iter()
                .position(|r| r.namespace == namespace)
                .unwrap();
            dev.facts.remove(index);
        }
        self.computation_work = self
            .computation_work
            .checked_add(dev.evolve(&mut self.current.learner)?)
            .ok_or(UError::Capacity)?;
        let policy = self
            .current
            .learner
            .recursive_search()
            .ok_or(UError::InvalidData)?
            .policy()
            .clone();
        let before_attempts = revision.attempts;
        let allow_proposal = dev.evaluator_attempts.get(&namespace).copied().unwrap_or(0) < 3;
        let work =
            revision.observe_reward_episode(dev.episodes, &completed, &policy, allow_proposal);
        *dev.evaluator_attempts.entry(namespace).or_default() +=
            revision.attempts - before_attempts;
        self.computation_work = self
            .computation_work
            .checked_add(work)
            .ok_or(UError::Capacity)?;
        if dev.seen_rollbacks != revision.rollbacks {
            dev.readout = None;
            for trial in &mut dev.trials {
                if revision.receipts[trial.receipt].revoked {
                    trial.revoked = true;
                }
            }
            dev.seen_rollbacks = revision.rollbacks;
        }
        if dev.seen_rollbacks == 0 && dev.open_trial().is_none() && dev.trials.len() < 3 {
            if let Some((index, receipt)) = revision
                .receipts
                .iter()
                .enumerate()
                .last()
                .filter(|(_, r)| !r.revoked)
            {
                if !dev.trials.iter().any(|t| t.program == receipt.program)
                    && dev.readout.as_ref() != Some(&receipt.program)
                {
                    dev.trials.push(LiveRevisionTrial {
                        receipt: index,
                        program: receipt.program.clone(),
                        incumbent: dev.readout.clone(),
                        after_episode: dev.episodes,
                        required_domains: dev.domains.keys().copied().collect(),
                        episodes: Vec::new(),
                        qualified: None,
                        revoked: false,
                        interrupted: false,
                    });
                }
            }
        }
        dev.reset_episode();
        Ok(())
    }
}

impl MetaRevision {
    fn observe_reward_episode(
        &mut self,
        episode: u64,
        rows: &[DevelopmentFact],
        policy: &MSearchPolicy,
        allow_proposal: bool,
    ) -> u64 {
        self.episodes = episode;
        self.rows
            .extend(rows.iter().filter(|r| r.eligible).map(|r| MetaWitness {
                episode,
                features: Self::features(r.fields),
                outcome: r.outcome,
            }));
        if self.rows.len() > 128 {
            self.rows.drain(..self.rows.len() - 128);
        }
        let before = self.work;
        self.revoke_on_new_contradictory_episodes();
        if self.shadow.is_some() {
            self.prospective_gate();
        } else if allow_proposal {
            self.propose_with_policy(Some(policy));
        }
        self.work.saturating_sub(before)
    }
}

impl ScientificRuntime {
    /// Assign a readout policy before any action in an observed external task.
    pub fn observe_external_sensor_task_start(&mut self,frame:&crate::construction::sensors::SensorFrame)->UResult<()> {
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if !u.freeze_controller && u.current.learner.carrier_readout_task_open() { return Err(UError::Busy); }
        self.observe_external_sensor_restart(frame)?;
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if !u.freeze_controller {
            let work=u.current.learner.carrier_begin_readout_task()?;
            u.computation_work=u.computation_work.saturating_add(work);
        }
        Ok(())
    }
    pub fn finish_external_sensor_task(&mut self)->UResult<()> {
        self.finish_external_episode()?;
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if !u.freeze_controller {
            let work=u.current.learner.carrier_end_readout_task();
            u.computation_work=u.computation_work.saturating_add(work);
        }
        Ok(())
    }
    pub fn enable_constructing_causal_readout(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.as_ref().is_none_or(|d|d.pending_actions!=0) { return Err(UError::Blocked); }
        u.current.learner.carrier_enable_causal_readout()?;Ok(())
    }
    pub fn enable_constructing_partial_posterior(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.as_ref().is_none_or(|d|d.pending_actions!=0) { return Err(UError::Blocked); }
        u.current.learner.carrier_enable_partial_posterior()?;Ok(())
    }
    /// Compress only existing factual successes into the same learner's model.
    /// Seeding work is charged without inventing external observations/actions.
    pub fn enable_constructing_consolidated_memory(&mut self) -> UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.as_ref().is_none_or(|d|d.pending_actions!=0) { return Err(UError::Blocked); }
        let work=u.current.learner.carrier_enable_consolidated_memory()?;
        u.computation_work=u.computation_work.saturating_add(work);
        Ok(())
    }
    /// Continue the same life with coverage-aware selection in its existing
    /// bounded factual memory. No second learner, extra prototypes or actor.
    pub fn enable_constructing_coverage_memory(&mut self) -> UResult<()> {
        let u = self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.as_ref().is_none_or(|d| d.pending_actions != 0) {
            return Err(UError::Blocked);
        }
        u.current.learner.carrier_enable_coverage_memory()
    }
    /// Opt in within the same existing life. A diagnostic control preserves
    /// acquisition/evidence but bypasses conditional programs in action values.
    pub fn enable_constructing_first_rule_readouts(&mut self, fixed_control: bool) -> UResult<()> {
        let u = self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.as_ref().is_none_or(|d| d.pending_actions != 0) {
            return Err(UError::Blocked);
        }
        u.current.learner.carrier_enable_first_rule_readouts(fixed_control)
    }
    /// Context is an observed sensory interface, never a hidden task label.
    /// Keep one U1 and condition its five learned utility connections on it.
    pub fn enable_constructing_contextual_control(&mut self) -> UResult<()> {
        let u = self.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.as_ref().is_none_or(|d| d.pending_actions != 0) || u.current.learner.report().poisoned
            || u.current.learner.external_reward_report().is_none_or(|r| r.pending_actions != 0) {
            return Err(UError::Busy);
        }
        if !self.organism.enable_contextual_meta_projection() { return Err(UError::Busy); }
        Ok(())
    }
    /// Enable acquisition in this existing living learner, including a loaded
    /// historical checkpoint. No state reset, host labels or supplied program.
    pub fn enable_constructing_acquired_sensory_rules(&mut self) -> UResult<()> {
        let u = self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.as_ref().is_none_or(|d| d.pending_actions != 0) {
            return Err(UError::Blocked);
        }
        u.current.learner.carrier_acquire_sensory_rules()
    }
    /// One owner, one factual journal and one U1. RSP2 admission is unchanged;
    /// evaluator promotion additionally needs real future randomized canaries.
    pub fn enable_constructing_organism_development(&mut self, recursive: bool) -> UResult<()> {
        let u = self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller
            || u.current.learner.factual_event_count() != 0
            || u.development.is_some()
            || u.self_revision.is_some()
            || u.current.learner.recursive_search().is_some()
            || !u.current.learner.external_reward_enabled()
        {
            return Err(UError::Busy);
        }
        u.current.learner.enable_recursive_search(EvolutionConfig {
            object: MSearchLimits {
                work: 6_000,
                evaluations: 32,
                max_nodes: 7,
                frontier: 64,
                edits_per_expansion: 8,
            },
            meta: MSearchLimits {
                work: 600_000,
                evaluations: 16,
                max_nodes: 13,
                frontier: 64,
                edits_per_expansion: 8,
            },
            max_lifetime_work: 40_000_000,
        })?;
        u.current.learner.carrier_use_shared_development()?;
        u.self_revision = Some(MetaRevision {
            behavior_guard_enabled: true,
            ..Default::default()
        });
        u.development = Some(OrganismDevelopment::new(recursive));
        Ok(())
    }
    pub fn organism_development_report(&self) -> Option<OrganismDevelopmentReport> {
        let u = self.constructing.as_ref()?;
        let dev = u.development.as_ref()?;
        let rsp = u.current.learner.recursive_search()?;
        let revision = u.self_revision.as_ref()?;
        Some(OrganismDevelopmentReport {
            episodes: dev.episodes,
            namespaces: dev.domains.keys().copied().collect(),
            retained_factual_examples: dev.facts.len(),
            recursive_enabled: dev.recursive,
            recursive_attempts: dev.search_attempts,
            recursive_generations: rsp.generation(),
            recursive_work: rsp.work(),
            recursive_errors: dev.search_errors,
            last_recursive_error: dev.last_search_error.clone(),
            active_search_policy: rsp.policy().fingerprint(),
            evaluator_searches: revision.shared_searches,
            evaluator_search_policy: revision.shared_search_policy,
            evaluator_attempts_per_namespace: dev.evaluator_attempts.clone(),
            recursive_attempts_per_pair: dev.search_pairs.clone(),
            behavioral_trials: dev.trials.len(),
            behavioral_admissions: dev
                .trials
                .iter()
                .filter(|t| t.qualified == Some(true) && !t.revoked)
                .count(),
            behavioral_rejections: dev
                .trials
                .iter()
                .filter(|t| t.qualified == Some(false) || t.revoked)
                .count(),
            behavioral_episodes: dev.trials.iter().map(|t| t.episodes.len()).sum(),
            candidate_actions: dev
                .trials
                .iter()
                .flat_map(|t| &t.episodes)
                .filter(|e| e.candidate)
                .map(|e| e.actions)
                .sum(),
            candidate_changed_readouts: dev
                .trials
                .iter()
                .flat_map(|t| &t.episodes)
                .filter(|e| e.candidate)
                .map(|e| e.changed_readouts)
                .sum(),
            admitted_readout: dev.readout.as_ref().map(|p| format!("{p:?}")),
            abandoned_episodes: dev.aborted_episodes,
        })
    }
}

#[cfg(test)]
mod organism_development_tests {
    use super::*;
    fn learned_revision() -> MetaRevision {
        let mut revision = MetaRevision {
            behavior_guard_enabled: true,
            ..Default::default()
        };
        for episode in 1..=16 {
            let success = episode % 2 == 0;
            let rows: Vec<_> = (0..4)
                .map(|i| DevelopmentFact {
                    event: (episode * 4 + i) as usize,
                    namespace: 4,
                    input: vec![0; 31],
                    fields: [0.0, if success { 1.0 } else { 0.0 }, 0.0, 0.0, 0.5],
                    eligible: true,
                    reward: if success { 1.0 } else { 0.0 },
                    outcome: if success { 100 } else { 0 },
                })
                .collect();
            revision.observe_reward_episode(episode, &rows, &MSearchPolicy::default(), true);
        }
        assert!(
            revision.active.is_some(),
            "actual VM search must acquire the fixture evaluator"
        );
        assert!(revision.shared_searches > 0);
        revision.validate().unwrap();
        revision
    }
    fn trial(revision: &MetaRevision) -> LiveRevisionTrial {
        LiveRevisionTrial {
            receipt: 0,
            program: revision.receipts[0].program.clone(),
            incumbent: None,
            after_episode: 16,
            required_domains: vec![4, 9],
            episodes: Vec::new(),
            qualified: None,
            revoked: false,
            interrupted: false,
        }
    }
    fn add(trial: &mut LiveRevisionTrial, advantage: bool, extra_cost: bool) {
        let index = trial.episodes.len();
        let namespace = if index % 2 == 0 { 4 } else { 9 };
        let candidate = trial.assignment(namespace);
        trial.episodes.push(LiveEpisode {
            episode: 17 + index as u64,
            first_event: index + 100,
            last_event: index + 100,
            namespace,
            candidate,
            reward: if candidate && advantage { 1.0 } else { 0.0 },
            cost: if candidate && extra_cost { 2 } else { 1 },
            actions: 1,
            changed_readouts: usize::from(candidate),
        });
        trial.qualified = trial.verdict();
    }
    #[test]
    fn predictive_admission_is_shadow_until_real_behavior_and_all_contexts_pass() {
        let revision = learned_revision();
        let mut dev = OrganismDevelopment::new(true);
        let mut fields = [0.0, 1.0, 0.0, 0.0, 0.5];
        assert!(!dev.adjust(&revision, &mut fields, true));
        assert_eq!(
            fields[0], 0.0,
            "predictive success alone must not deploy a scorer"
        );
        dev.trials.push(trial(&revision));
        dev.pending_arm = Some(true);
        assert!(dev.adjust(&revision, &mut fields, true));
        assert!(
            fields[0] >= 0.9,
            "registered canary must execute the learned scorer"
        );
        assert!(
            !dev.adjust(&revision, &mut fields, true),
            "an unchanged readout is not an intervention"
        );
        for _ in 0..23 {
            add(dev.trials.last_mut().unwrap(), true, false);
        }
        assert_eq!(dev.trials[0].qualified, None);
        add(&mut dev.trials[0], true, false);
        assert_eq!(dev.trials[0].qualified, Some(true));
        dev.readout = Some(dev.trials[0].program.clone());
        fields[0] = 0.0;
        dev.adjust(&revision, &mut fields, false);
        assert!(
            fields[0] >= 0.9,
            "qualified scorer also controls frozen inference"
        );
    }
    #[test]
    fn non_improvement_extra_cost_and_interrupted_canaries_cannot_be_promoted() {
        let revision = learned_revision();
        for (advantage, extra_cost) in [(false, false), (true, true)] {
            let mut t = trial(&revision);
            for _ in 0..24 {
                add(&mut t, advantage, extra_cost);
            }
            assert_eq!(t.qualified, Some(false));
        }
        let mut dev = OrganismDevelopment::new(true);
        dev.trials.push(trial(&revision));
        dev.pending_arm = Some(true);
        dev.pending_actions = 1;
        dev.abandon();
        assert_eq!(dev.trials[0].qualified, Some(false));
        assert!(dev.open_trial().is_none());
        assert!(dev.facts.is_empty());
    }
    #[test]
    fn freeze_closes_a_ready_canary_without_inventing_a_completed_episode() {
        let revision = learned_revision();
        let mut dev = OrganismDevelopment::new(true);
        dev.trials.push(trial(&revision));
        assert!(dev.open_trial().is_some());
        dev.freeze();
        assert!(dev.open_trial().is_none());
        assert_eq!(dev.trials[0].qualified, Some(false));
        assert_eq!(dev.episodes, 0);
        assert_eq!(dev.aborted_episodes, 0);
        assert!(dev.facts.is_empty());
    }
}
