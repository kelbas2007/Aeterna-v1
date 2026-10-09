// Bounded robust search: every retained symbolic alternative must satisfy the
// goal. An unobserved endpoint cannot certify success. Sensing roles are learned
// from factual output masks, never supplied as motor labels.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhasePartialDecisionKind {
    GoalPlan,
    InformationGathering,
    MaskExploration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhasePartialDecision {
    pub action: usize,
    pub kind: PhasePartialDecisionKind,
    pub planned_depth: usize,
    pub expected_information: usize,
}

#[derive(Clone)]
struct PartialSearchNode {
    hypotheses: Vec<PhasePartialVector>,
    first: Option<usize>,
}

fn partial_goal_distance(rows: &[PhasePartialVector], goal: &[Option<f32>]) -> f32 {
    rows.iter()
        .flat_map(|row| {
            row.iter().zip(goal).filter_map(|(value, target)| {
                target.map(|target| value.map_or(0.5, |value| rule_distance(&[value], &[target])))
            })
        })
        .fold(0.0, f32::max)
}

impl EvoPhase {
    fn partial_plan(&self, goal: &[Option<f32>]) -> Option<PhasePartialDecision> {
        let native = self.phase_native.as_ref()?;
        let cfg = &native.rules.as_ref()?.config;
        let episode = native.partial.as_ref()?.episode.as_ref()?;
        let supported = (0..self.config.motor_cells)
            .filter(|&a| self.partial_action_supported(a))
            .collect::<Vec<_>>();
        let key = |rows: &[PhasePartialVector]| {
            let mut values = rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|value| value.map_or(-1, |x| (x / cfg.tolerance).round() as i32))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            values.sort();
            values.dedup();
            values
        };
        let mut visited = std::collections::BTreeSet::new();
        visited.insert(key(&episode.hypotheses));
        let mut layer = vec![PartialSearchNode {
            hypotheses: episode.hypotheses.clone(),
            first: None,
        }];
        let mut evaluations = 0;
        for depth in 1..=cfg.planning_depth.min(native.config.horizon) {
            let mut next = Vec::new();
            for node in &layer {
                for &action in &supported {
                    if evaluations == cfg.node_budget {
                        return None;
                    }
                    evaluations += 1;
                    let (hypotheses, _) = self.partial_transition(action, &node.hypotheses)?;
                    let mask = &native.partial.as_ref()?.masks[action].visible;
                    if goal
                        .iter()
                        .zip(mask)
                        .all(|(g, &visible)| g.is_none() || visible)
                        && hypotheses
                            .iter()
                            .all(|row| partial_goal_matches(row, goal, cfg.tolerance))
                    {
                        return Some(PhasePartialDecision {
                            action: node.first.unwrap_or(action),
                            kind: PhasePartialDecisionKind::GoalPlan,
                            planned_depth: depth,
                            expected_information: 0,
                        });
                    }
                    if visited.insert(key(&hypotheses)) {
                        next.push(PartialSearchNode {
                            hypotheses,
                            first: Some(node.first.unwrap_or(action)),
                        });
                    }
                }
            }
            next.sort_by(|a, b| {
                partial_goal_distance(&a.hypotheses, goal)
                    .total_cmp(&partial_goal_distance(&b.hypotheses, goal))
                    .then_with(|| a.first.cmp(&b.first))
            });
            next.truncate(cfg.beam_width);
            if next.is_empty() {
                return None;
            }
            layer = next;
        }
        None
    }

    fn partial_information(&self, action: usize) -> Option<usize> {
        let native = self.phase_native.as_ref()?;
        let partial = native.partial.as_ref()?;
        let episode = partial.episode.as_ref()?;
        let tolerance = native.rules.as_ref()?.config.tolerance;
        let (predicted, _) = self.partial_transition(action, &episode.hypotheses)?;
        let mask = &partial.masks[action].visible;
        let mut information = 0;
        for (j, &visible) in mask.iter().enumerate() {
            if !visible {
                continue;
            }
            let first = predicted[0][j];
            if first.is_none()
                || predicted.iter().any(|row| match (first, row[j]) {
                    (Some(a), Some(b)) => rule_distance(&[a], &[b]) > tolerance,
                    _ => true,
                })
            {
                information += 1;
            }
        }
        Some(information)
    }

    /// Frozen model may gather factual state information using supported
    /// actions. It cannot learn a new motor law or output mask in that mode.
    pub fn phase_partial_decision(&self, goal: &[Option<f32>]) -> Option<PhasePartialDecision> {
        if self.phase_uncertain_observation_enabled() {
            return self.phase_uncertain_decision(goal);
        }
        let native = self.phase_native.as_ref()?;
        let partial = native.partial.as_ref()?;
        partial.episode.as_ref()?;
        if !partial_vector_valid(goal, self.config.sensory_cells)
            || !goal.iter().any(Option::is_some)
        {
            return None;
        }
        if self.phase_partial_goal_reached(goal) {
            return None;
        }
        if let Some(plan) = self.partial_plan(goal) {
            return Some(plan);
        }
        if native.config.learning_enabled {
            if (0..self.config.motor_cells)
                .any(|a| !self.phase_rule_action_info(a).is_some_and(|i| i.confirmed))
            {
                if let Some(factual) = partial
                    .episode
                    .as_ref()?
                    .factual
                    .iter()
                    .copied()
                    .collect::<Option<Vec<_>>>()
                {
                    let inquiry = self.phase_rule_inquiry_from(&factual)?;
                    return Some(PhasePartialDecision {
                        action: inquiry.action,
                        kind: PhasePartialDecisionKind::MaskExploration,
                        planned_depth: inquiry.planned_depth,
                        expected_information: 0,
                    });
                }
            }
            let unknown = (0..self.config.motor_cells)
                .filter(|&a| !self.partial_action_supported(a))
                .min_by_key(|&a| (partial.masks[a].observations, a));
            if let Some(action) = unknown {
                return Some(PhasePartialDecision {
                    action,
                    kind: PhasePartialDecisionKind::MaskExploration,
                    planned_depth: 1,
                    expected_information: 0,
                });
            }
        }
        let mut best = None;
        for action in 0..self.config.motor_cells {
            if !self.partial_action_supported(action) {
                continue;
            }
            let information = self.partial_information(action)?;
            if information > 0 && best.is_none_or(|(_, old)| information > old) {
                best = Some((action, information));
            }
        }
        best.map(|(action, information)| PhasePartialDecision {
            action,
            kind: PhasePartialDecisionKind::InformationGathering,
            planned_depth: 1,
            expected_information: information,
        })
    }
}
