//! Small recurrent predictive network with explicit truncated backpropagation.
//! Raw reconstruction prevents the latent prediction target alone collapsing
//! to a constant. Two bootstrap dynamics/reward heads expose disagreement.
use super::{kind, raw, word, ContextFrame, Experience, LearningWorldConfig, WorldActionForecast};
use crate::construction::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Coordinate {
    input: Vec<f64>,
    presence: Vec<f64>,
    decoder: Vec<f64>,
    bias: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Head {
    transition: Vec<f64>,
    action: Vec<f64>,
    bias: Vec<f64>,
    reward: Vec<f64>,
    value: Vec<f64>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Network {
    pub(super) dim: usize,
    pub(super) actions: usize,
    pub(super) coordinates: BTreeMap<String, Coordinate>,
    recurrence: Vec<f64>,
    previous_action: Vec<f64>,
    modality: Vec<f64>,
    bias: Vec<f64>,
    heads: Vec<Head>,
    action_counts: Vec<u64>,
    reward_noise: Vec<f64>,
}
#[derive(Debug)]
struct Activation {
    hidden: Vec<f64>,
    previous: Vec<f64>,
    input: Vec<(String, f64)>,
    scales: [f64; 4],
    action: Option<usize>,
    modality: usize,
}
#[derive(Debug)]
pub(super) struct Pass {
    frames: Vec<Activation>,
    dim: usize,
}
impl Pass {
    pub(super) fn last(&self) -> Vec<f64> {
        self.frames
            .last()
            .map_or_else(|| vec![0.0; self.dim], |a| a.hidden.clone())
    }
}

fn random(n: usize, scale: f64, rng: &mut u64) -> Vec<f64> {
    (0..n)
        .map(|_| ((word(rng) >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0) * scale)
        .collect()
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn coordinate_group(key: &str) -> usize {
    match key.as_bytes().first() {
        Some(b'i') => 0,
        Some(b's') => 1,
        Some(b'c') => 2,
        _ => 3,
    }
}
fn sampled_coordinates(values: &[(String, f64)], limit: usize, offset: usize) -> Vec<(usize, f64)> {
    let mut groups: [Vec<usize>; 4] = std::array::from_fn(|_| Vec::new());
    for (i, (key, _)) in values.iter().enumerate() {
        groups[coordinate_group(key)].push(i);
    }
    let active: Vec<_> = groups.iter().filter(|g| !g.is_empty()).collect();
    let mut selected = Vec::new();
    for (k, group) in active.iter().enumerate() {
        let count = group
            .len()
            .min(limit / active.len() + usize::from(k < limit % active.len()));
        for i in 0..count {
            let index = group[(offset + k + i * group.len() / count) % group.len()];
            selected.push((index, 1.0 / (active.len() * count) as f64));
        }
    }
    selected
}
fn projection(weights: &[f64], action: usize, h: &[f64]) -> f64 {
    let row = &weights[action * (h.len() + 1)..(action + 1) * (h.len() + 1)];
    (dot(&row[..h.len()], h) + row[h.len()]).tanh()
}
fn transition(head: &Head, dim: usize, h: &[f64], action: usize) -> Vec<f64> {
    (0..dim)
        .map(|i| {
            (dot(&head.transition[i * dim..(i + 1) * dim], h)
                + head.action[action * dim + i]
                + head.bias[i])
                .tanh()
        })
        .collect()
}
impl Network {
    pub(super) fn new(dim: usize, actions: usize, rng: &mut u64) -> Self {
        let mut recurrence = random(dim * dim, 0.04, rng);
        for i in 0..dim {
            recurrence[i * dim + i] += 0.5;
        }
        let heads = (0..2)
            .map(|_| {
                let mut t = random(dim * dim, 0.06, rng);
                for i in 0..dim {
                    t[i * dim + i] += 0.5;
                }
                Head {
                    transition: t,
                    action: random(actions * dim, 0.1, rng),
                    bias: vec![0.0; dim],
                    reward: random(actions * (dim + 1), 0.1, rng),
                    value: random(actions * (dim + 1), 0.1, rng),
                }
            })
            .collect();
        Self {
            dim,
            actions,
            coordinates: BTreeMap::new(),
            recurrence,
            previous_action: random(actions * dim, 0.05, rng),
            modality: vec![0.0; 4 * dim],
            bias: vec![0.0; dim],
            heads,
            action_counts: vec![0; actions],
            reward_noise: vec![1.0; actions],
        }
    }
    pub(super) fn add_coordinate(&mut self, key: String, rng: &mut u64) {
        self.coordinates.insert(
            key,
            Coordinate {
                input: random(self.dim, 0.3, rng),
                presence: vec![0.0; self.dim],
                decoder: random(self.dim, 0.1, rng),
                bias: 0.0,
            },
        );
    }
    pub(super) fn encode_cost(&self, frames: &[ContextFrame], max: usize) -> u64 {
        frames
            .iter()
            .map(|f| {
                raw(&f.frame, max).map_or(0, |x| x.len()) as u64 * (self.dim as u64 * 4 + 48)
                    + self.dim as u64 * self.dim as u64 * 4
                    + 128
            })
            .sum()
    }
    pub(super) fn encode(&self, frames: &[ContextFrame], max: usize) -> Pass {
        let mut previous = vec![0.0; self.dim];
        let mut activations = Vec::new();
        for frame in frames {
            let input = raw(&frame.frame, max).expect("admitted raw frame");
            let mut counts = [0usize; 4];
            for (key, _) in &input {
                counts[coordinate_group(key)] += 1;
            }
            let groups = counts.iter().filter(|&&n| n > 0).count().max(1);
            let scales = counts.map(|n| 1.0 / ((n.max(1) * groups) as f64).sqrt());
            let modality = kind(&frame.frame);
            let mut h = (0..self.dim)
                .map(|i| {
                    self.bias[i]
                        + self.modality[modality * self.dim + i]
                        + dot(
                            &self.recurrence[i * self.dim..(i + 1) * self.dim],
                            &previous,
                        )
                        + frame
                            .previous_action
                            .map_or(0.0, |a| self.previous_action[a * self.dim + i])
                })
                .collect::<Vec<_>>();
            for (key, value) in &input {
                if let Some(c) = self.coordinates.get(key) {
                    for (i, v) in h.iter_mut().enumerate() {
                        *v += scales[coordinate_group(key)] * (c.input[i] * value + c.presence[i]);
                    }
                }
            }
            for v in &mut h {
                *v = v.tanh();
            }
            activations.push(Activation {
                hidden: h.clone(),
                previous: previous.clone(),
                input,
                scales,
                action: frame.previous_action,
                modality,
            });
            previous = h;
        }
        Pass {
            frames: activations,
            dim: self.dim,
        }
    }
    pub(super) fn forecast_cost(&self) -> u64 {
        (self.actions * self.actions * self.dim * 12
            + self.actions * self.dim * self.dim * 8
            + 1024) as u64
    }
    pub(super) fn forecasts(
        &self,
        h: &[f64],
        allowed: &[bool],
        left: Option<usize>,
        discount: f64,
        goal: Option<&[f64]>,
    ) -> Vec<WorldActionForecast> {
        (0..self.actions)
            .filter(|&a| allowed[a])
            .map(|a| {
                let rewards: Vec<_> = self
                    .heads
                    .iter()
                    .map(|m| projection(&m.reward, a, h))
                    .collect();
                let reward = rewards.iter().sum::<f64>() / 2.0;
                let mut values = Vec::new();
                let mut future_states = Vec::new();
                for m in &self.heads {
                    let p = transition(m, self.dim, h, a);
                    let future = (0..self.actions)
                        .filter(|&b| allowed[b])
                        .map(|b| projection(&m.value, b, &p))
                        .fold(-1.0, f64::max);
                    let imagined = if left == Some(1) {
                        projection(&m.reward, a, h)
                    } else {
                        (projection(&m.reward, a, h) + discount * future).clamp(-1.0, 1.0)
                    };
                    let value = if left == Some(1) {
                        imagined
                    } else {
                        0.5 * projection(&m.value, a, h) + 0.5 * imagined
                    };
                    let value = if let Some(g) = goal {
                        let distance = p.iter().zip(g).map(|(a, b)| (a - b).powi(2)).sum::<f64>()
                            / self.dim as f64;
                        0.2 * value + 0.8 * (2.0 * (-2.0 * distance).exp() - 1.0)
                    } else {
                        value
                    };
                    values.push(value);
                    future_states.push(p);
                }
                let disagreement = (rewards[0] - rewards[1]).powi(2) * 0.25
                    + future_states[0]
                        .iter()
                        .zip(&future_states[1])
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>()
                        / (4.0 * self.dim as f64);
                let support = self.action_counts[a];
                // This is an ensemble/count heuristic, not a calibrated posterior.
                let epistemic = (disagreement / (0.01 + self.reward_noise[a])
                    + 1.0 / (1.0 + support as f64).sqrt())
                .clamp(0.0, 1.0);
                WorldActionForecast {
                    action: a,
                    reward,
                    value: (values[0] + values[1]) * 0.5,
                    epistemic,
                    outcome_noise: self.reward_noise[a],
                    support,
                }
            })
            .collect()
    }
    pub(super) fn sensor_error(
        &self,
        h: &[f64],
        action: usize,
        values: &[(String, f64)],
    ) -> (f64, u64) {
        let ps: Vec<_> = self
            .heads
            .iter()
            .map(|m| transition(m, self.dim, h, action))
            .collect();
        let mut sum = 0.0;
        let mut count = 0;
        for (key, target) in values {
            if let Some(c) = self.coordinates.get(key) {
                let prediction = c.bias + 0.5 * (dot(&c.decoder, &ps[0]) + dot(&c.decoder, &ps[1]));
                sum += (prediction - target).powi(2);
                count += 1;
            }
        }
        (sum, count)
    }
    pub(super) fn observe_outcome(&mut self, action: usize, error: Option<f64>) {
        self.action_counts[action] = self.action_counts[action].saturating_add(1);
        if let Some(error) = error {
            self.reward_noise[action] =
                (0.97 * self.reward_noise[action] + 0.03 * error.powi(2)).clamp(1e-6, 4.0);
        }
    }
    pub(super) fn training_cost(&self, sample: &Experience, cfg: &LearningWorldConfig) -> u64 {
        let mut next = sample.context.clone();
        next.push(sample.post.clone());
        while next.len() > cfg.context_frames {
            next.remove(0);
        }
        self.encode_cost(&sample.context, cfg.max_frame_values)
            + self.encode_cost(&next, cfg.max_frame_values)
            + sample
                .context
                .iter()
                .map(|f| {
                    raw(&f.frame, cfg.max_frame_values).map_or(0, |v| v.len()) as u64
                        * (self.dim as u64 * 16 + 64)
                })
                .sum::<u64>()
            + (cfg.reconstruction_targets * self.dim * 32
                + cfg.context_frames * self.dim * self.dim * 16
                + self.actions * self.dim * 32
                + 2048) as u64
    }
    pub(super) fn train(
        &mut self,
        sample: &Experience,
        cfg: &LearningWorldConfig,
        rng: &mut u64,
    ) -> f64 {
        let pass = self.encode(&sample.context, cfg.max_frame_values);
        let h = pass.last();
        let mut next = sample.context.clone();
        next.push(sample.post.clone());
        while next.len() > cfg.context_frames {
            next.remove(0);
        }
        let next_h = self.encode(&next, cfg.max_frame_values).last();
        let a = sample.post.previous_action.expect("actual action");
        let targets = raw(&sample.post.frame, cfg.max_frame_values).expect("admitted frame");
        let mut gradient = Gradient::new(self);
        let mut dh = vec![0.0; self.dim];
        let mut loss = 0.0;
        let offset = if targets.is_empty() {
            0
        } else {
            word(rng) as usize % targets.len()
        };
        let target_indices = sampled_coordinates(&targets, cfg.reconstruction_targets, offset);
        for (m, head) in self.heads.iter().enumerate() {
            let mut bootstrap = sample.event.wrapping_mul(0x9e3779b97f4a7c15)
                ^ (m as u64 + 1).wrapping_mul(0xbf58476d1ce4e5b9);
            let weight = (1 + (word(&mut bootstrap) % 2)) as f64 * 0.5;
            let predicted = transition(head, self.dim, &h, a);
            let mut dp = vec![0.0; self.dim];
            for &(index, sample_weight) in &target_indices {
                let (key, target) = &targets[index];
                if let Some(c) = self.coordinates.get(key) {
                    let output = dot(&c.decoder, &predicted) + c.bias;
                    let error = output - target;
                    let derivative = weight * error * sample_weight;
                    loss += 0.5 * weight * error * error * sample_weight;
                    let cg = gradient.coordinate(key, self.dim);
                    cg.bias += derivative;
                    for i in 0..self.dim {
                        cg.decoder[i] += derivative * predicted[i];
                        dp[i] += derivative * c.decoder[i];
                    }
                }
            }
            // Stop-gradient sensory target anchors the model rollout to the
            // SAME recurrent belief space; raw reconstruction prevents collapse.
            for i in 0..self.dim {
                dp[i] += 0.1 * weight * (predicted[i] - next_h[i]) / self.dim as f64;
            }
            let bootstrap_value = if sample.terminal {
                0.0
            } else {
                (0..self.actions)
                    .map(|b| projection(&head.value, b, &next_h))
                    .fold(-1.0, f64::max)
            };
            let qtarget = (sample.reward + cfg.discount * bootstrap_value).clamp(-1.0, 1.0);
            let gh = &mut gradient.heads[m];
            for (params, gparams, target, strength) in [
                (&head.reward, &mut gh.reward, sample.reward, 1.0),
                (&head.value, &mut gh.value, qtarget, 0.5),
            ] {
                let output = projection(params, a, &h);
                let error = output - target;
                let derivative = weight * strength * error * (1.0 - output * output);
                loss += 0.5 * weight * strength * error * error;
                for i in 0..self.dim {
                    gparams[a * (self.dim + 1) + i] += derivative * h[i];
                    dh[i] += derivative * params[a * (self.dim + 1) + i];
                }
                gparams[a * (self.dim + 1) + self.dim] += derivative;
            }
            for i in 0..self.dim {
                let d = dp[i] * (1.0 - predicted[i] * predicted[i]);
                gradient.heads[m].bias[i] += d;
                gradient.heads[m].action[a * self.dim + i] += d;
                for j in 0..self.dim {
                    gradient.heads[m].transition[i * self.dim + j] += d * h[j];
                    dh[j] += d * head.transition[i * self.dim + j];
                }
            }
        }
        // Reconstruct the current raw observation in addition to predicting
        // successors. Decoder biases alone cannot explain changing inputs.
        if let Some(last) = pass.frames.last() {
            for (index, sample_weight) in
                sampled_coordinates(&last.input, cfg.reconstruction_targets, offset)
            {
                let (key, target) = &last.input[index];
                if let Some(c) = self.coordinates.get(key) {
                    let error = dot(&c.decoder, &h) + c.bias - target;
                    let d = 0.2 * error * sample_weight;
                    let cg = gradient.coordinate(key, self.dim);
                    cg.bias += d;
                    for i in 0..self.dim {
                        cg.decoder[i] += d * h[i];
                        dh[i] += d * c.decoder[i];
                    }
                }
            }
        }
        // Backpropagate the actual prediction/reward error through the full
        // retained history and train the sensory projection and recurrent gate.
        for frame in pass.frames.iter().rev() {
            let dz: Vec<_> = dh
                .iter()
                .zip(&frame.hidden)
                .map(|(d, h)| d * (1.0 - h * h))
                .collect();
            let mut previous = vec![0.0; self.dim];
            for i in 0..self.dim {
                gradient.bias[i] += dz[i];
                gradient.modality[frame.modality * self.dim + i] += dz[i];
                if let Some(a) = frame.action {
                    gradient.previous_action[a * self.dim + i] += dz[i];
                }
                for j in 0..self.dim {
                    gradient.recurrence[i * self.dim + j] += dz[i] * frame.previous[j];
                    previous[j] += dz[i] * self.recurrence[i * self.dim + j];
                }
            }
            for (key, value) in &frame.input {
                if self.coordinates.contains_key(key) {
                    let cg = gradient.coordinate(key, self.dim);
                    for i in 0..self.dim {
                        cg.input[i] += dz[i] * value * frame.scales[coordinate_group(key)];
                        cg.presence[i] += dz[i] * frame.scales[coordinate_group(key)];
                    }
                }
            }
            dh = previous;
        }
        gradient.apply(self, cfg.learning_rate);
        loss.max(0.0)
    }
    pub(super) fn validate(&self, cfg: &LearningWorldConfig) -> Result<()> {
        let good =
            |v: &[f64], n: usize| v.len() == n && v.iter().all(|x| x.is_finite() && x.abs() <= 6.0);
        if self.dim != cfg.latent
            || !(1..=16).contains(&self.actions)
            || self.coordinates.len() > cfg.max_coordinates
            || self.heads.len() != 2
            || !good(&self.recurrence, self.dim * self.dim)
            || !good(&self.previous_action, self.actions * self.dim)
            || !good(&self.modality, 4 * self.dim)
            || !good(&self.bias, self.dim)
            || self.action_counts.len() != self.actions
            || self.reward_noise.len() != self.actions
            || self
                .reward_noise
                .iter()
                .any(|x| !x.is_finite() || !(1e-6..=4.0).contains(x))
        {
            return Err(Error::InvalidData);
        }
        for (key, c) in &self.coordinates {
            if key.len() > 48
                || !key
                    .bytes()
                    .all(|x| x.is_ascii_digit() || b"iscm:".contains(&x))
                || !good(&c.input, self.dim)
                || !good(&c.presence, self.dim)
                || !good(&c.decoder, self.dim)
                || !c.bias.is_finite()
                || c.bias.abs() > 6.0
            {
                return Err(Error::InvalidData);
            }
        }
        for h in &self.heads {
            if !good(&h.transition, self.dim * self.dim)
                || !good(&h.action, self.actions * self.dim)
                || !good(&h.bias, self.dim)
                || !good(&h.reward, self.actions * (self.dim + 1))
                || !good(&h.value, self.actions * (self.dim + 1))
            {
                return Err(Error::InvalidData);
            }
        }
        Ok(())
    }
}

struct Gradient {
    coordinates: BTreeMap<String, Coordinate>,
    recurrence: Vec<f64>,
    previous_action: Vec<f64>,
    modality: Vec<f64>,
    bias: Vec<f64>,
    heads: Vec<Head>,
}
impl Gradient {
    fn new(n: &Network) -> Self {
        Self {
            coordinates: BTreeMap::new(),
            recurrence: vec![0.0; n.dim * n.dim],
            previous_action: vec![0.0; n.actions * n.dim],
            modality: vec![0.0; 4 * n.dim],
            bias: vec![0.0; n.dim],
            heads: (0..2)
                .map(|_| Head {
                    transition: vec![0.0; n.dim * n.dim],
                    action: vec![0.0; n.actions * n.dim],
                    bias: vec![0.0; n.dim],
                    reward: vec![0.0; n.actions * (n.dim + 1)],
                    value: vec![0.0; n.actions * (n.dim + 1)],
                })
                .collect(),
        }
    }
    fn coordinate(&mut self, key: &str, d: usize) -> &mut Coordinate {
        self.coordinates
            .entry(key.into())
            .or_insert_with(|| Coordinate {
                input: vec![0.0; d],
                presence: vec![0.0; d],
                decoder: vec![0.0; d],
                bias: 0.0,
            })
    }
    fn apply(self, n: &mut Network, rate: f64) {
        // Clipping keeps online and replay updates bounded, including after
        // long out-of-distribution streams. No hidden optimizer state exists.
        fn update(p: &mut [f64], g: &[f64], r: f64) {
            for (p, g) in p.iter_mut().zip(g) {
                *p = (*p - r * g.clamp(-1.0, 1.0)).clamp(-6.0, 6.0);
            }
        }
        update(&mut n.recurrence, &self.recurrence, rate);
        update(&mut n.previous_action, &self.previous_action, rate);
        update(&mut n.modality, &self.modality, rate);
        update(&mut n.bias, &self.bias, rate);
        for (h, g) in n.heads.iter_mut().zip(&self.heads) {
            update(&mut h.transition, &g.transition, rate);
            update(&mut h.action, &g.action, rate);
            update(&mut h.bias, &g.bias, rate);
            update(&mut h.reward, &g.reward, rate);
            update(&mut h.value, &g.value, rate);
        }
        for (key, g) in self.coordinates {
            if let Some(c) = n.coordinates.get_mut(&key) {
                update(&mut c.input, &g.input, rate);
                update(&mut c.presence, &g.presence, rate);
                update(&mut c.decoder, &g.decoder, rate);
                c.bias = (c.bias - rate * g.bias.clamp(-1.0, 1.0)).clamp(-6.0, 6.0);
            }
        }
    }
}
