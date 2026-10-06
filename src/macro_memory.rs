use crate::hdc::PhaseVector;

#[derive(Debug, Clone)]
pub struct MacroConfig {
    pub motor_cells: usize,
    pub match_threshold: f32,
    pub min_promotion_support: u32,
    pub formation_enabled: bool,
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
}

#[derive(Debug, Clone)]
pub struct MacroAssembly {
    pub id: u64,
    pub entry: PhaseVector,
    pub first_action: usize,
    pub branches: Vec<MacroBranch>,
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

    pub fn macros(&self) -> &[MacroAssembly] {
        &self.macros
    }

    pub fn active_macro_id(&self) -> Option<u64> {
        self.active_macro
    }

    pub fn observe_successful_episode(
        &mut self,
        pre: PhaseVector,
        first_action: usize,
        mid: PhaseVector,
        second_action: usize,
        need: bool,
    ) {
        assert!(first_action < self.config.motor_cells);
        assert!(second_action < self.config.motor_cells);

        if !need || !self.config.learning_enabled || !self.config.formation_enabled {
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
        Self::merge_branch(
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
                support: candidate.support,
                utility: 0.10,
                revision: 0,
            });
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
                (similarity >= self.config.match_threshold)
                    .then_some((branch.next_action, similarity, branch.support))
            })
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap()
                    .then_with(|| a.2.cmp(&b.2))
            })
            .map(|(action, _, _)| action);

        self.active_macro = None;
        action
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
        Self::merge_branch(
            &mut macro_assembly.branches,
            mid,
            second_action,
            self.config.match_threshold,
        );
        macro_assembly.utility =
            (macro_assembly.utility + 0.02).clamp(-1.0, 1.0);
    }

    fn merge_branch(
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
}
