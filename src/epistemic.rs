use crate::hdc::PhaseVector;

#[derive(Debug, Clone)]
pub struct HypothesisPrediction {
    pub action: usize,
    pub post: PhaseVector,
}

#[derive(Debug, Clone)]
pub struct WorldHypothesis {
    pub id: u64,
    pub pre: PhaseVector,
    pub predictions: Vec<Option<HypothesisPrediction>>,
    pub support: u32,
    pub revision: u64,
}

#[derive(Debug, Clone)]
struct TuitionBuilder {
    pre: PhaseVector,
    predictions: Vec<Option<HypothesisPrediction>>,
}

#[derive(Debug, Clone, Default)]
pub struct EpistemicEpisode {
    active_ids: Vec<u64>,
    probe_usage: Vec<u32>,
    physical_probes: u32,
}

impl EpistemicEpisode {
    pub fn active_ids(&self) -> &[u64] {
        &self.active_ids
    }

    pub fn physical_probes(&self) -> u32 {
        self.physical_probes
    }
}

#[derive(Debug, Clone)]
pub struct EvoEpistemicState {
    motor_cells: usize,
    match_threshold: f32,
    hypotheses: Vec<WorldHypothesis>,
    next_id: u64,
    tuition: Option<TuitionBuilder>,
    episode: Option<EpistemicEpisode>,
}

impl EvoEpistemicState {
    pub fn new(motor_cells: usize, match_threshold: f32) -> Self {
        assert!(motor_cells > 0);
        assert!((0.0..=1.0).contains(&match_threshold));

        Self {
            motor_cells,
            match_threshold,
            hypotheses: Vec::new(),
            next_id: 1,
            tuition: None,
            episode: None,
        }
    }

    pub fn hypotheses(&self) -> &[WorldHypothesis] {
        &self.hypotheses
    }

    pub fn active_rivals(&self) -> usize {
        self.episode
            .as_ref()
            .map(|episode| episode.active_ids.len())
            .unwrap_or(0)
    }

    pub fn physical_probes(&self) -> u32 {
        self.episode
            .as_ref()
            .map(EpistemicEpisode::physical_probes)
            .unwrap_or(0)
    }

    pub fn begin_tuition_episode(&mut self, pre: PhaseVector) {
        assert!(self.tuition.is_none(), "finish previous tuition episode first");
        self.tuition = Some(TuitionBuilder {
            pre,
            predictions: vec![None; self.motor_cells],
        });
    }

    pub fn record_tuition_transition(&mut self, action: usize, post: PhaseVector) {
        assert!(action < self.motor_cells);
        let builder = self
            .tuition
            .as_mut()
            .expect("begin_tuition_episode must be called first");

        builder.predictions[action] = Some(HypothesisPrediction { action, post });
    }

    pub fn commit_tuition_episode(&mut self) -> u64 {
        let builder = self
            .tuition
            .take()
            .expect("begin_tuition_episode must be called first");

        assert!(
            builder.predictions.iter().any(Option::is_some),
            "tuition episode must contain factual transitions"
        );

        if let Some(idx) = self.compatible_hypothesis(&builder) {
            let hypothesis = &mut self.hypotheses[idx];
            for (dst, src) in hypothesis
                .predictions
                .iter_mut()
                .zip(builder.predictions.into_iter())
            {
                if dst.is_none() {
                    *dst = src;
                }
            }
            hypothesis.support = hypothesis.support.saturating_add(1);
            return hypothesis.id;
        }

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.hypotheses.push(WorldHypothesis {
            id,
            pre: builder.pre,
            predictions: builder.predictions,
            support: 1,
            revision: 0,
        });
        id
    }

    pub fn begin_unknown_episode(&mut self, pre: &PhaseVector) -> usize {
        let active_ids = self
            .hypotheses
            .iter()
            .filter(|hypothesis| hypothesis.pre.similarity(pre) >= self.match_threshold)
            .map(|hypothesis| hypothesis.id)
            .collect::<Vec<_>>();

        self.episode = Some(EpistemicEpisode {
            active_ids,
            probe_usage: vec![0; self.motor_cells],
            physical_probes: 0,
        });

        self.active_rivals()
    }

    pub fn choose_probe(&mut self, disagreement_enabled: bool) -> usize {
        let episode = self
            .episode
            .as_ref()
            .expect("begin_unknown_episode must be called first")
            .clone();

        let mut best: Option<(usize, f32, u32)> = None;

        for action in 0..self.motor_cells {
            let epistemic = if disagreement_enabled {
                self.action_disagreement(&episode.active_ids, action)
            } else {
                0.0
            };
            let usage = episode.probe_usage[action];

            // The generic component is novelty only. Epistemic value comes
            // exclusively from disagreement among acquired carrier hypotheses.
            let generic_novelty = 1.0 / ((usage + 1) as f32).sqrt();
            let score = generic_novelty + 4.0 * epistemic;

            match best {
                None => best = Some((action, score, usage)),
                Some((best_action, best_score, best_usage)) => {
                    let better = score > best_score + 1.0e-6
                        || ((score - best_score).abs() <= 1.0e-6
                            && (usage < best_usage
                                || (usage == best_usage && action < best_action)));
                    if better {
                        best = Some((action, score, usage));
                    }
                }
            }
        }

        let action = best.expect("at least one probe action").0;
        let episode = self
            .episode
            .as_mut()
            .expect("episode exists while choosing a probe");
        episode.probe_usage[action] = episode.probe_usage[action].saturating_add(1);
        episode.physical_probes = episode.physical_probes.saturating_add(1);
        action
    }

    pub fn observe_probe_result(&mut self, action: usize, post: &PhaseVector) -> usize {
        assert!(action < self.motor_cells);
        let active = self
            .episode
            .as_ref()
            .expect("begin_unknown_episode must be called first")
            .active_ids
            .clone();

        let survivors = active
            .into_iter()
            .filter(|id| {
                let hypothesis = self
                    .hypotheses
                    .iter()
                    .find(|hypothesis| hypothesis.id == *id)
                    .expect("active hypothesis id must exist");

                let Some(prediction) = hypothesis.predictions[action].as_ref() else {
                    return false;
                };

                prediction.post.similarity(post) >= self.match_threshold
            })
            .collect::<Vec<_>>();

        self.episode
            .as_mut()
            .expect("episode exists while observing")
            .active_ids = survivors;

        self.active_rivals()
    }

    pub fn predicted_post(&self, action: usize) -> Option<PhaseVector> {
        assert!(action < self.motor_cells);
        let episode = self.episode.as_ref()?;
        if episode.active_ids.len() != 1 {
            return None;
        }

        let id = episode.active_ids[0];
        self.hypotheses
            .iter()
            .find(|hypothesis| hypothesis.id == id)?
            .predictions[action]
            .as_ref()
            .map(|prediction| prediction.post.clone())
    }

    pub fn disagreement_for(&self, action: usize) -> f32 {
        assert!(action < self.motor_cells);
        let Some(episode) = self.episode.as_ref() else {
            return 0.0;
        };
        self.action_disagreement(&episode.active_ids, action)
    }

    fn compatible_hypothesis(&self, builder: &TuitionBuilder) -> Option<usize> {
        self.hypotheses.iter().position(|hypothesis| {
            if hypothesis.pre.similarity(&builder.pre) < self.match_threshold {
                return false;
            }

            for (existing, incoming) in hypothesis.predictions.iter().zip(&builder.predictions) {
                match (existing, incoming) {
                    (Some(a), Some(b)) => {
                        if a.post.similarity(&b.post) < self.match_threshold {
                            return false;
                        }
                    }
                    _ => {}
                }
            }

            true
        })
    }

    fn action_disagreement(&self, active_ids: &[u64], action: usize) -> f32 {
        let predictions = active_ids
            .iter()
            .filter_map(|id| {
                self.hypotheses
                    .iter()
                    .find(|hypothesis| hypothesis.id == *id)?
                    .predictions[action]
                    .as_ref()
                    .map(|prediction| &prediction.post)
            })
            .collect::<Vec<_>>();

        if predictions.len() < 2 {
            return 0.0;
        }

        let mut total = 0.0f32;
        let mut pairs = 0u32;

        for i in 0..predictions.len() {
            for j in (i + 1)..predictions.len() {
                total += (1.0 - predictions[i].similarity(predictions[j])).clamp(0.0, 2.0);
                pairs = pairs.saturating_add(1);
            }
        }

        if pairs == 0 {
            0.0
        } else {
            total / pairs as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disagreement_prefers_action_with_divergent_predictions() {
        let mut state = EvoEpistemicState::new(3, 0.95);
        let pre = PhaseVector::from_seed(64, 1);
        let same = PhaseVector::from_seed(64, 2);
        let alpha = PhaseVector::from_seed(64, 3);
        let beta = PhaseVector::from_seed(64, 4);

        state.begin_tuition_episode(pre.clone());
        state.record_tuition_transition(0, same.clone());
        state.record_tuition_transition(1, alpha.clone());
        state.record_tuition_transition(2, same.clone());
        state.commit_tuition_episode();

        state.begin_tuition_episode(pre.clone());
        state.record_tuition_transition(0, same.clone());
        state.record_tuition_transition(1, beta.clone());
        state.record_tuition_transition(2, same.clone());
        state.commit_tuition_episode();

        assert_eq!(state.hypotheses().len(), 2);
        assert_eq!(state.begin_unknown_episode(&pre), 2);
        assert!(state.disagreement_for(1) > state.disagreement_for(0));
        assert_eq!(state.choose_probe(true), 1);
    }
}
