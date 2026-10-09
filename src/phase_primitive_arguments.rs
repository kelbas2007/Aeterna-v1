// Argument transfer for the EXISTING acquired-operation interpreter.
// Included by phase_primitives.rs. No named target function is defined here.
// Binding search is inherited, bounded software over factual experience; it
// is NOT claimed to be an autonomously invented phase-native constructor.
// Call views are reconstructed from the bounded factual buffers. Definitions
// and their physical synapses are never cloned, rewritten or refitted.

#[derive(Debug, Clone)]
struct PrimitiveArgumentMatch {
    operation_index: usize,
    source_inputs: Vec<usize>,
    arguments: Vec<usize>,
    fit_facts: usize,
    future_checks: usize,
}

fn primitive_argument_sources(
    index: usize,
    state: &PhasePrimitiveState,
    width: usize,
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
) -> Option<Vec<usize>> {
    let op = state.operations.get(index)?;
    if !op.admitted || op.program.active_nodes == 0 {
        return None;
    }
    let mut used = std::collections::BTreeSet::new();
    for node in &op.program.nodes[..op.program.active_nodes] {
        if node.kind != InductionNodeKind::Branch {
            continue;
        }
        let test = links.get(node.test)?;
        if test.weight < 0.5 || test.confidence < 0.5 || !cells.get(test.from)?.recruited {
            return None;
        }
        if test.from < width {
            used.insert(test.from);
        } else {
            // Published operations can only depend on earlier definitions.
            let child = state.operations[..index]
                .iter()
                .position(|p| p.admitted && p.program.nodes[0].cell == test.from)?;
            used.extend(primitive_argument_sources(
                child, state, width, cells, links,
            )?);
        }
    }
    if used.is_empty() || used.len() > 3 {
        return None;
    }
    Some(used.into_iter().collect())
}

fn primitive_argument_read(
    binding: &PrimitiveArgumentMatch,
    state: &PhasePrimitiveState,
    config: &PhaseInductionConfig,
    input: &[f32],
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
) -> Option<(f32, usize)> {
    if binding.source_inputs.len() != binding.arguments.len() {
        return None;
    }
    let op = state.operations.get(binding.operation_index)?;
    if !op.admitted {
        return None;
    }
    let mut projected = input.to_vec();
    for (&source, &argument) in binding.source_inputs.iter().zip(&binding.arguments) {
        *projected.get_mut(source)? = *input.get(argument)?;
    }
    // Use the very same physical definition and ALL existing support guards.
    // No Boolean shortcuts, teacher labels, leaf refitting or prototype fallback.
    induction_read_inner(
        &op.program,
        config,
        &projected,
        cells,
        links,
        Some(state),
        binding.operation_index,
    )
}

fn primitive_argument_permutations(width: usize, arity: usize) -> Vec<Vec<usize>> {
    fn visit(width: usize, arity: usize, prefix: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if prefix.len() == arity {
            out.push(prefix.clone());
            return;
        }
        for next in 0..width {
            if prefix.contains(&next) {
                continue;
            }
            prefix.push(next);
            visit(width, arity, prefix, out);
            prefix.pop();
        }
    }
    let mut result = Vec::new();
    if width <= 8 && (1..=3).contains(&arity) && arity <= width {
        visit(width, arity, &mut Vec::new(), &mut result);
    }
    result
}

fn primitive_argument_match(
    facts: &std::collections::VecDeque<InductionFact>,
    state: &PhasePrimitiveState,
    config: &PhaseInductionConfig,
    cells: &[PhaseCell],
    links: &[PhaseSynapse],
) -> Option<PrimitiveArgumentMatch> {
    if !state.argument_transfer_enabled {
        return None;
    }
    let width = facts.front()?.input.len();
    if width > 8 {
        return None;
    }
    let future = config.min_future_checks as usize;
    let required_fit = config.min_child_support * 2;
    let mut budget = 32768usize;
    let mut best: Option<PrimitiveArgumentMatch> = None;
    let mut best_fit = 0usize;
    for (operation_index, op) in state.operations.iter().enumerate() {
        if !op.admitted {
            continue;
        }
        let Some(source_inputs) =
            primitive_argument_sources(operation_index, state, width, cells, links)
        else {
            continue;
        };
        // A definition may be supported only by facts observed after it was
        // published. Source-task labels are never recycled as target facts.
        let newer = facts
            .iter()
            .filter(|f| f.sequence > op.published_sequence)
            .collect::<Vec<_>>();
        if newer.len() < required_fit + future {
            continue;
        }
        let split = newer.len() - future;
        let fit = &newer[..split];
        for arguments in primitive_argument_permutations(width, source_inputs.len()) {
            let mut candidate = PrimitiveArgumentMatch {
                operation_index,
                source_inputs: source_inputs.clone(),
                arguments,
                fit_facts: 0,
                future_checks: future,
            };
            // A factual contradiction revokes support for this binding, not
            // the immutable underlying operation. Fit a recent consistent
            // segment using the PREFIX ONLY; later outcomes cannot select it.
            let mut start = 0usize;
            for (i, fact) in fit.iter().enumerate() {
                if budget == 0 {
                    return None;
                }
                budget -= 1;
                if !primitive_argument_read(&candidate, state, config, &fact.input, cells, links)
                    .is_some_and(|(v, _)| {
                        (v - f32::from(fact.success)).abs() <= 1.0 - config.minimum_outcome
                    })
                {
                    start = i + 1;
                }
            }
            let supported = &fit[start..];
            if supported.len() < required_fit || supported.len() <= best_fit {
                continue;
            }
            // A successful policy need not deliberately produce failures.
            // Instead require factual variation in EVERY selected argument,
            // with replicated observations on both sides of that variation.
            // Merely replaying one successful input cannot admit a binding.
            let diverse = candidate.arguments.iter().all(|&arg| {
                let lo = supported
                    .iter()
                    .map(|f| f.input[arg])
                    .fold(f32::INFINITY, f32::min);
                let hi = supported
                    .iter()
                    .map(|f| f.input[arg])
                    .fold(f32::NEG_INFINITY, f32::max);
                if !lo.is_finite() || !hi.is_finite() || hi - lo <= 1.0e-6 {
                    return false;
                }
                let middle = lo + (hi - lo) * 0.5;
                supported.iter().filter(|f| f.input[arg] <= middle).count()
                    >= config.min_child_support
                    && supported.iter().filter(|f| f.input[arg] > middle).count()
                        >= config.min_child_support
            });
            if !diverse {
                continue;
            }
            candidate.fit_facts = supported.len();
            best_fit = supported.len();
            best = Some(candidate);
        }
    }
    // Freeze the best prefix-supported candidate. A failing later suffix
    // returns no call; it does not choose another candidate using those facts.
    let chosen = best?;
    let op = &state.operations[chosen.operation_index];
    let newer = facts
        .iter()
        .filter(|f| f.sequence > op.published_sequence)
        .collect::<Vec<_>>();
    let suffix = &newer[newer.len() - future..];
    for fact in suffix {
        if budget == 0 {
            return None;
        }
        budget -= 1;
        if !primitive_argument_read(&chosen, state, config, &fact.input, cells, links).is_some_and(
            |(v, _)| (v - f32::from(fact.success)).abs() <= 1.0 - config.minimum_outcome,
        ) {
            return None;
        }
    }
    Some(chosen)
}

impl EvoPhase {
    /// Opt-in extension of existing primitive calls, not a new cognitive mode.
    /// The current implementation has up to three arguments and eight channels.
    /// Operation definitions remain physical; binding selection is software.
    pub fn set_phase_primitive_argument_transfer(&mut self, enabled: bool) -> bool {
        if enabled && self.config.sensory_cells > 8 {
            return false;
        }
        let Some(state) = self
            .phase_native
            .as_mut()
            .and_then(|n| n.vector.as_mut())
            .and_then(|v| v.induction.as_mut())
            .and_then(|s| s.primitives.as_mut())
        else {
            return false;
        };
        state.argument_transfer_enabled = enabled;
        true
    }

    pub fn phase_primitive_argument_sources(&self, output_cell: usize) -> Option<Vec<usize>> {
        let state = self
            .phase_native
            .as_ref()?
            .vector
            .as_ref()?
            .induction
            .as_ref()?;
        let primitives = state.primitives.as_ref()?;
        let index = primitives
            .operations
            .iter()
            .position(|op| op.admitted && op.program.nodes[0].cell == output_cell)?;
        primitive_argument_sources(
            index,
            primitives,
            self.config.sensory_cells,
            &self.cells,
            &self.synapses,
        )
    }

    /// Explicit parameterized call, useful for read-only mechanism audits.
    /// Normal action prediction infers arguments from its own factual buffers.
    pub fn phase_primitive_bound_value(
        &self,
        output_cell: usize,
        frame: &[Option<f32>],
        arguments: &[usize],
    ) -> Option<f32> {
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
        let primitives = state.primitives.as_ref()?;
        let index = primitives
            .operations
            .iter()
            .position(|op| op.admitted && op.program.nodes[0].cell == output_cell)?;
        let source_inputs = primitive_argument_sources(
            index,
            primitives,
            input.len(),
            &self.cells,
            &self.synapses,
        )?;
        if source_inputs.len() != arguments.len() || arguments.iter().any(|&a| a >= input.len()) {
            return None;
        }
        primitive_argument_read(
            &PrimitiveArgumentMatch {
                operation_index: index,
                source_inputs,
                arguments: arguments.to_vec(),
                fit_facts: 0,
                future_checks: 0,
            },
            primitives,
            &state.config,
            &input,
            &self.cells,
            &self.synapses,
        )
        .map(|v| v.0)
    }

    /// (action, original definition output, selected arguments, fitting facts,
    /// later validation facts). Diagnostics never alter any retained state.
    pub fn phase_primitive_argument_bindings(
        &self,
    ) -> Vec<(usize, usize, Vec<usize>, usize, usize)> {
        let Some(state) = self
            .phase_native
            .as_ref()
            .and_then(|n| n.vector.as_ref())
            .and_then(|v| v.induction.as_ref())
        else {
            return Vec::new();
        };
        let Some(primitives) = state.primitives.as_ref() else {
            return Vec::new();
        };
        state
            .programs
            .iter()
            .enumerate()
            .filter_map(|(action, p)| {
                let call = primitive_argument_match(
                    &p.facts,
                    primitives,
                    &state.config,
                    &self.cells,
                    &self.synapses,
                )?;
                Some((
                    action,
                    primitives.operations[call.operation_index].program.nodes[0].cell,
                    call.arguments,
                    call.fit_facts,
                    call.future_checks,
                ))
            })
            .collect()
    }

    pub fn phase_primitive_argument_prediction(
        &self,
        frame: &[Option<f32>],
    ) -> Option<PhaseInductionPrediction> {
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
        let primitives = state.primitives.as_ref()?;
        if !primitives.argument_transfer_enabled {
            return None;
        }
        let mut candidates = Vec::new();
        for (action, p) in state.programs.iter().enumerate() {
            let Some(call) = primitive_argument_match(
                &p.facts,
                primitives,
                &state.config,
                &self.cells,
                &self.synapses,
            ) else {
                continue;
            };
            let Some((value, leaf)) = primitive_argument_read(
                &call,
                primitives,
                &state.config,
                &input,
                &self.cells,
                &self.synapses,
            ) else {
                continue;
            };
            candidates.push((value, action, call.operation_index, leaf));
        }
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let &(value, action, index, leaf) = candidates.first()?;
        let margin = value - candidates.get(1).map_or(0.0, |c| c.0);
        if value < state.config.minimum_outcome || margin < state.config.minimum_margin {
            return None;
        }
        let op = &primitives.operations[index];
        Some(PhaseInductionPrediction {
            action,
            expected_outcome: value,
            margin,
            root_cell: op.program.nodes[0].cell,
            leaf_cell: op.program.nodes[leaf].cell,
            authority: Authority::Imagined,
        })
    }
}
