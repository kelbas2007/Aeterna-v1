use crate::authority::Authority;
use crate::trace::CarrierTrace;

#[derive(Debug, Clone)]
pub struct PlanningConfig {
    pub motor_cells: usize,
    pub match_threshold: f32,
    pub max_depth: usize,
    pub node_budget: usize,
    pub discount: f32,
    pub learning_enabled: bool,
}

impl PlanningConfig {
    pub fn new(motor_cells: usize) -> Self {
        Self {
            motor_cells,
            match_threshold: 0.97,
            max_depth: 4,
            node_budget: 128,
            discount: 0.95,
            learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LearnedTransition {
    pub from: CarrierTrace,
    pub action: usize,
    pub to: CarrierTrace,
    pub reward_sum: f32,
    pub support: u32,
}

impl LearnedTransition {
    pub fn mean_reward(&self) -> f32 {
        if self.support == 0 {
            0.0
        } else {
            self.reward_sum / self.support as f32
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImaginedNode {
    pub trace: CarrierTrace,
    pub cumulative_value: f32,
    pub depth: usize,
    pub first_action: usize,
    pub authority: Authority,
}

#[derive(Debug, Clone)]
pub struct PlanDecision {
    pub first_action: usize,
    pub predicted_value: f32,
    pub selected_depth: usize,
    pub expanded_nodes: usize,
    pub authority: Authority,
}

#[derive(Debug, Clone)]
pub struct EvoImaginationPlanner {
    config: PlanningConfig,
    transitions: Vec<LearnedTransition>,
    last_rollout: Vec<ImaginedNode>,
}

impl EvoImaginationPlanner {
    pub fn new(config: PlanningConfig) -> Self {
        assert!(config.motor_cells > 0);
        assert!(config.max_depth > 0);
        assert!(config.node_budget > 0);
        assert!((0.0..=1.0).contains(&config.discount));
        Self {
            config,
            transitions: Vec::new(),
            last_rollout: Vec::new(),
        }
    }

    pub fn transitions(&self) -> &[LearnedTransition] {
        &self.transitions
    }

    pub fn last_rollout(&self) -> &[ImaginedNode] {
        &self.last_rollout
    }

    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.config.learning_enabled = enabled;
    }

    pub fn observe_factual_transition(
        &mut self,
        from: CarrierTrace,
        action: usize,
        to: CarrierTrace,
        factual_value: f32,
    ) {
        assert!(action < self.config.motor_cells);
        assert!(factual_value.is_finite());
        if !self.config.learning_enabled {
            return;
        }

        if let Some(edge) = self.transitions.iter_mut().find(|edge| {
            edge.action == action
                && edge.from.similarity(&from) >= self.config.match_threshold
                && edge.to.similarity(&to) >= self.config.match_threshold
        }) {
            edge.reward_sum += factual_value;
            edge.support = edge.support.saturating_add(1);
            return;
        }

        self.transitions.push(LearnedTransition {
            from,
            action,
            to,
            reward_sum: factual_value,
            support: 1,
        });
    }

    pub fn plan(&mut self, start: &CarrierTrace) -> Option<PlanDecision> {
        self.plan_with_depth(start, self.config.max_depth)
    }

    pub fn choose_immediate_model(&mut self, start: &CarrierTrace) -> Option<PlanDecision> {
        self.last_rollout.clear();

        let mut best: Option<PlanDecision> = None;
        let mut considered = 0usize;

        for edge in self.transitions.iter().filter(|edge| {
            edge.from.similarity(start) >= self.config.match_threshold
        }) {
            considered = considered.saturating_add(1);
            let candidate = PlanDecision {
                first_action: edge.action,
                predicted_value: edge.mean_reward(),
                selected_depth: 1,
                expanded_nodes: considered,
                authority: Authority::Model,
            };

            let better = best
                .as_ref()
                .map(|current| {
                    candidate.predicted_value > current.predicted_value + 1.0e-6
                        || ((candidate.predicted_value - current.predicted_value).abs()
                            <= 1.0e-6
                            && candidate.first_action < current.first_action)
                })
                .unwrap_or(true);

            if better {
                best = Some(candidate);
            }
        }

        if let Some(best) = best.as_mut() {
            best.expanded_nodes = considered;
        }
        best
    }

    pub fn plan_with_depth(
        &mut self,
        start: &CarrierTrace,
        depth_limit: usize,
    ) -> Option<PlanDecision> {
        let depth_limit = depth_limit.min(self.config.max_depth).max(1);
        self.last_rollout.clear();

        #[derive(Clone)]
        struct FrontierNode {
            trace: CarrierTrace,
            value: f32,
            depth: usize,
            first_action: Option<usize>,
        }

        let mut frontier = vec![FrontierNode {
            trace: start.clone(),
            value: 0.0,
            depth: 0,
            first_action: None,
        }];

        let mut best: Option<PlanDecision> = None;
        let mut expanded = 0usize;

        while let Some(node) = frontier.pop() {
            if node.depth >= depth_limit || expanded >= self.config.node_budget {
                continue;
            }

            for edge in self.transitions.iter().filter(|edge| {
                edge.from.similarity(&node.trace) >= self.config.match_threshold
            }) {
                if expanded >= self.config.node_budget {
                    break;
                }

                let depth = node.depth + 1;
                let first_action = node.first_action.unwrap_or(edge.action);
                let discounted =
                    self.config.discount.powi((depth.saturating_sub(1)) as i32)
                        * edge.mean_reward();
                let value = node.value + discounted;

                self.last_rollout.push(ImaginedNode {
                    trace: edge.to.clone(),
                    cumulative_value: value,
                    depth,
                    first_action,
                    authority: Authority::Imagined,
                });
                expanded += 1;

                let candidate = PlanDecision {
                    first_action,
                    predicted_value: value,
                    selected_depth: depth,
                    expanded_nodes: expanded,
                    authority: Authority::Imagined,
                };

                let better = best
                    .as_ref()
                    .map(|current| {
                        candidate.predicted_value > current.predicted_value + 1.0e-6
                            || ((candidate.predicted_value - current.predicted_value).abs()
                                <= 1.0e-6
                                && candidate.selected_depth < current.selected_depth)
                    })
                    .unwrap_or(true);

                if better {
                    best = Some(candidate);
                }

                if depth < depth_limit {
                    frontier.push(FrontierNode {
                        trace: edge.to.clone(),
                        value,
                        depth,
                        first_action: Some(first_action),
                    });
                }
            }
        }

        if let Some(best) = best.as_mut() {
            best.expanded_nodes = expanded;
        }
        best
    }

    pub fn permute_successors_for_control(&mut self) {
        if self.transitions.len() < 2 {
            return;
        }
        let successors = self
            .transitions
            .iter()
            .map(|edge| edge.to.clone())
            .collect::<Vec<_>>();
        for (i, edge) in self.transitions.iter_mut().enumerate() {
            edge.to = successors[(i + 1) % successors.len()].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hdc::PhaseVector;

    #[test]
    fn depth_three_can_prefer_delayed_value_over_immediate_value() {
        let a = CarrierTrace::exact(PhaseVector::from_seed(64, 1));
        let b = CarrierTrace::exact(PhaseVector::from_seed(64, 2));
        let c = CarrierTrace::exact(PhaseVector::from_seed(64, 3));
        let d = CarrierTrace::exact(PhaseVector::from_seed(64, 4));
        let trap = CarrierTrace::exact(PhaseVector::from_seed(64, 5));

        let mut planner = EvoImaginationPlanner::new(PlanningConfig {
            motor_cells: 3,
            max_depth: 3,
            ..PlanningConfig::new(3)
        });

        planner.observe_factual_transition(a.clone(), 0, trap, 0.30);
        planner.observe_factual_transition(a.clone(), 1, b.clone(), 0.0);
        planner.observe_factual_transition(b, 2, c.clone(), 0.0);
        planner.observe_factual_transition(c, 2, d, 1.0);

        assert_eq!(planner.plan_with_depth(&a, 1).unwrap().first_action, 0);
        assert_eq!(planner.plan(&a).unwrap().first_action, 1);
        assert!(planner
            .last_rollout()
            .iter()
            .all(|node| node.authority == Authority::Imagined));
    }
}
