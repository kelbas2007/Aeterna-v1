// Explicit bounded software search over predictions read from acquired phase
// parameters. No world simulator, route curriculum or goal-derived fact enters
// the carrier. This algorithm is inherited, not an acquired exploration law.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseRuleDecisionKind {
    GoalPlan,
    Experiment,
    MoveToExperiment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseRuleDecision {
    pub action: usize,
    pub kind: PhaseRuleDecisionKind,
    pub planned_depth: usize,
    pub expected_disagreement: f32,
}

#[derive(Clone)]
struct RuleSearchNode {
    sensory: Vec<f32>,
    first_action: Option<usize>,
}

impl EvoPhase {
    fn phase_rule_plan(&self, input: &[f32], goal: &[f32]) -> Option<PhaseRuleDecision> {
        let native = self.phase_native.as_ref()?;
        let rules = native.rules.as_ref()?;
        let cfg = &rules.config;
        let confirmed = (0..self.config.motor_cells)
            .filter(|&action| {
                self.phase_rule_action_info(action)
                    .is_some_and(|i| i.confirmed)
            })
            .collect::<Vec<_>>();
        if confirmed.is_empty() {
            return None;
        }
        let key = |xs: &[f32]| {
            xs.iter()
                .map(|x| (x / cfg.tolerance).round() as i32)
                .collect::<Vec<_>>()
        };
        let mut visited = std::collections::BTreeSet::new();
        visited.insert(key(input));
        let mut layer = vec![RuleSearchNode {
            sensory: input.to_vec(),
            first_action: None,
        }];
        let mut evaluations = 0;
        for depth in 1..=cfg.planning_depth.min(native.config.horizon) {
            let mut next = Vec::new();
            for node in &layer {
                for &action in &confirmed {
                    if evaluations == cfg.node_budget {
                        return None;
                    }
                    evaluations += 1;
                    let forecast = self.phase_rule_predict(action, &node.sensory)?;
                    let first_action = node.first_action.unwrap_or(action);
                    if rule_distance(&forecast.sensory, goal) <= cfg.tolerance {
                        return Some(PhaseRuleDecision {
                            action: first_action,
                            kind: PhaseRuleDecisionKind::GoalPlan,
                            planned_depth: depth,
                            expected_disagreement: 0.0,
                        });
                    }
                    if visited.insert(key(&forecast.sensory)) {
                        next.push(RuleSearchNode {
                            sensory: forecast.sensory,
                            first_action: Some(first_action),
                        });
                    }
                }
            }
            next.sort_by(|a, b| {
                rule_distance(&a.sensory, goal)
                    .total_cmp(&rule_distance(&b.sensory, goal))
                    .then_with(|| a.first_action.cmp(&b.first_action))
            });
            next.truncate(cfg.beam_width);
            if next.is_empty() {
                return None;
            }
            layer = next;
        }
        None
    }

    /// Expected split among surviving hypotheses at a real or imagined input.
    /// Alternatives are not weighted as probabilities or treated as facts.
    pub fn phase_rule_disagreement(&self, action: usize, input: &[f32]) -> Option<f32> {
        let alternatives = self.phase_rule_hypotheses(action, input)?;
        let mut score = 0.0_f32;
        for values in alternatives {
            for (i, &a) in values.iter().enumerate() {
                for &b in &values[i + 1..] {
                    score = score.max(rule_distance(&[a], &[b]));
                }
            }
        }
        Some((score * 2.0).clamp(0.0, 1.0))
    }

    fn phase_rule_experiment_at(&self, input: &[f32]) -> Option<(usize, f32, f32)> {
        let rules = self.phase_native.as_ref()?.rules.as_ref()?;
        let mut best: Option<(usize, f32, f32, u64)> = None;
        for action in 0..self.config.motor_cells {
            let info = self.phase_rule_action_info(action)?;
            if info.confirmed {
                continue;
            }
            let disagreement = self.phase_rule_disagreement(action, input)?;
            let model = &rules.actions[action];
            let novel_pre = !model
                .evidence
                .iter()
                .any(|e| rule_distance(&e.pre, input) <= rules.config.tolerance);
            let debt = rules
                .config
                .min_distinct_support
                .saturating_sub(info.distinct_support) as f32
                / rules.config.min_distinct_support as f32;
            // Unknown actions and actual discriminating experiments precede
            // mere repetition. Distinct PRE support, not repeated ticks, counts.
            let score = if info.observations == 0 {
                2.0
            } else {
                1.0 + disagreement + if novel_pre { debt * 0.25 } else { 0.0 }
            };
            if best.is_none_or(|(old_action, old_score, _, old_n)| {
                score > old_score + 0.000001
                    || ((score - old_score).abs() <= 0.000001
                        && (info.observations, action) < (old_n, old_action))
            }) {
                best = Some((action, score, disagreement, info.observations));
            }
        }
        best.map(|(a, s, d, _)| (a, s, d))
    }

    /// Chooses a complete confirmed goal route, otherwise an informative
    /// factual experiment. Frozen mode never explores an unsupported rule.
    pub fn phase_rule_decision(&self, goal: &[f32]) -> Option<PhaseRuleDecision> {
        let native = self.phase_native.as_ref()?;
        native.rules.as_ref()?;
        let input = &self.current_real.as_ref()?.sensory;
        if !rule_vector_valid(input, self.config.sensory_cells)
            || !rule_vector_valid(goal, self.config.sensory_cells)
        {
            return None;
        }
        if self.phase_rule_goal_matches(input, goal) {
            return None;
        }
        if let Some(plan) = self.phase_rule_plan(input, goal) {
            return Some(plan);
        }
        if !native.config.learning_enabled {
            return None;
        }
        self.phase_rule_inquiry_from(input)
    }

    // Shared inquiry for fully factual inputs, including a fully visible
    // frame in partial mode. It is never called with imputed hidden channels.
    fn phase_rule_inquiry_from(&self, input: &[f32]) -> Option<PhaseRuleDecision> {
        let (action, score, disagreement) = self.phase_rule_experiment_at(input)?;
        let mut decision = PhaseRuleDecision {
            action,
            kind: PhaseRuleDecisionKind::Experiment,
            planned_depth: 1,
            expected_disagreement: disagreement,
        };
        let mut best_score = score;
        // One confirmed move may expose a better discrimination point. It
        // cannot manufacture that observation; the next call replans from POST.
        for motor in 0..self.config.motor_cells {
            if let Some(forecast) = self.phase_rule_predict(motor, input) {
                if let Some((_, future_score, future_disagreement)) =
                    self.phase_rule_experiment_at(&forecast.sensory)
                {
                    let discounted = future_score - 0.05;
                    if discounted > best_score + 0.000001 {
                        best_score = discounted;
                        decision = PhaseRuleDecision {
                            action: motor,
                            kind: PhaseRuleDecisionKind::MoveToExperiment,
                            planned_depth: 2,
                            expected_disagreement: future_disagreement,
                        };
                    }
                }
            }
        }
        Some(decision)
    }
}
