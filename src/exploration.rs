#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbeFeatures {
    pub disagreement: f32,
    pub coverage: f32,
    pub novelty: f32,
}

impl ProbeFeatures {
    pub fn as_array(self) -> [f32; 3] {
        [self.disagreement, self.coverage, self.novelty]
    }
}

#[derive(Debug, Clone)]
pub struct ExplorationConfig {
    pub learning_rate: f32,
    pub learning_enabled: bool,
    pub readout_enabled: bool,
}

impl Default for ExplorationConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.18,
            learning_enabled: true,
            readout_enabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvoExplorationStrategy {
    config: ExplorationConfig,
    weights: [f32; 3],
    bias: f32,
    observations: u32,
}

impl EvoExplorationStrategy {
    pub fn new(config: ExplorationConfig) -> Self {
        assert!(config.learning_rate > 0.0 && config.learning_rate <= 1.0);
        Self {
            config,
            weights: [0.0; 3],
            bias: 0.0,
            observations: 0,
        }
    }

    pub fn weights(&self) -> [f32; 3] {
        self.weights
    }

    pub fn bias(&self) -> f32 {
        self.bias
    }

    pub fn observations(&self) -> u32 {
        self.observations
    }

    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.config.learning_enabled = enabled;
    }

    pub fn set_readout_enabled(&mut self, enabled: bool) {
        self.config.readout_enabled = enabled;
    }

    pub fn score(&self, features: ProbeFeatures) -> f32 {
        let x = features.as_array();
        self.bias
            + self
                .weights
                .iter()
                .zip(x)
                .map(|(w, v)| w * v)
                .sum::<f32>()
    }

    pub fn observe_information_gain(
        &mut self,
        features: ProbeFeatures,
        rivals_before: usize,
        rivals_after: usize,
    ) {
        if !self.config.learning_enabled || rivals_before < 2 {
            return;
        }

        let denom = (rivals_before - 1) as f32;
        let reduction = rivals_before.saturating_sub(rivals_after) as f32;
        let target = (reduction / denom).clamp(0.0, 1.0);

        let prediction = self.score(features);
        let error = target - prediction;
        let lr = self.config.learning_rate;
        let x = features.as_array();

        for (weight, value) in self.weights.iter_mut().zip(x) {
            *weight = (*weight + lr * error * value).clamp(-4.0, 4.0);
        }
        self.bias = (self.bias + lr * error).clamp(-4.0, 4.0);
        self.observations = self.observations.saturating_add(1);
    }

    pub fn choose(&self, features: &[ProbeFeatures]) -> Option<usize> {
        if !self.config.readout_enabled || features.is_empty() {
            return None;
        }

        features
            .iter()
            .enumerate()
            .map(|(action, feature)| (action, self.score(*feature), feature.novelty))
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap()
                    .then_with(|| a.2.partial_cmp(&b.2).unwrap())
                    .then_with(|| b.0.cmp(&a.0))
            })
            .map(|(action, _, _)| action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factual_information_gain_learns_probe_scoring() {
        let mut strategy = EvoExplorationStrategy::new(ExplorationConfig::default());
        assert_eq!(strategy.weights(), [0.0; 3]);

        let weak = ProbeFeatures {
            disagreement: 0.0,
            coverage: 1.0,
            novelty: 1.0,
        };
        let strong = ProbeFeatures {
            disagreement: 1.0,
            coverage: 1.0,
            novelty: 1.0,
        };

        for _ in 0..20 {
            strategy.observe_information_gain(weak, 2, 2);
            strategy.observe_information_gain(strong, 2, 1);
        }

        strategy.set_learning_enabled(false);
        strategy.set_readout_enabled(true);

        assert!(strategy.weights()[0] > 0.1);
        assert_eq!(strategy.choose(&[weak, strong]), Some(1));
    }
}
