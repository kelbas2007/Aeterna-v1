// Experimental hybrid rule acquisition. The hypothesis language is inherited:
// Single-channel or explicitly selected circular affine formula families.
// Acquired continuous offsets live ONLY in shared PhaseSynapses. Structural
// selection, hypothesis bookkeeping and
// search are software algorithms, not a claim of fully phase-owned cognition.

const MAX_PHASE_RULE_SYNAPSES: usize = 32768;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PhaseRuleLanguage {
    #[default]
    SingleChannel,
    CircularAffine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseRuleFamily {
    Constant,
    SingleChannel,
    IntegerGain,
    TwoChannelSum,
    TwoChannelDifference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseRuleTerm {
    pub source: usize,
    pub coefficient: i8,
}

/// Acquired structural selection only. The offset has no metadata backup;
/// forecasts still require the corresponding shared physical synapses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseRuleFormula {
    pub family: PhaseRuleFamily,
    pub terms: Vec<PhaseRuleTerm>,
}

#[derive(Debug, Clone, Copy)]
struct PhaseRuleTemplate {
    source: usize,
    orientation: i8,
    secondary: Option<(usize, i8)>,
}

fn rule_templates(language: PhaseRuleLanguage, width: usize) -> Vec<PhaseRuleTemplate> {
    let mut templates = Vec::new();
    // Keep the original prefix/order for the default language and old files.
    for source in 0..width {
        for orientation in [1, -1] {
            templates.push(PhaseRuleTemplate {
                source,
                orientation,
                secondary: None,
            });
        }
    }
    if language == PhaseRuleLanguage::CircularAffine {
        templates.push(PhaseRuleTemplate {
            source: 0,
            orientation: 0,
            secondary: None,
        });
        for source in 0..width {
            for orientation in [2, -2, 3, -3] {
                templates.push(PhaseRuleTemplate {
                    source,
                    orientation,
                    secondary: None,
                });
            }
        }
        for source in 0..width {
            for other in source + 1..width {
                for orientation in [1, -1] {
                    for secondary_orientation in [1, -1] {
                        templates.push(PhaseRuleTemplate {
                            source,
                            orientation,
                            secondary: Some((other, secondary_orientation)),
                        });
                    }
                }
            }
        }
    }
    templates
}

fn rule_synapse_budget(
    templates: &[PhaseRuleTemplate],
    width: usize,
    motors: usize,
) -> Option<usize> {
    let per_output = templates.iter().try_fold(0_usize, |n, t| {
        n.checked_add(1 + usize::from(t.secondary.is_some()))
    })?;
    width.checked_mul(motors)?.checked_mul(per_output)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseRuleConfig {
    pub min_distinct_support: usize,
    pub evidence_capacity: usize,
    pub tolerance: f32,
    pub planning_depth: usize,
    pub beam_width: usize,
    pub node_budget: usize,
}

impl Default for PhaseRuleConfig {
    fn default() -> Self {
        Self {
            min_distinct_support: 3,
            evidence_capacity: 8,
            tolerance: 0.0001,
            planning_depth: 8,
            beam_width: 64,
            node_budget: 4096,
        }
    }
}

impl PhaseRuleConfig {
    fn valid(&self) -> bool {
        (2..=32).contains(&self.min_distinct_support)
            && (self.min_distinct_support..=32).contains(&self.evidence_capacity)
            && self.tolerance.is_finite()
            && (0.000001..=0.01).contains(&self.tolerance)
            && (1..=16).contains(&self.planning_depth)
            && (1..=256).contains(&self.beam_width)
            && (1..=65536).contains(&self.node_budget)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseRuleCandidate {
    source: usize,
    orientation: i8,
    synapse: usize,
    active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    secondary: Option<PhaseRuleSecondary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseRuleSecondary {
    source: usize,
    orientation: i8,
    synapse: usize,
}

impl PhaseRuleCandidate {
    fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        std::iter::once(self.synapse).chain(self.secondary.iter().map(|t| t.synapse))
    }

    fn input_sum(&self, input: &[f32]) -> Option<f32> {
        // Zero terms in a constant formula are a mathematical bias, not an
        // invented value for a missing sensory observation.
        let mut value = if self.orientation == 0 {
            0.0
        } else {
            self.orientation as f32 * input.get(self.source)?
        };
        if let Some(other) = &self.secondary {
            value += other.orientation as f32 * input.get(other.source)?;
        }
        Some(value)
    }

    fn formula(&self) -> PhaseRuleFormula {
        let mut terms = Vec::new();
        if self.orientation != 0 {
            terms.push(PhaseRuleTerm {
                source: self.source,
                coefficient: self.orientation,
            });
        }
        let family = if let Some(other) = &self.secondary {
            terms.push(PhaseRuleTerm {
                source: other.source,
                coefficient: other.orientation,
            });
            if self.orientation == other.orientation {
                PhaseRuleFamily::TwoChannelSum
            } else {
                PhaseRuleFamily::TwoChannelDifference
            }
        } else if self.orientation == 0 {
            PhaseRuleFamily::Constant
        } else if self.orientation.abs() == 1 {
            PhaseRuleFamily::SingleChannel
        } else {
            PhaseRuleFamily::IntegerGain
        };
        PhaseRuleFormula { family, terms }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseRuleOutput {
    cell: usize,
    candidates: Vec<PhaseRuleCandidate>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseRuleEvidence {
    sequence: u64,
    pre: Vec<f32>,
    post: Vec<f32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseRuleAction {
    outputs: Vec<PhaseRuleOutput>,
    evidence: Vec<PhaseRuleEvidence>,
    observations: u64,
    revision: u64,
    #[serde(default)]
    suspended: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseRuleState {
    config: PhaseRuleConfig,
    #[serde(default)]
    language: PhaseRuleLanguage,
    actions: Vec<PhaseRuleAction>,
    factual_sequence: u64,
    #[serde(default)]
    adaptive: Option<PhaseAdaptiveState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseRuleActionInfo {
    pub action: usize,
    pub observations: u64,
    pub revision: u64,
    pub distinct_support: usize,
    pub candidate_counts: Vec<usize>,
    pub confirmed: bool,
    pub synapses: Vec<usize>,
    pub evidence_sources: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseRuleForecast {
    pub sensory: Vec<f32>,
    pub authority: Authority,
    pub action_revision: u64,
    pub evidence_sources: Vec<u64>,
}

fn rule_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| {
            let d = (x - y).abs();
            d.min(1.0 - d)
        })
        .fold(0.0, f32::max)
}

fn rule_vector_valid(xs: &[f32], width: usize) -> bool {
    xs.len() == width && xs.iter().all(|x| x.is_finite() && (0.0..1.0).contains(x))
}

fn rule_candidate_output(
    candidate: &PhaseRuleCandidate,
    input: &[f32],
    cells: &[PhaseCell],
    synapses: &[PhaseSynapse],
) -> Option<f32> {
    rule_candidate_readout(
        candidate,
        |source| input.get(source).copied(),
        cells,
        synapses,
    )
}

fn rule_candidate_partial_output(
    candidate: &PhaseRuleCandidate,
    input: &[Option<f32>],
    cells: &[PhaseCell],
    synapses: &[PhaseSynapse],
) -> Option<f32> {
    rule_candidate_readout(
        candidate,
        |source| input.get(source).copied().flatten(),
        cells,
        synapses,
    )
}

fn rule_candidate_readout(
    candidate: &PhaseRuleCandidate,
    input: impl Fn(usize) -> Option<f32>,
    cells: &[PhaseCell],
    synapses: &[PhaseSynapse],
) -> Option<f32> {
    let mut phase = rule_candidate_phase(candidate, cells, synapses)?;
    // No independent backup offset or exact recurrence replaces this readout.
    if candidate.orientation != 0 {
        phase += candidate.orientation as f32 * input(candidate.source)? * std::f32::consts::TAU;
    }
    if let Some(other) = &candidate.secondary {
        phase += other.orientation as f32 * input(other.source)? * std::f32::consts::TAU;
    }
    Some(rule_canonical_value(
        wrap_phase(phase) / std::f32::consts::TAU,
    ))
}

fn rule_canonical_value(value: f32) -> f32 {
    if value.min(1.0 - value) <= 2.0 * f32::EPSILON {
        0.0
    } else {
        value
    }
}

fn rule_candidate_phase(
    candidate: &PhaseRuleCandidate,
    cells: &[PhaseCell],
    synapses: &[PhaseSynapse],
) -> Option<f32> {
    if !candidate.active {
        return None;
    }
    let mut phase = 0.0;
    for index in candidate.indices() {
        let synapse = synapses.get(index)?;
        if synapse.weight < 0.5
            || synapse.confidence < 0.5
            || !cells.get(synapse.from)?.recruited
            || !cells.get(synapse.to)?.recruited
        {
            return None;
        }
        phase += synapse.phase_offset;
    }
    Some(phase)
}

fn seed_rule_candidate(
    candidate: &mut PhaseRuleCandidate,
    pre: &[f32],
    post: f32,
    synapses: &mut [PhaseSynapse],
) -> Option<()> {
    let offset = wrap_phase(std::f32::consts::TAU * (post - candidate.input_sum(pre)?));
    candidate.active = true;
    for index in candidate.indices() {
        let synapse = synapses.get_mut(index)?;
        synapse.phase_offset = if index == candidate.synapse {
            offset
        } else {
            0.0
        };
        synapse.weight = 1.0;
        synapse.confidence = 1.0;
        synapse.eligibility = 1.0;
    }
    Some(())
}

fn suppress_rule_candidate(candidate: &mut PhaseRuleCandidate, synapses: &mut [PhaseSynapse]) {
    candidate.active = false;
    for index in candidate.indices() {
        let synapse = &mut synapses[index];
        synapse.weight = 0.0;
        synapse.confidence = 0.0;
    }
}

impl EvoPhase {
    /// Opt-in before any online experience. Reserves a fixed physical budget
    /// per action/channel; capacity never grows with the number of observations.
    pub fn enable_phase_rule_learning(&mut self, config: PhaseRuleConfig) -> bool {
        self.enable_phase_rule_learning_with_language(config, PhaseRuleLanguage::SingleChannel)
    }

    /// Selects an inherited hypothesis vocabulary, never a world's formulas,
    /// coefficients or offsets. All candidate selection needs factual tuition.
    pub fn enable_phase_rule_learning_with_language(
        &mut self,
        config: PhaseRuleConfig,
        language: PhaseRuleLanguage,
    ) -> bool {
        let width = self.config.sensory_cells;
        let motors = self.config.motor_cells;
        if !config.valid()
            || !(1..=16).contains(&width)
            || !(1..=32).contains(&motors)
            || !self.config.structural_growth_enabled
        {
            return false;
        }
        let templates = rule_templates(language, width);
        if rule_synapse_budget(&templates, width, motors)
            .is_none_or(|n| n > MAX_PHASE_RULE_SYNAPSES)
        {
            return false;
        }
        let Some(state) = self.phase_native.as_ref() else {
            return false;
        };
        if state.online.is_none()
            || state.rules.is_some()
            || !state.receptors.is_empty()
            || !state.circuits.is_empty()
            || state.drive.is_some()
            || state.concepts.is_some()
            || state.deep.is_some()
            || state.contextual.is_some()
            || state.perceptual.is_some()
            || state.compositional.is_some()
            || state.meta_control.is_some()
            || state.temporal_evidence.is_some()
            || self.concept_memory.is_some()
        {
            return false;
        }
        let free = self
            .dormant_range()
            .filter(|&i| !self.cells[i].recruited)
            .take(width * motors)
            .collect::<Vec<_>>();
        if free.len() != width * motors {
            return false;
        }
        let mut actions = Vec::with_capacity(motors);
        for action in 0..motors {
            let mut outputs = Vec::with_capacity(width);
            for channel in 0..width {
                let cell = free[action * width + channel];
                self.cells[cell].recruited = true;
                let mut candidates = Vec::with_capacity(templates.len());
                for template in &templates {
                    let from = if template.orientation == 0 {
                        width + motors
                    } else {
                        template.source
                    };
                    let synapse = self.native_synapse(from, cell);
                    let secondary =
                        template
                            .secondary
                            .map(|(source, orientation)| PhaseRuleSecondary {
                                source,
                                orientation,
                                synapse: self.native_synapse(source, cell),
                            });
                    candidates.push(PhaseRuleCandidate {
                        source: template.source,
                        orientation: template.orientation,
                        synapse,
                        active: false,
                        secondary,
                    });
                }
                outputs.push(PhaseRuleOutput { cell, candidates });
            }
            actions.push(PhaseRuleAction {
                outputs,
                evidence: Vec::new(),
                observations: 0,
                revision: 0,
                suspended: false,
            });
        }
        self.phase_native.as_mut().expect("native state").rules = Some(PhaseRuleState {
            config,
            language,
            actions,
            factual_sequence: 0,
            adaptive: None,
        });
        true
    }

    pub fn phase_rules_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .is_some_and(|s| s.rules.is_some())
    }

    pub fn phase_rule_language(&self) -> Option<PhaseRuleLanguage> {
        Some(self.phase_native.as_ref()?.rules.as_ref()?.language)
    }

    /// Expand existing single-channel knowledge using retained factual
    /// evidence. Adds no observations, revisions, event IDs or actuator power.
    /// New alternatives can make a formerly unique formula ambiguous again.
    pub fn expand_phase_rule_language(&mut self) -> bool {
        let Some(native) = self.phase_native.as_ref() else {
            return false;
        };
        let Some(rules) = native.rules.as_ref() else {
            return false;
        };
        if rules.adaptive.is_some()
            || rules.language != PhaseRuleLanguage::SingleChannel
            || !native.config.learning_enabled
            || !self.config.structural_growth_enabled
        {
            return false;
        }
        let width = self.config.sensory_cells;
        let motors = self.config.motor_cells;
        let templates = rule_templates(PhaseRuleLanguage::CircularAffine, width);
        if rule_synapse_budget(&templates, width, motors)
            .is_none_or(|n| n > MAX_PHASE_RULE_SYNAPSES)
        {
            return false;
        }
        // All resource checks precede mutation. Existing candidate parameters
        // and addresses remain intact; only the new vocabulary is instantiated.
        let mut native = self.phase_native.take().expect("checked native");
        let rules = native.rules.as_mut().expect("checked rules");
        for action in &mut rules.actions {
            for (j, output) in action.outputs.iter_mut().enumerate() {
                for template in templates.iter().skip(2 * width) {
                    let from = if template.orientation == 0 {
                        width + motors
                    } else {
                        template.source
                    };
                    let synapse = self.native_synapse(from, output.cell);
                    let secondary =
                        template
                            .secondary
                            .map(|(source, orientation)| PhaseRuleSecondary {
                                source,
                                orientation,
                                synapse: self.native_synapse(source, output.cell),
                            });
                    let mut candidate = PhaseRuleCandidate {
                        source: template.source,
                        orientation: template.orientation,
                        synapse,
                        active: false,
                        secondary,
                    };
                    if let Some(first) = action.evidence.first() {
                        seed_rule_candidate(
                            &mut candidate,
                            &first.pre,
                            first.post[j],
                            &mut self.synapses,
                        )
                        .expect("validated factual evidence");
                        if action.evidence.iter().any(|e| {
                            !rule_candidate_output(&candidate, &e.pre, &self.cells, &self.synapses)
                                .is_some_and(|v| {
                                    rule_distance(&[v], &[e.post[j]]) <= rules.config.tolerance
                                })
                        }) {
                            suppress_rule_candidate(&mut candidate, &mut self.synapses);
                        }
                    }
                    output.candidates.push(candidate);
                }
            }
        }
        rules.language = PhaseRuleLanguage::CircularAffine;
        if let Some(partial) = native.partial.as_mut() {
            partial.episode = None;
        }
        self.phase_native = Some(native);
        self.current_real = None;
        true
    }

    pub fn phase_rule_formulas(&self, action: usize) -> Option<Vec<PhaseRuleFormula>> {
        if !self.phase_rule_action_info(action)?.confirmed {
            return None;
        }
        self.phase_native
            .as_ref()?
            .rules
            .as_ref()?
            .actions
            .get(action)?
            .outputs
            .iter()
            .map(|output| Some(output.candidates.iter().find(|c| c.active)?.formula()))
            .collect()
    }

    fn is_phase_rule_synapse(&self, index: usize) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|s| s.rules.as_ref())
            .is_some_and(|rules| {
                rules.actions.iter().any(|a| {
                    a.outputs
                        .iter()
                        .any(|o| o.candidates.iter().any(|c| c.indices().any(|i| i == index)))
                }) || rules.adaptive.as_ref().is_some_and(|s| {
                    s.actions.iter().any(|a| {
                        a.slots.iter().any(|slot| {
                            slot.outputs.iter().any(|o| {
                                o.candidates.iter().any(|c| c.indices().any(|i| i == index))
                            })
                        })
                    })
                })
            })
    }

    pub fn phase_rule_goal_matches(&self, current: &[f32], goal: &[f32]) -> bool {
        let Some(rules) = self.phase_native.as_ref().and_then(|s| s.rules.as_ref()) else {
            return false;
        };
        rule_vector_valid(current, self.config.sensory_cells)
            && rule_vector_valid(goal, self.config.sensory_cells)
            && rule_distance(current, goal) <= rules.config.tolerance
    }

    pub fn phase_rule_action_info(&self, action: usize) -> Option<PhaseRuleActionInfo> {
        let rules = self.phase_native.as_ref()?.rules.as_ref()?;
        let model = rules.actions.get(action)?;
        let candidate_counts = model
            .outputs
            .iter()
            .map(|o| o.candidates.iter().filter(|c| c.active).count())
            .collect::<Vec<_>>();
        let confirmed = !model.suspended
            && model.evidence.len() >= rules.config.min_distinct_support
            && candidate_counts.iter().all(|&n| n == 1)
            && model.outputs.iter().all(|o| {
                o.candidates.iter().filter(|c| c.active).all(|c| {
                    rule_candidate_output(c, &model.evidence[0].pre, &self.cells, &self.synapses)
                        .is_some()
                })
            });
        Some(PhaseRuleActionInfo {
            action,
            observations: model.observations,
            revision: model.revision,
            distinct_support: model.evidence.len(),
            candidate_counts,
            confirmed,
            synapses: model
                .outputs
                .iter()
                .flat_map(|o| o.candidates.iter())
                .filter(|c| c.active)
                .flat_map(PhaseRuleCandidate::indices)
                .collect(),
            evidence_sources: model.evidence.iter().map(|e| e.sequence).collect(),
        })
    }

    /// A derived forecast has IMAGINED authority even with confirmed sources.
    /// Inputs and outputs are continuous circular channels in [0,1).
    pub fn phase_rule_predict(&self, action: usize, input: &[f32]) -> Option<PhaseRuleForecast> {
        if !rule_vector_valid(input, self.config.sensory_cells)
            || !self.phase_rule_action_info(action)?.confirmed
        {
            return None;
        }
        let model = &self.phase_native.as_ref()?.rules.as_ref()?.actions[action];
        let sensory = model
            .outputs
            .iter()
            .map(|o| {
                let candidate = o.candidates.iter().find(|c| c.active)?;
                rule_candidate_output(candidate, input, &self.cells, &self.synapses)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(PhaseRuleForecast {
            sensory,
            authority: Authority::Imagined,
            action_revision: model.revision,
            evidence_sources: model.evidence.iter().map(|e| e.sequence).collect(),
        })
    }

    /// Diagnostic alternatives per output, not observations or ready plans.
    pub fn phase_rule_hypotheses(&self, action: usize, input: &[f32]) -> Option<Vec<Vec<f32>>> {
        if !rule_vector_valid(input, self.config.sensory_cells) {
            return None;
        }
        let model = self
            .phase_native
            .as_ref()?
            .rules
            .as_ref()?
            .actions
            .get(action)?;
        Some(
            model
                .outputs
                .iter()
                .map(|o| {
                    o.candidates
                        .iter()
                        .filter_map(|c| {
                            rule_candidate_output(c, input, &self.cells, &self.synapses)
                        })
                        .collect()
                })
                .collect(),
        )
    }

    /// Called by the permitted action/validated POST boundary in the runtime.
    /// Goal changes, passive sensing and imagined search never invoke this.
    pub(crate) fn observe_phase_rule_result(
        &mut self,
        action: usize,
        pre: &[f32],
        post: &[f32],
    ) -> Option<usize> {
        if self.phase_adaptive_rules_enabled() {
            return self.observe_phase_adaptive_result(action, pre, post);
        }
        if !rule_vector_valid(pre, self.config.sensory_cells)
            || !rule_vector_valid(post, self.config.sensory_cells)
        {
            return None;
        }
        let mut state = self.phase_native.take()?;
        let result = (|| {
            if !state.config.learning_enabled {
                return None;
            }
            let rules = state.rules.as_mut()?;
            let model = rules.actions.get_mut(action)?;
            let sequence = rules.factual_sequence.checked_add(1)?;
            let tolerance = rules.config.tolerance;
            let has_evidence = !model.suspended && !model.evidence.is_empty();
            if model.suspended {
                // The visible contradiction already advanced the revision.
                // Reconfirmation starts only from a fully factual transition.
                model.evidence.clear();
                model.suspended = false;
            }
            let mut matching = Vec::new();
            if has_evidence {
                for (j, output) in model.outputs.iter().enumerate() {
                    matching.push(
                        output
                            .candidates
                            .iter()
                            .map(|c| {
                                rule_candidate_output(c, pre, &self.cells, &self.synapses)
                                    .is_some_and(|v| rule_distance(&[v], &[post[j]]) <= tolerance)
                            })
                            .collect::<Vec<_>>(),
                    );
                }
            }
            let contradicted = has_evidence && matching.iter().any(|row| !row.iter().any(|&x| x));
            if contradicted {
                model.revision = model.revision.checked_add(1)?;
                model.evidence.clear();
            }
            let mut suppressed = 0;
            for (j, output) in model.outputs.iter_mut().enumerate() {
                for (k, candidate) in output.candidates.iter_mut().enumerate() {
                    if !has_evidence || contradicted {
                        // First genuine sample of a new revision seeds every
                        // formula; later facts eliminate alternatives.
                        seed_rule_candidate(candidate, pre, post[j], &mut self.synapses)?;
                    } else if candidate.active && !matching[j][k] {
                        suppress_rule_candidate(candidate, &mut self.synapses);
                        suppressed += 1;
                    }
                }
            }
            if !model
                .evidence
                .iter()
                .any(|e| rule_distance(&e.pre, pre) <= tolerance)
            {
                if model.evidence.len() == rules.config.evidence_capacity {
                    model.evidence.remove(0);
                }
                model.evidence.push(PhaseRuleEvidence {
                    sequence,
                    pre: pre.to_vec(),
                    post: post.to_vec(),
                });
            }
            model.observations = model.observations.checked_add(1)?;
            rules.factual_sequence = sequence;
            Some(suppressed)
        })();
        self.phase_native = Some(state);
        result
    }
}
