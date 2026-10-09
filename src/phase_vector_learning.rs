// Generic bounded multivariate perception. Centers and response strengths live
// only in the shared phase synapses; the host never classifies an observation.
// The metric, online quantization and voting algorithm are inherited software.

const MAX_VECTOR_SYNAPSES: usize = 65_536;
const MAX_VECTOR_PROTOTYPES: usize = 512;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseVectorConfig {
    pub slots_per_motor: usize,
    pub neighbors: usize,
    pub merge_radius: f32,
    pub learning_rate: f32,
    pub rejection_radius: f32,
    pub minimum_margin: f32,
    pub exploration_observations: usize,
    pub sensing_support: usize,
    pub success_threshold: f32,
}
impl Default for PhaseVectorConfig {
    fn default() -> Self {
        Self {
            slots_per_motor: 32,
            neighbors: 3,
            merge_radius: 0.12,
            learning_rate: 0.05,
            rejection_radius: 0.25,
            minimum_margin: 0.05,
            exploration_observations: 32,
            sensing_support: 4,
            success_threshold: 0.95,
        }
    }
}
impl PhaseVectorConfig {
    fn valid(&self, width: usize, motors: usize) -> bool {
        (1..=1024).contains(&width)
            && (1..=64).contains(&motors)
            && (1..=64).contains(&self.slots_per_motor)
            && self.slots_per_motor.checked_mul(motors).is_some_and(|n| {
                n <= MAX_VECTOR_PROTOTYPES
                    && n.checked_mul(width + 1)
                        .is_some_and(|n| n <= MAX_VECTOR_SYNAPSES)
            })
            && (1..=7).contains(&self.neighbors)
            && self.merge_radius.is_finite()
            && (0.001..=0.5).contains(&self.merge_radius)
            && self.learning_rate.is_finite()
            && (0.001..=1.0).contains(&self.learning_rate)
            && self.rejection_radius.is_finite()
            && (self.merge_radius..=1.0).contains(&self.rejection_radius)
            && self.minimum_margin.is_finite()
            && (0.0..=1.0).contains(&self.minimum_margin)
            && self.exploration_observations <= 128
            && (2..=16).contains(&self.sensing_support)
            && self.success_threshold.is_finite()
            && (0.5..=1.0).contains(&self.success_threshold)
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseVectorPrototype {
    cell: usize,
    inputs: Vec<usize>,
    response: usize,
    support: u64,
    revision: u64,
    sources: Vec<u64>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseVectorMotor {
    prototypes: Vec<PhaseVectorPrototype>,
    observations: u64,
    positives: u64,
    preservation: u64,
    visible: Vec<bool>,
    mask_support: usize,
}
#[derive(Debug, Clone)]
struct PhaseVectorEpisode {
    factual: Vec<Option<f32>>,
    outcome: Option<f32>,
    rejected: Vec<bool>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseVectorState {
    config: PhaseVectorConfig,
    motors: Vec<PhaseVectorMotor>,
    factual_sequence: u64,
    #[serde(default)]
    induction: Option<PhaseInductionState>,
    #[serde(skip)]
    episode: Option<PhaseVectorEpisode>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseVectorNeighbor {
    pub action: usize,
    pub cell: usize,
    pub distance: f32,
    pub strength: f32,
    pub support: u64,
    pub evidence_sources: Vec<u64>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseVectorPrediction {
    pub action: usize,
    pub nearest_distance: f32,
    pub vote_margin: f32,
    pub neighbors: Vec<PhaseVectorNeighbor>,
    pub authority: Authority,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseVectorDecisionKind {
    Prediction,
    InducedProgram,
    Measurement,
    Experiment,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseVectorDecision {
    pub action: usize,
    pub kind: PhaseVectorDecisionKind,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseVectorInfo {
    pub observations: u64,
    pub active_prototypes: usize,
    pub allocated_prototypes: usize,
    pub positives_per_motor: Vec<u64>,
    pub observations_per_motor: Vec<u64>,
    pub prototype_synapses: Vec<usize>,
}

fn vector_valid(values: &[Option<f32>], width: usize) -> bool {
    values.len() == width
        && values
            .iter()
            .flatten()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
}
fn vector_distance(
    p: &PhaseVectorPrototype,
    input: &[f32],
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
) -> Option<f32> {
    if p.support == 0 || !cells[p.cell].recruited {
        return None;
    }
    let mut sum = 0.0;
    for (&index, &value) in p.inputs.iter().zip(input) {
        let link = &links[index];
        if link.weight < 0.5 || link.confidence < 0.5 || !cells[link.from].recruited {
            return None;
        }
        // Half-circle encoding distinguishes intensity 0 from intensity 1.
        let error = signed_phase_error(value * std::f32::consts::PI, link.phase_offset)
            / std::f32::consts::PI;
        sum += error * error;
    }
    Some((sum / input.len() as f32).sqrt())
}

impl EvoPhase {
    pub fn enable_phase_vector_learning(&mut self, config: PhaseVectorConfig) -> bool {
        let width = self.config.sensory_cells;
        let motors = self.config.motor_cells;
        if !config.valid(width, motors) || !self.config.structural_growth_enabled {
            return false;
        }
        let Some(native) = self.phase_native.as_ref() else {
            return false;
        };
        if native.online.is_none()
            || native.vector.is_some()
            || native.rules.is_some()
            || native.partial.is_some()
            || !native.receptors.is_empty()
            || !native.circuits.is_empty()
            || native.drive.is_some()
            || native.concepts.is_some()
            || native.deep.is_some()
            || native.contextual.is_some()
            || native.perceptual.is_some()
            || native.compositional.is_some()
            || native.meta_control.is_some()
            || native.temporal_evidence.is_some()
            || self.concept_memory.is_some()
        {
            return false;
        }
        let count = motors * config.slots_per_motor;
        let free = self
            .dormant_range()
            .filter(|&i| !self.cells[i].recruited)
            .take(count)
            .collect::<Vec<_>>();
        if free.len() != count {
            return false;
        }
        let mut actions = Vec::new();
        let mut next = free.into_iter();
        for a in 0..motors {
            let mut prototypes = Vec::new();
            for _ in 0..config.slots_per_motor {
                let cell = next.next().unwrap();
                self.cells[cell].recruited = true;
                let inputs = (0..width)
                    .map(|source| self.native_synapse(source, cell))
                    .collect();
                let response = self.native_synapse(cell, self.motor_cell(a));
                prototypes.push(PhaseVectorPrototype {
                    cell,
                    inputs,
                    response,
                    support: 0,
                    revision: 0,
                    sources: Vec::new(),
                });
            }
            actions.push(PhaseVectorMotor {
                prototypes,
                observations: 0,
                positives: 0,
                preservation: 0,
                visible: vec![false; width],
                mask_support: 0,
            });
        }
        self.phase_native.as_mut().unwrap().vector = Some(PhaseVectorState {
            config,
            motors: actions,
            factual_sequence: 0,
            induction: None,
            episode: None,
        });
        self.current_real = None;
        true
    }
    pub fn phase_vector_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .is_some_and(|n| n.vector.is_some())
    }
    pub fn phase_vector_info(&self) -> Option<PhaseVectorInfo> {
        let state = self.phase_native.as_ref()?.vector.as_ref()?;
        Some(PhaseVectorInfo {
            observations: state.factual_sequence,
            active_prototypes: state
                .motors
                .iter()
                .flat_map(|m| &m.prototypes)
                .filter(|p| p.support > 0)
                .count(),
            allocated_prototypes: state.motors.len() * state.config.slots_per_motor,
            positives_per_motor: state.motors.iter().map(|m| m.positives).collect(),
            observations_per_motor: state.motors.iter().map(|m| m.observations).collect(),
            prototype_synapses: state
                .motors
                .iter()
                .flat_map(|m| &m.prototypes)
                .flat_map(|p| p.inputs.iter().copied().chain(std::iter::once(p.response)))
                .collect(),
        })
    }
    fn is_phase_vector_synapse(&self, index: usize) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|n| n.vector.as_ref())
            .is_some_and(|s| {
                s.motors
                    .iter()
                    .flat_map(|m| &m.prototypes)
                    .any(|p| p.response == index || p.inputs.contains(&index))
            })
    }
    pub(crate) fn observe_phase_vector_initial(&mut self, frame: &[Option<f32>]) -> bool {
        if !vector_valid(frame, self.config.sensory_cells) {
            return false;
        }
        let Some(state) = self.phase_native.as_mut().and_then(|n| n.vector.as_mut()) else {
            return false;
        };
        state.episode = Some(PhaseVectorEpisode {
            factual: frame.to_vec(),
            outcome: None,
            rejected: vec![false; self.config.motor_cells],
        });
        self.current_real = None;
        true
    }
    pub(crate) fn phase_vector_last_outcome(&self) -> Option<f32> {
        self.phase_native
            .as_ref()?
            .vector
            .as_ref()?
            .episode
            .as_ref()?
            .outcome
    }
    pub fn phase_vector_predict(&self, frame: &[Option<f32>]) -> Option<PhaseVectorPrediction> {
        if !vector_valid(frame, self.config.sensory_cells) {
            return None;
        }
        let input = frame.iter().copied().collect::<Option<Vec<_>>>()?;
        let state = self.phase_native.as_ref()?.vector.as_ref()?;
        let mut neighbors = Vec::new();
        for m in &state.motors {
            for p in &m.prototypes {
                let response = &self.synapses[p.response];
                if response.weight <= 0.0
                    || response.confidence < 0.5
                    || !self.cells[response.to].recruited
                {
                    continue;
                }
                let action = response.to.checked_sub(self.config.sensory_cells)?;
                if action >= self.config.motor_cells {
                    return None;
                }
                if let Some(distance) = vector_distance(p, &input, &self.cells, &self.synapses) {
                    neighbors.push(PhaseVectorNeighbor {
                        action,
                        cell: p.cell,
                        distance,
                        strength: response.weight,
                        support: p.support,
                        evidence_sources: p.sources.clone(),
                    });
                }
            }
        }
        neighbors.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then_with(|| b.strength.total_cmp(&a.strength))
                .then_with(|| a.cell.cmp(&b.cell))
        });
        if neighbors.first()?.distance > state.config.rejection_radius {
            return None;
        }
        neighbors.truncate(state.config.neighbors);
        let mut votes = vec![0.0; self.config.motor_cells];
        for n in &neighbors {
            votes[n.action] += n.strength / (n.distance * n.distance + 0.0001);
        }
        let mut ordered = (0..votes.len()).collect::<Vec<_>>();
        ordered.sort_by(|&a, &b| votes[b].total_cmp(&votes[a]).then_with(|| a.cmp(&b)));
        let sum = votes.iter().sum::<f32>();
        let margin = (votes[ordered[0]] - ordered.get(1).map_or(0.0, |&i| votes[i])) / sum;
        if margin < state.config.minimum_margin {
            return None;
        }
        Some(PhaseVectorPrediction {
            action: ordered[0],
            nearest_distance: neighbors[0].distance,
            vote_margin: margin,
            neighbors,
            authority: Authority::Imagined,
        })
    }
    pub fn phase_vector_decision(&self) -> Option<PhaseVectorDecision> {
        let native = self.phase_native.as_ref()?;
        let s = native.vector.as_ref()?;
        let e = s.episode.as_ref()?;
        let missing = e
            .factual
            .iter()
            .enumerate()
            .filter_map(|(j, v)| v.is_none().then_some(j))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            let candidates = s
                .motors
                .iter()
                .enumerate()
                .filter(|(a, m)| {
                    !e.rejected[*a]
                        && m.observations >= s.config.sensing_support as u64
                        && m.positives == 0
                        && m.preservation == m.observations
                        && m.mask_support >= s.config.sensing_support
                        && missing.iter().all(|&j| m.visible[j])
                })
                .map(|(a, _)| a)
                .collect::<Vec<_>>();
            if candidates.len() == 1 {
                return Some(PhaseVectorDecision {
                    action: candidates[0],
                    kind: PhaseVectorDecisionKind::Measurement,
                });
            }
            if !native.config.learning_enabled {
                return None;
            }
        }
        if native.config.learning_enabled {
            if let Some(a) = (0..s.motors.len())
                .filter(|&a| {
                    !e.rejected[a]
                        && s.motors[a].observations < s.config.exploration_observations as u64
                })
                .min_by_key(|&a| {
                    (
                        s.motors[a].observations,
                        uncertain_exploration_rank(s.factual_sequence, a),
                    )
                })
            {
                return Some(PhaseVectorDecision {
                    action: a,
                    kind: PhaseVectorDecisionKind::Experiment,
                });
            }
        }
        // In an opt-in native argument-competition organism, an internal
        // hypothesis conflict can initiate a protected factual experiment.
        // No host lesson schedule supplies the motor or binding candidate.
        if let Some((action,_uncertainty))=self
            .phase_native_intrinsic_argument_probe(&e.factual,&e.rejected){
            return Some(PhaseVectorDecision{
                action,kind:PhaseVectorDecisionKind::Experiment,
            });
        }
        let prediction = if s.induction.is_some() {
            self.phase_induction_predict(&e.factual)
                .map(|p| (p.action, PhaseVectorDecisionKind::InducedProgram))
        } else {
            self.phase_vector_predict(&e.factual)
                .map(|p| (p.action, PhaseVectorDecisionKind::Prediction))
        };
        if let Some((action, kind)) = prediction {
            if !e.rejected[action] {
                return Some(PhaseVectorDecision { action, kind });
            }
        }
        native
            .config
            .learning_enabled
            .then(|| {
                (0..s.motors.len())
                    .filter(|&a| !e.rejected[a])
                    .min_by_key(|&a| {
                        (
                            s.motors[a].observations,
                            uncertain_exploration_rank(s.factual_sequence, a),
                        )
                    })
            })
            .flatten()
            .map(|action| PhaseVectorDecision {
                action,
                kind: PhaseVectorDecisionKind::Experiment,
            })
    }
    pub(crate) fn observe_phase_vector_result(
        &mut self,
        action: usize,
        post: &[Option<f32>],
        outcome: f32,
    ) -> Option<PhasePartialUpdate> {
        if action >= self.config.motor_cells
            || !vector_valid(post, self.config.sensory_cells)
            || !outcome.is_finite()
            || !(0.0..=1.0).contains(&outcome)
        {
            return None;
        }
        let native = self.phase_native.as_ref()?;
        let s = native.vector.as_ref()?;
        let old = s.episode.as_ref()?.clone();
        let learning = native.config.learning_enabled;
        let sequence = s.factual_sequence.checked_add(u64::from(learning))?;
        let induction_enabled = s.induction.is_some();
        let predicted = (!induction_enabled)
            .then(|| self.phase_vector_predict(&old.factual))
            .flatten();
        let mut state = self.phase_native.as_mut()?.vector.take()?;
        if learning {
            let m = &mut state.motors[action];
            m.observations += 1;
            let preserved = post.iter().any(Option::is_some)
                && old.factual.iter().zip(post).all(|(a, b)| {
                    a.is_none() || a.zip(*b).is_some_and(|(a, b)| (a - b).abs() <= 0.001)
                });
            m.preservation += u64::from(preserved);
            let visible = post.iter().map(Option::is_some).collect::<Vec<_>>();
            if m.visible != visible {
                m.mask_support = 0;
            }
            m.visible = visible;
            m.mask_support = (m.mask_support + 1).min(state.config.sensing_support);
            if outcome >= state.config.success_threshold {
                m.positives += 1;
                if let Some(input) = (!induction_enabled)
                    .then(|| old.factual.iter().copied().collect::<Option<Vec<_>>>())
                    .flatten()
                {
                    let nearest = m
                        .prototypes
                        .iter()
                        .enumerate()
                        .filter_map(|(i, p)| {
                            vector_distance(p, &input, &self.cells, &self.synapses).map(|d| (i, d))
                        })
                        .min_by(|a, b| a.1.total_cmp(&b.1));
                    let free = m.prototypes.iter().position(|p| p.support == 0);
                    let index = match (nearest, free) {
                        (Some((_, d)), Some(f)) if d > state.config.merge_radius => f,
                        (Some((i, _)), _) => i,
                        (None, Some(f)) => f,
                        (None, None) => 0,
                    };
                    let p = &mut m.prototypes[index];
                    let reset = p.support == 0
                        || vector_distance(p, &input, &self.cells, &self.synapses).is_none();
                    if reset {
                        p.support = 0;
                        p.sources.clear();
                    }
                    for (&link, &x) in p.inputs.iter().zip(&input) {
                        let syn = &mut self.synapses[link];
                        let target = x * std::f32::consts::PI;
                        syn.phase_offset = if reset {
                            target
                        } else {
                            (syn.phase_offset
                                + state.config.learning_rate * (target - syn.phase_offset))
                                .clamp(0.0, std::f32::consts::PI)
                        };
                        syn.weight = 1.0;
                        syn.confidence = 1.0;
                    }
                    let response = &mut self.synapses[p.response];
                    response.weight = if reset {
                        1.0
                    } else {
                        response.weight + state.config.learning_rate * (1.0 - response.weight)
                    };
                    response.confidence = 1.0;
                    p.support += 1;
                    p.revision += 1;
                    if p.sources.len() == state.config.sensing_support {
                        p.sources.remove(0);
                    }
                    p.sources.push(sequence);
                }
            } else if let Some(p) = predicted.filter(|p| p.action == action) {
                if let Some(n) = p.neighbors.iter().find(|n| n.action == action) {
                    if let Some(prototype) = m.prototypes.iter_mut().find(|p| p.cell == n.cell) {
                        self.synapses[prototype.response].weight *=
                            1.0 - state.config.learning_rate;
                        prototype.revision += 1;
                    }
                }
            }
            state.factual_sequence = sequence;
        }
        let mut rejected = old.rejected;
        if old.factual != post {
            rejected.fill(false);
        }
        if outcome < state.config.success_threshold {
            rejected[action] = true;
        }
        state.episode = Some(PhaseVectorEpisode {
            factual: post.to_vec(),
            outcome: Some(outcome),
            rejected,
        });
        self.phase_native.as_mut()?.vector = Some(state);
        if learning && induction_enabled {
            self.observe_phase_induction_result(action, &old.factual, outcome);
        }
        self.current_real = None;
        Some(PhasePartialUpdate {
            suppressed: 0,
            learned: learning,
        })
    }
}

impl PhaseVectorState {
    fn fingerprint(&self, cells: &[PhaseCell], links: &[PhaseSynapse]) -> u64 {
        // Stream the actual physical values rather than allocating a debug
        // string of every feature link on each action. No parameter cache.
        let mut h = 14_695_981_039_346_656_037u64;
        let mut feed = |v: u64| {
            h ^= v;
            h = h.wrapping_mul(1_099_511_628_211);
        };
        for c in cells {
            for v in [c.phase, c.threshold, c.utility] {
                feed(u64::from(v.to_bits()));
            }
            feed(c.age);
            feed(u64::from(c.recruited));
        }
        for l in links {
            feed(l.from as u64);
            feed(l.to as u64);
            for v in [l.weight, l.phase_offset, l.eligibility, l.confidence] {
                feed(u64::from(v.to_bits()));
            }
            feed(u64::from(l.plastic));
        }
        let c = &self.config;
        for n in [
            c.slots_per_motor,
            c.neighbors,
            c.exploration_observations,
            c.sensing_support,
        ] {
            feed(n as u64);
        }
        for n in [
            c.merge_radius,
            c.learning_rate,
            c.rejection_radius,
            c.minimum_margin,
            c.success_threshold,
        ] {
            feed(u64::from(n.to_bits()));
        }
        feed(self.factual_sequence);
        if let Some(induction) = &self.induction {
            induction.fingerprint_feed(&mut feed);
        }
        for m in &self.motors {
            feed(m.observations);
            feed(m.positives);
            feed(m.preservation);
            feed(m.mask_support as u64);
            for &v in &m.visible {
                feed(u64::from(v));
            }
            for p in &m.prototypes {
                feed(p.cell as u64);
                feed(p.response as u64);
                feed(p.support);
                feed(p.revision);
                feed(p.sources.len() as u64);
                for &v in &p.inputs {
                    feed(v as u64);
                }
                for &v in &p.sources {
                    feed(v);
                }
            }
        }
        h
    }
    fn validate_snapshot(
        &self,
        cfg: &super::EvoConfig,
        cells: &[PhaseCell],
        synapses: &[PhaseSynapse],
        allocated: &mut std::collections::BTreeSet<usize>,
        used: &mut std::collections::BTreeSet<usize>,
    ) -> bool {
        if !self.config.valid(cfg.sensory_cells, cfg.motor_cells)
            || self.motors.len() != cfg.motor_cells
            || self.episode.is_some()
            || self.factual_sequence == u64::MAX
        {
            return false;
        }
        let start = cfg.sensory_cells + cfg.motor_cells + 1;
        let mut observations = 0_u64;
        let mut sources = std::collections::BTreeSet::new();
        for (a, m) in self.motors.iter().enumerate() {
            let Some(sum) = observations.checked_add(m.observations) else {
                return false;
            };
            observations = sum;
            if m.prototypes.len() != self.config.slots_per_motor
                || m.positives > m.observations
                || m.preservation > m.observations
                || m.visible.len() != cfg.sensory_cells
                || m.mask_support > self.config.sensing_support
                || m.mask_support as u64 > m.observations
            {
                return false;
            }
            for p in &m.prototypes {
                if !(start..cells.len()).contains(&p.cell)
                    || !cells[p.cell].recruited
                    || !allocated.insert(p.cell)
                    || p.inputs.len() != cfg.sensory_cells
                    || p.support > m.positives
                    || p.revision > self.factual_sequence
                    || (p.support == 0) != p.sources.is_empty()
                    || p.sources.len() > self.config.sensing_support
                    || p.sources.len() as u64 > p.support
                    || p.sources.windows(2).any(|v| v[0] >= v[1])
                    || p.sources
                        .iter()
                        .any(|&id| id == 0 || id > self.factual_sequence || !sources.insert(id))
                {
                    return false;
                }
                for (j, &index) in p.inputs.iter().enumerate() {
                    if index >= synapses.len() || !used.insert(index) {
                        return false;
                    }
                    let link = &synapses[index];
                    if link.from != j
                        || link.to != p.cell
                        || !(0.0..=std::f32::consts::TAU).contains(&link.phase_offset)
                    {
                        return false;
                    }
                }
                if p.response >= synapses.len() || !used.insert(p.response) {
                    return false;
                }
                let link = &synapses[p.response];
                if link.from != p.cell || link.to != cfg.sensory_cells + a {
                    return false;
                }
            }
        }
        observations == self.factual_sequence
            && self.induction.as_ref().is_none_or(|i| {
                i.validate_snapshot(cfg, self.factual_sequence, cells, synapses, allocated, used)
            })
    }
}
