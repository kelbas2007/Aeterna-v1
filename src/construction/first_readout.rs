// Included in external_reward. These are immutable forecasts, never extra
// factual examples, labels, actuators, or an independently loaded learner.
const READOUT_PENDING: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuleFirstReadout {
    input: u64,
    namespace: i64,
    before_event: u64,
    incumbent: Option<u64>,
    candidate: Option<u64>,
    fixed: [f64; 16],
    old: [f64; 16],
    proposed: Option<[f64; 16]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstReadoutWindow {
    pub namespace: i64,
    pub program: u64,
    pub closed_at: u64,
    pub examples: usize,
    pub fixed_correct: usize,
    pub active_correct: usize,
    pub rolled_back: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FirstReadouts {
    fixed_readout_control: bool,
    enabled_at: u64,
    // Includes observed unsuccessful inputs and pre-existing retained inputs.
    // Eviction must never turn a later retry into a new first forecast.
    seen: std::collections::BTreeSet<u64>,
    pending: Vec<RuleFirstReadout>,
    captured: u64,
    resolved: u64,
    evicted: u64,
    exhausted: bool,
    windows: Vec<FirstReadoutWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstReadoutReport {
    pub fixed_readout_control: bool,
    pub enabled_at: u64,
    pub distinct_inputs: usize,
    pub pending: usize,
    pub captured: u64,
    pub resolved: u64,
    pub evicted: u64,
    pub exhausted: bool,
    pub monitoring_windows: Vec<FirstReadoutWindow>,
}

impl FirstReadouts {
    fn report(&self) -> FirstReadoutReport {
        FirstReadoutReport {
            fixed_readout_control: self.fixed_readout_control,
            enabled_at: self.enabled_at,
            distinct_inputs: self.seen.len(),
            pending: self.pending.len(),
            captured: self.captured,
            resolved: self.resolved,
            evicted: self.evicted,
            exhausted: self.exhausted,
            monitoring_windows: self.windows.clone(),
        }
    }
    fn validate(&self, events: u64) -> bool {
        let distribution = |p: &[f64; 16]| {
            let sum = p.iter().sum::<f64>();
            p.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                // rule_normalize deliberately retains sub-probability mass
                // when very low coverage falls below its numerical floor.
                && sum <= 1.0 + 1e-9
        };
        let mut keys = std::collections::BTreeSet::new();
        self.enabled_at <= events
            && self.seen.len() <= RULE_KEYS
            && self.pending.len() <= READOUT_PENDING
            && self.captured == self.resolved + self.evicted + self.pending.len() as u64
            && self.captured <= self.seen.len() as u64
            && self.pending.iter().all(|p| {
                self.seen.contains(&p.input)
                    && keys.insert(p.input)
                    && (0..=262143).contains(&p.namespace)
                    && p.before_event > self.enabled_at
                    && p.before_event <= events
                    && distribution(&p.fixed)
                    && distribution(&p.old)
                    && p.proposed.as_ref().is_none_or(distribution)
                    && p.proposed.is_some() == p.candidate.is_some()
            })
            && self.windows.len() <= 128
            && self.windows.iter().all(|w| {
                (0..=262143).contains(&w.namespace)
                    && w.closed_at > self.enabled_at
                    && w.closed_at <= events
                    && w.examples == RULE_EXAM
                    && w.fixed_correct <= w.examples
                    && w.active_correct <= w.examples
                    && w.rolled_back == (w.active_correct + 3 < w.fixed_correct)
            })
    }
}

impl AcquiredRules {
    fn enable_first_readouts(&mut self, events: u64, rows: &[Case], control: bool) -> Result<()> {
        if self.first_readouts.is_some() || self.domains.values().any(|d| d.shadow.is_some()) {
            return Err(Error::Busy);
        }
        if self.domains.values().any(|d| {
            d.active
                .as_ref()
                .is_some_and(|p| d.active_program_id() != Some(fingerprint(p)))
        }) {
            return Err(Error::InvalidData);
        }
        let mut readouts = FirstReadouts {
            enabled_at: events,
            fixed_readout_control: control,
            ..Default::default()
        };
        readouts
            .seen
            .extend(rows.iter().map(|r| fingerprint(&r.pre)));
        for domain in self.domains.values_mut() {
            // Do not mix late-retry observations with the new objective.
            // Historical completed audits and the active program are retained.
            domain.monitoring = 0;
            domain.fixed_correct = 0;
            domain.active_correct = 0;
        }
        self.first_readouts = Some(readouts);
        Ok(())
    }

    fn fixed_readout_control(&self) -> bool {
        self.first_readouts
            .as_ref()
            .is_some_and(|r| r.fixed_readout_control)
    }

    // Called for every measured PRE, including failed actions, before fitting
    // any part of its current feedback. No reward or chosen action is read.
    fn needs_first_readout(&mut self, pre: &[i64]) -> bool {
        let Some(readouts) = self.first_readouts.as_mut() else {
            return false;
        };
        let key = fingerprint(&pre);
        if readouts.seen.contains(&key) {
            return false;
        }
        if readouts.seen.len() == RULE_KEYS {
            readouts.exhausted = true;
            return false;
        }
        readouts.seen.insert(key);
        self.domains
            .get(&pre[0])
            .is_some_and(|d| !d.seen.contains(&key) && (d.active.is_some() || d.shadow.is_some()))
    }

    fn capture_first_readout(
        &mut self,
        pre: &[i64],
        event: u64,
        fixed: [f64; 16],
        old: [f64; 16],
        proposed: Option<[f64; 16]>,
    ) {
        let Some(readouts) = self.first_readouts.as_mut() else {
            return;
        };
        let Some(domain) = self.domains.get(&pre[0]) else {
            return;
        };
        if readouts.pending.len() == READOUT_PENDING {
            readouts.pending.remove(0);
            readouts.evicted += 1;
        }
        readouts.pending.push(RuleFirstReadout {
            input: fingerprint(&pre),
            namespace: pre[0],
            before_event: event,
            incumbent: domain.active_program_id(),
            candidate: domain
                .shadow
                .as_ref()
                .map(|t| t.comparison.program_fingerprint),
            fixed,
            old,
            proposed,
        });
        readouts.captured += 1;
    }

    fn resolve_first_readout(&mut self, row: &Case) -> Option<RuleFirstReadout> {
        let readouts = self.first_readouts.as_mut()?;
        let key = fingerprint(&row.pre);
        let index = readouts.pending.iter().position(|p| p.input == key)?;
        let prediction = readouts.pending.remove(index);
        readouts.resolved += 1;
        Some(prediction)
    }
}

impl RuleDomain {
    fn active_program_id(&self) -> Option<u64> {
        let program = self.active.as_ref()?;
        self.audits
            .iter()
            .rev()
            .find(|a| a.accepted && a.trained_through == program.trained_through)
            .map(|a| a.program_fingerprint)
    }
}

#[cfg(test)]
mod first_readout_tests {
    use super::*;

    fn book() -> AcquiredRules {
        let program = RewardProgram {
            trees: vec![
                vec![RewardNode {
                    votes: std::array::from_fn(|a| u16::from(a == 2)),
                    split: None,
                }];
                RULE_TREES
            ],
            trained_through: 1,
        };
        let audit = AcquiredRuleAudit {
            trained_through: 1,
            closed_at: 65,
            future_examples: RULE_EXAM,
            incumbent_correct: 0,
            candidate_correct: RULE_EXAM,
            fixed_correct: 0,
            incumbent_loss: 128.0,
            candidate_loss: 0.0,
            fixed_loss: 128.0,
            accepted: true,
            program_fingerprint: fingerprint(&program),
            incumbent_program: None,
        };
        let domain = RuleDomain {
            active: Some(program),
            audits: vec![audit],
            ..Default::default()
        };
        AcquiredRules {
            domains: [(4, domain)].into(),
            proposals: 1,
            work: RULE_LIFETIME_WORK,
            ..Default::default()
        }
    }
    fn example(id: u64, input: i64, reward: f64) -> Case {
        let mut pre = vec![0; 32];
        pre[0] = 4;
        pre[1] = 1;
        pre[2] = input;
        Case {
            id,
            post: pre.clone(),
            pre,
            action: 11,
            reward,
            cost: 1,
            value: reward,
            distance: 1,
        }
    }
    fn prediction(action: usize) -> [f64; 16] {
        std::array::from_fn(|a| f64::from(a == action))
    }

    #[test]
    fn retries_and_restore_cannot_rewrite_the_first_forecast_or_hide_regression() {
        let mut b = book();
        b.enable_first_readouts(100, &[], false).unwrap();
        for i in 0..64 {
            let failed = example(101 + i * 2, 400 + i as i64, 0.0);
            assert!(b.needs_first_readout(&failed.pre));
            b.capture_first_readout(&failed.pre, failed.id, prediction(11), prediction(2), None);
            b.observe(&failed, prediction(11), prediction(11), None, &[]);
            assert!(!b.needs_first_readout(&failed.pre));
            let saved = serde_json::to_string(&b).unwrap();
            b = serde_json::from_str(&saved).unwrap();
            assert_eq!(saved, serde_json::to_string(&b).unwrap());
            assert!(b.validate(failed.id));
            let success = example(failed.id + 1, failed.pre[2], 1.0);
            // These revised forecasts look perfect AFTER corrective feedback.
            // They must not replace the old, actually incorrect first one.
            b.observe(&success, prediction(11), prediction(11), None, &[]);
            assert!(!b.needs_first_readout(&success.pre));
        }
        let report = b.report();
        let readouts = report.first_readouts.unwrap();
        assert_eq!(readouts.captured, 64);
        assert_eq!(readouts.resolved, 64);
        assert_eq!(readouts.monitoring_windows.len(), 1);
        let window = &readouts.monitoring_windows[0];
        assert_eq!(window.fixed_correct, 64);
        assert_eq!(window.active_correct, 0);
        assert!(window.rolled_back);
        assert_eq!(report.rollbacks, 1);
        assert!(report.active_domains.is_empty());
        assert!(b.validate(228));
    }

    #[test]
    fn evicted_first_forecasts_never_become_fresh_and_malformed_receipts_fail() {
        let mut b = book();
        b.enable_first_readouts(100, &[], true).unwrap();
        for i in 0..=READOUT_PENDING {
            let row = example(101 + i as u64, 400 + i as i64, 0.0);
            assert!(b.needs_first_readout(&row.pre));
            b.capture_first_readout(&row.pre, row.id, prediction(11), prediction(2), None);
        }
        assert_eq!(b.first_readouts.as_ref().unwrap().evicted, 1);
        let first = example(999, 400, 1.0);
        assert!(!b.needs_first_readout(&first.pre));
        b.observe(&first, prediction(11), prediction(11), None, &[]);
        assert_eq!(b.domains[&4].monitoring, 0);
        assert!(b.validate(999));
        let mut bad = b.clone();
        bad.first_readouts.as_mut().unwrap().pending[0].fixed[0] = 2.0;
        assert!(!bad.validate(999));
        let mut sparse = b.clone();
        sparse.first_readouts.as_mut().unwrap().pending[0].fixed = [0.01; 16];
        assert!(
            sparse.validate(999),
            "low-coverage forecasts are valid sub-probabilities"
        );
        let mut bad = b.clone();
        bad.first_readouts.as_mut().unwrap().pending[0].before_event = 1000;
        assert!(!bad.validate(999));
        assert_eq!(b.enable_first_readouts(999, &[], false), Err(Error::Busy));
    }
}
