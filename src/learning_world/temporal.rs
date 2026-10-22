//! Causal scalar streams, learned lag kernels and executable update rules.
//! This is state of the existing learning world, not an actor or a file loader.
use super::{hash, SensorFrame};
use crate::construction::{
    search_policy::SearchPolicy,
    vm::{Context, Expr, Library, Meter},
    Error, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod rule_search;
use rule_search::{Lesson, RuleSearch};
const LAGS: usize = 8;
const SERIES: usize = 2048;
const LESSON_VALUES: usize = 48;
const MAX_RULE_NODES: usize = 15;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TemporalConfig {
    pub search: bool,
    pub recursive: bool,
    pub seed: u64,
}
impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            search: true,
            recursive: true,
            seed: 0x54454d504f52414c,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Weights {
    logits: Vec<f64>,
    previous_gradient: Vec<f64>,
}
impl Default for Weights {
    fn default() -> Self {
        let mut logits = vec![-8.; LAGS];
        logits[0] = 0.;
        Self {
            logits,
            previous_gradient: vec![0.; LAGS],
        }
    }
}
impl Weights {
    fn validate(&self) -> bool {
        self.logits.len() == LAGS
            && self.previous_gradient.len() == LAGS
            && self
                .logits
                .iter()
                .all(|x| x.is_finite() && (-20.00000001..=0.00000001).contains(x))
            && self
                .previous_gradient
                .iter()
                .all(|x| x.is_finite() && x.abs() <= 200.)
    }
    fn coefficients(&self, n: usize) -> Vec<f64> {
        let max = self.logits[..n]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let mut w: Vec<_> = self.logits[..n].iter().map(|x| (x - max).exp()).collect();
        let sum = w.iter().sum::<f64>();
        for x in &mut w {
            *x /= sum;
        }
        w
    }
    fn predict(&self, history: &[f64]) -> f64 {
        let n = history.len().min(LAGS);
        if n == 0 {
            return 0.;
        }
        self.coefficients(n)
            .iter()
            .zip(history.iter().rev())
            .map(|(w, x)| w * x)
            .sum()
    }
    fn learn(
        &mut self,
        history: &[f64],
        energy: f64,
        count: u64,
        target: f64,
        rule: &Expr,
        meter: &mut Meter,
    ) -> Result<()> {
        if history.len() < LAGS {
            return Ok(());
        }
        meter.charge((LAGS * 48) as u64)?;
        let base = *history.last().ok_or(Error::NoEvidence)?;
        let scale = energy.sqrt().max(base.abs() * 1e-9).max(1e-12);
        let x: Vec<_> = history
            .iter()
            .rev()
            .take(LAGS)
            .map(|x| ((x - base) / scale).clamp(-10., 10.))
            .collect();
        let weights = self.coefficients(LAGS);
        let prediction = weights.iter().zip(&x).map(|(w, x)| w * x).sum::<f64>();
        let residual = (prediction - (target - base) / scale).clamp(-10., 10.);
        for i in 0..LAGS {
            let gradient = residual * (x[i] - prediction);
            let input = [
                (gradient * 1024.).round() as i64,
                (self.previous_gradient[i] * 1024.).round() as i64,
                ((x[i] - prediction) * 1024.).round() as i64,
                (residual * 1024.).round() as i64,
                (self.logits[i] * 256.).round() as i64,
                count.min(1_000_000) as i64,
            ];
            // An invalid candidate cannot acquire a free or unbounded update.
            let output = match rule.eval(
                Context {
                    input: &input,
                    action: 0,
                    state: &[],
                    args: &[],
                },
                &Library::default(),
                meter,
            ) {
                Ok(v) => v.int()? as f64 / 1024.,
                Err(Error::Budget) => return Err(Error::Budget),
                Err(_) => gradient,
            };
            self.logits[i] -= 0.02 * output.clamp(-100., 100.);
            self.previous_gradient[i] = gradient;
        }
        let max = self
            .logits
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        for v in &mut self.logits {
            *v = (*v - max).max(-20.);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Series {
    weights: Weights,
    history: Vec<f64>,
    energy: f64,
    count: u64,
    buffer: Vec<f64>,
    buffer_seed: Option<Seed>,
    first_origin: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Seed {
    weights: Weights,
    history: Vec<f64>,
    energy: f64,
    count: u64,
}
impl Seed {
    fn capture(s: &Series) -> Self {
        Self {
            weights: s.weights.clone(),
            history: s.history.clone(),
            energy: s.energy,
            count: s.count,
        }
    }
    fn restore(&self) -> Series {
        Series {
            weights: self.weights.clone(),
            history: self.history.clone(),
            energy: self.energy,
            count: self.count,
            ..Default::default()
        }
    }
    fn valid(&self) -> bool {
        self.weights.validate()
            && self.history.len() <= LAGS
            && self.history.iter().all(|x| x.is_finite() && x.abs() <= 1e6)
            && self.energy.is_finite()
            && self.energy >= 0.
    }
}
impl Series {
    fn append(&mut self, value: f64) {
        if let Some(last) = self.history.last() {
            let difference = value - last;
            let rate = 1. / (self.count.max(1).min(64) as f64);
            self.energy += (difference * difference - self.energy) * rate;
        }
        self.count = self.count.saturating_add(1);
        self.history.push(value);
        if self.history.len() > LAGS {
            self.history.remove(0);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleTrial {
    candidate: Expr,
    proposed_after: u64,
    frames: u64,
    values: u64,
    before: f64,
    after: f64,
    by_kind: BTreeMap<u8, (u64, f64, f64)>,
    shadow: BTreeMap<String, Weights>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleReceipt {
    pub candidate: Expr,
    pub incumbent: Expr,
    pub proposed_after: u64,
    pub resolved_after: u64,
    pub future_frames: u64,
    pub future_values: u64,
    pub before: f64,
    pub after: f64,
    pub accepted: bool,
}
impl RuleTrial {
    fn verdict(&self) -> Option<bool> {
        if self.frames < 128 || self.values < 128 {
            return None;
        }
        Some(
            self.after <= 0.95 * self.before
                && self.before > 1e-18
                && self
                    .by_kind
                    .values()
                    .all(|(n, a, b)| *n < 8 || *b <= 1.02 * a),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPrediction {
    pub values: BTreeMap<String, f64>,
    pub shadow: Option<BTreeMap<String, f64>>,
    pub features: Vec<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPrediction {
    pub values: BTreeMap<String, f64>,
    pub temporal_values: BTreeMap<String, f64>,
    pub neural_values: BTreeMap<String, f64>,
    pub neural_weights: [f64; 3],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RewardResidual {
    weights: Vec<f64>,
    observations: u64,
}
impl Default for RewardResidual {
    fn default() -> Self {
        Self {
            weights: vec![0.; 10],
            observations: 0,
        }
    }
}
impl RewardResidual {
    fn correction(&self, features: &[f64]) -> f64 {
        (self
            .weights
            .iter()
            .zip(features)
            .map(|(w, x)| w * x)
            .sum::<f64>()
            * self.observations as f64
            / (self.observations as f64 + 16.))
            .clamp(-1., 1.)
    }
    fn learn(&mut self, features: &[f64], target: f64) {
        let prediction = self
            .weights
            .iter()
            .zip(features)
            .map(|(w, x)| w * x)
            .sum::<f64>();
        let rate = 0.05 * (target - prediction).clamp(-2., 2.)
            / (1. + features.iter().map(|x| x * x).sum::<f64>());
        for (w, x) in self.weights.iter_mut().zip(features) {
            *w = (*w + rate * x).clamp(-4., 4.);
        }
        self.observations = self.observations.saturating_add(1);
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalReport {
    pub world_forecast_values: u64,
    pub world_forecast_mse: f64,
    pub neural_prediction_weights: [f64; 3],
    pub accepted_rule_uses: u64,
    pub last_rule_trial: Option<RuleReceipt>,
    pub neural_reconstruction_paused: bool,
    pub comparison_frames: u64,
    pub recent_neural_mse: f64,
    pub recent_temporal_mse: f64,
    pub config: TemporalConfig,
    pub raw_values_presented: u64,
    pub parameter_updates: u64,
    pub series: usize,
    pub unrepresented_values: u64,
    pub committed_frames: u64,
    pub forecast_values: u64,
    pub forecast_mse: f64,
    pub baseline_mse: f64,
    pub rule: Expr,
    pub rule_trials: u64,
    pub rule_admissions: u64,
    pub rule_rejections: u64,
    pub pending_rule_frames: Option<u64>,
    pub pending_rule_losses: Option<(f64, f64)>,
    pub search: rule_search::SearchReport,
    pub learned_fingerprint: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Temporal {
    fusion_a: [f64; 3],
    fusion_b: [f64; 3],
    fusion_frames: [u64; 3],
    world_error: f64,
    world_values: u64,
    reward_recent: f64,
    rule_uses: u64,
    #[serde(skip)]
    pending_neural_values: BTreeMap<String, f64>,
    schema_version: u8,
    receipts: Vec<RuleReceipt>,
    comparison_frames: u64,
    neural_recent: f64,
    temporal_recent: f64,
    neural_paused: bool,
    config: TemporalConfig,
    models: BTreeMap<String, Series>,
    rule: Expr,
    search: RuleSearch,
    raw_values: u64,
    updates: u64,
    unrepresented: u64,
    frames: u64,
    values: u64,
    squared_error: f64,
    baseline_error: f64,
    trials: u64,
    admissions: u64,
    rejections: u64,
    trial: Option<RuleTrial>,
    residuals: Vec<RewardResidual>,
    #[serde(skip)]
    pending: Option<TemporalPrediction>,
    #[serde(skip)]
    base_rewards: BTreeMap<usize, f64>,
}

struct Block {
    key: String,
    keys: Vec<String>,
    values: Vec<f64>,
    kind: u8,
}
fn blocks(frame: &SensorFrame) -> Vec<Block> {
    match frame {
        SensorFrame::Signals { channels, .. } => channels
            .iter()
            .enumerate()
            .filter_map(|(axis, v)| {
                v.as_ref().map(|v| Block {
                    key: format!("s:{axis}"),
                    keys: (0..v.len())
                        .map(|t| format!("s:{axis}:{}:{t}", v.len()))
                        .collect(),
                    values: v.clone(),
                    kind: 1,
                })
            })
            .collect(),
        SensorFrame::Image { pixels: None, .. } => Vec::new(),
        SensorFrame::Image {
            pixels: Some(pixels),
            ..
        } => {
            let h = pixels.len();
            let w = pixels[0].len();
            pixels
                .iter()
                .flatten()
                .enumerate()
                .map(|(i, x)| {
                    let k = format!("i:{h}:{w}:{i}");
                    Block {
                        key: k.clone(),
                        keys: vec![k],
                        values: vec![f64::from(*x)],
                        kind: 0,
                    }
                })
                .collect()
        }
        SensorFrame::Scene { pixels, .. } => {
            let h = pixels.len();
            let w = pixels[0].len();
            pixels
                .iter()
                .flatten()
                .enumerate()
                .map(|(i, x)| {
                    let k = format!("c:{h}:{w}:{i}");
                    Block {
                        key: k.clone(),
                        keys: vec![k],
                        values: vec![f64::from(*x)],
                        kind: 2,
                    }
                })
                .collect()
        }
        SensorFrame::Multimodal { frames, .. } => frames.iter().flat_map(blocks).collect(),
    }
}
fn normalized(kind: u8, x: f64) -> f64 {
    match kind {
        0 => 2. * x / 255. - 1.,
        2 => 2. * x / 65535. - 1.,
        _ => x / (1. + x.abs()),
    }
}

impl Temporal {
    pub(super) fn new(config: TemporalConfig, policy: SearchPolicy) -> Result<Self> {
        policy.validate()?;
        Ok(Self {
            fusion_a: [0.; 3],
            fusion_b: [0.; 3],
            fusion_frames: [0; 3],
            world_error: 0.,
            world_values: 0,
            reward_recent: 0.,
            rule_uses: 0,
            pending_neural_values: BTreeMap::new(),
            schema_version: 1,
            receipts: Vec::new(),
            comparison_frames: 0,
            neural_recent: 0.,
            temporal_recent: 0.,
            neural_paused: false,
            search: RuleSearch::new(config.seed, policy),
            config,
            models: BTreeMap::new(),
            rule: Expr::Input(0),
            raw_values: 0,
            updates: 0,
            unrepresented: 0,
            frames: 0,
            values: 0,
            squared_error: 0.,
            baseline_error: 0.,
            trials: 0,
            admissions: 0,
            rejections: 0,
            trial: None,
            residuals: vec![RewardResidual::default(); 16],
            pending: None,
            base_rewards: BTreeMap::new(),
        })
    }
    pub(super) fn observation_bound(frame: &SensorFrame) -> u64 {
        let n: usize = blocks(frame).iter().map(|b| b.values.len()).sum();
        (n * (LAGS * (96 + MAX_RULE_NODES * 2) + 128)) as u64
    }
    pub(super) fn prediction_bound(frame: &SensorFrame) -> u64 {
        2048 + blocks(frame)
            .iter()
            .map(|b| (b.values.len() * (LAGS * 64 + 64)) as u64)
            .sum::<u64>()
    }
    pub(super) fn predict(
        &self,
        frame: &SensorFrame,
        meter: &mut Meter,
    ) -> Result<TemporalPrediction> {
        meter.charge(2048)?;
        let mut result = TemporalPrediction {
            values: BTreeMap::new(),
            shadow: self.trial.as_ref().map(|_| BTreeMap::new()),
            features: vec![0.; 10],
        };
        let mut summaries = [(0u64, 0., 0., 0.); 3];
        for b in blocks(frame) {
            let fallback_history = || {
                b.values
                    .iter()
                    .rev()
                    .take(LAGS)
                    .copied()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect()
            };
            let mut history = self
                .models
                .get(&b.key)
                .filter(|s| !s.history.is_empty())
                .map_or_else(fallback_history, |s| s.history.clone());
            let mut shadow_history = history.clone();
            let fallback = Weights::default();
            let weights = self.models.get(&b.key).map_or(&fallback, |s| &s.weights);
            let shadow = self
                .trial
                .as_ref()
                .and_then(|t| t.shadow.get(&b.key))
                .unwrap_or(weights);
            for (k, observed) in b.keys.into_iter().zip(b.values) {
                meter.charge((LAGS * 64 + 64) as u64)?;
                let value = weights.predict(&history);
                result.values.insert(k.clone(), normalized(b.kind, value));
                let p = normalized(b.kind, value);
                let s = &mut summaries[b.kind as usize];
                s.0 += 1;
                s.1 += p;
                s.2 += p * p;
                s.3 += p - normalized(b.kind, observed);
                history.push(value);
                if history.len() > LAGS {
                    history.remove(0);
                }
                if let Some(values) = result.shadow.as_mut() {
                    let value = shadow.predict(&shadow_history);
                    values.insert(k, normalized(b.kind, value));
                    shadow_history.push(value);
                    if shadow_history.len() > LAGS {
                        shadow_history.remove(0);
                    }
                }
            }
        }
        result.features[0] = 1.;
        for (k, (n, sum, squares, change)) in summaries.into_iter().enumerate() {
            if n > 0 {
                let n = n as f64;
                let mean = sum / n;
                result.features[1 + k * 3] = mean;
                result.features[2 + k * 3] = (squares / n - mean * mean).max(0.);
                result.features[3 + k * 3] = (change / n).clamp(-2., 2.);
            }
        }
        Ok(result)
    }
    pub(super) fn begin(&mut self, frame: &SensorFrame, meter: &mut Meter) -> Result<()> {
        self.pending = Some(self.predict(frame, meter)?);
        Ok(())
    }
    pub(super) fn correct(&self, features: &[f64], forecasts: &mut [super::WorldActionForecast]) {
        for p in forecasts {
            if let Some(head) = self.residuals.get(p.action) {
                let correction = head.correction(features);
                p.reward = (p.reward + correction).clamp(-1., 1.);
                p.value = (p.value + correction).clamp(-1., 1.);
            }
        }
    }
    pub(super) fn correct_pending(&mut self, forecasts: &mut [super::WorldActionForecast]) {
        self.base_rewards = forecasts.iter().map(|p| (p.action, p.reward)).collect();
        if let Some(p) = &self.pending {
            self.correct(&p.features, forecasts);
        }
    }
    pub(super) fn cancel(&mut self) {
        self.pending = None;
        self.base_rewards.clear();
        self.pending_neural_values.clear();
    }
    pub(super) fn pending_keys(&self) -> Vec<String> {
        self.pending
            .as_ref()
            .map_or_else(Vec::new, |p| p.values.keys().cloned().collect())
    }
    pub(super) fn note_neural_predictions(&mut self, predictions: BTreeMap<String, f64>) {
        self.pending_neural_values = predictions;
    }
    pub(super) fn neural_weights(&self) -> [f64; 3] {
        std::array::from_fn(|k| {
            if self.fusion_frames[k] < 8 || self.fusion_a[k] <= 1e-18 {
                0.
            } else {
                (self.fusion_b[k] / self.fusion_a[k]).clamp(0., 1.)
            }
        })
    }
    pub(super) fn fuse(
        &self,
        prediction: TemporalPrediction,
        neural: BTreeMap<String, f64>,
    ) -> WorldPrediction {
        let weights = self.neural_weights();
        let values = prediction
            .values
            .iter()
            .map(|(k, t)| {
                let kind = match k.as_bytes().first() {
                    Some(b'i') => 0,
                    Some(b'c') => 2,
                    _ => 1,
                };
                let weight = weights[kind];
                (
                    k.clone(),
                    neural.get(k).map_or(*t, |n| (1. - weight) * t + weight * n),
                )
            })
            .collect();
        WorldPrediction {
            values,
            temporal_values: prediction.values,
            neural_values: neural,
            neural_weights: weights,
        }
    }
    pub(super) fn pause_reconstruction(&self) -> bool {
        self.neural_paused
    }
    pub(super) fn restart(
        &mut self,
        frame: &SensorFrame,
        frozen: bool,
        meter: &mut Meter,
    ) -> Result<()> {
        self.cancel();
        if !frozen {
            self.fusion_a = [0.; 3];
            self.fusion_b = [0.; 3];
            self.fusion_frames = [0; 3];
            self.reward_recent = 0.;
            self.comparison_frames = 0;
            self.neural_recent = 0.;
            self.temporal_recent = 0.;
            self.neural_paused = false;
        }
        for s in self.models.values_mut() {
            s.history.clear();
            s.energy = 0.;
            s.count = 0;
            if !frozen {
                s.buffer.clear();
                s.buffer_seed = None;
                s.first_origin = 0;
            }
        }
        self.observe(frame, frozen, meter)
    }
    pub(super) fn observe(
        &mut self,
        frame: &SensorFrame,
        frozen: bool,
        meter: &mut Meter,
    ) -> Result<()> {
        let presented = blocks(frame);
        meter.charge((presented.iter().map(|b| b.values.len()).sum::<usize>() * 64) as u64)?;
        for block in presented {
            if !self.models.contains_key(&block.key) {
                if frozen || self.models.len() >= SERIES {
                    self.unrepresented =
                        self.unrepresented.saturating_add(block.values.len() as u64);
                    continue;
                }
                self.models.insert(block.key.clone(), Series::default());
            }
            let series = self.models.get_mut(&block.key).ok_or(Error::NoEvidence)?;
            for value in block.values {
                self.raw_values = self.raw_values.saturating_add(1);
                if !frozen {
                    if series.buffer.is_empty() {
                        series.buffer_seed = Some(Seed::capture(series));
                        series.first_origin = self.raw_values;
                    }
                    let learning_bound =
                        (LAGS * (48 + MAX_RULE_NODES) * if self.trial.is_some() { 2 } else { 1 })
                            as u64;
                    if meter.remaining() >= learning_bound {
                        if let Some(t) = self.trial.as_mut() {
                            let shadow = t
                                .shadow
                                .entry(block.key.clone())
                                .or_insert_with(|| series.weights.clone());
                            shadow.learn(
                                &series.history,
                                series.energy,
                                series.count,
                                value,
                                &t.candidate,
                                meter,
                            )?;
                        }
                        series.weights.learn(
                            &series.history,
                            series.energy,
                            series.count,
                            value,
                            &self.rule,
                            meter,
                        )?;
                        self.updates = self
                            .updates
                            .saturating_add(u64::from(series.history.len() >= LAGS));
                        if series.history.len() >= LAGS && self.rule != Expr::Input(0) {
                            self.rule_uses = self.rule_uses.saturating_add(1);
                        }
                    }
                    series.buffer.push(value);
                    if series.buffer.len() == LESSON_VALUES {
                        self.search.remember(Lesson {
                            values: std::mem::take(&mut series.buffer),
                            kind: block.kind,
                            origin: self.raw_values,
                            first_origin: series.first_origin,
                            initial: series.buffer_seed.take().ok_or(Error::InvalidData)?,
                        });
                    }
                }
                series.append(value);
            }
        }
        Ok(())
    }
    pub(super) fn committed(
        &mut self,
        action: usize,
        reward: f64,
        frame: &SensorFrame,
        frozen: bool,
        meter: &mut Meter,
    ) -> Result<()> {
        self.frames = self.frames.saturating_add(1);
        if let Some(prediction) = self.pending.take() {
            let neural_weights = self.neural_weights();
            let mut paired = [(0_u64, 0., 0., 0., 0.); 3];
            if !frozen {
                if let (Some(base), Some(head)) = (
                    self.base_rewards.get(&action),
                    self.residuals.get_mut(action),
                ) {
                    meter.charge(256)?;
                    let expected = (base + head.correction(&prediction.features)).clamp(-1., 1.);
                    self.reward_recent += ((reward - expected).powi(2) - self.reward_recent)
                        / self.frames.min(64) as f64;
                    head.learn(&prediction.features, reward - base);
                }
            }
            let mut sums = BTreeMap::<u8, (u64, f64, f64)>::new();
            for block in blocks(frame) {
                let baseline = self
                    .models
                    .get(&block.key)
                    .and_then(|s| s.history.last())
                    .copied();
                for (key, value) in block.keys.iter().zip(block.values) {
                    if let Some(p) = prediction.values.get(key) {
                        let target = normalized(block.kind, value);
                        let error = (p - target).powi(2);
                        self.values = self.values.saturating_add(1);
                        self.squared_error += error;
                        let neural_weight = neural_weights[block.kind as usize];
                        let neural = self.pending_neural_values.get(key).copied();
                        let fused =
                            neural.map_or(*p, |n| (1. - neural_weight) * p + neural_weight * n);
                        self.world_values = self.world_values.saturating_add(1);
                        self.world_error += (fused - target).powi(2);
                        if let Some(n) = neural {
                            let stats = &mut paired[block.kind as usize];
                            stats.0 += 1;
                            stats.1 += (n - target).powi(2);
                            stats.2 += error;
                            let difference = n - p;
                            stats.3 += difference * difference;
                            stats.4 += (target - p) * difference;
                        }
                        if let Some(b) = baseline {
                            self.baseline_error += (normalized(block.kind, b) - target).powi(2);
                        }
                        if let Some(q) = prediction.shadow.as_ref().and_then(|s| s.get(key)) {
                            let entry = sums.entry(block.kind).or_default();
                            entry.0 += 1;
                            entry.1 += (fused - target).powi(2);
                            let candidate =
                                neural.map_or(*q, |n| (1. - neural_weight) * q + neural_weight * n);
                            entry.2 += (candidate - target).powi(2);
                        }
                    }
                }
            }
            if !frozen {
                if paired.iter().any(|p| p.0 > 0) {
                    self.comparison_frames = self.comparison_frames.saturating_add(1);
                    let rate = 1. / self.comparison_frames.min(64) as f64;
                    let groups = paired.iter().filter(|p| p.0 > 0).count() as f64;
                    let sq = paired
                        .iter()
                        .filter(|p| p.0 > 0)
                        .map(|p| p.1 / p.0 as f64)
                        .sum::<f64>()
                        / groups;
                    let tq = paired
                        .iter()
                        .filter(|p| p.0 > 0)
                        .map(|p| p.2 / p.0 as f64)
                        .sum::<f64>()
                        / groups;
                    self.neural_recent += rate * (sq - self.neural_recent);
                    self.temporal_recent += rate * (tq - self.temporal_recent);
                    for (k, (n, _, _, a, b)) in paired.into_iter().enumerate() {
                        if n > 0 {
                            self.fusion_frames[k] = self.fusion_frames[k].saturating_add(1);
                            let rate = 1. / self.fusion_frames[k].min(64) as f64;
                            self.fusion_a[k] += rate * (a / n as f64 - self.fusion_a[k]);
                            self.fusion_b[k] += rate * (b / n as f64 - self.fusion_b[k]);
                        }
                    }
                    self.neural_paused = self.comparison_frames >= 128
                        && self.temporal_recent < 0.9 * self.neural_recent
                        && self.neural_weights().iter().all(|w| *w < 0.1)
                        && self.reward_recent < 0.05;
                } else {
                    self.neural_paused = false;
                    self.comparison_frames = 0;
                }
                if let Some(t) = self.trial.as_mut() {
                    if !sums.is_empty() {
                        t.frames += 1;
                    }
                    for (kind, (n, b, a)) in sums {
                        t.values += n;
                        t.before += b / n as f64;
                        t.after += a / n as f64;
                        let e = t.by_kind.entry(kind).or_default();
                        e.0 += 1;
                        e.1 += b / n as f64;
                        e.2 += a / n as f64;
                    }
                }
            }
        }
        self.base_rewards.clear();
        self.pending_neural_values.clear();
        self.observe(frame, frozen, meter)?;
        if !frozen {
            if let Some(verdict) = self.trial.as_ref().and_then(RuleTrial::verdict) {
                let t = self.trial.take().ok_or(Error::NoEvidence)?;
                self.receipts.push(RuleReceipt {
                    candidate: t.candidate.clone(),
                    incumbent: self.rule.clone(),
                    proposed_after: t.proposed_after,
                    resolved_after: self.frames,
                    future_frames: t.frames,
                    future_values: t.values,
                    before: t.before,
                    after: t.after,
                    accepted: verdict,
                });
                if self.receipts.len() > 32 {
                    self.receipts.remove(0);
                }
                if verdict {
                    self.rule = t.candidate;
                    for (key, weights) in t.shadow {
                        if let Some(s) = self.models.get_mut(&key) {
                            s.weights = weights;
                        }
                    }
                    self.admissions += 1;
                } else {
                    self.rejections += 1;
                    self.search.remember_rejection();
                }
            }
            if self.config.search {
                let copy_reserve = (self.models.len() * (LAGS * 16 + 64)) as u64;
                let mut search_meter = Meter::new(meter.remaining().saturating_sub(copy_reserve));
                let candidate = self.search.advance(
                    self.raw_values,
                    self.config.recursive,
                    self.trial.is_none(),
                    &self.rule,
                    &mut search_meter,
                )?;
                meter.charge(search_meter.spent)?;
                if let Some(candidate) = candidate {
                    if candidate != self.rule {
                        meter.charge(copy_reserve)?;
                        self.trials = self.trials.saturating_add(1);
                        self.trial = Some(RuleTrial {
                            candidate,
                            proposed_after: self.frames,
                            frames: 0,
                            values: 0,
                            before: 0.,
                            after: 0.,
                            by_kind: BTreeMap::new(),
                            shadow: self
                                .models
                                .iter()
                                .map(|(k, s)| (k.clone(), s.weights.clone()))
                                .collect(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn learned_hash(&self) -> u64 {
        hash(&(
            &self.rule,
            &self.search,
            self.models
                .iter()
                .map(|(k, s)| (k, &s.weights, &s.buffer, &s.buffer_seed, s.first_origin))
                .collect::<Vec<_>>(),
            &self.trial,
            &self.residuals,
            self.neural_paused,
            &self.receipts,
            self.fusion_a,
            self.fusion_b,
            self.fusion_frames,
            self.comparison_frames,
            self.neural_recent,
            self.temporal_recent,
            self.reward_recent,
        ))
    }
    pub(super) fn model_hash(&self) -> u64 {
        hash(&(
            &self.rule,
            self.models
                .iter()
                .map(|(k, s)| (k, &s.weights))
                .collect::<Vec<_>>(),
            &self.residuals,
            self.fusion_a,
            self.fusion_b,
            self.fusion_frames,
        ))
    }
    pub(super) fn report(&self) -> TemporalReport {
        TemporalReport {
            world_forecast_values: self.world_values,
            world_forecast_mse: self.world_error / self.world_values.max(1) as f64,
            neural_prediction_weights: self.neural_weights(),
            accepted_rule_uses: self.rule_uses,
            last_rule_trial: self.receipts.last().cloned(),
            neural_reconstruction_paused: self.neural_paused,
            comparison_frames: self.comparison_frames,
            recent_neural_mse: self.neural_recent,
            recent_temporal_mse: self.temporal_recent,
            config: self.config.clone(),
            raw_values_presented: self.raw_values,
            parameter_updates: self.updates,
            series: self.models.len(),
            unrepresented_values: self.unrepresented,
            committed_frames: self.frames,
            forecast_values: self.values,
            forecast_mse: self.squared_error / self.values.max(1) as f64,
            baseline_mse: self.baseline_error / self.values.max(1) as f64,
            rule: self.rule.clone(),
            rule_trials: self.trials,
            rule_admissions: self.admissions,
            rule_rejections: self.rejections,
            pending_rule_frames: self.trial.as_ref().map(|t| t.frames),
            pending_rule_losses: self.trial.as_ref().map(|t| (t.before, t.after)),
            search: self.search.report(),
            learned_fingerprint: self.learned_hash(),
        }
    }
    pub(super) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(Error::InvalidData);
        }
        valid_rule(&self.rule)?;
        if self.fusion_a.iter().any(|x| !x.is_finite() || *x < 0.)
            || self.fusion_b.iter().any(|x| !x.is_finite())
            || !self.world_error.is_finite()
            || !self.reward_recent.is_finite()
            || self.world_error < 0.
            || self.reward_recent < 0.
            || self.world_values > self.values
            || self.rule_uses > self.updates
            || self.fusion_frames.iter().any(|n| *n > self.frames)
        {
            return Err(Error::InvalidData);
        }
        if self.receipts.len() > 32
            || self.receipts.len() as u64 > self.admissions.saturating_add(self.rejections)
        {
            return Err(Error::InvalidData);
        }
        for r in &self.receipts {
            valid_rule(&r.candidate)?;
            valid_rule(&r.incumbent)?;
            if r.resolved_after > self.frames
                || r.proposed_after > r.resolved_after
                || r.future_frames < 128
                || r.future_frames > r.resolved_after - r.proposed_after
                || !r.before.is_finite()
                || !r.after.is_finite()
                || r.before < 0.
                || r.after < 0.
            {
                return Err(Error::InvalidData);
            }
        }
        if self.comparison_frames > self.frames
            || !self.neural_recent.is_finite()
            || !self.temporal_recent.is_finite()
            || self.neural_recent < 0.
            || self.temporal_recent < 0.
        {
            return Err(Error::InvalidData);
        }
        self.search.validate()?;
        if self.residuals.len() != 16
            || self.residuals.iter().any(|h| {
                h.observations > self.frames
                    || h.weights.len() != 10
                    || h.weights.iter().any(|x| !x.is_finite() || x.abs() > 4.)
            })
        {
            return Err(Error::InvalidData);
        }
        if self.models.len() > SERIES
            || self.updates > self.raw_values
            || self.trials > self.frames
            || self
                .admissions
                .checked_add(self.rejections)
                .and_then(|n| n.checked_add(u64::from(self.trial.is_some())))
                .is_none_or(|n| n != self.trials)
            || ![self.squared_error, self.baseline_error]
                .iter()
                .all(|x| x.is_finite() && *x >= 0.)
        {
            return Err(Error::InvalidData);
        }
        for s in self.models.values() {
            // Default weights need not yet have been normalized by an update.
            if !(s.weights.validate()
                || (s.weights.logits == Weights::default().logits
                    && s.weights.previous_gradient == vec![0.; LAGS]))
                || s.history.len() > LAGS
                || s.buffer.len() >= LESSON_VALUES
                || (s.buffer.is_empty() != s.buffer_seed.is_none())
                || s.buffer_seed.as_ref().is_some_and(|seed| !seed.valid())
                || !s.energy.is_finite()
                || s.energy < 0.
                || s.history
                    .iter()
                    .chain(&s.buffer)
                    .any(|x| !x.is_finite() || x.abs() > 1e6)
            {
                return Err(Error::InvalidData);
            }
        }
        if let Some(t) = &self.trial {
            valid_rule(&t.candidate)?;
            if t.proposed_after > self.frames
                || t.frames > self.frames
                || t.shadow.len() > SERIES
                || t.verdict().is_some()
                || !t.before.is_finite()
                || !t.after.is_finite()
                || t.before < 0.
                || t.after < 0.
                || t.shadow
                    .values()
                    .any(|w| !(w.validate() || w.logits == Weights::default().logits))
            {
                return Err(Error::InvalidData);
            }
        }
        Ok(())
    }
}

fn valid_rule(rule: &Expr) -> Result<()> {
    let mut stack = vec![(rule, 0)];
    let mut n = 0;
    while let Some((e, d)) = stack.pop() {
        n += 1;
        if n > MAX_RULE_NODES || d > 10 {
            return Err(Error::Capacity);
        }
        match e {
            Expr::Input(i) if *i < 6 => {}
            Expr::Lit(crate::construction::vm::Value::Int(i)) if i.abs_diff(0) <= 1_000_000 => {}
            Expr::Lit(crate::construction::vm::Value::Bool(_)) => {}
            Expr::Bin(_, a, b) => {
                stack.push((a, d + 1));
                stack.push((b, d + 1));
            }
            Expr::If(a, b, c) => {
                stack.push((a, d + 1));
                stack.push((b, d + 1));
                stack.push((c, d + 1));
            }
            Expr::Not(a) | Expr::AsInt(a) => stack.push((a, d + 1)),
            _ => return Err(Error::InvalidProgram),
        }
    }
    if rule.ty(&Library::default()) != Ok(crate::construction::vm::Ty::Int) {
        return Err(Error::Type);
    }
    Ok(())
}
