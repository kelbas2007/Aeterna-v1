// Resource-bounded planning over circular interval boxes. A plan is supported
// only when every alternative reaches a visible goal within the error bound.

#[derive(Clone)]
struct UncertainSearchNode {
    hypotheses: Vec<PhaseUncertainVector>,
    first: Option<usize>,
}

fn uncertain_exploration_rank(sequence: u64, action: usize) -> u64 {
    // Reproducible mixing breaks synchronization between a cyclic motor order
    // and a cyclic stream of examples. The factual sequence is checkpointed;
    // no world label, outcome prediction or motor role enters this tie-break.
    let mut value = sequence.wrapping_add((action as u64).wrapping_mul(0x9E3779B97F4A7C15));
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
    value ^ (value >> 31)
}

fn uncertain_goal_distance(rows: &[PhaseUncertainVector], goal: &[Option<f32>]) -> f32 {
    rows.iter()
        .flat_map(|row| {
            row.iter().zip(goal).filter_map(|(value, target)| {
                target.map(|target| {
                    value.map_or(0.5, |v| rule_distance(&[v.center], &[target]) + v.radius)
                })
            })
        })
        .fold(0.0, f32::max)
}

impl EvoPhase {
    fn uncertain_action_supported(&self, action: usize) -> bool {
        let Some(partial) = self.phase_native.as_ref().and_then(|n| n.partial.as_ref()) else {
            return false;
        };
        let Some(episode) = &partial.episode else {
            return false;
        };
        self.uncertain_model(action).is_some()
            && partial.masks[action].source_ids.len() >= partial.config.min_mask_support
            && !episode.invalid_rules[action]
            && !episode.invalid_masks[action]
    }

    fn uncertain_plan(&self, goal: &[Option<f32>]) -> Option<PhasePartialDecision> {
        let native = self.phase_native.as_ref()?;
        let rules = &native.rules.as_ref()?.config;
        let partial = native.partial.as_ref()?;
        let cfg = partial.uncertainty.as_ref()?;
        let tolerance = native
            .rules
            .as_ref()?
            .adaptive
            .as_ref()?
            .config
            .goal_tolerance;
        let initial = &partial.episode.as_ref()?.uncertain.as_ref()?.hypotheses;
        let supported = (0..self.config.motor_cells)
            .filter(|&a| self.uncertain_action_supported(a))
            .collect::<Vec<_>>();
        // Search deduplication includes radii. No quantization can turn a wide
        // interval into a narrow one or delete a modular alternative.
        let key = |rows: &[PhaseUncertainVector]| {
            let mut k = rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|v| v.map(|v| (v.center.to_bits(), v.radius.to_bits())))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            k.sort();
            k.dedup();
            k
        };
        let mut visited = std::collections::BTreeSet::new();
        visited.insert(key(initial));
        let mut layer = vec![UncertainSearchNode {
            hypotheses: initial.clone(),
            first: None,
        }];
        let mut evaluations = 0;
        for depth in 1..=rules.planning_depth.min(native.config.horizon) {
            let mut next = Vec::new();
            for node in &layer {
                for &action in &supported {
                    if evaluations >= rules.node_budget {
                        return None;
                    }
                    evaluations += 1;
                    let mut budget = cfg.inference_budget;
                    let (rows, _) =
                        self.uncertain_transition(action, &node.hypotheses, &mut budget)?;
                    if !rows.is_empty()
                        && goal
                            .iter()
                            .zip(&partial.masks[action].visible)
                            .all(|(g, &v)| g.is_none() || v)
                        && rows
                            .iter()
                            .all(|r| uncertain_goal_matches(r, goal, tolerance))
                    {
                        return Some(PhasePartialDecision {
                            action: node.first.unwrap_or(action),
                            kind: PhasePartialDecisionKind::GoalPlan,
                            planned_depth: depth,
                            expected_information: 0,
                        });
                    }
                    if visited.insert(key(&rows)) {
                        next.push(UncertainSearchNode {
                            hypotheses: rows,
                            first: Some(node.first.unwrap_or(action)),
                        });
                    }
                }
            }
            next.sort_by(|a, b| {
                uncertain_goal_distance(&a.hypotheses, goal)
                    .total_cmp(&uncertain_goal_distance(&b.hypotheses, goal))
                    .then_with(|| a.first.cmp(&b.first))
            });
            next.truncate(rules.beam_width);
            if next.is_empty() {
                return None;
            }
            layer = next;
        }
        None
    }

    fn uncertain_relevant_fields(&self, goal: &[Option<f32>]) -> Vec<bool> {
        let mut relevant = goal.iter().map(Option::is_some).collect::<Vec<_>>();
        for _ in 0..self.config.sensory_cells {
            let old = relevant.clone();
            for action in 0..self.config.motor_cells {
                let Some((a, model)) = self.uncertain_model(action) else {
                    continue;
                };
                for &slot in &model.slots {
                    for (j, output) in a.slots[slot].outputs.iter().enumerate() {
                        if !old[j] {
                            continue;
                        }
                        if let Some(condition) = &model.condition {
                            relevant[condition.source] = true;
                        }
                        for term in output
                            .candidates
                            .iter()
                            .find(|c| c.active)
                            .unwrap()
                            .formula()
                            .terms
                        {
                            relevant[term.source] = true;
                        }
                    }
                }
            }
            if old == relevant {
                break;
            }
        }
        relevant
    }

    fn uncertain_preserves_goal_state(&self, action: usize, goal: &[Option<f32>]) -> bool {
        let Some((a, model)) = self.uncertain_model(action) else {
            return false;
        };
        model.slots.iter().all(|&slot| {
            goal.iter().enumerate().all(|(j, g)| {
                if g.is_none() {
                    return true;
                }
                let candidate = a.slots[slot].outputs[j]
                    .candidates
                    .iter()
                    .find(|c| c.active)
                    .unwrap();
                let f = candidate.formula();
                f.terms.len() == 1
                    && f.terms[0].source == j
                    && f.terms[0].coefficient == 1
                    && rule_candidate_phase(candidate, &self.cells, &self.synapses).is_some_and(
                        |phase| {
                            rule_distance(&[phase / std::f32::consts::TAU], &[0.0])
                                <= self.uncertain_model_error(candidate)
                        },
                    )
            })
        })
    }

    fn uncertain_information(&self, action: usize, relevant: &[bool]) -> Option<f32> {
        let partial = self.phase_native.as_ref()?.partial.as_ref()?;
        let cfg = partial.uncertainty.as_ref()?;
        let episode = partial.episode.as_ref()?.uncertain.as_ref()?;
        if episode.unproductive[action] {
            return Some(0.0);
        }
        let mut budget = cfg.inference_budget;
        let (post, _) = self.uncertain_transition(action, &episode.hypotheses, &mut budget)?;
        let mut score = 0.0;
        for (j, &visible) in partial.masks[action].visible.iter().enumerate() {
            if visible && relevant[j] {
                // A propagated model error is not fresh information when this
                // channel has already been measured at the current precision.
                let before = uncertain_field_coverage(&episode.hypotheses, j);
                score += (before.min(uncertain_field_coverage(&post, j)) - cfg.measurement_radius)
                    .max(0.0);
            }
        }
        // An output can measure a hidden operand indirectly. Require all
        // other operands to be bounded; a lone sum of two unknowns is not a
        // complete measurement of either operand.
        let (a, model) = self.uncertain_model(action)?;
        for &slot in &model.slots {
            for (j, output) in a.slots[slot].outputs.iter().enumerate() {
                if !partial.masks[action].visible[j] {
                    continue;
                }
                let candidate = output.candidates.iter().find(|c| c.active)?;
                let formula = candidate.formula();
                for term in &formula.terms {
                    if !relevant[term.source] {
                        continue;
                    }
                    let mut guaranteed = f32::MAX;
                    for row in &episode.hypotheses {
                        let mut error =
                            cfg.measurement_radius + self.uncertain_model_error(candidate);
                        let mut supported = true;
                        for other in formula.terms.iter().filter(|t| t.source != term.source) {
                            if let Some(v) = row[other.source] {
                                error += f32::from(other.coefficient.unsigned_abs()) * v.radius;
                            } else {
                                supported = false;
                                break;
                            }
                        }
                        guaranteed = guaranteed.min(if supported {
                            (uncertain_field_coverage(&episode.hypotheses, term.source) - error)
                                .max(0.0)
                        } else {
                            0.0
                        });
                    }
                    if guaranteed != f32::MAX {
                        score += guaranteed;
                    }
                }
            }
        }
        Some(score)
    }

    fn phase_uncertain_decision(&self, goal: &[Option<f32>]) -> Option<PhasePartialDecision> {
        if !partial_vector_valid(goal, self.config.sensory_cells)
            || !goal.iter().any(Option::is_some)
            || self.phase_uncertain_goal_reached(goal)
        {
            return None;
        }
        let native = self.phase_native.as_ref()?;
        let partial = native.partial.as_ref()?;
        let cfg = partial.uncertainty.as_ref()?;
        partial.episode.as_ref()?.uncertain.as_ref()?;
        if native.config.learning_enabled {
            if let Some(action) = (0..self.config.motor_cells)
                .filter(|&a| partial.masks[a].observations < cfg.exploration_observations as u64)
                .min_by_key(|&a| {
                    (
                        partial.masks[a].observations,
                        uncertain_exploration_rank(partial.factual_sequence, a),
                    )
                })
            {
                return Some(PhasePartialDecision {
                    action,
                    kind: PhasePartialDecisionKind::MaskExploration,
                    planned_depth: 1,
                    expected_information: 0,
                });
            }
        }
        if let Some(plan) = self.uncertain_plan(goal) {
            return Some(plan);
        }
        if native.config.learning_enabled {
            if let Some(action) = (0..self.config.motor_cells)
                .filter(|&a| !self.uncertain_action_supported(a))
                .min_by_key(|&a| (partial.masks[a].observations, a))
            {
                return Some(PhasePartialDecision {
                    action,
                    kind: PhasePartialDecisionKind::MaskExploration,
                    planned_depth: 1,
                    expected_information: 0,
                });
            }
        }
        let relevant = self.uncertain_relevant_fields(goal);
        let mut best: Option<(usize, bool, f32)> = None;
        for action in 0..self.config.motor_cells {
            if !self.uncertain_action_supported(action) {
                continue;
            }
            let information = self.uncertain_information(action, &relevant)?;
            if information <= cfg.information_epsilon {
                continue;
            }
            let preserves = self.uncertain_preserves_goal_state(action, goal);
            if best.is_none_or(|(_, p, score)| {
                (preserves && !p) || (preserves == p && information > score)
            }) {
                best = Some((action, preserves, information));
            }
        }
        if let Some((action, _, score)) = best {
            return Some(PhasePartialDecision {
                action,
                kind: PhasePartialDecisionKind::InformationGathering,
                planned_depth: 1,
                expected_information: (score / cfg.information_epsilon).ceil() as usize,
            });
        }
        // Changed laws and unsupported condition gaps need new factual tuition.
        // This fallback remains disabled in frozen evaluation.
        native
            .config
            .learning_enabled
            .then(|| PhasePartialDecision {
                action: (0..self.config.motor_cells)
                    .min_by_key(|&a| (partial.masks[a].observations, a))
                    .unwrap(),
                kind: PhasePartialDecisionKind::MaskExploration,
                planned_depth: 1,
                expected_information: 0,
            })
    }
}
