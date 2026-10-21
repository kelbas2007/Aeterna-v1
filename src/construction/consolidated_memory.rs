// Included in external_reward. Sufficient statistics are learned models, not
// fabricated factual transitions. Actual reward rates and U1 remain shared.
const CONSOLIDATED_KEYS: usize = 8192;
const CONSOLIDATED_PENDING: usize = 128;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FeatureMoments {
    count: [u64; 30],
    mean: [f64; 30],
    m2: [f64; 30],
}
impl FeatureMoments {
    fn observe(&mut self, state: &[i64]) -> u64 {
        let mut work = 0;
        for j in 0..30 {
            if state[1] & (1 << j) != 0 {
                self.count[j] += 1;
                let delta = state[j + 2] as f64 - self.mean[j];
                self.mean[j] += delta / self.count[j] as f64;
                self.m2[j] = (self.m2[j] + delta * (state[j + 2] as f64 - self.mean[j])).max(0.0);
                work += 16;
            }
        }
        work
    }
    fn valid(&self, examples: u64) -> bool {
        (0..30).all(|j| {
            self.count[j] <= examples
                && self.mean[j].is_finite()
                && self.m2[j].is_finite()
                && self.m2[j] >= 0.0
                && (self.count[j] != 0 || (self.mean[j] == 0.0 && self.m2[j] == 0.0))
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct GaussianOutcome {
    examples: u64,
    features: FeatureMoments,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsolidatedDomain {
    global: GaussianOutcome,
    outcomes: Vec<GaussianOutcome>,
    comparisons: u64,
    baseline_loss: f64,
    consolidated_loss: f64,
    active: bool,
    switches: u64,
}
impl Default for ConsolidatedDomain {
    fn default() -> Self {
        Self {
            global: GaussianOutcome::default(),
            outcomes: vec![GaussianOutcome::default(); 16],
            comparisons: 0,
            baseline_loss: 0.0,
            consolidated_loss: 0.0,
            active: false,
            switches: 0,
        }
    }
}
impl ConsolidatedDomain {
    fn learn_positive(&mut self, pre: &[i64], action: usize) -> u64 {
        self.global.examples += 1;
        self.outcomes[action].examples += 1;
        32 + self.global.features.observe(pre) + self.outcomes[action].features.observe(pre)
    }
    fn compare(&mut self, baseline: f64, consolidated: f64, actual: f64) {
        self.comparisons += 1;
        let alpha = 1.0 / self.comparisons.min(128) as f64;
        self.baseline_loss += alpha * ((baseline - actual).powi(2) - self.baseline_loss);
        self.consolidated_loss +=
            alpha * ((consolidated - actual).powi(2) - self.consolidated_loss);
        let active =
            self.comparisons >= 64 && self.consolidated_loss + 0.005 < self.baseline_loss * 0.98;
        self.switches += u64::from(active != self.active);
        self.active = active;
    }
    fn valid(&self, bound: u64) -> bool {
        self.outcomes.len() == 16
            && self.global.examples <= bound
            && self
                .outcomes
                .iter()
                .try_fold(0_u64, |n, c| n.checked_add(c.examples))
                == Some(self.global.examples)
            && self.global.features.valid(self.global.examples)
            && self
                .outcomes
                .iter()
                .all(|c| c.examples <= bound && c.features.valid(c.examples))
            && (0..30).all(|j| {
                self.outcomes
                    .iter()
                    .try_fold(0_u64, |n, c| n.checked_add(c.features.count[j]))
                    == Some(self.global.features.count[j])
            })
            && self.baseline_loss.is_finite()
            && (0.0..=1.0).contains(&self.baseline_loss)
            && self.consolidated_loss.is_finite()
            && (0.0..=1.0).contains(&self.consolidated_loss)
            && self.switches <= self.comparisons
            && self.active
                == (self.comparisons >= 64
                    && self.consolidated_loss + 0.005 < self.baseline_loss * 0.98)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsolidatedForecast {
    key: u64,
    namespace: i64,
    before_event: u64,
    baseline: [f64; 16],
    consolidated: [f64; 16],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsolidatedMemory {
    revision: u8,
    enabled_at: u64,
    seeded_prototypes: usize,
    bootstrap_work: u64,
    training_work: u64,
    domains: BTreeMap<i64, ConsolidatedDomain>,
    seen: std::collections::BTreeSet<u64>,
    pending: Vec<ConsolidatedForecast>,
    captured: u64,
    resolved: u64,
    evicted: u64,
    exhausted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedDomainReport {
    pub positive_observations: u64,
    pub supported_motors: usize,
    pub comparisons: u64,
    pub baseline_loss: f64,
    pub consolidated_loss: f64,
    pub consolidated_active: bool,
    pub switches: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedMemoryReport {
    pub enabled_at: u64,
    pub seeded_prototypes: usize,
    pub bootstrap_work: u64,
    pub training_work: u64,
    pub captured: u64,
    pub resolved: u64,
    pub pending: usize,
    pub evicted: u64,
    pub distinct_inputs: usize,
    pub exhausted: bool,
    pub domains: BTreeMap<i64, ConsolidatedDomainReport>,
}
impl ConsolidatedMemory {
    fn seeded(events: u64, rows: &[Case]) -> Self {
        let mut m = Self {
            revision: 1,
            enabled_at: events,
            seeded_prototypes: 0,
            bootstrap_work: 0,
            training_work: 0,
            domains: BTreeMap::new(),
            seen: rows.iter().map(|r| fingerprint(&r.pre)).collect(),
            pending: Vec::new(),
            captured: 0,
            resolved: 0,
            evicted: 0,
            exhausted: false,
        };
        for row in rows.iter().filter(|r| r.reward > 0.0) {
            if m.domains.contains_key(&row.pre[0]) || m.domains.len() < 8 {
                m.bootstrap_work += m
                    .domains
                    .entry(row.pre[0])
                    .or_default()
                    .learn_positive(&row.pre, row.action);
                m.seeded_prototypes += 1;
            }
        }
        m
    }
    fn active(&self, namespace: i64) -> bool {
        self.domains.get(&namespace).is_some_and(|d| d.active)
    }
    fn needs_forecast(&mut self, pre: &[i64]) -> bool {
        let key = fingerprint(&pre);
        if self.seen.contains(&key) {
            return false;
        }
        if self.seen.len() == CONSOLIDATED_KEYS {
            self.exhausted = true;
            return false;
        }
        self.seen.insert(key);
        self.domains.contains_key(&pre[0])
    }
    fn capture(&mut self, pre: &[i64], event: u64, baseline: Vec<f64>, consolidated: Vec<f64>) {
        if self.pending.len() == CONSOLIDATED_PENDING {
            self.pending.remove(0);
            self.evicted += 1;
        }
        self.pending.push(ConsolidatedForecast {
            key: fingerprint(&pre),
            namespace: pre[0],
            before_event: event,
            baseline: std::array::from_fn(|a| baseline[a]),
            consolidated: std::array::from_fn(|a| consolidated[a]),
        });
        self.captured += 1;
    }
    fn feedback(&mut self, pre: &[i64], action: usize, reward: f64) -> u64 {
        let mut work = 128;
        // A known positive motor identifies an outcome action from OWN facts.
        // Compare only its actual reward; unexecuted motors get no fake labels.
        let outcome = reward > 0.0
            || self
                .domains
                .get(&pre[0])
                .is_some_and(|d| d.outcomes[action].examples > 0);
        if outcome {
            let key = fingerprint(&pre);
            if let Some(i) = self.pending.iter().position(|p| p.key == key) {
                let p = self.pending.remove(i);
                self.resolved += 1;
                self.domains.get_mut(&pre[0]).unwrap().compare(
                    p.baseline[action],
                    p.consolidated[action],
                    reward.clamp(0.0, 1.0),
                );
                work += 128;
            }
        }
        if reward > 0.0 && (self.domains.contains_key(&pre[0]) || self.domains.len() < 8) {
            work += self
                .domains
                .entry(pre[0])
                .or_default()
                .learn_positive(pre, action);
        }
        self.training_work += work;
        work
    }
    fn predict(
        &self,
        pre: &[i64],
        actions: usize,
        meter: &mut Meter,
    ) -> Option<(Vec<f64>, Vec<u64>)> {
        let d = self.domains.get(&pre[0])?;
        if pre[1] == 0 || d.outcomes.iter().filter(|c| c.examples > 0).count() < 2 {
            return None;
        }
        let mut logits = vec![f64::NEG_INFINITY; actions];
        let mut distances = vec![0.0; actions];
        for a in 0..actions {
            let c = &d.outcomes[a];
            if c.examples == 0 {
                continue;
            }
            let mut n = 0;
            for j in 0..30 {
                if pre[1] & (1 << j) == 0 {
                    continue;
                }
                meter.charge(16).ok()?;
                let global = &d.global.features;
                let gv = global.m2[j] / global.count[j].max(1) as f64;
                let (mean, variance) = if c.features.count[j] > 0 {
                    (
                        c.features.mean[j],
                        c.features.m2[j] / c.features.count[j] as f64,
                    )
                } else {
                    (global.mean[j], gv)
                };
                let variance = variance.max((gv * 0.05).max(64.0));
                let delta = pre[j + 2] as f64 - mean;
                let distance = delta * delta / variance;
                distances[a] += distance;
                if n == 0 {
                    logits[a] = 0.0;
                }
                logits[a] -= 0.5 * (variance.ln() + distance);
                n += 1;
            }
            distances[a] /= n.max(1) as f64;
        }
        let best = (0..actions).max_by(|&a, &b| logits[a].total_cmp(&logits[b]))?;
        let maximum = logits[best];
        if !maximum.is_finite() {
            return None;
        }
        let coverage = (-distances[best].sqrt() / 4.0).exp();
        let mut total = 0.0;
        for value in &mut logits {
            meter.charge(32).ok()?;
            *value = (*value - maximum).exp();
            total += *value;
        }
        for value in &mut logits {
            *value = (*value / total * coverage).clamp(0.0, 1.0);
        }
        let support = d
            .outcomes
            .iter()
            .take(actions)
            .map(|c| u64::from(c.examples > 0))
            .collect();
        Some((logits, support))
    }
    fn report(&self) -> ConsolidatedMemoryReport {
        ConsolidatedMemoryReport {
            enabled_at: self.enabled_at,
            seeded_prototypes: self.seeded_prototypes,
            bootstrap_work: self.bootstrap_work,
            training_work: self.training_work,
            captured: self.captured,
            resolved: self.resolved,
            pending: self.pending.len(),
            evicted: self.evicted,
            distinct_inputs: self.seen.len(),
            exhausted: self.exhausted,
            domains: self
                .domains
                .iter()
                .map(|(&key, d)| {
                    (
                        key,
                        ConsolidatedDomainReport {
                            positive_observations: d.global.examples,
                            supported_motors: d.outcomes.iter().filter(|c| c.examples > 0).count(),
                            comparisons: d.comparisons,
                            baseline_loss: d.baseline_loss,
                            consolidated_loss: d.consolidated_loss,
                            consolidated_active: d.active,
                            switches: d.switches,
                        },
                    )
                })
                .collect(),
        }
    }
    fn validate(&self, events: u64) -> bool {
        let mut keys = std::collections::BTreeSet::new();
        let probability = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
        self.revision == 1
            && self.enabled_at <= events
            && self.seeded_prototypes <= CAPACITY
            && self.bootstrap_work <= self.seeded_prototypes as u64 * 1024
            && self.training_work <= (events - self.enabled_at).saturating_mul(400_000)
            && self.domains.len() <= 8
            && self
                .domains
                .values()
                .try_fold(0_u64, |n, d| n.checked_add(d.global.examples))
                .is_some_and(|n| {
                    n <= (self.seeded_prototypes as u64).saturating_add(events - self.enabled_at)
                })
            && self
                .domains
                .iter()
                .all(|(&k, d)| (0..=262143).contains(&k) && d.valid(events))
            && self
                .domains
                .values()
                .try_fold(0_u64, |n, d| n.checked_add(d.comparisons))
                == Some(self.resolved)
            && self.seen.len() <= CONSOLIDATED_KEYS
            && self.pending.len() <= CONSOLIDATED_PENDING
            && self
                .captured
                .checked_sub(self.resolved)
                .and_then(|n| n.checked_sub(self.evicted))
                == Some(self.pending.len() as u64)
            && self.captured <= self.seen.len() as u64
            && self.pending.iter().all(|p| {
                self.seen.contains(&p.key)
                    && keys.insert(p.key)
                    && p.before_event > self.enabled_at
                    && p.before_event <= events
                    && self.domains.contains_key(&p.namespace)
                    && p.baseline.iter().copied().all(probability)
                    && p.consolidated.iter().copied().all(probability)
            })
    }
}

impl ExternalMemory {
    pub(crate) fn enable_consolidated_memory(&mut self) -> Result<u64> {
        if !self.enabled
            || self.adaptive
            || !self.pending.is_empty()
            || self.coverage_memory.is_some()
            || self.consolidated_memory.is_some()
        {
            return Err(Error::Busy);
        }
        let model = ConsolidatedMemory::seeded(self.events, &self.rows);
        let work = model.bootstrap_work;
        self.consolidated_memory = Some(model);
        Ok(work)
    }
    fn consolidated_prediction(
        &self,
        pre: &[i64],
        actions: usize,
        omit: Option<u64>,
        meter: &mut Meter,
    ) -> Option<(Vec<f64>, Vec<u64>)> {
        // The sufficient statistics are not a leave-one-out model. Keep the
        // existing excluded-row estimator for such callers, and reserve ALL
        // factual-guard work before returning any consolidated action values.
        if omit.is_some() {
            return None;
        }
        let guard_work = actions as u64 * 64;
        meter.charge(guard_work).ok()?;
        let (mut values, mut support) = self
            .consolidated_memory
            .as_ref()?
            .predict(pre, actions, meter)?;
        self.factual_reward_guards(
            pre,
            &mut values,
            &mut support,
            omit,
            &mut Meter::new(guard_work),
        );
        Some((values, support))
    }
    fn prepare_consolidated_forecast(&mut self, pre: &[i64]) -> u64 {
        if pre[1] == 0
            || !self
                .consolidated_memory
                .as_mut()
                .is_some_and(|c| c.needs_forecast(pre))
        {
            return 0;
        }
        let mut meter = Meter::new(300_000);
        if let Some((candidate, _)) = self.consolidated_prediction(pre, 16, None, &mut meter) {
            let program = self
                .acquired_rules
                .as_ref()
                .filter(|r| !r.fixed_readout_control())
                .and_then(|r| r.domains.get(&pre[0]))
                .and_then(|d| d.active.as_ref());
            let (baseline, _) = self.immediate_with_program(
                pre,
                16,
                &self.strategy.active,
                None,
                program,
                &mut meter,
            );
            self.consolidated_memory.as_mut().unwrap().capture(
                pre,
                self.events,
                baseline,
                candidate,
            );
        }
        let work = meter.spent + 128;
        self.consolidated_memory.as_mut().unwrap().training_work += work;
        work
    }
}

#[cfg(test)]
mod consolidated_memory_tests {
    use super::*;
    fn state(mask: i64, x: i64) -> Vec<i64> {
        let mut s = vec![0; 32];
        s[0] = 9;
        s[1] = mask;
        s[2] = x;
        s[3] = x;
        s
    }
    #[test]
    fn statistical_values_cannot_bypass_an_observed_failure_or_unpaid_guards() {
        let mut m = ExternalMemory::new(true, false);
        m.observe(state(1, 0), 2, state(1, 0), 1.0, 1, None, true);
        m.observe(state(1, 1000), 3, state(1, 1000), 1.0, 1, None, true);
        let failed = state(1, 1);
        m.observe(failed.clone(), 2, failed.clone(), 0.0, 1, None, true);
        m.finish();
        m.enable_consolidated_memory().unwrap();
        let (values, _) = m
            .consolidated_prediction(&failed, 16, None, &mut Meter::new(100_000))
            .unwrap();
        assert_eq!(values[2], 0.0);
        assert!(m
            .consolidated_prediction(&failed, 16, None, &mut Meter::new(0))
            .is_none());
        assert!(m
            .consolidated_prediction(&failed, 16, Some(1), &mut Meter::new(100_000))
            .is_none());
    }
    #[test]
    fn separately_observed_features_combine_without_a_full_mask_prototype() {
        let mut c = ConsolidatedMemory::seeded(0, &[]);
        for _ in 0..20 {
            for mask in [1, 2] {
                c.feedback(&state(mask, 0), 2, 1.0);
                c.feedback(&state(mask, 1000), 3, 1.0);
            }
        }
        let mut meter = Meter::new(100_000);
        let (p, _) = c.predict(&state(3, 0), 16, &mut meter).unwrap();
        assert!(p[2] > p[3] && p[2] > 0.8);
        let (p, _) = c.predict(&state(3, 1000), 16, &mut meter).unwrap();
        assert!(p[3] > p[2] && p[3] > 0.8);
        assert!(c.validate(80));
        let before = serde_json::to_string(&c).unwrap();
        let restored: ConsolidatedMemory = serde_json::from_str(&before).unwrap();
        assert_eq!(before, serde_json::to_string(&restored).unwrap());
    }
    #[test]
    fn first_feedback_selects_model_without_rewriting_forecasts_after_retries() {
        let mut c = ConsolidatedMemory::seeded(0, &[]);
        c.domains.insert(9, ConsolidatedDomain::default());
        c.domains
            .get_mut(&9)
            .unwrap()
            .learn_positive(&state(1, 0), 2);
        for i in 1..=64 {
            let pre = state(1, i);
            assert!(c.needs_forecast(&pre));
            let mut old = vec![0.0; 16];
            let mut new = old.clone();
            old[2] = 0.9;
            new[2] = 0.1;
            c.capture(&pre, i as u64, old, new);
            c.feedback(&pre, 2, 0.0);
            assert!(!c.needs_forecast(&pre));
            c.feedback(&pre, 2, 1.0);
        }
        assert_eq!(c.domains[&9].comparisons, 64);
        assert!(c.domains[&9].active);
        assert_eq!(c.resolved, 64);
        let before = c.domains[&9].consolidated_loss;
        assert!(
            before < 0.011,
            "later positive retries cannot replace the first actual negative reward"
        );
        let mut bad = c.clone();
        bad.domains.get_mut(&9).unwrap().global.features.count[0] += 1;
        assert!(!bad.validate(200));
    }
    #[test]
    fn frozen_feedback_preserves_consolidated_parameters_and_failure_history() {
        let mut m = ExternalMemory::new(true, false);
        m.observe(state(1, 0), 2, state(1, 0), 1.0, 1, None, true);
        m.observe(state(2, 1000), 3, state(2, 1000), 1.0, 1, None, true);
        let events = m.events;
        let before = fingerprint(&m.reward_rates);
        let work = m.enable_consolidated_memory().unwrap();
        assert!(work > 0);
        assert_eq!(m.events, events);
        assert_eq!(before, fingerprint(&m.reward_rates));
        assert_eq!(m.enable_coverage_memory(), Err(Error::Busy));
        let hash = m.report().learned_fingerprint;
        m.observe(state(3, 500), 2, state(3, 500), 0.0, 1, None, false);
        assert_eq!(hash, m.report().learned_fingerprint);
        m.validate(16, 3, false).unwrap();
    }
}
