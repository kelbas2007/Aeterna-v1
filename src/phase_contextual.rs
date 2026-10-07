// G21. Included in phase_native.rs: representation and learned transitions use
// the same private EvoPhase cell/synapse substrate. A bounded discovery record
// selects candidates; FUTURE physical transition supports alone validate them.

const CONTEXT_CANDIDATE_CAP: usize = 16;
const CONTEXT_DISCOVERY_CAP: usize = 128;
const CONTEXT_MIN_SAMPLES: u64 = 32;
const CONTEXT_MAX_SAMPLES: u64 = 128;
const CONTEXT_ALPHA: f64 = 0.01;

#[derive(Debug, Clone)]
struct ContextFact {
    predecessor: usize,
    base: usize,
    action: usize,
    post: usize,
}

#[derive(Debug, Clone)]
pub struct PhaseContextWitness {
    pub base_cell: usize,
    pub predecessor_cells: [usize; 2],
    pub successor_cells: [usize; 2],
    pub state_cells: [usize; 2],
    pub input_synapses: [[usize; 2]; 2],
    pub anchor_action: usize,
    pub promoted: bool,
    pub retired: bool,
    pub born_fact: u64,
    pub eligible_observations: u64,
    pub log_evidence: f64,
    pub context_switches: u64,
    last_context: Option<usize>,
    gate_observations: [u32; 2],
}

#[derive(Debug, Clone, Default)]
pub(super) struct PhaseContextState {
    previous_base: Option<usize>,
    discovery: Vec<ContextFact>,
    candidates: Vec<PhaseContextWitness>,
    factual_events: u64,
}

// Integrated Jeffreys/KT binary likelihood for an ORDERED observation string.
// No binomial coefficient: Q is a sequential probability, not a count mass.
fn context_log_kt(a: u64, b: u64) -> f64 {
    let positive = (0..a).map(|i| (i as f64 + 0.5).ln()).sum::<f64>()
        + (0..b).map(|i| (i as f64 + 0.5).ln()).sum::<f64>();
    positive - (1..=a + b).map(|i| (i as f64).ln()).sum::<f64>()
}

fn context_log_evidence(counts: [[u64; 2]; 2]) -> f64 {
    let a = counts[0][0] + counts[1][0];
    let b = counts[0][1] + counts[1][1];
    let n = a + b;
    if n == 0 { return 0.0; }
    let null_mle = [a, b].iter().filter(|&&x| x > 0)
        .map(|&x| x as f64 * (x as f64 / n as f64).ln()).sum::<f64>();
    context_log_kt(counts[0][0], counts[0][1])
        + context_log_kt(counts[1][0], counts[1][1]) - null_mle
}

fn context_gate(counts: [[u64; 2]; 2], switches: u64) -> bool {
    let n0 = counts[0][0] + counts[0][1];
    let n1 = counts[1][0] + counts[1][1];
    if n0 + n1 < CONTEXT_MIN_SAMPLES || n0 < 8 || n1 < 8 || switches < 4 {
        return false;
    }
    let effect = (counts[0][1] as f64 / n0 as f64
        - counts[1][1] as f64 / n1 as f64).abs();
    effect >= 0.60
        && context_log_evidence(counts)
            >= (CONTEXT_CANDIDATE_CAP as f64 / CONTEXT_ALPHA).ln()
}

impl EvoPhase {
    pub fn enable_phase_native_context_refinement(&mut self) -> bool {
        let Some(native) = self.phase_native.as_mut() else { return false; };
        if native.contextual.is_some() { return false; }
        native.contextual = Some(PhaseContextState::default());
        true
    }

    pub fn phase_native_context_enabled(&self) -> bool {
        self.phase_native.as_ref().map(|s| s.contextual.is_some()).unwrap_or(false)
    }

    pub fn phase_native_context_witnesses(&self) -> Vec<PhaseContextWitness> {
        self.phase_native.as_ref().and_then(|s| s.contextual.as_ref())
            .map(|c| c.candidates.clone()).unwrap_or_default()
    }

    /// Fresh external sensing is not permission to fabricate pre-restart history.
    pub fn clear_phase_native_context_history(&mut self) {
        if let Some(c) = self.phase_native.as_mut().and_then(|s| s.contextual.as_mut()) {
            c.previous_base = None;
        }
    }

    pub(super) fn is_native_context_synapse(&self, index: usize) -> bool {
        self.phase_native.as_ref().and_then(|s| s.contextual.as_ref())
            .map(|c| c.candidates.iter().any(|w|
                w.input_synapses.iter().any(|pair| pair.contains(&index))))
            .unwrap_or(false)
    }

    fn context_counts(&self, native: &PhaseNativeState, w: &PhaseContextWitness)
        -> [[u64; 2]; 2]
    {
        let mut counts = [[0u64; 2]; 2];
        let motor = self.motor_cell(w.anchor_action);
        for circuit in &native.circuits {
            let from = self.synapses[circuit.afferent_synapse].from;
            let to = self.synapses[circuit.successor_synapse].to;
            if self.synapses[circuit.motor_synapse].to != motor { continue; }
            for side in 0..2 {
                for outcome in 0..2 {
                    if from == w.state_cells[side] && to == w.successor_cells[outcome] {
                        counts[side][outcome] += circuit.support;
                    }
                }
            }
        }
        counts
    }

    /// Return (applicable, action). An applicable but unavailable contextual
    /// state MUST NOT silently fall back to the ambiguous parent representation.
    /// Goal information affects choice, never the stored evidence or split gate.
    pub fn phase_native_context_action(&mut self, goal_sensory: &[f32])
        -> (bool, Option<usize>)
    {
        let Some(real) = self.current_real.as_ref() else { return (false, None); };
        let Some(base) = self.phase_native_abstract_state(&real.sensory)
            else { return (false, None); };
        let Some(goal) = self.phase_native_abstract_state(goal_sensory)
            else { return (false, None); };
        let Some(native) = self.phase_native.as_ref() else { return (false, None); };
        let Some(ctx) = native.contextual.as_ref() else { return (false, None); };
        let Some(w) = ctx.candidates.iter()
            .find(|w| w.base_cell == base.cell && !w.retired)
            else { return (false, None); };
        if !w.promoted {
            // A selected diagnostic experiment, not an evaluator answer. Its
            // discovery observations were excluded from the inference score.
            if native.config.learning_enabled { return (true, Some(w.anchor_action)); }
            return (false, None);
        }
        let Some(previous) = ctx.previous_base else { return (true, None); };
        let floor = native.config.coherence_floor;
        let mut active = None;
        for side in 0..2 {
            let base_link = &self.synapses[w.input_synapses[side][0]];
            let memory_link = &self.synapses[w.input_synapses[side][1]];
            // Current sensory state and actual previous-state activity must
            // both arrive through learned physical links. Tags alone cannot
            // make the context usable when the links are damaged.
            let sensory_current = if base_link.from == base.cell {
                conductance(&self.cells, base_link, floor)
            } else { 0.0 };
            let memory_current = if memory_link.from == previous {
                conductance(&self.cells, memory_link, floor)
            } else { 0.0 };
            if sensory_current.min(memory_current) > 1.0e-8 {
                if active.is_some() { return (true, None); }
                active = Some(w.state_cells[side]);
            }
        }
        let Some(entry) = active else { return (true, None); };
        if native.config.learning_enabled {
            // New contextual outgoing knowledge is acquired by actual actions,
            // not transferred from a parent answer or a hypothetical POST.
            let unknown = (0..self.config.motor_cells)
                .find(|a| !self.phase_drive_action_known_at(native, entry, *a));
            if let Some(action) = unknown { return (true, Some(action)); }
        }
        (true, self.phase_native_goal_decision_from_cells(entry, goal.cell, None)
            .map(|d| d.first_action))
    }

    /// Actual POST only. Existing G20 behavior is unchanged unless explicitly
    /// enabled. Context discovery/recruitment and transition learning are native
    /// carrier operations; the host never receives a split or context-ID API.
    pub fn observe_phase_native_context_result(&mut self, action: usize, post: &[f32])
        -> Option<usize>
    {
        if action >= self.config.motor_cells { return None; }
        let pre_sensory = self.current_real.as_ref()?.sensory.clone();
        let pre = self.phase_native_abstract_state(&pre_sensory)?;
        let after = self.phase_native_abstract_state(post)?;
        let mut native = self.phase_native.take()?;
        let Some(mut ctx) = native.contextual.take() else {
            self.phase_native = Some(native);
            return self.observe_phase_native_rival_probe_result(action, post);
        };
        let learning = native.config.learning_enabled;
        let predecessor = ctx.previous_base;
        let mut born_now = false;
        if learning {
            ctx.factual_events = ctx.factual_events.saturating_add(1);
            if let Some(previous) = predecessor {
                let no_candidate = !ctx.candidates.iter().any(|w| w.base_cell == pre.cell);
                if no_candidate && ctx.candidates.len() < CONTEXT_CANDIDATE_CAP
                    && self.config.structural_growth_enabled
                {
                    let opposing = ctx.discovery.iter().rev().find(|f|
                        f.base == pre.cell && f.action == action
                        && f.predecessor != previous && f.post != after.cell).cloned();
                    if let Some(old) = opposing {
                        let free = self.dormant_range().filter(|i| !self.cells[*i].recruited)
                            .take(2).collect::<Vec<_>>();
                        if free.len() == 2 {
                            for &cell in &free { self.cells[cell].recruited = true; }
                            let states = [free[0], free[1]];
                            let predecessors = [old.predecessor, previous];
                            let inputs = [
                                [self.native_synapse(pre.cell, states[0]),
                                 self.native_synapse(predecessors[0], states[0])],
                                [self.native_synapse(pre.cell, states[1]),
                                 self.native_synapse(predecessors[1], states[1])],
                            ];
                            ctx.candidates.push(PhaseContextWitness {
                                base_cell: pre.cell, predecessor_cells: predecessors,
                                successor_cells: [old.post, after.cell], state_cells: states,
                                input_synapses: inputs, anchor_action: action,
                                promoted: false, retired: false, born_fact: ctx.factual_events,
                                eligible_observations: 0, log_evidence: 0.0,
                                context_switches: 0, last_context: None,
                                gate_observations: [0; 2],
                            });
                            born_now = true;
                        }
                    }
                }
                if !born_now {
                    if let Some(index) = ctx.candidates.iter()
                        .position(|w| w.base_cell == pre.cell && !w.retired)
                    {
                        let side = ctx.candidates[index].predecessor_cells.iter()
                            .position(|&p| p == previous);
                        if let Some(side) = side {
                            let snapshot = ctx.candidates[index].clone();
                            for synapse in snapshot.input_synapses[side] {
                                self.learn_phase_concept_running_mean_synapse(
                                    synapse, 1.0, snapshot.gate_observations[side]);
                            }
                            ctx.candidates[index].gate_observations[side] =
                                snapshot.gate_observations[side].saturating_add(1);
                            let accepted = self.native_cell_observation(
                                &mut native, snapshot.state_cells[side], action, after.cell, 0.0
                            ).is_some();
                            if !accepted { ctx.candidates[index].retired = true; }
                            if action == snapshot.anchor_action && !snapshot.promoted {
                                if !snapshot.successor_cells.contains(&after.cell) {
                                    ctx.candidates[index].retired = true;
                                } else if accepted {
                                    let counts = self.context_counts(&native, &snapshot);
                                    let w = &mut ctx.candidates[index];
                                    w.eligible_observations += 1;
                                    if w.last_context.map(|last| last != side).unwrap_or(false) {
                                        w.context_switches += 1;
                                    }
                                    w.last_context = Some(side);
                                    w.log_evidence = context_log_evidence(counts);
                                    w.promoted = context_gate(counts, w.context_switches);
                                    if !w.promoted && w.eligible_observations >= CONTEXT_MAX_SAMPLES {
                                        w.retired = true;
                                    }
                                }
                            }
                        }
                    }
                }
                if ctx.discovery.len() == CONTEXT_DISCOVERY_CAP { ctx.discovery.remove(0); }
                ctx.discovery.push(ContextFact {
                    predecessor: previous, base: pre.cell, action, post: after.cell,
                });
            }
        }
        // Transient, factual short-term memory advances even when learning is
        // frozen. It is cleared on external re-sensing and cognitive restart.
        ctx.previous_base = Some(pre.cell);
        let preserve_rivals = ctx.candidates.iter()
            .any(|w| w.base_cell == pre.cell && !w.retired);
        native.contextual = Some(ctx);
        self.phase_native = Some(native);
        if !learning {
            self.observe_initial_real(post, false);
            return Some(0);
        }
        if preserve_rivals {
            // The unresolved/conditioned parent alternatives remain present.
            // No last-observation-wins erasure is performed for this base.
            if !self.observe_phase_native_abstract_transition(&pre_sensory, action, post, 0.0) {
                return None;
            }
            self.observe_initial_real(post, false);
            Some(0)
        } else {
            self.observe_phase_native_rival_probe_result(action, post)
        }
    }
}

#[cfg(test)]
mod context_evidence_tests {
    use super::*;

    #[test]
    fn context_gate_requires_future_support_and_distinguishes_noise() {
        assert!(!context_gate([[1, 0], [0, 1]], 1));
        assert!(!context_gate([[16, 0], [0, 16]], 1));
        assert!(context_gate([[16, 0], [0, 16]], 10));
        assert!(!context_gate([[16, 16], [16, 16]], 30));
        let mut crossings = 0usize;
        for stream in 0..512u64 {
            let mut rng = (stream + 1).wrapping_mul(0x9E3779B97F4A7C15);
            let mut counts = [[0u64; 2]; 2];
            let mut switches = 0;
            let mut previous = None;
            let mut crossed = false;
            for _ in 0..128 {
                rng ^= rng >> 12; rng ^= rng << 25; rng ^= rng >> 27;
                let c = ((rng.wrapping_mul(0x2545F4914F6CDD1D) >> 60) & 1) as usize;
                rng ^= rng >> 12; rng ^= rng << 25; rng ^= rng >> 27;
                let threshold = [1u64, 2, 3][stream as usize % 3];
                let y = usize::from((rng.wrapping_mul(0x2545F4914F6CDD1D) >> 62) < threshold);
                if previous.map(|p| p != c).unwrap_or(false) { switches += 1; }
                previous = Some(c);
                counts[c][y] += 1;
                if context_gate(counts, switches) { crossed = true; break; }
            }
            crossings += usize::from(crossed);
        }
        println!("G21_NULL false_promotions={}/512 limit=15 post_selection_only=true", crossings);
        assert!(crossings <= 15);
    }

    #[test]
    fn context_log_probability_matches_sequential_kt_updates() {
        let stream = [(0,0),(1,1),(0,1),(0,0),(1,1),(1,0),(0,1)];
        let mut counts = [[0u64; 2]; 2];
        let mut log_q = 0.0;
        for (c,y) in stream {
            log_q += ((counts[c][y] as f64 + 0.5)
                / (counts[c][0] + counts[c][1] + 1) as f64).ln();
            counts[c][y] += 1;
        }
        let batch = context_log_kt(counts[0][0], counts[0][1])
            + context_log_kt(counts[1][0], counts[1][1]);
        assert!((batch - log_q).abs() < 1.0e-12);
    }
}
