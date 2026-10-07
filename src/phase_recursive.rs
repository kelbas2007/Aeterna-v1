// G11: recursive phase-native abstraction.
// Included by phase_native.rs after phase_concept.rs. Level-2 task state lives
// in the same EvoPhase physical cell/synapse arrays.

#[derive(Debug, Clone)]
pub struct PhaseRecursiveConceptInfo {
    pub concept_id: u64,
    pub child_concept_ids: [u64; 2],
    pub child_cells: [usize; 2],
    pub concept_cell: usize,
    pub child_synapses: [usize; 2],
    pub motor_synapses: Vec<usize>,
    pub support: u32,
    pub promoted: bool,
    action_support: Vec<u32>,
}

impl PhaseRecursiveConceptInfo {
    pub(super) fn contains_synapse(&self, index: usize) -> bool {
        self.child_synapses.contains(&index) || self.motor_synapses.contains(&index)
    }
}

impl EvoPhase {
    pub fn enable_phase_native_recursive_concepts(&mut self) -> bool {
        let Some(state) = self.phase_native.as_mut() else {
            return false;
        };
        let Some(concepts) = state.concepts.as_mut() else {
            return false;
        };
        concepts.recursive_formation_enabled = true;
        concepts.recursive_readout_enabled = true;
        true
    }

    pub fn set_phase_native_recursive_formation_enabled(&mut self, enabled: bool) {
        if let Some(concepts) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.concepts.as_mut())
        {
            concepts.recursive_formation_enabled = enabled;
        }
    }

    pub fn set_phase_native_recursive_readout_enabled(&mut self, enabled: bool) {
        if let Some(concepts) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.concepts.as_mut())
        {
            concepts.recursive_readout_enabled = enabled;
        }
    }

    pub fn phase_native_recursive_circuits(&self) -> &[PhaseRecursiveConceptInfo] {
        self.phase_native
            .as_ref()
            .and_then(|state| state.concepts.as_ref())
            .map(|concepts| concepts.recursive.as_slice())
            .unwrap_or(&[])
    }

    pub fn phase_native_promoted_recursive_count(&self) -> usize {
        self.phase_native_recursive_circuits()
            .iter()
            .filter(|circuit| circuit.promoted)
            .count()
    }

    pub fn phase_native_recursive_synapses(&self) -> Vec<usize> {
        self.phase_native_recursive_circuits()
            .iter()
            .flat_map(|circuit| {
                circuit
                    .child_synapses
                    .iter()
                    .copied()
                    .chain(circuit.motor_synapses.iter().copied())
            })
            .collect()
    }

    pub(super) fn is_native_recursive_synapse(&self, index: usize) -> bool {
        self.phase_native_recursive_circuits()
            .iter()
            .any(|circuit| circuit.contains_synapse(index))
    }

    fn active_level1_concepts(
        &self,
        sensory: &[f32],
        physical: &PhaseConceptState,
    ) -> Vec<usize> {
        let Some(memory) = self.concept_memory.as_ref() else {
            return Vec::new();
        };
        let active_atom_ids = memory.active_atom_ids(sensory);
        if active_atom_ids.len() < 2 {
            return Vec::new();
        }

        physical
            .circuits
            .iter()
            .enumerate()
            .filter_map(|(index, circuit)| {
                (circuit.promoted
                    && circuit
                        .child_ids
                        .iter()
                        .all(|id| active_atom_ids.binary_search(id).is_ok()))
                .then_some(index)
            })
            .collect()
    }

    /// G11 factual path. Existing promoted level-1 physical concepts are the
    /// only eligible children. Their individual top-action evidence and their
    /// pair evidence are both learned in actual physical synapses.
    pub fn observe_phase_native_recursive_concept_factual(
        &mut self,
        sensory: &[f32],
        action: usize,
        need: bool,
    ) -> bool {
        assert!(action < self.config.motor_cells);

        let (min_composite_support, child_ceiling, promotion_threshold) = match self
            .concept_memory
            .as_ref()
        {
            Some(memory) => memory.physical_promotion_thresholds(),
            None => return false,
        };
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

        let active_level1 = self.active_level1_concepts(sensory, &physical);
        if active_level1.len() < 2 {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return false;
        }

        // Each acquired level-1 concept remains a reusable physical feature.
        // Its existing concept->motor synapses accumulate evidence for the
        // new top-level motor channels.
        for &index in &active_level1 {
            let support_before = physical.circuits[index].action_support[action];
            let synapse = physical.circuits[index].motor_synapses[action];
            self.learn_phase_concept_running_mean_synapse(
                synapse,
                if need { 1.0 } else { 0.0 },
                support_before,
            );
            physical.circuits[index].action_support[action] =
                support_before.saturating_add(1);
        }

        if !physical.recursive_formation_enabled {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return true;
        }

        let mut missing = 0usize;
        for i in 0..active_level1.len() {
            for j in (i + 1)..active_level1.len() {
                let ids = ordered_recursive_pair(
                    physical.circuits[active_level1[i]].concept_id,
                    physical.circuits[active_level1[j]].concept_id,
                );
                if !physical
                    .recursive
                    .iter()
                    .any(|circuit| circuit.child_concept_ids == ids)
                {
                    missing += 1;
                }
            }
        }

        let free_cells = self
            .dormant_range()
            .filter(|index| !self.cells[*index].recruited)
            .count();
        if missing > 0
            && (!self.config.structural_growth_enabled || free_cells < missing)
        {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return false;
        }

        for i in 0..active_level1.len() {
            for j in (i + 1)..active_level1.len() {
                let left = active_level1[i];
                let right = active_level1[j];
                let child_concept_ids = ordered_recursive_pair(
                    physical.circuits[left].concept_id,
                    physical.circuits[right].concept_id,
                );

                let recursive_index = if let Some(index) = physical
                    .recursive
                    .iter()
                    .position(|circuit| circuit.child_concept_ids == child_concept_ids)
                {
                    index
                } else {
                    let child_cells = child_concept_ids.map(|concept_id| {
                        physical
                            .circuits
                            .iter()
                            .find(|circuit| {
                                circuit.promoted && circuit.concept_id == concept_id
                            })
                            .expect("recursive child must be promoted level-1 concept")
                            .concept_cell
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

                    let concept_id = physical.next_recursive_id;
                    physical.next_recursive_id =
                        physical.next_recursive_id.saturating_add(1);
                    physical.recursive.push(PhaseRecursiveConceptInfo {
                        concept_id,
                        child_concept_ids,
                        child_cells,
                        concept_cell,
                        child_synapses,
                        motor_synapses,
                        support: 0,
                        promoted: false,
                        action_support: vec![0; self.config.motor_cells],
                    });
                    physical.recursive.len() - 1
                };

                let support_before = physical.recursive[recursive_index].support;
                for synapse in physical.recursive[recursive_index].child_synapses {
                    self.learn_phase_concept_running_mean_synapse(
                        synapse,
                        1.0,
                        support_before,
                    );
                }

                let action_support_before =
                    physical.recursive[recursive_index].action_support[action];
                let motor_synapse =
                    physical.recursive[recursive_index].motor_synapses[action];
                self.learn_phase_concept_running_mean_synapse(
                    motor_synapse,
                    if need { 1.0 } else { 0.0 },
                    action_support_before,
                );
                physical.recursive[recursive_index].action_support[action] =
                    action_support_before.saturating_add(1);
                physical.recursive[recursive_index].support =
                    support_before.saturating_add(1);

                if !physical.recursive[recursive_index].promoted
                    && physical.recursive[recursive_index].support
                        >= min_composite_support
                {
                    let mut winning: Option<(usize, f32)> = None;
                    for motor in 0..self.config.motor_cells {
                        if physical.recursive[recursive_index].action_support[motor]
                            < min_action_support
                        {
                            continue;
                        }
                        let weight = self.synapses
                            [physical.recursive[recursive_index].motor_synapses[motor]]
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
                            let children_weak = physical.recursive[recursive_index]
                                .child_concept_ids
                                .iter()
                                .all(|concept_id| {
                                    let child = physical
                                        .circuits
                                        .iter()
                                        .find(|circuit| {
                                            circuit.promoted
                                                && circuit.concept_id == *concept_id
                                        })
                                        .expect("recursive child concept");
                                    if child.action_support[winning_motor]
                                        < min_action_support
                                    {
                                        return false;
                                    }
                                    let weight =
                                        self.synapses[child.motor_synapses[winning_motor]]
                                            .weight;
                                    (2.0 * weight - 1.0).abs()
                                        <= child_ceiling
                                });
                            if children_weak {
                                physical.recursive[recursive_index].promoted = true;
                            }
                        }
                    }
                }
            }
        }

        physical
            .recursive
            .sort_by_key(|circuit| circuit.concept_id);
        state.concepts = Some(physical);
        self.phase_native = Some(state);
        true
    }

    /// Physical recursive readout:
    /// raw atoms -> promoted L1 concept currents -> promoted L2 current -> motor.
    pub fn choose_phase_native_recursive_concept_action(
        &self,
        sensory: &[f32],
    ) -> Option<usize> {
        let memory = self.concept_memory.as_ref()?;
        let active_atom_ids = memory.active_atom_ids(sensory);
        if active_atom_ids.len() < 4 {
            return None;
        }

        let state = self.phase_native.as_ref()?;
        let physical = state.concepts.as_ref()?;
        if !physical.recursive_readout_enabled {
            return None;
        }
        let floor = state.config.coherence_floor;

        let mut activity = self.cells.clone();
        for cell in &mut activity {
            cell.charge = 0.0;
        }
        for atom in &physical.atoms {
            if active_atom_ids.binary_search(&atom.atom_id).is_ok() {
                activity[atom.cell].charge = 1.0;
            }
        }

        // First physical abstraction layer.
        for circuit in &physical.circuits {
            if !circuit.promoted
                || !circuit
                    .child_ids
                    .iter()
                    .all(|id| active_atom_ids.binary_search(id).is_ok())
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
            let current = left.min(right);
            if current > activity[circuit.concept_cell].charge {
                activity[circuit.concept_cell].charge = current;
            }
        }

        let mut motor_potentials = vec![0.0f32; self.config.motor_cells];

        // Second physical abstraction layer.
        for circuit in &physical.recursive {
            if !circuit.promoted {
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
            let current = left.min(right);
            if current <= 1.0e-8 {
                continue;
            }
            activity[circuit.concept_cell].charge = current;

            for (motor, index) in circuit.motor_synapses.iter().copied().enumerate() {
                let centered = 2.0 * self.synapses[index].weight - 1.0;
                if centered <= 0.0 {
                    continue;
                }
                let output = current
                    * centered
                    * coherence(&self.cells, &self.synapses[index], floor);
                if output > motor_potentials[motor] {
                    motor_potentials[motor] = output;
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
            .map(|(motor, _)| motor)
    }
}

fn ordered_recursive_pair(a: u64, b: u64) -> [u64; 2] {
    if a <= b { [a, b] } else { [b, a] }
}
