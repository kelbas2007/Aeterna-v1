//! Online predictive-state hypotheses learned only from executed transitions.
//! Hypothesis scores are prequential; observation entropy alone is not value
//! of information. Imagined branches never update these sufficient statistics.
use super::autonomy::ActionOffer;
use super::synthesis::Transition;
use super::vm::{Error, Meter, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub use super::inquiry::{InquiryDecision, InquiryMode, InquiryPrediction, InquiryQuestion, InquiryReport, InquiryResolution};
use super::inquiry::{InquiryState, MAX_FORECAST_OUTCOMES};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct CausalConfig {
    pub history_depth: usize,
    pub model_entries: usize,
    pub model_bytes: usize,
    pub outcomes_per_entry: usize,
    pub planning_states: usize,
}
impl Default for CausalConfig {
    fn default() -> Self {
        Self {
            history_depth: 8,
            model_entries: 2048,
            model_bytes: 2 * 1024 * 1024,
            outcomes_per_entry: 16,
            planning_states: 192,
        }
    }
}
impl CausalConfig {
    pub fn validate(&self) -> Result<()> {
        if !(1..=32).contains(&self.history_depth)
            || !(32..=16384).contains(&self.model_entries)
            || !(16384..=8 * 1024 * 1024).contains(&self.model_bytes)
            || !(2..=64).contains(&self.outcomes_per_entry)
            || !(16..=1024).contains(&self.planning_states)
        {
            return Err(Error::Capacity);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextStep {
    pub pre: Vec<i64>,
    pub action: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Outcome {
    post: Vec<i64>,
    count: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Counts {
    outcomes: Vec<Outcome>,
    other: u64,
    last: u64,
    depth: usize,
}
impl Counts {
    fn total(&self) -> u64 {
        self.outcomes
            .iter()
            .fold(self.other, |n, o| n.saturating_add(o.count))
    }
    fn age(&mut self) {
        // Effective recent evidence, with rare counterexamples retained.
        // Lifetime factual/learning counters remain absolute and separate.
        if self.total() >= 512 {
            self.other = self.other.div_ceil(2);
            for outcome in &mut self.outcomes {
                outcome.count = outcome.count.div_ceil(2);
            }
        }
    }
    fn bytes(&self) -> usize {
        // Account conservatively for RAM and JSON: signed i64 values need
        // up to twenty digits, not eight bytes, in persistent checkpoints.
        128 + self
            .outcomes
            .iter()
            .map(|o| 64 + o.post.len() * 24)
            .sum::<usize>()
    }
    fn distribution(&self) -> Vec<(Vec<i64>, f64)> {
        let total = self.total() as f64;
        if total == 0.0 {
            return Vec::new();
        }
        self.outcomes
            .iter()
            .map(|o| (o.post.clone(), o.count as f64 / total))
            .collect()
    }
    fn probability(&self, post: &[i64]) -> f64 {
        // A new outcome remains possible; noise never deletes an action law.
        let n = self.total() as f64;
        (self
            .outcomes
            .iter()
            .find(|o| o.post == post)
            .map_or(0.0, |o| o.count as f64)
            + 0.5)
            / (n + 0.5 * (self.outcomes.len() + 1) as f64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReport {
    pub factual_transitions: u64,
    pub learned_transitions: u64,
    pub learned_fingerprint: u64,
    pub retained_entries: usize,
    pub model_bytes: usize,
    pub evicted_entries: u64,
    pub compacted_transitions: u64,
    pub compacted_external_cost: u64,
    pub compacted_journal_digest: u64,
    pub compactions: u64,
    pub hypothesis_weights: Vec<f64>,
    pub completed_goal_episodes: u64,
    pub failed_goal_episodes: u64,
    pub useful_experiments: u64,
    pub delayed_experiment_credits: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBelief {
    pub action: usize,
    pub outcomes: Vec<(Vec<i64>, f64)>,
    pub information_gain: f64,
    pub goal_relevance: f64,
    pub support: u64,
    pub unknown_probability: f64,
}

/// Experimental, default-off inference of applicability from factual POSTs.
/// This selects no motor and receives no evaluator regime/switch labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegimeMode {
    Ordinary,
    Phase,
    PhaseScrambled,
    PhaseUncoupled,
}

const REGIME_INITIAL: [f64; 4] = [0.15, 0.25, 0.35, 0.45];
const REGIME_OMEGA: [f64; 4] = [-0.03, -0.01, 0.01, 0.03];
// Conservative fixed accounting includes the 32-byte four-channel core,
// diagnostic state, counters, and their checkpoint encoding. Both banks still
// share the single original table's entry and byte caps.
const REGIME_STATE_BYTES: usize = 2048;
// Fixed conservative work envelope, not an elapsed-time estimate. The update
// has eight count reads and at most eight four-channel integration substeps.
const REGIME_UPDATE_WORK: u64 = 8192;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeReport {
    pub mode: RegimeMode,
    pub bank: usize,
    pub admitted: bool,
    pub observations: u64,
    pub switches: u64,
    pub birth_surprises: u8,
    pub state: [f64; 4],
    pub likelihood_ratio: [f64; 4],
    pub likelihoods: [[f64; 2]; 4],
    pub supports: [[u64; 2]; 4],
    pub weights: [f64; 4],
    pub readout: f64,
    /// Charged conservative envelope (8192 per factual inference update),
    /// not measured CPU operations or elapsed time.
    pub extra_inference_work: u64,
    pub state_bytes: usize,
    pub dynamic_fingerprint: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegimeState {
    mode: RegimeMode,
    enabled_at: u64,
    bank: usize,
    admitted: bool,
    observations: u64,
    switches: u64,
    birth_surprises: u8,
    state: [f64; 4],
    likelihood_ratio: [f64; 4],
    likelihoods: [[f64; 2]; 4],
    supports: [[u64; 2]; 4],
    weights: [f64; 4],
    readout: f64,
    extra_inference_work: u64,
    innovation: u64,
}

fn regime_wrap(angle: f64) -> f64 {
    angle.rem_euclid(std::f64::consts::TAU)
}

// Angles are relative to a common reference. A common rotation of all
// physical phases AND that reference therefore leaves these equations intact.
fn regime_phase_velocity(state: [f64; 4], evidence: [f64; 4], coupling: f64) -> [f64; 4] {
    std::array::from_fn(|i| {
        let interaction = (0..4)
            .filter(|&j| j != i)
            .map(|j| (state[j] - state[i]).sin())
            .sum::<f64>()
            / 3.0;
        REGIME_OMEGA[i] + coupling * interaction - 2.0 * evidence[i] * state[i].sin()
    })
}

impl RegimeState {
    fn initial(mode: RegimeMode, bank: usize) -> [f64; 4] {
        REGIME_INITIAL.map(|angle| {
            let angle = if bank == 0 { angle } else { std::f64::consts::PI - angle };
            if mode == RegimeMode::Ordinary { (1.0 + angle.cos()) / 2.0 } else { angle }
        })
    }
    fn new(mode: RegimeMode, enabled_at: u64, weights: [f64; 4]) -> Self {
        let mut state = Self {
            mode, enabled_at, bank: 0, admitted: false, observations: 0,
            switches: 0, birth_surprises: 0, state: Self::initial(mode, 0),
            likelihood_ratio: [0.0; 4], likelihoods: [[0.5; 2]; 4],
            supports: [[0; 2]; 4], weights, readout: 0.0,
            extra_inference_work: 0, innovation: 0xa72e_4b19_d038_65cf,
        };
        state.refresh_readout();
        state
    }
    fn refresh_readout(&mut self) {
        self.readout = (0..4).map(|i| {
            self.weights[i] * if self.mode == RegimeMode::Ordinary {
                2.0 * self.state[i] - 1.0
            } else { self.state[i].cos() }
        }).sum::<f64>().clamp(-1.0, 1.0);
    }
    fn report(&self) -> RegimeReport {
        RegimeReport {
            mode: self.mode, bank: self.bank, admitted: self.admitted,
            observations: self.observations, switches: self.switches,
            birth_surprises: self.birth_surprises, state: self.state,
            likelihood_ratio: self.likelihood_ratio, likelihoods: self.likelihoods,
            supports: self.supports, weights: self.weights, readout: self.readout,
            extra_inference_work: self.extra_inference_work,
            state_bytes: REGIME_STATE_BYTES,
            dynamic_fingerprint: super::vm::fingerprint(&serde_json::to_vec(self).unwrap()),
        }
    }
    fn update(&mut self, likelihoods: [[f64; 2]; 4], supports: [[u64; 2]; 4], weights: [f64; 4], learning: bool) {
        self.likelihoods = likelihoods;
        self.supports = supports;
        self.weights = weights;
        self.observations = self.observations.saturating_add(1);
        self.extra_inference_work = self.extra_inference_work.saturating_add(REGIME_UPDATE_WORK);
        let prior_bank = self.bank;
        let pooled = (0..4).map(|i| weights[i] * likelihoods[i][prior_bank]).sum::<f64>();
        let support = supports.iter().map(|s| s[prior_bank]).max().unwrap_or(0);
        let mut born = false;
        if !self.admitted && learning && support >= 8 {
            if pooled < 0.25 {
                self.birth_surprises = self.birth_surprises.saturating_add(1);
            } else if pooled >= 0.5 {
                self.birth_surprises = 0;
            }
            if self.birth_surprises >= 2 {
                self.admitted = true;
                self.bank = 1;
                self.state = Self::initial(self.mode, 1);
                born = true;
            }
        }
        self.likelihood_ratio = std::array::from_fn(|i| {
            let [n0, n1] = supports[i];
            let [p0, p1] = likelihoods[i];
            // Knowledge of common sensors is not evidence against an empty
            // bank. A supported surprising consequence may compare to 0.5.
            let informative = (n0 >= 8 && n1 >= 8)
                || (n0 >= 8 && n1 == 0 && p0 < 0.25)
                || (n1 >= 8 && n0 == 0 && p1 < 0.25);
            if informative { (p0.max(1e-12) / p1.max(1e-12)).ln().clamp(-4.0, 4.0) }
            else { 0.0 }
        });
        if !born {
            if self.mode == RegimeMode::Ordinary {
                for i in 0..4 {
                    let prior = (0.98 * self.state[i] + 0.01).clamp(1e-12, 1.0 - 1e-12);
                    let odds = (prior / (1.0 - prior)).ln() + self.likelihood_ratio[i];
                    self.state[i] = 1.0 / (1.0 + (-odds).exp());
                }
            } else {
                let evidence = self.likelihood_ratio.map(|r| r / 4.0);
                let coupling = if self.mode == RegimeMode::PhaseUncoupled { 0.0 } else { 0.35 };
                for _ in 0..8 {
                    let velocity = regime_phase_velocity(self.state, evidence, coupling);
                    self.state = std::array::from_fn(|i| regime_wrap(self.state[i] + 0.125 * velocity[i]));
                }
            }
        }
        // Outcome-independent deterministic innovations preserve unit
        // magnitude and bank labels, while destroying relative-phase memory.
        // All arms advance the same innovation state for paired diagnostics.
        let mut innovations = [0.0; 4];
        for value in &mut innovations {
            self.innovation ^= self.innovation << 13;
            self.innovation ^= self.innovation >> 7;
            self.innovation ^= self.innovation << 17;
            *value = ((self.innovation >> 11) as f64 / ((1_u64 << 53) as f64)) * std::f64::consts::TAU;
        }
        if self.mode == RegimeMode::PhaseScrambled && !born {
            self.state = std::array::from_fn(|i| regime_wrap(self.state[i] + innovations[i]));
        }
        self.refresh_readout();
        if self.admitted && !born {
            if self.readout > 0.2 { self.bank = 0; }
            else if self.readout < -0.2 { self.bank = 1; }
        }
        if self.bank != prior_bank { self.switches = self.switches.saturating_add(1); }
    }
    fn validate(&self, events: u64) -> Result<()> {
        if self.bank > 1 || (!self.admitted && self.bank != 0)
            || self.birth_surprises > 2 || self.enabled_at.checked_add(self.observations) != Some(events)
            || self.switches > self.observations
            || self.extra_inference_work != self.observations.saturating_mul(REGIME_UPDATE_WORK)
            || self.state.iter().any(|x| !x.is_finite() || if self.mode == RegimeMode::Ordinary {
                !(0.0..=1.0).contains(x)
            } else { !(0.0..std::f64::consts::TAU).contains(x) })
            || self.likelihood_ratio.iter().any(|x| !x.is_finite() || !(-4.0..=4.0).contains(x))
            || self.likelihoods.iter().flatten().any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
            || self.supports.iter().flatten().any(|&n| n > events)
            || self.weights.iter().any(|w| !w.is_finite() || !(0.0..=1.0).contains(w))
            || (self.weights.iter().sum::<f64>() - 1.0).abs() > 1e-9
            || !self.readout.is_finite() || !(-1.0..=1.0).contains(&self.readout)
        { return Err(Error::InvalidData); }
        let mut check = self.clone();
        check.refresh_readout();
        if (check.readout - self.readout).abs() > 1e-12 { return Err(Error::InvalidData); }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub(crate) struct CausalProposal {
    pub action: usize,
    pub value: f64,
    pub information_gain: f64,
    pub confidence: f64,
    pub expected: Option<Vec<i64>>,
    pub immediate_goals: Vec<f64>,
    pub immediate_support: Vec<u64>,
    pub inquiry: Option<InquiryDecision>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExperimentCredit {
    fields: [f32; 5],
    elapsed_cost: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CausalMemory {
    pub config: CausalConfig,
    counts: BTreeMap<String, Counts>,
    scores: Vec<f64>,
    context: Vec<ContextStep>,
    events: u64,
    learned_events: u64,
    bytes: usize,
    evicted: u64,
    pub offset: u64,
    pub compacted_cost: u64,
    pub journal_digest: u64,
    pub seed_model: u64,
    pub seed_state: Vec<i64>,
    pub previous_seed: Option<(u64, Vec<i64>)>,
    compactions: u64,
    goal: Option<Vec<Option<i64>>>,
    episode_cost: u64,
    pending: Vec<ExperimentCredit>,
    credits: Vec<([f32; 5], f64)>,
    successes: u64,
    failures: u64,
    experiments: u64,
    rewarded_experiments: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    regime: Option<RegimeState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inquiry: Option<InquiryState>,
}

fn key(pre: &[i64], action: usize, context: &[ContextStep], depth: usize) -> String {
    // Exact, canonical keys: no collision can merge physical contexts.
    serde_json::to_string(&(pre, action, &context[context.len().saturating_sub(depth)..])).unwrap()
}

fn regime_key(pre: &[i64], action: usize, context: &[ContextStep], depth: usize, bank: usize) -> String {
    let original = key(pre, action, context, depth);
    if bank == 0 { original } else { format!("r1:{original}") }
}
fn entropy(p: impl Iterator<Item = f64>) -> f64 {
    p.filter(|&v| v > 0.0).map(|v| -v * v.ln()).sum()
}
fn digamma(mut x: f64) -> f64 {
    let mut value = 0.0;
    while x < 8.0 {
        value -= 1.0 / x;
        x += 1.0;
    }
    let inv = 1.0 / x;
    let square = inv * inv;
    value + x.ln() - 0.5 * inv - square * (1.0 / 12.0 - square * (1.0 / 120.0 - square / 252.0))
}
fn goal_law_information(support: u64, goal_probability: f64) -> f64 {
    // Beta(1,1) prior on the causal goal outcome of this motor/context.
    // I(theta; next factual goal bit), not outcome entropy or visit novelty.
    let a = 1.0 + support as f64 * goal_probability;
    let b = 1.0 + support as f64 * (1.0 - goal_probability);
    let n = a + b;
    let p = a / n;
    let predictive = entropy([p, 1.0 - p].into_iter());
    let conditional = digamma(n + 1.0) - p * digamma(a + 1.0) - (1.0 - p) * digamma(b + 1.0);
    (predictive - conditional).clamp(0.0, std::f64::consts::LN_2)
}
fn matches_goal(state: &[i64], goal: &[Option<i64>]) -> bool {
    state.len() == goal.len()
        && goal.iter().any(Option::is_some)
        && state
            .iter()
            .zip(goal)
            .all(|(&v, g)| g.is_none_or(|x| x == v))
}
impl CausalMemory {
    pub fn new(config: CausalConfig, model: u64, state: Vec<i64>) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            scores: (0..=config.history_depth)
                .map(|d| -(d as f64) * 0.5)
                .collect(),
            config,
            counts: BTreeMap::new(),
            context: Vec::new(),
            events: 0,
            learned_events: 0,
            bytes: 0,
            evicted: 0,
            offset: 0,
            compacted_cost: 0,
            journal_digest: 0,
            seed_model: model,
            seed_state: state,
            previous_seed: None,
            compactions: 0,
            goal: None,
            episode_cost: 0,
            pending: Vec::new(),
            credits: Vec::new(),
            successes: 0,
            failures: 0,
            experiments: 0,
            rewarded_experiments: 0,
            regime: None,
            inquiry: None,
        })
    }
    pub(crate) fn enable_regime_inference(&mut self, mode: RegimeMode) -> Result<()> {
        if self.regime.is_some() || self.config.history_depth != 3 { return Err(Error::Busy); }
        let bytes = self.bytes.checked_add(REGIME_STATE_BYTES).ok_or(Error::Capacity)?;
        if bytes > self.config.model_bytes { return Err(Error::Capacity); }
        let weights: [f64; 4] = self.weights().try_into().map_err(|_| Error::InvalidData)?;
        // Existing evidence keeps its byte-identical bank-zero keys. No
        // experience is copied, relabeled, discarded, or replayed here.
        self.regime = Some(RegimeState::new(mode, self.events, weights));
        self.bytes = bytes;
        Ok(())
    }
    pub fn regime_report(&self) -> Option<RegimeReport> {
        self.regime.as_ref().map(RegimeState::report)
    }
    /// Enable a factual inquiry agenda on the same live model. Allocation is
    /// checked before ordinary routing is enabled, so failure is atomic.
    pub(crate) fn enable_autonomous_inquiry(&mut self, mode: InquiryMode) -> Result<()> {
        if self.inquiry.is_some() || self.config.history_depth != 3
            || self.regime.as_ref().is_some_and(|r| r.mode != RegimeMode::Ordinary)
        { return Err(Error::Busy); }
        let required = super::inquiry::STATE_BYTES + if self.regime.is_none() { REGIME_STATE_BYTES } else { 0 };
        if self.bytes.checked_add(required).is_none_or(|n| n > self.config.model_bytes) {
            return Err(Error::Capacity);
        }
        if self.regime.is_none() { self.enable_regime_inference(RegimeMode::Ordinary)?; }
        self.inquiry = Some(InquiryState::new(mode, self.events));
        self.bytes += super::inquiry::STATE_BYTES;
        Ok(())
    }
    pub fn inquiry_report(&self) -> Option<InquiryReport> { self.inquiry.as_ref().map(InquiryState::report) }
    pub(crate) fn inquiry_inference_work(&self) -> u64 {
        self.inquiry.as_ref().map_or(0, InquiryState::inference_work)
    }
    pub(crate) fn record_inquiry_decision(&mut self, action: usize, decision: Option<&InquiryDecision>) -> Result<()> {
        let Some(state) = self.inquiry.as_mut() else {
            return if decision.is_none() { Ok(()) } else { Err(Error::NoEvidence) };
        };
        if let Some(d) = decision {
            let width = state.active.as_ref().ok_or(Error::Stale)?.triggering_pre.len();
            d.validate(width, 16, self.events)?;
        }
        state.record(action, decision, self.events)
    }
    fn active_inquiry(&self) -> Option<&InquiryQuestion> {
        self.inquiry.as_ref().filter(|s| s.mode == InquiryMode::Active).and_then(|s| s.active.as_ref())
    }
    fn bank_forecast(&self, pre: &[i64], action: usize, context: &[ContextStep], bank: usize,
        meter: &mut Meter) -> Result<InquiryPrediction> {
        let mut mixture = BTreeMap::<Vec<i64>, f64>::new();
        let mut mass = 0.0;
        let mut support = 0;
        for (depth, weight) in self.weights().into_iter().enumerate() {
            let mut d = depth.min(context.len());
            loop {
                meter.charge((pre.len().saturating_mul(d + 1) + d + 4) as u64)?;
                if let Some(c) = self.counts.get(&regime_key(pre, action, context, d, bank)) {
                    // Large, noisy supports are deliberately not compressed
                    // into falsely sharp rival forecasts.
                    if c.outcomes.len() > MAX_FORECAST_OUTCOMES {
                        return Ok(InquiryPrediction {bank, support: 0, outcomes: Vec::new(), unknown_probability: 1.0});
                    }
                    support = support.max(c.total());
                    mass += weight;
                    for (post, p) in c.distribution() {
                        meter.charge((post.len() + 4) as u64)?;
                        *mixture.entry(post).or_default() += weight * p;
                    }
                    break;
                }
                if d == 0 { break; }
                d -= 1;
            }
        }
        if mass == 0.0 || mixture.len() > MAX_FORECAST_OUTCOMES {
            return Ok(InquiryPrediction {bank, support: 0, outcomes: Vec::new(), unknown_probability: 1.0});
        }
        for p in mixture.values_mut() { *p /= mass; }
        let unknown_probability = (1.0 - mixture.values().sum::<f64>()).clamp(0.0, 1.0);
        Ok(InquiryPrediction {bank, support, outcomes: mixture.into_iter().collect(), unknown_probability})
    }
    fn inquiry_factual_forecasts(&self, t: &Transition) -> Option<Vec<InquiryPrediction>> {
        self.inquiry.as_ref()?;
        let mut meter = Meter::new(super::inquiry::UPDATE_WORK);
        Some((0..2).map(|bank| self.bank_forecast(&t.pre, t.action, &self.context, bank, &mut meter)
            .unwrap_or(InquiryPrediction {bank, support: 0, outcomes: Vec::new(), unknown_probability: 1.0})).collect())
    }
    fn active_key(&self, pre: &[i64], action: usize, context: &[ContextStep], depth: usize) -> String {
        regime_key(pre, action, context, depth, self.regime.as_ref().map_or(0, |r| r.bank))
    }
    fn infer_regime(&mut self, t: &Transition, learning: bool) -> Option<Vec<f64>> {
        let bank = self.regime.as_ref()?.bank;
        let weights: [f64; 4] = self.weights().try_into().expect("four-channel opt-in validated");
        let samples: [[(f64, u64); 2]; 4] = std::array::from_fn(|i| {
            std::array::from_fn(|candidate| {
                self.counts.get(&regime_key(&t.pre, t.action, &self.context, i.min(self.context.len()), candidate))
                    .map_or((0.5, 0), |c| (c.probability(&t.post), c.total()))
            })
        });
        let probabilities = samples.iter().map(|s| s[bank].0).collect();
        let likelihoods = samples.map(|s| s.map(|v| v.0));
        let supports = samples.map(|s| s.map(|v| v.1));
        self.regime.as_mut().unwrap().update(likelihoods, supports, weights, learning);
        Some(probabilities)
    }
    pub fn weights(&self) -> Vec<f64> {
        let high = self
            .scores
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let w = self
            .scores
            .iter()
            .map(|s| (s - high).max(-40.0).exp())
            .collect::<Vec<_>>();
        let sum = w.iter().sum::<f64>();
        w.into_iter().map(|x| x / sum).collect()
    }
    pub fn report(&self) -> CausalReport {
        let encoded = serde_json::to_vec(&(&self.counts, &self.scores)).unwrap();
        CausalReport {
            factual_transitions: self.events,
            learned_transitions: self.learned_events,
            learned_fingerprint: super::vm::fingerprint(&encoded),
            retained_entries: self.counts.len(),
            model_bytes: self.bytes,
            evicted_entries: self.evicted,
            compacted_transitions: self.offset,
            compacted_external_cost: self.compacted_cost,
            compacted_journal_digest: self.journal_digest,
            compactions: self.compactions,
            hypothesis_weights: self.weights(),
            completed_goal_episodes: self.successes,
            failed_goal_episodes: self.failures,
            useful_experiments: self.experiments,
            delayed_experiment_credits: self.rewarded_experiments,
        }
    }
    pub fn reset(&mut self, learning: bool) {
        if learning {
            self.finish_episode(false)
        } else {
            self.pending.clear();
        }
        self.context.clear();
        self.goal = None;
        self.episode_cost = 0;
    }
    pub(crate) fn finish_learning(&mut self) {
        // Close the final factual training episode before its decisions are
        // frozen. Preserve predictive context for subsequent frozen actions.
        self.finish_episode(false);
        self.goal = None;
        self.episode_cost = 0;
    }
    pub(crate) fn prepare_goal(&mut self, goal: Option<&[Option<i64>]>) {
        if self.goal.as_deref() != goal {
            // Close the old mission now. The next mission is opened only
            // after a real POST, so blocked/already-satisfied requests do not
            // invent an attempted episode.
            self.finish_learning();
        }
    }
    fn finish_episode(&mut self, success: bool) {
        if self.goal.is_some() {
            if success {
                self.successes = self.successes.saturating_add(1)
            } else {
                self.failures = self.failures.saturating_add(1)
            }
        }
        for probe in self.pending.drain(..) {
            let utility = if success {
                1.0 / (1.0 + 0.05 * probe.elapsed_cost as f64)
            } else {
                0.0
            };
            if success && probe.fields[2] > 0.0 {
                self.rewarded_experiments = self.rewarded_experiments.saturating_add(1)
            }
            if self.credits.len() < 128 {
                self.credits.push((probe.fields, utility));
            }
        }
    }
    pub fn feedback(
        &mut self,
        goal: Option<&[Option<i64>]>,
        post: &[i64],
        cost: u64,
        chosen_experiment: Option<[f32; 5]>,
        learning: bool,
    ) {
        if !learning {
            return;
        }
        let goal = goal.filter(|g| g.len() == post.len() && g.iter().any(Option::is_some));
        if self.goal.as_deref() != goal {
            self.finish_episode(false);
            self.goal = goal.map(<[Option<i64>]>::to_vec);
            self.episode_cost = 0;
        }
        self.episode_cost = self.episode_cost.saturating_add(cost);
        for credit in &mut self.pending {
            credit.elapsed_cost = credit.elapsed_cost.saturating_add(cost)
        }
        if let Some(fields) = chosen_experiment {
            if self.pending.len() < 32 && goal.is_some() {
                self.pending.push(ExperimentCredit {
                    fields,
                    elapsed_cost: cost,
                });
                if fields[2] > 0.0 {
                    self.experiments = self.experiments.saturating_add(1);
                }
            }
        }
        if goal.is_some_and(|g| matches_goal(post, g)) {
            self.finish_episode(true);
            self.goal = None;
            self.episode_cost = 0;
        }
    }
    pub fn take_credits(&mut self) -> Vec<([f32; 5], f64)> {
        std::mem::take(&mut self.credits)
    }
    pub fn observe(&mut self, t: &Transition, learning: bool) {
        if t.restart {
            self.context.clear()
        }
        // Routing observes the factual POST only after execution. All values
        // used here were read before updating ANY learned table or score.
        // Inference remains live when learning is frozen, like context history.
        let inquiry_forecasts = self.inquiry_factual_forecasts(t);
        let trigger = inquiry_forecasts.as_ref().and_then(|forecasts| {
            let bank = self.regime.as_ref().map_or(0, |r| r.bank);
            let forecast = &forecasts[bank];
            let (predicted_post, probability) = forecast.outcomes.iter().max_by(|a,b| a.1.total_cmp(&b.1))?;
            let observed = forecast.probability(&t.post);
            if forecast.support < 8 || *probability < 0.75 || observed >= 0.25 || *predicted_post == t.post { return None; }
            Some(InquiryQuestion {id: t.id, opened_event: t.id, triggering_action: t.action,
                triggering_pre: t.pre.clone(), predicted_post: predicted_post.clone(), actual_post: t.post.clone(),
                prior_bank: bank, prediction_probability: *probability, observed_probability: observed,
                support: forecast.support, context_fingerprint: super::vm::fingerprint(&serde_json::to_vec(&self.context).unwrap()),
                age: 0, evidence_events: 0, factual_experiments: 0, log_bayes_factor: 0.0,
                candidate_bank: None, candidate_confirmations: 0})
        });
        let regime_probabilities = self.infer_regime(t, learning);
        if let (Some(state), Some(forecasts), Some(regime)) = (self.inquiry.as_mut(), inquiry_forecasts, self.regime.as_ref()) {
            state.observe(t.id, t.action, &t.post, &forecasts, regime.bank, (1.0 + regime.readout.abs()) / 2.0, trigger);
        }
        if !learning {
            self.context.push(ContextStep {
                pre: t.pre.clone(),
                action: t.action,
            });
            if self.context.len() > self.config.history_depth {
                self.context.remove(0);
            }
            self.events = self.events.saturating_add(1);
            return;
        }
        // Score EVERY hypothesis on a prediction committed before this POST.
        let depths = (0..=self.config.history_depth)
            .map(|d| d.min(self.context.len()))
            .collect::<Vec<_>>();
        let probabilities = regime_probabilities.unwrap_or_else(|| depths
            .iter()
            .map(|&d| {
                self.counts
                    .get(&key(&t.pre, t.action, &self.context, d))
                    .map_or(0.5, |c| c.probability(&t.post))
            })
            .collect::<Vec<_>>());
        for (score, p) in self.scores.iter_mut().zip(probabilities) {
            *score = (0.995 * *score + p.max(1e-12).ln()).max(-1e9)
        }
        let high = self
            .scores
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        for score in &mut self.scores {
            *score -= high
        }
        for depth in 0..=self.config.history_depth.min(self.context.len()) {
            let k = self.active_key(&t.pre, t.action, &self.context, depth);
            let c = self.counts.entry(k.clone()).or_insert(Counts {
                outcomes: Vec::new(),
                other: 0,
                last: t.id,
                depth,
            });
            c.age();
            if let Some(o) = c.outcomes.iter_mut().find(|o| o.post == t.post) {
                o.count = o.count.saturating_add(1)
            } else if c.outcomes.len() < self.config.outcomes_per_entry {
                c.outcomes.push(Outcome {
                    post: t.post.clone(),
                    count: 1,
                })
            } else {
                // Bounded frequent-items admission. Demoted observations stay
                // as unknown mass; a recurring new outcome can gain a slot.
                c.other = c.other.saturating_add(c.outcomes.len() as u64 + 1);
                for outcome in &mut c.outcomes {
                    outcome.count -= 1;
                }
                c.outcomes.retain(|outcome| outcome.count > 0);
                if c.outcomes.len() < self.config.outcomes_per_entry {
                    c.other -= 1;
                    c.outcomes.push(Outcome {
                        post: t.post.clone(),
                        count: 1,
                    });
                }
            }
            c.last = t.id;
        }
        self.context.push(ContextStep {
            pre: t.pre.clone(),
            action: t.action,
        });
        if self.context.len() > self.config.history_depth {
            self.context.remove(0);
        }
        self.events = self.events.saturating_add(1);
        self.learned_events = self.learned_events.saturating_add(1);
        self.bytes = self
            .counts
            .iter()
            .map(|(k, c)| 2 * k.len() + c.bytes())
            .sum::<usize>() + self.regime.as_ref().map_or(0, |_| REGIME_STATE_BYTES)
                + self.inquiry.as_ref().map_or(0, |_| super::inquiry::STATE_BYTES);
        while self.counts.len() > self.config.model_entries || self.bytes > self.config.model_bytes
        {
            // Keep frequent evidence and counterexamples; expire least useful
            // old contexts, never erase the actor's external cost/audit debt.
            let weights = self.weights();
            let value = |k: &String, c: &Counts| {
                weights[c.depth].sqrt()
                    * (c.total() as f64).sqrt()
                    * (1.0 + (c.outcomes.len() as f64).ln())
                    / (2 * k.len() + c.bytes()).max(1) as f64
                    / (1.0 + t.id.saturating_sub(c.last) as f64 / 256.0)
            };
            // Reserve probation capacity for new evidence. Otherwise every
            // count-one insertion loses immediately to an established cache
            // and can never become a recurring learned law.
            let mut probation_count = 0usize;
            let mut probation_bytes = 0usize;
            let mut largest = 0usize;
            let probation = |c: &Counts| {
                c.total() < 4
                    || t.id.saturating_sub(c.last) as f64
                        > 64.0 * (1.0 + (c.total().min(512) as f64).sqrt())
            };
            for (k, c) in &self.counts {
                if probation(c) {
                    let bytes = 2 * k.len() + c.bytes();
                    probation_count += 1;
                    probation_bytes += bytes;
                    largest = largest.max(bytes);
                }
            }
            let byte_quota = (self.config.model_bytes / 4)
                .max(largest.saturating_mul(2))
                .min(self.config.model_bytes);
            let trim_probation =
                probation_count > self.config.model_entries / 4 || probation_bytes > byte_quota;
            let victim = self
                .counts
                .iter()
                .filter(|(_, c)| probation(c) == trim_probation)
                .min_by(|(ka, a), (kb, b)| {
                    if trim_probation {
                        a.last
                            .cmp(&b.last)
                            .then_with(|| value(ka, a).total_cmp(&value(kb, b)))
                    } else {
                        value(ka, a)
                            .total_cmp(&value(kb, b))
                            .then_with(|| a.last.cmp(&b.last))
                    }
                })
                .map(|(k, _)| k.clone())
                .or_else(|| {
                    self.counts
                        .iter()
                        .min_by_key(|(_, c)| c.last)
                        .map(|(k, _)| k.clone())
                });
            let Some(k) = victim else { break };
            if let Some(c) = self.counts.remove(&k) {
                self.bytes = self.bytes.saturating_sub(2 * k.len() + c.bytes());
                self.evicted = self.evicted.saturating_add(1);
            }
        }
    }
    fn beliefs_at(
        &self, pre: &[i64], context: &[ContextStep], offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>, meter: &mut Meter,
    ) -> Result<Vec<ActionBelief>> {
        self.beliefs_at_bank(pre, context, offers, goal, meter, self.regime.as_ref().map_or(0, |r| r.bank))
    }
    fn beliefs_at_bank(
        &self,
        pre: &[i64],
        context: &[ContextStep],
        offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>,
        meter: &mut Meter,
        bank: usize,
    ) -> Result<Vec<ActionBelief>> {
        let weights = self.weights();
        let mut all = Vec::new();
        // The relevance term measures disagreement about a GOAL decision,
        // not disagreement about irrelevant sensor noise.
        let mut preferred = Vec::new();
        for depth in 0..weights.len() {
            let d = depth.min(context.len());
            let mut best = (0.0, None);
            for (a, offer) in offers.iter().enumerate().filter(|(_, o)| o.available) {
                let _ = offer;
                meter.charge((pre.len().saturating_mul(d + 1) + d + 4) as u64)?;
                if let Some(c) = self.counts.get(&regime_key(pre, a, context, d, bank)) {
                    let value = c
                        .distribution()
                        .iter()
                        .map(|(p, w)| w * goal.map_or(0.0, |g| f64::from(matches_goal(p, g))))
                        .sum::<f64>();
                    if value > best.0 {
                        best = (value, Some(a))
                    }
                }
            }
            preferred.push(best);
        }
        let mut choices = BTreeMap::<Option<usize>, f64>::new();
        for (w, (_, a)) in weights.iter().zip(&preferred) {
            *choices.entry(*a).or_default() += w
        }
        let decision_disagreement =
            entropy(choices.values().copied()) / (weights.len() as f64).ln().max(1.0);
        for (action, _) in offers.iter().enumerate().filter(|(_, o)| o.available) {
            let mut mixture = BTreeMap::<Vec<i64>, f64>::new();
            let mut components = Vec::new();
            let mut support = 0;
            for (depth, &weight) in weights.iter().enumerate() {
                let mut d = depth.min(context.len());
                let c = loop {
                    meter.charge((pre.len().saturating_mul(d + 1) + d + 4) as u64)?;
                    if let Some(c) = self.counts.get(&regime_key(pre, action, context, d, bank)) {
                        break Some(c);
                    }
                    if d == 0 {
                        break None;
                    }
                    d -= 1;
                };
                if let Some(c) = c {
                    support = support.max(c.total());
                    let distribution = c.distribution();
                    for (post, p) in &distribution {
                        *mixture.entry(post.clone()).or_default() += weight * p
                    }
                    components.push((weight, distribution));
                }
            }
            let mass = components.iter().map(|(w, _)| *w).sum::<f64>();
            if mass == 0.0 {
                all.push(ActionBelief {
                    action,
                    outcomes: Vec::new(),
                    information_gain: 0.0,
                    goal_relevance: 0.0,
                    support: 0,
                    unknown_probability: 1.0,
                });
                continue;
            }
            for p in mixture.values_mut() {
                *p /= mass
            }
            let unknown = (1.0 - mixture.values().sum::<f64>()).max(0.0);
            let conditional = components
                .iter()
                .map(|(w, p)| {
                    w / mass
                        * entropy(p.iter().map(|(_, p)| *p).chain(std::iter::once(
                            (1.0 - p.iter().map(|(_, p)| p).sum::<f64>()).max(0.0),
                        )))
                })
                .sum::<f64>();
            let gain = (entropy(mixture.values().copied().chain(std::iter::once(unknown)))
                - conditional)
                .max(0.0);
            let relevance = if goal.is_none() {
                1.0
            } else {
                decision_disagreement
            };
            all.push(ActionBelief {
                action,
                outcomes: mixture.into_iter().collect(),
                information_gain: gain,
                goal_relevance: relevance,
                support,
                unknown_probability: unknown,
            });
        }
        Ok(all)
    }
    pub fn beliefs(
        &self,
        pre: &[i64],
        offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>,
        work: u64,
    ) -> Result<(Vec<ActionBelief>, u64)> {
        let mut meter = Meter::new(work);
        let b = self.beliefs_at(pre, &self.context, offers, goal, &mut meter)?;
        Ok((b, meter.spent))
    }
    fn planning_beliefs_at(&self, pre: &[i64], context: &[ContextStep], offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>, prior: [f64; 2], meter: &mut Meter,
    ) -> Result<(Vec<ActionBelief>, Vec<f64>, Vec<Vec<InquiryPrediction>>)> {
        let mut diagnostic = vec![0.0; offers.len()];
        let mut forecasts = vec![Vec::new(); offers.len()];
        if self.active_inquiry().is_none() {
            return Ok((self.beliefs_at(pre, context, offers, goal, meter)?, diagnostic, forecasts));
        }
        let zero = self.beliefs_at_bank(pre, context, offers, goal, meter, 0)?;
        let one = self.beliefs_at_bank(pre, context, offers, goal, meter, 1)?;
        let goal_value = |b: &ActionBelief| b.outcomes.iter()
            .filter(|(p, _)| goal.is_some_and(|g| matches_goal(p,g))).map(|(_,p)| p).sum::<f64>();
        let best = |beliefs: &[ActionBelief]| beliefs.iter().filter(|b| b.support >= 8)
            .map(|b| (b.action, goal_value(b))).filter(|(_,q)| *q > 0.05)
            .max_by(|a,b| a.1.total_cmp(&b.1)).map(|(a,_)| a);
        // Evidence about an unrelated random sensor is not a goal-relevant
        // experiment. Later graph states can still expose a useful contrast.
        let relevant = goal.is_none() || best(&zero) != best(&one)
            || zero.iter().zip(&one).any(|(a,b)| (goal_value(a)-goal_value(b)).abs() >= 0.1);
        let mut all = Vec::new();
        for (a,b) in zero.into_iter().zip(one) {
            meter.charge(((a.outcomes.len()+b.outcomes.len()) * (pre.len()+4) + 32) as u64)?;
            let action = a.action;
            let mut mixture = BTreeMap::<Vec<i64>,f64>::new();
            for (belief,p) in [(&a,prior[0]),(&b,prior[1])] {
                for (post,probability) in &belief.outcomes {
                    *mixture.entry(post.clone()).or_default() += p * probability;
                }
            }
            let unknown = (prior[0]*a.unknown_probability + prior[1]*b.unknown_probability).clamp(0.0,1.0);
            let forecast = |bank:usize, b:&ActionBelief| {
                if b.outcomes.len() > MAX_FORECAST_OUTCOMES || b.support == 0 {
                    InquiryPrediction {bank,support:0,outcomes:Vec::new(),unknown_probability:1.0}
                } else { InquiryPrediction {bank,support:b.support,outcomes:b.outcomes.clone(),unknown_probability:b.unknown_probability} }
            };
            forecasts[action] = vec![forecast(0,&a),forecast(1,&b)];
            if relevant && forecasts[action].iter().all(|p|p.support>=8 && p.unknown_probability<0.05) {
                let conditional = prior[0]*entropy(a.outcomes.iter().map(|(_,p)|*p).chain(std::iter::once(a.unknown_probability)))
                    + prior[1]*entropy(b.outcomes.iter().map(|(_,p)|*p).chain(std::iter::once(b.unknown_probability)));
                diagnostic[action] = (entropy(mixture.values().copied().chain(std::iter::once(unknown))) - conditional)
                    .clamp(0.0,std::f64::consts::LN_2);
            }
            all.push(ActionBelief {action,outcomes:mixture.into_iter().collect(),
                information_gain:prior[0]*a.information_gain+prior[1]*b.information_gain,
                goal_relevance:prior[0]*a.goal_relevance+prior[1]*b.goal_relevance,
                support:a.support.max(b.support),unknown_probability:unknown});
        }
        Ok((all,diagnostic,forecasts))
    }
    fn inquiry_decision(&self, action:usize, direct:f64, expected:f64, value:f64,
        prior:[f64;2], forecasts:&[Vec<InquiryPrediction>]) -> Option<InquiryDecision> {
        let question = self.active_inquiry()?;
        if expected <= 1e-12 || forecasts.get(action)?.len() != 2 { return None; }
        Some(InquiryDecision {question_id:question.id,event:self.events,action,
            immediate_information_gain:direct,expected_information_gain:expected,
            expected_goal_probability:value.clamp(0.0,1.0),bank_probability:prior,
            predictions:forecasts[action].clone()})
    }
    pub(crate) fn propose(
        &self,
        pre: &[i64],
        offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>,
        horizon: usize,
        budget: u64,
    ) -> (Option<CausalProposal>, u64) {
        self.propose_with_cost_limit(pre, offers, goal, horizon, u64::MAX, budget)
    }
    pub(crate) fn propose_with_cost_limit(
        &self,
        pre: &[i64],
        offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>,
        horizon: usize,
        remaining_cost: u64,
        budget: u64,
    ) -> (Option<CausalProposal>, u64) {
        let mut meter = Meter::new(budget);
        let p = if horizon == 0 {
            None
        } else {
            self.propose_inner(pre, offers, goal, horizon, remaining_cost, &mut meter)
        };
        (p, meter.spent)
    }
    fn propose_inner(
        &self,
        pre: &[i64],
        offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>,
        horizon: usize,
        remaining_cost: u64,
        meter: &mut Meter,
    ) -> Option<CausalProposal> {
        let affordable_offers = offers
            .iter()
            .map(|o| ActionOffer {
                available: o.available && o.max_cost <= remaining_cost,
                max_cost: o.max_cost,
            })
            .collect::<Vec<_>>();
        let offers = affordable_offers.as_slice();
        let inquiry_active = self.active_inquiry().is_some();
        let quantize = |p:f64| { let p=(p.clamp(0.025,0.975)*64.0).round()/64.0; [p,1.0-p] };
        let prior = self.active_inquiry().map_or([1.0,0.0], |q| quantize(q.bank_probability()[0]));
        let (initial, initial_diagnostic, initial_forecasts) = self
            .planning_beliefs_at(pre, &self.context, offers, goal, prior, meter).ok()?;
        if initial.iter().all(|b| b.support == 0) {
            return None;
        }
        let mut immediate_goals = vec![0.0; offers.len()];
        let mut immediate_support = vec![0; offers.len()];
        for b in &initial {
            immediate_support[b.action] = b.support;
            for (post, probability) in &b.outcomes {
                if meter.charge((post.len() + 1) as u64).is_err() {
                    return None;
                }
                if goal.is_some_and(|g| matches_goal(post, g)) {
                    immediate_goals[b.action] += probability;
                }
            }
        }
        if goal.is_none() {
            let b = initial.iter().filter(|b| b.support > 0).max_by(|a, b| {
                ((a.information_gain + initial_diagnostic[a.action]) / (1.0 + offers[a.action].max_cost as f64))
                    .total_cmp(&((b.information_gain + initial_diagnostic[b.action]) / (1.0 + offers[b.action].max_cost as f64)))
            })?;
            return Some(CausalProposal {
                action: b.action,
                value: 0.0,
                information_gain: b.information_gain + initial_diagnostic[b.action],
                confidence: b.support as f64 / (b.support as f64 + 2.0),
                expected: None,
                immediate_goals,
                immediate_support,
                inquiry:self.inquiry_decision(b.action,initial_diagnostic[b.action],initial_diagnostic[b.action],
                    0.0,prior,&initial_forecasts),
            });
        }
        let credible_depth = self
            .weights()
            .iter()
            .enumerate()
            .filter(|(_, w)| **w >= 0.02)
            .map(|(d, _)| d)
            .max()
            .unwrap_or(0);
        // A bounded empirical belief graph. Unseen paths back off to shorter
        // hypotheses; all route values remain MODEL predictions.
        // Distinguish path budgets only when they constrain this horizon.
        // This preserves the unconstrained graph and avoids multiplying its
        // states by costs when every possible path is already affordable.
        let cost_bound = !offers
            .iter()
            .filter(|o| o.available)
            .map(|o| o.max_cost)
            .max()
            .unwrap_or(0)
            .checked_mul(horizon as u64)
            .is_some_and(|cost| cost <= remaining_cost);
        let graph_key = |state: &[i64], context: &[ContextStep], left: u64, belief:[f64;2]| {
            let k = key(state, 0, context, context.len());
            let k = if cost_bound {
                format!("{k}:{left}")
            } else {
                k
            };
            if inquiry_active { format!("{k}:b{}",(belief[0]*64.0).round() as u32) } else { k }
        };
        let mut states = vec![(pre.to_vec(), self.context.clone(), remaining_cost, prior)];
        let mut edges = Vec::new();
        let mut keys = BTreeMap::new();
        keys.insert(graph_key(pre, &self.context, remaining_cost, prior), 0usize);
        let mut pos = 0;
        let mut layers = vec![0usize];
        // Reserve the actual backup/readout work for every admitted graph row.
        // An exhausted speculative expansion must not erase the root forecast.
        // The legacy route keeps its original meter and byte-identical choices.
        let full_limit = meter.limit;
        let backup_work = |n:usize| (3*n + 8) as u64;
        let mut reserved = if inquiry_active { initial.iter().map(|b|
            backup_work(b.outcomes.len()).saturating_mul(horizon.saturating_sub(1) as u64)
                + (b.outcomes.len()*(pre.len()+3)+64) as u64).sum::<u64>() } else {0};
        if inquiry_active {
            let limit = full_limit.saturating_sub(reserved);
            if meter.spent > limit { return None; }
            meter.limit = limit;
        }
        'expansion: while pos < states.len() {
            if meter.remaining() < 512 {
                break;
            }
            let (state, context, left, belief_prior) = states[pos].clone();
            let local_offers = offers
                .iter()
                .map(|o| ActionOffer {
                    available: o.available && o.max_cost <= left,
                    max_cost: o.max_cost,
                })
                .collect::<Vec<_>>();
            let (b, diagnostic, forecasts) = if pos == 0 {
                (initial.clone(),initial_diagnostic.clone(),initial_forecasts.clone())
            } else {
                match self.planning_beliefs_at(&state, &context, &local_offers, goal, belief_prior, meter) {
                    Ok(v) => v,
                    Err(_) => break,
                }
            };
            if inquiry_active && pos > 0 {
                let extra = b.iter().map(|b| backup_work(b.outcomes.len())
                    .saturating_mul(horizon.saturating_sub(1) as u64)).sum::<u64>();
                let limit = full_limit.saturating_sub(reserved.saturating_add(extra));
                if meter.spent > limit { break; }
                reserved = reserved.saturating_add(extra);
                meter.limit = limit;
            }
            let mut row = Vec::new();
            for belief in b {
                if !local_offers[belief.action].available {
                    continue;
                }
                let next_left = left - offers[belief.action].max_cost;
                let mut outcomes = Vec::new();
                for (post, p) in &belief.outcomes {
                    if meter.charge((post.len() + 4) as u64).is_err() {
                        if inquiry_active { break 'expansion; } else { return None; }
                    }
                    let reached = goal.is_some_and(|g| matches_goal(post, g));
                    let mut next_context = context.clone();
                    next_context.push(ContextStep {
                        pre: state.clone(),
                        action: belief.action,
                    });
                    if next_context.len() > self.config.history_depth {
                        next_context.remove(0);
                    }
                    // Only distinguish histories which have acquired support.
                    let mut depth = next_context.len().min(credible_depth);
                    if inquiry_active {
                        while depth > 0 {
                            let mut supported = false;
                            'support: for action in 0..offers.len() { for bank in 0..2 {
                                if meter.charge((post.len()*(depth+1)+depth+4) as u64).is_err() { break 'expansion; }
                                if self.counts.contains_key(&regime_key(post,action,&next_context,depth,bank)) {
                                    supported=true; break 'support;
                                }
                            }}
                            if supported { break; }
                            depth -= 1;
                        }
                    } else {
                        while depth > 0 && !(0..offers.len()).any(|a|
                            self.counts.contains_key(&self.active_key(post,a,&next_context,depth))) {
                            depth -= 1;
                        }
                    }
                    if depth < next_context.len() {
                        next_context = next_context[next_context.len() - depth..].to_vec()
                    }
                    if inquiry_active {
                        let work = forecasts[belief.action].iter().map(|f| f.outcomes.len()*(post.len()+4)+8)
                            .sum::<usize>() + post.len()*(next_context.len()+1)+next_context.len()+8;
                        if meter.charge(work as u64).is_err() { break 'expansion; }
                    }
                    let next_belief = if inquiry_active && forecasts[belief.action].iter()
                        .all(|f| f.support >= 8 && f.unknown_probability < 0.05) {
                        let f = &forecasts[belief.action];
                        let p0 = belief_prior[0] * f[0].probability(post);
                        let p1 = belief_prior[1] * f[1].probability(post);
                        quantize(p0 / (p0+p1).max(1e-12))
                    } else { belief_prior };
                    let k = graph_key(post, &next_context, next_left, next_belief);
                    let target = if reached {
                        None
                    } else if let Some(&id) = keys.get(&k) {
                        Some(id)
                    } else if states.len() < self.config.planning_states
                        && layers[pos] + 1 < horizon
                    {
                        let id = states.len();
                        keys.insert(k, id);
                        states.push((post.clone(), next_context, next_left, next_belief));
                        layers.push(layers[pos] + 1);
                        Some(id)
                    } else {
                        None
                    };
                    outcomes.push((target, *p, reached));
                }
                let direct = diagnostic[belief.action];
                row.push((belief, outcomes, direct));
            }
            edges.push(row);
            pos += 1;
        }
        meter.limit = full_limit;
        let mut values = vec![0.0_f64; states.len()];
        let mut information_values = vec![0.0_f64; states.len()];
        let mut policies = vec![None; states.len()];
        // The root action below consumes one step of the stated horizon.
        for _ in 1..horizon {
            let mut next = values.clone();
            let mut next_information = information_values.clone();
            for (i, row) in edges.iter().enumerate() {
                let mut best = 0.0_f64;
                for (b, outcomes, direct) in row {
                    if meter.charge(if inquiry_active {backup_work(outcomes.len())} else {(outcomes.len()+2) as u64}).is_err() {
                        break;
                    }
                    let goal_value = outcomes
                        .iter()
                        .map(|(id, p, goal)| {
                            p * if *goal {
                                1.0
                            } else {
                                id.map_or(0.0, |j| 0.98 * values[j])
                            }
                        })
                        .sum::<f64>();
                    let diagnostic = if inquiry_active { (*direct).max(outcomes.iter().map(|(id,p,terminal)|
                        if *terminal {0.0} else {p*id.map_or(0.0,|j|0.98*information_values[j])}).sum::<f64>()) } else {0.0};
                    let q = goal_value - 0.01 * offers[b.action].max_cost as f64
                        + if inquiry_active {0.2*diagnostic/(1.0+offers[b.action].max_cost as f64)} else {0.0};
                    if q > best {
                        best = q;
                        policies[i] = Some(b.action);
                        if inquiry_active {
                            next[i] = goal_value;
                            next_information[i] = diagnostic;
                        }
                    }
                }
                if !inquiry_active { next[i] = best; }
            }
            values = next;
            information_values = next_information;
        }
        let mut best = None::<(f64, &ActionBelief, f64, f64, f64, f64)>;
        for (b, outcomes, direct) in edges.first()? {
            meter
                .charge((outcomes.len() * pre.len() + 48
                    + if inquiry_active {3*outcomes.len()+16} else {0}) as u64)
                .ok()?;
            let q = outcomes
                .iter()
                .map(|(id, p, goal)| {
                    p * if *goal {
                        1.0
                    } else {
                        id.map_or(0.0, |j| 0.98 * values[j])
                    }
                })
                .sum::<f64>();
            // Missing laws must remain testable even when every currently
            // known route has zero goal value. The parameter-information prior
            // decays on actual outcomes, including pure noise. Never put this
            // optimistic learning value into the predicted GOAL field.
            let frontier = if q < 0.05 {
                goal_law_information(b.support, immediate_goals[b.action])
            } else {
                0.0
            };
            let diagnostic = if inquiry_active { (*direct).max(outcomes.iter().map(|(id,p,terminal)|
                if *terminal {0.0} else {p*id.map_or(0.0,|j|0.98*information_values[j])}).sum::<f64>()) } else {0.0};
            let raw_information = b.information_gain * b.goal_relevance + frontier + diagnostic;
            let info = raw_information / (1.0 + offers[b.action].max_cost as f64);
            let score = q + 0.2 * info - 0.01 * offers[b.action].max_cost as f64;
            let mut decisions = BTreeMap::<usize, f64>::new();
            for (target, p, terminal) in outcomes {
                if !terminal {
                    if let Some(j) = target {
                        if values[*j] > 0.0 {
                            if let Some(a) = policies[*j] {
                                *decisions.entry(a).or_default() += p;
                            }
                        }
                    }
                }
            }
            let mass = decisions.values().sum::<f64>();
            let decision_information = if decisions.len() > 1 && mass > 0.0 {
                mass * entropy(decisions.values().map(|p| p / mass))
            } else {
                0.0
            };
            if best.is_none_or(|(v, _, _, _, _, _)| score > v) {
                best = Some((score, b, decision_information, q, raw_information, diagnostic))
            }
        }
        let (_, b, decision_information, goal_value, raw_information, diagnostic) = best?;
        Some(CausalProposal {
            action: b.action,
            value: goal_value.clamp(0.0, 1.0),
            information_gain: raw_information.max(decision_information),
            confidence: b.support as f64 / (b.support as f64 + 2.0),
            expected: b
                .outcomes
                .iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(p, _)| p.clone()),
            immediate_goals,
            immediate_support,
            inquiry:self.inquiry_decision(b.action,initial_diagnostic[b.action],diagnostic,
                goal_value,prior,&initial_forecasts),
        })
    }
    pub fn validate(
        &self,
        width: usize,
        actions: usize,
        next_id: u64,
        model: u64,
        cells: usize,
    ) -> Result<()> {
        self.config.validate()?;
        if let Some(regime) = &self.regime {
            if self.config.history_depth != 3 { return Err(Error::InvalidData); }
            regime.validate(self.events)?;
        }
        if let Some(inquiry) = &self.inquiry {
            if !self.regime.as_ref().is_some_and(|r| r.mode == RegimeMode::Ordinary) {
                return Err(Error::InvalidData);
            }
            inquiry.validate(width, actions, self.events)?;
        }
        if self.scores.len() != self.config.history_depth + 1
            || self
                .scores
                .iter()
                .any(|s| !s.is_finite() || *s > 1e-9 || *s < -1e9)
            || self.context.len() > self.config.history_depth
            || self
                .context
                .iter()
                .any(|c| c.pre.len() != width || c.action >= actions)
            || self.counts.len() > self.config.model_entries
            || self.bytes > self.config.model_bytes
            || self.events != next_id
            || self.learned_events > self.events
            || self.offset > next_id
            || self.seed_model != model
            || self.seed_state.len() != cells
            || self.pending.len() > 32
            || self.credits.len() > 128
            || self.goal.as_ref().is_some_and(|g| g.len() != width)
        {
            return Err(Error::InvalidData);
        }
        let mut bytes = self.regime.as_ref().map_or(0, |_| REGIME_STATE_BYTES)
            + self.inquiry.as_ref().map_or(0, |_| super::inquiry::STATE_BYTES);
        for (k, c) in &self.counts {
            let (bank, original) = if let Some(original) = k.strip_prefix("r1:") {
                if !self.regime.as_ref().is_some_and(|r| r.admitted) { return Err(Error::InvalidData); }
                (1, original)
            } else { (0, k.as_str()) };
            let (pre, action, context): (Vec<i64>, usize, Vec<ContextStep>) =
                serde_json::from_str(original).map_err(|_| Error::InvalidData)?;
            if c.depth != context.len()
                || regime_key(&pre, action, &context, context.len(), bank) != *k
                || pre.len() != width
                || action >= actions
                || context.len() > self.config.history_depth
                || context
                    .iter()
                    .any(|s| s.pre.len() != width || s.action >= actions)
                || c.last >= next_id
                || c.outcomes.is_empty()
                || c.outcomes.len() > self.config.outcomes_per_entry
                || c.outcomes
                    .iter()
                    .any(|o| o.post.len() != width || o.count == 0)
                || c.total() > next_id
            {
                return Err(Error::InvalidData);
            }
            let unique = c
                .outcomes
                .iter()
                .map(|o| &o.post)
                .collect::<std::collections::BTreeSet<_>>();
            if unique.len() != c.outcomes.len() {
                return Err(Error::InvalidData);
            }
            bytes += 2 * k.len() + c.bytes();
        }
        if bytes != self.bytes
            || self.pending.iter().any(|c| {
                c.fields
                    .iter()
                    .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
            })
            || self.credits.iter().any(|(f, u)| {
                f.iter().any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
                    || !u.is_finite()
                    || !(0.0..=1.0).contains(u)
            })
        {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    pub fn compacted(&mut self, n: usize, cost: u64, model: u64, seed: Vec<i64>, digest: u64) {
        self.offset = self.offset.saturating_add(n as u64);
        self.compacted_cost = self.compacted_cost.saturating_add(cost);
        self.seed_model = model;
        self.seed_state = seed;
        self.journal_digest = digest;
        self.compactions = self.compactions.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inquiry_broad_noise_is_not_a_confident_contradiction() {
        let mut m=CausalMemory::new(CausalConfig{history_depth:3,..Default::default()},0,vec![]).unwrap();
        for i in 0..64 { fact(&mut m,0,0,10+i%8,true); }
        m.enable_autonomous_inquiry(InquiryMode::Active).unwrap();
        for i in 0..64 { fact(&mut m,0,0,10+i%8,true); }
        assert_eq!(m.inquiry_report().unwrap().questions_opened,0,
            "low probability in a broad distribution is not a violated confident forecast");
        m.validate(1,1,m.events,0,0).unwrap();
    }
    #[test]
    fn inquiry_observe_only_has_exact_ordinary_decisions_and_learning() {
        let base=regime_fixture();
        let mut ordinary=base.clone();
        ordinary.enable_regime_inference(RegimeMode::Ordinary).unwrap();
        let mut observer=base;
        observer.enable_autonomous_inquiry(InquiryMode::ObserveOnly).unwrap();
        let offers=[ActionOffer{available:true,max_cost:1};2];
        for post in [2,2,2,2,2,2,2,2,2,2,1,1,1,1,1,1] {
            let (a,wa)=ordinary.propose(&[0],&offers,Some(&[Some(1)]),3,100_000);
            let (b,wb)=observer.propose(&[0],&offers,Some(&[Some(1)]),3,100_000);
            assert_eq!(format!("{a:?}"),format!("{b:?}"));
            assert_eq!(wa,wb);
            fact(&mut ordinary,0,0,post,true); fact(&mut observer,0,0,post,true);
            assert_eq!(ordinary.report().learned_fingerprint,observer.report().learned_fingerprint);
            assert_eq!(ordinary.regime_report().unwrap().dynamic_fingerprint,
                observer.regime_report().unwrap().dynamic_fingerprint);
        }
        assert!(observer.inquiry_report().unwrap().questions_opened>0);
        assert_eq!(observer.inquiry_report().unwrap().selected_experiments,0);
    }
    #[test]
    fn inquiry_unresolved_question_persists_across_restart_and_expires_honestly() {
        let mut m=regime_fixture();
        m.enable_autonomous_inquiry(InquiryMode::Active).unwrap();
        let model=m.report().learned_fingerprint;
        let t=Transition{id:m.events,restart:true,pre:vec![0],action:0,post:vec![2]};
        m.observe(&t,false);
        let id=m.inquiry_report().unwrap().active.unwrap().id;
        m.reset(false);
        assert_eq!(m.inquiry_report().unwrap().active.unwrap().id,id);
        for _ in 0..super::super::inquiry::MAX_AGE {
            let t=Transition{id:m.events,restart:true,pre:vec![0],action:0,post:vec![1]};
            m.observe(&t,false);
        }
        let r=m.inquiry_report().unwrap();
        assert!(r.active.is_none()); assert_eq!(r.questions_resolved,0); assert_eq!(r.questions_expired,1);
        let ended=r.last_resolution.unwrap(); assert_eq!(ended.status,"expired"); assert_eq!(ended.bank,None);
        assert_eq!(m.report().learned_fingerprint,model);
        m.validate(1,1,m.events,0,0).unwrap();
    }
    #[test]
    fn inquiry_partial_planning_keeps_a_readout_without_exceeding_budget() {
        let mut m=regime_fixture();
        m.enable_autonomous_inquiry(InquiryMode::Active).unwrap();
        fact(&mut m,0,0,2,true); m.reset(false);
        let before=serde_json::to_vec(&m).unwrap();
        let offers=[ActionOffer{available:true,max_cost:1};2];
        for budget in [1200,2400,4800] {
            let (p,work)=m.propose(&[0],&offers,Some(&[Some(1)]),4,budget);
            assert!(p.is_some(),"root readout vanished at budget {budget}, work {work}");
            assert!(work<=budget);
            assert!(p.unwrap().value<=0.5,"unknown context cannot inherit a certain old success");
        }
        assert_eq!(serde_json::to_vec(&m).unwrap(),before,"imagined branches are never factual inquiry evidence");
    }
    #[test]
    fn inquiry_prospectively_discriminates_factual_rivals_and_resolves_after_execution() {
        let mut m=CausalMemory::new(CausalConfig{history_depth:3,..Default::default()},0,vec![]).unwrap();
        for i in 0..32 {
            fact(&mut m,0,0,1,true); fact(&mut m,0,1,2,true); fact(&mut m,0,2,3+i%2,true);
        }
        m.enable_autonomous_inquiry(InquiryMode::Active).unwrap();
        for i in 0..32 {
            fact(&mut m,0,0,2,true); fact(&mut m,0,1,1,true); fact(&mut m,0,2,3+i%2,true);
        }
        assert_eq!(m.regime_report().unwrap().bank,1);
        assert!(m.inquiry_report().unwrap().active.is_none());
        fact(&mut m,0,0,1,true); m.reset(false);
        let question=m.inquiry_report().unwrap().active.unwrap();
        let offers=[ActionOffer{available:true,max_cost:1};3];
        let before=serde_json::to_vec(&m).unwrap();
        let (p,work)=m.propose(&[0],&offers,Some(&[Some(1)]),3,100_000);
        let p=p.unwrap(); let decision=p.inquiry.unwrap();
        assert_ne!(p.action,2,"balanced noise cannot discriminate the factual contexts");
        assert!(decision.immediate_information_gain>0.01);
        assert_eq!(decision.question_id,question.id);
        assert!(decision.predictions.iter().all(|p|p.support>=8));
        assert!(work<=100_000);
        assert_eq!(serde_json::to_vec(&m).unwrap(),before);
        m.record_inquiry_decision(p.action,Some(&decision)).unwrap();
        assert_eq!(m.inquiry_report().unwrap().factual_experiments,0);
        fact(&mut m,0,p.action,if p.action==0{1}else{2},true);
        let r=m.inquiry_report().unwrap();
        assert_eq!(r.selected_experiments,1); assert_eq!(r.factual_experiments,1);
        let resolution=r.last_resolution.unwrap();
        assert_eq!(resolution.question_id,question.id); assert_eq!(resolution.status,"resolved");
        assert_eq!(resolution.bank,Some(0)); assert_eq!(resolution.factual_experiments,1);
        m.validate(1,3,m.events,0,0).unwrap();
        let encoded=serde_json::to_vec(&m).unwrap();
        let restored:CausalMemory=serde_json::from_slice(&encoded).unwrap();
        restored.validate(1,3,restored.events,0,0).unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(),encoded);
    }
    fn regime_fixture() -> CausalMemory {
        let mut memory = CausalMemory::new(CausalConfig { history_depth: 3, ..Default::default() }, 0, vec![]).unwrap();
        for _ in 0..32 { fact(&mut memory, 0, 0, 1, true); }
        memory
    }
    const REGIME_MODES: [RegimeMode; 4] = [RegimeMode::Ordinary, RegimeMode::Phase,
        RegimeMode::PhaseScrambled, RegimeMode::PhaseUncoupled];

    #[test]
    fn regime_enable_preserves_warm_model_and_legacy_encoding() {
        let legacy = regime_fixture();
        let bytes = serde_json::to_vec(&legacy).unwrap();
        assert!(!serde_json::from_slice::<serde_json::Value>(&bytes).unwrap().as_object().unwrap().contains_key("regime"));
        let restored: CausalMemory = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        for mode in REGIME_MODES {
            let mut memory = legacy.clone();
            let counts = serde_json::to_vec(&memory.counts).unwrap();
            let before = memory.report();
            memory.enable_regime_inference(mode).unwrap();
            assert_eq!(serde_json::to_vec(&memory.counts).unwrap(), counts);
            assert_eq!(memory.scores, legacy.scores);
            assert_eq!(memory.events, legacy.events);
            assert_eq!(memory.learned_events, legacy.learned_events);
            assert_eq!(memory.report().learned_fingerprint, before.learned_fingerprint);
            assert_eq!(memory.report().model_bytes, before.model_bytes + REGIME_STATE_BYTES);
            assert_eq!(memory.enable_regime_inference(mode), Err(Error::Busy));
            memory.validate(1, 1, memory.events, 0, 0).unwrap();
            let encoded = serde_json::to_vec(&memory).unwrap();
            let restored: CausalMemory = serde_json::from_slice(&encoded).unwrap();
            restored.validate(1, 1, restored.events, 0, 0).unwrap();
            assert_eq!(serde_json::to_vec(&restored).unwrap(), encoded);
        }
    }

    #[test]
    fn regime_birth_is_shared_supported_and_factual() {
        let mut fingerprints = Vec::new();
        for mode in REGIME_MODES {
            let mut memory = regime_fixture();
            memory.enable_regime_inference(mode).unwrap();
            fact(&mut memory, 0, 0, 2, true);
            assert!(!memory.regime.as_ref().unwrap().admitted);
            assert_eq!(memory.regime.as_ref().unwrap().birth_surprises, 1);
            // A diagnostic read is not new evidence and cannot admit a bank.
            let before = serde_json::to_vec(&memory).unwrap();
            memory.beliefs(&[0], &[ActionOffer { available: true, max_cost: 1 }], None, 100_000).unwrap();
            assert_eq!(serde_json::to_vec(&memory).unwrap(), before);
            fact(&mut memory, 0, 0, 2, true);
            let regime = memory.regime.as_ref().unwrap();
            assert!(regime.admitted);
            assert_eq!(regime.bank, 1);
            assert_eq!(regime.switches, 1);
            assert_eq!(regime.extra_inference_work, 2 * REGIME_UPDATE_WORK);
            assert_eq!(memory.counts[&regime_key(&[0], 0, &[], 0, 1)].total(), 1);
            // The forecast scored on the triggering POST came from bank zero;
            // only its subsequent learning assignment uses the admitted bank.
            assert!(regime.likelihoods.iter().all(|p| p[0] < 0.25 && p[1] == 0.5));
            assert!(regime.supports.iter().all(|n| n[0] == 33 && n[1] == 0));
            fingerprints.push(memory.report().learned_fingerprint);
            memory.validate(1, 1, memory.events, 0, 0).unwrap();
        }
        assert!(fingerprints.iter().all(|f| *f == fingerprints[0]));

        let mut memory = regime_fixture();
        memory.enable_regime_inference(RegimeMode::Ordinary).unwrap();
        fact(&mut memory, 0, 0, 2, true);
        fact(&mut memory, 0, 0, 1, true);
        assert_eq!(memory.regime.as_ref().unwrap().birth_surprises, 0);
        fact(&mut memory, 0, 0, 2, true);
        assert!(!memory.regime.as_ref().unwrap().admitted);
    }

    #[test]
    fn regime_frozen_inference_retains_model_and_survives_reset() {
        for mode in REGIME_MODES {
            let mut memory = regime_fixture();
            memory.enable_regime_inference(mode).unwrap();
            fact(&mut memory, 0, 0, 2, true);
            fact(&mut memory, 0, 0, 2, true);
            let model = memory.report();
            let before = memory.regime_report().unwrap();
            let transition = Transition { id: memory.events, restart: true,
                pre: vec![0], action: 0, post: vec![1] };
            memory.observe(&transition, false);
            assert_eq!(memory.report().learned_fingerprint, model.learned_fingerprint);
            assert_eq!(memory.report().learned_transitions, model.learned_transitions);
            assert_eq!(memory.regime_report().unwrap().observations, before.observations + 1);
            assert_ne!(memory.regime_report().unwrap().dynamic_fingerprint, before.dynamic_fingerprint);
            let dynamic = memory.regime_report().unwrap().dynamic_fingerprint;
            memory.reset(false);
            assert_eq!(memory.regime_report().unwrap().dynamic_fingerprint, dynamic);
            memory.validate(1, 1, memory.events, 0, 0).unwrap();
            let encoded = serde_json::to_vec(&memory).unwrap();
            let restored: CausalMemory = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(serde_json::to_vec(&restored).unwrap(), encoded);
            restored.validate(1, 1, restored.events, 0, 0).unwrap();
        }
    }

    #[test]
    fn regime_shared_capacity_and_state_validation() {
        let mut memory = regime_fixture();
        memory.config.model_entries = 32;
        memory.config.model_bytes = 16_384;
        memory.enable_regime_inference(RegimeMode::Phase).unwrap();
        fact(&mut memory, 0, 0, 2, true);
        fact(&mut memory, 0, 0, 2, true);
        for i in 0..192 {
            fact(&mut memory, i % 40, 0, i % 7, true);
            assert!(memory.counts.len() <= 32);
            assert!(memory.bytes <= 16_384);
            assert!(serde_json::to_vec(memory.regime.as_ref().unwrap()).unwrap().len() <= REGIME_STATE_BYTES);
        }
        assert!(memory.evicted > 0);
        memory.validate(1, 1, memory.events, 0, 0).unwrap();
        let mut corrupted = memory.clone();
        corrupted.regime.as_mut().unwrap().observations += 1;
        assert_eq!(corrupted.validate(1, 1, corrupted.events, 0, 0), Err(Error::InvalidData));
        let mut corrupted = memory.clone();
        corrupted.regime.as_mut().unwrap().state[0] = f64::NAN;
        assert_eq!(corrupted.validate(1, 1, corrupted.events, 0, 0), Err(Error::InvalidData));
        let mut corrupted = memory.clone();
        corrupted.bytes -= REGIME_STATE_BYTES;
        assert_eq!(corrupted.validate(1, 1, corrupted.events, 0, 0), Err(Error::InvalidData));
    }

    #[test]
    fn regime_common_rotation_preserves_relative_equations_and_readout() {
        let reference = 0.7;
        let rotation = 2.3;
        let relative = REGIME_INITIAL;
        let rotated_relative = relative.map(|angle| regime_wrap(regime_wrap(angle + reference + rotation)
            - regime_wrap(reference + rotation)));
        let evidence = [-0.8, 0.2, 0.9, -0.3];
        let before = regime_phase_velocity(relative, evidence, 0.35);
        let after = regime_phase_velocity(rotated_relative, evidence, 0.35);
        for i in 0..4 {
            assert!((before[i] - after[i]).abs() < 1e-12);
            assert!((relative[i].cos() - rotated_relative[i].cos()).abs() < 1e-12);
        }
    }

    #[test]
    fn a_full_outcome_set_can_learn_a_new_stable_law_and_later_drift() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        for outcome in 0..16 {
            fact(&mut m, 0, 0, outcome, true);
        }
        for outcome in [99, 1234] {
            for _ in 0..2000 {
                fact(&mut m, 0, 0, outcome, true);
            }
            let beliefs = m
                .beliefs(
                    &[0],
                    &[ActionOffer {
                        available: true,
                        max_cost: 1,
                    }],
                    Some(&[Some(outcome)]),
                    100_000,
                )
                .unwrap()
                .0;
            let probability = beliefs[0]
                .outcomes
                .iter()
                .find(|(post, _)| post == &[outcome])
                .unwrap()
                .1;
            assert!(
                probability > 0.95,
                "the current repeated successor must become usable"
            );
            assert!(
                beliefs[0].unknown_probability > 0.0,
                "forgotten contradictions remain unknown mass"
            );
        }
        assert_eq!(m.report().learned_transitions, 4016);
    }
    #[test]
    fn full_entry_and_byte_caches_admit_a_new_recurring_domain() {
        for (entries, bytes) in [(32, 2 * 1024 * 1024), (1024, 16384)] {
            let mut m = CausalMemory::new(
                CausalConfig {
                    history_depth: 1,
                    model_entries: entries,
                    model_bytes: bytes,
                    ..Default::default()
                },
                0,
                vec![],
            )
            .unwrap();
            if entries == 32 {
                for _ in 0..16 {
                    for pre in 0..32 {
                        fact(&mut m, pre, 0, pre + 1, true);
                    }
                }
            } else {
                for pre in 0..128 {
                    for _ in 0..16 {
                        fact(&mut m, pre, 0, pre + 1, true);
                    }
                }
            }
            // The new working set is larger than the probation reserve.
            // Stale protected entries must eventually yield, too.
            for round in 0..1200 {
                let pre = 1000 + round % 20;
                fact(&mut m, pre, 0, pre + 1, true);
            }
            for pre in 1000..1020 {
                let b = m
                    .beliefs(
                        &[pre],
                        &[ActionOffer {
                            available: true,
                            max_cost: 1,
                        }],
                        None,
                        100_000,
                    )
                    .unwrap()
                    .0;
                assert!(
                    b[0].support >= 4,
                    "recurring context {pre} must accumulate confirmations"
                );
                assert!(b[0].outcomes.iter().any(|(post, _)| post == &[pre + 1]));
            }
            assert!(m.report().retained_entries <= entries && m.report().model_bytes <= bytes);
        }
    }
    #[test]
    fn freezing_finalizes_failure_once_without_erasing_predictive_context() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        fact(&mut m, 0, 0, 1, true);
        let context = m.context.clone();
        let learned = m.report().learned_fingerprint;
        let fields = [0.5; 5];
        m.feedback(Some(&[Some(9)]), &[1], 1, Some(fields), true);
        m.finish_learning();
        assert_eq!(m.report().failed_goal_episodes, 1);
        assert_eq!(m.take_credits(), vec![(fields, 0.0)]);
        assert_eq!(m.context, context);
        assert_eq!(m.report().learned_fingerprint, learned);
        m.finish_learning();
        m.reset(false);
        assert_eq!(m.report().failed_goal_episodes, 1);
        assert!(m.take_credits().is_empty());
    }
    #[test]
    fn heterogeneous_quotes_preserve_cheap_routes_without_exceeding_path_budget() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        for _ in 0..12 {
            for stage in 0..10 {
                fact(&mut m, stage, 0, stage + 1, true);
                fact(&mut m, stage, 1, stage, true);
            }
        }
        m.context.clear();
        let offers = [
            ActionOffer {
                available: true,
                max_cost: 1,
            },
            ActionOffer {
                available: true,
                max_cost: 10,
            },
        ];
        let (plan, work) =
            m.propose_with_cost_limit(&[0], &offers, Some(&[Some(10)]), 10, 10, 200_000);
        let plan = plan.unwrap();
        assert_eq!(plan.action, 0);
        assert!(
            plan.value > 0.5,
            "ten cheap actions fit despite the unrelated expensive offer"
        );
        assert!(work <= 200_000);
        let short = m
            .propose_with_cost_limit(&[0], &offers, Some(&[Some(10)]), 10, 9, 200_000)
            .0
            .unwrap();
        assert_eq!(
            short.value, 0.0,
            "an unaffordable continuation cannot contribute goal value"
        );
    }
    #[test]
    fn an_empty_goal_mask_cannot_complete_an_episode() {
        assert!(!matches_goal(&[42], &[None]));
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        m.feedback(Some(&[None]), &[42], 1, Some([0.5; 5]), true);
        assert_eq!(m.report().completed_goal_episodes, 0);
        assert_eq!(m.report().useful_experiments, 0);
    }
    #[test]
    fn accounted_model_bytes_cover_encoded_extreme_integer_outcomes() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        for id in 0..64 {
            m.observe(
                &Transition {
                    id,
                    restart: true,
                    pre: vec![i64::MIN; 32],
                    action: 0,
                    post: vec![i64::MAX - id as i64; 32],
                },
                true,
            );
        }
        assert!(serde_json::to_vec(&m.counts).unwrap().len() <= m.bytes);
    }
    #[test]
    fn goal_parameter_information_decreases_while_noise_entropy_stays_high() {
        let empty = goal_law_information(0, 0.5);
        assert!((empty - (std::f64::consts::LN_2 - 0.5)).abs() < 1e-7);
        assert!(empty > goal_law_information(10, 0.5));
        assert!(goal_law_information(10, 0.5) > goal_law_information(1000, 0.5));
        assert!(goal_law_information(1_000_000, 0.5) < 1e-5);
    }
    #[test]
    fn an_unknown_law_can_be_tested_without_fabricating_goal_value() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        for _ in 0..30 {
            fact(&mut m, 0, 0, 0, true);
        }
        let offers = [ActionOffer {
            available: true,
            max_cost: 1,
        }; 2];
        let p = m
            .propose(&[0], &offers, Some(&[Some(9)]), 4, 100_000)
            .0
            .unwrap();
        assert_eq!(p.action, 1);
        assert_eq!(
            p.value, 0.0,
            "information is not evidence that the goal will be reached"
        );
        assert!(p.information_gain > 0.1);
        assert_eq!(p.immediate_goals, vec![0.0, 0.0]);
    }
    fn fact(m: &mut CausalMemory, pre: i64, action: usize, post: i64, restart: bool) {
        let t = Transition {
            id: m.events,
            pre: vec![pre],
            action,
            post: vec![post],
            restart,
        };
        m.observe(&t, true);
    }
    #[test]
    fn genuine_conflicts_are_retained_as_probability_not_deleted_edges() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        for k in 0..40 {
            fact(&mut m, 0, 0, if k % 4 == 0 { 7 } else { 9 }, true)
        }
        let b = m
            .beliefs(
                &[0],
                &[ActionOffer {
                    available: true,
                    max_cost: 1,
                }],
                None,
                10000,
            )
            .unwrap()
            .0;
        assert_eq!(b[0].outcomes.len(), 2);
        assert!((b[0].outcomes.iter().find(|(p, _)| p == &[7]).unwrap().1 - 0.25).abs() < 1e-9);
        assert!((b[0].outcomes.iter().find(|(p, _)| p == &[9]).unwrap().1 - 0.75).abs() < 1e-9);
        assert!(
            b[0].information_gain < 1e-9,
            "unpredictable noise alone is not epistemic gain"
        );
    }
    #[test]
    fn information_gain_uses_rival_predictions_and_goal_decisions_not_noise_frequency() {
        let mut m = CausalMemory::new(
            CausalConfig {
                history_depth: 1,
                ..Default::default()
            },
            0,
            vec![],
        )
        .unwrap();
        for round in 0..4 {
            for context in 0..2 {
                for action in 2..5 {
                    fact(&mut m, 0, context, 0, true);
                    let post = if action == 4 {
                        10 + round % 2
                    } else if action - 2 == context {
                        7
                    } else {
                        9
                    };
                    fact(&mut m, 0, action, post as i64, false);
                }
            }
        }
        fact(&mut m, 0, 1, 0, true);
        let b = m
            .beliefs(
                &[0],
                &vec![
                    ActionOffer {
                        available: true,
                        max_cost: 1
                    };
                    5
                ],
                Some(&[Some(7)]),
                10000,
            )
            .unwrap()
            .0;
        eprintln!("CAUSAL_IG {b:?}");
        let goal = b.iter().find(|b| b.action == 2).unwrap();
        let noise = b.iter().find(|b| b.action == 4).unwrap();
        assert!(goal.information_gain > noise.information_gain + 1e-8);
        assert!(
            goal.goal_relevance > 0.0,
            "rivals imply different goal decisions"
        );
        assert!(
            noise.information_gain < 1e-8,
            "balanced stochastic nuisance must not be mistaken for model disagreement"
        );
    }
    #[test]
    fn learned_stochastic_branches_make_a_costly_sensor_worthwhile() {
        // Same acquired constructor: a sensor leads to one of two observed
        // cues, which have different useful motors. No sensor/cue meanings
        // exist in the learner; this test supplies only factual transitions.
        let mut m = CausalMemory::new(
            CausalConfig {
                history_depth: 1,
                ..Default::default()
            },
            0,
            vec![],
        )
        .unwrap();
        for _ in 0..12 {
            for (pre, action, post) in [
                (0, 0, 7),
                (0, 0, 9),
                (0, 1, 7),
                (0, 1, 9),
                (0, 2, 11),
                (0, 2, 13),
                (11, 0, 7),
                (11, 1, 9),
                (13, 0, 9),
                (13, 1, 7),
            ] {
                fact(&mut m, pre, action, post, true);
            }
        }
        m.reset(false);
        let offers = [ActionOffer {
            available: true,
            max_cost: 1,
        }; 3];
        let (p, work) = m.propose(&[0], &offers, Some(&[Some(7)]), 4, 100000);
        let p = p.unwrap();
        assert_eq!(
            p.action, 2,
            "a paid observation improves expected goal success"
        );
        assert!(p.value > 0.7 && work <= 100000);
        let expensive = [
            offers[0],
            offers[1],
            ActionOffer {
                available: true,
                max_cost: 100,
            },
        ];
        assert_ne!(
            m.propose(&[0], &expensive, Some(&[Some(7)]), 4, 100000)
                .0
                .unwrap()
                .action,
            2
        );
    }
    #[test]
    fn delayed_information_credit_requires_an_actual_goal_and_charges_intervening_cost() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        let f = [0.0, 0.0, 0.5, 0.5, 0.5];
        m.feedback(Some(&[Some(7)]), &[0], 2, Some(f), true);
        assert!(
            m.take_credits().is_empty(),
            "an observation cannot mint goal credit"
        );
        m.feedback(Some(&[Some(7)]), &[7], 3, None, true);
        let c = m.take_credits();
        assert_eq!(c.len(), 1);
        assert!((c[0].1 - 0.8).abs() < 1e-9);
        m.feedback(Some(&[Some(7)]), &[0], 1, Some(f), true);
        m.reset(true);
        assert_eq!(
            m.take_credits()[0].1,
            0.0,
            "unfinished episodes must not reward exploration"
        );
    }
    #[test]
    fn horizons_beyond_five_reach_a_learned_ten_transition_goal() {
        let mut m = CausalMemory::new(
            CausalConfig {
                history_depth: 1,
                ..Default::default()
            },
            0,
            vec![],
        )
        .unwrap();
        for _ in 0..4 {
            for state in 0..10 {
                fact(&mut m, state, 0, state, true);
                fact(&mut m, state, 1, state + 1, true);
            }
        }
        let offers = [ActionOffer {
            available: true,
            max_cost: 1,
        }; 2];
        let short = m
            .propose(&[0], &offers, Some(&[Some(10)]), 5, 100000)
            .0
            .unwrap();
        let long = m
            .propose(&[0], &offers, Some(&[Some(10)]), 12, 100000)
            .0
            .unwrap();
        assert!(long.value > short.value + 0.5);
        assert_eq!(long.action, 1);
    }
    #[test]
    fn exhausted_search_still_reports_work_and_changes_no_acquired_evidence() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        fact(&mut m, 0, 0, 1, true);
        let before = m.report().learned_fingerprint;
        let (_, work) = m.propose(
            &[0],
            &[ActionOffer {
                available: true,
                max_cost: 1,
            }],
            Some(&[Some(9)]),
            32,
            40,
        );
        assert!(work > 0 && work <= 40);
        assert_eq!(m.report().learned_fingerprint, before);
    }
    #[test]
    fn a_one_action_horizon_cannot_borrow_the_second_action_goal() {
        let mut m = CausalMemory::new(CausalConfig::default(), 0, vec![]).unwrap();
        for _ in 0..20 {
            fact(&mut m, 0, 0, 1, true);
            fact(&mut m, 1, 0, 9, false);
        }
        m.reset(false);
        let offers = [ActionOffer {
            available: true,
            max_cost: 1,
        }];
        let short = m
            .propose(&[0], &offers, Some(&[Some(9)]), 1, 100000)
            .0
            .unwrap();
        let long = m
            .propose(&[0], &offers, Some(&[Some(9)]), 2, 100000)
            .0
            .unwrap();
        assert_eq!(short.value, 0.0);
        assert!(long.value > 0.5);
    }
    #[test]
    fn consolidation_retains_supported_old_laws_and_their_counterexamples() {
        let mut m = CausalMemory::new(
            CausalConfig {
                history_depth: 1,
                model_entries: 32,
                ..Default::default()
            },
            0,
            vec![],
        )
        .unwrap();
        for _ in 0..100 {
            fact(&mut m, 0, 0, 9, true)
        }
        fact(&mut m, 0, 0, 7, true);
        for state in 100..300 {
            fact(&mut m, state, 0, state + 1, true)
        }
        let report = m.report();
        assert!(report.evicted_entries > 0 && report.retained_entries <= 32);
        let b = m
            .beliefs(
                &[0],
                &[ActionOffer {
                    available: true,
                    max_cost: 1,
                }],
                None,
                10000,
            )
            .unwrap()
            .0;
        assert_eq!(
            b[0].support, 101,
            "old evidence must survive more new states than memory slots"
        );
        assert!(
            b[0].outcomes.iter().any(|(p, w)| p == &[7] && *w > 0.0),
            "a rare real contradiction must remain represented"
        );
    }
}
