// Partial evidence is explicit. None denotes ANY circular value, never zero.
// Belief propagation is software inference over acquired phase parameters;
// predicted values cannot become factual transition tuition or REAL success.

pub type PhasePartialVector = Vec<Option<f32>>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhasePartialConfig {
    pub max_hypotheses: usize,
    pub min_mask_support: usize,
}
impl Default for PhasePartialConfig {
    fn default() -> Self {
        Self {
            max_hypotheses: 32,
            min_mask_support: 3,
        }
    }
}
impl PhasePartialConfig {
    fn valid(&self) -> bool {
        (1..=128).contains(&self.max_hypotheses) && (2..=32).contains(&self.min_mask_support)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhasePartialMask {
    visible: Vec<bool>,
    observations: u64,
    revision: u64,
    source_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
struct PhasePartialEpisode {
    factual: PhasePartialVector,
    hypotheses: Vec<PhasePartialVector>,
    widened: bool,
    invalid_rules: Vec<bool>,
    invalid_masks: Vec<bool>,
    inverse: Option<PhaseInverseReport>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhasePartialState {
    config: PhasePartialConfig,
    masks: Vec<PhasePartialMask>,
    factual_sequence: u64,
    #[serde(default)]
    inverse_enabled: bool,
    #[serde(skip)]
    episode: Option<PhasePartialEpisode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhasePartialMaskInfo {
    pub action: usize,
    pub visible: Vec<bool>,
    pub observations: u64,
    pub revision: u64,
    pub evidence_sources: Vec<u64>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhasePartialBeliefInfo {
    pub factual: PhasePartialVector,
    pub hypotheses: Vec<PhasePartialVector>,
    pub hypotheses_authority: Authority,
    pub widened: bool,
    pub invalidated_rules: Vec<usize>,
    pub invalidated_masks: Vec<usize>,
    pub inverse: Option<PhaseInverseReport>,
}

pub(crate) struct PhasePartialUpdate {
    pub suppressed: usize,
    pub learned: bool,
}

fn partial_vector_valid(values: &[Option<f32>], width: usize) -> bool {
    values.len() == width
        && values
            .iter()
            .flatten()
            .all(|x| x.is_finite() && (0.0..1.0).contains(x))
}

fn partial_goal_matches(values: &[Option<f32>], goal: &[Option<f32>], tolerance: f32) -> bool {
    values.iter().zip(goal).all(|(value, target)| match target {
        None => true,
        Some(target) => value.is_some_and(|value| rule_distance(&[value], &[*target]) <= tolerance),
    })
}

fn partial_hull(rows: &[PhasePartialVector], tolerance: f32) -> PhasePartialVector {
    (0..rows[0].len())
        .map(|j| {
            let first = rows[0][j]?;
            rows.iter()
                .all(|row| row[j].is_some_and(|x| rule_distance(&[x], &[first]) <= tolerance))
                .then_some(first)
        })
        .collect()
}

impl EvoPhase {
    /// Enables partial inference over the opt-in rule language. Raw full
    /// tuition may precede this; no observation mask or motor identity is given.
    pub fn enable_phase_partial_observation(&mut self, config: PhasePartialConfig) -> bool {
        if !config.valid() {
            return false;
        }
        let Some(native) = self.phase_native.as_mut() else {
            return false;
        };
        if native.rules.as_ref().is_none_or(|r| r.adaptive.is_some()) || native.partial.is_some() {
            return false;
        }
        native.partial = Some(PhasePartialState {
            config,
            masks: (0..self.config.motor_cells)
                .map(|_| PhasePartialMask {
                    visible: vec![false; self.config.sensory_cells],
                    observations: 0,
                    revision: 0,
                    source_ids: Vec::new(),
                })
                .collect(),
            factual_sequence: 0,
            inverse_enabled: false,
            episode: None,
        });
        self.current_real = None;
        true
    }

    pub fn phase_partial_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .is_some_and(|s| s.partial.is_some())
    }

    pub fn phase_partial_belief(&self) -> Option<PhasePartialBeliefInfo> {
        let episode = self
            .phase_native
            .as_ref()?
            .partial
            .as_ref()?
            .episode
            .as_ref()?;
        Some(PhasePartialBeliefInfo {
            factual: episode.factual.clone(),
            hypotheses: episode.hypotheses.clone(),
            hypotheses_authority: Authority::Imagined,
            widened: episode.widened,
            invalidated_rules: episode
                .invalid_rules
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| x.then_some(i))
                .collect(),
            invalidated_masks: episode
                .invalid_masks
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| x.then_some(i))
                .collect(),
            inverse: episode.inverse.clone(),
        })
    }

    pub fn phase_partial_mask_info(&self, action: usize) -> Option<PhasePartialMaskInfo> {
        let partial = self.phase_native.as_ref()?.partial.as_ref()?;
        let mask = partial.masks.get(action)?;
        Some(PhasePartialMaskInfo {
            action,
            visible: mask.visible.clone(),
            observations: mask.observations,
            revision: mask.revision,
            evidence_sources: mask.source_ids.clone(),
            confirmed: mask.source_ids.len() >= partial.config.min_mask_support,
        })
    }

    pub(crate) fn observe_phase_partial_initial(&mut self, observation: &[Option<f32>]) -> bool {
        if !partial_vector_valid(observation, self.config.sensory_cells) {
            return false;
        }
        let Some(partial) = self.phase_native.as_mut().and_then(|s| s.partial.as_mut()) else {
            return false;
        };
        partial.episode = Some(PhasePartialEpisode {
            factual: observation.to_vec(),
            hypotheses: vec![observation.to_vec()],
            widened: false,
            invalid_rules: vec![false; self.config.motor_cells],
            invalid_masks: vec![false; self.config.motor_cells],
            inverse: None,
        });
        self.current_real = None;
        true
    }

    pub fn phase_partial_goal_reached(&self, goal: &[Option<f32>]) -> bool {
        let Some(native) = self.phase_native.as_ref() else {
            return false;
        };
        let Some(episode) = native.partial.as_ref().and_then(|s| s.episode.as_ref()) else {
            return false;
        };
        let tolerance = native
            .rules
            .as_ref()
            .expect("partial rule mode")
            .config
            .tolerance;
        partial_vector_valid(goal, self.config.sensory_cells)
            && goal.iter().any(Option::is_some)
            && partial_goal_matches(&episode.factual, goal, tolerance)
    }

    fn partial_action_supported(&self, action: usize) -> bool {
        let Some(partial) = self.phase_native.as_ref().and_then(|s| s.partial.as_ref()) else {
            return false;
        };
        let Some(episode) = partial.episode.as_ref() else {
            return false;
        };
        self.phase_rule_action_info(action)
            .is_some_and(|i| i.confirmed)
            && self
                .phase_partial_mask_info(action)
                .is_some_and(|i| i.confirmed)
            && !episode.invalid_rules[action]
            && !episode.invalid_masks[action]
    }

    /// May widen to a symbolic superset at capacity, but never prune a live
    /// alternative to fabricate confidence. Cartesian products deliberately
    /// overapproximate correlations; an unknown value stands for the full circle.
    fn partial_transition(
        &self,
        action: usize,
        input: &[PhasePartialVector],
    ) -> Option<(Vec<PhasePartialVector>, bool)> {
        let native = self.phase_native.as_ref()?;
        let rules = native.rules.as_ref()?;
        let partial = native.partial.as_ref()?;
        let model = rules.actions.get(action)?;
        let width = self.config.sensory_cells;
        let tolerance = rules.config.tolerance;
        let max = partial.config.max_hypotheses;
        let invalid = partial
            .episode
            .as_ref()
            .is_some_and(|e| e.invalid_rules[action]);
        if model.suspended || invalid || model.evidence.is_empty() {
            return Some((vec![vec![None; width]], false));
        }
        let mut rows = Vec::new();
        let mut widened = false;
        for pre in input {
            let mut choices = Vec::with_capacity(width);
            for output in &model.outputs {
                let mut options = Vec::new();
                for candidate in output.candidates.iter().filter(|c| c.active) {
                    let value =
                        rule_candidate_partial_output(candidate, pre, &self.cells, &self.synapses);
                    match value {
                        None => {
                            options.clear();
                            options.push(None);
                            break;
                        }
                        Some(value)
                            if !options
                                .iter()
                                .flatten()
                                .any(|&old| rule_distance(&[old], &[value]) <= tolerance) =>
                        {
                            options.push(Some(value))
                        }
                        Some(_) => {}
                    }
                }
                if options.is_empty() {
                    options.push(None);
                }
                choices.push(options);
            }
            let cardinality = choices
                .iter()
                .try_fold(1_usize, |n, values| n.checked_mul(values.len()));
            let mut generated = if cardinality.is_none_or(|n| n > max) {
                widened = true;
                vec![choices
                    .iter()
                    .map(|options| if options.len() == 1 { options[0] } else { None })
                    .collect()]
            } else {
                let mut product = vec![Vec::new()];
                for options in choices {
                    let mut next = Vec::new();
                    for prefix in &product {
                        for option in &options {
                            let mut row = prefix.clone();
                            row.push(*option);
                            next.push(row);
                        }
                    }
                    product = next;
                }
                product
            };
            rows.append(&mut generated);
            if rows.len() > max {
                rows = vec![partial_hull(&rows, tolerance)];
                widened = true;
            }
        }
        Some((rows, widened))
    }

    pub(crate) fn observe_phase_partial_result(
        &mut self,
        action: usize,
        post: &[Option<f32>],
    ) -> Option<PhasePartialUpdate> {
        if !partial_vector_valid(post, self.config.sensory_cells) {
            return None;
        }
        let native = self.phase_native.as_ref()?;
        let old = native.partial.as_ref()?.episode.as_ref()?.clone();
        let tolerance = native.rules.as_ref()?.config.tolerance;
        let learning = native.config.learning_enabled;
        let mask_before = self.phase_partial_mask_info(action)?;
        let (predicted, widened, inverse) =
            self.partial_observed_transition(action, &old.hypotheses, post)?;
        let predicted_count = predicted.len();
        let mut compatible = predicted
            .into_iter()
            .filter(|row| {
                row.iter()
                    .zip(post)
                    .all(|(derived, factual)| match (derived, factual) {
                        (Some(a), Some(b)) => rule_distance(&[*a], &[*b]) <= tolerance,
                        _ => true,
                    })
            })
            .map(|mut row| {
                for (derived, factual) in row.iter_mut().zip(post) {
                    if factual.is_some() {
                        *derived = *factual;
                    }
                }
                row
            })
            .collect::<Vec<_>>();
        let conflict = compatible.is_empty();
        if conflict {
            compatible.push(post.to_vec());
        }
        let mut learned = false;
        let mut full_tuition = false;
        let mut suppressed = predicted_count.saturating_sub(compatible.len());
        if learning {
            // Inferred hidden PRE/POST values never qualify as tuition.
            if let (Some(pre), Some(post)) = (
                old.factual.iter().copied().collect::<Option<Vec<_>>>(),
                post.iter().copied().collect::<Option<Vec<_>>>(),
            ) {
                suppressed += self.observe_phase_rule_result(action, &pre, &post)?;
                learned = true;
                full_tuition = true;
            } else if conflict {
                let model = &mut self.phase_native.as_mut()?.rules.as_mut()?.actions[action];
                if !model.suspended {
                    model.revision = model.revision.checked_add(1)?;
                    model.suspended = true;
                    learned = true;
                }
            }
        }
        let rule_reconfirmed = full_tuition && self.phase_rule_action_info(action)?.confirmed;
        let partial = self.phase_native.as_mut()?.partial.as_mut()?;
        let visible = post.iter().map(Option::is_some).collect::<Vec<_>>();
        let mask_conflict = mask_before.confirmed && visible != mask_before.visible;
        if learning {
            let sequence = partial.factual_sequence.checked_add(1)?;
            let mask = &mut partial.masks[action];
            if mask.observations > 0 && mask.visible != visible {
                mask.revision = mask.revision.checked_add(1)?;
                mask.source_ids.clear();
            }
            mask.visible = visible;
            mask.observations = mask.observations.checked_add(1)?;
            if mask.source_ids.len() == partial.config.min_mask_support {
                mask.source_ids.remove(0);
            }
            mask.source_ids.push(sequence);
            partial.factual_sequence = sequence;
            learned = true;
        }
        let mut invalid_rules = old.invalid_rules;
        let mut invalid_masks = old.invalid_masks;
        invalid_rules[action] |= conflict;
        invalid_masks[action] |= mask_conflict;
        if rule_reconfirmed {
            invalid_rules[action] = false;
        }
        if learning && partial.masks[action].source_ids.len() >= partial.config.min_mask_support {
            invalid_masks[action] = false;
        }
        partial.episode = Some(PhasePartialEpisode {
            factual: post.to_vec(),
            hypotheses: compatible,
            widened: old.widened || widened,
            invalid_rules,
            invalid_masks,
            inverse,
        });
        self.current_real = None;
        Some(PhasePartialUpdate {
            suppressed,
            learned,
        })
    }
}
