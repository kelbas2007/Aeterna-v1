// G13: one depth-generic higher-abstraction engine.
// Included by phase_native.rs after phase_concept.rs. The algorithm has no
// task-specific L2/L3 branch: every higher node carries its own level and
// children, and the same factual rule may create level+1.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhaseDeepChildRef {
    pub level: u8,
    pub id: u64,
    pub cell: usize,
}

#[derive(Debug, Clone)]
pub struct PhaseDeepNodeInfo {
    pub id: u64,
    pub level: u8,
    pub children: [PhaseDeepChildRef; 2],
    pub concept_cell: usize,
    pub child_synapses: [usize; 2],
    pub motor_synapses: Vec<usize>,
    pub support: u32,
    pub promoted: bool,
    action_support: Vec<u32>,
}

impl PhaseDeepNodeInfo {
    fn contains_synapse(&self, index: usize) -> bool {
        self.child_synapses.contains(&index) || self.motor_synapses.contains(&index)
    }
}

#[derive(Debug, Clone)]
pub(super) struct PhaseDeepState {
    max_level: u8,
    nodes: Vec<PhaseDeepNodeInfo>,
    next_id: u64,
}

impl PhaseDeepState {
    fn new(max_level: u8) -> Self {
        Self {
            max_level,
            nodes: Vec::new(),
            next_id: 1,
        }
    }
}

const OPEN_DEPTH_SAFETY_CEILING: u8 = 16;

impl EvoPhase {
    /// Ordinary self-selected-depth entrypoint. The ceiling is a generic
    /// substrate safety bound, not a task-supplied abstraction depth.
    pub fn enable_phase_native_open_depth_abstraction(&mut self) -> bool {
        self.enable_phase_native_depth_generic_abstraction(OPEN_DEPTH_SAFETY_CEILING)
    }

    pub fn enable_phase_native_depth_generic_abstraction(
        &mut self,
        max_level: u8,
    ) -> bool {
        if max_level < 2 || self.concept_memory.is_none() {
            return false;
        }
        let Some(state) = self.phase_native.as_mut() else {
            return false;
        };
        if state.concepts.is_none() || state.deep.is_some() {
            return false;
        }
        state.deep = Some(PhaseDeepState::new(max_level));
        true
    }

    pub fn phase_native_deep_nodes(&self) -> &[PhaseDeepNodeInfo] {
        self.phase_native
            .as_ref()
            .and_then(|state| state.deep.as_ref())
            .map(|deep| deep.nodes.as_slice())
            .unwrap_or(&[])
    }

    pub fn phase_native_promoted_deep_count(&self, level: u8) -> usize {
        self.phase_native_deep_nodes()
            .iter()
            .filter(|node| node.level == level && node.promoted)
            .count()
    }

    pub fn phase_native_deep_candidate_count(&self, level: u8) -> usize {
        self.phase_native_deep_nodes()
            .iter()
            .filter(|node| node.level == level)
            .count()
    }

    pub fn phase_native_deep_max_level(&self) -> Option<u8> {
        self.phase_native
            .as_ref()?
            .deep
            .as_ref()
            .map(|deep| deep.max_level)
    }

    pub fn phase_native_deep_synapses(&self) -> Vec<usize> {
        self.phase_native_deep_nodes()
            .iter()
            .flat_map(|node| {
                node.child_synapses
                    .iter()
                    .copied()
                    .chain(node.motor_synapses.iter().copied())
            })
            .collect()
    }

    pub(super) fn is_native_deep_synapse(&self, index: usize) -> bool {
        self.phase_native_deep_nodes()
            .iter()
            .any(|node| node.contains_synapse(index))
    }

    fn active_deep_refs(
        &self,
        sensory: &[f32],
        concepts: &PhaseConceptState,
        deep: &PhaseDeepState,
    ) -> Vec<PhaseDeepChildRef> {
        let Some(memory) = self.concept_memory.as_ref() else {
            return Vec::new();
        };
        let active_atom_ids = memory.active_atom_ids(sensory);
        let mut active = Vec::<PhaseDeepChildRef>::new();

        for circuit in &concepts.circuits {
            if circuit.promoted
                && circuit
                    .child_ids
                    .iter()
                    .all(|id| active_atom_ids.binary_search(id).is_ok())
            {
                active.push(PhaseDeepChildRef {
                    level: 1,
                    id: circuit.concept_id,
                    cell: circuit.concept_cell,
                });
            }
        }

        for level in 2..=deep.max_level {
            let prior = active
                .iter()
                .copied()
                .filter(|child| child.level + 1 == level)
                .collect::<Vec<_>>();
            if prior.is_empty() {
                continue;
            }
            for node in deep
                .nodes
                .iter()
                .filter(|node| node.promoted && node.level == level)
            {
                if node.children.iter().all(|wanted| {
                    prior.iter().any(|child| {
                        child.level == wanted.level
                            && child.id == wanted.id
                            && child.cell == wanted.cell
                    })
                }) {
                    active.push(PhaseDeepChildRef {
                        level,
                        id: node.id,
                        cell: node.concept_cell,
                    });
                }
            }
        }

        active.sort_unstable();
        active.dedup();
        active
    }

    fn deep_child_motor_state(
        &self,
        child: PhaseDeepChildRef,
        action: usize,
        concepts: &PhaseConceptState,
        deep: &PhaseDeepState,
    ) -> Option<(usize, u32)> {
        if child.level == 1 {
            let circuit = concepts
                .circuits
                .iter()
                .find(|circuit| {
                    circuit.promoted
                        && circuit.concept_id == child.id
                        && circuit.concept_cell == child.cell
                })?;
            return Some((
                circuit.motor_synapses[action],
                circuit.action_support[action],
            ));
        }

        let node = deep.nodes.iter().find(|node| {
            node.promoted
                && node.level == child.level
                && node.id == child.id
                && node.concept_cell == child.cell
        })?;
        Some((node.motor_synapses[action], node.action_support[action]))
    }

    fn increment_deep_child_support(
        child: PhaseDeepChildRef,
        action: usize,
        concepts: &mut PhaseConceptState,
        deep: &mut PhaseDeepState,
    ) {
        if child.level == 1 {
            let circuit = concepts
                .circuits
                .iter_mut()
                .find(|circuit| {
                    circuit.promoted
                        && circuit.concept_id == child.id
                        && circuit.concept_cell == child.cell
                })
                .expect("active promoted L1 child");
            circuit.action_support[action] =
                circuit.action_support[action].saturating_add(1);
            return;
        }

        let node = deep
            .nodes
            .iter_mut()
            .find(|node| {
                node.promoted
                    && node.level == child.level
                    && node.id == child.id
                    && node.concept_cell == child.cell
            })
            .expect("active promoted deep child");
        node.action_support[action] =
            node.action_support[action].saturating_add(1);
    }

    fn ordered_deep_children(
        a: PhaseDeepChildRef,
        b: PhaseDeepChildRef,
    ) -> [PhaseDeepChildRef; 2] {
        if (a.level, a.id, a.cell) <= (b.level, b.id, b.cell) {
            [a, b]
        } else {
            [b, a]
        }
    }

    fn deep_children_supported_weak(
        &self,
        children: &[PhaseDeepChildRef],
        action: usize,
        concepts: &PhaseConceptState,
        deep: &PhaseDeepState,
        min_action_support: u32,
        child_ceiling: f32,
    ) -> bool {
        !children.is_empty()
            && children.iter().all(|child| {
                let Some((synapse, support)) =
                    self.deep_child_motor_state(*child, action, concepts, deep)
                else {
                    return false;
                };
                if support < min_action_support {
                    return false;
                }
                (2.0 * self.synapses[synapse].weight - 1.0).abs()
                    <= child_ceiling
            })
    }

    fn reevaluate_deep_promotions(
        &mut self,
        concepts: &PhaseConceptState,
        deep: &mut PhaseDeepState,
        min_composite_support: u32,
        min_action_support: u32,
        child_ceiling: f32,
        promotion_threshold: f32,
    ) {
        for index in 0..deep.nodes.len() {
            if deep.nodes[index].promoted
                || deep.nodes[index].support < min_composite_support
            {
                continue;
            }

            let mut winning: Option<(usize, f32)> = None;
            for motor in 0..self.config.motor_cells {
                if deep.nodes[index].action_support[motor] < min_action_support {
                    continue;
                }
                let weight =
                    self.synapses[deep.nodes[index].motor_synapses[motor]].weight;
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

            let Some((winning_motor, evidence)) = winning else {
                continue;
            };
            if evidence < promotion_threshold {
                continue;
            }

            if self.deep_children_supported_weak(
                &deep.nodes[index].children,
                winning_motor,
                concepts,
                deep,
                min_action_support,
                child_ceiling,
            ) {
                deep.nodes[index].promoted = true;
            }
        }
    }

    /// One factual API for every higher level. No evaluator abstraction-level
    /// token is accepted. The currently highest active promoted level becomes
    /// the explanatory child set.
    pub fn observe_phase_native_depth_generic_factual(
        &mut self,
        sensory: &[f32],
        action: usize,
        need: bool,
    ) -> bool {
        assert!(action < self.config.motor_cells);

        let Some(memory) = self.concept_memory.as_ref() else {
            return false;
        };
        let (min_composite_support, child_ceiling, promotion_threshold) =
            memory.physical_promotion_thresholds();
        let min_action_support = memory.min_action_support();

        let mut state = match self.phase_native.take() {
            Some(state) => state,
            None => return false,
        };
        let Some(mut concepts) = state.concepts.take() else {
            self.phase_native = Some(state);
            return false;
        };
        let Some(mut deep) = state.deep.take() else {
            state.concepts = Some(concepts);
            self.phase_native = Some(state);
            return false;
        };

        if !state.config.learning_enabled {
            state.deep = Some(deep);
            state.concepts = Some(concepts);
            self.phase_native = Some(state);
            return false;
        }

        let active = self.active_deep_refs(sensory, &concepts, &deep);
        let Some(highest_level) = active.iter().map(|child| child.level).max()
        else {
            state.deep = Some(deep);
            state.concepts = Some(concepts);
            self.phase_native = Some(state);
            return false;
        };
        let children = active
            .into_iter()
            .filter(|child| child.level == highest_level)
            .collect::<Vec<_>>();

        let weak_before = self.deep_children_supported_weak(
            &children,
            action,
            &concepts,
            &deep,
            min_action_support,
            child_ceiling,
        );

        for child in &children {
            let (synapse, support_before) = self
                .deep_child_motor_state(
                    *child,
                    action,
                    &concepts,
                    &deep,
                )
                .expect("active deep child motor state");
            self.learn_phase_concept_running_mean_synapse(
                synapse,
                if need { 1.0 } else { 0.0 },
                support_before,
            );
            Self::increment_deep_child_support(
                *child,
                action,
                &mut concepts,
                &mut deep,
            );
        }

        let weak_after = self.deep_children_supported_weak(
            &children,
            action,
            &concepts,
            &deep,
            min_action_support,
            child_ceiling,
        );
        let allow_new = weak_before || weak_after;
        let target_level = highest_level.saturating_add(1);

        if children.len() >= 2 && target_level <= deep.max_level {
            let mut missing = 0usize;
            if allow_new {
                for i in 0..children.len() {
                    for j in (i + 1)..children.len() {
                        let pair = Self::ordered_deep_children(
                            children[i],
                            children[j],
                        );
                        if !deep.nodes.iter().any(|node| {
                            node.level == target_level
                                && node.children == pair
                        }) {
                            missing += 1;
                        }
                    }
                }
            }

            let free_cells = self
                .dormant_range()
                .filter(|cell| !self.cells[*cell].recruited)
                .count();
            if missing > 0
                && (!self.config.structural_growth_enabled
                    || free_cells < missing)
            {
                state.deep = Some(deep);
                state.concepts = Some(concepts);
                self.phase_native = Some(state);
                return false;
            }

            for i in 0..children.len() {
                for j in (i + 1)..children.len() {
                    let pair = Self::ordered_deep_children(
                        children[i],
                        children[j],
                    );
                    let node_index = if let Some(index) =
                        deep.nodes.iter().position(|node| {
                            node.level == target_level
                                && node.children == pair
                        })
                    {
                        index
                    } else {
                        if !allow_new {
                            continue;
                        }
                        let Some(concept_cell) = self
                            .dormant_range()
                            .find(|cell| !self.cells[*cell].recruited)
                        else {
                            state.deep = Some(deep);
                            state.concepts = Some(concepts);
                            self.phase_native = Some(state);
                            return false;
                        };
                        self.cells[concept_cell].recruited = true;
                        let child_synapses = [
                            self.native_synapse(pair[0].cell, concept_cell),
                            self.native_synapse(pair[1].cell, concept_cell),
                        ];
                        let motor_synapses = (0..self.config.motor_cells)
                            .map(|motor| {
                                self.native_synapse(
                                    concept_cell,
                                    self.motor_cell(motor),
                                )
                            })
                            .collect::<Vec<_>>();
                        let id = deep.next_id;
                        deep.next_id = deep.next_id.saturating_add(1);
                        deep.nodes.push(PhaseDeepNodeInfo {
                            id,
                            level: target_level,
                            children: pair,
                            concept_cell,
                            child_synapses,
                            motor_synapses,
                            support: 0,
                            promoted: false,
                            action_support: vec![
                                0;
                                self.config.motor_cells
                            ],
                        });
                        deep.nodes.len() - 1
                    };

                    let support_before = deep.nodes[node_index].support;
                    for synapse in deep.nodes[node_index].child_synapses {
                        self.learn_phase_concept_running_mean_synapse(
                            synapse,
                            1.0,
                            support_before,
                        );
                    }

                    let action_support_before =
                        deep.nodes[node_index].action_support[action];
                    let motor_synapse =
                        deep.nodes[node_index].motor_synapses[action];
                    self.learn_phase_concept_running_mean_synapse(
                        motor_synapse,
                        if need { 1.0 } else { 0.0 },
                        action_support_before,
                    );
                    deep.nodes[node_index].action_support[action] =
                        action_support_before.saturating_add(1);
                    deep.nodes[node_index].support =
                        support_before.saturating_add(1);
                }
            }
        }

        self.reevaluate_deep_promotions(
            &concepts,
            &mut deep,
            min_composite_support,
            min_action_support,
            child_ceiling,
            promotion_threshold,
        );

        deep.nodes.sort_by_key(|node| (node.level, node.id));
        state.deep = Some(deep);
        state.concepts = Some(concepts);
        self.phase_native = Some(state);
        true
    }

    /// Physical readout over an arbitrary configured depth. The same loop
    /// activates every promoted generic level in ascending level order.
    pub fn choose_phase_native_depth_generic_action(
        &self,
        sensory: &[f32],
    ) -> Option<usize> {
        let memory = self.concept_memory.as_ref()?;
        let active_atom_ids = memory.active_atom_ids(sensory);
        let state = self.phase_native.as_ref()?;
        let concepts = state.concepts.as_ref()?;
        let deep = state.deep.as_ref()?;
        let floor = state.config.coherence_floor;

        let mut activity = self.cells.clone();
        for cell in &mut activity {
            cell.charge = 0.0;
        }

        for atom in &concepts.atoms {
            if active_atom_ids.binary_search(&atom.atom_id).is_ok() {
                activity[atom.cell].charge = 1.0;
            }
        }

        let mut highest_active = 0u8;

        for circuit in &concepts.circuits {
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
            if current > 1.0e-8 {
                activity[circuit.concept_cell].charge =
                    activity[circuit.concept_cell].charge.max(current);
                highest_active = highest_active.max(1);
            }
        }

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
                if current > 1.0e-8 {
                    activity[node.concept_cell].charge =
                        activity[node.concept_cell].charge.max(current);
                    highest_active = highest_active.max(level);
                }
            }
        }

        if highest_active == 0 {
            return None;
        }

        let mut motor_potentials =
            vec![0.0f32; self.config.motor_cells];

        if highest_active == 1 {
            for circuit in &concepts.circuits {
                let current = activity[circuit.concept_cell].charge;
                if !circuit.promoted || current <= 1.0e-8 {
                    continue;
                }
                for (motor, synapse) in
                    circuit.motor_synapses.iter().copied().enumerate()
                {
                    let centered = 2.0 * self.synapses[synapse].weight - 1.0;
                    if centered <= 0.0 {
                        continue;
                    }
                    let output = current
                        * centered
                        * coherence(
                            &self.cells,
                            &self.synapses[synapse],
                            floor,
                        );
                    motor_potentials[motor] =
                        motor_potentials[motor].max(output);
                }
            }
        } else {
            for node in deep.nodes.iter().filter(|node| {
                node.promoted && node.level == highest_active
            }) {
                let current = activity[node.concept_cell].charge;
                if current <= 1.0e-8 {
                    continue;
                }
                for (motor, synapse) in
                    node.motor_synapses.iter().copied().enumerate()
                {
                    let centered = 2.0 * self.synapses[synapse].weight - 1.0;
                    if centered <= 0.0 {
                        continue;
                    }
                    let output = current
                        * centered
                        * coherence(
                            &self.cells,
                            &self.synapses[synapse],
                            floor,
                        );
                    motor_potentials[motor] =
                        motor_potentials[motor].max(output);
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
