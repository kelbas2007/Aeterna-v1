// G10-PHYS: physical execution of acquired composite concepts.
// Included by phase_native.rs, so it operates on the SAME EvoPhase cells/synapses.

#[derive(Debug, Clone)]
struct PhaseConceptAtomBinding {
    atom_id: u64,
    cell: usize,
}

#[derive(Debug, Clone)]
pub struct PhaseConceptCircuitInfo {
    pub concept_id: u64,
    pub child_ids: [u64; 2],
    pub child_cells: [usize; 2],
    pub concept_cell: usize,
    pub child_synapses: [usize; 2],
    pub motor_synapses: Vec<usize>,
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
}

impl PhaseConceptState {
    fn new() -> Self {
        Self {
            atoms: Vec::new(),
            circuits: Vec::new(),
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

    pub fn phase_native_concept_synapses(&self) -> Vec<usize> {
        self.phase_native_concept_circuits()
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

    pub(super) fn is_native_concept_synapse(&self, index: usize) -> bool {
        self.phase_native_concept_circuits()
            .iter()
            .any(|circuit| circuit.contains_synapse(index))
    }

    /// Factual G10-PHYS learning path. The dedicated concept memory receives
    /// raw factual evidence first, then its newly acquired structural IDs are
    /// materialized into physical carrier cells/synapses.
    pub fn observe_phase_native_concept_factual(
        &mut self,
        sensory: &[f32],
        action: usize,
        need: bool,
    ) -> bool {
        if self.phase_native.is_none()
            || self
                .phase_native
                .as_ref()
                .and_then(|state| state.concepts.as_ref())
                .is_none()
        {
            return false;
        }

        self.concept_memory
            .as_mut()
            .expect("enable concept memory first")
            .observe_factual(sensory, action, need);

        self.sync_phase_native_concepts()
    }

    fn sync_phase_native_concepts(&mut self) -> bool {
        let atom_ids = self
            .concept_memory
            .as_ref()
            .expect("concept memory")
            .atoms()
            .iter()
            .map(|atom| atom.id)
            .collect::<Vec<_>>();
        let composites = self
            .concept_memory
            .as_ref()
            .expect("concept memory")
            .composites()
            .to_vec();

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
            return true;
        }

        let missing_atoms = atom_ids
            .iter()
            .filter(|id| !physical.atoms.iter().any(|binding| binding.atom_id == **id))
            .count();
        let missing_composites = composites
            .iter()
            .filter(|concept| {
                !physical
                    .circuits
                    .iter()
                    .any(|circuit| circuit.concept_id == concept.id)
            })
            .count();
        let required_cells = missing_atoms + missing_composites;
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

        for atom_id in atom_ids {
            if physical.atoms.iter().any(|binding| binding.atom_id == atom_id) {
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
            physical.atoms.push(PhaseConceptAtomBinding { atom_id, cell });
        }

        for concept in &composites {
            if physical
                .circuits
                .iter()
                .any(|circuit| circuit.concept_id == concept.id)
            {
                continue;
            }

            let child_cells = concept.child_ids.map(|child_id| {
                physical
                    .atoms
                    .iter()
                    .find(|binding| binding.atom_id == child_id)
                    .expect("promoted concept children must have physical atom cells")
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
                .map(|action| self.native_synapse(concept_cell, self.motor_cell(action)))
                .collect::<Vec<_>>();

            physical.circuits.push(PhaseConceptCircuitInfo {
                concept_id: concept.id,
                child_ids: concept.child_ids,
                child_cells,
                concept_cell,
                child_synapses,
                motor_synapses,
            });
        }

        // Factual structural statistics may live in the provenance layer, but
        // the executable state is calibrated into actual shared synapses.
        for circuit in &physical.circuits {
            for index in circuit.child_synapses {
                self.learn_phase_concept_synapse(index, 1.0);
            }
            for (action, index) in circuit.motor_synapses.iter().copied().enumerate() {
                let target = composites
                    .iter()
                    .find(|concept| concept.id == circuit.concept_id)
                    .and_then(|concept| {
                        concept
                            .outcomes
                            .get(action)
                            .map(|stat| {
                                stat.signed_value(
                                    self.concept_memory
                                        .as_ref()
                                        .expect("concept memory")
                                        .config_min_action_support(),
                                )
                            })
                    })
                    .unwrap_or(0.0)
                    .max(0.0);
                self.learn_phase_concept_synapse(index, target);
            }
        }

        physical.atoms.sort_by_key(|binding| binding.atom_id);
        physical.circuits.sort_by_key(|circuit| circuit.concept_id);
        state.concepts = Some(physical);
        self.phase_native = Some(state);
        true
    }

    fn learn_phase_concept_synapse(&mut self, index: usize, target_weight: f32) {
        let from = self.synapses[index].from;
        let to = self.synapses[index].to;
        let target_phase = wrap_phase(self.cells[to].phase - self.cells[from].phase);
        let synapse = &mut self.synapses[index];
        if !synapse.plastic {
            return;
        }
        synapse.eligibility = 1.0;
        synapse.weight +=
            self.config.weight_learning_rate * (target_weight.clamp(0.0, 1.0) - synapse.weight);
        synapse.phase_offset = wrap_phase(
            synapse.phase_offset
                + self.config.phase_learning_rate
                    * signed_phase_error(target_phase, synapse.phase_offset),
        );
        synapse.confidence +=
            self.config.weight_learning_rate * (1.0 - synapse.confidence);
    }

    /// Physical G10 readout. No dedicated concept-memory action selector is
    /// called here; only active acquired atom IDs, structural addresses and
    /// actual cell/synapse conductance are used.
    pub fn choose_phase_native_concept_action(&self, sensory: &[f32]) -> Option<usize> {
        let active_ids = self
            .concept_memory
            .as_ref()?
            .active_atom_ids(sensory);
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

        for binding in &physical.atoms {
            if active_ids.binary_search(&binding.atom_id).is_ok() {
                activity[binding.cell].charge = 1.0;
            }
        }

        let mut motor_potentials = vec![0.0f32; self.config.motor_cells];

        for circuit in &physical.circuits {
            if !circuit
                .child_ids
                .iter()
                .all(|id| active_ids.binary_search(id).is_ok())
            {
                continue;
            }

            let left = activity[circuit.child_cells[0]].charge
                * conductance(&self.cells, &self.synapses[circuit.child_synapses[0]], floor);
            let right = activity[circuit.child_cells[1]].charge
                * conductance(&self.cells, &self.synapses[circuit.child_synapses[1]], floor);
            let concept_current = left.min(right);
            if concept_current <= 1.0e-8 {
                continue;
            }
            activity[circuit.concept_cell].charge = concept_current;

            for (action, index) in circuit.motor_synapses.iter().copied().enumerate() {
                let current = concept_current
                    * conductance(&self.cells, &self.synapses[index], floor);
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
