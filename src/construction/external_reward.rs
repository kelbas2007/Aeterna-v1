//! Reward-conditioned extension of the factual learner. All stored successors
//! and delayed returns come from executed protected actions, never rollouts.
//! Kernel / mutation operators are authored; acquired strategies are not AGI.
use super::vm::{Error, Meter, Result};
use super::{autonomy::ActionOffer, causal_memory::CausalProposal, sensors::SensorEncoder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const CAPACITY: usize = 2048;
const MODEL_BYTES: usize = 4 * 1024 * 1024;
const EPISODE: usize = 512;
const GAMMA: f64 = 0.95;
const FAILURE_WIDTH: usize = 2048;
const FAILURE_DEPTH: usize = 4;

include!("acquired_rules.rs");
include!("first_readout.rs");

fn zero(value: &f64) -> bool {
    *value == 0.0
}

/// Bounded count-min summary of failures lost from the exact rate table.
/// Collisions can overestimate failure mass, never manufacture successes.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetiredFailures {
    buckets: Vec<f64>,
}
impl Default for RetiredFailures {
    fn default() -> Self {
        Self {
            buckets: vec![0.0; FAILURE_WIDTH * FAILURE_DEPTH],
        }
    }
}
impl RetiredFailures {
    fn indices(key: &str) -> [usize; FAILURE_DEPTH] {
        std::array::from_fn(|row| {
            let seed = 0xcbf29ce484222325_u64 ^ (row as u64).wrapping_mul(0x9e3779b97f4a7c15);
            let hash = key
                .as_bytes()
                .iter()
                .fold(seed, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x100000001b3));
            row * FAILURE_WIDTH + hash as usize % FAILURE_WIDTH
        })
    }
    fn add(&mut self, key: &str, failures: f64) {
        for index in Self::indices(key) {
            self.buckets[index] += failures;
        }
    }
    fn estimate(&self, key: &str) -> f64 {
        Self::indices(key)
            .into_iter()
            .map(|i| self.buckets[i])
            .fold(f64::INFINITY, f64::min)
    }
}

fn fingerprint<T: Serialize>(value: &T) -> u64 {
    serde_json::to_vec(value)
        .unwrap_or_default()
        .iter()
        .fold(0xcbf29ce484222325, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metric {
    count: Vec<u64>,
    mean: Vec<f64>,
    m2: Vec<f64>,
}
impl Default for Metric {
    fn default() -> Self {
        Self {
            count: vec![0; 30],
            mean: vec![0.0; 30],
            m2: vec![0.0; 30],
        }
    }
}
impl Metric {
    fn observe(&mut self, state: &[i64]) {
        for j in 0..30 {
            if state[1] & (1 << j) != 0 {
                self.count[j] = self.count[j].saturating_add(1);
                let d = state[j + 2] as f64 - self.mean[j];
                self.mean[j] += d / self.count[j] as f64;
                self.m2[j] = (self.m2[j] + d * (state[j + 2] as f64 - self.mean[j])).max(0.0);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Case {
    id: u64,
    pre: Vec<i64>,
    action: usize,
    post: Vec<i64>,
    reward: f64,
    cost: u64,
    value: f64,
    distance: usize,
}
impl Case {
    fn bytes(&self) -> usize {
        serde_json::to_vec(self).map_or(MODEL_BYTES + 1, |b| b.len() + 32)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pending {
    row: Case,
    fields: Option<[f32; 5]>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Visit {
    pre: Vec<i64>,
    action: usize,
    post: Vec<i64>,
    reward: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RewardRate {
    count: u64,
    sum: f64,
    last: u64,
    #[serde(default, skip_serializing_if = "zero")]
    retired_failures: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Strategy {
    neighbors: usize,
    exponent: f64,
    groups: [f64; 3],
}
impl Default for Strategy {
    fn default() -> Self {
        Self {
            neighbors: 3,
            exponent: 1.0,
            groups: [1.0; 3],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecursiveStrategy {
    active: Strategy,
    candidate: Strategy,
    generation: u64,
    accepted: u64,
    rejected: u64,
    trials: usize,
    old_loss: f64,
    new_loss: f64,
    mutation_weights: [f64; 3],
    operator: usize,
    trained_before_event: u64,
    audits: Vec<StrategyAudit>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAudit {
    pub generation: u64,
    pub trained_before_event: u64,
    pub closed_at_event: u64,
    pub comparisons: usize,
    pub incumbent_loss: f64,
    pub candidate_loss: f64,
    pub retention_guard_passed: bool,
    pub accepted: bool,
    pub mutation_family: usize,
}
impl Default for RecursiveStrategy {
    fn default() -> Self {
        let mut s = Self {
            active: Strategy::default(),
            candidate: Strategy::default(),
            generation: 0,
            accepted: 0,
            rejected: 0,
            trials: 0,
            old_loss: 0.0,
            new_loss: 0.0,
            mutation_weights: [1.0; 3],
            operator: 0,
            trained_before_event: 0,
            audits: Vec::new(),
        };
        s.propose();
        s
    }
}
impl RecursiveStrategy {
    fn propose(&mut self) {
        self.candidate = self.active.clone();
        // The generator learns which mutation family to revisit, and every
        // candidate descends from the last accepted learned strategy.
        let total = self.mutation_weights.iter().sum::<f64>();
        let draw = ((self.generation.wrapping_mul(0x9e3779b97f4a7c15) >> 11) as f64
            / (1u64 << 53) as f64)
            * total;
        let mut cumulative = 0.0;
        self.operator = 2;
        for (i, w) in self.mutation_weights.iter().enumerate() {
            cumulative += w;
            if draw < cumulative {
                self.operator = i;
                break;
            }
        }
        match self.operator {
            0 => {
                self.candidate.neighbors = match self.active.neighbors {
                    1 => 3,
                    3 => 5,
                    5 => 7,
                    _ => 1,
                }
            }
            1 => {
                self.candidate.exponent = if self.active.exponent < 1.5 {
                    self.active.exponent + 0.5
                } else {
                    0.5
                }
            }
            _ => {
                let axis = (self.generation / 3) as usize % 3;
                self.candidate.groups[axis] = if self.active.groups[axis] < 2.0 {
                    self.active.groups[axis] * 2.0
                } else {
                    0.5
                };
            }
        }
        self.trials = 0;
        self.old_loss = 0.0;
        self.new_loss = 0.0;
    }
    fn feedback(&mut self, old: f64, new: f64, actual: f64, event: u64, retention_ok: bool) {
        self.old_loss += (old - actual).powi(2);
        self.new_loss += (new - actual).powi(2);
        self.trials += 1;
        if self.trials < 64 {
            return;
        }
        let accepted = retention_ok && self.new_loss + 0.01 < self.old_loss * 0.98;
        self.audits.push(StrategyAudit {
            generation: self.generation,
            trained_before_event: self.trained_before_event,
            closed_at_event: event,
            comparisons: self.trials,
            incumbent_loss: self.old_loss,
            candidate_loss: self.new_loss,
            retention_guard_passed: retention_ok,
            accepted,
            mutation_family: self.operator,
        });
        if self.audits.len() > 64 {
            self.audits.remove(0);
        }
        if accepted {
            self.active = self.candidate.clone();
            self.accepted += 1;
        } else {
            self.rejected += 1;
        }
        self.mutation_weights[self.operator] = (self.mutation_weights[self.operator] * 0.8
            + if accepted { 0.4 } else { 0.04 })
        .clamp(0.2, 4.0);
        self.generation += 1;
        self.trained_before_event = event;
        self.propose();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExternalMemory {
    pub encoder: SensorEncoder,
    pub enabled: bool,
    adaptive: bool,
    rows: Vec<Case>,
    metrics: BTreeMap<i64, Metric>,
    pending: Vec<Pending>,
    recent: Vec<Visit>,
    reward_rates: BTreeMap<String, RewardRate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retired_failures: Option<RetiredFailures>,
    strategy: RecursiveStrategy,
    events: u64,
    learned: u64,
    bytes: usize,
    evictions: u64,
    #[serde(default)]
    credits: Vec<([f32; 5], f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    acquired_rules: Option<AcquiredRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalRewardReport {
    pub enabled: bool,
    pub factual_events: u64,
    pub learned_events: u64,
    pub learned_fingerprint: u64,
    pub retained_cases: usize,
    pub model_bytes: usize,
    pub evictions: u64,
    pub pending_actions: usize,
    pub working_transitions: usize,
    pub retained_reward_rates: usize,
    pub namespaces: usize,
    pub recursive_generations: u64,
    pub accepted_strategy_changes: u64,
    pub rejected_strategy_changes: u64,
    pub learned_mutation_weights: [f64; 3],
    pub acquired_neighbors: usize,
    pub acquired_kernel_exponent: f64,
    pub strategy_audits: Vec<StrategyAudit>,
    pub acquired_rules: Option<AcquiredRulesReport>,
}

impl ExternalMemory {
    pub(crate) fn enable_first_rule_readouts(&mut self, control: bool) -> Result<()> {
        if !self.enabled || self.adaptive || !self.pending.is_empty() { return Err(Error::Busy); }
        self.acquired_rules.as_mut().ok_or(Error::NoEvidence)?
            .enable_first_readouts(self.events, &self.rows, control)
    }
    pub(crate) fn enable_acquired_rules(&mut self) -> Result<()> {
        if !self.enabled || self.adaptive || self.acquired_rules.is_some() || !self.pending.is_empty() {
            return Err(Error::Busy);
        }
        self.acquired_rules = Some(AcquiredRules::default());
        Ok(())
    }
    pub(crate) fn use_shared_development(&mut self) {
        // The existing RSP2/trajectory mechanism owns development in this mode.
        // Keep EXTERNAL5 parameter adaptation only for its historical protocol.
        self.adaptive = false;
    }
    pub fn new(enabled: bool, adaptive: bool) -> Self {
        Self {
            encoder: SensorEncoder::default(),
            enabled,
            adaptive,
            rows: Vec::new(),
            metrics: BTreeMap::new(),
            pending: Vec::new(),
            recent: Vec::new(),
            reward_rates: BTreeMap::new(),
            retired_failures: None,
            strategy: RecursiveStrategy::default(),
            events: 0,
            learned: 0,
            bytes: 0,
            evictions: 0,
            credits: Vec::new(),
            acquired_rules: None,
        }
    }
    pub fn report(&self) -> ExternalRewardReport {
        let previous_fingerprint = fingerprint(&(
            &self.rows,
            &self.metrics,
            &self.strategy,
            &self.reward_rates,
            self.enabled,
            self.adaptive,
        ));
        let previous_fingerprint = self.acquired_rules.as_ref().map_or(previous_fingerprint,
            |rules| fingerprint(&(previous_fingerprint, rules)));
        ExternalRewardReport {
            enabled: self.enabled,
            factual_events: self.events,
            learned_events: self.learned,
            learned_fingerprint: self
                .retired_failures
                .as_ref()
                .map_or(previous_fingerprint, |failures| {
                    fingerprint(&(previous_fingerprint, failures))
                }),
            retained_cases: self.rows.len(),
            model_bytes: self.bytes
                + self.acquired_rules.as_ref().map_or(0, |r| serde_json::to_vec(r).map_or(0, |b| b.len()))
                + serde_json::to_vec(&self.metrics).map_or(0, |b| b.len())
                + serde_json::to_vec(&self.reward_rates).map_or(0, |b| b.len())
                + self
                    .retired_failures
                    .as_ref()
                    .map_or(0, |s| s.buckets.len() * std::mem::size_of::<f64>()),
            evictions: self.evictions,
            pending_actions: self.pending.len(),
            working_transitions: self.recent.len(),
            retained_reward_rates: self.reward_rates.len(),
            namespaces: self.metrics.len(),
            recursive_generations: self.strategy.generation,
            accepted_strategy_changes: self.strategy.accepted,
            rejected_strategy_changes: self.strategy.rejected,
            learned_mutation_weights: self.strategy.mutation_weights,
            acquired_neighbors: self.strategy.active.neighbors,
            acquired_kernel_exponent: self.strategy.active.exponent,
            strategy_audits: self.strategy.audits.clone(),
            acquired_rules: self.acquired_rules.as_ref().map(AcquiredRules::report),
        }
    }
    pub fn take_credits(&mut self) -> Vec<([f32; 5], f64)> {
        std::mem::take(&mut self.credits)
    }
    pub fn clear_working(&mut self) {
        self.recent.clear();
    }
    pub fn recently_tried(&self, pre: &[i64], action: usize) -> bool {
        self.recent
            .iter()
            .any(|v| v.action == action && v.pre == pre)
    }
    fn note_visit(&mut self, pre: &[i64], action: usize, post: &[i64], reward: f64) {
        self.recent.push(Visit {
            pre: pre.to_vec(),
            action,
            post: post.to_vec(),
            reward,
        });
        if self.recent.len() > 128 {
            self.recent.remove(0);
        }
    }
    fn distance(&self, a: &[i64], b: &[i64], policy: &Strategy) -> Option<f64> {
        if a[0] != b[0] || b[1] & a[1] != a[1] {
            return None;
        }
        let metric = self.metrics.get(&a[0]);
        let mut sum = 0.0;
        let mut n = 0;
        for j in 0..30 {
            if a[1] & (1 << j) != 0 {
                let scale = metric.map_or(256.0, |m| {
                    (m.m2[j] / m.count[j].max(1) as f64).sqrt().max(8.0)
                });
                let d = (a[j + 2] as f64 - b[j + 2] as f64) / scale;
                sum += policy.groups[j % 3] * d * d;
                n += 1;
            }
        }
        Some(sum / n.max(1) as f64)
    }
    fn nearest_positive(
        &self,
        state: &[i64],
        policy: &Strategy,
        omit: Option<u64>,
        meter: &mut Meter,
    ) -> Vec<(f64, usize)> {
        let mut nearest = Vec::<(f64, usize)>::new();
        for (i, row) in self.rows.iter().enumerate() {
            if meter.charge(1).is_err() {
                break;
            }
            if row.reward <= 0.0 || Some(row.id) == omit {
                continue;
            }
            if meter.charge(33).is_err() {
                break;
            }
            if let Some(d) = self.distance(state, &row.pre, policy) {
                let at = nearest.partition_point(|(v, j)| (*v, *j) < (d, i));
                if at < policy.neighbors {
                    nearest.insert(at, (d, i));
                    nearest.truncate(policy.neighbors);
                }
            }
        }
        nearest
    }
    fn immediate(
        &self,
        state: &[i64],
        actions: usize,
        policy: &Strategy,
        meter: &mut Meter,
    ) -> (Vec<f64>, Vec<u64>) {
        self.immediate_except(state, actions, policy, None, meter)
    }
    fn immediate_except(
        &self,
        state: &[i64],
        actions: usize,
        policy: &Strategy,
        omit: Option<u64>,
        meter: &mut Meter,
    ) -> (Vec<f64>, Vec<u64>) {
        let program = self.acquired_rules.as_ref().filter(|r| !r.fixed_readout_control())
            .and_then(|r| r.domains.get(&state[0])).and_then(|d| d.active.as_ref());
        self.immediate_with_program(state, actions, policy, omit, program, meter)
    }
    fn immediate_with_program(
        &self, state: &[i64], actions: usize, policy: &Strategy, omit: Option<u64>,
        program: Option<&RewardProgram>, meter: &mut Meter,
    ) -> (Vec<f64>, Vec<u64>) {
        let nearest = self.nearest_positive(state, policy, omit, meter);
        let mut values = vec![0.0; actions];
        let mut support = vec![0; actions];
        let mut total = 0.0;
        for (d, i) in &nearest {
            let row = &self.rows[*i];
            let weight = 1.0 / (0.01 + *d).powf(policy.exponent);
            values[row.action] += weight * row.reward;
            support[row.action] += 1;
            total += weight;
        }
        let coverage = nearest
            .first()
            .map_or(0.0, |(d, _)| (-d.sqrt() / 4.0).exp());
        if total > 0.0 {
            for v in &mut values {
                *v = (*v / total * coverage).clamp(0.0, 1.0);
            }
        }
        // Acquired conditional programs change this SAME reward estimator.
        // Exact factual contradiction/evicted-failure guards below still apply.
        if let Some(prediction) = program.and_then(|p| p.infer(state, meter)) {
            let base = rule_normalize(&values);
            for a in 0..actions {
                values[a] = (0.5 * base[a] + 0.5 * prediction[a]) * coverage;
            }
        }
        // An unmeasured or aliased state can have contradictory outcomes.
        // Exact retained frequencies override a fortunate nearest prototype.
        // After eviction, a conservative failure prior prevents 1/1 resets.
        if omit.is_none() {
            for a in 0..actions {
                if meter.charge(64).is_err() {
                    break;
                }
                let key = serde_json::to_string(&(state, a)).unwrap_or_default();
                if let Some(rate) = self.reward_rates.get(&key) {
                    values[a] = rate.sum / (rate.count as f64 + rate.retired_failures);
                    support[a] = rate.count;
                } else if let Some(retired) = &self.retired_failures {
                    // Apply forgotten failures BEFORE choosing the first
                    // revisiting action, not only after its new POST arrives.
                    values[a] /= 1.0 + retired.estimate(&key);
                }
            }
        }
        (values, support)
    }
    pub fn propose(
        &self,
        pre: &[i64],
        offers: &[ActionOffer],
        horizon: usize,
        remaining_cost: u64,
        budget: u64,
    ) -> (Option<CausalProposal>, u64) {
        let mut meter = Meter::new(budget);
        let policy = &self.strategy.active;
        let (immediate, support) = self.immediate(pre, offers.len(), policy, &mut meter);
        let future_quote = offers.iter().map(|o| o.max_cost).max().unwrap_or(0);
        let noops = (0..offers.len())
            .map(|action| {
                self.recent
                    .iter()
                    .filter(|v| {
                        v.action == action && v.pre == pre && v.post == pre && v.reward <= 0.0
                    })
                    .count()
            })
            .collect::<Vec<_>>();
        let calibrated = immediate
            .iter()
            .enumerate()
            .map(|(a, q)| q / (1.0 + noops[a] as f64))
            .collect::<Vec<_>>();
        let mut root = vec![Vec::<(f64, usize)>::new(); offers.len()];
        for (index, row) in self.rows.iter().enumerate() {
            if meter.charge(1).is_err() {
                break;
            }
            if !offers[row.action].available {
                continue;
            }
            if meter.charge(33).is_err() {
                break;
            }
            if let Some(distance) = self.distance(pre, &row.pre, policy) {
                let nearest = &mut root[row.action];
                let at = nearest.partition_point(|(d, i)| (*d, *i) < (distance, index));
                if at < 3 {
                    nearest.insert(at, (distance, index));
                    nearest.truncate(3);
                }
            }
        }
        let mut best = None;
        let mut best_score = f64::NEG_INFINITY;
        for (action, offer) in offers.iter().enumerate() {
            if !offer.available || offer.max_cost > remaining_cost {
                continue;
            }
            let nearest = &root[action];
            let mut value = immediate[action];
            let mut expected = None;
            let mut confidence = support[action] as f64 / (support[action] + 2) as f64;
            if horizon > 1 && value < GAMMA {
                for (rank, &(distance, index)) in nearest.iter().enumerate() {
                    let row = &self.rows[index];
                    let coverage = (-distance.sqrt() / 4.0).exp();
                    // A recorded rewarded predecessor is an acquired subgoal.
                    // A ten-action return cannot satisfy a one-action deadline.
                    if row.distance > 1
                        && row.distance <= horizon
                        && future_quote
                            .checked_mul(row.distance.saturating_sub(1) as u64)
                            .and_then(|tail| offer.max_cost.checked_add(tail))
                            .is_some_and(|quote| quote <= remaining_cost)
                        && row.value * coverage > value
                    {
                        value = row.value * coverage;
                        expected = Some(row.post.clone());
                        confidence = coverage;
                    }
                    if rank == 0
                        && row.post != row.pre
                        && offer
                            .max_cost
                            .checked_add(future_quote)
                            .is_some_and(|quote| quote <= remaining_cost)
                    {
                        let (future, _) =
                            self.immediate(&row.post, offers.len(), policy, &mut meter);
                        let next = future.into_iter().fold(0.0, f64::max) * GAMMA * coverage;
                        if next > value {
                            value = next;
                            expected = Some(row.post.clone());
                            confidence = coverage;
                        }
                    }
                }
            }
            let visits = self
                .recent
                .iter()
                .filter(|v| v.action == action && v.pre == pre)
                .count();
            // These are current-episode beliefs, including frozen inference.
            // A failed transfer should trigger another experiment instead of
            // repeating an optimistic route into the same observed obstacle.
            value /= 1.0 + noops[action] as f64;
            let information = 1.0 / ((1.0 + nearest.len() as f64) * (1.0 + visits as f64));
            let score = value + 0.03 * information - 0.005 * (offer.max_cost as f64).ln_1p();
            if score > best_score {
                best_score = score;
                best = Some(CausalProposal {
                    action,
                    value,
                    information_gain: information * std::f64::consts::LN_2,
                    confidence,
                    expected,
                    immediate_goals: calibrated.clone(),
                    immediate_support: support.clone(),
                });
            }
        }
        (best, meter.spent)
    }
    pub fn observe(
        &mut self,
        pre: Vec<i64>,
        action: usize,
        post: Vec<i64>,
        reward: f64,
        cost: u64,
        fields: Option<[f32; 5]>,
        learning: bool,
    ) -> u64 {
        self.events = self.events.saturating_add(1);
        if !learning || !self.enabled {
            self.note_visit(&pre, action, &post, reward);
            return 0;
        }
        let mut work = 64;
        if self.acquired_rules.is_some() && pre[1] != 0 {
            // Score a future outcome before inserting or fitting it. The
            // authored 3NN is the fixed comparison; the active rule program
            // competes with the next acquired revision inside observe().
            let first_mode = self.acquired_rules.as_ref().unwrap().first_readouts.is_some();
            if first_mode { work += 128; } // bounded input hash/lookup and immutable receipt bookkeeping
            let predict = if first_mode {
                self.acquired_rules.as_mut().unwrap().needs_first_readout(&pre)
            } else { reward > 0.0 };
            let mut meter = Meter::new(300_000);
            let rules = self.acquired_rules.as_ref().unwrap();
            let domain = rules.domains.get(&pre[0]);
            let baseline = if predict { self.immediate_with_program(&pre, 16, &self.strategy.active, None, None, &mut meter).0 }
                else { vec![0.0; 16] };
            let old = domain.filter(|_| predict).and_then(|d| d.active.as_ref()).map_or_else(|| baseline.clone(), |p|
                self.immediate_with_program(&pre, 16, &self.strategy.active, None, Some(p), &mut meter).0);
            let candidate = domain.filter(|_| predict).and_then(|d| d.shadow.as_ref()).map(|t|
                self.immediate_with_program(&pre, 16, &self.strategy.active, None, Some(&t.program), &mut meter).0);
            if first_mode && predict {
                work += 128;
                self.acquired_rules.as_mut().unwrap().capture_first_readout(&pre, self.events,
                    rule_normalize(&baseline), rule_normalize(&old), candidate.as_ref().map(|p| rule_normalize(p)));
            }
            let row = Case { id: self.events, pre: pre.clone(), action, post: post.clone(), reward, cost, value: reward, distance: 1 };
            work += meter.spent + self.acquired_rules.as_mut().unwrap().observe(&row,
                rule_normalize(&baseline), rule_normalize(&old), candidate.as_ref().map(|p| rule_normalize(p)), &self.rows);
        }
        if self.pending.len() >= EPISODE {
            self.finish();
        }
        // Evaluate both strategies BEFORE fitting this observation. No future
        // answer or held-out loss can admit a mutation retroactively.
        if self.adaptive && self.rows.len() >= 64 && self.events % 16 == 0 {
            let mut meter = Meter::new(600_000);
            let (old, _) = self.immediate(&pre, 16, &self.strategy.active, &mut meter);
            let (new, _) = self.immediate(&pre, 16, &self.strategy.candidate, &mut meter);
            let mut retention_ok = true;
            if self.strategy.trials == 63 {
                let old_cases = self
                    .rows
                    .iter()
                    .filter(|r| r.pre[0] != pre[0] && r.reward > 0.0)
                    .collect::<Vec<_>>();
                for index in (0..3).map(|i| i * old_cases.len() / 3) {
                    if let Some(row) = old_cases.get(index) {
                        let (before, _) = self.immediate_except(
                            &row.pre,
                            16,
                            &self.strategy.active,
                            Some(row.id),
                            &mut meter,
                        );
                        let (after, _) = self.immediate_except(
                            &row.pre,
                            16,
                            &self.strategy.candidate,
                            Some(row.id),
                            &mut meter,
                        );
                        retention_ok &= (after[row.action] - row.reward).powi(2)
                            <= (before[row.action] - row.reward).powi(2) + 0.01;
                    }
                }
                retention_ok &= meter.spent < 600_000;
            }
            self.strategy.feedback(
                old[action],
                new[action],
                reward.max(0.0),
                self.events,
                retention_ok,
            );
            work += meter.spent;
        }
        self.note_visit(&pre, action, &post, reward);
        // Outcome-stratified prototype storage must not inflate frequencies.
        let key = serde_json::to_string(&(&pre, action)).unwrap_or_default();
        if !self.reward_rates.contains_key(&key) && self.reward_rates.len() >= 1024 {
            if let Some(old) = self
                .reward_rates
                .iter()
                .min_by_key(|(_, r)| (r.count, r.last))
                .map(|(k, _)| k.clone())
            {
                let rate = self.reward_rates.remove(&old).unwrap();
                // Only retire newly observed mass. The prior already lives in
                // the sketch and must not be counted again on the next eviction.
                self.retired_failures
                    .get_or_insert_with(RetiredFailures::default)
                    .add(&old, (rate.count as f64 - rate.sum).max(0.0));
            }
        }
        let prior = self
            .retired_failures
            .as_ref()
            .map_or(0.0, |s| s.estimate(&key));
        let rate = self.reward_rates.entry(key).or_insert(RewardRate {
            count: 0,
            sum: 0.0,
            last: self.events,
            retired_failures: prior,
        });
        rate.count = rate.count.saturating_add(1);
        rate.sum += reward.max(0.0);
        rate.last = self.events;
        for state in [&pre, &post] {
            if self.metrics.contains_key(&state[0]) || self.metrics.len() < 8 {
                self.metrics.entry(state[0]).or_default().observe(state);
            }
        }
        self.learned = self.learned.saturating_add(1);
        let row = Case {
            id: self.events,
            pre,
            action,
            post,
            reward,
            cost,
            value: reward,
            distance: usize::from(reward > 0.0),
        };
        self.insert(row.clone());
        self.pending.push(Pending { row, fields });
        if reward > 0.0 {
            self.finish();
        }
        work
    }
    fn insert(&mut self, row: Case) {
        if let Some(i) = self.rows.iter().position(|r| r.id == row.id) {
            self.bytes = self.bytes.saturating_sub(self.rows[i].bytes());
            self.rows.remove(i);
        }
        let size = row.bytes();
        if size > MODEL_BYTES {
            return;
        }
        while self.rows.len() >= CAPACITY || self.bytes.saturating_add(size) > MODEL_BYTES {
            // Bounded stratification protects both older sensor interfaces and
            // rare actual rewards. Old facts stay owned by the same learner.
            let mut buckets = BTreeMap::<(i64, bool), usize>::new();
            for r in &self.rows {
                *buckets.entry((r.pre[0], r.reward > 0.0)).or_default() += 1;
            }
            let bucket = buckets
                .iter()
                .max_by(|(ka, na), (kb, nb)| {
                    let wa = if ka.1 { 3 } else { 1 };
                    let wb = if kb.1 { 3 } else { 1 };
                    (**na * wb, ka).cmp(&(**nb * wa, kb))
                })
                .map(|(k, _)| *k)
                .unwrap();
            let index = self
                .rows
                .iter()
                .position(|r| (r.pre[0], r.reward > 0.0) == bucket)
                .unwrap();
            self.bytes = self.bytes.saturating_sub(self.rows[index].bytes());
            self.rows.remove(index);
            self.evictions += 1;
        }
        self.bytes += size;
        self.rows.push(row);
    }
    pub fn finish(&mut self) {
        let pending = std::mem::take(&mut self.pending);
        let mut value = 0.0;
        let mut distance = 0;
        for mut step in pending.into_iter().rev() {
            value = (step.row.reward + GAMMA * value).clamp(-1.0, 1.0);
            if step.row.reward > 0.0 {
                distance = 1;
            } else if distance > 0 {
                distance += 1;
            }
            step.row.value = value;
            step.row.distance = distance;
            if let Some(fields) = step.fields {
                self.credits.push((fields, value.max(0.0)));
            }
            self.insert(step.row);
        }
    }
    pub fn validate(&self, actions: usize, events: u64, frozen: bool) -> Result<()> {
        self.encoder.validate()?;
        let valid_state = |s: &Vec<i64>| {
            s.len() == 32 && (0..=262143).contains(&s[0]) && (0..(1_i64 << 30)).contains(&s[1])
        };
        let valid_row = |r: &Case| {
            valid_state(&r.pre)
                && valid_state(&r.post)
                && r.pre[0] == r.post[0]
                && r.action < actions
                && r.id > 0
                && r.id <= events
                && r.reward.is_finite()
                && (-1.0..=1.0).contains(&r.reward)
                && r.value.is_finite()
                && (-1.0..=1.0).contains(&r.value)
                && r.distance <= EPISODE
        };
        let valid_policy = |p: &Strategy| {
            [1, 3, 5, 7].contains(&p.neighbors)
                && (0.5..=2.0).contains(&p.exponent)
                && p.groups
                    .iter()
                    .all(|x| x.is_finite() && (0.5..=2.0).contains(x))
        };
        let unique = self
            .rows
            .iter()
            .map(|r| r.id)
            .collect::<std::collections::BTreeSet<_>>();
        if self.events != events
            || self.acquired_rules.as_ref().is_some_and(|r| !self.enabled || self.adaptive || !r.validate(events))
            || self.learned > events
            || self.rows.len() > CAPACITY
            || self.bytes > MODEL_BYTES
            || unique.len() != self.rows.len()
            || self.rows.len() as u64 > self.learned
            || self.strategy.generation
                != self
                    .strategy
                    .accepted
                    .saturating_add(self.strategy.rejected)
            || self.strategy.audits.len() > 64
            || self.strategy.trained_before_event > events
            || self.strategy.audits.iter().any(|a| {
                a.generation >= self.strategy.generation
                    || a.closed_at_event > events
                    || a.trained_before_event > a.closed_at_event
                    || a.comparisons != 64
                    || !a.incumbent_loss.is_finite()
                    || !a.candidate_loss.is_finite()
                    || a.incumbent_loss < 0.0
                    || a.candidate_loss < 0.0
                    || a.mutation_family >= 3
                    || (a.accepted
                        && (!a.retention_guard_passed
                            || a.candidate_loss + 0.01 >= a.incumbent_loss * 0.98))
            })
            || (!self.enabled
                && (!self.rows.is_empty()
                    || !self.pending.is_empty()
                    || self.learned != 0
                    || !self.metrics.is_empty()))
            || (!self.enabled && !self.reward_rates.is_empty())
            || (!self.enabled && self.retired_failures.is_some())
            || self.retired_failures.as_ref().is_some_and(|s| {
                s.buckets.len() != FAILURE_WIDTH * FAILURE_DEPTH
                    || s.buckets
                        .iter()
                        .any(|v| !v.is_finite() || *v < 0.0 || *v > events as f64 + 1e-9)
            })
            || self.reward_rates.len() > 1024
            || self.reward_rates.iter().any(|(key, r)| {
                let parsed = serde_json::from_str::<(Vec<i64>, usize)>(key);
                key.len() > 1024
                    || r.count == 0
                    || r.count > events
                    || r.last > events
                    || !r.sum.is_finite()
                    || r.sum < 0.0
                    || r.sum > r.count as f64 + 1e-9
                    || !r.retired_failures.is_finite()
                    || r.retired_failures < 0.0
                    || r.retired_failures > events as f64 + 1e-9
                    || (r.retired_failures > 0.0 && self.retired_failures.is_none())
                    || self
                        .retired_failures
                        .as_ref()
                        .is_some_and(|s| r.retired_failures > s.estimate(key) + 1e-9)
                    || parsed.is_err()
                    || parsed.is_ok_and(|(s, a)| !valid_state(&s) || a >= actions)
            })
            || self.bytes != self.rows.iter().map(Case::bytes).sum::<usize>()
            || self.pending.len() > EPISODE
            || self.recent.len() > 128
            || self.recent.iter().any(|v| {
                !valid_state(&v.pre)
                    || !valid_state(&v.post)
                    || v.pre[0] != v.post[0]
                    || v.action >= actions
                    || !v.reward.is_finite()
                    || !(-1.0..=1.0).contains(&v.reward)
            })
            || self.credits.len() > EPISODE
            || (frozen && !self.pending.is_empty())
            || self.metrics.len() > 8
            || self.rows.iter().any(|r| !valid_row(r))
            || self.pending.iter().any(|p| !valid_row(&p.row))
            || !valid_policy(&self.strategy.active)
            || !valid_policy(&self.strategy.candidate)
            || self.strategy.trials >= 64
            || !self.strategy.old_loss.is_finite()
            || !self.strategy.new_loss.is_finite()
            || self.strategy.operator >= 3
            || self
                .strategy
                .mutation_weights
                .iter()
                .any(|x| !x.is_finite() || !(0.2..=4.0).contains(x))
        {
            return Err(Error::InvalidData);
        }
        for m in self.metrics.values() {
            if m.count.len() != 30
                || m.mean.len() != 30
                || m.m2.len() != 30
                || m.mean.iter().any(|x| !x.is_finite())
                || m.m2.iter().any(|x| !x.is_finite() || *x < 0.0)
            {
                return Err(Error::InvalidData);
            }
        }
        if self.credits.iter().any(|(f, v)| {
            f.iter().any(|x| !x.is_finite()) || !v.is_finite() || !(0.0..=1.0).contains(v)
        }) {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state(namespace: i64, x: i64) -> Vec<i64> {
        let mut v = vec![0; 32];
        v[0] = namespace;
        v[1] = 1;
        v[2] = x;
        v
    }
    #[test]
    fn actual_delayed_reward_teaches_a_route_without_a_goal_vector() {
        let mut m = ExternalMemory::new(true, false);
        for _ in 0..8 {
            m.observe(state(1, 0), 0, state(1, 1000), 0.0, 1, None, true);
            m.observe(state(1, 1000), 1, state(1, 2000), 1.0, 1, None, true);
        }
        let offers = vec![
            ActionOffer {
                available: true,
                max_cost: 1
            };
            2
        ];
        let p = m.propose(&state(1, 0), &offers, 2, 2, 1_000_000).0.unwrap();
        assert_eq!(p.action, 0);
        assert!(p.value >= 0.94);
        assert!(p.expected.is_some());
        let limited = m.propose(&state(1, 0), &offers, 1, 1, 1_000_000).0.unwrap();
        assert_ne!(
            limited.action, 0,
            "a delayed return must not fit in one action"
        );
        assert!(m
            .propose(&state(1, 0), &offers, 2, 0, 1_000_000)
            .0
            .is_none());
    }
    #[test]
    fn forced_followup_settles_one_prior_choice_and_freeze_preserves_knowledge() {
        let mut m = ExternalMemory::new(true, true);
        let fields = [0.2; 5];
        m.observe(state(1, 0), 0, state(1, 1000), 0.0, 1, Some(fields), true);
        assert!(m.take_credits().is_empty());
        m.observe(state(1, 1000), 1, state(1, 2000), 1.0, 1, None, true);
        assert_eq!(m.take_credits(), vec![(fields, 0.95)]);
        let hash = m.report().learned_fingerprint;
        m.observe(state(1, 0), 1, state(1, 1000), 0.0, 1, Some(fields), false);
        assert_eq!(hash, m.report().learned_fingerprint);
        assert!(m.take_credits().is_empty());
        m.validate(2, 3, true).unwrap();
    }
    #[test]
    fn recursive_candidates_descend_from_accepted_strategies() {
        let mut s = RecursiveStrategy::default();
        let candidate = s.candidate.clone();
        for event in 1..=64 {
            s.feedback(0.9, 0.1, 0.0, event, true);
        }
        assert_eq!(s.active, candidate);
        assert_eq!(s.accepted, 1);
        assert_eq!(s.generation, 1);
        assert_ne!(s.candidate, s.active);
        assert_ne!(s.mutation_weights, [1.0; 3]);
        let active = s.active.clone();
        for event in 65..=128 {
            s.feedback(0.0, 1.0, 0.0, event, true);
        }
        assert_eq!(s.active, active);
        assert_eq!(s.rejected, 1);
        let active = s.active.clone();
        for event in 129..=192 {
            s.feedback(0.9, 0.1, 0.0, event, false);
        }
        assert_eq!(s.active, active);
        assert!(!s.audits.last().unwrap().retention_guard_passed);
    }
    #[test]
    fn contradictory_rewards_at_an_aliased_state_reduce_the_forecast() {
        let mut m = ExternalMemory::new(true, false);
        let blank = state(1, 0);
        m.observe(blank.clone(), 0, blank.clone(), 1.0, 1, None, true);
        m.observe(blank.clone(), 0, blank.clone(), 0.0, 1, None, true);
        m.finish();
        let mut meter = Meter::new(100_000);
        let (q, n) = m.immediate(&blank, 2, &Strategy::default(), &mut meter);
        assert_eq!(q[0], 0.5);
        assert_eq!(n[0], 2);
    }
    #[test]
    fn frozen_working_memory_changes_an_unproductive_experiment_without_learning() {
        let mut m = ExternalMemory::new(true, false);
        let pre = state(1, 0);
        let offers = vec![
            ActionOffer {
                available: true,
                max_cost: 1
            };
            2
        ];
        assert_eq!(m.propose(&pre, &offers, 1, 1, 100_000).0.unwrap().action, 0);
        let hash = m.report().learned_fingerprint;
        m.observe(pre.clone(), 0, pre.clone(), 0.0, 1, None, false);
        assert_eq!(m.propose(&pre, &offers, 1, 1, 100_000).0.unwrap().action, 1);
        assert_eq!(hash, m.report().learned_fingerprint);
        m.validate(2, 1, true).unwrap();
    }
    #[test]
    fn delayed_returns_do_not_treat_expensive_future_motors_as_cheap() {
        let mut m = ExternalMemory::new(true, false);
        m.observe(state(1, 0), 0, state(1, 1000), 0.0, 1, None, true);
        m.observe(state(1, 1000), 1, state(1, 2000), 1.0, 100, None, true);
        let offers = vec![
            ActionOffer {
                available: true,
                max_cost: 1,
            },
            ActionOffer {
                available: true,
                max_cost: 100,
            },
        ];
        let p = m.propose(&state(1, 0), &offers, 2, 2, 1_000_000).0.unwrap();
        assert_eq!(p.action, 0);
        assert_eq!(p.value, 0.0);
        assert!(p.expected.is_none());
        let overflow = vec![
            ActionOffer {
                available: true,
                max_cost: u64::MAX
            };
            2
        ];
        let p = m
            .propose(&state(1, 0), &overflow, 2, u64::MAX, 1_000_000)
            .0
            .unwrap();
        assert!(p.expected.is_none());
    }
    #[test]
    fn prototype_eviction_does_not_turn_failed_guesses_into_likely_rewards() {
        let mut m = ExternalMemory::new(true, false);
        for i in 0..2500 {
            let repeated = i % 10 == 0;
            let pre = state(1, if repeated { 0 } else { i + 1000 });
            let reward = f64::from(i % 20 == 0);
            m.observe(pre.clone(), 0, pre, reward, 1, None, true);
            m.finish();
        }
        m.clear_working();
        assert!(m.report().evictions > 0);
        let (q, n) = m.immediate(
            &state(1, 0),
            2,
            &Strategy::default(),
            &mut Meter::new(1_000_000),
        );
        assert_eq!(q[0], 0.5);
        assert_eq!(n[0], 250);
        m.validate(2, 2500, false).unwrap();
    }
    #[test]
    fn evicted_rate_retains_failure_evidence_across_restore_and_reentry() {
        let mut m = ExternalMemory::new(true, false);
        let original = state(1, 0);
        m.observe(original.clone(), 0, original.clone(), 0.0, 1, None, true);
        m.finish();
        for i in 1..=1024 {
            let pre = state(1, i);
            m.observe(pre.clone(), 0, pre, 1.0, 1, None, true);
        }
        let key = serde_json::to_string(&(&original, 0_usize)).unwrap();
        assert!(!m.reward_rates.contains_key(&key));
        let saved = serde_json::to_string(&m).unwrap();
        let mut restored: ExternalMemory = serde_json::from_str(&saved).unwrap();
        restored.validate(2, 1025, false).unwrap();
        assert_eq!(
            m.report().learned_fingerprint,
            restored.report().learned_fingerprint
        );
        let (first_revisit, _) = restored.immediate(
            &original,
            2,
            &Strategy::default(),
            &mut Meter::new(1_000_000),
        );
        assert!(first_revisit[0] > 0.0 && first_revisit[0] <= 0.5);
        restored.observe(original.clone(), 0, original.clone(), 1.0, 1, None, true);
        let (values, _) = restored.immediate(
            &original,
            2,
            &Strategy::default(),
            &mut Meter::new(1_000_000),
        );
        assert!(values[0] > 0.0 && values[0] <= 0.5);
        restored.validate(2, 1026, false).unwrap();
        for i in 2000..3024 {
            let pre = state(1, i);
            restored.observe(pre.clone(), 0, pre, 1.0, 1, None, true);
        }
        assert!(!restored.reward_rates.contains_key(&key));
        // Re-eviction must not add the same retired failure prior twice.
        assert_eq!(
            restored.retired_failures.as_ref().unwrap().estimate(&key),
            1.0
        );
        restored.observe(original.clone(), 0, original.clone(), 1.0, 1, None, true);
        restored.validate(2, 2051, false).unwrap();
        let hash = restored.report().learned_fingerprint;
        restored.observe(original.clone(), 0, original, 0.0, 1, None, false);
        assert_eq!(hash, restored.report().learned_fingerprint);
        restored.retired_failures.as_mut().unwrap().buckets.pop();
        assert!(restored.validate(2, 2052, false).is_err());
    }
    #[test]
    fn long_retained_rates_do_not_halve_away_old_failures() {
        let mut m = ExternalMemory::new(true, false);
        let pre = state(1, 0);
        for i in 0..600 {
            m.observe(
                pre.clone(),
                0,
                pre.clone(),
                f64::from(i % 4 == 0),
                1,
                None,
                true,
            );
            m.finish();
        }
        let (values, support) =
            m.immediate(&pre, 2, &Strategy::default(), &mut Meter::new(1_000_000));
        assert_eq!(values[0], 0.25);
        assert_eq!(support[0], 600);
        m.validate(2, 600, false).unwrap();
    }
}
