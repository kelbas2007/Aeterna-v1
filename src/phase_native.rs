//! P1: bounded, phase-coupled value propagation in the SAME carrier arrays.
//! This child module has access to the carrier's private physical substrate.
//! It does not use a transition table, frontier, route enumeration or planner.
//! The recurrence and local winner-take-all rule are inherited, not learned.

use super::{EvoPhase, PhaseCell, PhaseSynapse};
use crate::authority::Authority;
use crate::phase::{signed_phase_error, wrap_phase};
use crate::planning::PlanDecision;
use crate::trace::CarrierTrace;

#[derive(Debug, Clone)]
pub struct PhaseNativeConfig {
    pub match_threshold: f32,
    pub coherence_floor: f32,
    pub horizon: usize,
    pub discount: f32,
    pub learning_enabled: bool,
}

impl Default for PhaseNativeConfig {
    fn default() -> Self {
        Self {
            match_threshold: 0.97,
            coherence_floor: 0.95,
            horizon: 6,
            discount: 0.95,
            learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
struct Receptor {
    trace: CarrierTrace,
    cell: usize,
}

/// Structural addresses, not an executable copy of transition/reward content.
#[derive(Debug, Clone)]
pub struct PhaseCircuitInfo {
    pub relay_cell: usize,
    pub afferent_synapse: usize,
    pub successor_synapse: usize,
    pub outcome_synapse: usize,
    pub motor_synapse: usize,
    pub support: u64,
    pub revision: u64,
    pub counterexamples: Vec<(u64, f32)>,
}

impl PhaseCircuitInfo {
    fn indices(&self) -> [usize; 4] {
        [self.afferent_synapse, self.successor_synapse,
         self.outcome_synapse, self.motor_synapse]
    }
}

#[derive(Debug, Clone)]
pub(super) struct PhaseNativeState {
    pub(super) config: PhaseNativeConfig,
    receptors: Vec<Receptor>,
    circuits: Vec<PhaseCircuitInfo>,
    last_motor_potentials: Vec<f32>,
    last_local_updates: usize,
}

fn coherence(cells: &[PhaseCell], syn: &PhaseSynapse, floor: f32) -> f32 {
    if !cells[syn.from].recruited || !cells[syn.to].recruited {
        return 0.0;
    }
    let arrived = wrap_phase(cells[syn.from].phase + syn.phase_offset);
    let agreement = signed_phase_error(cells[syn.to].phase, arrived).cos();
    ((agreement - floor) / (1.0 - floor)).clamp(0.0, 1.0)
}

fn conductance(cells: &[PhaseCell], syn: &PhaseSynapse, floor: f32) -> f32 {
    syn.weight.clamp(0.0, 1.0) * coherence(cells, syn, floor)
}

impl EvoPhase {
    /// Starts a cold native circuit. It never compiles the legacy planner table.
    /// Explicit opt-in preserves the historical negative audit of the old path.
    pub fn enable_phase_native_planning(&mut self, config: PhaseNativeConfig) {
        assert!(self.phase_native.is_none(), "native circuit already enabled");
        assert!(config.horizon > 0 && config.horizon <= 1024);
        assert!((0.0..=1.0).contains(&config.match_threshold));
        assert!((0.0..1.0).contains(&config.coherence_floor));
        assert!((0.0..1.0).contains(&config.discount));
        assert!((0.0..=1.0).contains(&self.config.weight_learning_rate));
        assert!((0.0..=1.0).contains(&self.config.phase_learning_rate));
        self.imagination_planner = None;
        self.phase_native = Some(PhaseNativeState {
            config,
            receptors: Vec::new(),
            circuits: Vec::new(),
            last_motor_potentials: vec![0.0; self.config.motor_cells],
            last_local_updates: 0,
        });
    }

    pub fn phase_native_circuits(&self) -> &[PhaseCircuitInfo] {
        self.phase_native.as_ref().map(|s| s.circuits.as_slice()).unwrap_or(&[])
    }

    pub fn phase_native_motor_potentials(&self) -> &[f32] {
        self.phase_native.as_ref()
            .map(|s| s.last_motor_potentials.as_slice()).unwrap_or(&[])
    }

    pub fn phase_native_local_updates(&self) -> usize {
        self.phase_native.as_ref().map(|s| s.last_local_updates).unwrap_or(0)
    }

    pub fn phase_native_synapse(&self, index: usize) -> Option<PhaseSynapse> {
        let state = self.phase_native.as_ref()?;
        if !state.circuits.iter().any(|c| c.indices().contains(&index))
            && !self.is_native_decoder_synapse(index) {
            return None;
        }
        self.synapses.get(index).cloned()
    }

    /// Diagnostic intervention on the ACTUAL synapse, not on a readout flag.
    pub fn perturb_phase_native_synapse_for_control(
        &mut self, index: usize, weight_scale: f32, phase_shift: f32,
    ) -> Option<PhaseSynapse> {
        assert!(weight_scale.is_finite() && (0.0..=1.0).contains(&weight_scale));
        assert!(phase_shift.is_finite());
        let old = self.phase_native_synapse(index)?;
        let syn = &mut self.synapses[index];
        syn.weight *= weight_scale;
        syn.phase_offset = wrap_phase(syn.phase_offset + phase_shift);
        Some(old)
    }

    pub fn restore_phase_native_synapse_for_control(
        &mut self, index: usize, saved: PhaseSynapse,
    ) {
        let current = self.phase_native_synapse(index).expect("native synapse address");
        assert_eq!((current.from, current.to), (saved.from, saved.to));
        assert!(saved.weight.is_finite() && saved.phase_offset.is_finite());
        self.synapses[index] = saved;
    }

    /// Fingerprint of learned metadata and the actual shared physical substrate.
    /// Readout diagnostics and IMAGINED scratch membranes are deliberately absent.
    pub fn phase_native_learned_fingerprint(&self) -> u64 {
        let Some(state) = self.phase_native.as_ref() else { return 0; };
        let mut h = 14_695_981_039_346_656_037_u64;
        let text = format!("{:?}|{:?}|{:?}|{:?}",
            state.receptors, state.circuits, self.cells, self.synapses);
        for byte in text.bytes() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(1_099_511_628_211);
        }
        h
    }

    fn native_receptor(&mut self, state: &mut PhaseNativeState, trace: CarrierTrace) -> Option<usize> {
        if let Some(r) = state.receptors.iter()
            .find(|r| r.trace.similarity(&trace) >= state.config.match_threshold) {
            return Some(r.cell);
        }
        if !self.config.structural_growth_enabled { return None; }
        let cell = self.dormant_range().find(|i| !self.cells[*i].recruited)?;
        self.cells[cell].recruited = true;
        state.receptors.push(Receptor { trace, cell });
        Some(cell)
    }

    fn native_synapse(&mut self, from: usize, to: usize) -> usize {
        let index = self.synapses.len();
        self.synapses.push(PhaseSynapse {
            from, to, weight: 0.0, phase_offset: 0.0,
            eligibility: 0.0, confidence: 0.0, plastic: true,
        });
        index
    }

    /// Factual coactivity is the only path that updates acquired synaptic values.
    pub(super) fn observe_phase_native_trace(
        &mut self, pre: CarrierTrace, action: usize, post: CarrierTrace, value: f32,
    ) {
        assert!(action < self.config.motor_cells);
        assert!(value.is_finite() && (0.0..=1.0).contains(&value),
            "P1 qualifies nonnegative bounded outcomes only");
        let mut state = self.phase_native.take().expect("enable native mode first");
        if !state.config.learning_enabled {
            self.phase_native = Some(state);
            return;
        }
        self.native_observation(&mut state, pre, action, post, value);
        self.phase_native = Some(state);
    }

    fn native_observation(
        &mut self, state: &mut PhaseNativeState,
        pre: CarrierTrace, action: usize, post: CarrierTrace, value: f32,
    ) -> Option<()> {
        let matched_pre = state.receptors.iter()
            .find(|r| r.trace.similarity(&pre) >= state.config.match_threshold).map(|r| r.cell);
        let matched_post = state.receptors.iter()
            .find(|r| r.trace.similarity(&post) >= state.config.match_threshold).map(|r| r.cell);
        let motor = self.motor_cell(action);
        let existing = state.circuits.iter().position(|c| {
            Some(self.synapses[c.afferent_synapse].from) == matched_pre
                && Some(self.synapses[c.successor_synapse].to) == matched_post
                && self.synapses[c.motor_synapse].to == motor
        });
        if existing.is_none() {
            if !self.config.structural_growth_enabled { return None; }
            // Conservative preflight prevents partial allocation on capacity failure.
            let required = usize::from(matched_pre.is_none())
                + usize::from(matched_post.is_none()) + 1;
            if self.dormant_range().filter(|i| !self.cells[*i].recruited).count() < required {
                return None;
            }
        }
        let from = self.native_receptor(state, pre)?;
        let to = self.native_receptor(state, post)?;
        let ci = if let Some(index) = existing { index } else {
            let relay = self.dormant_range().find(|i| !self.cells[*i].recruited)?;
            self.cells[relay].recruited = true;
            let afferent = self.native_synapse(from, relay);
            let successor = self.native_synapse(relay, to);
            let outcome = self.native_synapse(relay, self.need_cell());
            let motor_synapse = self.native_synapse(relay, motor);
            state.circuits.push(PhaseCircuitInfo {
                relay_cell: relay, afferent_synapse: afferent,
                successor_synapse: successor, outcome_synapse: outcome,
                motor_synapse, support: 0, revision: 0, counterexamples: Vec::new(),
            });
            state.circuits.len() - 1
        };
        let circuit = &mut state.circuits[ci];
        let residual = (value - self.synapses[circuit.outcome_synapse].weight).abs();
        if circuit.support >= u64::from(self.config.min_recruit_support)
            && residual >= self.config.residual_recruit_threshold {
            circuit.revision = circuit.revision.saturating_add(1);
            circuit.counterexamples.push((circuit.support, value));
        }
        circuit.support = circuit.support.saturating_add(1);
        for index in circuit.indices() {
            let syn = &mut self.synapses[index];
            if !syn.plastic { continue; }
            let target = if index == circuit.outcome_synapse { value } else { 1.0 };
            syn.eligibility = 1.0;
            syn.weight += self.config.weight_learning_rate * syn.eligibility * (target - syn.weight);
            let target_phase = wrap_phase(self.cells[syn.to].phase - self.cells[syn.from].phase);
            syn.phase_offset = wrap_phase(syn.phase_offset
                + self.config.phase_learning_rate * syn.eligibility
                    * signed_phase_error(target_phase, syn.phase_offset));
            syn.confidence += self.config.weight_learning_rate * (1.0 - syn.confidence);
        }
        Some(())
    }

    pub(super) fn phase_native_decision(
        &mut self, trace: &CarrierTrace, depth: Option<usize>,
    ) -> Option<PlanDecision> {
        let state = self.phase_native.as_mut()?;
        state.last_local_updates = 0;
        state.last_motor_potentials.fill(0.0);
        let entry = state.receptors.iter()
            .find(|r| r.trace.similarity(trace) >= state.config.match_threshold)?.cell;
        let horizon = depth.unwrap_or(state.config.horizon).min(state.config.horizon).max(1);
        let floor = state.config.coherence_floor;
        // Mode-isolated membranes indexed by the SAME physical cell IDs.
        // No new task topology, synthetic transitions, graph paths or action scripts.
        let mut membranes = self.cells.clone();
        for cell in &mut membranes { cell.charge = 0.0; }
        let mut latency = vec![0usize; membranes.len()];
        for _ in 0..horizon {
            let old = membranes.clone();
            let old_latency = latency.clone();
            for cell in &mut membranes { cell.charge = 0.0; }
            latency.fill(0);
            for c in &state.circuits {
                state.last_local_updates += 1;
                if c.support < u64::from(self.config.min_recruit_support) { continue; }
                let afferent = &self.synapses[c.afferent_synapse];
                let successor = &self.synapses[c.successor_synapse];
                let outcome = &self.synapses[c.outcome_synapse];
                let future = state.config.discount
                    * conductance(&old, successor, floor) * old[successor.to].charge;
                let immediate = outcome.weight.clamp(0.0, 1.0) * coherence(&old, outcome, floor);
                let current = conductance(&old, afferent, floor) * (immediate + future);
                membranes[c.relay_cell].charge = current;
                let delay = if future > 1.0e-8 { old_latency[successor.to] + 1 } else { 1 };
                latency[c.relay_cell] = delay;
                // Fixed local dendritic competition, not route comparison.
                if current > membranes[afferent.from].charge {
                    membranes[afferent.from].charge = current;
                    latency[afferent.from] = delay;
                }
            }
        }
        let mut motor_latency = vec![0usize; self.config.motor_cells];
        for c in &state.circuits {
            if self.synapses[c.afferent_synapse].from != entry { continue; }
            let output = &self.synapses[c.motor_synapse];
            let action = output.to.checked_sub(self.config.sensory_cells)?;
            if action >= self.config.motor_cells { return None; }
            let current = membranes[c.relay_cell].charge * conductance(&membranes, output, floor);
            if current > state.last_motor_potentials[action] {
                state.last_motor_potentials[action] = current;
                motor_latency[action] = latency[c.relay_cell];
            }
        }
        let mut selected = None;
        let mut peak = 1.0e-8;
        for (motor, potential) in state.last_motor_potentials.iter().copied().enumerate() {
            if potential > peak {
                peak = potential;
                selected = Some(motor);
            }
        }
        let action = selected?;
        Some(PlanDecision {
            first_action: action, predicted_value: peak,
            selected_depth: motor_latency[action],
            // Compatibility field: local relay updates, NOT graph nodes in native mode.
            expanded_nodes: state.last_local_updates,
            authority: Authority::Imagined,
        })
    }
}

include!("phase_forward.rs");
