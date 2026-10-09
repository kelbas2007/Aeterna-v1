// Acquired composite instructions. The interpreter and plasticity rule are
// inherited substrate; no named Boolean operation or task label is stored here.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhasePrimitiveConfig {
    pub capacity: usize,
    pub policy_learning_rate: f32,
}
impl Default for PhasePrimitiveConfig {
    fn default() -> Self {
        Self {
            capacity: 8,
            policy_learning_rate: 0.02,
        }
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhasePrimitive {
    program: InductionProgram,
    admitted: bool,
    source_action: usize,
    source_generation: u64,
    published_sequence: u64,
    construction_uses: u64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseNativeBoundCall {
    action: usize,
    operation_index: usize,
    source_inputs: Vec<usize>,
    /// Source sensory cell -> acquired argument sensory cell.
    binding_synapses: Vec<usize>,
    fit_facts: usize,
    future_checks: usize,
}

/// A competing physical candidate is a real input-to-input phase synapse.
/// Its endpoints *are* the two proposed argument channels; its conductance
/// is acquired from factual prediction/outcome agreement.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseSynapticArgumentCandidate {
    action: usize,
    operation_index: usize,
    source_inputs: [usize;2],
    candidate_synapse: usize,
    observations: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhasePrimitiveState {
    config: PhasePrimitiveConfig,
    operations: Vec<PhasePrimitive>,
    priorities: Vec<usize>,
    policy_updates: u64,
    #[serde(default)]
    argument_transfer_enabled: bool,
    /// Acquired sensory argument wiring; indices refer to actual PhaseSynapse.
    /// The original operation definitions are not mutated.
    #[serde(default)]
    bound_calls: Vec<PhaseNativeBoundCall>,
    /// Alternative opt-in: candidate competition is acquired in physical
    /// synapses instead of a bounded retrospective permutation scan.
    #[serde(default)]
    native_argument_competition: bool,
    #[serde(default)]
    argument_candidates: Vec<PhaseSynapticArgumentCandidate>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhasePrimitiveInfo {
    pub output_cell: usize,
    pub source_action: usize,
    pub source_generation: u64,
    pub published_sequence: u64,
    pub construction_uses: u64,
    pub active_nodes: usize,
    pub synapses: Vec<usize>,
    pub priority_synapse: usize,
    pub priority: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseConstructorInfo {
    pub factual_updates: u64,
    pub input_priorities: Vec<(usize, usize, f32)>,
}
fn primitive_fit_inputs(
    facts: &std::collections::VecDeque<InductionFact>,
    primitives: Option<&PhasePrimitiveState>,
    cfg: &PhaseInductionConfig,
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
) -> (Vec<InductionFact>, Vec<usize>, Vec<f32>) {
    let width = facts.front().map_or(0, |f| f.input.len());
    let mut fitted = facts.iter().cloned().collect::<Vec<_>>();
    let mut sources = (0..width).collect::<Vec<_>>();
    let mut priorities = vec![0.5; width];
    if let Some(s) = primitives {
        for (j, &index) in s.priorities.iter().take(width).enumerate() {
            let l = &links[index];
            priorities[j] = if l.confidence >= 0.5 { l.weight } else { 0.0 };
        }
        for (index, op) in s.operations.iter().enumerate() {
            if !op.admitted {
                continue;
            }
            // No fabricated value for an unsupported derived input. Include a
            // unit in a fit only when its physical definition covers every fact.
            let values = facts
                .iter()
                .map(|f| {
                    induction_read_inner(&op.program, cfg, &f.input, cells, links, Some(s), index)
                        .map(|v| v.0)
                })
                .collect::<Option<Vec<_>>>();
            if let Some(values) = values {
                for (f, x) in fitted.iter_mut().zip(values) {
                    f.input.push(x);
                }
                sources.push(op.program.nodes[0].cell);
                let l = &links[s.priorities[width + index]];
                priorities.push(if l.confidence >= 0.5 { l.weight } else { 0.0 });
            }
        }
    }
    (fitted, sources, priorities)
}
fn primitive_signature(p: &InductionProgram, links: &[PhaseSynapse]) -> Vec<u64> {
    let mut result = Vec::new();
    for n in &p.nodes[..p.active_nodes] {
        result.push(n.kind as u64);
        match n.kind {
            InductionNodeKind::Branch => {
                result.extend([
                    links[n.test].from as u64,
                    u64::from(links[n.test].phase_offset.to_bits()),
                ]);
                for edge in n.edges {
                    result.push(
                        p.nodes
                            .iter()
                            .position(|n| n.cell == links[edge].to)
                            .unwrap() as u64,
                    );
                }
            }
            InductionNodeKind::Leaf => result.push(u64::from(links[n.response].weight.to_bits())),
            InductionNodeKind::Unused => {}
        }
    }
    result
}
impl PhasePrimitiveState {
    fn fingerprint_feed(&self, feed: &mut impl FnMut(u64)) {
        feed(self.config.capacity as u64);
        feed(u64::from(self.config.policy_learning_rate.to_bits()));
        feed(self.policy_updates);
        if self.argument_transfer_enabled {
            feed(0x4152475452414E53);
        }
        feed(u64::from(self.native_argument_competition));
        for c in &self.argument_candidates {
            for v in [c.action,c.operation_index,c.source_inputs[0],
                      c.source_inputs[1],c.candidate_synapse,c.observations as usize] {
                feed(v as u64);
            }
        }
        for call in &self.bound_calls {
            for n in [
                call.action,
                call.operation_index,
                call.fit_facts,
                call.future_checks,
            ] {
                feed(n as u64);
            }
            for &i in call
                .source_inputs
                .iter()
                .chain(call.binding_synapses.iter())
            {
                feed(i as u64);
            }
        }
        for &i in &self.priorities {
            feed(i as u64);
        }
        for op in &self.operations {
            for value in [
                u64::from(op.admitted),
                op.source_action as u64,
                op.source_generation,
                op.published_sequence,
                op.construction_uses,
                op.program.active_nodes as u64,
                op.program.generation,
                op.program.born_sequence,
                op.program.future_checks,
            ] {
                feed(value);
            }
            for n in &op.program.nodes {
                feed(n.cell as u64);
                feed(n.kind as u64);
                feed(n.support);
                feed(n.future_checks);
                for i in n.links() {
                    feed(i as u64);
                }
            }
        }
    }
    fn contains_synapse(&self, index: usize) -> bool {
        self.argument_candidates.iter().any(|c|c.candidate_synapse==index)
            ||self.bound_calls
            .iter()
            .any(|b| b.binding_synapses.contains(&index))
            || self.priorities.contains(&index)
            || self
                .operations
                .iter()
                .flat_map(|op| &op.program.nodes)
                .any(|n| n.links().any(|i| i == index))
    }
    fn observe(
        &mut self,
        action: usize,
        input: &[f32],
        success: bool,
        cfg: &PhaseInductionConfig,
        programs: &mut [InductionProgram],
        cells: &[PhaseCell],
        links: &mut [PhaseSynapse],
    ) {
        let p = &programs[action];
        let Some((score, _)) = induction_read(p, cfg, input, cells, links, Some(self)) else {
            return;
        };
        // Factual credit belongs to the entire candidate's selected tests. This
        // learned policy breaks equal-loss search ties; it is not an oracle.
        let delta = if (score - f32::from(success)).abs() <= 1.0 - cfg.minimum_outcome {
            self.config.policy_learning_rate
        } else {
            -self.config.policy_learning_rate
        };
        let sources = p.nodes[..p.active_nodes]
            .iter()
            .filter(|n| n.kind == InductionNodeKind::Branch)
            .map(|n| links[n.test].from)
            .collect::<std::collections::BTreeSet<_>>();
        if !sources.is_empty() {
            self.policy_updates += 1;
            for &i in &self.priorities {
                if sources.contains(&links[i].from) {
                    links[i].weight = (links[i].weight + delta).clamp(0.0, 1.0);
                }
            }
        }
    }
    fn acquire(
        &mut self,
        action: usize,
        sequence: u64,
        cfg: &PhaseInductionConfig,
        p: &InductionProgram,
        links: &mut [PhaseSynapse],
    ) {
        if p.active_nodes < 3 || p.future_checks < cfg.min_future_checks {
            return;
        }
        let leaves = p.nodes[..p.active_nodes]
            .iter()
            .filter(|n| n.kind == InductionNodeKind::Leaf)
            .collect::<Vec<_>>();
        if leaves.iter().any(|n| {
            n.future_checks < cfg.min_leaf_checks
                || (links[n.response].weight > 1.0 - cfg.minimum_outcome
                    && links[n.response].weight < cfg.minimum_outcome)
        }) || !leaves
            .iter()
            .any(|n| links[n.response].weight >= cfg.minimum_outcome)
            || !leaves
                .iter()
                .any(|n| links[n.response].weight <= 1.0 - cfg.minimum_outcome)
        {
            return;
        }
        let signature = primitive_signature(p, links);
        if self
            .operations
            .iter()
            .any(|op| op.admitted && primitive_signature(&op.program, links) == signature)
        {
            return;
        }
        let Some(index) = self.operations.iter().position(|op| !op.admitted) else {
            return;
        };
        let root = self.operations[index].program.nodes[0].cell;
        // A call may refer only to earlier published definitions: a bounded DAG,
        // never self-modifying executable code or a cyclic call graph.
        if p.nodes[..p.active_nodes]
            .iter()
            .filter(|n| n.kind == InductionNodeKind::Branch)
            .any(|n| {
                links[n.test].from >= p.nodes[0].bounds.len()
                    && !self.operations[..index]
                        .iter()
                        .any(|op| op.admitted && op.program.nodes[0].cell == links[n.test].from)
            })
        {
            return;
        }
        let op = &mut self.operations[index];
        for j in 0..p.active_nodes {
            let old = &p.nodes[j];
            let new = &mut op.program.nodes[j];
            new.kind = old.kind;
            new.support = old.support;
            new.future_checks = old.future_checks;
            let old_test = links[old.test].clone();
            links[new.test].from = old_test.from;
            links[new.test].phase_offset = old_test.phase_offset;
            links[new.test].weight = old_test.weight;
            links[new.test].confidence = old_test.confidence;
            let old_response = links[old.response].clone();
            links[new.response].weight = old_response.weight;
            links[new.response].confidence = old_response.confidence;
            links[new.response].to = root;
            for (old_pair, new_pair) in old.bounds.iter().zip(&new.bounds) {
                for (&old_i, &new_i) in old_pair.iter().zip(new_pair) {
                    let value = links[old_i].clone();
                    links[new_i].phase_offset = value.phase_offset;
                    links[new_i].weight = value.weight;
                    links[new_i].confidence = value.confidence;
                }
            }
        }
        let mapped_cells = op.program.nodes.iter().map(|n| n.cell).collect::<Vec<_>>();
        for j in 0..p.active_nodes {
            if p.nodes[j].kind == InductionNodeKind::Branch {
                for side in 0..2 {
                    let old_edge = &links[p.nodes[j].edges[side]];
                    let dest = p.nodes.iter().position(|n| n.cell == old_edge.to).unwrap();
                    let weight = old_edge.weight;
                    let confidence = old_edge.confidence;
                    let edge = &mut links[op.program.nodes[j].edges[side]];
                    edge.to = mapped_cells[dest];
                    edge.weight = weight;
                    edge.confidence = confidence;
                }
            }
        }
        op.program.active_nodes = p.active_nodes;
        op.program.generation = 1;
        op.program.born_sequence = p.born_sequence;
        op.program.future_checks = p.future_checks;
        op.admitted = true;
        op.source_action = action;
        op.source_generation = p.generation;
        op.published_sequence = sequence;
    }
    fn record_construction(&mut self, p: &InductionProgram, links: &[PhaseSynapse]) {
        for op in &mut self.operations {
            if op.admitted
                && p.nodes[..p.active_nodes].iter().any(|n| {
                    n.kind == InductionNodeKind::Branch
                        && links[n.test].from == op.program.nodes[0].cell
                })
            {
                op.construction_uses += 1;
            }
        }
    }
}
impl EvoPhase {
    pub fn enable_phase_primitives(&mut self, config: PhasePrimitiveConfig) -> bool {
        let Some(vector) = self.phase_native.as_ref().and_then(|n| n.vector.as_ref()) else {
            return false;
        };
        let Some(state) = vector.induction.as_ref() else {
            return false;
        };
        let width = self.config.sensory_cells;
        let motors = self.config.motor_cells;
        let nodes = state.config.nodes_per_motor;
        if vector.factual_sequence != 0
            || state.primitives.is_some()
            || !(1..=8).contains(&config.capacity)
            || !config.policy_learning_rate.is_finite()
            || !(0.001..=0.25).contains(&config.policy_learning_rate)
            || (config.capacity + motors) * nodes > 512
            || self.synapses.len()
                + config.capacity * nodes * (4 + 2 * width)
                + width
                + config.capacity
                > MAX_VECTOR_SYNAPSES
        {
            return false;
        }
        let free = self
            .dormant_range()
            .filter(|&i| !self.cells[i].recruited)
            .take(config.capacity * nodes)
            .collect::<Vec<_>>();
        if free.len() != config.capacity * nodes {
            return false;
        }
        let mut next = free.into_iter();
        let mut operations = Vec::new();
        for _ in 0..config.capacity {
            let mut pool = Vec::new();
            for _ in 0..nodes {
                let cell = next.next().unwrap();
                self.cells[cell].recruited = true;
                let test = self.native_synapse(width + motors, cell);
                let edges = [
                    self.native_synapse(cell, cell),
                    self.native_synapse(cell, cell),
                ];
                let response = self.native_synapse(cell, cell);
                let bounds = (0..width)
                    .map(|j| [self.native_synapse(j, cell), self.native_synapse(j, cell)])
                    .collect();
                pool.push(InductionNode {
                    cell,
                    test,
                    edges,
                    response,
                    bounds,
                    kind: InductionNodeKind::Unused,
                    support: 0,
                    future_checks: 0,
                });
            }
            let root = pool[0].cell;
            for n in &pool {
                self.synapses[n.response].to = root;
            }
            operations.push(PhasePrimitive {
                program: InductionProgram {
                    nodes: pool,
                    active_nodes: 0,
                    generation: 0,
                    born_sequence: 0,
                    future_checks: 0,
                    revisions: 0,
                    discarded_conflicts: 0,
                    facts: Default::default(),
                    last_search_expansions: 0,
                },
                admitted: false,
                source_action: 0,
                source_generation: 0,
                published_sequence: 0,
                construction_uses: 0,
            });
        }
        let mut priorities = Vec::new();
        for source in (0..width).chain(operations.iter().map(|op| op.program.nodes[0].cell)) {
            let i = self.native_synapse(source, width + motors);
            self.synapses[i].weight = 0.5;
            self.synapses[i].confidence = 1.0;
            priorities.push(i);
        }
        self.phase_native
            .as_mut()
            .unwrap()
            .vector
            .as_mut()
            .unwrap()
            .induction
            .as_mut()
            .unwrap()
            .primitives = Some(PhasePrimitiveState {
            config,
            operations,
            priorities,
            policy_updates: 0,
            argument_transfer_enabled: false,
            bound_calls: Vec::new(),
            native_argument_competition:false,
            argument_candidates:Vec::new(),
        });
        true
    }
    pub fn phase_primitives(&self) -> Vec<PhasePrimitiveInfo> {
        let Some(s) = self
            .phase_native
            .as_ref()
            .and_then(|n| n.vector.as_ref())
            .and_then(|v| v.induction.as_ref())
            .and_then(|s| s.primitives.as_ref())
        else {
            return Vec::new();
        };
        s.operations
            .iter()
            .enumerate()
            .filter(|(_, op)| op.admitted)
            .map(|(i, op)| {
                let priority_synapse = s.priorities[self.config.sensory_cells + i];
                PhasePrimitiveInfo {
                    output_cell: op.program.nodes[0].cell,
                    source_action: op.source_action,
                    source_generation: op.source_generation,
                    published_sequence: op.published_sequence,
                    construction_uses: op.construction_uses,
                    active_nodes: op.program.active_nodes,
                    synapses: op.program.nodes.iter().flat_map(|n| n.links()).collect(),
                    priority_synapse,
                    priority: self.synapses[priority_synapse].weight,
                }
            })
            .collect()
    }
    pub fn phase_primitive_value(&self, output_cell: usize, frame: &[Option<f32>]) -> Option<f32> {
        if !vector_valid(frame, self.config.sensory_cells) {
            return None;
        }
        let input = frame.iter().copied().collect::<Option<Vec<_>>>()?;
        let state = self
            .phase_native
            .as_ref()?
            .vector
            .as_ref()?
            .induction
            .as_ref()?;
        let s = state.primitives.as_ref()?;
        let index = s
            .operations
            .iter()
            .position(|op| op.admitted && op.program.nodes[0].cell == output_cell)?;
        induction_read_inner(
            &s.operations[index].program,
            &state.config,
            &input,
            &self.cells,
            &self.synapses,
            Some(s),
            index,
        )
        .map(|v| v.0)
    }
    pub fn phase_constructor_info(&self) -> Option<PhaseConstructorInfo> {
        let s = self
            .phase_native
            .as_ref()?
            .vector
            .as_ref()?
            .induction
            .as_ref()?
            .primitives
            .as_ref()?;
        Some(PhaseConstructorInfo {
            factual_updates: s.policy_updates,
            input_priorities: s
                .priorities
                .iter()
                .map(|&i| (self.synapses[i].from, i, self.synapses[i].weight))
                .collect(),
        })
    }
}

// Parameterized views of the same acquired physical definitions.
include!("phase_primitive_arguments.rs");
