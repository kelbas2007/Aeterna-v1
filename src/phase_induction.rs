// Generic experience-built conditional programs. Included in phase_native.rs.
// Learned numerical tests, edges and responses live in shared physical links.
// Partition search is inherited software, not an invented primitive or solver.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseInductionConfig {
    pub nodes_per_motor: usize,
    pub max_depth: usize,
    pub search_expansions: usize,
    pub facts_per_motor: usize,
    pub min_child_support: usize,
    pub min_future_checks: u64,
    pub min_leaf_checks: u64,
    pub coverage_radius: f32,
    pub minimum_outcome: f32,
    pub minimum_margin: f32,
}
impl Default for PhaseInductionConfig {
    fn default() -> Self {
        Self {
            nodes_per_motor: 15,
            max_depth: 6,
            search_expansions: 2048,
            facts_per_motor: 128,
            min_child_support: 2,
            min_future_checks: 8,
            min_leaf_checks: 2,
            coverage_radius: 0.15,
            minimum_outcome: 0.9,
            minimum_margin: 0.1,
        }
    }
}
impl PhaseInductionConfig {
    fn valid(&self, width: usize, motors: usize) -> bool {
        (1..=64).contains(&width)
            && (1..=64).contains(&motors)
            && (3..=63).contains(&self.nodes_per_motor)
            && self.nodes_per_motor % 2 == 1
            && self
                .nodes_per_motor
                .checked_mul(motors)
                .is_some_and(|n| n <= 512)
            && (1..=8).contains(&self.max_depth)
            && (32..=8192).contains(&self.search_expansions)
            && (8..=256).contains(&self.facts_per_motor)
            && width
                .checked_mul(motors)
                .and_then(|n| n.checked_mul(self.facts_per_motor))
                .is_some_and(|n| n <= 131_072)
            && (2..=16).contains(&self.min_child_support)
            && self.facts_per_motor >= self.min_child_support * 2
            && (2..=64).contains(&self.min_future_checks)
            && (1..=16).contains(&self.min_leaf_checks)
            && self.coverage_radius.is_finite()
            && (0.0..=0.25).contains(&self.coverage_radius)
            && self.minimum_outcome.is_finite()
            && (0.5..=1.0).contains(&self.minimum_outcome)
            && self.minimum_margin.is_finite()
            && (0.0..=1.0).contains(&self.minimum_margin)
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InductionFact {
    input: Vec<f32>,
    success: bool,
    sequence: u64,
}
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
enum InductionNodeKind {
    Unused,
    Leaf,
    Branch,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InductionNode {
    cell: usize,
    test: usize,
    edges: [usize; 2],
    response: usize,
    bounds: Vec<[usize; 2]>,
    kind: InductionNodeKind,
    support: u64,
    future_checks: u64,
}
impl InductionNode {
    fn links(&self) -> impl Iterator<Item = usize> + '_ {
        std::iter::once(self.test)
            .chain(self.edges)
            .chain(std::iter::once(self.response))
            .chain(self.bounds.iter().flatten().copied())
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InductionProgram {
    nodes: Vec<InductionNode>,
    active_nodes: usize,
    generation: u64,
    born_sequence: u64,
    future_checks: u64,
    revisions: u64,
    discarded_conflicts: u64,
    facts: std::collections::VecDeque<InductionFact>,
    #[serde(default)]
    last_search_expansions: usize,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseInductionState {
    config: PhaseInductionConfig,
    programs: Vec<InductionProgram>,
    #[serde(default)]
    primitives: Option<PhasePrimitiveState>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseInductionPrediction {
    pub action: usize,
    pub expected_outcome: f32,
    pub margin: f32,
    pub root_cell: usize,
    pub leaf_cell: usize,
    pub authority: Authority,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseInductionNodeInfo {
    pub cell: usize,
    pub input: Option<usize>,
    pub threshold: Option<f32>,
    pub children: Option<[usize; 2]>,
    pub outcome: Option<f32>,
    pub future_checks: u64,
    pub synapses: Vec<usize>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseInductionProgramInfo {
    pub action: usize,
    pub generation: u64,
    pub revisions: u64,
    pub discarded_conflicts: u64,
    pub future_checks: u64,
    pub retained_facts: usize,
    pub allocated_nodes: usize,
    pub primitive_inputs: Vec<usize>,
    pub last_search_expansions: usize,
    pub nodes: Vec<PhaseInductionNodeInfo>,
}
// Transient construction only; never serialized or consulted by prediction.
enum InductionDraft {
    Leaf {
        outcome: f32,
        support: u64,
        bounds: Vec<(f32, f32)>,
    },
    Branch {
        input: usize,
        threshold: f32,
        lower: Box<Self>,
        upper: Box<Self>,
    },
}
fn induction_fit(
    facts: &[&InductionFact],
    cfg: &PhaseInductionConfig,
    depth: usize,
    budget: usize,
    search_left: &mut usize,
    priorities: &[f32],
) -> (InductionDraft, f64, usize) {
    let width = facts[0].input.len();
    let positives = facts.iter().filter(|f| f.success).count();
    let loss = positives as f64 * (facts.len() - positives) as f64 / facts.len() as f64;
    let leaf = InductionDraft::Leaf {
        outcome: positives as f32 / facts.len() as f32,
        support: facts.len() as u64,
        bounds: (0..width)
            .map(|j| {
                let lo = facts
                    .iter()
                    .map(|f| f.input[j])
                    .fold(f32::INFINITY, f32::min);
                let hi = facts
                    .iter()
                    .map(|f| f.input[j])
                    .fold(f32::NEG_INFINITY, f32::max);
                (lo, hi)
            })
            .collect(),
    };
    if positives == 0
        || positives == facts.len()
        || depth == cfg.max_depth
        || budget < 3
        || *search_left == 0
    {
        return (leaf, loss, 1);
    }
    let mut options = Vec::new();
    for input in 0..width {
        let mut values = facts.iter().map(|f| f.input[input]).collect::<Vec<_>>();
        values.sort_by(f32::total_cmp);
        values.dedup();
        for pair in values.windows(2) {
            let threshold = pair[0] + (pair[1] - pair[0]) * 0.5;
            let mut counts = [[0usize; 2]; 2];
            for f in facts {
                counts[usize::from(f.input[input] > threshold)][usize::from(f.success)] += 1;
            }
            if counts
                .iter()
                .any(|c| c.iter().sum::<usize>() < cfg.min_child_support)
            {
                continue;
            }
            let immediate_loss = counts
                .iter()
                .map(|c| c[0] as f64 * c[1] as f64 / (c[0] + c[1]) as f64)
                .sum::<f64>();
            options.push((immediate_loss, input, threshold));
        }
    }
    options.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| priorities[b.1].total_cmp(&priorities[a.1]))
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.total_cmp(&b.2))
    });
    let mut best = (leaf, loss, 1);
    for (_, input, threshold) in options {
        if *search_left == 0 {
            break;
        }
        *search_left -= 1;
        let (lower, upper): (Vec<_>, Vec<_>) = facts
            .iter()
            .copied()
            .partition(|f| f.input[input] <= threshold);
        let (lower, lower_loss, lower_nodes) = induction_fit(
            &lower,
            cfg,
            depth + 1,
            (budget - 1) / 2,
            search_left,
            priorities,
        );
        let (upper, upper_loss, upper_nodes) = induction_fit(
            &upper,
            cfg,
            depth + 1,
            budget - 1 - lower_nodes,
            search_left,
            priorities,
        );
        let candidate_loss = lower_loss + upper_loss;
        let nodes = 1 + lower_nodes + upper_nodes;
        if candidate_loss < best.1 || (candidate_loss == best.1 && nodes < best.2) {
            best = (
                InductionDraft::Branch {
                    input,
                    threshold,
                    lower: Box::new(lower),
                    upper: Box::new(upper),
                },
                candidate_loss,
                nodes,
            );
        }
        if best.1 == 0.0 {
            break;
        }
    }
    best
}
fn induction_read(
    program: &InductionProgram,
    cfg: &PhaseInductionConfig,
    input: &[f32],
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
    primitives: Option<&PhasePrimitiveState>,
) -> Option<(f32, usize)> {
    induction_read_inner(
        program,
        cfg,
        input,
        cells,
        links,
        primitives,
        primitives.map_or(0, |s| s.operations.len()),
    )
}
fn induction_read_inner(
    program: &InductionProgram,
    cfg: &PhaseInductionConfig,
    input: &[f32],
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
    primitives: Option<&PhasePrimitiveState>,
    primitive_limit: usize,
) -> Option<(f32, usize)> {
    // Ephemeral results of reading physical definitions in this one query.
    // Each acquired operation is evaluated at most once, even in a deep DAG.
    let mut cache = vec![None; primitives.map_or(0, |s| s.operations.len())];
    induction_read_cached(
        program,
        cfg,
        input,
        cells,
        links,
        primitives,
        primitive_limit,
        &mut cache,
    )
}
fn induction_read_cached(
    program: &InductionProgram,
    cfg: &PhaseInductionConfig,
    input: &[f32],
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
    primitives: Option<&PhasePrimitiveState>,
    primitive_limit: usize,
    cache: &mut [Option<Option<f32>>],
) -> Option<(f32, usize)> {
    if program.active_nodes == 0 {
        return None;
    }
    let mut index = 0;
    for _ in 0..=cfg.max_depth {
        let node = program.nodes.get(index)?;
        if index >= program.active_nodes || !cells[node.cell].recruited {
            return None;
        }
        match node.kind {
            InductionNodeKind::Unused => return None,
            InductionNodeKind::Leaf => {
                let response = &links[node.response];
                if response.confidence < 0.5 || !cells[response.to].recruited {
                    return None;
                }
                for (&x, pair) in input.iter().zip(&node.bounds) {
                    let lo = &links[pair[0]];
                    let hi = &links[pair[1]];
                    if lo.weight < 0.5
                        || hi.weight < 0.5
                        || lo.confidence < 0.5
                        || hi.confidence < 0.5
                        || x < lo.phase_offset / std::f32::consts::PI - cfg.coverage_radius
                        || x > hi.phase_offset / std::f32::consts::PI + cfg.coverage_radius
                    {
                        return None;
                    }
                }
                return Some((response.weight, index));
            }
            InductionNodeKind::Branch => {
                let test = &links[node.test];
                if test.weight < 0.5 || test.confidence < 0.5 || !cells[test.from].recruited {
                    return None;
                }
                let value = if let Some(&x) = input.get(test.from) {
                    x
                } else {
                    let s = primitives?;
                    let index = s
                        .operations
                        .iter()
                        .take(primitive_limit)
                        .position(|op| op.program.nodes[0].cell == test.from && op.admitted)?;
                    let value = if let Some(value) = cache[index] {
                        value
                    } else {
                        let value = induction_read_cached(
                            &s.operations[index].program,
                            cfg,
                            input,
                            cells,
                            links,
                            primitives,
                            index,
                            cache,
                        )
                        .map(|v| v.0);
                        cache[index] = Some(value);
                        value
                    };
                    value?
                };
                let side = usize::from(value > test.phase_offset / std::f32::consts::PI);
                let edge = &links[node.edges[side]];
                if edge.weight < 0.5 || edge.confidence < 0.5 {
                    return None;
                }
                index = program.nodes.iter().position(|n| n.cell == edge.to)?;
            }
        }
    }
    None
}
fn induction_install(
    draft: InductionDraft,
    program: &mut InductionProgram,
    links: &mut [PhaseSynapse],
    sources: &[usize],
) -> usize {
    let index = program.active_nodes;
    program.active_nodes += 1;
    match draft {
        InductionDraft::Leaf {
            outcome,
            support,
            bounds,
        } => {
            let node = &mut program.nodes[index];
            node.kind = InductionNodeKind::Leaf;
            node.support = support;
            let response = &mut links[node.response];
            response.weight = outcome;
            response.confidence = 1.0;
            for (j, ((lo, hi), pair)) in bounds.into_iter().zip(&node.bounds).enumerate() {
                for (i, value) in [(pair[0], lo), (pair[1], hi)] {
                    let l = &mut links[i];
                    l.from = j;
                    l.phase_offset = value * std::f32::consts::PI;
                    l.weight = 1.0;
                    l.confidence = 1.0;
                }
            }
        }
        InductionDraft::Branch {
            input,
            threshold,
            lower,
            upper,
        } => {
            program.nodes[index].kind = InductionNodeKind::Branch;
            let test = &mut links[program.nodes[index].test];
            test.from = sources[input];
            test.phase_offset = threshold * std::f32::consts::PI;
            test.weight = 1.0;
            test.confidence = 1.0;
            let children = [
                induction_install(*lower, program, links, sources),
                induction_install(*upper, program, links, sources),
            ];
            for (side, child) in children.into_iter().enumerate() {
                let edge = &mut links[program.nodes[index].edges[side]];
                edge.to = program.nodes[child].cell;
                edge.weight = 1.0;
                edge.confidence = 1.0;
            }
        }
    }
    index
}
impl EvoPhase {
    pub fn enable_phase_induction(&mut self, config: PhaseInductionConfig) -> bool {
        let width = self.config.sensory_cells;
        let motors = self.config.motor_cells;
        if !config.valid(width, motors) || !self.config.structural_growth_enabled {
            return false;
        }
        let Some(vector) = self.phase_native.as_ref().and_then(|n| n.vector.as_ref()) else {
            return false;
        };
        if vector.induction.is_some() || vector.factual_sequence != 0 {
            return false;
        }
        let count = motors * config.nodes_per_motor;
        if self.synapses.len() + count * (4 + 2 * width) > MAX_VECTOR_SYNAPSES {
            return false;
        }
        let free = self
            .dormant_range()
            .filter(|&i| !self.cells[i].recruited)
            .take(count)
            .collect::<Vec<_>>();
        if free.len() != count {
            return false;
        }
        let mut next = free.into_iter();
        let mut programs = Vec::new();
        for action in 0..motors {
            let mut nodes = Vec::new();
            for _ in 0..config.nodes_per_motor {
                let cell = next.next().unwrap();
                self.cells[cell].recruited = true;
                let test = self.native_synapse(width + motors, cell);
                let edges = [
                    self.native_synapse(cell, cell),
                    self.native_synapse(cell, cell),
                ];
                let response = self.native_synapse(cell, self.motor_cell(action));
                let bounds = (0..width)
                    .map(|j| [self.native_synapse(j, cell), self.native_synapse(j, cell)])
                    .collect();
                nodes.push(InductionNode {
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
            programs.push(InductionProgram {
                nodes,
                active_nodes: 0,
                generation: 0,
                born_sequence: 0,
                future_checks: 0,
                revisions: 0,
                discarded_conflicts: 0,
                facts: Default::default(),
                last_search_expansions: 0,
            });
        }
        self.phase_native
            .as_mut()
            .unwrap()
            .vector
            .as_mut()
            .unwrap()
            .induction = Some(PhaseInductionState {
            config,
            programs,
            primitives: None,
        });
        true
    }
    pub fn phase_induction_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|n| n.vector.as_ref())
            .is_some_and(|v| v.induction.is_some())
    }
    pub fn phase_induction_predict(
        &self,
        frame: &[Option<f32>],
    ) -> Option<PhaseInductionPrediction> {
        if let Some(call) = self.phase_primitive_argument_prediction(frame) {
            return Some(call);
        }
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
        let mut choices = Vec::new();
        for (action, p) in state.programs.iter().enumerate() {
            if p.future_checks < state.config.min_future_checks {
                continue;
            }
            let Some((score, index)) = induction_read(
                p,
                &state.config,
                &input,
                &self.cells,
                &self.synapses,
                state.primitives.as_ref(),
            ) else {
                continue;
            };
            if p.nodes[index].future_checks < state.config.min_leaf_checks {
                continue;
            }
            choices.push((score, action, index));
        }
        choices.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let &(score, action, index) = choices.first()?;
        let margin = score - choices.get(1).map_or(0.0, |c| c.0);
        if score < state.config.minimum_outcome || margin < state.config.minimum_margin {
            return None;
        }
        let p = &state.programs[action];
        Some(PhaseInductionPrediction {
            action,
            expected_outcome: score,
            margin,
            root_cell: p.nodes[0].cell,
            leaf_cell: p.nodes[index].cell,
            authority: Authority::Imagined,
        })
    }
    pub fn phase_induction_programs(&self) -> Vec<PhaseInductionProgramInfo> {
        let Some(state) = self
            .phase_native
            .as_ref()
            .and_then(|n| n.vector.as_ref())
            .and_then(|v| v.induction.as_ref())
        else {
            return Vec::new();
        };
        state
            .programs
            .iter()
            .enumerate()
            .map(|(action, p)| PhaseInductionProgramInfo {
                action,
                generation: p.generation,
                revisions: p.revisions,
                discarded_conflicts: p.discarded_conflicts,
                future_checks: p.future_checks,
                retained_facts: p.facts.len(),
                allocated_nodes: p.nodes.len(),
                primitive_inputs: p.nodes[..p.active_nodes]
                    .iter()
                    .filter(|n| n.kind == InductionNodeKind::Branch)
                    .map(|n| self.synapses[n.test].from)
                    .filter(|&source| source >= self.config.sensory_cells)
                    .collect(),
                last_search_expansions: p.last_search_expansions,
                nodes: p.nodes[..p.active_nodes]
                    .iter()
                    .map(|n| PhaseInductionNodeInfo {
                        cell: n.cell,
                        input: (n.kind == InductionNodeKind::Branch)
                            .then(|| self.synapses[n.test].from),
                        threshold: (n.kind == InductionNodeKind::Branch)
                            .then(|| self.synapses[n.test].phase_offset / std::f32::consts::PI),
                        children: (n.kind == InductionNodeKind::Branch)
                            .then(|| n.edges.map(|i| self.synapses[i].to)),
                        outcome: (n.kind == InductionNodeKind::Leaf)
                            .then(|| self.synapses[n.response].weight),
                        future_checks: n.future_checks,
                        synapses: n.links().collect(),
                    })
                    .collect(),
            })
            .collect()
    }
    fn is_phase_induction_synapse(&self, index: usize) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|n| n.vector.as_ref())
            .and_then(|v| v.induction.as_ref())
            .is_some_and(|s| {
                s.programs
                    .iter()
                    .flat_map(|p| &p.nodes)
                    .any(|n| n.links().any(|i| i == index))
                    || s.primitives
                        .as_ref()
                        .is_some_and(|p| p.contains_synapse(index))
            })
    }
    fn observe_phase_induction_result(&mut self, action: usize, pre: &[Option<f32>], outcome: f32) {
        let Some(input) = pre.iter().copied().collect::<Option<Vec<_>>>() else {
            return;
        };
        let Some(vector) = self.phase_native.as_mut().and_then(|n| n.vector.as_mut()) else {
            return;
        };
        let Some(state) = vector.induction.as_mut() else {
            return;
        };
        let sequence = vector.factual_sequence;
        let success = outcome >= vector.config.success_threshold;
        if let Some(primitives) = state.primitives.as_mut() {
            primitives.observe(
                action,
                &input,
                success,
                &state.config,
                &mut state.programs,
                &self.cells,
                &mut self.synapses,
            );
        }
        let p = &mut state.programs[action];
        let prediction = induction_read(
            p,
            &state.config,
            &input,
            &self.cells,
            &self.synapses,
            state.primitives.as_ref(),
        );
        let agrees =
            |score: f32| (score - f32::from(success)).abs() <= 1.0 - state.config.minimum_outcome;
        let rebuild = prediction.is_none_or(|(score, _)| !agrees(score));
        if rebuild {
            // Withdraw the old program immediately, even when replacing
            // conflicting tuples leaves too little evidence to build a new one.
            p.future_checks = 0;
            for node in &mut p.nodes {
                node.future_checks = 0;
            }
        }
        if let Some((_, index)) = prediction.filter(|(score, _)| agrees(*score)) {
            p.future_checks += 1;
            p.nodes[index].future_checks += 1;
        }
        // Latest genuine feedback supersedes conflicting facts for the same
        // measured input in this deterministic program mode. Keep aggregate
        // revision/contradiction evidence, rather than an unbounded replay log.
        let conflicts = p
            .facts
            .iter()
            .filter(|f| f.input == input && f.success != success)
            .count();
        p.discarded_conflicts += conflicts as u64;
        p.facts.retain(|f| f.input != input || f.success == success);
        if p.facts.len() == state.config.facts_per_motor {
            // Preserve both factual outcomes under self-selected experience.
            let same = p.facts.iter().filter(|f| f.success == success).count();
            let position = p
                .facts
                .iter()
                .position(|f| {
                    f.success
                        == (if same >= state.config.facts_per_motor / 2 {
                            success
                        } else {
                            !success
                        })
                })
                .unwrap();
            p.facts.remove(position);
        }
        p.facts.push_back(InductionFact {
            input,
            success,
            sequence,
        });
        // Reuse a factually validated call before rebuilding its definition.
        if state.primitives.as_ref().is_some_and(|library| {
            primitive_argument_match(
                &p.facts,
                library,
                &state.config,
                &self.cells,
                &self.synapses,
            )
            .is_some()
        }) {
            return;
        }
        if rebuild && p.facts.len() >= state.config.min_child_support {
            let (fit_facts, sources, priorities) = primitive_fit_inputs(
                &p.facts,
                state.primitives.as_ref(),
                &state.config,
                &self.cells,
                &self.synapses,
            );
            let facts = fit_facts.iter().collect::<Vec<_>>();
            let mut search_left = state.config.search_expansions;
            let (draft, _, _) = induction_fit(
                &facts,
                &state.config,
                0,
                p.nodes.len(),
                &mut search_left,
                &priorities,
            );
            p.last_search_expansions = state.config.search_expansions - search_left;
            p.active_nodes = 0;
            p.future_checks = 0;
            p.generation += 1;
            p.revisions += u64::from(p.generation > 1);
            p.born_sequence = sequence;
            for node in &mut p.nodes {
                node.kind = InductionNodeKind::Unused;
                node.support = 0;
                node.future_checks = 0;
                for i in node.links() {
                    let l = &mut self.synapses[i];
                    l.weight = 0.0;
                    l.confidence = 0.0;
                    l.phase_offset = 0.0;
                }
            }
            induction_install(draft, p, &mut self.synapses, &sources);
            if let Some(primitives) = state.primitives.as_mut() {
                primitives.record_construction(p, &self.synapses);
            }
        }
        if let Some(primitives) = state.primitives.as_mut() {
            primitives.acquire(
                action,
                sequence,
                &state.config,
                &state.programs[action],
                &mut self.synapses,
            );
        }
    }
}
impl PhaseInductionState {
    fn validate_snapshot(
        &self,
        cfg: &super::EvoConfig,
        sequence: u64,
        cells: &[PhaseCell],
        links: &[PhaseSynapse],
        allocated: &mut std::collections::BTreeSet<usize>,
        used: &mut std::collections::BTreeSet<usize>,
    ) -> bool {
        if !self.config.valid(cfg.sensory_cells, cfg.motor_cells)
            || self.programs.len() != cfg.motor_cells
            || links.len() > MAX_VECTOR_SYNAPSES
        {
            return false;
        }
        let start = cfg.sensory_cells + cfg.motor_cells + 1;
        let mut sources = std::collections::BTreeSet::new();
        if let Some(s) = &self.primitives {
            if !(1..=8).contains(&s.config.capacity)
                || s.operations.len() != s.config.capacity
                || !s.config.policy_learning_rate.is_finite()
                || !(0.001..=0.25).contains(&s.config.policy_learning_rate)
                || (cfg.motor_cells + s.config.capacity) * self.config.nodes_per_motor > 512
                || s.policy_updates > sequence
                || s.priorities.len() != cfg.sensory_cells + s.config.capacity
            {
                return false;
            }
            for (j, &index) in s.priorities.iter().enumerate() {
                if index >= links.len() || !used.insert(index) {
                    return false;
                }
                let source = if j < cfg.sensory_cells {
                    j
                } else {
                    let Some(n) = s.operations[j - cfg.sensory_cells].program.nodes.first() else {
                        return false;
                    };
                    n.cell
                };
                if links[index].from != source
                    || links[index].to != cfg.sensory_cells + cfg.motor_cells
                {
                    return false;
                }
            }
            for op in &s.operations {
                if op.admitted != (op.program.active_nodes > 0)
                    || op.program.generation != u64::from(op.admitted)
                    || op.source_action >= cfg.motor_cells
                    || op.published_sequence > sequence
                    || op.program.born_sequence > op.published_sequence
                    || op.source_generation > op.published_sequence
                    || op.program.future_checks
                        > op.published_sequence
                            .saturating_sub(op.program.born_sequence)
                    || op.source_generation > sequence
                    || (op.admitted
                        && (op.published_sequence == 0
                            || op.source_generation == 0
                            || op.program.future_checks < self.config.min_future_checks))
                    || op.construction_uses > sequence
                    || (!op.admitted
                        && (op.source_generation != 0
                            || op.published_sequence != 0
                            || op.construction_uses != 0))
                    || !op.program.facts.is_empty()
                {
                    return false;
                }
            }
        }
        let main_entries = self
            .programs
            .iter()
            .enumerate()
            .map(|(action, p)| (p, cfg.sensory_cells + action, None));
        let primitive_entries = self
            .primitives
            .iter()
            .flat_map(|s| s.operations.iter().enumerate())
            .map(|(index, op)| {
                (
                    &op.program,
                    op.program.nodes.first().map_or(usize::MAX, |n| n.cell),
                    Some(index),
                )
            });
        for (p, output_cell, primitive_index) in main_entries.chain(primitive_entries) {
            if p.nodes.len() != self.config.nodes_per_motor
                || p.active_nodes > p.nodes.len()
                || (p.active_nodes == 0) != (p.generation == 0)
                || p.generation > sequence
                || p.born_sequence > sequence
                || p.revisions != p.generation.saturating_sub(1)
                || p.future_checks > sequence
                || p.future_checks > sequence.saturating_sub(p.born_sequence)
                || p.discarded_conflicts > sequence
                || p.facts.len() > self.config.facts_per_motor
                || p.last_search_expansions > self.config.search_expansions
            {
                return false;
            }
            for f in &p.facts {
                if f.input.len() != cfg.sensory_cells
                    || f.input
                        .iter()
                        .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
                    || f.sequence == 0
                    || f.sequence > sequence
                    || !sources.insert(f.sequence)
                {
                    return false;
                }
            }
            if p.facts
                .iter()
                .map(|f| f.sequence)
                .collect::<Vec<_>>()
                .windows(2)
                .any(|v| v[0] >= v[1])
            {
                return false;
            }
            for (i, n) in p.nodes.iter().enumerate() {
                if !(start..cells.len()).contains(&n.cell)
                    || !cells[n.cell].recruited
                    || !allocated.insert(n.cell)
                    || n.bounds.len() != cfg.sensory_cells
                    || (i < p.active_nodes) == (n.kind == InductionNodeKind::Unused)
                    || n.future_checks > p.future_checks
                    || n.support > sequence
                {
                    return false;
                }
                for index in n.links() {
                    if index >= links.len() || !used.insert(index) {
                        return false;
                    }
                }
                let test = &links[n.test];
                let response = &links[n.response];
                if test.to != n.cell
                    || response.from != n.cell
                    || response.to != output_cell
                    || !(0.0..=std::f32::consts::PI).contains(&test.phase_offset)
                {
                    return false;
                }
                if n.kind == InductionNodeKind::Branch && test.from >= cfg.sensory_cells {
                    let Some(s) = &self.primitives else {
                        return false;
                    };
                    let limit = primitive_index.unwrap_or(s.operations.len());
                    if !s.operations[..limit].iter().any(|op| {
                        op.admitted
                            && op
                                .program
                                .nodes
                                .first()
                                .is_some_and(|n| n.cell == test.from)
                    }) {
                        return false;
                    }
                }
                for &e in &n.edges {
                    if links[e].from != n.cell || !p.nodes.iter().any(|n| n.cell == links[e].to) {
                        return false;
                    }
                }
                for (j, pair) in n.bounds.iter().enumerate() {
                    let lo = &links[pair[0]];
                    let hi = &links[pair[1]];
                    if lo.from != j
                        || hi.from != j
                        || lo.to != n.cell
                        || hi.to != n.cell
                        || !(0.0..=std::f32::consts::PI).contains(&lo.phase_offset)
                        || !(0.0..=std::f32::consts::PI).contains(&hi.phase_offset)
                        || lo.phase_offset > hi.phase_offset
                    {
                        return false;
                    }
                }
            }
            if p.active_nodes > 0 {
                let mut seen = std::collections::BTreeSet::new();
                let mut pending = vec![(0, 0)];
                let mut checks = 0_u64;
                while let Some((i, depth)) = pending.pop() {
                    if i >= p.active_nodes || depth > self.config.max_depth || !seen.insert(i) {
                        return false;
                    }
                    let n = &p.nodes[i];
                    if n.kind == InductionNodeKind::Branch {
                        for &edge in &n.edges {
                            let Some(child) = p.nodes.iter().position(|n| n.cell == links[edge].to)
                            else {
                                return false;
                            };
                            pending.push((child, depth + 1));
                        }
                    } else {
                        if n.support == 0 {
                            return false;
                        }
                        let Some(sum) = checks.checked_add(n.future_checks) else {
                            return false;
                        };
                        checks = sum;
                    }
                }
                if seen.len() != p.active_nodes || checks != p.future_checks {
                    return false;
                }
            }
        }
        true
    }
}
impl PhaseInductionState {
    fn fingerprint_feed(&self, feed: &mut impl FnMut(u64)) {
        let c = &self.config;
        for n in [
            c.nodes_per_motor,
            c.max_depth,
            c.search_expansions,
            c.facts_per_motor,
            c.min_child_support,
        ] {
            feed(n as u64);
        }
        feed(c.min_future_checks);
        feed(c.min_leaf_checks);
        for x in [c.coverage_radius, c.minimum_outcome, c.minimum_margin] {
            feed(u64::from(x.to_bits()));
        }
        for p in &self.programs {
            for n in [
                p.active_nodes as u64,
                p.generation,
                p.born_sequence,
                p.future_checks,
                p.revisions,
                p.discarded_conflicts,
                p.facts.len() as u64,
                p.last_search_expansions as u64,
            ] {
                feed(n);
            }
            for n in &p.nodes {
                feed(n.cell as u64);
                feed(n.kind as u64);
                feed(n.support);
                feed(n.future_checks);
                for i in n.links() {
                    feed(i as u64);
                }
            }
            for f in &p.facts {
                feed(f.sequence);
                feed(u64::from(f.success));
                for &x in &f.input {
                    feed(u64::from(x.to_bits()));
                }
            }
        }
        if let Some(s) = &self.primitives {
            s.fingerprint_feed(feed);
        }
    }
}
