// G10-PHYS: physical acquisition/execution of composite concepts.
// Included by phase_native.rs, so all executable concept state lives in the
// SAME EvoPhase cell/synapse arrays as P1-P5.

#[derive(Debug, Clone)]
struct PhaseConceptAtomBinding {
    atom_id: u64,
    cell: usize,
    motor_synapses: Vec<usize>,
    action_support: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct PhaseConceptCircuitInfo {
    pub concept_id: u64,
    pub child_ids: [u64; 2],
    pub child_cells: [usize; 2],
    pub concept_cell: usize,
    pub child_synapses: [usize; 2],
    pub motor_synapses: Vec<usize>,
    pub support: u32,
    pub promoted: bool,
    action_support: Vec<u32>,
}

impl PhaseConceptCircuitInfo {
    pub(super) fn contains_synapse(&self, index: usize) -> bool {
        self.child_synapses.contains(&index) || self.motor_synapses.contains(&index)
    }
}

#[derive(Debug, Clone)]
pub(super) struct PhaseConceptState {
    atoms: Vec<PhaseConceptAtomBinding>,
    circuits: Vec<PhaseConceptCircuitInfo>,
    next_concept_id: u64,
}

impl PhaseConceptState {
    fn new() -> Self {
        Self {
            atoms: Vec::new(),
            circuits: Vec::new(),
            next_concept_id: 1,
        }
    }
}

impl EvoPhase {
    pub fn enable_phase_native_concepts(&mut self) -> bool {
        if self.concept_memory.is_none() {
            return false;
        }
        let Some(state) = self.phase_native.as_mut() else {
            return false;
        };
        if state.concepts.is_some() {
            return false;
        }
        state.concepts = Some(PhaseConceptState::new());
        true
    }

    pub fn phase_native_concept_circuits(&self) -> &[PhaseConceptCircuitInfo] {
        self.phase_native
            .as_ref()
            .and_then(|state| state.concepts.as_ref())
            .map(|concepts| concepts.circuits.as_slice())
            .unwrap_or(&[])
    }

    pub fn phase_native_concept_atom_count(&self) -> usize {
        self.phase_native
            .as_ref()
            .and_then(|state| state.concepts.as_ref())
            .map(|concepts| concepts.atoms.len())
            .unwrap_or(0)
    }

    pub fn phase_native_promoted_concept_count(&self) -> usize {
        self.phase_native_concept_circuits()
            .iter()
            .filter(|circuit| circuit.promoted)
            .count()
    }

    pub fn phase_native_concept_synapses(&self) -> Vec<usize> {
        let Some(physical) = self
            .phase_native
            .as_ref()
            .and_then(|state| state.concepts.as_ref())
        else {
            return Vec::new();
        };

        physical
            .atoms
            .iter()
            .flat_map(|atom| atom.motor_synapses.iter().copied())
            .chain(physical.circuits.iter().flat_map(|circuit| {
                circuit
                    .child_synapses
                    .iter()
                    .copied()
                    .chain(circuit.motor_synapses.iter().copied())
            }))
            .collect()
    }

    pub(super) fn is_native_concept_synapse(&self, index: usize) -> bool {
        let Some(physical) = self
            .phase_native
            .as_ref()
            .and_then(|state| state.concepts.as_ref())
        else {
            return false;
        };
        physical
            .atoms
            .iter()
            .any(|atom| atom.motor_synapses.contains(&index))
            || physical
                .circuits
                .iter()
                .any(|circuit| circuit.contains_synapse(index))
    }

    /// Physical factual learning path.
    ///
    /// The generic concept memory is used only to acquire/recognize lower-level
    /// relation atom IDs. Pair evidence, promotion state and motor meaning are
    /// learned in actual carrier synapses below.
    pub fn observe_phase_native_concept_factual(
        &mut self,
        sensory: &[f32],
        action: usize,
        need: bool,
    ) -> bool {
        assert!(action < self.config.motor_cells);
        if self
            .phase_native
            .as_ref()
            .and_then(|state| state.concepts.as_ref())
            .is_none()
        {
            return false;
        }

        let active_ids = self
            .concept_memory
            .as_mut()
            .expect("enable concept memory first")
            .observe_atoms(sensory);

        if active_ids.len() < 2 {
            return false;
        }

        let (min_composite_support, child_ceiling, promotion_threshold) = self
            .concept_memory
            .as_ref()
            .expect("concept memory")
            .physical_promotion_thresholds();
        let min_action_support = self
            .concept_memory
            .as_ref()
            .expect("concept memory")
            .min_action_support();

        let mut state = match self.phase_native.take() {
            Some(state) => state,
            None => return false,
        };
        let Some(mut physical) = state.concepts.take() else {
            self.phase_native = Some(state);
            return false;
        };

        if !state.config.learning_enabled {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return false;
        }

        let missing_atoms = active_ids
            .iter()
            .filter(|id| !physical.atoms.iter().any(|atom| atom.atom_id == **id))
            .count();

        let mut missing_pairs = 0usize;
        for i in 0..active_ids.len() {
            for j in (i + 1)..active_ids.len() {
                let pair = ordered_concept_pair(active_ids[i], active_ids[j]);
                if !physical
                    .circuits
                    .iter()
                    .any(|circuit| circuit.child_ids == pair)
                {
                    missing_pairs += 1;
                }
            }
        }

        let required_cells = missing_atoms + missing_pairs;
        let free_cells = self
            .dormant_range()
            .filter(|index| !self.cells[*index].recruited)
            .count();

        if required_cells > 0
            && (!self.config.structural_growth_enabled || free_cells < required_cells)
        {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return false;
        }

        for atom_id in &active_ids {
            if physical.atoms.iter().any(|atom| atom.atom_id == *atom_id) {
                continue;
            }
            let Some(cell) = self
                .dormant_range()
                .find(|index| !self.cells[*index].recruited)
            else {
                state.concepts = Some(physical);
                self.phase_native = Some(state);
                return false;
            };
            self.cells[cell].recruited = true;
            let motor_synapses = (0..self.config.motor_cells)
                .map(|motor| self.native_synapse(cell, self.motor_cell(motor)))
                .collect::<Vec<_>>();
            physical.atoms.push(PhaseConceptAtomBinding {
                atom_id: *atom_id,
                cell,
                motor_synapses,
                action_support: vec![0; self.config.motor_cells],
            });
        }
        physical.atoms.sort_by_key(|atom| atom.atom_id);

        // Individual-child factual evidence is stored in physical atom->motor
        // weights. Balanced XOR-like experience should drive it toward 0.5.
        for atom_id in &active_ids {
            let atom_index = physical
                .atoms
                .iter()
                .position(|atom| atom.atom_id == *atom_id)
                .expect("active atom binding");
            let support_before = physical.atoms[atom_index].action_support[action];
            let synapse = physical.atoms[atom_index].motor_synapses[action];
            self.learn_phase_concept_running_mean_synapse(
                synapse,
                if need { 1.0 } else { 0.0 },
                support_before,
            );
            physical.atoms[atom_index].action_support[action] =
                support_before.saturating_add(1);
        }

        for i in 0..active_ids.len() {
            for j in (i + 1)..active_ids.len() {
                let child_ids = ordered_concept_pair(active_ids[i], active_ids[j]);
                let circuit_index = if let Some(index) = physical
                    .circuits
                    .iter()
                    .position(|circuit| circuit.child_ids == child_ids)
                {
                    index
                } else {
                    let child_cells = child_ids.map(|child_id| {
                        physical
                            .atoms
                            .iter()
                            .find(|atom| atom.atom_id == child_id)
                            .expect("physical atom before composite")
                            .cell
                    });
                    let Some(concept_cell) = self
                        .dormant_range()
                        .find(|index| !self.cells[*index].recruited)
                    else {
                        state.concepts = Some(physical);
                        self.phase_native = Some(state);
                        return false;
                    };
                    self.cells[concept_cell].recruited = true;
                    let child_synapses = [
                        self.native_synapse(child_cells[0], concept_cell),
                        self.native_synapse(child_cells[1], concept_cell),
                    ];
                    let motor_synapses = (0..self.config.motor_cells)
                        .map(|motor| {
                            self.native_synapse(concept_cell, self.motor_cell(motor))
                        })
                        .collect::<Vec<_>>();
                    let concept_id = physical.next_concept_id;
                    physical.next_concept_id = physical.next_concept_id.saturating_add(1);
                    physical.circuits.push(PhaseConceptCircuitInfo {
                        concept_id,
                        child_ids,
                        child_cells,
                        concept_cell,
                        child_synapses,
                        motor_synapses,
                        support: 0,
                        promoted: false,
                        action_support: vec![0; self.config.motor_cells],
                    });
                    physical.circuits.len() - 1
                };

                let support_before = physical.circuits[circuit_index].support;
                for child_synapse in physical.circuits[circuit_index].child_synapses {
                    self.learn_phase_concept_running_mean_synapse(
                        child_synapse,
                        1.0,
                        support_before,
                    );
                }

                let action_support_before =
                    physical.circuits[circuit_index].action_support[action];
                let motor_synapse =
                    physical.circuits[circuit_index].motor_synapses[action];
                self.learn_phase_concept_running_mean_synapse(
                    motor_synapse,
                    if need { 1.0 } else { 0.0 },
                    action_support_before,
                );
                physical.circuits[circuit_index].action_support[action] =
                    action_support_before.saturating_add(1);
                physical.circuits[circuit_index].support =
                    support_before.saturating_add(1);

                if !physical.circuits[circuit_index].promoted
                    && physical.circuits[circuit_index].support >= min_composite_support
                {
                    let mut winning: Option<(usize, f32)> = None;
                    for motor in 0..self.config.motor_cells {
                        if physical.circuits[circuit_index].action_support[motor]
                            < min_action_support
                        {
                            continue;
                        }
                        let weight = self.synapses
                            [physical.circuits[circuit_index].motor_synapses[motor]]
                            .weight;
                        let centered = 2.0 * weight - 1.0;
                        if centered <= 0.0 {
                            continue;
                        }
                        if winning
                            .map(|(_, best)| centered > best)
                            .unwrap_or(true)
                        {
                            winning = Some((motor, centered));
                        }
                    }

                    if let Some((winning_motor, evidence)) = winning {
                        if evidence >= promotion_threshold {
                            let children_weak = physical.circuits[circuit_index]
                                .child_ids
                                .iter()
                                .all(|child_id| {
                                    let atom = physical
                                        .atoms
                                        .iter()
                                        .find(|atom| atom.atom_id == *child_id)
                                        .expect("physical child atom");
                                    if atom.action_support[winning_motor] < min_action_support {
                                        return false;
                                    }
                                    let weight =
                                        self.synapses[atom.motor_synapses[winning_motor]].weight;
                                    (2.0 * weight - 1.0).abs() <= child_ceiling
                                });
                            if children_weak {
                                physical.circuits[circuit_index].promoted = true;
                            }
                        }
                    }
                }
            }
        }

        physical.circuits.sort_by_key(|circuit| circuit.concept_id);
        state.concepts = Some(physical);
        self.phase_native = Some(state);
        true
    }

    fn learn_phase_concept_running_mean_synapse(
        &mut self,
        index: usize,
        target: f32,
        support_before: u32,
    ) {
        let from = self.synapses[index].from;
        let to = self.synapses[index].to;
        let target_phase = wrap_phase(self.cells[to].phase - self.cells[from].phase);
        let synapse = &mut self.synapses[index];
        if !synapse.plastic {
            return;
        }

        let divisor = support_before.saturating_add(1) as f32;
        let value_step = self.config.weight_learning_rate / divisor;
        synapse.eligibility = 1.0;
        synapse.weight += value_step * (target.clamp(0.0, 1.0) - synapse.weight);

        synapse.phase_offset = wrap_phase(
            synapse.phase_offset
                + self.config.phase_learning_rate
                    * signed_phase_error(target_phase, synapse.phase_offset),
        );
        let confidence_target =
            divisor / (divisor + 1.0);
        synapse.confidence += self.config.weight_learning_rate
            * (confidence_target - synapse.confidence);
    }

    /// Physical G10 readout. Dedicated concept-memory pair statistics and
    /// action selectors are not consulted here.
    pub fn choose_phase_native_concept_action(&self, sensory: &[f32]) -> Option<usize> {
        let active_ids = self.concept_memory.as_ref()?.active_atom_ids(sensory);
        if active_ids.len() < 2 {
            return None;
        }

        let state = self.phase_native.as_ref()?;
        let physical = state.concepts.as_ref()?;
        let floor = state.config.coherence_floor;

        let mut activity = self.cells.clone();
        for cell in &mut activity {
            cell.charge = 0.0;
        }
        for atom in &physical.atoms {
            if active_ids.binary_search(&atom.atom_id).is_ok() {
                activity[atom.cell].charge = 1.0;
            }
        }

        let mut motor_potentials = vec![0.0f32; self.config.motor_cells];

        for circuit in &physical.circuits {
            if !circuit.promoted
                || !circuit
                    .child_ids
                    .iter()
                    .all(|id| active_ids.binary_search(id).is_ok())
            {
                continue;
            }

            let left = activity[circuit.child_cells[0]].charge
                * conductance(
                    &self.cells,
                    &self.synapses[circuit.child_synapses[0]],
                    floor,
                );
            let right = activity[circuit.child_cells[1]].charge
                * conductance(
                    &self.cells,
                    &self.synapses[circuit.child_synapses[1]],
                    floor,
                );
            let concept_current = left.min(right);
            if concept_current <= 1.0e-8 {
                continue;
            }
            activity[circuit.concept_cell].charge = concept_current;

            for (action, index) in circuit.motor_synapses.iter().copied().enumerate() {
                let centered = 2.0 * self.synapses[index].weight - 1.0;
                if centered <= 0.0 {
                    continue;
                }
                let current = concept_current
                    * centered
                    * coherence(&self.cells, &self.synapses[index], floor);
                if current > motor_potentials[action] {
                    motor_potentials[action] = current;
                }
            }
        }

        motor_potentials
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, value)| *value > 1.0e-8)
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap()
                    .then_with(|| b.0.cmp(&a.0))
            })
            .map(|(action, _)| action)
    }
}

fn ordered_concept_pair(a: u64, b: u64) -> [u64; 2] {
    if a <= b { [a, b] } else { [b, a] }
}
