// G12: self-triggered abstraction escalation.
// Included by phase_native.rs after the explicit G11 recursive mechanism.
// The FULL path receives only raw sensory + opaque action + factual Need and
// chooses its learning level from already-acquired physical evidence.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoAbstractionMetrics {
    pub observations: u64,
    pub recursive_candidates: usize,
    pub recursive_promoted: usize,
    pub first_weak_observation: Option<u64>,
    pub first_candidate_observation: Option<u64>,
    pub first_promotion_observation: Option<u64>,
}

impl EvoPhase {
    /// Generic once-per-organism capability switch. This does not select a
    /// task phase or abstraction level; later factual evidence does that.
    pub fn enable_phase_native_auto_abstraction(&mut self) -> bool {
        let Some(concepts) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.concepts.as_mut())
        else {
            return false;
        };
        concepts.auto_abstraction_enabled = true;
        concepts.auto_always_escalate = false;
        concepts.auto_child_revision_enabled = true;
        concepts.recursive_readout_enabled = true;
        true
    }

    pub fn set_phase_native_auto_abstraction_enabled(&mut self, enabled: bool) {
        if let Some(concepts) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.concepts.as_mut())
        {
            concepts.auto_abstraction_enabled = enabled;
        }
    }

    /// Diagnostic baseline only: bypass the child-sufficiency gate and recruit
    /// higher candidates whenever >=2 acquired L1 concepts are co-active.
    #[doc(hidden)]
    pub fn set_phase_native_auto_always_escalate_for_control(
        &mut self,
        enabled: bool,
    ) {
        if let Some(concepts) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.concepts.as_mut())
        {
            concepts.auto_always_escalate = enabled;
        }
    }

    /// Matched control: keep the previously learned child explanation fixed
    /// while still presenting the same later facts.
    #[doc(hidden)]
    pub fn set_phase_native_auto_child_revision_for_control(
        &mut self,
        enabled: bool,
    ) {
        if let Some(concepts) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.concepts.as_mut())
        {
            concepts.auto_child_revision_enabled = enabled;
        }
    }

    pub fn phase_native_auto_abstraction_metrics(
        &self,
    ) -> Option<AutoAbstractionMetrics> {
        let concepts = self.phase_native.as_ref()?.concepts.as_ref()?;
        Some(AutoAbstractionMetrics {
            observations: concepts.auto_observations,
            recursive_candidates: concepts.recursive.len(),
            recursive_promoted: concepts
                .recursive
                .iter()
                .filter(|circuit| circuit.promoted)
                .count(),
            first_weak_observation: concepts.auto_first_weak_observation,
            first_candidate_observation: concepts.auto_first_candidate_observation,
            first_promotion_observation: concepts.auto_first_promotion_observation,
        })
    }

    /// One generic factual concept-learning entry point.
    ///
    /// If the raw scene is not already explained by acquired L1 concepts, the
    /// fact is routed to the physical atom/L1 learner. If acquired L1 concepts
    /// are active, their physical action evidence is revised first. Higher
    /// pair structure is recruited only when the active child evidence for the
    /// current action is sufficiently supported yet weak.
    pub fn observe_phase_native_adaptive_concept_factual(
        &mut self,
        sensory: &[f32],
        action: usize,
        need: bool,
    ) -> bool {
        assert!(action < self.config.motor_cells);

        let active_level1 = {
            let Some(state) = self.phase_native.as_ref() else {
                return false;
            };
            let Some(physical) = state.concepts.as_ref() else {
                return false;
            };
            self.active_level1_concepts(sensory, physical)
        };

        // Cold / insufficiently abstracted observations stay at the lower
        // physical layer. No evaluator task-phase label is involved.
        if active_level1.is_empty() {
            return self.observe_phase_native_concept_factual(
                sensory,
                action,
                need,
            );
        }

        let (min_composite_support, child_ceiling, promotion_threshold) =
            self.concept_memory
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

        physical.auto_observations =
            physical.auto_observations.saturating_add(1);
        let observation_index = physical.auto_observations;

        // Revision of the existing explanation happens before deciding whether
        // a more abstract structure is justified.
        if physical.auto_child_revision_enabled {
            for &index in &active_level1 {
                let support_before =
                    physical.circuits[index].action_support[action];
                let synapse =
                    physical.circuits[index].motor_synapses[action];
                self.learn_phase_concept_running_mean_synapse(
                    synapse,
                    if need { 1.0 } else { 0.0 },
                    support_before,
                );
                physical.circuits[index].action_support[action] =
                    support_before.saturating_add(1);
            }
        }

        // One acquired concept can be revised/used directly; there is no
        // reason to allocate a pair abstraction.
        if active_level1.len() < 2 {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return true;
        }

        let children_supported_and_weak = active_level1.iter().all(|index| {
            let child = &physical.circuits[*index];
            if child.action_support[action] < min_action_support {
                return false;
            }
            let weight =
                self.synapses[child.motor_synapses[action]].weight;
            (2.0 * weight - 1.0).abs() <= child_ceiling
        });

        if children_supported_and_weak
            && physical.auto_first_weak_observation.is_none()
        {
            physical.auto_first_weak_observation = Some(observation_index);
        }

        if !physical.auto_abstraction_enabled {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return true;
        }

        if !physical.auto_always_escalate && !children_supported_and_weak {
            state.concepts = Some(physical);
            self.phase_native = Some(state);
            return true;
        }

        let candidates_before = physical.recursive.len();
        let promoted_before = physical
            .recursive
            .iter()
            .filter(|circuit| circuit.promoted)
            .count();

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
                    .position(|circuit| {
                        circuit.child_concept_ids == child_concept_ids
                    })
                {
                    index
                } else {
                    let child_cells = child_concept_ids.map(|concept_id| {
                        physical
                            .circuits
                            .iter()
                            .find(|circuit| {
                                circuit.promoted
                                    && circuit.concept_id == concept_id
                            })
                            .expect("auto abstraction child L1 concept")
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
                            self.native_synapse(
                                concept_cell,
                                self.motor_cell(motor),
                            )
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

                let support_before =
                    physical.recursive[recursive_index].support;
                for synapse in
                    physical.recursive[recursive_index].child_synapses
                {
                    self.learn_phase_concept_running_mean_synapse(
                        synapse,
                        1.0,
                        support_before,
                    );
                }

                let action_support_before =
                    physical.recursive[recursive_index]
                        .action_support[action];
                let motor_synapse =
                    physical.recursive[recursive_index]
                        .motor_synapses[action];
                self.learn_phase_concept_running_mean_synapse(
                    motor_synapse,
                    if need { 1.0 } else { 0.0 },
                    action_support_before,
                );
                physical.recursive[recursive_index]
                    .action_support[action] =
                    action_support_before.saturating_add(1);
                physical.recursive[recursive_index].support =
                    support_before.saturating_add(1);
            }
        }

        if physical.recursive.len() > candidates_before
            && physical.auto_first_candidate_observation.is_none()
        {
            physical.auto_first_candidate_observation =
                Some(observation_index);
        }

        self.reevaluate_recursive_promotions(
            &mut physical,
            min_composite_support,
            min_action_support,
            child_ceiling,
            promotion_threshold,
        );

        let promoted_after = physical
            .recursive
            .iter()
            .filter(|circuit| circuit.promoted)
            .count();
        if promoted_after > promoted_before
            && physical.auto_first_promotion_observation.is_none()
        {
            physical.auto_first_promotion_observation =
                Some(observation_index);
        }

        physical
            .recursive
            .sort_by_key(|circuit| circuit.concept_id);
        state.concepts = Some(physical);
        self.phase_native = Some(state);
        true
    }
}
