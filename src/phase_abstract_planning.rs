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
                // evaluator task mapping or target-depth knowledge.
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

    /// G18 goal-conditioned active information selector.
    ///
    /// The current raw goal is resolved to an acquired physical abstract cell.
    /// Goal relevance is propagated backward through the already-known physical
    /// abstract model. Unknown-action novelty is then weighted by that relevance
    /// and propagated backward again. The SAME two transferred P4 drive weights
    /// score direct and reachable goal-relevant epistemic value.
    pub fn choose_phase_native_goal_active_action(
        &mut self,
        goal_sensory: &[f32],
    ) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if entry.level != goal.level || entry.cell == goal.cell {
            return None;
        }

        let state_cells = self.phase_native_abstract_cells_at_level(entry.level);
        if !state_cells.contains(&entry.cell) || !state_cells.contains(&goal.cell) {
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

            let relevance =
                self.phase_goal_relevance_for_cells(state, goal.cell, &state_cells);
            let frontier = self.phase_goal_frontier_for_cells(
                state,
                goal.cell,
                &state_cells,
                &relevance,
            );

            let min_support = u64::from(self.config.min_recruit_support);
            let mut best: Option<(usize, f32, u64, [f32; 2])> = None;

            for action in 0..self.config.motor_cells {
                let features = if !self.phase_drive_action_known_at(
                    state,
                    entry.cell,
                    action,
                ) {
                    [relevance[entry.cell].clamp(0.0, 1.0), 0.0]
                } else {
                    let motor = self.config.sensory_cells + action;
                    let floor = state.config.coherence_floor;
                    let reachable = state
                        .circuits
                        .iter()
                        .filter(|circuit| {
                            circuit.support >= min_support
                                && self.synapses[circuit.afferent_synapse].from
                                    == entry.cell
                                && self.synapses[circuit.motor_synapse].to == motor
                        })
                        .map(|circuit| {
                            let afferent =
                                &self.synapses[circuit.afferent_synapse];
                            let successor =
                                &self.synapses[circuit.successor_synapse];
                            conductance(&self.cells, afferent, floor)
                                * conductance(&self.cells, successor, floor)
                                * frontier[successor.to]
                        })
                        .fold(0.0_f32, f32::max);
                    [0.0, reachable.clamp(0.0, 1.0)]
                };

                let score = self.phase_drive_score(state, features)?;
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
                            best =
                                Some((action, score, global_support, features));
                        }
                    }
                }
            }
            best
        }?;

        let (action, score, _, features) = best;
        if score <= 1.0e-8 {
            return self
                .phase_native_goal_decision_from_cells(
                    entry.cell,
                    goal.cell,
                    None,
                )
                .map(|decision| decision.first_action);
        }

        self.phase_native
            .as_mut()?
            .drive
            .as_mut()?
            .pending_features = Some(features);
        Some(action)
    }

    fn phase_goal_relevance_for_cells(
        &self,
        state: &PhaseNativeState,
        goal: usize,
        state_cells: &[usize],
    ) -> Vec<f32> {
        let mut relevance = vec![0.0_f32; self.cells.len()];
        if !state_cells.contains(&goal) {
            return relevance;
        }

        relevance[goal] = 1.0;
        let floor = state.config.coherence_floor;
        let min_support = u64::from(self.config.min_recruit_support);

        for _ in 0..state.config.horizon {
            let old = relevance.clone();
            relevance[goal] = 1.0;
            for circuit in &state.circuits {
                if circuit.support < min_support {
                    continue;
                }
                let afferent = &self.synapses[circuit.afferent_synapse];
                let successor = &self.synapses[circuit.successor_synapse];
                if !state_cells.contains(&afferent.from)
                    || !state_cells.contains(&successor.to)
                {
                    continue;
                }
                let propagated = state.config.discount
                    * conductance(&self.cells, afferent, floor)
                    * conductance(&self.cells, successor, floor)
                    * old[successor.to];
                if propagated > relevance[afferent.from] {
                    relevance[afferent.from] = propagated;
                }
            }
        }
        relevance
    }

    fn phase_goal_frontier_for_cells(
        &self,
        state: &PhaseNativeState,
        goal: usize,
        state_cells: &[usize],
        relevance: &[f32],
    ) -> Vec<f32> {
        let motor_cells = self.config.motor_cells;
        let floor = state.config.coherence_floor;
        let min_support = u64::from(self.config.min_recruit_support);

        let mut frontier = vec![0.0_f32; self.cells.len()];
        for &cell in state_cells {
            if cell == goal {
                continue;
            }
            let unknown = (0..motor_cells)
                .filter(|action| {
                    !self.phase_drive_action_known_at(state, cell, *action)
                })
                .count();
            frontier[cell] = relevance[cell]
                * (unknown as f32 / motor_cells as f32);
        }

        for _ in 0..state.config.horizon {
            let old = frontier.clone();
            for circuit in &state.circuits {
                if circuit.support < min_support {
                    continue;
                }
                let afferent = &self.synapses[circuit.afferent_synapse];
                let successor = &self.synapses[circuit.successor_synapse];
                if !state_cells.contains(&afferent.from)
                    || !state_cells.contains(&successor.to)
                {
                    continue;
                }
                let propagated = state.config.discount
                    * conductance(&self.cells, afferent, floor)
                    * conductance(&self.cells, successor, floor)
                    * old[successor.to];
                if propagated > frontier[afferent.from] {
                    frontier[afferent.from] = propagated;
                }
            }
        }
        frontier
    }

    /// G19: choose a physical experiment that discriminates supported rival
    /// transition predictions in a way that matters to the current raw goal.
    ///
    /// Rivals are ordinary supported phase-native transition circuits sharing
    /// pre-cell + opaque motor but predicting distinct acquired successor cells.
    pub fn choose_phase_native_goal_rival_probe(
        &mut self,
        goal_sensory: &[f32],
    ) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if entry.level != goal.level || entry.cell == goal.cell {
            return None;
        }

        let state_cells = self.phase_native_abstract_cells_at_level(entry.level);
        let state = self.phase_native.as_ref()?;
        let relevance =
            self.phase_goal_relevance_for_cells(state, goal.cell, &state_cells);

        let mut best: Option<(usize, f32, u64)> = None;
        for action in 0..self.config.motor_cells {
            let score = self.phase_goal_rival_disagreement_score(
                state,
                entry.cell,
                action,
                &relevance,
            );
            if score <= 1.0e-8 {
                continue;
            }
            let support = self.phase_action_global_support(state, action);
            match best {
                None => best = Some((action, score, support)),
                Some((best_action, best_score, best_support)) => {
                    if score > best_score + 1.0e-6
                        || ((score - best_score).abs() <= 1.0e-6
                            && (support > best_support
                                || (support == best_support
                                    && action < best_action)))
                    {
                        best = Some((action, score, support));
                    }
                }
            }
        }
        best.map(|(action, _, _)| action)
    }

    /// G19 diagnostic control: physical successor disagreement without a goal
    /// relevance field. Two equally-rival motors reduce to generic evidence /
    /// opaque-index tie breaking.
    #[doc(hidden)]
    pub fn choose_phase_native_rival_probe_no_goal_for_control(
        &self,
    ) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        let state = self.phase_native.as_ref()?;

        let mut best: Option<(usize, usize, u64)> = None;
        for action in 0..self.config.motor_cells {
            let rivals =
                self.phase_supported_rival_successors(state, entry.cell, action);
            if rivals.len() < 2 {
                continue;
            }
            let support = self.phase_action_global_support(state, action);
            match best {
                None => best = Some((action, rivals.len(), support)),
                Some((best_action, best_count, best_support)) => {
                    if rivals.len() > best_count
                        || (rivals.len() == best_count
                            && (support > best_support
                                || (support == best_support
                                    && action < best_action)))
                    {
                        best = Some((action, rivals.len(), support));
                    }
                }
            }
        }
        best.map(|(action, _, _)| action)
    }

    /// G19 matched novelty-only control: use the G18 goal-conditioned unknown
    /// machinery, but do not fall through into ordinary exploitation when no
    /// positive unknown/frontier score exists.
    #[doc(hidden)]
    pub fn choose_phase_native_goal_unknown_probe_only_for_control(
        &self,
        goal_sensory: &[f32],
    ) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if entry.level != goal.level || entry.cell == goal.cell {
            return None;
        }
        let state_cells = self.phase_native_abstract_cells_at_level(entry.level);
        let state = self.phase_native.as_ref()?;
        let relevance =
            self.phase_goal_relevance_for_cells(state, goal.cell, &state_cells);
        let frontier = self.phase_goal_frontier_for_cells(
            state,
            goal.cell,
            &state_cells,
            &relevance,
        );

        let min_support = u64::from(self.config.min_recruit_support);
        let mut best: Option<(usize, f32, u64)> = None;
        for action in 0..self.config.motor_cells {
            let features = if !self.phase_drive_action_known_at(
                state,
                entry.cell,
                action,
            ) {
                [relevance[entry.cell].clamp(0.0, 1.0), 0.0]
            } else {
                let motor = self.config.sensory_cells + action;
                let floor = state.config.coherence_floor;
                let reachable = state
                    .circuits
                    .iter()
                    .filter(|circuit| {
                        circuit.support >= min_support
                            && self.synapses[circuit.afferent_synapse].from
                                == entry.cell
                            && self.synapses[circuit.motor_synapse].to == motor
                    })
                    .map(|circuit| {
                        let afferent =
                            &self.synapses[circuit.afferent_synapse];
                        let successor =
                            &self.synapses[circuit.successor_synapse];
                        conductance(&self.cells, afferent, floor)
                            * conductance(&self.cells, successor, floor)
                            * frontier[successor.to]
                    })
                    .fold(0.0_f32, f32::max);
                [0.0, reachable.clamp(0.0, 1.0)]
            };

            let score = self.phase_drive_score(state, features)?;
            if score <= 1.0e-8 {
                continue;
            }
            let support = self.phase_action_global_support(state, action);
            match best {
                None => best = Some((action, score, support)),
                Some((best_action, best_score, best_support)) => {
                    if score > best_score + 1.0e-6
                        || ((score - best_score).abs() <= 1.0e-6
                            && (support > best_support
                                || (support == best_support
                                    && action < best_action)))
                    {
                        best = Some((action, score, support));
                    }
                }
            }
        }
        best.map(|(action, _, _)| action)
    }

    /// Commit one factual G19 probe. Matching physical prediction is learned
    /// normally; same-pre/same-action predictions with a different successor
    /// receive a physical contradiction update rather than host-side deletion.
    pub fn observe_phase_native_rival_probe_result(
        &mut self,
        action: usize,
        post_sensory: &[f32],
    ) -> Option<usize> {
        assert!(action < self.config.motor_cells);
        let pre_sensory = self.current_real.as_ref()?.sensory.clone();
        let pre = self.phase_native_abstract_state(&pre_sensory)?;
        let post = self.phase_native_abstract_state(post_sensory)?;
        if pre.level != post.level {
            return None;
        }

        if !self.observe_phase_native_abstract_transition(
            &pre_sensory,
            action,
            post_sensory,
            0.0,
        ) {
            return None;
        }

        let motor = self.motor_cell(action);
        let min_support = u64::from(self.config.min_recruit_support);
        let state = self.phase_native.as_ref()?;
        let rivals = state
            .circuits
            .iter()
            .enumerate()
            .filter_map(|(index, circuit)| {
                if circuit.support < min_support {
                    return None;
                }
                let afferent = &self.synapses[circuit.afferent_synapse];
                let successor = &self.synapses[circuit.successor_synapse];
                let output = &self.synapses[circuit.motor_synapse];
                (afferent.from == pre.cell
                    && output.to == motor
                    && successor.to != post.cell
                    && afferent.weight > 0.25
                    && successor.weight > 0.25)
                    .then_some(index)
            })
            .collect::<Vec<_>>();

        let mut state = self.phase_native.take()?;
        for circuit_index in &rivals {
            let circuit = &mut state.circuits[*circuit_index];
            circuit.revision = circuit.revision.saturating_add(1);
            circuit.counterexamples.push((circuit.support, 0.0));

            // The contradicted prediction remains structurally present for
            // provenance but loses executable successor conductance.
            let successor = &mut self.synapses[circuit.successor_synapse];
            successor.eligibility = 1.0;
            successor.weight = 0.0;
            successor.confidence = 0.0;
        }
        self.phase_native = Some(state);

        self.observe_initial_real(post_sensory, false);
        Some(rivals.len())
    }

    /// Diagnostic causal readout of G19's goal-relevant rival signal.
    #[doc(hidden)]
    pub fn phase_native_goal_rival_disagreement_for_control(
        &self,
        sensory: &[f32],
        goal_sensory: &[f32],
        action: usize,
    ) -> Option<f32> {
        let entry = self.phase_native_abstract_state(sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if entry.level != goal.level {
            return None;
        }
        let state_cells = self.phase_native_abstract_cells_at_level(entry.level);
        let state = self.phase_native.as_ref()?;
        let relevance =
            self.phase_goal_relevance_for_cells(state, goal.cell, &state_cells);
        Some(self.phase_goal_rival_disagreement_score(
            state,
            entry.cell,
            action,
            &relevance,
        ))
    }

    fn phase_action_global_support(
        &self,
        state: &PhaseNativeState,
        action: usize,
    ) -> u64 {
        let min_support = u64::from(self.config.min_recruit_support);
        let motor = self.config.sensory_cells + action;
        state
            .circuits
            .iter()
            .filter(|circuit| {
                circuit.support >= min_support
                    && self.synapses[circuit.motor_synapse].to == motor
            })
            .map(|circuit| circuit.support)
            .sum()
    }

    fn phase_supported_rival_successors(
        &self,
        state: &PhaseNativeState,
        entry: usize,
        action: usize,
    ) -> Vec<usize> {
        let min_support = u64::from(self.config.min_recruit_support);
        let motor = self.config.sensory_cells + action;
        let floor = state.config.coherence_floor;
        let mut successors = state
            .circuits
            .iter()
            .filter_map(|circuit| {
                if circuit.support < min_support {
                    return None;
                }
                let afferent = &self.synapses[circuit.afferent_synapse];
                let successor = &self.synapses[circuit.successor_synapse];
                let output = &self.synapses[circuit.motor_synapse];
                if afferent.from != entry
                    || output.to != motor
                    || conductance(&self.cells, afferent, floor) <= 1.0e-8
                    || conductance(&self.cells, successor, floor) <= 1.0e-8
                {
                    return None;
                }
                Some(successor.to)
            })
            .collect::<Vec<_>>();
        successors.sort_unstable();
        successors.dedup();
        successors
    }

    fn phase_goal_rival_disagreement_score(
        &self,
        state: &PhaseNativeState,
        entry: usize,
        action: usize,
        relevance: &[f32],
    ) -> f32 {
        let successors =
            self.phase_supported_rival_successors(state, entry, action);
        if successors.len() < 2 {
            return 0.0;
        }

        let mut min_relevance = f32::INFINITY;
        let mut max_relevance = f32::NEG_INFINITY;
        for successor in successors {
            let value = relevance.get(successor).copied().unwrap_or(0.0);
            min_relevance = min_relevance.min(value);
            max_relevance = max_relevance.max(value);
        }
        (max_relevance - min_relevance).max(0.0)
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

    /// Goal-conditioned physical planning. The goal is supplied only as a raw
    /// observation that must resolve to an already-acquired abstract cell.
    /// No learned outcome value is required for the goal signal.
    pub fn plan_phase_native_abstract_goal(
        &mut self,
        sensory: &[f32],
        goal_sensory: &[f32],
        depth: Option<usize>,
    ) -> Option<PlanDecision> {
        let current = self.phase_native_abstract_state(sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if current.level != goal.level || current.cell == goal.cell {
            return None;
        }
        self.phase_native_goal_decision_from_cells(
            current.cell,
            goal.cell,
            depth,
        )
    }

    fn phase_native_goal_decision_from_cells(
        &mut self,
        entry: usize,
        goal: usize,
        depth: Option<usize>,
    ) -> Option<PlanDecision> {
        if entry >= self.cells.len()
            || goal >= self.cells.len()
            || !self.cells[entry].recruited
            || !self.cells[goal].recruited
        {
            return None;
        }

        let state = self.phase_native.as_mut()?;
        state.last_local_updates = 0;
        state.last_motor_potentials.fill(0.0);
        let horizon = depth
            .unwrap_or(state.config.horizon)
            .min(state.config.horizon)
            .max(1);
        let floor = state.config.coherence_floor;

        let mut membranes = self.cells.clone();
        for cell in &mut membranes {
            cell.charge = 0.0;
        }
        membranes[goal].charge = 1.0;
        let mut latency = vec![0usize; membranes.len()];

        for _ in 0..horizon {
            let old = membranes.clone();
            let old_latency = latency.clone();

            for cell in &mut membranes {
                cell.charge = 0.0;
            }
            latency.fill(0);

            // Re-seed the requested acquired goal on every recurrence step so
            // paths shorter than the configured horizon remain available.
            membranes[goal].charge = 1.0;

            for circuit in &state.circuits {
                state.last_local_updates += 1;
                if circuit.support < u64::from(self.config.min_recruit_support) {
                    continue;
                }

                let afferent = &self.synapses[circuit.afferent_synapse];
                let successor = &self.synapses[circuit.successor_synapse];
                let future = state.config.discount
                    * conductance(&old, successor, floor)
                    * old[successor.to].charge;
                if future <= 1.0e-8 {
                    continue;
                }

                let current =
                    conductance(&old, afferent, floor) * future;
                if current > membranes[circuit.relay_cell].charge {
                    membranes[circuit.relay_cell].charge = current;
                }

                let delay = old_latency[successor.to].saturating_add(1);
                latency[circuit.relay_cell] = delay;

                if current > membranes[afferent.from].charge {
                    membranes[afferent.from].charge = current;
                    latency[afferent.from] = delay;
                }
            }
        }

        let mut motor_latency = vec![0usize; self.config.motor_cells];
        for circuit in &state.circuits {
            if self.synapses[circuit.afferent_synapse].from != entry {
                continue;
            }
            let output = &self.synapses[circuit.motor_synapse];
            let action =
                output.to.checked_sub(self.config.sensory_cells)?;
            if action >= self.config.motor_cells {
                return None;
            }
            let current = membranes[circuit.relay_cell].charge
                * conductance(&membranes, output, floor);
            if current > state.last_motor_potentials[action] {
                state.last_motor_potentials[action] = current;
                motor_latency[action] = latency[circuit.relay_cell];
            }
        }

        let mut selected = None;
        let mut peak = 1.0e-8;
        for (motor, potential) in
            state.last_motor_potentials.iter().copied().enumerate()
        {
            if potential > peak {
                peak = potential;
                selected = Some(motor);
            }
        }
        let action = selected?;

        Some(PlanDecision {
            first_action: action,
            predicted_value: peak,
            selected_depth: motor_latency[action],
            expanded_nodes: state.last_local_updates,
            authority: Authority::Imagined,
        })
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
