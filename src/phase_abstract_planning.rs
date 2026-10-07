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
        let memory = self.concept_memory.as_ref()?;
        let state = self.phase_native.as_ref()?;
        let concepts = state.concepts.as_ref()?;
        let active_atom_ids = memory.active_atom_ids(sensory);
        let floor = state.config.coherence_floor;

        // Raw recognition supplies only already-acquired atom identities.
        // Whether a higher abstraction is physically active is determined by
        // current through its ACTUAL acquired phase-sensitive synapses.
        let mut activity = self.cells.clone();
        for cell in &mut activity {
            cell.charge = 0.0;
        }
        for atom in &concepts.atoms {
            if active_atom_ids.binary_search(&atom.atom_id).is_ok() {
                activity[atom.cell].charge = 1.0;
            }
        }

        let mut active_refs = Vec::<PhaseAbstractStateRef>::new();

        for circuit in &concepts.circuits {
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
            activity[circuit.concept_cell].charge =
                activity[circuit.concept_cell].charge.max(current);
            active_refs.push(PhaseAbstractStateRef {
                level: 1,
                id: circuit.concept_id,
                cell: circuit.concept_cell,
            });
        }

        if let Some(deep) = state.deep.as_ref() {
            for level in 2..=deep.max_level {
                for node in deep
                    .nodes
                    .iter()
                    .filter(|node| node.promoted && node.level == level)
                {
                    let left = activity[node.children[0].cell].charge
                        * conductance(
                            &self.cells,
                            &self.synapses[node.child_synapses[0]],
                            floor,
                        );
                    let right = activity[node.children[1].cell].charge
                        * conductance(
                            &self.cells,
                            &self.synapses[node.child_synapses[1]],
                            floor,
                        );
                    let current = left.min(right);
                    if current <= 1.0e-8 {
                        continue;
                    }
                    activity[node.concept_cell].charge =
                        activity[node.concept_cell].charge.max(current);
                    active_refs.push(PhaseAbstractStateRef {
                        level,
                        id: node.id,
                        cell: node.concept_cell,
                    });
                }
            }
        }

        let highest = active_refs.iter().map(|state| state.level).max()?;
        let mut at_highest = active_refs
            .into_iter()
            .filter(|state| state.level == highest);
        let selected = at_highest.next()?;
        if at_highest.next().is_some() {
            return None;
        }
        Some(selected)
    }

    fn phase_native_abstract_cells_at_level(&self, level: u8) -> Vec<usize> {
        let Some(state) = self.phase_native.as_ref() else {
            return Vec::new();
        };
        let Some(concepts) = state.concepts.as_ref() else {
            return Vec::new();
        };

        let mut cells = if level == 1 {
            concepts
                .circuits
                .iter()
                .filter(|circuit| circuit.promoted)
                .map(|circuit| circuit.concept_cell)
                .collect::<Vec<_>>()
        } else {
            state
                .deep
                .as_ref()
                .map(|deep| {
                    deep.nodes
                        .iter()
                        .filter(|node| node.promoted && node.level == level)
                        .map(|node| node.concept_cell)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };
        cells.sort_unstable();
        cells.dedup();
        cells
    }

    /// G16 target selector: apply the SAME learned P4 epistemic weights to
    /// acquired abstract state cells. If no epistemic value remains, ordinary
    /// abstract model-based exploitation is allowed.
    pub fn choose_phase_native_abstract_learned_drive_action(
        &mut self,
    ) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        let state_cells = self.phase_native_abstract_cells_at_level(entry.level);
        if !state_cells.contains(&entry.cell) {
            return None;
        }

        let best = {
            let state = self.phase_native.as_ref()?;
            if state
                .drive
                .as_ref()
                .map(|drive| drive.config.readout_enabled)
                != Some(true)
            {
                return None;
            }

            let frontier =
                self.phase_drive_frontier_activity_for_cells(state, &state_cells);
            let min_support = u64::from(self.config.min_recruit_support);
            let mut best: Option<(usize, f32, u64, [f32; 2])> = None;
            for action in 0..self.config.motor_cells {
                let features =
                    self.phase_drive_features(state, entry.cell, action, &frontier);
                let score = self.phase_drive_score(state, features)?;

                // Evidence-order tie-break only. It does NOT change the two
                // inherited P4 epistemic features or their learned weights.
                // When two opaque actions have identical epistemic score,
                // prefer the motor that already owns more supported physical
                // transitions somewhere in the acquired abstract model.
                // This reuses factual cross-state evidence and contains no
                // route/depth/correct-action mapping.
                let global_support = state
                    .circuits
                    .iter()
                    .filter(|circuit| {
                        circuit.support >= min_support
                            && self.synapses[circuit.motor_synapse].to
                                == self.config.sensory_cells + action
                    })
                    .map(|circuit| circuit.support)
                    .sum::<u64>();

                match best {
                    None => best = Some((action, score, global_support, features)),
                    Some((best_action, best_score, best_support, _)) => {
                        if score > best_score + 1.0e-6
                            || ((score - best_score).abs() <= 1.0e-6
                                && (global_support > best_support
                                    || (global_support == best_support
                                        && action < best_action)))
                        {
                            best = Some((action, score, global_support, features));
                        }
                    }
                }
            }
            best
        }?;

        let (action, score, _, features) = best;
        if score <= 1.0e-8 {
            return self
                .phase_native_decision_from_cell(entry.cell, None)
                .map(|decision| decision.first_action);
        }

        self.phase_native
            .as_mut()?
            .drive
            .as_mut()?
            .pending_features = Some(features);
        Some(action)
    }

    /// Matched G16 diagnostic: explore only a locally unmodelled action at
    /// the current abstract state. It cannot deliberately navigate back to a
    /// deeper reachable frontier after a reset.
    pub fn choose_phase_native_abstract_direct_action(&self) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        let state = self.phase_native.as_ref()?;
        (0..self.config.motor_cells)
            .find(|action| !self.phase_drive_action_known_at(state, entry.cell, *action))
    }

    /// Commit one actually executed abstract action and update the transferred
    /// drive only from structural information gain. REAL advances to factual
    /// POST; no imagined state can enter this API.
    pub fn observe_phase_native_abstract_action_result(
        &mut self,
        action: usize,
        post_sensory: &[f32],
        value: f32,
        episode_boundary: bool,
    ) -> Option<bool> {
        let pre_sensory = self.current_real.as_ref()?.sensory.clone();
        let pre = self.phase_native_abstract_state(&pre_sensory)?;
        let post = self.phase_native_abstract_state(post_sensory)?;
        if pre.level != post.level {
            return None;
        }
        let state_cells = self.phase_native_abstract_cells_at_level(pre.level);
        let before = self.phase_native_circuits().len();
        let accepted = self.observe_phase_native_abstract_transition_boundary(
            &pre_sensory,
            action,
            post_sensory,
            value,
            episode_boundary,
        );
        let structural_gain = self.phase_native_circuits().len() > before;
        self.phase_native_drive_after_cell_fact(
            post.cell,
            &state_cells,
            structural_gain,
        );
        self.observe_initial_real(post_sensory, value >= 1.0);
        Some(accepted)
    }

    fn observe_phase_native_abstract_transition_boundary(
        &mut self,
        pre_sensory: &[f32],
        action: usize,
        post_sensory: &[f32],
        value: f32,
        episode_boundary: bool,
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

        if accepted && episode_boundary {
            let motor = self.motor_cell(action);
            if let Some(circuit) = state.circuits.iter().find(|circuit| {
                self.synapses[circuit.afferent_synapse].from == pre.cell
                    && self.synapses[circuit.successor_synapse].to == post.cell
                    && self.synapses[circuit.motor_synapse].to == motor
            }) {
                // A factual episode boundary is encoded in the physical
                // successor link itself: the transition remains known
                // (weight>0), but a pi phase offset blocks fictitious future
                // continuation/frontier propagation past the terminal edge.
                let index = circuit.successor_synapse;
                let synapse = &mut self.synapses[index];
                let coherent = wrap_phase(
                    self.cells[synapse.to].phase - self.cells[synapse.from].phase
                );
                synapse.phase_offset =
                    wrap_phase(coherent + std::f32::consts::PI);
            }
        }

        self.phase_native = Some(state);
        accepted
    }

    /// Learn one nonterminal factual transition directly between already-acquired
    /// physical abstraction cells using the SAME P1 circuit and synapse arrays.
    pub fn observe_phase_native_abstract_transition(
        &mut self,
        pre_sensory: &[f32],
        action: usize,
        post_sensory: &[f32],
        value: f32,
    ) -> bool {
        self.observe_phase_native_abstract_transition_boundary(
            pre_sensory,
            action,
            post_sensory,
            value,
            false,
        )
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
