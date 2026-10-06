use crate::hdc::PhaseVector;
use crate::trace::CarrierTrace;

#[derive(Debug, Clone)]
pub struct BeliefConfig {
    pub motor_cells: usize,
    pub hdc_dim: usize,
    pub history_permutation: usize,
}

impl BeliefConfig {
    pub fn new(motor_cells: usize, hdc_dim: usize) -> Self {
        Self {
            motor_cells,
            hdc_dim,
            history_permutation: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvoBeliefState {
    config: BeliefConfig,
    action_roles: Vec<PhaseVector>,
    current: Option<PhaseVector>,
    steps: u64,
}

impl EvoBeliefState {
    pub fn new(config: BeliefConfig) -> Self {
        assert!(config.motor_cells > 0);
        assert!(config.hdc_dim > 0);
        assert!(config.history_permutation > 0);

        let action_roles = (0..config.motor_cells)
            .map(|action| {
                PhaseVector::from_seed(
                    config.hdc_dim,
                    0xB311_EF00_0000_0000u64 ^ action as u64,
                )
            })
            .collect();

        Self {
            config,
            action_roles,
            current: None,
            steps: 0,
        }
    }

    pub fn reset(&mut self, observation: PhaseVector) -> CarrierTrace {
        assert_eq!(observation.dim(), self.config.hdc_dim);
        self.current = Some(observation.clone());
        self.steps = 0;
        CarrierTrace::exact(observation)
    }

    pub fn advance(
        &mut self,
        action: usize,
        observation: PhaseVector,
    ) -> CarrierTrace {
        assert!(action < self.config.motor_cells);
        assert_eq!(observation.dim(), self.config.hdc_dim);

        let previous = self
            .current
            .as_ref()
            .expect("belief advance requires reset/current state");
        let history = previous.permute_dims(self.config.history_permutation);
        let next = history
            .bind(&self.action_roles[action])
            .bind(&observation);

        self.current = Some(next.clone());
        self.steps = self.steps.saturating_add(1);
        CarrierTrace::exact(next)
    }

    pub fn current_trace(&self) -> Option<CarrierTrace> {
        self.current.clone().map(CarrierTrace::exact)
    }

    pub fn steps(&self) -> u64 {
        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_current_observation_can_retain_different_histories() {
        let cfg = BeliefConfig::new(4, 128);
        let mut a = EvoBeliefState::new(cfg.clone());
        let mut b = EvoBeliefState::new(cfg);

        let cue_a = PhaseVector::from_seed(128, 1);
        let cue_b = PhaseVector::from_seed(128, 2);
        let corridor = PhaseVector::from_seed(128, 3);

        a.reset(cue_a);
        b.reset(cue_b);
        let belief_a = a.advance(1, corridor.clone());
        let belief_b = b.advance(1, corridor);

        assert!(
            belief_a.similarity(&belief_b) < 0.95,
            "recurrent carrier state must preserve history through an aliased observation"
        );
    }
}
