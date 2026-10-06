use crate::hdc::PhaseVector;

#[derive(Debug, Clone)]
pub struct MacroConfig {
    pub motor_cells: usize,
    pub match_threshold: f32,
    pub min_promotion_support: u32,
    pub formation_enabled: bool,
    pub revision_enabled: bool,
    pub readout_enabled: bool,
    pub learning_enabled: bool,
}

impl MacroConfig {
    pub fn new(motor_cells: usize) -> Self {
        Self {
            motor_cells,
            match_threshold: 0.97,
            min_promotion_support: 4,
            formation_enabled: true,
            revision_enabled: true,
            readout_enabled: false,
            learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroBranch {
    pub post: PhaseVector,
    pub next_action: usize,
    pub support: u32,
    pub failures: u32,
    pub revision: u64,
}

impl MacroBranch {
    fn value(&self) -> f32 {
        let success = self.support as f32;
        let failure = self.failures as f32;
        // Smoothed factual success estimate. Counterexamples remain in state
        // rather than deleting the obsolete action evidence.
        (success + 1.0) / (success + failure + 2.0)
    }
}

#[derive(Debug, Clone)]
pub struct MacroCounterexample {
    pub post: PhaseVector,
    pub action: usize,
    pub observed_need: bool,
}

#[derive(Debug, Clone)]
pub struct MacroAssembly {
    pub id: u64,
    pub entry: PhaseVector,
    pub first_action: usize,
    pub branches: Vec<MacroBranch>,
    pub counterexamples: Vec<MacroCounterexample>,
    pub support: u32,
    pub utility: f32,
    pub revision: u64,
}

#[derive(Debug, Clone)]
struct MacroCandidate {
    entry: PhaseVector,
    first_action: usize,
    branches: Vec<MacroBranch>,
    support: u32,
}

#[derive(Debug, Clone)]
pub struct EvoMacroMemory {
    config: MacroConfig,
    candidates: Vec<MacroCandidate>,
    macros: Vec<MacroAssembly>,
    next_id: u64,
    active_macro: Option<u64>,
}

impl EvoMacroMemory {
    pub fn new(config: MacroConfig) -> Self {
        assert!(config.motor_cells > 0);
        assert!((0.0..=1.0).contains(&config.match_threshold));
        assert!(config.min_promotion_support > 0);

        Self {
            config,
            candidates: Vec::new(),
            macros: Vec::new(),
            next_id: 1,
            active_macro: None,
        }
    }

    pub fn set_readout_enabled(&mut self, enabled: bool) {
        self.config.readout_enabled = enabled;
        if !enabled {
            self.active_macro = None;
        }
    }

    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.config.learning_enabled = enabled;
    }

    pub fn set_revision_enabled(&mut self, enabled: bool) {
        self.config.revision_enabled = enabled;
    }

    pub fn macros(&self) -> &[MacroAssembly] {
        &self.macros
    }

    pub fn active_macro_id(&self) -> Option<u64> {
        self.active_macro
    }

    pub fn apply_id_permutation(&mut self, mapping: &[(u64, u64)]) {
        if mapping.is_empty() {
            return;
        }

        let existing = self.macros.iter().map(|m| m.id).collect::<std::collections::BTreeSet<_>>();
        let from = mapping.iter().map(|(a, _)| *a).collect::<std::collections::BTreeSet<_>>();
        let to = mapping.iter().map(|(_, b)| *b).collect::<std::collections::BTreeSet<_>>();

        assert_eq!(from.len(), mapping.len(), "macro id permutation source ids must be unique");
        assert_eq!(to.len(), mapping.len(), "macro id permutation target ids must be unique");
        assert!(from.is_subset(&existing), "macro id permutation references unknown source id");
        assert_eq!(from, to, "macro id permutation must be a bijection over the same acquired ids");

        for macro_assembly in &mut self.macros {
            if let Some((_, new_id)) = mapping.iter().find(|(old_id, _)| *old_id == macro_assembly.id) {
                macro_assembly.id = *new_id;
            }
        }

        if let Some(active) = self.active_macro {
            if let Some((_, new_id)) = mapping.iter().find(|(old_id, _)| *old_id == active) {
                self.active_macro = Some(*new_id);
            }
        }
    }

    // G3 compatibility path: failed exploratory episodes do not revise a
    // freshly acquired macro. G4 uses observe_factual_episode explicitly.
    pub fn observe_successful_episode(
        &mut self,
        pre: PhaseVector,
        first_action: usize,
        mid: PhaseVector,
        second_action: usize,
        need: bool,
    ) {
        if !need {
            return;
        }
        self.observe_positive_episode(pre, first_action, mid, second_action);
    }

    pub fn observe_factual_episode(
        &mut self,
        pre: PhaseVector,
        first_action: usize,
        mid: PhaseVector,
        second_action: usize,
        need: bool,
    ) {
        assert!(first_action < self.config.motor_cells);
        assert!(second_action < self.config.motor_cells);

        if !self.config.learning_enabled {
            return;
        }

        if let Some(idx) = self.matching_macro_index(&pre, first_action) {
            // G4 ablation: once a macro exists, disabling revision freezes that
            // acquired program while preserving the same external factual stream.
            if !self.config.revision_enabled {
                return;
            }

            if need {
                self.update_promoted(idx, &mid, second_action);
                return;
            }

            let match_threshold = self.config.match_threshold;
            let macro_assembly = &mut self.macros[idx];
            macro_assembly.counterexamples.push(MacroCounterexample {
                post: mid.clone(),
                action: second_action,
                observed_need: false,
            });

            if let Some(branch) = macro_assembly.branches.iter_mut().find(|branch| {
                branch.next_action == second_action
                    && branch.post.similarity(&mid) >= match_threshold
            }) {
                branch.failures = branch.failures.saturating_add(1);
                branch.revision = branch.revision.saturating_add(1);
            }

            macro_assembly.revision = macro_assembly.revision.saturating_add(1);
            macro_assembly.utility = (macro_assembly.utility - 0.03).clamp(-1.0, 1.0);
            return;
        }

        // Before a macro exists, positive factual trajectories may still form one.
        if need {
            self.observe_positive_episode(pre, first_action, mid, second_action);
        }
    }

    pub fn begin(&mut self, pre: &PhaseVector) -> Option<usize> {
        self.active_macro = None;
        if !self.config.readout_enabled {
            return None;
        }

        let idx = self
            .macros
            .iter()
            .enumerate()
            .filter_map(|(idx, macro_assembly)| {
                let similarity = macro_assembly.entry.similarity(pre);
                (similarity >= self.config.match_threshold).then_some((idx, similarity))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?
            .0;

        let macro_assembly = &self.macros[idx];
        self.active_macro = Some(macro_assembly.id);
        Some(macro_assembly.first_action)
    }

    pub fn begin_by_id(&mut self, pre: &PhaseVector, macro_id: u64) -> Option<usize> {
        self.active_macro = None;
        if !self.config.readout_enabled {
            return None;
        }

        let macro_assembly = self.macros.iter().find(|m| m.id == macro_id)?;
        if macro_assembly.entry.similarity(pre) < self.config.match_threshold {
            return None;
        }

        self.active_macro = Some(macro_assembly.id);
        Some(macro_assembly.first_action)
    }

    pub fn continue_after_factual_post(&mut self, mid: &PhaseVector) -> Option<usize> {
        if !self.config.readout_enabled {
            self.active_macro = None;
            return None;
        }

        let id = self.active_macro?;
        let macro_assembly = self.macros.iter().find(|m| m.id == id)?;

        let action = macro_assembly
            .branches
            .iter()
            .filter_map(|branch| {
                let similarity = branch.post.similarity(mid);
                (similarity >= self.config.match_threshold).then_some((
                    branch.next_action,
                    similarity,
                    branch.value(),
                    branch.support,
                ))
            })
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap()
                    .then_with(|| a.2.partial_cmp(&b.2).unwrap())
                    .then_with(|| a.3.cmp(&b.3))
            })
            .map(|(action, _, _, _)| action);

        self.active_macro = None;
        action
    }

    fn observe_positive_episode(
        &mut self,
        pre: PhaseVector,
        first_action: usize,
        mid: PhaseVector,
        second_action: usize,
    ) {
        assert!(first_action < self.config.motor_cells);
        assert!(second_action < self.config.motor_cells);

        if !self.config.learning_enabled || !self.config.formation_enabled {
            return;
        }

        if let Some(idx) = self.matching_macro_index(&pre, first_action) {
            self.update_promoted(idx, &mid, second_action);
            return;
        }

        let candidate_idx = self
            .candidates
            .iter()
            .position(|candidate| {
                candidate.first_action == first_action
                    && candidate.entry.similarity(&pre) >= self.config.match_threshold
            })
            .unwrap_or_else(|| {
                self.candidates.push(MacroCandidate {
                    entry: pre.clone(),
                    first_action,
                    branches: Vec::new(),
                    support: 0,
                });
                self.candidates.len() - 1
            });

        let candidate = &mut self.candidates[candidate_idx];
        candidate.support = candidate.support.saturating_add(1);
        Self::merge_positive_branch(
            &mut candidate.branches,
            &mid,
            second_action,
            self.config.match_threshold,
        );

        let branch_support: u32 = candidate.branches.iter().map(|branch| branch.support).sum();
        let has_branching = candidate.branches.len() >= 2;
        let ready = candidate.support >= self.config.min_promotion_support
            && branch_support >= self.config.min_promotion_support
            && has_branching;

        if ready {
            let candidate = self.candidates.remove(candidate_idx);
            let id = self.next_id;
            self.next_id = self.next_id.saturating_add(1);
            self.macros.push(MacroAssembly {
                id,
                entry: candidate.entry,
                first_action: candidate.first_action,
                branches: candidate.branches,
                counterexamples: Vec::new(),
                support: candidate.support,
                utility: 0.10,
                revision: 0,
            });
        }
    }

    fn matching_macro_index(&self, pre: &PhaseVector, first_action: usize) -> Option<usize> {
        self.macros.iter().position(|macro_assembly| {
            macro_assembly.first_action == first_action
                && macro_assembly.entry.similarity(pre) >= self.config.match_threshold
        })
    }

    fn update_promoted(&mut self, idx: usize, mid: &PhaseVector, second_action: usize) {
        let macro_assembly = &mut self.macros[idx];
        macro_assembly.support = macro_assembly.support.saturating_add(1);
        Self::merge_positive_branch(
            &mut macro_assembly.branches,
            mid,
            second_action,
            self.config.match_threshold,
        );
        macro_assembly.utility = (macro_assembly.utility + 0.02).clamp(-1.0, 1.0);
    }

    fn merge_positive_branch(
        branches: &mut Vec<MacroBranch>,
        mid: &PhaseVector,
        second_action: usize,
        match_threshold: f32,
    ) {
        if let Some(branch) = branches.iter_mut().find(|branch| {
            branch.next_action == second_action
                && branch.post.similarity(mid) >= match_threshold
        }) {
            branch.support = branch.support.saturating_add(1);
            return;
        }

        branches.push(MacroBranch {
            post: mid.clone(),
            next_action: second_action,
            support: 1,
            failures: 0,
            revision: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_successful_branching_trajectories_promote_macro() {
        let mut memory = EvoMacroMemory::new(MacroConfig {
            motor_cells: 3,
            min_promotion_support: 4,
            ..MacroConfig::new(3)
        });

        let pre = PhaseVector::from_seed(64, 1);
        let a = PhaseVector::from_seed(64, 2);
        let b = PhaseVector::from_seed(64, 3);

        for mid in [&a, &b, &a, &b] {
            let terminal = if mid.similarity(&a) > 0.99 { 2 } else { 0 };
            memory.observe_successful_episode(
                pre.clone(),
                1,
                mid.clone(),
                terminal,
                true,
            );
        }

        assert_eq!(memory.macros().len(), 1);
        assert_eq!(memory.macros()[0].first_action, 1);
        assert_eq!(memory.macros()[0].branches.len(), 2);
    }

    #[test]
    fn factual_counterexample_revises_same_macro_without_erasing_old_evidence() {
        let mut memory = EvoMacroMemory::new(MacroConfig {
            motor_cells: 3,
            min_promotion_support: 4,
            readout_enabled: true,
            ..MacroConfig::new(3)
        });

        let pre = PhaseVector::from_seed(64, 11);
        let a = PhaseVector::from_seed(64, 12);
        let b = PhaseVector::from_seed(64, 13);

        for mid in [&a, &b, &a, &b] {
            let terminal = if mid.similarity(&a) > 0.99 { 2 } else { 0 };
            memory.observe_successful_episode(
                pre.clone(),
                1,
                mid.clone(),
                terminal,
                true,
            );
        }

        let id = memory.macros()[0].id;
        for _ in 0..4 {
            memory.observe_factual_episode(pre.clone(), 1, a.clone(), 2, false);
            memory.observe_factual_episode(pre.clone(), 1, a.clone(), 1, true);
        }

        assert_eq!(memory.macros().len(), 1);
        assert_eq!(memory.macros()[0].id, id);
        assert!(memory.macros()[0].revision >= 4);
        assert!(memory.macros()[0].counterexamples.len() >= 4);

        assert_eq!(memory.begin(&pre), Some(1));
        assert_eq!(memory.continue_after_factual_post(&a), Some(1));

        let old = memory.macros()[0]
            .branches
            .iter()
            .find(|branch| branch.next_action == 2)
            .unwrap();
        assert!(old.failures >= 4);
    }
}
