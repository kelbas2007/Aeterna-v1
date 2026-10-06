use crate::authority::Authority;
use crate::hdc::PhaseVector;
use crate::phase::{phase_similarity, signed_phase_error, wrap_phase};

#[derive(Debug, Clone)]
pub struct EvoConfig {
    pub sensory_cells: usize,
    pub motor_cells: usize,
    pub dormant_cells: usize,
    pub hdc_dim: usize,
    pub weight_learning_rate: f32,
    pub phase_learning_rate: f32,
    pub eligibility_decay: f32,
    pub residual_recruit_threshold: f32,
    pub min_recruit_support: u32,
    pub structural_growth_enabled: bool,
    pub acquired_readout_enabled: bool,
}

impl Default for EvoConfig {
    fn default() -> Self {
        Self {
            sensory_cells: 8,
            motor_cells: 2,
            dormant_cells: 32,
            hdc_dim: 128,
            weight_learning_rate: 0.20,
            phase_learning_rate: 0.05,
            eligibility_decay: 0.92,
            residual_recruit_threshold: 0.30,
            min_recruit_support: 2,
            structural_growth_enabled: true,
            acquired_readout_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhaseCell {
    pub phase: f32,
    pub charge: f32,
    pub threshold: f32,
    pub utility: f32,
    pub age: u64,
    pub recruited: bool,
}

#[derive(Debug, Clone)]
pub struct PhaseSynapse {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
    pub phase_offset: f32,
    pub eligibility: f32,
    pub confidence: f32,
    pub plastic: bool,
}

#[derive(Debug, Clone)]
pub struct DendriticBranch {
    pub relay_cell: usize,
    pub inputs: Vec<usize>,
    pub motor: usize,
    pub preferred_relative_phase: Vec<f32>,
    pub phase_tolerance: f32,
    pub support: u32,
    pub utility: f32,
    pub revision: u64,
}

#[derive(Debug, Clone)]
struct ContextEvidence {
    on_count: u32,
    off_count: u32,
    on_need_mean: f32,
    off_need_mean: f32,
}

impl Default for ContextEvidence {
    fn default() -> Self {
        Self {
            on_count: 0,
            off_count: 0,
            on_need_mean: 0.5,
            off_need_mean: 0.5,
        }
    }
}

impl ContextEvidence {
    fn observe(&mut self, active: bool, target_need: f32) {
        if active {
            self.on_count = self.on_count.saturating_add(1);
            self.on_need_mean += (target_need - self.on_need_mean) / self.on_count as f32;
        } else {
            self.off_count = self.off_count.saturating_add(1);
            self.off_need_mean += (target_need - self.off_need_mean) / self.off_count as f32;
        }
    }

    fn contrast(&self) -> f32 {
        let support = self.on_count.min(self.off_count);
        if support == 0 {
            return 0.0;
        }
        let confidence = (support as f32 / 4.0).clamp(0.0, 1.0);
        (self.on_need_mean - self.off_need_mean).abs() * confidence
    }
}

#[derive(Debug, Clone)]
pub struct FactualFrame {
    pub sensory: Vec<f32>,
    pub need: bool,
    pub tick: u64,
}

#[derive(Debug, Clone)]
pub struct Prediction {
    pub sensory: Vec<f32>,
    pub need: f32,
    pub confidence: f32,
    pub authority: Authority,
}

#[derive(Debug, Clone)]
pub struct LearningReport {
    pub residual: f32,
    pub sensory_residual: f32,
    pub need_residual: f32,
    pub recruited_relay: Option<usize>,
    pub active_branches: usize,
    pub factual_tick: u64,
}

#[derive(Debug, Clone)]
pub struct EvoPhase {
    config: EvoConfig,
    cells: Vec<PhaseCell>,
    synapses: Vec<PhaseSynapse>,
    branches: Vec<DendriticBranch>,
    current_real: Option<FactualFrame>,
    tick: u64,
    action_usage: Vec<u64>,
    residual_support: Vec<u32>,
    sensory_roles: Vec<PhaseVector>,
    motor_roles: Vec<PhaseVector>,
    context_evidence: Vec<Vec<ContextEvidence>>,
}

impl EvoPhase {
    pub fn new(config: EvoConfig) -> Self {
        assert!(config.sensory_cells > 0);
        assert!(config.motor_cells > 0);
        assert!(config.hdc_dim > 0);

        let need_cell = config.sensory_cells + config.motor_cells;
        let total = need_cell + 1 + config.dormant_cells;
        let mut cells = Vec::with_capacity(total);
        for i in 0..total {
            cells.push(PhaseCell {
                phase: wrap_phase((i as f32 * 0.618_034) % std::f32::consts::TAU),
                charge: 0.0,
                threshold: 0.5,
                utility: 0.0,
                age: 0,
                recruited: i <= need_cell,
            });
        }

        let sensory_roles = (0..config.sensory_cells)
            .map(|i| PhaseVector::from_seed(config.hdc_dim, 0xA11C_E000 + i as u64))
            .collect();
        let motor_roles = (0..config.motor_cells)
            .map(|i| PhaseVector::from_seed(config.hdc_dim, 0xBEEF_0000 + i as u64))
            .collect();
        let context_evidence = vec![
            vec![ContextEvidence::default(); config.sensory_cells];
            config.motor_cells
        ];

        Self {
            action_usage: vec![0; config.motor_cells],
            residual_support: vec![0; config.motor_cells],
            config,
            cells,
            synapses: Vec::new(),
            branches: Vec::new(),
            current_real: None,
            tick: 0,
            sensory_roles,
            motor_roles,
            context_evidence,
        }
    }

    pub fn config(&self) -> &EvoConfig {
        &self.config
    }

    pub fn current_real(&self) -> Option<&FactualFrame> {
        self.current_real.as_ref()
    }

    pub fn set_acquired_readout_enabled(&mut self, enabled: bool) {
        self.config.acquired_readout_enabled = enabled;
    }

    pub fn branches(&self) -> &[DendriticBranch] {
        &self.branches
    }

    pub fn recruited_relays(&self) -> usize {
        self.dormant_range()
            .filter(|idx| self.cells[*idx].recruited)
            .count()
    }

    pub fn observe_initial_real(&mut self, sensory: &[f32], need: bool) {
        self.assert_sensory(sensory);
        self.tick += 1;
        self.load_real(sensory, need);
    }

    pub fn predict(&self, action: usize, authority: Authority) -> Prediction {
        assert!(action < self.config.motor_cells);
        assert!(authority != Authority::Real, "MODEL/IMAGINED output cannot claim REAL authority");
        let pre = self.current_real.as_ref().expect("predict requires factual PRE");

        let mut need_sum = 0.0f32;
        let mut need_mass = 0.0f32;

        if let Some(s) = self.direct_need_synapse(action) {
            let c = s.confidence.max(0.05);
            need_sum += s.weight.clamp(0.0, 1.0) * c;
            need_mass += c;
        }

        if self.config.acquired_readout_enabled {
            for branch in &self.branches {
                if branch.motor != action || !self.branch_matches(branch, &pre.sensory) {
                    continue;
                }
                if let Some(s) = self.relay_need_synapse(branch.relay_cell) {
                    let c = (s.confidence * (0.5 + 0.5 * branch.utility.max(0.0))).max(0.05);
                    need_sum += s.weight.clamp(0.0, 1.0) * c;
                    need_mass += c;
                }
            }
        }

        let need = if need_mass > 0.0 { need_sum / need_mass } else { 0.0 };
        let confidence = (need_mass / (1.0 + need_mass)).clamp(0.0, 1.0);

        Prediction {
            sensory: pre.sensory.clone(),
            need,
            confidence,
            authority,
        }
    }

    pub fn choose_motor(&mut self) -> usize {
        let pre = self.current_real.as_ref().expect("choose_motor requires factual PRE").clone();
        let mut best: Option<(usize, f32)> = None;

        for action in 0..self.config.motor_cells {
            let p = self.predict(action, Authority::Model);
            let uncertainty = 1.0 - p.confidence;
            let novelty = 1.0 / ((self.action_usage[action] + 1) as f32).sqrt();

            let motor_phase = self.cells[self.motor_cell(action)].phase;
            let mut coherence = 0.0f32;
            let mut terms = 0.0f32;
            for (i, x) in pre.sensory.iter().enumerate() {
                if *x >= 0.5 {
                    coherence += phase_similarity(self.cells[i].phase, motor_phase);
                    terms += 1.0;
                }
            }
            if terms > 0.0 {
                coherence /= terms;
            }

            let branch_utility: f32 = if self.config.acquired_readout_enabled {
                self.branches.iter()
                    .filter(|b| b.motor == action && self.branch_matches(b, &pre.sensory))
                    .map(|b| b.utility.max(0.0))
                    .sum()
            } else {
                0.0
            };

            // Goal value, epistemic pressure, generic exploration and carrier coherence.
            // No task label, coordinate, map or correct action is available here.
            let score =
                2.0 * p.need
                + 1.00 * uncertainty
                + 0.80 * novelty
                + 0.20 * branch_utility
                + 0.05 * coherence;

            if best.map(|(_, s)| score > s).unwrap_or(true) {
                best = Some((action, score));
            }
        }

        let action = best.expect("at least one motor").0;
        self.action_usage[action] += 1;
        action
    }

    pub fn learn_factual_transition(
        &mut self,
        action: usize,
        post_sensory: &[f32],
        post_need: bool,
    ) -> LearningReport {
        assert!(action < self.config.motor_cells);
        self.assert_sensory(post_sensory);
        let pre = self.current_real.clone().expect("learning requires factual PRE");
        let prediction = self.predict(action, Authority::Model);

        let target_need = if post_need { 1.0 } else { 0.0 };
        let need_residual = (target_need - prediction.need).abs();
        let sensory_residual = prediction.sensory.iter()
            .zip(post_sensory)
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>() / self.config.sensory_cells as f32;
        let residual = 0.65 * need_residual + 0.35 * sensory_residual;

        self.update_context_evidence(action, &pre.sensory, target_need);
        self.update_direct_need(action, target_need);

        let matching: Vec<usize> = self.branches.iter().enumerate()
            .filter(|(_, b)| b.motor == action && self.branch_matches(b, &pre.sensory))
            .map(|(i, _)| i)
            .collect();

        for idx in matching.iter().copied() {
            self.revise_branch(idx, target_need, residual);
        }

        let mut recruited_relay = None;
        if residual >= self.config.residual_recruit_threshold {
            self.residual_support[action] = self.residual_support[action].saturating_add(1);
            if self.config.structural_growth_enabled
                && self.residual_support[action] >= self.config.min_recruit_support
                && !self.has_exact_context_branch(action, &pre.sensory)
            {
                recruited_relay = self.recruit_context_branch(action, &pre.sensory, target_need);
                if recruited_relay.is_some() {
                    self.residual_support[action] = 0;
                }
            }
        } else {
            self.residual_support[action] = 0;
        }

        for syn in &mut self.synapses {
            syn.eligibility *= self.config.eligibility_decay;
        }
        for cell in &mut self.cells {
            cell.age = cell.age.saturating_add(1);
        }

        self.tick += 1;
        self.load_real(post_sensory, post_need);

        LearningReport {
            residual,
            sensory_residual,
            need_residual,
            recruited_relay,
            active_branches: matching.len(),
            factual_tick: self.tick,
        }
    }

    fn update_direct_need(&mut self, action: usize, target: f32) {
        let from = self.motor_cell(action);
        let to = self.need_cell();
        if let Some(i) = self.synapses.iter().position(|s| s.from == from && s.to == to) {
            let s = &mut self.synapses[i];
            let error = target - s.weight;
            s.weight = (s.weight + self.config.weight_learning_rate * 0.25 * error).clamp(0.0, 1.0);
            s.confidence = (s.confidence + 0.03).clamp(0.0, 1.0);
            s.eligibility = 1.0;
        } else {
            self.synapses.push(PhaseSynapse {
                from,
                to,
                weight: target * 0.15,
                phase_offset: 0.0,
                eligibility: 1.0,
                confidence: 0.10,
                plastic: true,
            });
        }
    }

    fn revise_branch(&mut self, branch_idx: usize, target: f32, residual: f32) {
        let relay = self.branches[branch_idx].relay_cell;
        if let Some(i) = self.synapses.iter().position(|s| s.from == relay && s.to == self.need_cell()) {
            let s = &mut self.synapses[i];
            let error = target - s.weight;
            s.weight = (s.weight + self.config.weight_learning_rate * error).clamp(0.0, 1.0);
            s.confidence = (s.confidence + 0.08).clamp(0.0, 1.0);
            s.eligibility = 1.0;
        }
        let b = &mut self.branches[branch_idx];
        b.support = b.support.saturating_add(1);
        b.utility = (b.utility + (1.0 - residual) * 0.10 - residual * 0.05).clamp(-1.0, 1.0);
        if residual >= self.config.residual_recruit_threshold {
            b.revision = b.revision.saturating_add(1);
        }
    }

    fn recruit_context_branch(&mut self, action: usize, sensory: &[f32], target: f32) -> Option<usize> {
        let relay = self.dormant_range().find(|idx| !self.cells[*idx].recruited)?;
        let inputs = self.select_context_inputs(action, sensory)?;
        let motor_phase = self.cells[self.motor_cell(action)].phase;
        let preferred_relative_phase = inputs.iter()
            .map(|idx| signed_phase_error(self.cells[*idx].phase, motor_phase))
            .collect::<Vec<_>>();

        self.cells[relay].recruited = true;
        self.cells[relay].utility = 0.05;

        for (k, input) in inputs.iter().copied().enumerate() {
            self.synapses.push(PhaseSynapse {
                from: input,
                to: relay,
                weight: 1.0,
                phase_offset: preferred_relative_phase[k],
                eligibility: 1.0,
                confidence: 0.25,
                plastic: true,
            });
        }
        self.synapses.push(PhaseSynapse {
            from: relay,
            to: self.need_cell(),
            weight: target,
            phase_offset: 0.0,
            eligibility: 1.0,
            confidence: 0.60,
            plastic: true,
        });

        self.branches.push(DendriticBranch {
            relay_cell: relay,
            inputs,
            motor: action,
            preferred_relative_phase,
            phase_tolerance: std::f32::consts::PI,
            support: 1,
            utility: 0.10,
            revision: 0,
        });
        Some(relay)
    }

    fn update_context_evidence(&mut self, action: usize, sensory: &[f32], target_need: f32) {
        for (idx, value) in sensory.iter().copied().enumerate() {
            self.context_evidence[action][idx].observe(value >= 0.5, target_need);
        }
    }

    fn select_context_inputs(&self, action: usize, sensory: &[f32]) -> Option<Vec<usize>> {
        let active: Vec<usize> = sensory.iter().enumerate()
            .filter(|(_, x)| **x >= 0.5)
            .map(|(i, _)| i)
            .collect();
        if active.len() < 2 {
            return None;
        }

        // First use only carrier-owned local evidence: a sensory cell is valuable
        // when the same action has produced different factual Need outcomes when
        // that cell was present versus absent. No world label or coordinate enters.
        let mut ranked = active.iter().copied()
            .map(|idx| (self.context_evidence[action][idx].contrast(), idx))
            .collect::<Vec<_>>();
        ranked.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let evidence_inputs = ranked.iter()
            .filter(|(score, _)| *score >= 0.05)
            .map(|(_, idx)| *idx)
            .take(2)
            .collect::<Vec<_>>();
        if evidence_inputs.len() == 2 {
            return Some(evidence_inputs);
        }

        // Developmental fallback before enough positive/negative contrast exists:
        // use phase-role diversity, still without task semantics.
        let anchor = active[0];
        let motor_role = &self.motor_roles[action];
        let mut second = active[1];
        let mut score = self.sensory_roles[second].similarity(motor_role).abs();
        for idx in active.iter().copied().skip(1) {
            let s = self.sensory_roles[idx].similarity(motor_role).abs();
            if s < score {
                score = s;
                second = idx;
            }
        }
        Some(vec![anchor, second])
    }

    fn branch_matches(&self, branch: &DendriticBranch, sensory: &[f32]) -> bool {
        if branch.inputs.iter().any(|idx| sensory[*idx] < 0.5) {
            return false;
        }
        let motor_phase = self.cells[self.motor_cell(branch.motor)].phase;
        branch.inputs.iter().zip(&branch.preferred_relative_phase).all(|(idx, expected)| {
            let actual = signed_phase_error(self.cells[*idx].phase, motor_phase);
            signed_phase_error(*expected, actual).abs() <= branch.phase_tolerance
        })
    }

    fn has_exact_context_branch(&self, action: usize, sensory: &[f32]) -> bool {
        self.branches.iter().any(|b| {
            b.motor == action
                && b.inputs.iter().all(|idx| sensory[*idx] >= 0.5)
        })
    }

    fn load_real(&mut self, sensory: &[f32], need: bool) {
        for (i, x) in sensory.iter().copied().enumerate() {
            self.cells[i].charge = x.clamp(0.0, 1.0);
            if x >= 0.5 {
                self.cells[i].phase = wrap_phase(self.cells[i].phase + 0.01);
            }
        }
        self.cells[self.need_cell()].charge = if need { 1.0 } else { 0.0 };
        self.current_real = Some(FactualFrame {
            sensory: sensory.to_vec(),
            need,
            tick: self.tick,
        });
    }

    fn direct_need_synapse(&self, action: usize) -> Option<&PhaseSynapse> {
        let from = self.motor_cell(action);
        let to = self.need_cell();
        self.synapses.iter().find(|s| s.from == from && s.to == to)
    }

    fn relay_need_synapse(&self, relay: usize) -> Option<&PhaseSynapse> {
        let to = self.need_cell();
        self.synapses.iter().find(|s| s.from == relay && s.to == to)
    }

    fn motor_cell(&self, action: usize) -> usize {
        self.config.sensory_cells + action
    }

    fn need_cell(&self) -> usize {
        self.config.sensory_cells + self.config.motor_cells
    }

    fn dormant_range(&self) -> std::ops::Range<usize> {
        (self.need_cell() + 1)..self.cells.len()
    }

    fn assert_sensory(&self, sensory: &[f32]) {
        assert_eq!(sensory.len(), self.config.sensory_cells);
        assert!(sensory.iter().all(|x| x.is_finite() && *x >= 0.0 && *x <= 1.0));
    }
}
