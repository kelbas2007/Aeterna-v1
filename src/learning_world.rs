//! Shared raw-sensory learning state. This module predicts and learns; it has
//! no actuator, environment callback, permission handle or independent loader.
//! Its only owner is ScientificRuntime's joint construction checkpoint.
use crate::construction::{sensors::SensorFrame, Error, Result};
use serde::{Deserialize, Serialize};

mod network;
use network::{Network, Pass};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LearningWorldConfig {
    pub latent: usize,
    pub context_frames: usize,
    pub max_coordinates: usize,
    pub max_frame_values: usize,
    pub memory_entries: usize,
    pub memory_bytes: usize,
    pub work_per_step: u64,
    pub replay_updates: usize,
    pub reconstruction_targets: usize,
    pub learning_rate: f64,
    pub discount: f64,
    pub seed: u64,
}
impl Default for LearningWorldConfig {
    fn default() -> Self {
        Self {
            latent: 16,
            context_frames: 6,
            max_coordinates: 8192,
            max_frame_values: 8192,
            memory_entries: 128,
            memory_bytes: 4 * 1024 * 1024,
            work_per_step: 16_000_000,
            replay_updates: 2,
            reconstruction_targets: 64,
            learning_rate: 0.03,
            discount: 0.95,
            seed: 0x41455445524e41,
        }
    }
}
impl LearningWorldConfig {
    pub fn validate(&self) -> Result<()> {
        if !(4..=64).contains(&self.latent) || !(2..=16).contains(&self.context_frames)
            || !(16..=16384).contains(&self.max_coordinates)
            || !(16..=16384).contains(&self.max_frame_values)
            || !(4..=512).contains(&self.memory_entries)
            || !(16384..=8*1024*1024).contains(&self.memory_bytes)
            || !(100_000..=50_000_000).contains(&self.work_per_step)
            || self.replay_updates>8 || !(8..=256).contains(&self.reconstruction_targets)
            || !self.learning_rate.is_finite() || !(0.0001..=0.1).contains(&self.learning_rate)
            || !self.discount.is_finite() || !(0.0..0.999).contains(&self.discount)
            // Bound dense coordinate state BEFORE allocating it.
            || self.max_coordinates*self.latent>262144
            || (self.max_frame_values*(self.context_frames*(self.latent*4+48)+128)) as u64
                + (self.context_frames*self.latent*self.latent*4+4096) as u64
                > self.work_per_step/2
        {
            return Err(Error::Capacity);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextFrame {
    frame: SensorFrame,
    previous_action: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Experience {
    event: u64,
    context: Vec<ContextFrame>,
    post: ContextFrame,
    reward: f64,
    cost: u64,
    terminal: bool,
    context_key: u64,
    // Stable upper reservation includes variable-sized numerical metadata.
    reserved_bytes: usize,
    last_loss: f64,
    progress: f64,
    replays: u64,
}
impl Experience {
    fn reserve(&self) -> usize {
        self.context
            .iter()
            .chain(std::iter::once(&self.post))
            .map(|f| serde_json::to_vec(&f.frame).map_or(usize::MAX / 32, |b| b.len() + 128))
            .sum::<usize>()
            .saturating_add(4096)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldActionForecast {
    pub action: usize,
    pub reward: f64,
    pub value: f64,
    pub epistemic: f64,
    pub outcome_noise: f64,
    pub support: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningWorldReport {
    pub version: u32,
    pub config: LearningWorldConfig,
    pub attached_after_event: u64,
    pub factual_updates: u64,
    pub representation_version: u64,
    pub online_updates: u64,
    pub consolidation_updates: u64,
    pub retained_experiences: usize,
    pub reserved_memory_bytes: usize,
    pub coordinates: usize,
    pub unrepresented_values: u64,
    pub budget_skipped_updates: u64,
    pub work: u64,
    pub predicted_actions: u64,
    pub selected_proposals: u64,
    pub episodic_predictions: u64,
    pub prequential_reward_count: u64,
    pub prequential_reward_mse: f64,
    pub prequential_sensor_count: u64,
    pub prequential_sensor_mse: f64,
    pub context_frames: usize,
    pub belief: Vec<f64>,
    pub goal_active: bool,
    pub goal_satisfied: Option<bool>,
    pub action_forecasts: Vec<WorldActionForecast>,
    pub learned_fingerprint: u64,
    pub model_fingerprint: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LearningWorld {
    config: LearningWorldConfig,
    network: Network,
    attached_after_event: u64,
    factual_updates: u64,
    representation_version: u64,
    online_updates: u64,
    consolidation_updates: u64,
    memory: Vec<Experience>,
    context: Vec<ContextFrame>,
    bridge: Vec<i64>,
    goal: Option<SensorFrame>,
    goal_tolerance: f64,
    goal_utility: Option<f32>,
    belief: Vec<f64>,
    belief_version: u64,
    rng: u64,
    work: u64,
    step_work: u64,
    unrepresented_values: u64,
    budget_skipped_updates: u64,
    predicted_actions: u64,
    selected_proposals: u64,
    episodic_predictions: u64,
    reward_count: u64,
    reward_squared_error: f64,
    sensor_count: u64,
    sensor_squared_error: f64,
    last_experience: Option<u64>,
    /// An ephemeral prediction commitment, never an executable capability.
    #[serde(skip)]
    pending: Option<Decision>,
}
#[derive(Debug)]
struct Decision {
    pass: Pass,
    forecasts: Vec<WorldActionForecast>,
    version: u64,
    context_key: u64,
}

fn word(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut x = *state;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
fn hash<T: Serialize>(v: &T) -> u64 {
    crate::construction::vm::fingerprint(&serde_json::to_vec(v).expect("finite learning state"))
}
fn kind(frame: &SensorFrame) -> usize {
    match frame {
        SensorFrame::Image { .. } => 0,
        SensorFrame::Signals { .. } => 1,
        SensorFrame::Scene { .. } => 2,
        SensorFrame::Multimodal { .. } => 3,
    }
}
fn namespace(frame: &SensorFrame) -> i64 {
    match frame {
        SensorFrame::Image { namespace, .. } => i64::from(*namespace) * 4,
        SensorFrame::Signals { namespace, .. } => i64::from(*namespace) * 4 + 1,
        SensorFrame::Scene { namespace, .. } => i64::from(*namespace) * 4 + 2,
        SensorFrame::Multimodal { namespace, .. } => i64::from(*namespace) * 4 + 3,
    }
}

/// Coordinates describe physical layout, never namespace, class or motor role.
/// All available pixels/samples reach the trainable encoder, in their order.
fn raw(frame: &SensorFrame, max: usize) -> Result<Vec<(String, f64)>> {
    let mut out = Vec::new();
    match frame {
        SensorFrame::Image { pixels: None, .. } => {}
        SensorFrame::Image {
            pixels: Some(p), ..
        } => {
            let h = p.len();
            let w = p.first().map_or(0, Vec::len);
            if !(5..=256).contains(&h)
                || !(6..=256).contains(&w)
                || h * w > max
                || p.iter().any(|r| r.len() != w)
            {
                return Err(Error::InvalidData);
            }
            for (i, &v) in p.iter().flatten().enumerate() {
                out.push((format!("i:{h}:{w}:{i}"), 2.0 * f64::from(v) / 255.0 - 1.0));
            }
        }
        SensorFrame::Signals { channels, .. } => {
            if channels.len() != 9 {
                return Err(Error::InvalidData);
            }
            let count: usize = channels.iter().flatten().map(Vec::len).sum();
            if count > max {
                return Err(Error::Capacity);
            }
            for (axis, xs) in channels.iter().enumerate() {
                if let Some(xs) = xs {
                    if xs.is_empty() || xs.len() > 2048 {
                        return Err(Error::InvalidData);
                    }
                    for (t, &x) in xs.iter().enumerate() {
                        if !x.is_finite() || x.abs() > 1e6 {
                            return Err(Error::InvalidData);
                        }
                        out.push((format!("s:{axis}:{}:{t}", xs.len()), x / (1.0 + x.abs())));
                    }
                }
            }
        }
        SensorFrame::Scene { pixels: p, .. } => {
            let h = p.len();
            let w = p.first().map_or(0, Vec::len);
            if h == 0
                || w == 0
                || h > 256
                || w > 256
                || h * w > max
                || p.iter().any(|r| r.len() != w)
            {
                return Err(Error::InvalidData);
            }
            for (i, &v) in p.iter().flatten().enumerate() {
                out.push((format!("c:{h}:{w}:{i}"), 2.0 * f64::from(v) / 65535.0 - 1.0));
            }
        }
        SensorFrame::Multimodal {
            namespace: ns,
            frames,
        } => {
            if frames.is_empty() || frames.len() > 3 {
                return Err(Error::InvalidData);
            }
            let mut seen = std::collections::BTreeSet::new();
            for child in frames {
                if matches!(child, SensorFrame::Multimodal { .. })
                    || namespace(child) / 4 != i64::from(*ns)
                    || !seen.insert(kind(child))
                {
                    return Err(Error::InvalidData);
                }
                out.push((format!("m:{}", kind(child)), 1.0));
                out.extend(raw(child, max.saturating_sub(out.len()))?);
                if out.len() > max {
                    return Err(Error::Capacity);
                }
            }
            // Input packet order does not change the represented event.
            out.sort_by(|a, b| a.0.cmp(&b.0));
        }
    }
    Ok(out)
}
pub(crate) fn admit_frame(frame: &SensorFrame, max: usize) -> Result<()> {
    raw(frame, max).map(|_| ())
}
fn context_key(context: &[ContextFrame], max: usize) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for f in context {
        for byte in [
            kind(&f.frame) as u8,
            f.previous_action.map_or(255, |a| a as u8),
        ] {
            h = (h ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
        for (key, value) in raw(&f.frame, max).expect("admitted context") {
            for byte in key
                .bytes()
                .chain(std::iter::once(255))
                .chain(value.to_bits().to_le_bytes())
            {
                h = (h ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
    }
    h
}

impl LearningWorld {
    pub(crate) fn new(config: LearningWorldConfig, actions: usize, event: u64) -> Result<Self> {
        config.validate()?;
        if !(1..=16).contains(&actions) {
            return Err(Error::Capacity);
        }
        let mut rng = config.seed;
        let network = Network::new(config.latent, actions, &mut rng);
        Ok(Self {
            belief: vec![0.0; config.latent],
            network,
            config,
            attached_after_event: event,
            factual_updates: 0,
            representation_version: 0,
            online_updates: 0,
            consolidation_updates: 0,
            memory: Vec::new(),
            context: Vec::new(),
            bridge: Vec::new(),
            goal: None,
            goal_tolerance: 0.01,
            goal_utility: None,
            belief_version: 0,
            rng,
            work: 0,
            step_work: 0,
            unrepresented_values: 0,
            budget_skipped_updates: 0,
            predicted_actions: 0,
            selected_proposals: 0,
            episodic_predictions: 0,
            reward_count: 0,
            reward_squared_error: 0.0,
            sensor_count: 0,
            sensor_squared_error: 0.0,
            last_experience: None,
            pending: None,
        })
    }
    pub(crate) fn check_frame(&self, frame: &SensorFrame) -> Result<()> {
        raw(frame, self.config.max_frame_values).map(|_| ())
    }
    pub(crate) fn frame_limit(&self) -> usize {
        self.config.max_frame_values
    }
    pub(crate) fn total_work(&self) -> u64 {
        self.work
    }
    pub(crate) fn set_goal(&mut self, frame: Option<SensorFrame>, tolerance: f64) -> Result<u64> {
        if !tolerance.is_finite() || !(0.0..=1.0).contains(&tolerance) {
            return Err(Error::InvalidData);
        }
        if let Some(f) = &frame {
            let input = raw(f, self.config.max_frame_values)?;
            if input.is_empty()
                || input
                    .iter()
                    .any(|(key, _)| !self.network.coordinates.contains_key(key))
            {
                return Err(Error::NoEvidence);
            }
        }
        self.goal = frame;
        self.goal_tolerance = tolerance;
        self.pending = None;
        let cost = self.goal_cost();
        self.work = self.work.saturating_add(cost);
        self.goal_utility = self.compute_goal_utility();
        Ok(cost)
    }
    fn compute_goal_utility(&self) -> Option<f32> {
        let target = raw(self.goal.as_ref()?, self.config.max_frame_values).ok()?;
        let actual = raw(&self.context.last()?.frame, self.config.max_frame_values).ok()?;
        let actual: std::collections::BTreeMap<_, _> = actual.into_iter().collect();
        let distance = target
            .iter()
            .map(|(key, value)| actual.get(key).map_or(4.0, |x| (x - value).powi(2)))
            .sum::<f64>()
            / target.len().max(1) as f64;
        Some(f32::from(
            distance <= self.goal_tolerance * self.goal_tolerance,
        ))
    }
    pub(crate) fn goal_utility(&self) -> Option<f32> {
        self.goal_utility
    }
    fn goal_cost(&self) -> u64 {
        self.goal.as_ref().map_or(0, |g| {
            (raw(g, self.config.max_frame_values).map_or(0, |v| v.len())
                + self.context.last().map_or(0, |f| {
                    raw(&f.frame, self.config.max_frame_values).map_or(0, |v| v.len())
                })) as u64
                * 64
        })
    }
    fn refresh_reserve(&self) -> u64 {
        (self.config.max_frame_values
            * (self.config.context_frames * (self.config.latent * 4 + 48) + 128)
            + self.config.context_frames * (self.config.latent * self.config.latent * 4 + 128))
            as u64
    }
    fn charge(&mut self, cost: u64) -> bool {
        if cost > self.config.work_per_step.saturating_sub(self.step_work) {
            return false;
        }
        self.step_work += cost;
        self.work = self.work.saturating_add(cost);
        true
    }
    fn register(&mut self, frames: &[ContextFrame], frozen: bool) {
        if frozen {
            return;
        }
        for f in frames {
            for (key, _) in raw(&f.frame, self.config.max_frame_values).expect("validated frame") {
                if !self.network.coordinates.contains_key(&key) {
                    if self.network.coordinates.len() < self.config.max_coordinates {
                        let cost = (self.config.latent * 32 + 128) as u64;
                        if cost + self.refresh_reserve()
                            <= self.config.work_per_step.saturating_sub(self.step_work)
                            && self.charge(cost)
                        {
                            self.network.add_coordinate(key, &mut self.rng);
                        } else {
                            self.unrepresented_values = self.unrepresented_values.saturating_add(1);
                        }
                    } else {
                        self.unrepresented_values = self.unrepresented_values.saturating_add(1);
                    }
                }
            }
        }
    }
    fn refresh(&mut self) {
        self.belief = self
            .network
            .encode(&self.context, self.config.max_frame_values)
            .last();
        self.belief_version = self.representation_version;
        self.goal_utility = self.compute_goal_utility();
    }
    fn episodic_mix(&self, key: u64, forecasts: &mut [WorldActionForecast]) -> u64 {
        let mut used = 0;
        for p in forecasts {
            let rows: Vec<_> = self
                .memory
                .iter()
                .filter(|s| s.context_key == key && s.post.previous_action == Some(p.action))
                .collect();
            if rows.is_empty() {
                continue;
            }
            let n = rows.len() as f64;
            let mean = rows.iter().map(|s| s.reward).sum::<f64>() / n;
            let variance = rows.iter().map(|s| (s.reward - mean).powi(2)).sum::<f64>() / n;
            let weight = n / (n + 2.0);
            let correction = weight * (mean - p.reward);
            p.reward += correction;
            p.value = (p.value + correction).clamp(-1.0, 1.0);
            p.outcome_noise = (1.0 - weight) * p.outcome_noise + weight * variance;
            p.epistemic = (p.epistemic / (1.0 + n.sqrt())).clamp(0.0, 1.0);
            used += 1;
        }
        used
    }
    pub(crate) fn restart(
        &mut self,
        frame: &SensorFrame,
        bridge: &[i64],
        frozen: bool,
    ) -> Result<u64> {
        self.check_frame(frame)?;
        if bridge.len() != 32 || bridge[0] != namespace(frame) {
            return Err(Error::InvalidData);
        }
        self.step_work = 0;
        self.pending = None;
        self.last_experience = None;
        self.context = vec![ContextFrame {
            frame: frame.clone(),
            previous_action: None,
        }];
        self.bridge = bridge.to_vec();
        self.register(&self.context.clone(), frozen);
        let cost = self
            .network
            .encode_cost(&self.context, self.config.max_frame_values)
            + self.goal_cost();
        // Valid profiles must be able to perceive one admitted frame.
        if !self.charge(cost) {
            return Err(Error::Budget);
        }
        self.refresh();
        Ok(self.step_work)
    }
    pub(crate) fn begin_decision(
        &mut self,
        allowed: &[bool],
        left: Option<usize>,
    ) -> (Vec<WorldActionForecast>, u64) {
        self.pending = None;
        self.step_work = 0;
        if self.context.is_empty() || allowed.len() != self.network.actions {
            return (Vec::new(), 0);
        }
        let goal_frames: Vec<_> = self
            .goal
            .iter()
            .map(|f| ContextFrame {
                frame: f.clone(),
                previous_action: None,
            })
            .collect();
        let cost = self
            .network
            .encode_cost(&self.context, self.config.max_frame_values)
            + self
                .network
                .encode_cost(&goal_frames, self.config.max_frame_values)
            + self.network.forecast_cost()
            + self
                .context
                .iter()
                .map(|f| {
                    raw(&f.frame, self.config.max_frame_values).map_or(0, |v| v.len()) as u64 * 64
                })
                .sum::<u64>()
            + (self.memory.len() * self.network.actions * 16) as u64;
        if cost + self.refresh_reserve() > self.config.work_per_step || !self.charge(cost) {
            return (Vec::new(), 0);
        }
        let pass = self
            .network
            .encode(&self.context, self.config.max_frame_values);
        let goal = (!goal_frames.is_empty()).then(|| {
            self.network
                .encode(&goal_frames, self.config.max_frame_values)
                .last()
        });
        let mut forecasts = self.network.forecasts(
            &pass.last(),
            allowed,
            left,
            self.config.discount,
            goal.as_deref(),
        );
        let key = context_key(&self.context, self.config.max_frame_values);
        self.episodic_predictions += self.episodic_mix(key, &mut forecasts);
        self.belief = pass.last();
        self.belief_version = self.representation_version;
        self.predicted_actions = self
            .predicted_actions
            .saturating_add(forecasts.len() as u64);
        self.pending = Some(Decision {
            pass,
            forecasts: forecasts.clone(),
            version: self.representation_version,
            context_key: key,
        });
        (forecasts, cost)
    }
    pub(crate) fn cancel_decision(&mut self) {
        self.pending = None;
    }
    pub(crate) fn committed(
        &mut self,
        event: u64,
        action: usize,
        post: &SensorFrame,
        bridge: &[i64],
        reward: f64,
        cost: u64,
        frozen: bool,
        selected: bool,
    ) -> Result<u64> {
        self.check_frame(post)?;
        if bridge.len() != 32
            || bridge[0] != namespace(post)
            || action >= self.network.actions
            || !reward.is_finite()
            || !(-1.0..=1.0).contains(&reward)
        {
            return Err(Error::InvalidData);
        }
        let start = self.work;
        let pending = self.pending.take();
        let indexing = self
            .context
            .iter()
            .map(|f| raw(&f.frame, self.config.max_frame_values).map_or(0, |v| v.len()) as u64 * 64)
            .sum::<u64>();
        let key = if let Some(pre) = &pending {
            Some(pre.context_key)
        } else if !frozen
            && indexing + self.refresh_reserve()
                <= self.config.work_per_step.saturating_sub(self.step_work)
            && self.charge(indexing)
        {
            Some(context_key(&self.context, self.config.max_frame_values))
        } else {
            None
        };
        let predicted_reward = pending
            .as_ref()
            .and_then(|p| p.forecasts.iter().find(|p| p.action == action))
            .map(|p| p.reward);
        if !frozen {
            self.network
                .observe_outcome(action, predicted_reward.map(|p| p - reward));
        }
        let mut sample = Experience {
            event,
            context: self.context.clone(),
            post: ContextFrame {
                frame: post.clone(),
                previous_action: Some(action),
            },
            reward,
            cost,
            terminal: false,
            context_key: key.unwrap_or(0),
            reserved_bytes: 0,
            last_loss: 0.0,
            progress: 0.0,
            replays: 0,
        };
        if let Some(pre) = pending {
            if pre.version != self.representation_version {
                return Err(Error::Stale);
            }
            if let Some(p) = pre.forecasts.iter().find(|p| p.action == action) {
                self.reward_squared_error += (p.reward - reward).powi(2);
                self.reward_count += 1;
            }
            let values = raw(post, self.config.max_frame_values)?;
            let metric_cost = (values.len() * (self.config.latent * 4 + 64)
                + self.config.latent * self.config.latent * 8) as u64;
            if metric_cost + self.refresh_reserve()
                <= self.config.work_per_step.saturating_sub(self.step_work)
                && self.charge(metric_cost)
            {
                let (sq, n) = self.network.sensor_error(&pre.pass.last(), action, &values);
                self.sensor_squared_error += sq;
                self.sensor_count += n;
                sample.last_loss = sq / n.max(1) as f64;
            }
        }
        self.factual_updates += 1;
        if selected {
            self.selected_proposals += 1;
        }
        if !frozen {
            self.register(std::slice::from_ref(&sample.post), false);
            self.train(&sample, false);
            sample.reserved_bytes = sample.reserve();
            if key.is_some() {
                self.remember(sample);
                self.last_experience = Some(event);
            }
            for _ in 0..self.config.replay_updates {
                if !self.replay_one() {
                    break;
                }
            }
        }
        self.context.push(ContextFrame {
            frame: post.clone(),
            previous_action: Some(action),
        });
        while self.context.len() > self.config.context_frames {
            self.context.remove(0);
        }
        self.bridge = bridge.to_vec();
        // Re-encode retained raw context after representation updates; no stale
        // latent vectors are ever treated as current episodic embeddings.
        let refresh = self
            .network
            .encode_cost(&self.context, self.config.max_frame_values)
            + self.goal_cost();
        // Reserve for this before any training in train().
        self.work = self.work.saturating_add(refresh);
        self.step_work = self.step_work.saturating_add(refresh);
        self.refresh();
        Ok(self.work - start)
    }
    fn train(&mut self, sample: &Experience, replay: bool) -> bool {
        let cost = self.network.training_cost(sample, &self.config);
        // Always leave enough for recomputing the largest live context.
        let reserve = self.refresh_reserve();
        if cost.saturating_add(reserve) > self.config.work_per_step.saturating_sub(self.step_work) {
            self.budget_skipped_updates += 1;
            return false;
        }
        if !self.charge(cost) {
            return false;
        }
        let loss = self.network.train(sample, &self.config, &mut self.rng);
        if replay {
            self.consolidation_updates += 1;
        } else {
            self.online_updates += 1;
        }
        self.representation_version += 1;
        if let Some(old) = self.memory.iter_mut().find(|s| s.event == sample.event) {
            old.progress = 0.9 * old.progress + 0.1 * (old.last_loss - loss).clamp(-4.0, 4.0);
            old.last_loss = loss;
            old.replays += u64::from(replay);
        }
        true
    }
    fn remember(&mut self, sample: Experience) {
        if sample.reserved_bytes > self.config.memory_bytes {
            return;
        }
        self.memory.push(sample);
        while self.memory.len() > self.config.memory_entries
            || self.memory.iter().map(|s| s.reserved_bytes).sum::<usize>()
                > self.config.memory_bytes
        {
            // Preserve scarce sensory modalities; oldest entry in the most
            // represented physical modality leaves first. No corpus IDs.
            let mut counts = [0usize; 4];
            for s in &self.memory {
                counts[kind(&s.post.frame)] += 1;
            }
            let group = (0..4).max_by_key(|&k| counts[k]).unwrap();
            let victim = self
                .memory
                .iter()
                .position(|s| kind(&s.post.frame) == group)
                .unwrap();
            self.memory.remove(victim);
        }
    }
    fn replay_one(&mut self) -> bool {
        if self.memory.len() < 2 {
            return false;
        }
        // Alternate progress-prioritized access with uniform access so noisy
        // or low-progress episodes cannot permanently monopolize replay.
        let index = if self.consolidation_updates % 2 == 0 {
            word(&mut self.rng) as usize % self.memory.len()
        } else {
            self.memory
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    (a.progress.max(0.0) + 1.0 / (1.0 + a.replays as f64))
                        .total_cmp(&(b.progress.max(0.0) + 1.0 / (1.0 + b.replays as f64)))
                })
                .map(|(i, _)| i)
                .unwrap()
        };
        let sample = self.memory[index].clone();
        self.train(&sample, true)
    }
    pub(crate) fn finish_episode(&mut self, frozen: bool) -> u64 {
        self.pending = None;
        if frozen {
            return 0;
        }
        let start = self.work;
        self.step_work = 0;
        if let Some(event) = self.last_experience.take() {
            if let Some(index) = self.memory.iter().position(|s| s.event == event) {
                self.memory[index].terminal = true;
                let sample = self.memory[index].clone();
                self.train(&sample, true);
                let cost = self
                    .network
                    .encode_cost(&self.context, self.config.max_frame_values)
                    + self.goal_cost();
                self.work = self.work.saturating_add(cost);
                self.step_work += cost;
                self.refresh();
            }
        }
        self.work - start
    }
    pub(crate) fn validate(&self, current: &[i64], factual_events: u64) -> Result<()> {
        self.config.validate()?;
        self.network.validate(&self.config)?;
        if self.context.is_empty()
            || self.context.len() > self.config.context_frames
            || self.bridge != current
            || self.bridge.len() != 32
            || self.belief.len() != self.config.latent
            || self.belief_version != self.representation_version
            || self.representation_version != self.online_updates + self.consolidation_updates
            || self.online_updates > self.factual_updates
            || self.reward_count > self.factual_updates
            || self.selected_proposals > self.factual_updates
            || self.step_work > self.config.work_per_step
            || self.step_work > self.work
            || self.consolidation_updates
                > self
                    .factual_updates
                    .saturating_mul(self.config.replay_updates as u64 + 1)
            || self.attached_after_event.checked_add(self.factual_updates) != Some(factual_events)
            || self.memory.len() > self.config.memory_entries
            || self
                .memory
                .iter()
                .map(|s| s.reserved_bytes as u64)
                .sum::<u64>()
                > self.config.memory_bytes as u64
            || !self.reward_squared_error.is_finite()
            || self.reward_squared_error < 0.0
            || !self.sensor_squared_error.is_finite()
            || self.sensor_squared_error < 0.0
            || self.belief.iter().any(|x| !x.is_finite() || x.abs() > 1.0)
            || self
                .context
                .last()
                .is_none_or(|f| namespace(&f.frame) != current[0])
        {
            return Err(Error::InvalidData);
        }
        let valid = |f: &ContextFrame| -> Result<()> {
            if f.previous_action.is_some_and(|a| a >= self.network.actions) {
                return Err(Error::InvalidData);
            }
            self.check_frame(&f.frame)
        };
        for f in &self.context {
            valid(f)?;
        }
        if !self.goal_tolerance.is_finite() || !(0.0..=1.0).contains(&self.goal_tolerance) {
            return Err(Error::InvalidData);
        }
        if let Some(goal) = &self.goal {
            self.check_frame(goal)?;
        }
        let mut events = std::collections::BTreeSet::new();
        for s in &self.memory {
            if s.context.is_empty()
                || s.context.len() > self.config.context_frames
                || s.event <= self.attached_after_event
                || s.event > factual_events
                || !events.insert(s.event)
                || s.post.previous_action.is_none()
                || s.reserved_bytes != s.reserve()
                || s.context_key != context_key(&s.context, self.config.max_frame_values)
                || !s.reward.is_finite()
                || !(-1.0..=1.0).contains(&s.reward)
                || !s.last_loss.is_finite()
                || s.last_loss < 0.0
                || !s.progress.is_finite()
                || s.progress.abs() > 4.0
            {
                return Err(Error::InvalidData);
            }
            for f in s.context.iter().chain(std::iter::once(&s.post)) {
                valid(f)?;
            }
        }
        if self
            .network
            .encode(&self.context, self.config.max_frame_values)
            .last()
            .iter()
            .zip(&self.belief)
            .any(|(a, b)| (a - b).abs() > 1e-12)
            || self.compute_goal_utility() != self.goal_utility
        {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    pub(crate) fn report(&self) -> LearningWorldReport {
        let goal = self.goal.as_ref().map(|f| {
            self.network
                .encode(
                    &[ContextFrame {
                        frame: f.clone(),
                        previous_action: None,
                    }],
                    self.config.max_frame_values,
                )
                .last()
        });
        let mut action_forecasts = self.network.forecasts(
            &self.belief,
            &vec![true; self.network.actions],
            None,
            self.config.discount,
            goal.as_deref(),
        );
        self.episodic_mix(
            context_key(&self.context, self.config.max_frame_values),
            &mut action_forecasts,
        );
        LearningWorldReport {
            version: 1,
            config: self.config.clone(),
            attached_after_event: self.attached_after_event,
            factual_updates: self.factual_updates,
            representation_version: self.representation_version,
            online_updates: self.online_updates,
            consolidation_updates: self.consolidation_updates,
            retained_experiences: self.memory.len(),
            reserved_memory_bytes: self.memory.iter().map(|s| s.reserved_bytes).sum(),
            coordinates: self.network.coordinates.len(),
            unrepresented_values: self.unrepresented_values,
            budget_skipped_updates: self.budget_skipped_updates,
            work: self.work,
            predicted_actions: self.predicted_actions,
            selected_proposals: self.selected_proposals,
            episodic_predictions: self.episodic_predictions,
            prequential_reward_count: self.reward_count,
            prequential_reward_mse: self.reward_squared_error / self.reward_count.max(1) as f64,
            prequential_sensor_count: self.sensor_count,
            prequential_sensor_mse: self.sensor_squared_error / self.sensor_count.max(1) as f64,
            context_frames: self.context.len(),
            belief: self.belief.clone(),
            goal_active: self.goal.is_some(),
            goal_satisfied: self.goal_utility.map(|v| v > 0.0),
            action_forecasts,
            learned_fingerprint: hash(&(
                &self.network,
                &self.memory,
                self.representation_version,
                self.rng,
            )),
            model_fingerprint: hash(&self.network),
        }
    }
}
