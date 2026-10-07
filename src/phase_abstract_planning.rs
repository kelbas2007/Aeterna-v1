// G15: bridge acquired physical abstractions into the existing P1
// phase-native transition/value recurrence. No separate graph model.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseAbstractStateRef {
    pub level: u8,
    pub id: u64,
    pub cell: usize,
}

impl EvoPhase {
    /// Resolve the unique highest active acquired abstraction for raw sensory.
    /// This performs recognition only; it does not map state to action/reward.
    pub fn phase_native_abstract_state(
        &self,
        sensory: &[f32],
    ) -> Option<PhaseAbstractStateRef> {
        let state = self.phase_native.as_ref()?;
        let concepts = state.concepts.as_ref()?;

        if let Some(deep) = state.deep.as_ref() {
            let active = self.active_deep_refs(sensory, concepts, deep);
            let highest = active.iter().map(|child| child.level).max()?;
            let mut at_highest = active
                .into_iter()
                .filter(|child| child.level == highest);
            let selected = at_highest.next()?;
            if at_highest.next().is_some() {
                return None;
            }
            return Some(PhaseAbstractStateRef {
                level: selected.level,
                id: selected.id,
                cell: selected.cell,
            });
        }

        let memory = self.concept_memory.as_ref()?;
        let active_atoms = memory.active_atom_ids(sensory);
        let mut active_l1 = concepts
            .circuits
            .iter()
            .filter(|circuit| {
                circuit.promoted
                    && circuit
                        .child_ids
                        .iter()
                        .all(|id| active_atoms.binary_search(id).is_ok())
            });
        let selected = active_l1.next()?;
        if active_l1.next().is_some() {
            return None;
        }
        Some(PhaseAbstractStateRef {
            level: 1,
            id: selected.concept_id,
            cell: selected.concept_cell,
        })
    }

    /// Learn one factual transition directly between already-acquired physical
    /// abstraction cells using the SAME P1 circuit and synapse arrays.
    pub fn observe_phase_native_abstract_transition(
        &mut self,
        pre_sensory: &[f32],
        action: usize,
        post_sensory: &[f32],
        value: f32,
    ) -> bool {
        assert!(action < self.config.motor_cells);
        assert!(value.is_finite() && (0.0..=1.0).contains(&value));

        let Some(pre) = self.phase_native_abstract_state(pre_sensory) else {
            return false;
        };
        let Some(post) = self.phase_native_abstract_state(post_sensory) else {
            return false;
        };

        let mut state = match self.phase_native.take() {
            Some(state) => state,
            None => return false,
        };
        if !state.config.learning_enabled {
            self.phase_native = Some(state);
            return false;
        }

        let accepted = self
            .native_cell_observation(
                &mut state,
                pre.cell,
                action,
                post.cell,
                value,
            )
            .is_some();
        self.phase_native = Some(state);
        accepted
    }

    /// Plan from the unique highest active acquired abstraction using P1's
    /// physical backward value recurrence. The optional bound is diagnostic;
    /// None uses the native configured horizon.
    pub fn plan_phase_native_abstract(
        &mut self,
        sensory: &[f32],
        depth: Option<usize>,
    ) -> Option<PlanDecision> {
        let state = self.phase_native_abstract_state(sensory)?;
        self.phase_native_decision_from_cell(state.cell, depth)
    }
}
