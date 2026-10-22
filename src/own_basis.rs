//! Constructive perceptual learning from explicit teaching facts.
//! All parameters start locally. Frozen feature programs are shared between
//! named learning goals; only this module's owner may admit a new program.
//! It has no actuator. Teaching is not an unlabelled physical experiment.
use crate::construction::{sensors::SensorFrame, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BasisConfig {
    pub grow: bool,
    pub reuse_features: bool,
    pub max_coordinates: usize,
    pub max_features: usize,
    pub max_parameters: usize,
    pub memory_examples: usize,
    pub memory_bytes: usize,
    pub work_per_example: u64,
    pub growth_units: usize,
    pub seed: u64,
}
impl Default for BasisConfig {
    fn default() -> Self {
        Self {
            grow: true,
            reuse_features: true,
            max_coordinates: 2048,
            max_features: 64,
            max_parameters: 262144,
            memory_examples: 256,
            memory_bytes: 4 * 1024 * 1024,
            work_per_example: 8_000_000,
            growth_units: 16,
            seed: 0x424153495331,
        }
    }
}
impl BasisConfig {
    fn validate(&self) -> Result<()> {
        if !(16..=4096).contains(&self.max_coordinates)
            || self.max_features > 128
            || !(4096..=524288).contains(&self.max_parameters)
            || !(32..=512).contains(&self.memory_examples)
            || !(65536..=8 * 1024 * 1024).contains(&self.memory_bytes)
            || !(100_000..=50_000_000).contains(&self.work_per_example)
            || self.work_per_example < self.max_coordinates as u64 * 128
            || !(1..=32).contains(&self.growth_units)
        {
            return Err(Error::Capacity);
        }
        Ok(())
    }
}
fn word(rng: &mut u64) -> u64 {
    *rng = rng.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *rng;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
fn hash<T: Serialize>(v: &T) -> u64 {
    crate::construction::vm::fingerprint(&serde_json::to_vec(v).expect("finite basis state"))
}
fn name_ok(s: &str) -> bool {
    !s.is_empty() && s.len() <= 96 && !s.chars().any(char::is_control)
}
fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut p: Vec<_> = logits.iter().map(|v| (v - max).exp()).collect();
    let sum = p.iter().sum::<f64>().max(1e-30);
    for v in &mut p {
        *v /= sum;
    }
    p
}
fn winner(p: &[f64]) -> Option<usize> {
    p.iter()
        .enumerate()
        .max_by(|(a, x), (b, y)| x.total_cmp(y).then_with(|| b.cmp(a)))
        .map(|(i, _)| i)
}

/// An acquired executable composition. Parent node IDs refer only backwards.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureProgram {
    pub first_node: usize,
    pub parents: Vec<usize>,
    pub units: usize,
    pub weights: Vec<f64>,
    pub learned_for: String,
    pub admitted_after_teaching_event: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Library {
    coordinates: Vec<String>,
    programs: Vec<FeatureProgram>,
}
impl Library {
    fn units(&self) -> usize {
        self.programs.iter().map(|p| p.units).sum()
    }
    fn base(&self, x: &[f32], mask: Option<&[bool]>) -> Vec<f64> {
        // Unit masks would create thousands of identical bias parameters.
        let presence = 1. / (x.len().max(1) as f64).sqrt();
        x.iter()
            .map(|v| f64::from(*v))
            .chain((0..x.len()).map(|i| {
                if mask.is_none_or(|m| m[i]) {
                    presence
                } else {
                    0.
                }
            }))
            .collect()
    }
    fn activations(
        &self,
        x: &[f32],
        mask: Option<&[bool]>,
        extra: Option<&FeatureProgram>,
    ) -> Vec<f64> {
        let base = self.base(x, mask);
        let mut values = Vec::new();
        for p in self.programs.iter().chain(extra) {
            let input: Vec<_> = base
                .iter()
                .copied()
                .chain(p.parents.iter().map(|&i| values[i]))
                .chain([1.])
                .collect();
            for unit in 0..p.units {
                let row = &p.weights[unit * input.len()..(unit + 1) * input.len()];
                values.push(
                    row.iter()
                        .zip(&input)
                        .map(|(a, b)| a * b)
                        .sum::<f64>()
                        .tanh(),
                );
            }
        }
        values
    }
    fn features(
        &self,
        x: &[f32],
        mask: Option<&[bool]>,
        visible: &[usize],
        extra: Option<&FeatureProgram>,
    ) -> Vec<f64> {
        let h = self.activations(x, mask, extra);
        self.base(x, mask)
            .into_iter()
            .chain(visible.iter().map(|&i| h[i]))
            .chain([1.])
            .collect()
    }
    fn forward_cost(&self, extra: Option<&FeatureProgram>) -> u64 {
        128 + self
            .programs
            .iter()
            .chain(extra)
            .map(|p| (p.weights.len() * 4 + p.units * 64) as u64)
            .sum::<u64>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Head {
    visible: Vec<usize>,
    classes: usize,
    weights: Vec<f64>,
}
impl Head {
    fn new(width: usize, visible: Vec<usize>) -> Self {
        Self {
            visible,
            classes: 0,
            weights: Vec::with_capacity(width),
        }
    }
    fn width(&self, coordinates: usize) -> usize {
        coordinates * 2 + self.visible.len() + 1
    }
    fn add_class(&mut self, coordinates: usize) {
        self.weights
            .extend(std::iter::repeat_n(0., self.width(coordinates)));
        self.classes += 1;
    }
    fn probabilities(&self, features: &[f64]) -> Vec<f64> {
        if self.classes == 0 {
            return Vec::new();
        }
        softmax(
            &(0..self.classes)
                .map(|c| {
                    self.weights[c * features.len()..(c + 1) * features.len()]
                        .iter()
                        .zip(features)
                        .map(|(w, x)| w * x)
                        .sum()
                })
                .collect::<Vec<_>>(),
        )
    }
    fn expanded(&self, coordinates: usize, visible: Vec<usize>) -> Self {
        let old = self.width(coordinates);
        let new = coordinates * 2 + visible.len() + 1;
        let mut weights = vec![0.; new * self.classes];
        for c in 0..self.classes {
            weights[c * new..c * new + coordinates * 2]
                .copy_from_slice(&self.weights[c * old..c * old + coordinates * 2]);
            for (j, id) in self.visible.iter().enumerate() {
                if let Some(i) = visible.iter().position(|x| x == id) {
                    weights[c * new + coordinates * 2 + i] =
                        self.weights[c * old + coordinates * 2 + j];
                }
            }
            weights[c * new + new - 1] = self.weights[c * old + old - 1];
        }
        Self {
            visible,
            classes: self.classes,
            weights,
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Adam {
    mean: Vec<f64>,
    square: Vec<f64>,
    step: u64,
}
impl Adam {
    fn update(&mut self, parameters: &mut [f64], gradient: &[f64]) {
        if self.mean.len() != parameters.len() {
            self.mean = vec![0.; parameters.len()];
            self.square = vec![0.; parameters.len()];
            self.step = 0;
        }
        self.step = self.step.saturating_add(1);
        let m = 1. - 0.9_f64.powf(self.step as f64);
        let v = 1. - 0.999_f64.powf(self.step as f64);
        for i in 0..parameters.len() {
            let g = gradient[i].clamp(-10., 10.);
            self.mean[i] = 0.9 * self.mean[i] + 0.1 * g;
            self.square[i] = 0.999 * self.square[i] + 0.001 * g * g;
            parameters[i] = (parameters[i]
                - 0.003 * (self.mean[i] / m) / ((self.square[i] / v).sqrt() + 1e-8))
                .clamp(-8., 8.);
        }
    }
    fn valid(&self, n: usize) -> bool {
        (self.mean.is_empty() && self.square.is_empty() && self.step == 0)
            || (self.mean.len() == n
                && self.square.len() == n
                && self
                    .mean
                    .iter()
                    .all(|v| v.is_finite() && v.abs() <= 10.000001)
                && self
                    .square
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=100.000001).contains(v)))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Example {
    values: Vec<f32>,
    class: usize,
    origin: u64,
    fingerprint: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Trial {
    head: Head,
    optimizer: Adam,
    program: Option<FeatureProgram>,
    program_optimizer: Adam,
    proposed_after: u64,
    warmup: u64,
    future: u64,
    before_loss: f64,
    after_loss: f64,
    before_correct: u64,
    after_correct: u64,
    first_future: Option<u64>,
    last_future: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthReceipt {
    pub skill: String,
    pub proposed_after: u64,
    pub first_future: Option<u64>,
    pub last_future: u64,
    pub future_examples: u64,
    pub before_loss: f64,
    pub after_loss: f64,
    pub before_correct: u64,
    pub after_correct: u64,
    pub new_features: usize,
    pub accepted: bool,
    pub interrupted: bool,
}
impl GrowthReceipt {
    fn verdict(&self) -> bool {
        !self.interrupted
            && self.future_examples >= 128
            && self.before_loss > 0.
            && self.after_loss <= 0.95 * self.before_loss
            && self.after_correct >= self.before_correct
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Skill {
    library: u64,
    classes: Vec<String>,
    head: Head,
    optimizer: Adam,
    examples: Vec<Example>,
    seen: u64,
    distinct: u64,
    bloom: Vec<u64>,
    candidate: Option<Trial>,
    next_growth: u64,
    predicted: u64,
    correct: u64,
    loss: f64,
}
impl Skill {
    fn new(library: u64, coordinates: usize, visible: Vec<usize>) -> Self {
        Self {
            library,
            classes: Vec::new(),
            head: Head::new(coordinates, visible),
            optimizer: Adam::default(),
            examples: Vec::new(),
            seen: 0,
            distinct: 0,
            bloom: vec![0; 1024],
            candidate: None,
            next_growth: 256,
            predicted: 0,
            correct: 0,
            loss: 0.,
        }
    }
    fn new_input(&mut self, h: u64) -> bool {
        let mut fresh = false;
        for k in 0..4 {
            let bit = (h.rotate_left(k * 13).wrapping_mul(0x9e3779b97f4a7c15) as usize) % 65536;
            fresh |= self.bloom[bit / 64] & (1_u64 << (bit % 64)) == 0;
            self.bloom[bit / 64] |= 1_u64 << (bit % 64);
        }
        fresh
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasisPrediction {
    pub skill: String,
    pub probabilities: BTreeMap<String, f64>,
    pub coverage: f64,
    pub teaching_examples: u64,
    pub visible_features: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasisGoal {
    pub skill: String,
    pub answers: Vec<BasisAnswer>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasisAnswer {
    pub motor: usize,
    pub concept: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReport {
    pub name: String,
    pub concepts: Vec<String>,
    pub teaching_examples: u64,
    pub distinct_inputs: u64,
    pub visible_features: usize,
    pub reused_features: usize,
    pub prequential_predictions: u64,
    pub prequential_correct: u64,
    pub prequential_log_loss: f64,
    pub candidate: bool,
    pub model_fingerprint: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasisReport {
    pub config: BasisConfig,
    pub teaching_events: u64,
    pub work: u64,
    pub parameters: usize,
    pub stored_examples: usize,
    pub programs: usize,
    pub features: usize,
    pub admitted_growth: u64,
    pub rejected_growth: u64,
    pub trials: u64,
    pub frozen: bool,
    pub skills: Vec<SkillReport>,
    pub last_growth: Option<GrowthReceipt>,
    pub learned_fingerprint: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingReceipt {
    pub teaching_event: u64,
    pub skill: String,
    pub distinct_input: bool,
    pub prediction_before: Option<BasisPrediction>,
    pub work: u64,
    pub source_id: String,
    pub input_fingerprint: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OwnBasis {
    version: u32,
    config: BasisConfig,
    libraries: BTreeMap<u64, Library>,
    skills: BTreeMap<String, Skill>,
    rng: u64,
    events: u64,
    work: u64,
    trials: u64,
    admitted: u64,
    rejected: u64,
    receipts: Vec<GrowthReceipt>,
    frozen: bool,
    goal: Option<BasisGoal>,
    context: Option<SensorFrame>,
}
const BATCH: usize = 8;
impl OwnBasis {
    pub(crate) fn new(config: BasisConfig) -> Result<Self> {
        config.validate()?;
        let rng = config.seed;
        Ok(Self {
            version: 1,
            config,
            libraries: BTreeMap::new(),
            skills: BTreeMap::new(),
            rng,
            events: 0,
            work: 0,
            trials: 0,
            admitted: 0,
            rejected: 0,
            receipts: Vec::new(),
            frozen: false,
            goal: None,
            context: None,
        })
    }
    pub(crate) fn total_work(&self) -> u64 {
        self.work
    }
    pub(crate) fn frame_limit(&self) -> usize {
        self.config.max_coordinates
    }
    pub(crate) fn check_frame(&self, frame: &SensorFrame) -> Result<()> {
        crate::learning_world::admit_frame(frame, self.config.max_coordinates)
    }
    fn extract(&self, frame: &SensorFrame) -> Result<Vec<(String, f64)>> {
        let x = crate::learning_world::raw(frame, self.config.max_coordinates)?;
        if x.is_empty() {
            return Err(Error::NoEvidence);
        }
        Ok(x)
    }
    fn encode(library: &Library, values: &[(String, f64)]) -> (Vec<f32>, Vec<bool>, f64) {
        let values: BTreeMap<_, _> = values.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        let mut x = Vec::new();
        let mut mask = Vec::new();
        let mut count = 0;
        for k in &library.coordinates {
            let value = values.get(k.as_str());
            x.push(value.copied().unwrap_or(0.) as f32);
            mask.push(value.is_some());
            count += usize::from(value.is_some());
        }
        let coverage = count as f64 / library.coordinates.len().max(1) as f64;
        (x, mask, coverage)
    }
    fn predict_inner(&self, name: &str, frame: &SensorFrame) -> Result<(BasisPrediction, u64)> {
        let skill = self.skills.get(name).ok_or(Error::NoEvidence)?;
        let library = &self.libraries[&skill.library];
        let values = self.extract(frame)?;
        let (x, mask, coverage) = Self::encode(library, &values);
        if coverage == 0. || skill.classes.is_empty() {
            return Err(Error::NoEvidence);
        }
        let f = library.features(&x, Some(&mask), &skill.head.visible, None);
        let probabilities = skill
            .classes
            .iter()
            .cloned()
            .zip(skill.head.probabilities(&f))
            .collect();
        let cost =
            library.forward_cost(None) + (skill.head.weights.len() * 4 + x.len() * 16) as u64;
        Ok((
            BasisPrediction {
                skill: name.into(),
                probabilities,
                coverage,
                teaching_examples: skill.seen,
                visible_features: skill.head.visible.len(),
            },
            cost,
        ))
    }
    pub(crate) fn predict(&mut self, name: &str, frame: &SensorFrame) -> Result<BasisPrediction> {
        let (p, work) = self.predict_inner(name, frame)?;
        self.work = self.work.saturating_add(work);
        Ok(p)
    }
    pub(crate) fn observe_context(&mut self, frame: &SensorFrame) {
        self.context = Some(frame.clone());
    }
    pub(crate) fn set_goal(&mut self, goal: Option<BasisGoal>, actions: usize) -> Result<()> {
        if let Some(g) = &goal {
            let s = self.skills.get(&g.skill).ok_or(Error::NoEvidence)?;
            if g.answers.is_empty() || g.answers.len() > actions {
                return Err(Error::InvalidData);
            }
            let mut motors = std::collections::BTreeSet::new();
            for a in &g.answers {
                if a.motor >= actions || !motors.insert(a.motor) || !s.classes.contains(&a.concept)
                {
                    return Err(Error::InvalidData);
                }
            }
        }
        self.goal = goal;
        Ok(())
    }
    pub(crate) fn goal_forecasts(
        &mut self,
        allowed: &[bool],
    ) -> Option<(Vec<(usize, f64)>, f64, u64)> {
        let goal = self.goal.as_ref()?;
        let frame = self.context.as_ref()?;
        let (p, cost) = self.predict_inner(&goal.skill, frame).ok()?;
        self.work = self.work.saturating_add(cost);
        let answers = goal
            .answers
            .iter()
            .filter(|a| allowed[a.motor])
            .map(|a| {
                (
                    a.motor,
                    p.probabilities.get(&a.concept).copied().unwrap_or(0.),
                )
            })
            .collect::<Vec<_>>();
        // Removing a motor cannot make an incorrect concept certain.
        Some((answers, p.coverage, p.teaching_examples))
    }
    fn parameters(&self) -> usize {
        self.libraries
            .values()
            .flat_map(|l| &l.programs)
            .map(|p| p.weights.len())
            .sum::<usize>()
            + self
                .skills
                .values()
                .map(|s| {
                    s.head.weights.len()
                        + s.candidate.as_ref().map_or(0, |c| {
                            c.head.weights.len() + c.program.as_ref().map_or(0, |p| p.weights.len())
                        })
                })
                .sum::<usize>()
    }
    fn sample_bytes(&self) -> usize {
        // Payload budget uses the in-memory f32 representation plus a
        // conservative fixed record overhead; checkpoint has its own byte cap.
        self.skills
            .values()
            .flat_map(|s| &s.examples)
            .map(|e| e.values.len() * 4 + 64)
            .sum()
    }
    fn compact(&mut self, current: &str) {
        while self.sample_bytes() > self.config.memory_bytes {
            let key = self
                .skills
                .iter()
                .filter(|(_, s)| !s.examples.is_empty())
                .min_by_key(|(name, s)| (name.as_str() == current, s.examples[0].origin))
                .map(|(k, _)| k.clone());
            let Some(k) = key else { break };
            self.skills.get_mut(&k).unwrap().examples.remove(0);
        }
    }
    pub(crate) fn teach(
        &mut self,
        name: &str,
        frame: &SensorFrame,
        label: &str,
        source_id: &str,
    ) -> Result<TeachingReceipt> {
        if self.frozen {
            return Err(Error::Blocked);
        }
        if !name_ok(name) || !name_ok(label) || source_id.len() > 192 {
            return Err(Error::InvalidData);
        }
        let raw = self.extract(frame)?;
        let keys: Vec<_> = raw.iter().map(|(k, _)| k.clone()).collect();
        let layout = hash(&keys);
        if let Some(s) = self.skills.get(name) {
            if s.library != layout {
                return Err(Error::InvalidData);
            }
        } else if self.skills.len() >= 16 {
            return Err(Error::Capacity);
        }
        let new_class = self
            .skills
            .get(name)
            .is_none_or(|s| !s.classes.iter().any(|c| c == label));
        if self.skills.get(name).is_some_and(|s| s.classes.len() >= 64) && new_class {
            return Err(Error::Capacity);
        }
        if self.parameters()
            + if new_class {
                keys.len() * 2 + self.config.max_features + 1
            } else {
                0
            }
            > self.config.max_parameters
        {
            return Err(Error::Capacity);
        }
        // Reject unaffordable lessons before vocabulary, replay or RNG changes.
        let library = self.libraries.get(&layout);
        let forward = library.map_or(128, |l| l.forward_cost(None));
        let existing = self.skills.get(name);
        let visible = existing.map_or_else(
            || {
                if self.config.reuse_features {
                    library.map_or(0, Library::units)
                } else {
                    0
                }
            },
            |s| s.head.visible.len(),
        );
        let width = raw.len() * 2 + visible + 1;
        let weights = existing.map_or(0, |s| s.head.weights.len()) + usize::from(new_class) * width;
        let prediction = existing.map_or(0, |s| {
            forward + (s.head.weights.len() * 4 + raw.len() * 16) as u64
        });
        let child = existing.and_then(|s| {
            if new_class {
                None
            } else {
                s.candidate.as_ref()
            }
        });
        let child_forward = child.map_or(0, |c| {
            library.unwrap().forward_cost(c.program.as_ref()) + (c.head.weights.len() * 4) as u64
        });
        let overhead = (raw.len() * 64 + weights * 4) as u64 + forward + prediction + child_forward;
        let updates = if existing.map_or(1, |s| s.examples.len() + 1) >= BATCH {
            BATCH as u64 * (forward + (weights * 5) as u64)
                + (weights * 24) as u64
                + 1024
                + child.map_or(0, |c| {
                    update_cost(library.unwrap(), &c.head, c.program.as_ref())
                })
        } else {
            0
        };
        if overhead.saturating_add(updates) > self.config.work_per_example {
            return Err(Error::Budget);
        }
        self.libraries.entry(layout).or_insert_with(|| Library {
            coordinates: keys,
            programs: Vec::new(),
        });
        let visible = if self.config.reuse_features {
            (0..self.libraries[&layout].units()).collect()
        } else {
            Vec::new()
        };
        self.skills
            .entry(name.into())
            .or_insert_with(|| Skill::new(layout, raw.len(), visible));
        let (before, prediction_cost) = self
            .predict_inner(name, frame)
            .map_or((None, 0), |(p, w)| (Some(p), w));
        let x: Vec<_> = raw.iter().map(|(_, v)| *v as f32).collect();
        let fingerprint = hash(&x);
        self.events = self.events.saturating_add(1);
        let event = self.events;
        let mut skill = self.skills.remove(name).unwrap();
        let class = if let Some(c) = skill.classes.iter().position(|v| v == label) {
            c
        } else {
            if let Some(trial) = skill.candidate.take() {
                self.close_trial(name, trial, true);
            }
            skill.classes.push(label.into());
            skill.head.add_class(x.len());
            skill.classes.len() - 1
        };
        let fresh = skill.new_input(fingerprint);
        skill.distinct += u64::from(fresh);
        skill.seen += 1;
        let library = self.libraries.get_mut(&layout).unwrap();
        let f = library.features(&x, None, &skill.head.visible, None);
        let old = skill.head.probabilities(&f);
        if let Some(p) = before.as_ref().and_then(|p| p.probabilities.get(label)) {
            skill.predicted += 1;
            skill.loss -= p.max(1e-12).ln();
            skill.correct += u64::from(
                before
                    .as_ref()
                    .unwrap()
                    .probabilities
                    .iter()
                    .max_by(|(a, x), (b, y)| x.total_cmp(y).then_with(|| b.cmp(a)))
                    .map(|(c, _)| c.as_str())
                    == Some(label),
            );
        }
        if let Some(c) = skill.candidate.as_mut() {
            if c.warmup >= 64 && fresh {
                let f = library.features(&x, None, &c.head.visible, c.program.as_ref());
                let new = c.head.probabilities(&f);
                c.future += 1;
                c.before_loss -= old[class].max(1e-12).ln();
                c.after_loss -= new[class].max(1e-12).ln();
                c.before_correct += u64::from(winner(&old) == Some(class));
                c.after_correct += u64::from(winner(&new) == Some(class));
                c.first_future.get_or_insert(event);
                c.last_future = event;
            }
            c.warmup += u64::from(fresh);
        }
        let e = Example {
            values: x,
            class,
            origin: event,
            fingerprint,
        };
        if skill.examples.len() < self.config.memory_examples {
            skill.examples.push(e)
        } else {
            let index = word(&mut self.rng) % skill.seen;
            if (index as usize) < skill.examples.len() {
                skill.examples[index as usize] = e
            }
        }
        let start = self.work;
        let mut spent = (raw.len() * 64
            + library.forward_cost(None) as usize
            + skill.head.weights.len() * 4) as u64
            + prediction_cost;
        if let Some(c) = skill.candidate.as_ref() {
            spent = spent.saturating_add(
                library.forward_cost(c.program.as_ref()) + (c.head.weights.len() * 4) as u64,
            );
        }
        if skill.examples.len() >= BATCH {
            for _ in 0..64 {
                let batch: Vec<_> = (0..BATCH)
                    .map(|_| {
                        let i = word(&mut self.rng) as usize % skill.examples.len();
                        skill.examples[i].clone()
                    })
                    .collect();
                let base_cost = update_cost(library, &skill.head, None);
                let extra_cost = skill
                    .candidate
                    .as_ref()
                    .map_or(0, |c| update_cost(library, &c.head, c.program.as_ref()));
                if spent.saturating_add(base_cost).saturating_add(extra_cost)
                    > self.config.work_per_example
                {
                    break;
                }
                update(
                    library,
                    &mut skill.head,
                    &mut skill.optimizer,
                    None,
                    None,
                    &batch,
                );
                spent += base_cost;
                if let Some(c) = skill.candidate.as_mut() {
                    update(
                        library,
                        &mut c.head,
                        &mut c.optimizer,
                        c.program.as_mut(),
                        Some(&mut c.program_optimizer),
                        &batch,
                    );
                    spent += extra_cost;
                }
            }
        }
        self.work = self.work.saturating_add(spent);
        if skill.candidate.as_ref().is_some_and(|c| c.future >= 128) {
            let c = skill.candidate.take().unwrap();
            let accepted = c.after_loss <= 0.95 * c.before_loss
                && c.before_loss > 0.
                && c.after_correct >= c.before_correct;
            if accepted {
                if let Some(mut p) = c.program.clone() {
                    p.admitted_after_teaching_event = event;
                    library.programs.push(p);
                }
                skill.head = c.head.clone();
                skill.optimizer = c.optimizer.clone();
            }
            self.close_trial_with_program(name, c, false, accepted);
            // A candidate's provisional node IDs belong to the library
            // prefix it was created against. Do not silently retarget another
            // skill's candidate after the shared prefix grows.
            if accepted {
                let obsolete: Vec<_> = self
                    .skills
                    .iter_mut()
                    .filter(|(_, s)| s.library == layout)
                    .filter_map(|(n, s)| s.candidate.take().map(|c| (n.clone(), c)))
                    .collect();
                for (n, c) in obsolete {
                    self.close_trial(&n, c, true);
                }
            }
        }
        let library = &self.libraries[&layout];
        let available = library.units();
        let new_units = self
            .config
            .growth_units
            .min(self.config.max_features.saturating_sub(available));
        if self.config.grow
            && skill.candidate.is_none()
            && skill.distinct >= skill.next_growth
            && skill.classes.len() >= 2
        {
            skill.next_growth = skill.distinct.saturating_add(256);
            let parents = if self.config.reuse_features {
                (0..available).collect::<Vec<_>>()
            } else {
                skill.head.visible.clone()
            };
            let dimensions = raw.len() * 2 + parents.len() + 1;
            let extra_parameters = dimensions * new_units
                + (raw.len() * 2 + parents.len() + new_units + 1) * skill.classes.len();
            let used = self.parameters() + skill.head.weights.len();
            let child_weights =
                (raw.len() * 2 + parents.len() + new_units + 1) * skill.classes.len();
            let block_weights = dimensions * new_units;
            let child_forward =
                library.forward_cost(None) + (block_weights * 4 + new_units * 64) as u64;
            let child_update = BATCH as u64 * (child_forward + (child_weights * 5) as u64)
                + (child_weights * 24 + block_weights * (BATCH * 5 + 24)) as u64
                + 1024;
            let future_overhead =
                (raw.len() * 80 + skill.head.weights.len() * 8 + child_weights * 4) as u64
                    + 2 * library.forward_cost(None)
                    + child_forward;
            let affordable = future_overhead
                .saturating_add(update_cost(library, &skill.head, None))
                .saturating_add(child_update)
                <= self.config.work_per_example;
            if new_units > 0 && affordable && used + extra_parameters <= self.config.max_parameters
            {
                let scale = 1. / (dimensions as f64).sqrt();
                let weights = (0..dimensions * new_units)
                    .map(|_| {
                        ((word(&mut self.rng) >> 11) as f64 / (1_u64 << 53) as f64 * 2. - 1.)
                            * scale
                    })
                    .collect();
                let p = FeatureProgram {
                    first_node: available,
                    parents: parents.clone(),
                    units: new_units,
                    weights,
                    learned_for: name.into(),
                    admitted_after_teaching_event: 0,
                };
                let visible = parents
                    .into_iter()
                    .chain(available..available + new_units)
                    .collect();
                let head = skill.head.expanded(raw.len(), visible);
                skill.candidate = Some(Trial {
                    head,
                    optimizer: Adam::default(),
                    program: Some(p),
                    program_optimizer: Adam::default(),
                    proposed_after: event,
                    warmup: 0,
                    future: 0,
                    before_loss: 0.,
                    after_loss: 0.,
                    before_correct: 0,
                    after_correct: 0,
                    first_future: None,
                    last_future: 0,
                });
                self.trials += 1;
            }
        }
        self.skills.insert(name.into(), skill);
        self.compact(name);
        Ok(TeachingReceipt {
            teaching_event: event,
            skill: name.into(),
            distinct_input: fresh,
            prediction_before: before,
            work: self.work - start,
            source_id: source_id.into(),
            input_fingerprint: fingerprint,
        })
    }
    fn close_trial(&mut self, name: &str, trial: Trial, interrupted: bool) {
        self.close_trial_with_program(name, trial, interrupted, false);
    }
    fn close_trial_with_program(
        &mut self,
        name: &str,
        trial: Trial,
        interrupted: bool,
        accepted: bool,
    ) {
        let receipt = GrowthReceipt {
            skill: name.into(),
            proposed_after: trial.proposed_after,
            first_future: trial.first_future,
            last_future: trial.last_future,
            future_examples: trial.future,
            before_loss: trial.before_loss,
            after_loss: trial.after_loss,
            before_correct: trial.before_correct,
            after_correct: trial.after_correct,
            new_features: trial.program.as_ref().map_or(0, |p| p.units),
            accepted,
            interrupted,
        };
        if accepted {
            self.admitted += 1
        } else {
            self.rejected += 1
        }
        self.receipts.push(receipt);
        if self.receipts.len() > 64 {
            self.receipts.remove(0);
        }
    }
    pub(crate) fn freeze(&mut self) {
        let pending: Vec<_> = self
            .skills
            .iter_mut()
            .filter_map(|(n, s)| s.candidate.take().map(|c| (n.clone(), c)))
            .collect();
        for (n, c) in pending {
            self.close_trial(&n, c, true)
        }
        self.frozen = true;
    }
    pub(crate) fn learned_hash(&self) -> u64 {
        hash(&(
            &self.config,
            &self.libraries,
            &self.skills,
            self.rng,
            self.events,
            self.trials,
            self.admitted,
            self.rejected,
            &self.receipts,
        ))
    }
    pub(crate) fn report(&self) -> BasisReport {
        BasisReport {
            config: self.config.clone(),
            teaching_events: self.events,
            work: self.work,
            parameters: self.parameters(),
            stored_examples: self.skills.values().map(|s| s.examples.len()).sum(),
            programs: self.libraries.values().map(|l| l.programs.len()).sum(),
            features: self.libraries.values().map(Library::units).sum(),
            admitted_growth: self.admitted,
            rejected_growth: self.rejected,
            trials: self.trials,
            frozen: self.frozen,
            skills: self
                .skills
                .iter()
                .map(|(name, s)| SkillReport {
                    name: name.clone(),
                    concepts: s.classes.clone(),
                    teaching_examples: s.seen,
                    distinct_inputs: s.distinct,
                    visible_features: s.head.visible.len(),
                    reused_features: s
                        .head
                        .visible
                        .iter()
                        .enumerate()
                        .filter(|&(slot, &id)| {
                            self.libraries[&s.library].programs.iter().any(|p| {
                                p.learned_for != *name
                                    && (p.first_node..p.first_node + p.units).contains(&id)
                            }) && (0..s.head.classes).any(|c| {
                                s.head.weights[c * s
                                    .head
                                    .width(self.libraries[&s.library].coordinates.len())
                                    + self.libraries[&s.library].coordinates.len() * 2
                                    + slot]
                                    .abs()
                                    > 1e-12
                            })
                        })
                        .count(),
                    prequential_predictions: s.predicted,
                    prequential_correct: s.correct,
                    prequential_log_loss: s.loss / s.predicted.max(1) as f64,
                    candidate: s.candidate.is_some(),
                    model_fingerprint: hash(&(
                        &s.classes,
                        &s.head,
                        &self.libraries[&s.library]
                            .programs
                            .iter()
                            .filter(|p| {
                                s.head
                                    .visible
                                    .iter()
                                    .copied()
                                    .max()
                                    .is_some_and(|last| p.first_node <= last)
                            })
                            .collect::<Vec<_>>(),
                    )),
                })
                .collect(),
            last_growth: self.receipts.last().cloned(),
            learned_fingerprint: self.learned_hash(),
        }
    }
    pub(crate) fn validate(&self, actions: usize, frozen: bool) -> Result<()> {
        self.config.validate()?;
        if self.version != 1
            || self.skills.len() > 16
            || self.libraries.len() > 16
            || self.receipts.len() > 64
            || self.admitted
                != self
                    .libraries
                    .values()
                    .map(|l| l.programs.len() as u64)
                    .sum::<u64>()
            || self.parameters() > self.config.max_parameters
            || self.sample_bytes() > self.config.memory_bytes
            || self.events != self.skills.values().map(|s| s.seen).sum::<u64>()
            || self.trials
                != self.admitted
                    + self.rejected
                    + self
                        .skills
                        .values()
                        .filter(|s| s.candidate.is_some())
                        .count() as u64
            || self.frozen != frozen
        {
            return Err(Error::InvalidData);
        }
        let good = |v: &[f64]| v.iter().all(|v| v.is_finite() && v.abs() <= 8.000001);
        for (id, l) in &self.libraries {
            if l.coordinates.is_empty()
                || l.coordinates.len() > self.config.max_coordinates
                || *id != hash(&l.coordinates)
                || l.units() > self.config.max_features
                || l.coordinates
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != l.coordinates.len()
            {
                return Err(Error::InvalidData);
            }
            let mut seen = 0;
            for p in &l.programs {
                if p.first_node != seen
                    || p.units == 0
                    || p.units > 32
                    || p.parents.iter().any(|&i| i >= seen)
                    || p.parents.windows(2).any(|a| a[0] >= a[1])
                    || p.weights.len() != (l.coordinates.len() * 2 + p.parents.len() + 1) * p.units
                    || !good(&p.weights)
                    || !self.skills.contains_key(&p.learned_for)
                    || p.admitted_after_teaching_event == 0
                    || p.admitted_after_teaching_event > self.events
                {
                    return Err(Error::InvalidData);
                }
                seen += p.units;
            }
        }
        for (name, s) in &self.skills {
            let l = self.libraries.get(&s.library).ok_or(Error::InvalidData)?;
            let check_head = |h: &Head, n: usize| {
                h.classes == s.classes.len()
                    && h.weights.len() == h.width(l.coordinates.len()) * h.classes
                    && h.visible.iter().all(|&i| i < n)
                    && h.visible.windows(2).all(|a| a[0] < a[1])
                    && good(&h.weights)
            };
            if !name_ok(name)
                || s.classes.is_empty()
                || s.classes.len() > 64
                || s.classes.iter().any(|c| !name_ok(c))
                || s.classes
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != s.classes.len()
                || s.examples.len() > self.config.memory_examples
                || s.bloom.len() != 1024
                || s.distinct > s.seen
                || s.predicted > s.seen
                || s.correct > s.predicted
                || !s.loss.is_finite()
                || s.loss < 0.
                || !check_head(&s.head, l.units())
                || !s.optimizer.valid(s.head.weights.len())
                || s.examples.iter().any(|e| {
                    e.values.len() != l.coordinates.len()
                        || e.class >= s.classes.len()
                        || e.origin == 0
                        || e.origin > self.events
                        || e.values
                            .iter()
                            .any(|v| !v.is_finite() || v.abs() > 1.000001)
                        || e.fingerprint != hash(&e.values)
                })
            {
                return Err(Error::InvalidData);
            }
            if let Some(c) = &s.candidate {
                if c.program.is_none() {
                    return Err(Error::InvalidData);
                }
                let added = c.program.as_ref().map_or(0, |p| p.units);
                if self.frozen
                    || c.proposed_after > self.events
                    || c.future > c.warmup
                    || c.future > 128
                    || !c.before_loss.is_finite()
                    || !c.after_loss.is_finite()
                    || c.before_loss < 0.
                    || c.after_loss < 0.
                    || c.before_correct > c.future
                    || c.after_correct > c.future
                    || !check_head(&c.head, l.units() + added)
                    || !c.optimizer.valid(c.head.weights.len())
                {
                    return Err(Error::InvalidData);
                }
                if let Some(p) = &c.program {
                    if p.first_node != l.units()
                        || p.units == 0
                        || p.units > 32
                        || l.units() + p.units > self.config.max_features
                        || p.parents.iter().any(|&i| i >= l.units())
                        || p.parents.windows(2).any(|a| a[0] >= a[1])
                        || p.learned_for != *name
                        || p.admitted_after_teaching_event != 0
                        || c.head.visible
                            != p.parents
                                .iter()
                                .copied()
                                .chain(p.first_node..p.first_node + p.units)
                                .collect::<Vec<_>>()
                        || p.weights.len()
                            != (l.coordinates.len() * 2 + p.parents.len() + 1) * p.units
                        || !good(&p.weights)
                        || !c.program_optimizer.valid(p.weights.len())
                    {
                        return Err(Error::InvalidData);
                    }
                }
            }
        }
        for r in &self.receipts {
            if !self.skills.contains_key(&r.skill)
                || r.accepted != r.verdict()
                || r.proposed_after > self.events
                || r.last_future > self.events
                || r.first_future
                    .is_some_and(|f| f <= r.proposed_after || f > r.last_future)
                || !r.before_loss.is_finite()
                || !r.after_loss.is_finite()
                || r.before_loss < 0.
                || r.after_loss < 0.
                || r.before_correct > r.future_examples
                || r.after_correct > r.future_examples
            {
                return Err(Error::InvalidData);
            }
        }
        if let Some(g) = &self.goal {
            let s = self.skills.get(&g.skill).ok_or(Error::InvalidData)?;
            if g.answers.is_empty()
                || g.answers.len() > actions
                || g.answers
                    .iter()
                    .any(|a| a.motor >= actions || !s.classes.contains(&a.concept))
                || g.answers
                    .iter()
                    .map(|a| a.motor)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != g.answers.len()
            {
                return Err(Error::InvalidData);
            }
        }
        if let Some(f) = &self.context {
            crate::learning_world::admit_frame(f, self.config.max_coordinates)?;
        }
        Ok(())
    }
}

fn update_cost(l: &Library, h: &Head, p: Option<&FeatureProgram>) -> u64 {
    BATCH as u64 * (l.forward_cost(p) + (h.weights.len() * 5) as u64)
        + (h.weights.len() * 24) as u64
        + p.map_or(0, |p| (p.weights.len() * (BATCH * 5 + 24)) as u64)
        + 1024
}
fn update(
    l: &Library,
    h: &mut Head,
    opt: &mut Adam,
    mut program: Option<&mut FeatureProgram>,
    mut program_opt: Option<&mut Adam>,
    batch: &[Example],
) {
    let mut gradient = vec![0.; h.weights.len()];
    let mut program_gradient = program.as_ref().map(|p| vec![0.; p.weights.len()]);
    let width = h.width(l.coordinates.len());
    for e in batch {
        let f = l.features(&e.values, None, &h.visible, program.as_deref());
        let p = h.probabilities(&f);
        let delta: Vec<_> = p
            .iter()
            .enumerate()
            .map(|(i, p)| (p - if i == e.class { 1. } else { 0. }) / batch.len() as f64)
            .collect();
        for c in 0..h.classes {
            for j in 0..width {
                gradient[c * width + j] += delta[c] * f[j];
            }
        }
        if let (Some(unit), Some(g)) = (program.as_deref(), program_gradient.as_mut()) {
            let input: Vec<_> = f[..l.coordinates.len() * 2]
                .iter()
                .copied()
                .chain(unit.parents.iter().map(|i| {
                    f[l.coordinates.len() * 2
                        + h.visible.binary_search(i).expect("parent feature visible")]
                }))
                .chain([1.])
                .collect();
            for k in 0..unit.units {
                let id = unit.first_node + k;
                let slot = h
                    .visible
                    .iter()
                    .position(|&i| i == id)
                    .expect("candidate feature visible")
                    + l.coordinates.len() * 2;
                let derivative = delta
                    .iter()
                    .enumerate()
                    .map(|(c, d)| d * h.weights[c * width + slot])
                    .sum::<f64>()
                    * (1. - f[slot] * f[slot]);
                for j in 0..input.len() {
                    g[k * input.len() + j] += derivative * input[j];
                }
            }
        }
    }
    for (g, w) in gradient.iter_mut().zip(&h.weights) {
        *g += 0.0001 * w;
    }
    opt.update(&mut h.weights, &gradient);
    if let (Some(p), Some(g), Some(opt)) = (
        program.as_mut(),
        program_gradient.as_mut(),
        program_opt.as_mut(),
    ) {
        for (g, w) in g.iter_mut().zip(&p.weights) {
            *g += 0.0001 * w;
        }
        opt.update(&mut p.weights, g);
    }
}
