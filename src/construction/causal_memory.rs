//! Online predictive-state hypotheses learned only from executed transitions.
//! Hypothesis scores are prequential; observation entropy alone is not value
//! of information. Imagined branches never update these sufficient statistics.
use super::autonomy::ActionOffer;
use super::synthesis::Transition;
use super::vm::{Error, Meter, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
#[derive(Debug, Clone)]
pub(crate) struct CausalProposal {
    pub action: usize,
    pub value: f64,
    pub information_gain: f64,
    pub confidence: f64,
    pub expected: Option<Vec<i64>>,
    pub immediate_goals: Vec<f64>,
    pub immediate_support: Vec<u64>,
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
}

fn key(pre: &[i64], action: usize, context: &[ContextStep], depth: usize) -> String {
    // Exact, canonical keys: no collision can merge physical contexts.
    serde_json::to_string(&(pre, action, &context[context.len().saturating_sub(depth)..])).unwrap()
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
        })
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
    pub fn cancel_experiment_credit(&mut self) {
        self.pending.clear();
    }
    pub fn observe(&mut self, t: &Transition, learning: bool) {
        if t.restart {
            self.context.clear()
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
        let probabilities = depths
            .iter()
            .map(|&d| {
                self.counts
                    .get(&key(&t.pre, t.action, &self.context, d))
                    .map_or(0.5, |c| c.probability(&t.post))
            })
            .collect::<Vec<_>>();
        for (score, p) in self.scores.iter_mut().zip(probabilities) {
            *score = (*score + p.max(1e-12).ln()).max(-1e9)
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
            let k = key(&t.pre, t.action, &self.context, depth);
            let c = self.counts.entry(k.clone()).or_insert(Counts {
                outcomes: Vec::new(),
                other: 0,
                last: t.id,
                depth,
            });
            if let Some(o) = c.outcomes.iter_mut().find(|o| o.post == t.post) {
                o.count = o.count.saturating_add(1)
            } else if c.outcomes.len() < self.config.outcomes_per_entry {
                c.outcomes.push(Outcome {
                    post: t.post.clone(),
                    count: 1,
                })
            } else {
                c.other = c.other.saturating_add(1)
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
            .sum();
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
            };
            let victim = self
                .counts
                .iter()
                .min_by(|(ka, a), (kb, b)| {
                    value(ka, a)
                        .total_cmp(&value(kb, b))
                        .then_with(|| a.last.cmp(&b.last))
                })
                .map(|(k, _)| k.clone());
            let Some(k) = victim else { break };
            if let Some(c) = self.counts.remove(&k) {
                self.bytes = self.bytes.saturating_sub(2 * k.len() + c.bytes());
                self.evicted = self.evicted.saturating_add(1);
            }
        }
    }
    fn beliefs_at(
        &self,
        pre: &[i64],
        context: &[ContextStep],
        offers: &[ActionOffer],
        goal: Option<&[Option<i64>]>,
        meter: &mut Meter,
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
                if let Some(c) = self.counts.get(&key(pre, a, context, d)) {
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
                    if let Some(c) = self.counts.get(&key(pre, action, context, d)) {
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
        let initial = self
            .beliefs_at(pre, &self.context, offers, goal, meter)
            .ok()?;
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
                (a.information_gain / (1.0 + offers[a.action].max_cost as f64))
                    .total_cmp(&(b.information_gain / (1.0 + offers[b.action].max_cost as f64)))
            })?;
            return Some(CausalProposal {
                action: b.action,
                value: 0.0,
                information_gain: b.information_gain,
                confidence: b.support as f64 / (b.support as f64 + 2.0),
                expected: None,
                immediate_goals,
                immediate_support,
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
        let graph_key = |state: &[i64], context: &[ContextStep], left: u64| {
            let k = key(state, 0, context, context.len());
            if cost_bound {
                format!("{k}:{left}")
            } else {
                k
            }
        };
        let mut states = vec![(pre.to_vec(), self.context.clone(), remaining_cost)];
        let mut edges = Vec::new();
        let mut keys = BTreeMap::new();
        keys.insert(graph_key(pre, &self.context, remaining_cost), 0usize);
        let mut pos = 0;
        let mut layers = vec![0usize];
        while pos < states.len() {
            if meter.remaining() < 512 {
                break;
            }
            let (state, context, left) = states[pos].clone();
            let local_offers = offers
                .iter()
                .map(|o| ActionOffer {
                    available: o.available && o.max_cost <= left,
                    max_cost: o.max_cost,
                })
                .collect::<Vec<_>>();
            let b = if pos == 0 {
                initial.clone()
            } else {
                match self.beliefs_at(&state, &context, &local_offers, goal, meter) {
                    Ok(v) => v,
                    Err(_) => break,
                }
            };
            let mut row = Vec::new();
            for belief in b {
                if !local_offers[belief.action].available {
                    continue;
                }
                let next_left = left - offers[belief.action].max_cost;
                let mut outcomes = Vec::new();
                for (post, p) in &belief.outcomes {
                    if meter.charge((post.len() + 4) as u64).is_err() {
                        return None;
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
                    while depth > 0
                        && !(0..offers.len()).any(|a| {
                            self.counts
                                .contains_key(&key(post, a, &next_context, depth))
                        })
                    {
                        depth -= 1
                    }
                    if depth < next_context.len() {
                        next_context = next_context[next_context.len() - depth..].to_vec()
                    }
                    let k = graph_key(post, &next_context, next_left);
                    let target = if reached {
                        None
                    } else if let Some(&id) = keys.get(&k) {
                        Some(id)
                    } else if states.len() < self.config.planning_states
                        && layers[pos] + 1 < horizon
                    {
                        let id = states.len();
                        keys.insert(k, id);
                        states.push((post.clone(), next_context, next_left));
                        layers.push(layers[pos] + 1);
                        Some(id)
                    } else {
                        None
                    };
                    outcomes.push((target, *p, reached));
                }
                row.push((belief, outcomes));
            }
            edges.push(row);
            pos += 1;
        }
        let mut values = vec![0.0_f64; states.len()];
        let mut policies = vec![None; states.len()];
        // The root action below consumes one step of the stated horizon.
        for _ in 1..horizon {
            let mut next = values.clone();
            for (i, row) in edges.iter().enumerate() {
                let mut best = 0.0_f64;
                for (b, outcomes) in row {
                    if meter.charge((outcomes.len() + 2) as u64).is_err() {
                        break;
                    }
                    let q = outcomes
                        .iter()
                        .map(|(id, p, goal)| {
                            p * if *goal {
                                1.0
                            } else {
                                id.map_or(0.0, |j| 0.98 * values[j])
                            }
                        })
                        .sum::<f64>()
                        - 0.01 * offers[b.action].max_cost as f64;
                    if q > best {
                        best = q;
                        policies[i] = Some(b.action)
                    }
                }
                next[i] = best;
            }
            values = next;
        }
        let mut best = None::<(f64, &ActionBelief, f64, f64, f64)>;
        for (b, outcomes) in edges.first()? {
            meter
                .charge((outcomes.len() * pre.len() + 48) as u64)
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
            let raw_information = b.information_gain * b.goal_relevance + frontier;
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
            if best.is_none_or(|(v, _, _, _, _)| score > v) {
                best = Some((score, b, decision_information, q, raw_information))
            }
        }
        let (_, b, decision_information, goal_value, raw_information) = best?;
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
        let mut bytes = 0;
        for (k, c) in &self.counts {
            let (pre, action, context): (Vec<i64>, usize, Vec<ContextStep>) =
                serde_json::from_str(k).map_err(|_| Error::InvalidData)?;
            if c.depth != context.len()
                || key(&pre, action, &context, context.len()) != *k
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
