// Bounded inverse constraint propagation. Only factual permitted POST supplies
// equations; inferred PRE/POST remains IMAGINED and never becomes tuition.

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseInverseReport {
    pub action: usize,
    pub action_revision: u64,
    pub evidence_sources: Vec<u64>,
    pub observed_post: PhasePartialVector,
    pub pre_hypotheses: Vec<PhasePartialVector>,
    pub authority: Authority,
    pub widened: bool,
    pub contradicted: bool,
}

impl EvoPhase {
    /// Opt-in inverse inference for exact circular rules and partial sensors.
    /// Changing the setting clears the current episode, requiring a new frame.
    pub fn set_phase_inverse_inference(&mut self, enabled: bool) -> bool {
        let Some(partial) = self.phase_native.as_mut().and_then(|s| s.partial.as_mut()) else {
            return false;
        };
        if partial.inverse_enabled != enabled {
            partial.inverse_enabled = enabled;
            partial.episode = None;
            self.current_real = None;
        }
        true
    }

    pub fn phase_inverse_inference_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|s| s.partial.as_ref())
            .is_some_and(|p| p.inverse_enabled)
    }

    fn partial_observed_transition(
        &self,
        action: usize,
        input: &[PhasePartialVector],
        post: &[Option<f32>],
    ) -> Option<(Vec<PhasePartialVector>, bool, Option<PhaseInverseReport>)> {
        let native = self.phase_native.as_ref()?;
        let partial = native.partial.as_ref()?;
        let info = self.phase_rule_action_info(action)?;
        if !partial.inverse_enabled
            || !info.confirmed
            || partial.episode.as_ref()?.invalid_rules[action]
        {
            let (rows, widened) = self.partial_transition(action, input)?;
            return Some((rows, widened, None));
        }
        let rules = native.rules.as_ref()?;
        let model = rules.actions.get(action)?;
        let tolerance = rules.config.tolerance;
        let max = partial.config.max_hypotheses;
        let mut rows = input.to_vec();
        let mut widened = false;
        // At most `width` passes: each productive equation fills a missing
        // operand. Relations with two unknown operands remain symbolic unknown.
        for _ in 0..self.config.sensory_cells {
            let before = rows.clone();
            for (output, observed) in model.outputs.iter().zip(post) {
                let Some(observed) = observed else { continue };
                let candidate = output.candidates.iter().find(|c| c.active)?;
                // This reads both physical links. A cut link has no fallback.
                let phase = rule_candidate_phase(candidate, &self.cells, &self.synapses)?;
                let formula = candidate.formula();
                let offset = f64::from(phase) / f64::from(std::f32::consts::TAU);
                let mut refined = Vec::new();
                for row in &rows {
                    let unknown = formula
                        .terms
                        .iter()
                        .filter(|t| row[t.source].is_none())
                        .collect::<Vec<_>>();
                    if unknown.is_empty() {
                        if rule_candidate_partial_output(
                            candidate,
                            row,
                            &self.cells,
                            &self.synapses,
                        )
                        .is_some_and(|x| rule_distance(&[x], &[*observed]) <= tolerance)
                        {
                            refined.push(row.clone());
                        }
                    } else if unknown.len() == 1 {
                        let term = unknown[0];
                        let known = formula
                            .terms
                            .iter()
                            .filter_map(|t| {
                                row[t.source].map(|x| f64::from(t.coefficient) * f64::from(x))
                            })
                            .sum::<f64>();
                        let rhs = (f64::from(*observed) - offset - known).rem_euclid(1.0);
                        let gain = term.coefficient.unsigned_abs();
                        for k in 0..gain {
                            let root = (f64::from(term.coefficient.signum())
                                * (rhs + f64::from(k))
                                / f64::from(gain))
                            .rem_euclid(1.0);
                            let mut branch = row.clone();
                            branch[term.source] = Some(rule_canonical_value(root as f32));
                            if rule_candidate_partial_output(
                                candidate,
                                &branch,
                                &self.cells,
                                &self.synapses,
                            )
                            .is_some_and(|x| rule_distance(&[x], &[*observed]) <= tolerance)
                            {
                                refined.push(branch);
                            }
                        }
                    } else {
                        // No finite set can represent x+y=c with both unknown.
                        refined.push(row.clone());
                    }
                    if refined.len() > max {
                        refined = vec![partial_hull(&refined, tolerance)];
                        widened = true;
                    }
                }
                // Deduplication only merges equal alternatives; never select
                // an arbitrary root to satisfy a goal or capacity constraint.
                let mut unique = Vec::new();
                for row in refined {
                    if !unique.contains(&row) {
                        unique.push(row);
                    }
                }
                rows = unique;
            }
            if rows == before || rows.is_empty() {
                break;
            }
        }
        let report = PhaseInverseReport {
            action,
            action_revision: info.revision,
            evidence_sources: info.evidence_sources,
            observed_post: post.to_vec(),
            pre_hypotheses: rows.clone(),
            authority: Authority::Imagined,
            widened,
            contradicted: rows.is_empty(),
        };
        if rows.is_empty() {
            return Some((Vec::new(), widened, Some(report)));
        }
        let (predicted, forward_widened) = self.partial_transition(action, &rows)?;
        Some((predicted, widened || forward_widened, Some(report)))
    }
}
