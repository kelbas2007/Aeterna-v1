// G23. Evidence-gated composition of raw perceptual descriptors.
// Included by phase_native.rs, sharing the private carrier substrate.

const COMPOSITION_CANDIDATE_CAP: usize = 16;
const COMPOSITION_DISCOVERY_CAP: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhasePerceptProgram {
    Atom(PhasePerceptFeature),
    And(PhasePerceptFeature, PhasePerceptFeature),
    Xor(PhasePerceptFeature, PhasePerceptFeature),
}

impl PhasePerceptProgram {
    fn atoms(self) -> [PhasePerceptFeature; 2] {
        match self {
            Self::Atom(a) => [a, a],
            Self::And(a, b) | Self::Xor(a, b) => [a, b],
        }
    }

    fn eval(self, features: &[PhasePerceptFeature]) -> bool {
        let contains = |f: &PhasePerceptFeature| features.binary_search(f).is_ok();
        match self {
            Self::Atom(a) => contains(&a),
            Self::And(a, b) => contains(&a) && contains(&b),
            Self::Xor(a, b) => contains(&a) ^ contains(&b),
        }
    }

    fn node_count(self) -> u8 {
        match self {
            Self::Atom(_) => 1,
            Self::And(_, _) | Self::Xor(_, _) => 3,
        }
    }

    fn op_rank(self) -> u8 {
        match self {
            Self::Atom(_) => 0,
            Self::And(_, _) => 1,
            Self::Xor(_, _) => 2,
        }
    }
}

#[derive(Debug, Clone)]
struct CompositionFact {
    base: usize,
    action: usize,
    post: usize,
    features: Vec<PhasePerceptFeature>,
}

#[derive(Debug, Clone)]
pub struct PhaseCompositionWitness {
    pub base_cell: usize,
    pub program: PhasePerceptProgram,
    pub program_cells: [usize; 2],
    pub successor_cells: [usize; 2],
    pub state_cells: [usize; 2],
    pub input_synapses: [[usize; 2]; 2],
    pub anchor_action: usize,
    pub promoted: bool,
    pub retired: bool,
    pub born_fact: u64,
    pub eligible_observations: u64,
    pub log_evidence: f64,
    pub side_switches: u64,
    pub atom_effects: [f64; 2],
    last_side: Option<usize>,
    gate_observations: [u32; 2],
    atom_counts: [[[u64; 2]; 2]; 2],
}

#[derive(Debug, Clone, Default)]
pub(super) struct PhaseCompositionState {
    discovery: Vec<CompositionFact>,
    candidates: Vec<PhaseCompositionWitness>,
    factual_events: u64,
}

impl EvoPhase {
    pub fn enable_phase_native_compositional_refinement(&mut self) -> bool {
        let Some(native) = self.phase_native.as_mut() else { return false; };
        if native.compositional.is_some() || self.concept_memory.is_none() {
            return false;
        }
        native.compositional = Some(PhaseCompositionState::default());
        true
    }

    pub fn phase_native_compositional_enabled(&self) -> bool {
        self.phase_native.as_ref()
            .map(|s| s.compositional.is_some()).unwrap_or(false)
    }

    pub fn phase_native_composition_witnesses(&self) -> Vec<PhaseCompositionWitness> {
        self.phase_native.as_ref().and_then(|s| s.compositional.as_ref())
            .map(|c| c.candidates.clone()).unwrap_or_default()
    }

    pub(super) fn is_native_composition_synapse(&self, index: usize) -> bool {
        self.phase_native.as_ref().and_then(|s| s.compositional.as_ref())
            .map(|c| c.candidates.iter().any(|w|
                w.input_synapses.iter().any(|pair| pair.contains(&index))))
            .unwrap_or(false)
    }

    fn canonical_pair(
        a: PhasePerceptFeature,
        b: PhasePerceptFeature,
    ) -> (PhasePerceptFeature, PhasePerceptFeature) {
        if a <= b { (a, b) } else { (b, a) }
    }

    fn candidate_programs(
        old: &[PhasePerceptFeature],
        new: &[PhasePerceptFeature],
    ) -> Vec<PhasePerceptProgram> {
        let mut union = old.iter().chain(new.iter()).copied()
            .collect::<Vec<_>>();
        union.sort_unstable();
        union.dedup();
        let mut programs = Vec::new();

        for &a in &union {
            let p = PhasePerceptProgram::Atom(a);
            if p.eval(old) != p.eval(new) { programs.push(p); }
        }
        for i in 0..union.len() {
            for j in (i + 1)..union.len() {
                let (a, b) = Self::canonical_pair(union[i], union[j]);
                for p in [
                    PhasePerceptProgram::And(a, b),
                    PhasePerceptProgram::Xor(a, b),
                ] {
                    if p.eval(old) != p.eval(new) { programs.push(p); }
                }
            }
        }

        programs.sort_by_key(|p| {
            let atoms = p.atoms();
            (p.node_count(), p.op_rank(), atoms[0], atoms[1])
        });
        programs.dedup();
        programs
    }

    // Choose an untried explanation from factual contradictory observations.
    // Retired hypotheses remain in the provenance ledger and cannot be reborn
    // with an identical (base, action, program, unordered successor pair).
    fn composition_untried_hypotheses(
        comp: &PhaseCompositionState,
        base: usize,
        action: usize,
        after: usize,
        raw: &[PhasePerceptFeature],
    ) -> Vec<(PhasePerceptProgram, usize)> {
        let prior_on_base = comp.candidates.iter().any(|w| w.base_cell == base);
        for old in comp.discovery.iter().rev() {
            if old.base != base || old.action != action || old.post == after {
                continue;
            }
            let mut untried = Self::candidate_programs(&old.features, raw)
                .into_iter()
                .filter(|program| !comp.candidates.iter().any(|w| {
                    w.base_cell == base
                        && w.anchor_action == action
                        && w.program == *program
                        && ((w.successor_cells[0] == old.post
                            && w.successor_cells[1] == after)
                            || (w.successor_cells[0] == after
                                && w.successor_cells[1] == old.post))
                }))
                .map(|program| (program, old.post))
                .collect::<Vec<_>>();
            if !untried.is_empty() {
                // Continue a failed hypothesis search incrementally, instead
                // of allocating the entire bounded carrier for one collision.
                if prior_on_base {
                    untried.truncate(1);
                }
                return untried;
            }
        }
        Vec::new()
    }

    fn composition_counts(
        &self,
        native: &PhaseNativeState,
        w: &PhaseCompositionWitness,
    ) -> [[u64; 2]; 2] {
        let mut counts = [[0u64; 2]; 2];
        let motor = self.motor_cell(w.anchor_action);
        for circuit in &native.circuits {
            if self.synapses[circuit.motor_synapse].to != motor { continue; }
            let from = self.synapses[circuit.afferent_synapse].from;
            let to = self.synapses[circuit.successor_synapse].to;
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

    fn binary_effect(counts: [[u64; 2]; 2]) -> f64 {
        let n0 = counts[0][0] + counts[0][1];
        let n1 = counts[1][0] + counts[1][1];
        if n0 == 0 || n1 == 0 { return 0.0; }
        (counts[0][1] as f64 / n0 as f64
            - counts[1][1] as f64 / n1 as f64).abs()
    }

    fn atom_effects(w: &PhaseCompositionWitness) -> [f64; 2] {
        [
            Self::binary_effect(w.atom_counts[0]),
            Self::binary_effect(w.atom_counts[1]),
        ]
    }

    fn composition_gate(
        w: &PhaseCompositionWitness,
        counts: [[u64; 2]; 2],
    ) -> bool {
        if !context_gate(counts, w.side_switches) {
            return false;
        }
        match w.program {
            PhasePerceptProgram::Atom(_) => true,
            PhasePerceptProgram::And(_, _) | PhasePerceptProgram::Xor(_, _) => {
                let effects = Self::atom_effects(w);
                effects[0] <= 0.25 && effects[1] <= 0.25
            }
        }
    }

    pub fn phase_native_compositional_action(&mut self, goal_sensory: &[f32])
        -> (bool, Option<usize>)
    {
        let Some(real) = self.current_real.as_ref() else { return (false, None); };
        let sensory = real.sensory.clone();
        let Some(base) = self.phase_native_abstract_state(&sensory)
            else { return (false, None); };
        let Some(goal) = self.phase_native_abstract_state(goal_sensory)
            else { return (false, None); };
        let Some(raw) = self.phase_raw_features(&sensory) else { return (false, None); };
        let Some(native) = self.phase_native.as_ref() else { return (false, None); };
        let Some(comp) = native.compositional.as_ref() else { return (false, None); };

        let active_candidates = comp.candidates.iter()
            .filter(|w| w.base_cell == base.cell && !w.retired)
            .collect::<Vec<_>>();
        if active_candidates.is_empty() { return (false, None); }

        if native.config.learning_enabled {
            if let Some(w) = active_candidates.iter().find(|w| !w.promoted) {
                return (true, Some(w.anchor_action));
            }
        }

        let applicable = active_candidates.iter().filter_map(|w| {
            if !w.promoted { return None; }
            let side = usize::from(w.program.eval(&raw));
            let floor = native.config.coherence_floor;
            let base_link = &self.synapses[w.input_synapses[side][0]];
            let program_link = &self.synapses[w.input_synapses[side][1]];
            let base_current = if base_link.from == base.cell {
                conductance(&self.cells, base_link, floor)
            } else { 0.0 };
            let program_current = if program_link.from == w.program_cells[side] {
                conductance(&self.cells, program_link, floor)
            } else { 0.0 };
            if base_current.min(program_current) > 1.0e-8 {
                Some(w.state_cells[side])
            } else {
                None
            }
        }).collect::<Vec<_>>();

        if applicable.len() != 1 {
            return (true, None);
        }
        let entry = applicable[0];

        if native.config.learning_enabled {
            if let Some(action) = (0..self.config.motor_cells)
                .find(|a| !self.phase_drive_action_known_at(native, entry, *a))
            {
                return (true, Some(action));
            }
        }

        (true, self.phase_native_goal_decision_from_cells(entry, goal.cell, None)
            .map(|d| d.first_action))
    }

    pub fn observe_phase_native_compositional_result(
        &mut self,
        action: usize,
        post: &[f32],
    ) -> Option<usize> {
        if action >= self.config.motor_cells { return None; }
        let pre_sensory = self.current_real.as_ref()?.sensory.clone();
        let pre = self.phase_native_abstract_state(&pre_sensory)?;
        let after = self.phase_native_abstract_state(post)?;
        let raw = self.phase_raw_features(&pre_sensory)?;

        let mut native = self.phase_native.take()?;
        let Some(mut comp) = native.compositional.take() else {
            self.phase_native = Some(native);
            return self.observe_phase_native_perceptual_result(action, post);
        };
        let learning = native.config.learning_enabled;

        if learning {
            comp.factual_events = comp.factual_events.saturating_add(1);

            let no_active_base_candidates = !comp.candidates.iter()
                .any(|w| w.base_cell == pre.cell && !w.retired);
            if no_active_base_candidates && self.config.structural_growth_enabled {
                let proposed = Self::composition_untried_hypotheses(
                    &comp, pre.cell, action, after.cell, &raw
                );
                for (program, old_post) in proposed {
                        let free = self.dormant_range()
                            .filter(|i| !self.cells[*i].recruited)
                            .take(4).collect::<Vec<_>>();
                        if free.len() != 4 { break; }
                        for &cell in &free { self.cells[cell].recruited = true; }
                        let program_cells = [free[0], free[1]];
                        let states = [free[2], free[3]];
                        let inputs = [
                            [self.native_synapse(pre.cell, states[0]),
                             self.native_synapse(program_cells[0], states[0])],
                            [self.native_synapse(pre.cell, states[1]),
                             self.native_synapse(program_cells[1], states[1])],
                        ];
                        comp.candidates.push(PhaseCompositionWitness {
                            base_cell: pre.cell,
                            program,
                            program_cells,
                            successor_cells: [old_post, after.cell],
                            state_cells: states,
                            input_synapses: inputs,
                            anchor_action: action,
                            promoted: false,
                            retired: false,
                            born_fact: comp.factual_events,
                            eligible_observations: 0,
                            log_evidence: 0.0,
                            side_switches: 0,
                            atom_effects: [0.0; 2],
                            last_side: None,
                            gate_observations: [0; 2],
                            atom_counts: [[[0; 2]; 2]; 2],
                        });
                    }
            } else {
                for index in 0..comp.candidates.len() {
                    if comp.candidates[index].base_cell != pre.cell
                        || comp.candidates[index].retired
                    {
                        continue;
                    }
                    let snapshot = comp.candidates[index].clone();
                    let side = usize::from(snapshot.program.eval(&raw));
                    for synapse in snapshot.input_synapses[side] {
                        self.learn_phase_concept_running_mean_synapse(
                            synapse, 1.0, snapshot.gate_observations[side]);
                    }
                    comp.candidates[index].gate_observations[side] =
                        snapshot.gate_observations[side].saturating_add(1);
                    let accepted = self.native_cell_observation(
                        &mut native, snapshot.state_cells[side],
                        action, after.cell, 0.0
                    ).is_some();
                    if !accepted {
                        comp.candidates[index].retired = true;
                        continue;
                    }

                    if action == snapshot.anchor_action && !snapshot.promoted {
                        let outcome = snapshot.successor_cells.iter()
                            .position(|&cell| cell == after.cell);
                        let Some(outcome) = outcome else {
                            comp.candidates[index].retired = true;
                            continue;
                        };

                        let atoms = snapshot.program.atoms();
                        for atom_index in 0..2 {
                            let present = usize::from(raw.binary_search(&atoms[atom_index]).is_ok());
                            comp.candidates[index].atom_counts[atom_index][present][outcome] += 1;
                        }

                        let counts = self.composition_counts(&native, &snapshot);
                        let w = &mut comp.candidates[index];
                        w.eligible_observations = w.eligible_observations.saturating_add(1);
                        if w.last_side.map(|last| last != side).unwrap_or(false) {
                            w.side_switches = w.side_switches.saturating_add(1);
                        }
                        w.last_side = Some(side);
                        w.log_evidence = context_log_evidence(counts);
                        w.atom_effects = Self::atom_effects(w);
                        w.promoted = Self::composition_gate(w, counts);
                        if !w.promoted && w.eligible_observations >= CONTEXT_MAX_SAMPLES {
                            w.retired = true;
                        }
                    }
                }
            }

            if comp.discovery.len() == COMPOSITION_DISCOVERY_CAP {
                comp.discovery.remove(0);
            }
            comp.discovery.push(CompositionFact {
                base: pre.cell, action, post: after.cell, features: raw,
            });
        }

        let preserve_rivals = comp.candidates.iter()
            .any(|w| w.base_cell == pre.cell && !w.retired);
        native.compositional = Some(comp);
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
