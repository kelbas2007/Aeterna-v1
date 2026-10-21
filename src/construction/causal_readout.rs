// Included in external_reward. Compare complete, actually executed policies,
// rather than interpreting probability calibration as a policy improvement.
const READOUT_PAIR_WINDOW: usize = 128;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ReadoutArm {
    tasks: u64,
    reward: f64,
    cost: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReadoutTask {
    namespace: i64,
    block: u64,
    experimental: bool,
    candidate: bool,
    cost: u64,
    steps: u64,
    first_reward: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReadoutPart {
    block: u64,
    candidate: bool,
    reward: f64,
    cost: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReadoutPair {
    baseline_reward: f64,
    candidate_reward: f64,
    baseline_cost: u64,
    candidate_cost: u64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ReadoutDomain {
    started: u64,
    trials_started: u64,
    exploited: u64,
    abandoned: u64,
    baseline: ReadoutArm,
    candidate: ReadoutArm,
    partial: Option<ReadoutPart>,
    pairs: Vec<ReadoutPair>,
    completed_pairs: u64,
    decisions: u64,
    active: bool,
    admissions: u64,
    rollbacks: u64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    settled: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalReadout {
    #[serde(
        default = "readout_revision_one",
        skip_serializing_if = "readout_is_revision_one"
    )]
    revision: u8,
    enabled_at: u64,
    frozen: bool,
    work: u64,
    domains: BTreeMap<i64, ReadoutDomain>,
    task: Option<ReadoutTask>,
}
fn readout_revision_one() -> u8 {
    1
}
fn readout_is_revision_one(value: &u8) -> bool {
    *value == 1
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReadoutDomainReport {
    pub exploit_tasks: u64,
    pub baseline_tasks: u64,
    pub candidate_tasks: u64,
    pub baseline_reward: f64,
    pub candidate_reward: f64,
    pub baseline_cost: u64,
    pub candidate_cost: u64,
    pub paired_tasks: u64,
    pub window_candidate_wins: usize,
    pub window_baseline_wins: usize,
    pub decisions: u64,
    pub consolidated_active: bool,
    pub admissions: u64,
    pub rollbacks: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReadoutReport {
    pub enabled_at: u64,
    pub frozen: bool,
    pub work: u64,
    pub task_open: bool,
    pub domains: BTreeMap<i64, CausalReadoutDomainReport>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReadoutAssignment {
    pub namespace: i64,
    pub block: u64,
    pub candidate: bool,
    pub experimental: bool,
}

fn readout_sign_tail(wins: usize, losses: usize) -> f64 {
    let n = wins + losses;
    let mut term = 2.0_f64.powi(-(n as i32));
    let mut tail = if wins == 0 { term } else { 0.0 };
    for k in 1..=n {
        term *= (n + 1 - k) as f64 / k as f64;
        if k >= wins {
            tail += term;
        }
    }
    tail.clamp(0.0, 1.0)
}
impl ReadoutDomain {
    fn wins(&self) -> (usize, usize) {
        (
            self.pairs
                .iter()
                .filter(|p| p.candidate_reward > p.baseline_reward)
                .count(),
            self.pairs
                .iter()
                .filter(|p| p.baseline_reward > p.candidate_reward)
                .count(),
        )
    }
    fn decide(&mut self, revision: u8) {
        self.decisions += 1;
        let (wins, losses) = self.wins();
        let alpha = 0.05 / (self.decisions as f64 * (self.decisions as f64 + 1.0));
        let baseline_cost = self
            .pairs
            .iter()
            .map(|p| p.baseline_cost as f64)
            .sum::<f64>();
        let candidate_cost = self
            .pairs
            .iter()
            .map(|p| p.candidate_cost as f64)
            .sum::<f64>();
        let affordable = candidate_cost <= 1.25 * baseline_cost;
        let previous = self.active;
        if affordable && wins > losses && readout_sign_tail(wins, losses) <= alpha {
            self.active = true;
            self.settled = revision >= 2;
        } else if !affordable || (losses > wins && readout_sign_tail(losses, wins) <= alpha) {
            self.active = false;
            self.settled = revision >= 2;
        }
        self.admissions += u64::from(self.active && !previous);
        self.rollbacks += u64::from(previous && !self.active);
    }
}
impl CausalReadout {
    fn new(events: u64) -> Self {
        Self {
            revision: 2,
            enabled_at: events,
            ..Self::default()
        }
    }
    fn active(&self, namespace: i64) -> bool {
        if !self.frozen {
            if let Some(task) = self.task.as_ref().filter(|t| t.namespace == namespace) {
                return task.candidate;
            }
        }
        self.domains.get(&namespace).is_some_and(|d| d.active)
    }
    fn begin(&mut self, namespace: i64, eligible: bool) -> Result<u64> {
        if self.frozen {
            return Ok(0);
        }
        if self.task.is_some() {
            return Err(Error::Busy);
        }
        if !eligible {
            return Ok(0);
        }
        if self.domains.len() == 8 && !self.domains.contains_key(&namespace) {
            return Err(Error::Capacity);
        }
        let d = self.domains.entry(namespace).or_default();
        let next = d.started.checked_add(1).ok_or(Error::Capacity)?;
        // An inconclusive comparison does not justify suppressing exploration.
        // Revision one is retained for exact replay of the first failed trial.
        let unresolved = if self.revision == 1 {
            d.decisions == 0
        } else {
            !d.settled
        };
        let experimental = unresolved || d.started % 8 < 2;
        let block = d.trials_started / 2;
        // Randomized AB/BA blocks; assigned before the first task action.
        // No feature, reward, class identity or future outcome enters this bit.
        let mut seed = fingerprint(&(self.enabled_at, namespace, block));
        seed = (seed ^ (seed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        seed = (seed ^ (seed >> 27)).wrapping_mul(0x94d049bb133111eb);
        let bit = (seed ^ (seed >> 31)) & 1 != 0;
        self.task = Some(ReadoutTask {
            namespace,
            block,
            experimental,
            candidate: if experimental {
                bit ^ (d.trials_started % 2 != 0)
            } else {
                d.active
            },
            cost: 0,
            steps: 0,
            first_reward: None,
        });
        d.started = next;
        if experimental {
            d.trials_started += 1;
        }
        self.work = self.work.saturating_add(128);
        Ok(128)
    }
    fn observe(&mut self, namespace: i64, outcome: bool, reward: f64, cost: u64) -> u64 {
        if self.frozen {
            return 0;
        }
        let Some(task) = self.task.as_mut().filter(|t| t.namespace == namespace) else {
            return 0;
        };
        task.cost = task.cost.saturating_add(cost);
        task.steps = task.steps.saturating_add(1);
        if outcome && task.first_reward.is_none() {
            task.first_reward = Some(reward.clamp(0.0, 1.0));
        }
        self.work = self.work.saturating_add(64);
        64
    }
    fn end(&mut self) -> u64 {
        if self.frozen {
            return 0;
        }
        let Some(task) = self.task.take() else {
            return 0;
        };
        let d = self.domains.get_mut(&task.namespace).unwrap();
        let mut work = 128;
        if task.steps == 0 {
            d.abandoned += 1;
        } else if !task.experimental {
            d.exploited += 1;
        } else {
            let reward = task.first_reward.unwrap_or(0.0);
            let arm = if task.candidate {
                &mut d.candidate
            } else {
                &mut d.baseline
            };
            arm.tasks += 1;
            arm.reward += reward;
            arm.cost = arm.cost.saturating_add(task.cost);
            let part = ReadoutPart {
                block: task.block,
                candidate: task.candidate,
                reward,
                cost: task.cost,
            };
            if let Some(old) = d
                .partial
                .take()
                .filter(|p| p.block == part.block && p.candidate != part.candidate)
            {
                let (baseline, candidate) = if part.candidate {
                    (old, part)
                } else {
                    (part, old)
                };
                d.pairs.push(ReadoutPair {
                    baseline_reward: baseline.reward,
                    candidate_reward: candidate.reward,
                    baseline_cost: baseline.cost,
                    candidate_cost: candidate.cost,
                });
                if d.pairs.len() > READOUT_PAIR_WINDOW {
                    d.pairs.remove(0);
                }
                d.completed_pairs += 1;
                work += 256;
                if d.completed_pairs % 64 == 0 {
                    d.decide(self.revision);
                    work += 8192;
                }
            } else {
                d.partial = Some(part);
            }
        }
        self.work = self.work.saturating_add(work);
        work
    }
    fn freeze(&mut self) {
        if let Some(task) = self.task.take() {
            self.domains.get_mut(&task.namespace).unwrap().abandoned += 1;
        }
        self.frozen = true;
    }
    fn report(&self) -> CausalReadoutReport {
        CausalReadoutReport {
            enabled_at: self.enabled_at,
            frozen: self.frozen,
            work: self.work,
            task_open: self.task.is_some(),
            domains: self
                .domains
                .iter()
                .map(|(&n, d)| {
                    let (wins, losses) = d.wins();
                    (
                        n,
                        CausalReadoutDomainReport {
                            exploit_tasks: d.exploited,
                            baseline_tasks: d.baseline.tasks,
                            candidate_tasks: d.candidate.tasks,
                            baseline_reward: d.baseline.reward,
                            candidate_reward: d.candidate.reward,
                            baseline_cost: d.baseline.cost,
                            candidate_cost: d.candidate.cost,
                            paired_tasks: d.completed_pairs,
                            window_candidate_wins: wins,
                            window_baseline_wins: losses,
                            decisions: d.decisions,
                            consolidated_active: d.active,
                            admissions: d.admissions,
                            rollbacks: d.rollbacks,
                        },
                    )
                })
                .collect(),
        }
    }
    fn valid(&self, events: u64) -> bool {
        let reward = |x: f64| x.is_finite() && (0.0..=1.0).contains(&x);
        (1..=2).contains(&self.revision)
            && self.enabled_at <= events
            && self.domains.len() <= 8
            && (!self.frozen || self.task.is_none())
            && self.domains.iter().all(|(&n, d)| {
                let open = u64::from(self.task.as_ref().is_some_and(|t| t.namespace == n));
                (0..=262143).contains(&n)
                    && d.baseline
                        .tasks
                        .checked_add(d.candidate.tasks)
                        .and_then(|v| v.checked_add(d.exploited))
                        .and_then(|v| v.checked_add(d.abandoned))
                        .and_then(|v| v.checked_add(open))
                        == Some(d.started)
                    && d.baseline.reward.is_finite()
                    && (0.0..=d.baseline.tasks as f64).contains(&d.baseline.reward)
                    && d.candidate.reward.is_finite()
                    && (0.0..=d.candidate.tasks as f64).contains(&d.candidate.reward)
                    && d.completed_pairs <= d.baseline.tasks.min(d.candidate.tasks)
                    && d.trials_started <= d.started
                    && d.baseline
                        .tasks
                        .checked_add(d.candidate.tasks)
                        .is_some_and(|n| n <= d.trials_started)
                    && d.pairs.len() == (d.completed_pairs as usize).min(READOUT_PAIR_WINDOW)
                    && d.decisions == d.completed_pairs / 64
                    && (!d.settled || (self.revision == 2 && d.decisions > 0))
                    && d.admissions <= d.decisions
                    && d.rollbacks <= d.admissions
                    && d.admissions.checked_sub(d.rollbacks) == Some(u64::from(d.active))
                    && d.partial
                        .as_ref()
                        .is_none_or(|p| p.block <= d.trials_started / 2 && reward(p.reward))
                    && d.pairs
                        .iter()
                        .all(|p| reward(p.baseline_reward) && reward(p.candidate_reward))
            })
            && self.task.as_ref().is_none_or(|t| {
                self.domains.get(&t.namespace).is_some_and(|d| {
                    d.started > 0
                        && (!t.experimental
                            || (d.trials_started > 0 && t.block == (d.trials_started - 1) / 2))
                }) && t.first_reward.is_none_or(reward)
                    && t.steps <= events - self.enabled_at
                    && (t.steps > 0 || (t.cost == 0 && t.first_reward.is_none()))
            })
            && self
                .domains
                .values()
                .try_fold(0_u64, |n, d| {
                    n.checked_add(d.baseline.tasks)?
                        .checked_add(d.candidate.tasks)?
                        .checked_add(d.exploited)
                })
                .is_some_and(|n| n <= events - self.enabled_at)
    }
}

#[cfg(test)]
mod causal_readout_tests {
    use super::*;
    #[test]
    fn inconclusive_trials_keep_exploring_and_legacy_revision_replays() {
        for revision in [1, 2] {
            let mut p = CausalReadout::new(0);
            p.revision = revision;
            for _ in 0..256 {
                p.begin(9, true).unwrap();
                p.observe(9, true, 1.0, 1);
                p.end();
            }
            assert_eq!(
                p.domains[&9].completed_pairs,
                if revision == 1 { 80 } else { 128 }
            );
            assert!(!p.domains[&9].settled);
            assert!(!p.domains[&9].active);
            assert!(p.valid(256));
        }
    }
    #[test]
    fn actual_zero_cost_actions_are_facts_and_empty_tasks_are_abandoned() {
        let mut p = CausalReadout::new(0);
        p.begin(9, true).unwrap();
        p.end();
        assert_eq!(p.domains[&9].abandoned, 1);
        p.begin(9, true).unwrap();
        p.observe(9, true, 1.0, 0);
        p.end();
        assert_eq!(
            p.domains[&9].baseline.tasks + p.domains[&9].candidate.tasks,
            1
        );
        assert!(p.valid(1));
        assert!(!p.valid(0));
    }
    #[test]
    fn assignment_precedes_feedback_and_retries_cannot_replace_first_failure() {
        let mut p = CausalReadout::new(100);
        p.begin(9, true).unwrap();
        let arm = p.active(9);
        p.observe(9, false, 0.0, 1);
        p.observe(9, true, 0.0, 1);
        p.observe(9, true, 1.0, 1);
        assert_eq!(p.active(9), arm);
        p.end();
        let first = if arm {
            &p.domains[&9].candidate
        } else {
            &p.domains[&9].baseline
        };
        assert_eq!(first.reward, 0.0);
        assert_eq!(first.cost, 3);
        p.begin(9, true).unwrap();
        assert_ne!(p.active(9), arm);
        p.observe(9, true, 1.0, 1);
        p.end();
        assert_eq!(p.domains[&9].completed_pairs, 1);
        assert!(p.valid(104));
    }
    #[test]
    fn real_policy_rewards_admit_and_then_revoke_a_readout_and_freeze_exactly() {
        let mut p = CausalReadout::new(0);
        for i in 0..1664 {
            p.begin(9, true).unwrap();
            let arm = p.active(9);
            p.observe(9, true, f64::from(if i < 128 { arm } else { !arm }), 1);
            p.end();
            if i == 127 {
                assert!(p.domains[&9].active);
                assert_eq!(p.domains[&9].admissions, 1);
            }
        }
        assert!(!p.domains[&9].active);
        assert_eq!(p.domains[&9].rollbacks, 1);
        p.freeze();
        let before = serde_json::to_string(&p).unwrap();
        p.begin(9, true).unwrap();
        p.observe(9, true, 1.0, 1);
        p.end();
        assert_eq!(serde_json::to_string(&p).unwrap(), before);
        let restored: CausalReadout = serde_json::from_str(&before).unwrap();
        assert!(restored.valid(1664));
    }
    #[test]
    fn equal_rewards_and_excessive_action_cost_cannot_admit_a_candidate() {
        for equal in [false, true] {
            let mut p = CausalReadout::new(0);
            for _ in 0..128 {
                p.begin(4, true).unwrap();
                let arm = p.active(4);
                p.observe(
                    4,
                    true,
                    if equal { 1.0 } else { f64::from(arm) },
                    if arm { 4 } else { 1 },
                );
                p.end();
            }
            assert!(!p.domains[&4].active);
            assert!(p.valid(128));
        }
        assert!((readout_sign_tail(4, 0) - 0.0625).abs() < 1e-12);
        assert!((readout_sign_tail(5, 0) - 0.03125).abs() < 1e-12);
    }
}

impl ExternalMemory {
    pub(crate) fn readout_assignment(&self) -> Option<CausalReadoutAssignment> {
        let t = self
            .consolidated_memory
            .as_ref()?
            .causal_readout
            .as_ref()?
            .task
            .as_ref()?;
        Some(CausalReadoutAssignment {
            namespace: t.namespace,
            block: t.block,
            candidate: t.candidate,
            experimental: t.experimental,
        })
    }
    pub(crate) fn enable_causal_readout(&mut self) -> Result<()> {
        if !self.pending.is_empty() {
            return Err(Error::Busy);
        }
        let model = self.consolidated_memory.as_mut().ok_or(Error::NoEvidence)?;
        if model.causal_readout.is_some() {
            return Err(Error::Busy);
        }
        model.causal_readout = Some(CausalReadout::new(self.events));
        Ok(())
    }
    pub(crate) fn readout_task_is_open(&self) -> bool {
        self.consolidated_memory
            .as_ref()
            .and_then(|m| m.causal_readout.as_ref())
            .is_some_and(|p| p.task.is_some())
    }
    pub(crate) fn begin_readout_task(&mut self, namespace: i64) -> Result<u64> {
        let Some(model) = self.consolidated_memory.as_mut() else {
            return Ok(0);
        };
        let eligible = model
            .domains
            .get(&namespace)
            .is_some_and(|d| d.outcomes.iter().filter(|c| c.examples > 0).count() >= 2);
        match model.causal_readout.as_mut() {
            Some(policy) => policy.begin(namespace, eligible),
            None => Ok(0),
        }
    }
    pub(crate) fn end_readout_task(&mut self) -> u64 {
        self.consolidated_memory
            .as_mut()
            .and_then(|m| m.causal_readout.as_mut())
            .map_or(0, CausalReadout::end)
    }
    pub(crate) fn freeze_readout_policy(&mut self) {
        if let Some(p) = self
            .consolidated_memory
            .as_mut()
            .and_then(|m| m.causal_readout.as_mut())
        {
            p.freeze();
        }
    }
    fn observe_readout_task(&mut self, pre: &[i64], action: usize, reward: f64, cost: u64) -> u64 {
        let Some(m) = self.consolidated_memory.as_mut() else {
            return 0;
        };
        let outcome = reward > 0.0
            || m.domains
                .get(&pre[0])
                .is_some_and(|d| d.outcomes[action].examples > 0);
        m.causal_readout
            .as_mut()
            .map_or(0, |p| p.observe(pre[0], outcome, reward, cost))
    }
}
