// INTEL-1 Repair-3. One factual event fans out to every enabled
// representation learner, while the shared parent transition and REAL frame
// are committed exactly once.

impl EvoPhase {
    fn phase_context_sidecar(
        &mut self,
        native: &mut PhaseNativeState,
        pre_cell: usize,
        action: usize,
        after_cell: usize,
    ) -> Option<bool> {
        let Some(mut ctx) = native.contextual.take() else { return Some(false); };
        let learning = native.config.learning_enabled;
        let predecessor = ctx.previous_base;
        let mut born_index = None;

        if learning {
            ctx.factual_events = ctx.factual_events.saturating_add(1);
            if let Some(previous) = predecessor {
                if ctx.candidates.len() < CONTEXT_CANDIDATE_CAP
                    && self.config.structural_growth_enabled
                {
                    if let Some(old) = Self::context_find_novel_collision(
                        &ctx, pre_cell, action, previous, after_cell)
                    {
                        let free = self.dormant_range()
                            .filter(|i| !self.cells[*i].recruited)
                            .take(2).collect::<Vec<_>>();
                        if free.len() == 2 {
                            for &cell in &free { self.cells[cell].recruited = true; }
                            let states = [free[0], free[1]];
                            let predecessors = [old.predecessor, previous];
                            let inputs = [
                                [self.native_synapse(pre_cell, states[0]),
                                 self.native_synapse(predecessors[0], states[0])],
                                [self.native_synapse(pre_cell, states[1]),
                                 self.native_synapse(predecessors[1], states[1])],
                            ];
                            ctx.candidates.push(PhaseContextWitness {
                                base_cell: pre_cell,
                                predecessor_cells: predecessors,
                                successor_cells: [old.post, after_cell],
                                state_cells: states,
                                input_synapses: inputs,
                                anchor_action: action,
                                promoted: false,
                                retired: false,
                                born_fact: ctx.factual_events,
                                eligible_observations: 0,
                                log_evidence: 0.0,
                                context_switches: 0,
                                last_context: None,
                                gate_observations: [0; 2],
                            });
                            born_index = Some(ctx.candidates.len() - 1);
                        }
                    }
                }

                let indices = (0..ctx.candidates.len()).filter(|&index| {
                    Some(index) != born_index
                        && ctx.candidates[index].base_cell == pre_cell
                        && !ctx.candidates[index].retired
                        && ctx.candidates[index].predecessor_cells.contains(&previous)
                }).collect::<Vec<_>>();

                for index in indices {
                    let snapshot = ctx.candidates[index].clone();
                    let Some(side) = snapshot.predecessor_cells.iter()
                        .position(|&p| p == previous) else { continue; };

                    for synapse in snapshot.input_synapses[side] {
                        self.learn_phase_concept_running_mean_synapse(
                            synapse, 1.0, snapshot.gate_observations[side]);
                    }
                    ctx.candidates[index].gate_observations[side] =
                        snapshot.gate_observations[side].saturating_add(1);

                    let accepted = self.native_cell_observation(
                        native,
                        snapshot.state_cells[side],
                        action,
                        after_cell,
                        0.0,
                    ).is_some();
                    if !accepted {
                        ctx.candidates[index].retired = true;
                        continue;
                    }

                    if action == snapshot.anchor_action && !snapshot.promoted {
                        if !snapshot.successor_cells.contains(&after_cell) {
                            ctx.candidates[index].retired = true;
                        } else {
                            let counts = self.context_counts(native, &snapshot);
                            let w = &mut ctx.candidates[index];
                            w.eligible_observations =
                                w.eligible_observations.saturating_add(1);
                            if w.last_context
                                .map(|last| last != side).unwrap_or(false)
                            {
                                w.context_switches =
                                    w.context_switches.saturating_add(1);
                            }
                            w.last_context = Some(side);
                            w.log_evidence = context_log_evidence(counts);
                            w.promoted = context_gate(counts, w.context_switches);
                            if !w.promoted
                                && w.eligible_observations >= CONTEXT_MAX_SAMPLES
                            {
                                w.retired = true;
                            }
                        }
                    }
                }

                if ctx.discovery.len() == CONTEXT_DISCOVERY_CAP {
                    ctx.discovery.remove(0);
                }
                ctx.discovery.push(ContextFact {
                    predecessor: previous,
                    base: pre_cell,
                    action,
                    post: after_cell,
                });
            }
        }

        ctx.previous_base = Some(pre_cell);
        let preserve = ctx.candidates.iter()
            .any(|w| w.base_cell == pre_cell && !w.retired);
        native.contextual = Some(ctx);
        Some(preserve)
    }

    fn phase_perceptual_sidecar(
        &mut self,
        native: &mut PhaseNativeState,
        pre_cell: usize,
        action: usize,
        after_cell: usize,
        raw: &[PhasePerceptFeature],
    ) -> Option<bool> {
        let Some(mut perceptual) = native.perceptual.take() else { return Some(false); };
        let learning = native.config.learning_enabled;
        let mut born_now = false;

        if learning {
            perceptual.factual_events = perceptual.factual_events.saturating_add(1);

            let no_candidate = !perceptual.candidates.iter()
                .any(|w| w.base_cell == pre_cell);
            if no_candidate
                && perceptual.candidates.len() < PERCEPT_CANDIDATE_CAP
                && self.config.structural_growth_enabled
            {
                let opposing = perceptual.discovery.iter().rev().find(|fact|
                    fact.base == pre_cell
                        && fact.action == action
                        && fact.post != after_cell
                ).cloned();
                if let Some(old) = opposing {
                    if let Some(feature_pair) =
                        Self::percept_candidate_pair(&old.features, raw)
                    {
                        let free = self.dormant_range()
                            .filter(|i| !self.cells[*i].recruited)
                            .take(4).collect::<Vec<_>>();
                        if free.len() == 4 {
                            for &cell in &free { self.cells[cell].recruited = true; }
                            let feature_cells = [free[0], free[1]];
                            let states = [free[2], free[3]];
                            let inputs = [
                                [self.native_synapse(pre_cell, states[0]),
                                 self.native_synapse(feature_cells[0], states[0])],
                                [self.native_synapse(pre_cell, states[1]),
                                 self.native_synapse(feature_cells[1], states[1])],
                            ];
                            perceptual.candidates.push(PhasePerceptWitness {
                                base_cell: pre_cell,
                                features: feature_pair,
                                feature_cells,
                                successor_cells: [old.post, after_cell],
                                state_cells: states,
                                input_synapses: inputs,
                                anchor_action: action,
                                promoted: false,
                                retired: false,
                                born_fact: perceptual.factual_events,
                                eligible_observations: 0,
                                log_evidence: 0.0,
                                side_switches: 0,
                                last_side: None,
                                gate_observations: [0; 2],
                            });
                            born_now = true;
                        }
                    }
                }
            }

            if !born_now {
                if let Some(index) = perceptual.candidates.iter()
                    .position(|w| w.base_cell == pre_cell && !w.retired)
                {
                    let snapshot = perceptual.candidates[index].clone();
                    if let Some(side) = Self::percept_side(raw, &snapshot) {
                        for synapse in snapshot.input_synapses[side] {
                            self.learn_phase_concept_running_mean_synapse(
                                synapse, 1.0, snapshot.gate_observations[side]);
                        }
                        perceptual.candidates[index].gate_observations[side] =
                            snapshot.gate_observations[side].saturating_add(1);

                        let accepted = self.native_cell_observation(
                            native,
                            snapshot.state_cells[side],
                            action,
                            after_cell,
                            0.0,
                        ).is_some();
                        if !accepted { perceptual.candidates[index].retired = true; }

                        if action == snapshot.anchor_action && !snapshot.promoted {
                            if !snapshot.successor_cells.contains(&after_cell) {
                                perceptual.candidates[index].retired = true;
                            } else if accepted {
                                let counts = self.percept_counts(native, &snapshot);
                                let w = &mut perceptual.candidates[index];
                                w.eligible_observations =
                                    w.eligible_observations.saturating_add(1);
                                if w.last_side
                                    .map(|last| last != side).unwrap_or(false)
                                {
                                    w.side_switches = w.side_switches.saturating_add(1);
                                }
                                w.last_side = Some(side);
                                w.log_evidence = context_log_evidence(counts);
                                w.promoted = context_gate(counts, w.side_switches);
                                if !w.promoted
                                    && w.eligible_observations >= CONTEXT_MAX_SAMPLES
                                {
                                    w.retired = true;
                                }
                            }
                        }
                    }
                }
            }

            if perceptual.discovery.len() == PERCEPT_DISCOVERY_CAP {
                perceptual.discovery.remove(0);
            }
            perceptual.discovery.push(PerceptFact {
                base: pre_cell,
                action,
                post: after_cell,
                features: raw.to_vec(),
            });
        }

        let preserve = perceptual.candidates.iter()
            .any(|w| w.base_cell == pre_cell && !w.retired);
        native.perceptual = Some(perceptual);
        Some(preserve)
    }

    fn phase_compositional_sidecar(
        &mut self,
        native: &mut PhaseNativeState,
        pre_cell: usize,
        action: usize,
        after_cell: usize,
        raw: &[PhasePerceptFeature],
    ) -> Option<bool> {
        let Some(mut comp) = native.compositional.take() else { return Some(false); };
        let learning = native.config.learning_enabled;

        if learning {
            comp.factual_events = comp.factual_events.saturating_add(1);

            let no_base_candidates = !comp.candidates.iter()
                .any(|w| w.base_cell == pre_cell);
            if no_base_candidates && self.config.structural_growth_enabled {
                let opposing = comp.discovery.iter().rev().find(|fact|
                    fact.base == pre_cell
                        && fact.action == action
                        && fact.post != after_cell
                ).cloned();
                if let Some(old) = opposing {
                    let programs = Self::candidate_programs(&old.features, raw);
                    for program in programs {
                        if comp.candidates.len() >= COMPOSITION_CANDIDATE_CAP { break; }
                        let free = self.dormant_range()
                            .filter(|i| !self.cells[*i].recruited)
                            .take(4).collect::<Vec<_>>();
                        if free.len() != 4 { break; }
                        for &cell in &free { self.cells[cell].recruited = true; }
                        let program_cells = [free[0], free[1]];
                        let states = [free[2], free[3]];
                        let inputs = [
                            [self.native_synapse(pre_cell, states[0]),
                             self.native_synapse(program_cells[0], states[0])],
                            [self.native_synapse(pre_cell, states[1]),
                             self.native_synapse(program_cells[1], states[1])],
                        ];
                        comp.candidates.push(PhaseCompositionWitness {
                            base_cell: pre_cell,
                            program,
                            program_cells,
                            successor_cells: [old.post, after_cell],
                            state_cells: states,
                            input_synapses: inputs,
                            anchor_action: action,
                            promoted: false,
                            retired: false,
                            born_fact: comp.factual_events,
                            eligible_observations: 0,
                            log_evidence: 0.0,
                            side_switches: 0,
                            atom_effects: [0.0; 2],
                            last_side: None,
                            gate_observations: [0; 2],
                            atom_counts: [[[0; 2]; 2]; 2],
                        });
                    }
                }
            } else {
                for index in 0..comp.candidates.len() {
                    if comp.candidates[index].base_cell != pre_cell
                        || comp.candidates[index].retired
                    {
                        continue;
                    }
                    let snapshot = comp.candidates[index].clone();
                    let side = usize::from(snapshot.program.eval(raw));
                    for synapse in snapshot.input_synapses[side] {
                        self.learn_phase_concept_running_mean_synapse(
                            synapse, 1.0, snapshot.gate_observations[side]);
                    }
                    comp.candidates[index].gate_observations[side] =
                        snapshot.gate_observations[side].saturating_add(1);

                    let accepted = self.native_cell_observation(
                        native,
                        snapshot.state_cells[side],
                        action,
                        after_cell,
                        0.0,
                    ).is_some();
                    if !accepted {
                        comp.candidates[index].retired = true;
                        continue;
                    }

                    if action == snapshot.anchor_action && !snapshot.promoted {
                        let outcome = snapshot.successor_cells.iter()
                            .position(|&cell| cell == after_cell);
                        let Some(outcome) = outcome else {
                            comp.candidates[index].retired = true;
                            continue;
                        };

                        let atoms = snapshot.program.atoms();
                        for atom_index in 0..2 {
                            let present = usize::from(
                                raw.binary_search(&atoms[atom_index]).is_ok());
                            comp.candidates[index]
                                .atom_counts[atom_index][present][outcome] += 1;
                        }

                        let counts = self.composition_counts(native, &snapshot);
                        let w = &mut comp.candidates[index];
                        w.eligible_observations =
                            w.eligible_observations.saturating_add(1);
                        if w.last_side.map(|last| last != side).unwrap_or(false) {
                            w.side_switches = w.side_switches.saturating_add(1);
                        }
                        w.last_side = Some(side);
                        w.log_evidence = context_log_evidence(counts);
                        w.atom_effects = Self::atom_effects(w);
                        w.promoted = Self::composition_gate(w, counts);
                        if !w.promoted
                            && w.eligible_observations >= CONTEXT_MAX_SAMPLES
                        {
                            w.retired = true;
                        }
                    }
                }
            }

            if comp.discovery.len() == COMPOSITION_DISCOVERY_CAP {
                comp.discovery.remove(0);
            }
            comp.discovery.push(CompositionFact {
                base: pre_cell,
                action,
                post: after_cell,
                features: raw.to_vec(),
            });
        }

        let preserve = comp.candidates.iter()
            .any(|w| w.base_cell == pre_cell && !w.retired);
        native.compositional = Some(comp);
        Some(preserve)
    }

    /// Consume one external factual action/result exactly once while allowing
    /// every enabled representation learner to inspect the same fact.
    pub fn observe_phase_native_refinement_fanout_result(
        &mut self,
        action: usize,
        post: &[f32],
    ) -> Option<usize> {
        if action >= self.config.motor_cells { return None; }

        let pre_sensory = self.current_real.as_ref()?.sensory.clone();
        let pre = self.phase_native_abstract_state(&pre_sensory)?;
        let after = self.phase_native_abstract_state(post)?;
        if pre.level != after.level { return None; }
        let raw = self.phase_raw_features(&pre_sensory)?;

        let mut native = self.phase_native.take()?;
        let learning = native.config.learning_enabled;

        let context_preserve =
            self.phase_context_sidecar(&mut native, pre.cell, action, after.cell)?;
        let perceptual_preserve =
            self.phase_perceptual_sidecar(&mut native, pre.cell, action, after.cell, &raw)?;
        let compositional_preserve =
            self.phase_compositional_sidecar(&mut native, pre.cell, action, after.cell, &raw)?;

        self.phase_native = Some(native);

        if !learning {
            self.observe_initial_real(post, false);
            return Some(0);
        }

        let preserve_rivals =
            context_preserve || perceptual_preserve || compositional_preserve;

        if preserve_rivals {
            if !self.observe_phase_native_abstract_transition(
                &pre_sensory, action, post, 0.0)
            {
                return None;
            }
            self.observe_initial_real(post, false);
            Some(0)
        } else {
            self.observe_phase_native_rival_probe_result(action, post)
        }
    }
}
