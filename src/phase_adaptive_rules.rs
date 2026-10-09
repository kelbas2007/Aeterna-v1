// Opt-in bounded, noisy, piecewise circular-affine learning. The fitting and
// threshold search are inherited software; selected offsets have no backup
// outside the shared physical synapses. Only permitted full PRE/POST teaches.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseAdaptiveConfig {
    pub min_support: usize,
    pub evidence_capacity: usize,
    pub model_slots: usize,
    pub residual_tolerance: f32,
    pub goal_tolerance: f32,
    pub min_inlier_fraction: f32,
    pub change_support: usize,
    pub allow_conditions: bool,
    pub fit_budget: usize,
}
impl Default for PhaseAdaptiveConfig {
    fn default() -> Self {
        Self {
            min_support: 6,
            evidence_capacity: 64,
            model_slots: 4,
            residual_tolerance: 0.002,
            goal_tolerance: 0.01,
            min_inlier_fraction: 0.9,
            change_support: 6,
            allow_conditions: true,
            fit_budget: 1_000_000,
        }
    }
}
impl PhaseAdaptiveConfig {
    fn valid(&self) -> bool {
        (4..=32).contains(&self.min_support)
            && (2 * self.min_support..=128).contains(&self.evidence_capacity)
            && (4..=8).contains(&self.model_slots)
            && self.residual_tolerance.is_finite()
            && (0.0001..=0.01).contains(&self.residual_tolerance)
            && self.goal_tolerance.is_finite()
            && (self.residual_tolerance..=0.05).contains(&self.goal_tolerance)
            && self.min_inlier_fraction.is_finite()
            && (0.8..=1.0).contains(&self.min_inlier_fraction)
            && (3..=self.min_support).contains(&self.change_support)
            && (1024..=8_000_000).contains(&self.fit_budget)
    }
}

/// A learned interval bracketing a threshold, not an exact supplied boundary.
/// The unsupported interval between lower and upper leads to abstention.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseAdaptiveCondition {
    pub source: usize,
    pub lower: f32,
    pub upper: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseAdaptiveModel {
    condition: Option<PhaseAdaptiveCondition>,
    slots: Vec<usize>,
    evidence_sources: Vec<u64>,
    supports: Vec<usize>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseAdaptiveSlot {
    outputs: Vec<PhaseRuleOutput>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseAdaptiveAction {
    slots: Vec<PhaseAdaptiveSlot>,
    active: Option<PhaseAdaptiveModel>,
    archives: Vec<PhaseAdaptiveModel>,
    evidence: Vec<PhaseRuleEvidence>,
    pending: Vec<PhaseRuleEvidence>,
    observations: u64,
    revision: u64,
    reactivations: u64,
    evictions: u64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseAdaptiveState {
    config: PhaseAdaptiveConfig,
    actions: Vec<PhaseAdaptiveAction>,
    factual_sequence: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseAdaptiveModelInfo {
    pub condition: Option<PhaseAdaptiveCondition>,
    pub formulas: Vec<Vec<PhaseRuleFormula>>,
    pub supports: Vec<usize>,
    pub evidence_sources: Vec<u64>,
    pub synapses: Vec<usize>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseAdaptiveActionInfo {
    pub action: usize,
    pub observations: u64,
    pub revision: u64,
    pub reactivations: u64,
    pub evictions: u64,
    pub retained_examples: usize,
    pub active: Option<PhaseAdaptiveModelInfo>,
    pub archives: Vec<PhaseAdaptiveModelInfo>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseAdaptiveForecast {
    pub sensory: Vec<f32>,
    pub error_radius: Vec<f32>,
    pub authority: Authority,
    pub revision: u64,
    pub evidence_sources: Vec<u64>,
}

struct AdaptiveFitProgram {
    candidates: Vec<usize>,
    offsets: Vec<f32>,
    support: usize,
}
struct AdaptiveFit {
    condition: Option<PhaseAdaptiveCondition>,
    programs: Vec<AdaptiveFitProgram>,
    score: usize,
}

fn adaptive_branch(
    condition: Option<&PhaseAdaptiveCondition>,
    pre: &[f32],
    radius: &[f32],
) -> Option<usize> {
    if let Some(c) = condition {
        // Circular uncertainty crossing zero can include both threshold
        // branches even when its center is near one. Never choose by center.
        if radius[c.source] > 0.0
            && (pre[c.source] - radius[c.source] < 0.0 || pre[c.source] + radius[c.source] >= 1.0)
        {
            return None;
        }
    }
    match condition {
        None => Some(0),
        Some(c) if pre[c.source] + radius[c.source] <= c.lower => Some(0),
        Some(c) if pre[c.source] - radius[c.source] >= c.upper => Some(1),
        Some(_) => None,
    }
}
fn adaptive_predict_model(
    action: &PhaseAdaptiveAction,
    model: &PhaseAdaptiveModel,
    pre: &[f32],
    radius: &[f32],
    tolerance: f32,
    cells: &[PhaseCell],
    synapses: &[PhaseSynapse],
) -> Option<(Vec<f32>, Vec<f32>)> {
    let mut context_radius = radius.to_vec();
    if let Some(condition) = &model.condition {
        // The measured context itself can be noisy on the first step too.
        context_radius[condition.source] += tolerance;
    }
    let branch = adaptive_branch(model.condition.as_ref(), pre, &context_radius)?;
    let slot = &action.slots[*model.slots.get(branch)?];
    let mut values = Vec::new();
    let mut uncertainty = Vec::new();
    for output in &slot.outputs {
        let candidate = output.candidates.iter().find(|c| c.active)?;
        values.push(rule_candidate_output(candidate, pre, cells, synapses)?);
        uncertainty.push(
            tolerance
                + candidate
                    .formula()
                    .terms
                    .iter()
                    .map(|t| f32::from(t.coefficient.unsigned_abs()) * radius[t.source])
                    .sum::<f32>(),
        );
    }
    Some((values, uncertainty))
}

fn adaptive_circular_median(values: &mut [f32]) -> f32 {
    let tau = std::f32::consts::TAU;
    let (sin, cos) = values.iter().fold((0.0_f32, 0.0_f32), |(s, c), x| {
        (s + (x * tau).sin(), c + (x * tau).cos())
    });
    let anchor = sin.atan2(cos) / tau;
    for x in values.iter_mut() {
        *x = (*x - anchor + 0.5).rem_euclid(1.0) - 0.5;
    }
    values.sort_by(f32::total_cmp);
    let n = values.len();
    let median = if n.is_multiple_of(2) {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    } else {
        values[n / 2]
    };
    rule_canonical_value((anchor + median).rem_euclid(1.0))
}
fn adaptive_fit_program(
    rows: &[&PhaseRuleEvidence],
    templates: &[PhaseRuleTemplate],
    width: usize,
    cfg: &PhaseAdaptiveConfig,
    budget: &mut usize,
) -> Option<AdaptiveFitProgram> {
    if rows.len() < cfg.min_support {
        return None;
    }
    let mut candidates = Vec::new();
    let mut offsets = Vec::new();
    let mut support = rows.len();
    for j in 0..width {
        let mut acceptable = Vec::new();
        for (index, t) in templates.iter().enumerate() {
            *budget = budget.checked_sub(rows.len())?;
            let sum = |pre: &[f32]| {
                f32::from(t.orientation) * pre[t.source]
                    + t.secondary.map_or(0.0, |(s, a)| f32::from(a) * pre[s])
            };
            let mut residuals = rows
                .iter()
                .map(|e| (e.post[j] - sum(&e.pre)).rem_euclid(1.0))
                .collect::<Vec<_>>();
            let offset = adaptive_circular_median(&mut residuals);
            let count = rows
                .iter()
                .filter(|e| {
                    rule_distance(&[(sum(&e.pre) + offset).rem_euclid(1.0)], &[e.post[j]])
                        <= cfg.residual_tolerance
                })
                .count();
            if count >= cfg.min_support
                && count as f32 / rows.len() as f32 >= cfg.min_inlier_fraction
            {
                acceptable.push((index, offset, count));
            }
        }
        // Correlated observations cannot decide between multiple formulas.
        if acceptable.len() != 1 {
            return None;
        }
        let (index, offset, count) = acceptable[0];
        candidates.push(index);
        offsets.push(offset);
        support = support.min(count);
    }
    Some(AdaptiveFitProgram {
        candidates,
        offsets,
        support,
    })
}
fn adaptive_fit(
    rows: &[PhaseRuleEvidence],
    templates: &[PhaseRuleTemplate],
    width: usize,
    cfg: &PhaseAdaptiveConfig,
) -> Option<AdaptiveFit> {
    let full = rows.iter().collect::<Vec<_>>();
    let mut budget = cfg.fit_budget;
    if let Some(program) = adaptive_fit_program(&full, templates, width, cfg, &mut budget) {
        let score = program.support;
        return Some(AdaptiveFit {
            condition: None,
            programs: vec![program],
            score,
        });
    }
    if !cfg.allow_conditions || rows.len() < 2 * cfg.min_support {
        return None;
    }
    let mut best: Option<AdaptiveFit> = None;
    for source in 0..width {
        let mut ordered = full.clone();
        ordered.sort_by(|a, b| a.pre[source].total_cmp(&b.pre[source]));
        for split in cfg.min_support..=ordered.len() - cfg.min_support {
            let lower = ordered[split - 1].pre[source];
            let upper = ordered[split].pre[source];
            if upper - lower <= 2.0 * cfg.residual_tolerance {
                continue;
            }
            let Some(left) =
                adaptive_fit_program(&ordered[..split], templates, width, cfg, &mut budget)
            else {
                if budget < ordered.len() {
                    return None;
                }
                continue;
            };
            let Some(right) =
                adaptive_fit_program(&ordered[split..], templates, width, cfg, &mut budget)
            else {
                if budget < ordered.len() {
                    return None;
                }
                continue;
            };
            let score = left.support + right.support;
            let replace = best.as_ref().is_none_or(|b| {
                score > b.score
                    || (score == b.score
                        && upper - lower
                            > b.condition.as_ref().unwrap().upper
                                - b.condition.as_ref().unwrap().lower)
            });
            if replace {
                best = Some(AdaptiveFit {
                    condition: Some(PhaseAdaptiveCondition {
                        source,
                        lower,
                        upper,
                    }),
                    programs: vec![left, right],
                    score,
                });
            }
        }
    }
    best
}

impl EvoPhase {
    /// Cold opt-in on an existing rule vocabulary. No thresholds, sources,
    /// coefficients, noise samples or motor meanings are accepted here.
    pub fn enable_phase_adaptive_rules(&mut self, config: PhaseAdaptiveConfig) -> bool {
        if !config.valid() || !self.config.structural_growth_enabled {
            return false;
        }
        let Some(native) = self.phase_native.as_ref() else {
            return false;
        };
        let Some(rules) = native.rules.as_ref() else {
            return false;
        };
        if rules.adaptive.is_some() || rules.factual_sequence != 0 || native.partial.is_some() {
            return false;
        }
        let width = self.config.sensory_cells;
        let motors = self.config.motor_cells;
        let templates = rule_templates(rules.language, width);
        let budget = rule_synapse_budget(&templates, width, motors)
            .and_then(|n| n.checked_mul(config.model_slots + 1));
        if budget.is_none_or(|n| n > MAX_PHASE_RULE_SYNAPSES) {
            return false;
        }
        let count = width * motors * config.model_slots;
        let free = self
            .dormant_range()
            .filter(|&i| !self.cells[i].recruited)
            .take(count)
            .collect::<Vec<_>>();
        if free.len() != count {
            return false;
        }
        let mut actions = Vec::new();
        let mut next_cell = free.into_iter();
        for _ in 0..motors {
            let mut slots = Vec::new();
            for _ in 0..config.model_slots {
                let mut outputs = Vec::new();
                for _ in 0..width {
                    let cell = next_cell.next().expect("prechecked capacity");
                    self.cells[cell].recruited = true;
                    let mut candidates = Vec::new();
                    for t in &templates {
                        let from = if t.orientation == 0 {
                            width + motors
                        } else {
                            t.source
                        };
                        let synapse = self.native_synapse(from, cell);
                        let secondary =
                            t.secondary.map(|(source, orientation)| PhaseRuleSecondary {
                                source,
                                orientation,
                                synapse: self.native_synapse(source, cell),
                            });
                        candidates.push(PhaseRuleCandidate {
                            source: t.source,
                            orientation: t.orientation,
                            synapse,
                            active: false,
                            secondary,
                        });
                    }
                    outputs.push(PhaseRuleOutput { cell, candidates });
                }
                slots.push(PhaseAdaptiveSlot { outputs });
            }
            actions.push(PhaseAdaptiveAction {
                slots,
                active: None,
                archives: Vec::new(),
                evidence: Vec::new(),
                pending: Vec::new(),
                observations: 0,
                revision: 0,
                reactivations: 0,
                evictions: 0,
            });
        }
        self.phase_native
            .as_mut()
            .unwrap()
            .rules
            .as_mut()
            .unwrap()
            .adaptive = Some(PhaseAdaptiveState {
            config,
            actions,
            factual_sequence: 0,
        });
        self.current_real = None;
        true
    }
    pub fn phase_adaptive_rules_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|s| s.rules.as_ref())
            .is_some_and(|r| r.adaptive.is_some())
    }
    pub fn phase_adaptive_action_info(&self, action: usize) -> Option<PhaseAdaptiveActionInfo> {
        let adaptive = self
            .phase_native
            .as_ref()?
            .rules
            .as_ref()?
            .adaptive
            .as_ref()?;
        let a = adaptive.actions.get(action)?;
        let info = |m: &PhaseAdaptiveModel| PhaseAdaptiveModelInfo {
            condition: m.condition.clone(),
            supports: m.supports.clone(),
            evidence_sources: m.evidence_sources.clone(),
            formulas: m
                .slots
                .iter()
                .map(|&s| {
                    a.slots[s]
                        .outputs
                        .iter()
                        .filter_map(|o| {
                            o.candidates
                                .iter()
                                .find(|c| c.active)
                                .map(PhaseRuleCandidate::formula)
                        })
                        .collect()
                })
                .collect(),
            synapses: m
                .slots
                .iter()
                .flat_map(|&s| a.slots[s].outputs.iter())
                .flat_map(|o| {
                    o.candidates
                        .iter()
                        .filter(|c| c.active)
                        .flat_map(PhaseRuleCandidate::indices)
                })
                .collect(),
        };
        Some(PhaseAdaptiveActionInfo {
            action,
            observations: a.observations,
            revision: a.revision,
            reactivations: a.reactivations,
            evictions: a.evictions,
            retained_examples: a.evidence.len(),
            active: a.active.as_ref().map(info),
            archives: a.archives.iter().map(info).collect(),
        })
    }
    pub fn phase_adaptive_predict(
        &self,
        action: usize,
        pre: &[f32],
    ) -> Option<PhaseAdaptiveForecast> {
        if !rule_vector_valid(pre, self.config.sensory_cells) {
            return None;
        }
        let adaptive = self
            .phase_native
            .as_ref()?
            .rules
            .as_ref()?
            .adaptive
            .as_ref()?;
        let a = adaptive.actions.get(action)?;
        let m = a.active.as_ref()?;
        let (sensory, error_radius) = adaptive_predict_model(
            a,
            m,
            pre,
            &vec![0.0; pre.len()],
            adaptive.config.residual_tolerance,
            &self.cells,
            &self.synapses,
        )?;
        Some(PhaseAdaptiveForecast {
            sensory,
            error_radius,
            authority: Authority::Imagined,
            revision: a.revision,
            evidence_sources: m.evidence_sources.clone(),
        })
    }
    pub fn phase_adaptive_goal_matches(&self, current: &[f32], goal: &[f32]) -> bool {
        let Some(adaptive) = self
            .phase_native
            .as_ref()
            .and_then(|s| s.rules.as_ref())
            .and_then(|r| r.adaptive.as_ref())
        else {
            return false;
        };
        rule_vector_valid(current, self.config.sensory_cells)
            && rule_vector_valid(goal, self.config.sensory_cells)
            && rule_distance(current, goal) <= adaptive.config.goal_tolerance
    }

    fn adaptive_same_programs(
        &self,
        a: &PhaseAdaptiveAction,
        m: &PhaseAdaptiveModel,
        fit: &AdaptiveFit,
        tolerance: f32,
    ) -> bool {
        m.slots.len() == fit.programs.len()
            && m.slots.iter().zip(&fit.programs).all(|(&s, p)| {
                a.slots[s].outputs.iter().enumerate().all(|(j, o)| {
                    o.candidates.get(p.candidates[j]).is_some_and(|c| c.active)
                        && rule_candidate_phase(
                            &o.candidates[p.candidates[j]],
                            &self.cells,
                            &self.synapses,
                        )
                        .is_some_and(|v| {
                            rule_distance(&[wrap_phase(v) / std::f32::consts::TAU], &[p.offsets[j]])
                                <= tolerance
                        })
                })
            })
    }
    fn adaptive_publish(
        &mut self,
        a: &mut PhaseAdaptiveAction,
        fit: AdaptiveFit,
        sequence: u64,
        cfg: &PhaseAdaptiveConfig,
    ) -> Option<()> {
        if a.active
            .as_ref()
            .is_some_and(|m| self.adaptive_same_programs(a, m, &fit, cfg.residual_tolerance))
        {
            let m = a.active.as_mut()?;
            m.condition = fit.condition;
            m.supports = fit.programs.iter().map(|p| p.support).collect();
            m.evidence_sources = a.evidence.iter().map(|e| e.sequence).collect();
            return Some(());
        }
        let needed = fit.programs.len();
        let mut occupied = a
            .active
            .iter()
            .chain(&a.archives)
            .flat_map(|m| m.slots.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        while a.slots.len() - occupied.len() < needed {
            if a.archives.is_empty() {
                return None;
            }
            let old = a.archives.remove(0);
            for slot in old.slots {
                occupied.remove(&slot);
                for output in &mut a.slots[slot].outputs {
                    for c in &mut output.candidates {
                        suppress_rule_candidate(c, &mut self.synapses);
                    }
                }
            }
            a.evictions = a.evictions.checked_add(1)?;
        }
        let slots = (0..a.slots.len())
            .filter(|s| !occupied.contains(s))
            .take(needed)
            .collect::<Vec<_>>();
        for (&s, p) in slots.iter().zip(&fit.programs) {
            for (j, o) in a.slots[s].outputs.iter_mut().enumerate() {
                for (k, c) in o.candidates.iter_mut().enumerate() {
                    suppress_rule_candidate(c, &mut self.synapses);
                    if k == p.candidates[j] {
                        c.active = true;
                        for i in c.indices() {
                            let link = &mut self.synapses[i];
                            link.phase_offset = if i == c.synapse {
                                wrap_phase(p.offsets[j] * std::f32::consts::TAU)
                            } else {
                                0.0
                            };
                            link.weight = 1.0;
                            link.confidence = 1.0;
                            link.eligibility = 1.0;
                        }
                    }
                }
            }
        }
        if let Some(previous) = a.active.take() {
            a.archives.push(previous);
        }
        a.active = Some(PhaseAdaptiveModel {
            condition: fit.condition,
            slots,
            supports: fit.programs.iter().map(|p| p.support).collect(),
            evidence_sources: a.evidence.iter().map(|e| e.sequence).collect(),
        });
        a.revision = a.revision.checked_add(1)?;
        debug_assert!(a
            .active
            .as_ref()?
            .evidence_sources
            .iter()
            .all(|&s| s <= sequence));
        Some(())
    }
    pub(crate) fn observe_phase_adaptive_result(
        &mut self,
        action: usize,
        pre: &[f32],
        post: &[f32],
    ) -> Option<usize> {
        if !rule_vector_valid(pre, self.config.sensory_cells)
            || !rule_vector_valid(post, self.config.sensory_cells)
        {
            return None;
        }
        let mut native = self.phase_native.take()?;
        let result = (|| {
            if !native.config.learning_enabled {
                return Some(0);
            }
            let rules = native.rules.as_mut()?;
            let templates = rule_templates(rules.language, self.config.sensory_cells);
            let adaptive = rules.adaptive.as_mut()?;
            let cfg = adaptive.config.clone();
            let a = adaptive.actions.get_mut(action)?;
            let sequence = adaptive.factual_sequence.checked_add(1)?;
            let sample = PhaseRuleEvidence {
                sequence,
                pre: pre.to_vec(),
                post: post.to_vec(),
            };
            a.observations = a.observations.checked_add(1)?;
            adaptive.factual_sequence = sequence;
            let zero = vec![0.0; pre.len()];
            let predicted = a.active.as_ref().and_then(|m| {
                adaptive_predict_model(
                    a,
                    m,
                    pre,
                    &zero,
                    cfg.residual_tolerance,
                    &self.cells,
                    &self.synapses,
                )
            });
            let good = predicted
                .as_ref()
                .is_some_and(|(v, _)| rule_distance(v, post) <= cfg.residual_tolerance);
            if !a
                .evidence
                .iter()
                .any(|e| rule_distance(&e.pre, pre) <= 2.0 * cfg.residual_tolerance)
            {
                if a.evidence.len() == cfg.evidence_capacity {
                    a.evidence.remove(0);
                }
                a.evidence.push(sample.clone());
            }
            if good {
                a.pending.clear();
                return Some(0);
            }
            if predicted.is_some() {
                if !a
                    .pending
                    .iter()
                    .any(|e| rule_distance(&e.pre, pre) <= 2.0 * cfg.residual_tolerance)
                {
                    a.pending.push(sample);
                }
                if a.pending.len() >= cfg.change_support {
                    let matching = a
                        .archives
                        .iter()
                        .enumerate()
                        .filter(|(_, m)| {
                            a.pending.iter().all(|e| {
                                adaptive_predict_model(
                                    a,
                                    m,
                                    &e.pre,
                                    &zero,
                                    cfg.residual_tolerance,
                                    &self.cells,
                                    &self.synapses,
                                )
                                .is_some_and(|(v, _)| {
                                    rule_distance(&v, &e.post) <= cfg.residual_tolerance
                                })
                            })
                        })
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();
                    if matching.len() == 1 {
                        let old = a.archives.remove(matching[0]);
                        if let Some(current) = a.active.take() {
                            a.archives.push(current);
                        }
                        a.active = Some(old);
                        a.pending.clear();
                        a.reactivations = a.reactivations.checked_add(1)?;
                        a.revision = a.revision.checked_add(1)?;
                        return Some(1);
                    }
                    if let Some(current) = a.active.take() {
                        a.archives.push(current);
                    }
                    a.evidence = std::mem::take(&mut a.pending);
                    a.revision = a.revision.checked_add(1)?;
                }
            } else {
                // An unsupported context is a coverage gap, not proof that
                // the old law changed. Its existing physical model survives.
                a.pending.clear();
            }
            if let Some(fit) =
                adaptive_fit(&a.evidence, &templates, self.config.sensory_cells, &cfg)
            {
                self.adaptive_publish(a, fit, sequence, &cfg)?;
            }
            Some(0)
        })();
        self.phase_native = Some(native);
        result
    }

    pub fn phase_adaptive_decision(&self, goal: &[f32]) -> Option<PhaseRuleDecision> {
        if !rule_vector_valid(goal, self.config.sensory_cells) {
            return None;
        }
        let native = self.phase_native.as_ref()?;
        let rules = native.rules.as_ref()?;
        let adaptive = rules.adaptive.as_ref()?;
        let start = &self.current_real.as_ref()?.sensory;
        let mut layer = vec![(start.clone(), vec![0.0; start.len()], None)];
        let mut count = 0;
        for depth in 1..=rules.config.planning_depth.min(native.config.horizon) {
            let mut next = Vec::new();
            for (pre, radius, first) in &layer {
                for (action, a) in adaptive.actions.iter().enumerate() {
                    if count == rules.config.node_budget {
                        break;
                    }
                    count += 1;
                    let Some(m) = &a.active else { continue };
                    let Some((post, bounds)) = adaptive_predict_model(
                        a,
                        m,
                        pre,
                        radius,
                        adaptive.config.residual_tolerance,
                        &self.cells,
                        &self.synapses,
                    ) else {
                        continue;
                    };
                    let first = Some(first.unwrap_or(action));
                    if post.iter().zip(goal).zip(&bounds).all(|((&x, &g), &r)| {
                        rule_distance(&[x], &[g]) + r <= adaptive.config.goal_tolerance
                    }) {
                        return Some(PhaseRuleDecision {
                            action: first?,
                            kind: PhaseRuleDecisionKind::GoalPlan,
                            planned_depth: depth,
                            expected_disagreement: 0.0,
                        });
                    }
                    if bounds.iter().all(|&r| r < 0.5) {
                        next.push((post, bounds, first));
                    }
                }
            }
            next.sort_by(|a, b| rule_distance(&a.0, goal).total_cmp(&rule_distance(&b.0, goal)));
            next.truncate(rules.config.beam_width);
            layer = next;
            if layer.is_empty() || count == rules.config.node_budget {
                break;
            }
        }
        if !native.config.learning_enabled {
            return None;
        }
        let action = (0..adaptive.actions.len()).min_by_key(|&a| {
            (
                usize::from(self.phase_adaptive_predict(a, start).is_some()),
                adaptive.actions[a].observations,
                a,
            )
        })?;
        Some(PhaseRuleDecision {
            action,
            kind: PhaseRuleDecisionKind::Experiment,
            planned_depth: 1,
            expected_disagreement: 0.0,
        })
    }
}
