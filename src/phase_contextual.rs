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

    fn context_signature(
        predecessor_a: usize,
        successor_a: usize,
        predecessor_b: usize,
        successor_b: usize,
    ) -> [(usize, usize); 2] {
        let mut pairs = [
            (predecessor_a, successor_a),
            (predecessor_b, successor_b),
        ];
        pairs.sort_unstable();
        pairs
    }

    fn context_witness_signature(w: &PhaseContextWitness) -> [(usize, usize); 2] {
        Self::context_signature(
            w.predecessor_cells[0],
            w.successor_cells[0],
            w.predecessor_cells[1],
            w.successor_cells[1],
        )
    }

    fn context_find_novel_collision(
        ctx: &PhaseContextState,
        base: usize,
        action: usize,
        previous: usize,
        after: usize,
    ) -> Option<ContextFact> {
        ctx.discovery.iter().rev().find_map(|old| {
            if old.base != base
                || old.action != action
                || old.predecessor == previous
                || old.post == after
            {
                return None;
            }
            let signature = Self::context_signature(
                old.predecessor,
                old.post,
                previous,
                after,
            );
            let duplicate = ctx.candidates.iter().any(|w| {
                !w.retired
                    && w.base_cell == base
                    && w.anchor_action == action
                    && Self::context_witness_signature(w) == signature
            });
            (!duplicate).then_some(old.clone())
        })
    }

    fn context_uncovered_action(
        ctx: &PhaseContextState,
        base: usize,
        previous: usize,
        motor_cells: usize,
    ) -> Option<usize> {
        // Coverage becomes meaningful only after this base has actually been
        // encountered from some other factual predecessor. Ordinary chain
        // states with a single predecessor are left to the existing explorer.
        let has_other_predecessor = ctx.discovery.iter().any(|fact| {
            fact.base == base && fact.predecessor != previous
        });
        if !has_other_predecessor {
            return None;
        }

        (0..motor_cells).find(|action| {
            !ctx.discovery.iter().any(|fact| {
                fact.base == base
                    && fact.predecessor == previous
                    && fact.action == *action
            })
        })
    }

    /// Return (applicable, action). Multiple bounded context hypotheses may
    /// coexist on one acquired base. Only hypotheses whose factual predecessor
    /// pair contains the current predecessor are applicable.
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

        let base_has_promoted = ctx.candidates.iter()
            .any(|w| w.base_cell == base.cell && !w.retired && w.promoted);
        let Some(previous) = ctx.previous_base else {
            // Preserve the G21 restart invariant: a known context-dependent base
            // without factual predecessor history must abstain.
            return if base_has_promoted { (true, None) } else { (false, None) };
        };

        let matching = ctx.candidates.iter()
            .filter(|w| {
                w.base_cell == base.cell
                    && !w.retired
                    && w.predecessor_cells.contains(&previous)
            })
            .collect::<Vec<_>>();
        if matching.is_empty() {
            // An old hypothesis about another predecessor pair must not block
            // ordinary reasoning. But once this same base is known to have
            // multiple factual predecessors, an action unseen under the
            // current predecessor remains a context-coverage frontier.
            if native.config.learning_enabled {
                if let Some(action) = Self::context_uncovered_action(
                    ctx,
                    base.cell,
                    previous,
                    self.config.motor_cells,
                ) {
                    return (true, Some(action));
                }
            }
            return (false, None);
        }

        let floor = native.config.coherence_floor;
        let mut active_states = Vec::new();
        let mut has_promoted_match = false;
        for w in matching.iter().copied().filter(|w| w.promoted) {
            has_promoted_match = true;
            let Some(side) = w.predecessor_cells.iter().position(|&p| p == previous)
                else { continue; };
            let base_link = &self.synapses[w.input_synapses[side][0]];
            let memory_link = &self.synapses[w.input_synapses[side][1]];
            let sensory_current = if base_link.from == base.cell {
                conductance(&self.cells, base_link, floor)
            } else { 0.0 };
            let memory_current = if memory_link.from == previous {
                conductance(&self.cells, memory_link, floor)
            } else { 0.0 };
            if sensory_current.min(memory_current) > 1.0e-8 {
                if !active_states.contains(&w.state_cells[side]) {
                    active_states.push(w.state_cells[side]);
                }
            }
        }

        if active_states.len() > 1 {
            // Competing promoted explanations disagree: do not choose one by a
            // host-side tie-break.
            return (true, None);
        }
        if let Some(&entry) = active_states.first() {
            if native.config.learning_enabled {
                if let Some(action) = (0..self.config.motor_cells)
                    .find(|a| !self.phase_drive_action_known_at(native, entry, *a))
                {
                    return (true, Some(action));
                }
            }
            return (
                true,
                self.phase_native_goal_decision_from_cells(entry, goal.cell, None)
                    .map(|d| d.first_action),
            );
        }
        if has_promoted_match {
            // A required physical context path exists but is damaged/unavailable.
            return (true, None);
        }

        if native.config.learning_enabled {
            // Repair-5: an unfinished hypothesis may request its anchor only
            // when the currently observed predecessor side is not already more
            // sampled than its opposite side. Repeating an over-sampled side
            // cannot reduce the hypothesis's missing information, so ordinary
            // epistemic reasoning must get a chance instead.
            let experiment = matching.iter()
                .copied()
                .filter(|w| !w.promoted)
                .filter_map(|w| {
                    let side = w.predecessor_cells.iter()
                        .position(|&p| p == previous)?;
                    let counts = self.context_counts(native, w);
                    let current = counts[side][0] + counts[side][1];
                    let opposite = counts[1 - side][0] + counts[1 - side][1];
                    (current <= opposite).then_some((
                        current,
                        current + opposite,
                        w.born_fact,
                        w.anchor_action,
                    ))
                })
                .min();

            if let Some((_, _, _, action)) = experiment {
                return (true, Some(action));
            }

            if let Some(action) = Self::context_uncovered_action(
                ctx,
                base.cell,
                previous,
                self.config.motor_cells,
            ) {
                return (true, Some(action));
            }
        }
        // No currently informative context experiment or predecessor-specific
        // coverage gap is available. Lower-priority reasoning may continue.
        (false, None)
    }

    /// Repair-6: G19 may independently probe a rival action only while that
    /// same action still lacks balanced contextual anchor evidence for the
    /// current factual predecessor. If no contextual hypothesis owns the
    /// current base/predecessor/action, rival behavior remains unchanged.
    pub(super) fn phase_context_rival_probe_allowed(
        &self,
        base_cell: usize,
        action: usize,
    ) -> bool {
        let Some(native) = self.phase_native.as_ref() else { return true; };
        let Some(ctx) = native.contextual.as_ref() else { return true; };
        let Some(previous) = ctx.previous_base else { return true; };

        let matching = ctx.candidates.iter().filter(|w| {
            w.base_cell == base_cell
                && w.anchor_action == action
                && !w.retired
                && !w.promoted
                && w.predecessor_cells.contains(&previous)
        }).collect::<Vec<_>>();

        if matching.is_empty() {
            return true;
        }

        matching.into_iter().any(|w| {
            let Some(side) = w.predecessor_cells.iter()
                .position(|&p| p == previous) else { return false; };
            let counts = self.context_counts(native, w);
            let current = counts[side][0] + counts[side][1];
            let opposite = counts[1 - side][0] + counts[1 - side][1];
            current <= opposite
        })
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
        let mut born_index = None;

        if learning {
            ctx.factual_events = ctx.factual_events.saturating_add(1);
            if let Some(previous) = predecessor {
                if ctx.candidates.len() < CONTEXT_CANDIDATE_CAP
                    && self.config.structural_growth_enabled
                {
                    if let Some(old) = Self::context_find_novel_collision(
                        &ctx, pre.cell, action, previous, after.cell)
                    {
                        let free = self.dormant_range()
                            .filter(|i| !self.cells[*i].recruited)
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
                                base_cell: pre.cell,
                                predecessor_cells: predecessors,
                                successor_cells: [old.post, after.cell],
                                state_cells: states,
                                input_synapses: inputs,
                                anchor_action: action,
                                promoted: false,
                                retired: false,
                                born_fact: ctx.factual_events,
                                eligible_observations: 0,
                                log_evidence: 0.0,
                                context_switches: 0,
                                last_context: None,
                                gate_observations: [0; 2],
                            });
                            born_index = Some(ctx.candidates.len() - 1);
                        }
                    }
                }

                let indices = (0..ctx.candidates.len()).filter(|&index| {
                    Some(index) != born_index
                        && ctx.candidates[index].base_cell == pre.cell
                        && !ctx.candidates[index].retired
                        && ctx.candidates[index].predecessor_cells.contains(&previous)
                }).collect::<Vec<_>>();

                for index in indices {
                    let snapshot = ctx.candidates[index].clone();
                    let Some(side) = snapshot.predecessor_cells.iter()
                        .position(|&p| p == previous) else { continue; };

                    for synapse in snapshot.input_synapses[side] {
                        self.learn_phase_concept_running_mean_synapse(
                            synapse, 1.0, snapshot.gate_observations[side]);
                    }
                    ctx.candidates[index].gate_observations[side] =
                        snapshot.gate_observations[side].saturating_add(1);

                    let accepted = self.native_cell_observation(
                        &mut native,
                        snapshot.state_cells[side],
                        action,
                        after.cell,
                        0.0,
                    ).is_some();
                    if !accepted {
                        ctx.candidates[index].retired = true;
                        continue;
                    }

                    if action == snapshot.anchor_action && !snapshot.promoted {
                        if !snapshot.successor_cells.contains(&after.cell) {
                            ctx.candidates[index].retired = true;
                        } else {
                            let counts = self.context_counts(&native, &snapshot);
                            let w = &mut ctx.candidates[index];
                            w.eligible_observations =
                                w.eligible_observations.saturating_add(1);
                            if w.last_context
                                .map(|last| last != side).unwrap_or(false)
                            {
                                w.context_switches =
                                    w.context_switches.saturating_add(1);
                            }
                            w.last_context = Some(side);
                            w.log_evidence = context_log_evidence(counts);
                            w.promoted = context_gate(counts, w.context_switches);
                            if !w.promoted
                                && w.eligible_observations >= CONTEXT_MAX_SAMPLES
                            {
                                w.retired = true;
                            }
                        }
                    }
                }

                if ctx.discovery.len() == CONTEXT_DISCOVERY_CAP {
                    ctx.discovery.remove(0);
                }
                ctx.discovery.push(ContextFact {
                    predecessor: previous,
                    base: pre.cell,
                    action,
                    post: after.cell,
                });
            }
        }

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
            if !self.observe_phase_native_abstract_transition(
                &pre_sensory, action, post, 0.0)
            {
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
