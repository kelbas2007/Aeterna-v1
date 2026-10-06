use crate::hdc::PhaseVector;

#[derive(Debug, Clone)]
pub struct HierarchyConfig {
    pub match_threshold: f32,
    pub min_promotion_support: u32,
    pub formation_enabled: bool,
    pub readout_enabled: bool,
    pub learning_enabled: bool,
}

impl Default for HierarchyConfig {
    fn default() -> Self {
        Self {
            match_threshold: 0.97,
            min_promotion_support: 3,
            formation_enabled: true,
            readout_enabled: false,
            learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParentMacro {
    pub id: u64,
    pub cue: PhaseVector,
    pub child_sequence: Vec<u64>,
    pub support: u32,
    pub utility: f32,
    pub revision: u64,
}

#[derive(Debug, Clone)]
struct ParentCandidate {
    cue: PhaseVector,
    child_sequence: Vec<u64>,
    support: u32,
}

#[derive(Debug, Clone)]
pub struct EvoHierarchyMemory {
    config: HierarchyConfig,
    candidates: Vec<ParentCandidate>,
    parents: Vec<ParentMacro>,
    next_id: u64,
}

impl EvoHierarchyMemory {
    pub fn new(config: HierarchyConfig) -> Self {
        assert!((0.0..=1.0).contains(&config.match_threshold));
        assert!(config.min_promotion_support > 0);

        Self {
            config,
            candidates: Vec::new(),
            parents: Vec::new(),
            next_id: 1,
        }
    }

    pub fn set_readout_enabled(&mut self, enabled: bool) {
        self.config.readout_enabled = enabled;
    }

    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.config.learning_enabled = enabled;
    }

    pub fn parents(&self) -> &[ParentMacro] {
        &self.parents
    }

    pub fn observe_successful_sequence(
        &mut self,
        cue: PhaseVector,
        child_sequence: Vec<u64>,
        need: bool,
    ) {
        if !need
            || !self.config.learning_enabled
            || !self.config.formation_enabled
            || child_sequence.is_empty()
        {
            return;
        }

        if let Some(parent) = self.parents.iter_mut().find(|parent| {
            parent.child_sequence == child_sequence
                && parent.cue.similarity(&cue) >= self.config.match_threshold
        }) {
            parent.support = parent.support.saturating_add(1);
            parent.utility = (parent.utility + 0.02).clamp(-1.0, 1.0);
            return;
        }

        let candidate_idx = self
            .candidates
            .iter()
            .position(|candidate| {
                candidate.child_sequence == child_sequence
                    && candidate.cue.similarity(&cue) >= self.config.match_threshold
            })
            .unwrap_or_else(|| {
                self.candidates.push(ParentCandidate {
                    cue: cue.clone(),
                    child_sequence: child_sequence.clone(),
                    support: 0,
                });
                self.candidates.len() - 1
            });

        let candidate = &mut self.candidates[candidate_idx];
        candidate.support = candidate.support.saturating_add(1);

        if candidate.support >= self.config.min_promotion_support {
            let candidate = self.candidates.remove(candidate_idx);
            let id = self.next_id;
            self.next_id = self.next_id.saturating_add(1);

            self.parents.push(ParentMacro {
                id,
                cue: candidate.cue,
                child_sequence: candidate.child_sequence,
                support: candidate.support,
                utility: 0.10,
                revision: 0,
            });
        }
    }

    pub fn select_sequence(&self, cue: &PhaseVector) -> Option<Vec<u64>> {
        if !self.config.readout_enabled {
            return None;
        }

        self.parents
            .iter()
            .filter_map(|parent| {
                let similarity = parent.cue.similarity(cue);
                (similarity >= self.config.match_threshold)
                    .then_some((parent, similarity))
            })
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap()
                    .then_with(|| a.0.support.cmp(&b.0.support))
            })
            .map(|(parent, _)| parent.child_sequence.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_child_sequences_promote_parent_macro() {
        let mut memory = EvoHierarchyMemory::new(HierarchyConfig {
            min_promotion_support: 3,
            ..HierarchyConfig::default()
        });

        let cue = PhaseVector::from_seed(64, 1);
        for _ in 0..3 {
            memory.observe_successful_sequence(cue.clone(), vec![10, 20], true);
        }

        assert_eq!(memory.parents().len(), 1);
        assert_eq!(memory.parents()[0].child_sequence, vec![10, 20]);

        memory.set_readout_enabled(true);
        assert_eq!(memory.select_sequence(&cue), Some(vec![10, 20]));
    }
}
