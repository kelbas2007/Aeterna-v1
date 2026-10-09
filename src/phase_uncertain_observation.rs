// Circular interval inference over acquired adaptive phase parameters. Intervals
// are conservative engineering bounds, not posterior probabilities. Factual
// measurements remain separate from inferred state and from numeric tuition.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseUncertainConfig {
    pub measurement_radius: f32,
    pub inference_budget: usize,
    pub information_epsilon: f32,
    /// Bounded initial factual exposure to every opaque motor before exploiting
    /// a goal model. Zero explicitly disables this acquisition quota.
    pub exploration_observations: usize,
}
impl Default for PhaseUncertainConfig {
    fn default() -> Self {
        Self {
            measurement_radius: 0.0005,
            inference_budget: 8192,
            information_epsilon: 0.0001,
            exploration_observations: 32,
        }
    }
}
impl PhaseUncertainConfig {
    fn valid(&self) -> bool {
        self.measurement_radius.is_finite()
            && (0.0..=0.01).contains(&self.measurement_radius)
            && (128..=1_000_000).contains(&self.inference_budget)
            && self.information_epsilon.is_finite()
            && (0.000001..=0.05).contains(&self.information_epsilon)
            && self.exploration_observations <= 128
    }
}

/// A closed circular arc. None in a vector represents the whole circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseInterval {
    pub center: f32,
    pub radius: f32,
}
impl PhaseInterval {
    pub fn contains(&self, value: f32) -> bool {
        value.is_finite()
            && (0.0..1.0).contains(&value)
            && rule_distance(&[self.center], &[value]) <= self.radius + 0.000002
    }
    fn valid(&self) -> bool {
        self.center.is_finite()
            && (0.0..1.0).contains(&self.center)
            && self.radius.is_finite()
            && (0.0..0.5).contains(&self.radius)
    }
}
pub type PhaseUncertainVector = Vec<Option<PhaseInterval>>;

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseUncertainInverseReport {
    pub action: usize,
    pub action_revision: u64,
    pub evidence_sources: Vec<u64>,
    pub observed_post: PhasePartialVector,
    pub pre_hypotheses: Vec<PhaseUncertainVector>,
    pub authority: Authority,
    pub widened: bool,
    pub contradicted: bool,
    pub operations: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseUncertainBeliefInfo {
    pub factual: PhasePartialVector,
    pub measurement_radius: f32,
    pub hypotheses: Vec<PhaseUncertainVector>,
    pub authority: Authority,
    pub widened: bool,
    pub inverse: Option<PhaseUncertainInverseReport>,
    pub unproductive_measurements: Vec<usize>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseUncertainForecast {
    pub hypotheses: Vec<PhaseUncertainVector>,
    pub authority: Authority,
    pub action_revision: u64,
    pub evidence_sources: Vec<u64>,
    pub widened: bool,
}
#[derive(Debug, Clone)]
struct PhaseUncertainEpisode {
    hypotheses: Vec<PhaseUncertainVector>,
    inverse: Option<PhaseUncertainInverseReport>,
    unproductive: Vec<bool>,
}

fn uncertain_arc(center: f64, radius: f64) -> Option<PhaseInterval> {
    if radius + 0.0000002 >= 0.5 {
        None
    } else {
        Some(PhaseInterval {
            center: rule_canonical_value(center.rem_euclid(1.0) as f32),
            radius: (radius.max(0.0) + 0.0000002) as f32,
        })
    }
}
fn uncertain_measured(values: &[Option<f32>], radius: f32) -> PhaseUncertainVector {
    values
        .iter()
        .map(|value| value.map(|center| PhaseInterval { center, radius }))
        .collect()
}
fn uncertain_segments(value: Option<PhaseInterval>) -> Vec<(f64, f64)> {
    let Some(value) = value else {
        return vec![(0.0, 1.0)];
    };
    let low = f64::from(value.center) - f64::from(value.radius);
    let high = f64::from(value.center) + f64::from(value.radius);
    if low < 0.0 {
        vec![(0.0, high), (low + 1.0, 1.0)]
    } else if high >= 1.0 {
        vec![(0.0, high - 1.0), (low, 1.0)]
    } else {
        vec![(low, high)]
    }
}
fn uncertain_from_segments(mut segments: Vec<(f64, f64)>) -> Vec<Option<PhaseInterval>> {
    segments.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for (low, high) in segments {
        if let Some(last) = merged.last_mut().filter(|last| low <= last.1 + 0.0000001) {
            last.1 = last.1.max(high);
        } else {
            merged.push((low, high));
        }
    }
    let mut out = Vec::new();
    if merged.len() > 1 && merged[0].0 == 0.0 && merged.last().is_some_and(|s| s.1 == 1.0) {
        let (last_low, _) = merged.pop().unwrap();
        let (_, first_high) = merged.remove(0);
        let span = 1.0 - last_low + first_high;
        out.push(uncertain_arc(last_low + span / 2.0, span / 2.0));
    }
    out.extend(
        merged
            .into_iter()
            .map(|(low, high)| uncertain_arc((low + high) / 2.0, (high - low) / 2.0)),
    );
    out
}
fn uncertain_intersect(
    a: Option<PhaseInterval>,
    b: Option<PhaseInterval>,
) -> Vec<Option<PhaseInterval>> {
    let mut segments = Vec::new();
    for (al, ah) in uncertain_segments(a) {
        for (bl, bh) in uncertain_segments(b) {
            let low = al.max(bl);
            let high = ah.min(bh);
            if low <= high + 0.0000001 {
                segments.push((low.min(high), high));
            }
        }
    }
    uncertain_from_segments(segments)
}
fn uncertain_hull(rows: &[PhaseUncertainVector]) -> PhaseUncertainVector {
    (0..rows[0].len())
        .map(|j| {
            let first = rows[0][j]?;
            let mut radius = first.radius;
            for row in rows {
                let value = row[j]?;
                radius = radius.max(rule_distance(&[first.center], &[value.center]) + value.radius);
            }
            uncertain_arc(f64::from(first.center), f64::from(radius))
        })
        .collect()
}
fn uncertain_bound(rows: &mut Vec<PhaseUncertainVector>, max: usize) -> bool {
    // Only equal boxes are deduplicated. Subsets are not discarded by a
    // tolerance heuristic, which could remove a legitimate modular root.
    let mut unique = Vec::new();
    for row in std::mem::take(rows) {
        if !unique.contains(&row) {
            unique.push(row);
        }
    }
    let widened = unique.len() > max;
    if widened {
        unique = vec![uncertain_hull(&unique)];
    }
    *rows = unique;
    widened
}
fn uncertain_field_coverage(rows: &[PhaseUncertainVector], field: usize) -> f32 {
    let mut segments = rows
        .iter()
        .flat_map(|row| uncertain_segments(row[field]))
        .collect::<Vec<_>>();
    segments.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut end = 0.0_f64;
    let mut total = 0.0_f64;
    for (low, high) in segments {
        total += (high - low.max(end)).max(0.0);
        end = end.max(high);
    }
    (total / 2.0).min(0.5) as f32
}
fn uncertain_goal_matches(
    row: &PhaseUncertainVector,
    goal: &[Option<f32>],
    tolerance: f32,
) -> bool {
    row.iter().zip(goal).all(|(value, target)| match target {
        None => true,
        Some(target) => {
            value.is_some_and(|v| rule_distance(&[v.center], &[*target]) + v.radius <= tolerance)
        }
    })
}

struct UncertainProgram {
    pre: PhaseUncertainVector,
    slot: Option<usize>,
}

impl EvoPhase {
    /// Combines adaptive numeric models with partial sensing. Sources, motor
    /// roles, thresholds, formulas and offsets must already come from tuition
    /// or be acquired after enabling this mode; none are accepted as inputs.
    pub fn enable_phase_uncertain_observation(
        &mut self,
        partial: PhasePartialConfig,
        uncertainty: PhaseUncertainConfig,
    ) -> bool {
        if !partial.valid() || !uncertainty.valid() {
            return false;
        }
        let Some(native) = self.phase_native.as_mut() else {
            return false;
        };
        if native.partial.is_some() || !native.rules.as_ref().is_some_and(|r| r.adaptive.is_some())
        {
            return false;
        }
        native.partial = Some(PhasePartialState {
            masks: (0..self.config.motor_cells)
                .map(|_| PhasePartialMask {
                    visible: vec![false; self.config.sensory_cells],
                    observations: 0,
                    revision: 0,
                    source_ids: Vec::new(),
                })
                .collect(),
            config: partial,
            factual_sequence: 0,
            inverse_enabled: true,
            uncertainty: Some(uncertainty),
            episode: None,
        });
        self.current_real = None;
        true
    }
    pub fn phase_uncertain_observation_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|s| s.partial.as_ref())
            .is_some_and(|p| p.uncertainty.is_some())
    }
    pub fn phase_uncertain_belief(&self) -> Option<PhaseUncertainBeliefInfo> {
        let partial = self.phase_native.as_ref()?.partial.as_ref()?;
        let cfg = partial.uncertainty.as_ref()?;
        let episode = partial.episode.as_ref()?;
        let uncertain = episode.uncertain.as_ref()?;
        Some(PhaseUncertainBeliefInfo {
            factual: episode.factual.clone(),
            measurement_radius: cfg.measurement_radius,
            hypotheses: uncertain.hypotheses.clone(),
            authority: Authority::Imagined,
            widened: episode.widened,
            inverse: uncertain.inverse.clone(),
            unproductive_measurements: uncertain
                .unproductive
                .iter()
                .enumerate()
                .filter_map(|(a, &v)| v.then_some(a))
                .collect(),
        })
    }
    fn observe_phase_uncertain_initial(&mut self, frame: &[Option<f32>]) -> bool {
        let Some(partial) = self.phase_native.as_mut().and_then(|s| s.partial.as_mut()) else {
            return false;
        };
        let Some(cfg) = &partial.uncertainty else {
            return false;
        };
        partial.episode = Some(PhasePartialEpisode {
            factual: frame.to_vec(),
            hypotheses: vec![frame.to_vec()],
            widened: false,
            invalid_rules: vec![false; self.config.motor_cells],
            invalid_masks: vec![false; self.config.motor_cells],
            inverse: None,
            uncertain: Some(PhaseUncertainEpisode {
                hypotheses: vec![uncertain_measured(frame, cfg.measurement_radius)],
                inverse: None,
                unproductive: vec![false; self.config.motor_cells],
            }),
        });
        self.current_real = None;
        true
    }
    fn phase_uncertain_goal_reached(&self, goal: &[Option<f32>]) -> bool {
        let Some(native) = self.phase_native.as_ref() else {
            return false;
        };
        let Some(partial) = native.partial.as_ref() else {
            return false;
        };
        let Some(cfg) = partial.uncertainty.as_ref() else {
            return false;
        };
        let Some(episode) = partial.episode.as_ref() else {
            return false;
        };
        let tolerance = native
            .rules
            .as_ref()
            .unwrap()
            .adaptive
            .as_ref()
            .unwrap()
            .config
            .goal_tolerance;
        partial_vector_valid(goal, self.config.sensory_cells)
            && goal.iter().any(Option::is_some)
            && uncertain_goal_matches(
                &uncertain_measured(&episode.factual, cfg.measurement_radius),
                goal,
                tolerance,
            )
    }
    fn uncertain_model(
        &self,
        action: usize,
    ) -> Option<(&PhaseAdaptiveAction, &PhaseAdaptiveModel)> {
        let a = self
            .phase_native
            .as_ref()?
            .rules
            .as_ref()?
            .adaptive
            .as_ref()?
            .actions
            .get(action)?;
        let m = a.active.as_ref()?;
        // Physical support is checked even when the operand is unobserved.
        for &slot in &m.slots {
            for output in &a.slots[slot].outputs {
                rule_candidate_phase(
                    output.candidates.iter().find(|c| c.active)?,
                    &self.cells,
                    &self.synapses,
                )?;
            }
        }
        Some((a, m))
    }
    fn uncertain_programs(
        &self,
        action: usize,
        pre: &PhaseUncertainVector,
    ) -> Option<Vec<UncertainProgram>> {
        let (_, m) = self.uncertain_model(action)?;
        let native = self.phase_native.as_ref()?;
        let adaptive = native.rules.as_ref()?.adaptive.as_ref()?;
        let cfg = native.partial.as_ref()?.uncertainty.as_ref()?;
        let Some(condition) = &m.condition else {
            return Some(vec![UncertainProgram {
                pre: pre.clone(),
                slot: Some(m.slots[0]),
            }]);
        };
        let guard = adaptive.config.residual_tolerance + cfg.measurement_radius;
        let low = (condition.lower - guard).max(0.0);
        let high = (condition.upper + guard).min(1.0);
        let domains = [
            (0.0, f64::from(low), Some(m.slots[0])),
            (f64::from(high), 1.0, Some(m.slots[1])),
            (f64::from(low), f64::from(high), None),
        ];
        let mut out = Vec::new();
        for (lower, upper, slot) in domains {
            if lower == upper {
                continue;
            }
            let domain = uncertain_arc((lower + upper) / 2.0, (upper - lower) / 2.0);
            for value in uncertain_intersect(pre[condition.source], domain) {
                let mut row = pre.clone();
                row[condition.source] = value;
                out.push(UncertainProgram { pre: row, slot });
            }
        }
        Some(out)
    }
    fn uncertain_model_error(&self, candidate: &PhaseRuleCandidate) -> f32 {
        let native = self.phase_native.as_ref().unwrap();
        let cfg = native
            .partial
            .as_ref()
            .unwrap()
            .uncertainty
            .as_ref()
            .unwrap();
        let residual = native
            .rules
            .as_ref()
            .unwrap()
            .adaptive
            .as_ref()
            .unwrap()
            .config
            .residual_tolerance;
        // Offset error covers measured training operands and measured POST in
        // addition to the residual accepted by the bounded fitter.
        residual
            + cfg.measurement_radius
                * (1.0
                    + candidate
                        .formula()
                        .terms
                        .iter()
                        .map(|t| f32::from(t.coefficient.unsigned_abs()))
                        .sum::<f32>())
    }
    fn uncertain_output(
        &self,
        candidate: &PhaseRuleCandidate,
        pre: &PhaseUncertainVector,
    ) -> Option<Option<PhaseInterval>> {
        let phase = rule_candidate_phase(candidate, &self.cells, &self.synapses)?;
        let mut center = f64::from(phase) / f64::from(std::f32::consts::TAU);
        let mut radius = f64::from(self.uncertain_model_error(candidate));
        for term in candidate.formula().terms {
            let Some(value) = pre[term.source] else {
                return Some(None);
            };
            center += f64::from(term.coefficient) * f64::from(value.center);
            radius += f64::from(term.coefficient.unsigned_abs()) * f64::from(value.radius);
        }
        Some(uncertain_arc(center, radius))
    }
    fn uncertain_transition(
        &self,
        action: usize,
        input: &[PhaseUncertainVector],
        budget: &mut usize,
    ) -> Option<(Vec<PhaseUncertainVector>, bool)> {
        let width = self.config.sensory_cells;
        let native = self.phase_native.as_ref()?;
        let partial = native.partial.as_ref()?;
        let (a, _) = self.uncertain_model(action)?;
        let mut out = Vec::new();
        let mut widened = false;
        for pre in input {
            for program in self.uncertain_programs(action, pre)? {
                let Some(slot) = program.slot else {
                    out.push(vec![None; width]);
                    continue;
                };
                let mut row = Vec::new();
                for output in &a.slots[slot].outputs {
                    if *budget == 0 {
                        return Some((vec![vec![None; width]], true));
                    }
                    *budget -= 1;
                    row.push(self.uncertain_output(
                        output.candidates.iter().find(|c| c.active)?,
                        &program.pre,
                    )?);
                }
                out.push(row);
            }
            widened |= uncertain_bound(&mut out, partial.config.max_hypotheses);
        }
        Some((out, widened))
    }
    pub fn phase_uncertain_predict(
        &self,
        action: usize,
        input: &[PhaseUncertainVector],
    ) -> Option<PhaseUncertainForecast> {
        let partial = self.phase_native.as_ref()?.partial.as_ref()?;
        let cfg = partial.uncertainty.as_ref()?;
        if input.is_empty()
            || input.len() > partial.config.max_hypotheses
            || input.iter().any(|row| {
                row.len() != self.config.sensory_cells || row.iter().flatten().any(|v| !v.valid())
            })
        {
            return None;
        }
        let (a, m) = self.uncertain_model(action)?;
        let mut budget = cfg.inference_budget;
        let (hypotheses, widened) = self.uncertain_transition(action, input, &mut budget)?;
        Some(PhaseUncertainForecast {
            hypotheses,
            widened,
            authority: Authority::Imagined,
            action_revision: a.revision,
            evidence_sources: m.evidence_sources.clone(),
        })
    }
}

impl EvoPhase {
    fn uncertain_refine_candidate(
        &self,
        candidate: &PhaseRuleCandidate,
        input: &[PhaseUncertainVector],
        observed: PhaseInterval,
        budget: &mut usize,
        max: usize,
    ) -> Option<(Vec<PhaseUncertainVector>, bool)> {
        let phase = f64::from(rule_candidate_phase(
            candidate,
            &self.cells,
            &self.synapses,
        )?) / f64::from(std::f32::consts::TAU);
        let formula = candidate.formula();
        let mut rows = input.to_vec();
        let mut widened = false;
        // Substitution works with bounded operands too. Two wholly unknown
        // operands are left unresolved until an independent equation exists.
        for target in &formula.terms {
            let mut refined = Vec::new();
            for row in &rows {
                if *budget == 0 {
                    return Some((vec![vec![None; self.config.sensory_cells]], true));
                }
                *budget -= 1;
                let mut center = f64::from(observed.center) - phase;
                let mut radius = f64::from(observed.radius + self.uncertain_model_error(candidate));
                let mut supported = true;
                for term in formula
                    .terms
                    .iter()
                    .filter(|term| term.source != target.source)
                {
                    if let Some(value) = row[term.source] {
                        center -= f64::from(term.coefficient) * f64::from(value.center);
                        radius +=
                            f64::from(term.coefficient.unsigned_abs()) * f64::from(value.radius);
                    } else {
                        supported = false;
                        break;
                    }
                }
                if !supported || radius >= 0.5 {
                    refined.push(row.clone());
                    continue;
                }
                let rhs = center.rem_euclid(1.0);
                let gain = target.coefficient.unsigned_abs();
                for k in 0..gain {
                    let root = uncertain_arc(
                        f64::from(target.coefficient.signum()) * (rhs + f64::from(k))
                            / f64::from(gain),
                        radius / f64::from(gain),
                    );
                    for value in uncertain_intersect(row[target.source], root) {
                        let mut branch = row.clone();
                        branch[target.source] = value;
                        refined.push(branch);
                    }
                }
            }
            widened |= uncertain_bound(&mut refined, max);
            rows = refined;
        }
        let mut compatible = Vec::new();
        for row in rows {
            if *budget == 0 {
                return Some((vec![vec![None; self.config.sensory_cells]], true));
            }
            *budget -= 1;
            if !uncertain_intersect(self.uncertain_output(candidate, &row)?, Some(observed))
                .is_empty()
            {
                compatible.push(row);
            }
        }
        Some((compatible, widened))
    }
    fn uncertain_refine_pair(
        &self,
        a: &PhaseRuleCandidate,
        b: &PhaseRuleCandidate,
        input: &[PhaseUncertainVector],
        observed_a: PhaseInterval,
        observed_b: PhaseInterval,
        budget: &mut usize,
        max: usize,
    ) -> Option<(Vec<PhaseUncertainVector>, bool)> {
        let fa = a.formula();
        let fb = b.formula();
        if fa.terms.len() != 2 || fb.terms.len() != 2 {
            return Some((input.to_vec(), false));
        }
        let x = fa.terms[0].source;
        let y = fa.terms[1].source;
        let coefficient = |formula: &PhaseRuleFormula, source| {
            formula
                .terms
                .iter()
                .find(|t| t.source == source)
                .map(|t| i32::from(t.coefficient))
        };
        let Some(bx) = coefficient(&fb, x) else {
            return Some((input.to_vec(), false));
        };
        let Some(by) = coefficient(&fb, y) else {
            return Some((input.to_vec(), false));
        };
        let ax = i32::from(fa.terms[0].coefficient);
        let ay = i32::from(fa.terms[1].coefficient);
        let determinant = ax * by - ay * bx;
        if determinant == 0 {
            return Some((input.to_vec(), false));
        }
        let pa = f64::from(rule_candidate_phase(a, &self.cells, &self.synapses)?)
            / f64::from(std::f32::consts::TAU);
        let pb = f64::from(rule_candidate_phase(b, &self.cells, &self.synapses)?)
            / f64::from(std::f32::consts::TAU);
        let ca = (f64::from(observed_a.center) - pa).rem_euclid(1.0);
        let cb = (f64::from(observed_b.center) - pb).rem_euclid(1.0);
        let ra = f64::from(observed_a.radius + self.uncertain_model_error(a));
        let rb = f64::from(observed_b.radius + self.uncertain_model_error(b));
        let den = f64::from(determinant.abs());
        let rx = (f64::from(by.abs()) * ra + f64::from(ay.abs()) * rb) / den;
        let ry = (f64::from(bx.abs()) * ra + f64::from(ax.abs()) * rb) / den;
        if rx >= 0.5 || ry >= 0.5 {
            return Some((input.to_vec(), true));
        }
        let mut out = Vec::new();
        for row in input {
            for k in 0..determinant.unsigned_abs() {
                for l in 0..determinant.unsigned_abs() {
                    if *budget == 0 {
                        return Some((vec![vec![None; self.config.sensory_cells]], true));
                    }
                    *budget -= 1;
                    let rhs_a = ca + f64::from(k);
                    let rhs_b = cb + f64::from(l);
                    let vx = uncertain_arc(
                        (f64::from(by) * rhs_a - f64::from(ay) * rhs_b) / f64::from(determinant),
                        rx,
                    );
                    let vy = uncertain_arc(
                        (f64::from(ax) * rhs_b - f64::from(bx) * rhs_a) / f64::from(determinant),
                        ry,
                    );
                    for ix in uncertain_intersect(row[x], vx) {
                        for iy in uncertain_intersect(row[y], vy) {
                            let mut branch = row.clone();
                            branch[x] = ix;
                            branch[y] = iy;
                            out.push(branch);
                        }
                    }
                }
            }
        }
        let widened = uncertain_bound(&mut out, max);
        Some((out, widened))
    }
    fn uncertain_inverse_transition(
        &self,
        action: usize,
        input: &[PhaseUncertainVector],
        post: &[Option<f32>],
    ) -> Option<(
        Vec<PhaseUncertainVector>,
        bool,
        Option<PhaseUncertainInverseReport>,
    )> {
        let native = self.phase_native.as_ref()?;
        let partial = native.partial.as_ref()?;
        let cfg = partial.uncertainty.as_ref()?;
        let width = self.config.sensory_cells;
        let Some((a, m)) = self.uncertain_model(action) else {
            return Some((vec![vec![None; width]], false, None));
        };
        let mut budget = cfg.inference_budget;
        if partial.episode.as_ref()?.invalid_rules[action] {
            return Some((vec![vec![None; width]], false, None));
        }
        if !partial.inverse_enabled {
            let (rows, widened) = self.uncertain_transition(action, input, &mut budget)?;
            return Some((rows, widened, None));
        }
        let measured = uncertain_measured(post, cfg.measurement_radius);
        let mut recovered = Vec::new();
        let mut widened = false;
        'programs: for pre in input {
            for program in self.uncertain_programs(action, pre)? {
                let Some(slot) = program.slot else {
                    recovered.push(program.pre);
                    continue;
                };
                let outputs = &a.slots[slot].outputs;
                let mut rows = vec![program.pre];
                for _ in 0..width {
                    let before = rows.clone();
                    for (j, observed) in measured.iter().enumerate() {
                        let Some(observed) = observed else { continue };
                        let candidate = outputs[j].candidates.iter().find(|c| c.active)?;
                        let (refined, did_widen) = self.uncertain_refine_candidate(
                            candidate,
                            &rows,
                            *observed,
                            &mut budget,
                            partial.config.max_hypotheses,
                        )?;
                        rows = refined;
                        widened |= did_widen;
                        for (k, second) in measured.iter().enumerate().skip(j + 1) {
                            let Some(second) = second else { continue };
                            let other = outputs[k].candidates.iter().find(|c| c.active)?;
                            let (refined, did_widen) = self.uncertain_refine_pair(
                                candidate,
                                other,
                                &rows,
                                *observed,
                                *second,
                                &mut budget,
                                partial.config.max_hypotheses,
                            )?;
                            rows = refined;
                            widened |= did_widen;
                        }
                        if budget == 0 {
                            // Include every unprocessed input/program. A hard
                            // budget cannot turn an incomplete search into proof.
                            recovered = vec![vec![None; width]];
                            widened = true;
                            break 'programs;
                        }
                    }
                    if rows.is_empty() || rows == before {
                        break;
                    }
                }
                recovered.extend(rows);
                widened |= uncertain_bound(&mut recovered, partial.config.max_hypotheses);
            }
        }
        let mut report = PhaseUncertainInverseReport {
            action,
            action_revision: a.revision,
            evidence_sources: m.evidence_sources.clone(),
            observed_post: post.to_vec(),
            pre_hypotheses: recovered.clone(),
            authority: Authority::Imagined,
            widened,
            contradicted: recovered.is_empty(),
            operations: cfg.inference_budget - budget,
        };
        if recovered.is_empty() {
            return Some((Vec::new(), widened, Some(report)));
        }
        let (predicted, forward_widened) =
            self.uncertain_transition(action, &recovered, &mut budget)?;
        report.operations = cfg.inference_budget - budget;
        report.widened |= forward_widened;
        Some((predicted, widened || forward_widened, Some(report)))
    }
    fn observe_phase_uncertain_result(
        &mut self,
        action: usize,
        post: &[Option<f32>],
    ) -> Option<PhasePartialUpdate> {
        if !partial_vector_valid(post, self.config.sensory_cells)
            || action >= self.config.motor_cells
        {
            return None;
        }
        let native = self.phase_native.as_ref()?;
        let partial = native.partial.as_ref()?;
        let cfg = partial.uncertainty.as_ref()?.clone();
        let old = partial.episode.as_ref()?.clone();
        let uncertain = old.uncertain.as_ref()?;
        let learning = native.config.learning_enabled;
        let residual_tolerance = native
            .rules
            .as_ref()?
            .adaptive
            .as_ref()?
            .config
            .residual_tolerance;
        let max = partial.config.max_hypotheses;
        let mask_before = self.phase_partial_mask_info(action)?;
        let (predicted, mut widened, mut inverse) =
            self.uncertain_inverse_transition(action, &uncertain.hypotheses, post)?;
        let predicted_count = predicted.len();
        let measured = uncertain_measured(post, cfg.measurement_radius);
        let mut compatible = predicted;
        for (j, factual) in measured.iter().enumerate() {
            if factual.is_none() {
                continue;
            }
            let mut next = Vec::new();
            for row in &compatible {
                for value in uncertain_intersect(row[j], *factual) {
                    let mut branch = row.clone();
                    branch[j] = value;
                    next.push(branch);
                }
            }
            widened |= uncertain_bound(&mut next, max);
            compatible = next;
        }
        let conflict = compatible.is_empty();
        if conflict {
            compatible.push(measured.clone());
        }
        if let Some(report) = inverse.as_mut() {
            report.contradicted |= conflict;
        }
        let mut suppressed = predicted_count.saturating_sub(compatible.len());
        let mut learned = false;
        let mut reconfirmed = false;
        if learning {
            // No centers from inferred intervals may substitute for actual
            // full PRE/POST. Missing training channels remain missing tuition.
            if let (Some(pre), Some(full_post)) = (
                old.factual.iter().copied().collect::<Option<Vec<_>>>(),
                post.iter().copied().collect::<Option<Vec<_>>>(),
            ) {
                suppressed += self.observe_phase_adaptive_result(action, &pre, &full_post)?;
                learned = true;
                reconfirmed = self.phase_adaptive_predict(action, &pre).is_some_and(|p| {
                    p.sensory.iter().zip(&full_post).zip(&p.error_radius).all(
                        |((&expected, &actual), &radius)| {
                            rule_distance(&[expected], &[actual])
                                <= radius + 2.0 * cfg.measurement_radius
                        },
                    )
                });
                // A factual full POST is sufficient regardless of whether a
                // newly revised model invalidates the previous imagined path.
                compatible = vec![measured.clone()];
            }
        }
        let mut unproductive = uncertain.unproductive.clone();
        let old_coverage = (0..self.config.sensory_cells)
            .map(|j| uncertain_field_coverage(&uncertain.hypotheses, j))
            .sum::<f32>();
        let new_coverage = (0..self.config.sensory_cells)
            .map(|j| uncertain_field_coverage(&compatible, j))
            .sum::<f32>();
        let changed_frame = old.factual.iter().zip(post).any(|(a, b)| match (a, b) {
            (Some(a), Some(b)) => {
                rule_distance(&[*a], &[*b]) > 2.0 * cfg.measurement_radius + residual_tolerance
            }
            (None, Some(_)) => true,
            _ => false,
        });
        if old_coverage - new_coverage > cfg.information_epsilon || changed_frame {
            unproductive.fill(false);
        } else {
            unproductive[action] = true;
        }
        let partial = self.phase_native.as_mut()?.partial.as_mut()?;
        let visible = post.iter().map(Option::is_some).collect::<Vec<_>>();
        let mask_conflict = mask_before.confirmed && mask_before.visible != visible;
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
        if reconfirmed {
            invalid_rules[action] = false;
        }
        if learning && partial.masks[action].source_ids.len() >= partial.config.min_mask_support {
            invalid_masks[action] = false;
        }
        partial.episode = Some(PhasePartialEpisode {
            factual: post.to_vec(),
            hypotheses: compatible.iter().map(|_| post.to_vec()).collect(),
            widened: old.widened || widened,
            invalid_rules,
            invalid_masks,
            inverse: None,
            uncertain: Some(PhaseUncertainEpisode {
                hypotheses: compatible,
                inverse,
                unproductive,
            }),
        });
        self.current_real = None;
        Some(PhasePartialUpdate {
            suppressed,
            learned,
        })
    }
}
