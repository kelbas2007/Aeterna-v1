use crate::hdc::PhaseVector;

#[derive(Debug, Clone)]
pub struct ConceptConfig {
    pub width: usize,
    pub height: usize,
    pub motor_cells: usize,
    pub hdc_dim: usize,
    pub local_radius: usize,
    pub atom_match_threshold: f32,
    pub max_atoms: usize,
    pub min_action_support: u32,
    pub min_composite_support: u32,
    pub child_predictiveness_ceiling: f32,
    pub composite_promotion_threshold: f32,
    pub atom_formation_enabled: bool,
    pub composite_formation_enabled: bool,
    pub readout_enabled: bool,
    pub learning_enabled: bool,
}

impl ConceptConfig {
    pub fn for_raster(width: usize, height: usize, motor_cells: usize, hdc_dim: usize) -> Self {
        Self {
            width,
            height,
            motor_cells,
            hdc_dim,
            local_radius: 2,
            atom_match_threshold: 0.97,
            max_atoms: 64,
            min_action_support: 4,
            min_composite_support: 8,
            child_predictiveness_ceiling: 0.20,
            composite_promotion_threshold: 0.60,
            atom_formation_enabled: true,
            composite_formation_enabled: true,
            readout_enabled: false,
            learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConceptOutcomeStat {
    pub success: u32,
    pub failure: u32,
}

impl ConceptOutcomeStat {
    pub fn support(&self) -> u32 {
        self.success.saturating_add(self.failure)
    }

    pub fn signed_value(&self, min_support: u32) -> f32 {
        let support = self.support();
        if support < min_support || support == 0 {
            return 0.0;
        }
        (self.success as f32 - self.failure as f32) / support as f32
    }

    fn observe(&mut self, need: bool) {
        if need {
            self.success = self.success.saturating_add(1);
        } else {
            self.failure = self.failure.saturating_add(1);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConceptAtom {
    pub id: u64,
    pub prototype: PhaseVector,
    pub support: u32,
    pub outcomes: Vec<ConceptOutcomeStat>,
}

#[derive(Debug, Clone)]
pub struct CompositeConcept {
    pub id: u64,
    pub child_ids: [u64; 2],
    pub outcomes: Vec<ConceptOutcomeStat>,
    pub support: u32,
    pub utility: f32,
    pub revision: u64,
}

#[derive(Debug, Clone)]
struct CompositeCandidate {
    child_ids: [u64; 2],
    outcomes: Vec<ConceptOutcomeStat>,
    support: u32,
}

#[derive(Debug, Clone)]
pub struct EvoConceptMemory {
    config: ConceptConfig,
    position_roles: Vec<PhaseVector>,
    atoms: Vec<ConceptAtom>,
    candidates: Vec<CompositeCandidate>,
    composites: Vec<CompositeConcept>,
    next_atom_id: u64,
    next_composite_id: u64,
}

impl EvoConceptMemory {
    pub fn new(config: ConceptConfig) -> Self {
        assert!(config.width > 0 && config.height > 0);
        assert!(config.motor_cells > 0);
        assert!(config.hdc_dim > 0);
        assert!(config.local_radius > 0);
        assert!((0.0..=1.0).contains(&config.atom_match_threshold));
        assert!((0.0..=1.0).contains(&config.child_predictiveness_ceiling));
        assert!((0.0..=1.0).contains(&config.composite_promotion_threshold));
        assert!(config.min_action_support > 0);
        assert!(config.min_composite_support > 0);

        let axis_x = PhaseVector::from_seed(config.hdc_dim, 0xC011_CE70_0000_0001);
        let axis_y = PhaseVector::from_seed(config.hdc_dim, 0xC011_CE70_0000_0002);
        let mut position_roles = Vec::with_capacity(config.width * config.height);
        for y in 0..config.height {
            for x in 0..config.width {
                position_roles.push(axis_x.powi(x as i32).bind(&axis_y.powi(y as i32)));
            }
        }

        Self {
            config,
            position_roles,
            atoms: Vec::new(),
            candidates: Vec::new(),
            composites: Vec::new(),
            next_atom_id: 1,
            next_composite_id: 1,
        }
    }

    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.config.learning_enabled = enabled;
    }

    pub fn set_readout_enabled(&mut self, enabled: bool) {
        self.config.readout_enabled = enabled;
    }

    pub fn atoms(&self) -> &[ConceptAtom] {
        &self.atoms
    }

    pub fn min_action_support(&self) -> u32 {
        self.config.min_action_support
    }

    pub fn physical_promotion_thresholds(&self) -> (u32, f32, f32) {
        (
            self.config.min_composite_support,
            self.config.child_predictiveness_ceiling,
            self.config.composite_promotion_threshold,
        )
    }

    /// Atom-only acquisition path for phase-native concept execution.
    /// It may recruit/match generic lower-level relation prototypes, but it
    /// does not update pair candidates or action-conditioned concept answers.
    pub fn observe_atoms(&mut self, raster: &[f32]) -> Vec<u64> {
        self.assert_raster(raster);
        if !self.config.learning_enabled {
            return self.active_atom_ids(raster);
        }

        let traces = self.local_relation_traces(raster);
        let mut active = Vec::new();
        for trace in traces {
            let atom_idx = if let Some((idx, similarity)) = self.best_atom(&trace) {
                if similarity >= self.config.atom_match_threshold {
                    Some(idx)
                } else if self.config.atom_formation_enabled && self.atoms.len() < self.config.max_atoms {
                    Some(self.recruit_atom(trace))
                } else {
                    None
                }
            } else if self.config.atom_formation_enabled && self.atoms.len() < self.config.max_atoms {
                Some(self.recruit_atom(trace))
            } else {
                None
            };
            if let Some(idx) = atom_idx {
                if !active.contains(&idx) {
                    active.push(idx);
                }
            }
        }

        active.sort_unstable();
        active.dedup();
        for idx in &active {
            self.atoms[*idx].support = self.atoms[*idx].support.saturating_add(1);
        }
        let mut ids = active.into_iter().map(|idx| self.atoms[idx].id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub fn composites(&self) -> &[CompositeConcept] {
        &self.composites
    }

    pub fn active_atom_ids(&self, raster: &[f32]) -> Vec<u64> {
        self.assert_raster(raster);
        let traces = self.local_relation_traces(raster);
        let mut ids = traces
            .iter()
            .filter_map(|trace| {
                self.best_atom(trace)
                    .filter(|(_, similarity)| *similarity >= self.config.atom_match_threshold)
                    .map(|(idx, _)| self.atoms[idx].id)
            })
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    pub fn atom_action_evidence(&self, atom_id: u64, action: usize) -> Option<f32> {
        assert!(action < self.config.motor_cells);
        let atom = self.atoms.iter().find(|atom| atom.id == atom_id)?;
        Some(atom.outcomes[action].signed_value(self.config.min_action_support))
    }

    pub fn composite_action_evidence(&self, concept_id: u64, action: usize) -> Option<f32> {
        assert!(action < self.config.motor_cells);
        let concept = self.composites.iter().find(|concept| concept.id == concept_id)?;
        Some(concept.outcomes[action].signed_value(self.config.min_action_support))
    }

    pub fn observe_factual(&mut self, raster: &[f32], action: usize, need: bool) {
        assert!(action < self.config.motor_cells);
        self.assert_raster(raster);
        if !self.config.learning_enabled {
            return;
        }

        let traces = self.local_relation_traces(raster);
        let mut active = Vec::new();

        for trace in traces {
            let atom_idx = if let Some((idx, similarity)) = self.best_atom(&trace) {
                if similarity >= self.config.atom_match_threshold {
                    Some(idx)
                } else if self.config.atom_formation_enabled && self.atoms.len() < self.config.max_atoms {
                    Some(self.recruit_atom(trace))
                } else {
                    None
                }
            } else if self.config.atom_formation_enabled && self.atoms.len() < self.config.max_atoms {
                Some(self.recruit_atom(trace))
            } else {
                None
            };

            if let Some(idx) = atom_idx {
                if !active.contains(&idx) {
                    active.push(idx);
                }
            }
        }

        active.sort_unstable();
        active.dedup();

        for idx in &active {
            let atom = &mut self.atoms[*idx];
            atom.support = atom.support.saturating_add(1);
            atom.outcomes[action].observe(need);
        }

        let active_ids = active.iter().map(|idx| self.atoms[*idx].id).collect::<Vec<_>>();
        for i in 0..active_ids.len() {
            for j in (i + 1)..active_ids.len() {
                let child_ids = ordered_pair(active_ids[i], active_ids[j]);
                if let Some(idx) = self
                    .composites
                    .iter()
                    .position(|concept| concept.child_ids == child_ids)
                {
                    let before = self.composites[idx].outcomes[action]
                        .signed_value(self.config.min_action_support);
                    self.composites[idx].outcomes[action].observe(need);
                    self.composites[idx].support = self.composites[idx].support.saturating_add(1);
                    let after = self.composites[idx].outcomes[action]
                        .signed_value(self.config.min_action_support);
                    if before.signum() != 0.0
                        && after.signum() != 0.0
                        && before.signum() != after.signum()
                    {
                        self.composites[idx].revision =
                            self.composites[idx].revision.saturating_add(1);
                    }
                    self.composites[idx].utility =
                        self.max_abs_outcome(&self.composites[idx].outcomes);
                    continue;
                }

                let candidate_idx = self
                    .candidates
                    .iter()
                    .position(|candidate| candidate.child_ids == child_ids)
                    .unwrap_or_else(|| {
                        self.candidates.push(CompositeCandidate {
                            child_ids,
                            outcomes: vec![ConceptOutcomeStat::default(); self.config.motor_cells],
                            support: 0,
                        });
                        self.candidates.len() - 1
                    });
                let candidate = &mut self.candidates[candidate_idx];
                candidate.support = candidate.support.saturating_add(1);
                candidate.outcomes[action].observe(need);
            }
        }

        self.promote_ready_candidates();
    }

    pub fn choose_composite_action(&self, raster: &[f32]) -> Option<usize> {
        if !self.config.readout_enabled {
            return None;
        }
        let active = self.active_atom_ids(raster);
        if active.len() < 2 {
            return None;
        }

        let mut best: Option<(usize, f32, u32, u64)> = None;
        for i in 0..active.len() {
            for j in (i + 1)..active.len() {
                let pair = ordered_pair(active[i], active[j]);
                let Some(concept) = self.composites.iter().find(|c| c.child_ids == pair) else {
                    continue;
                };
                for action in 0..self.config.motor_cells {
                    let score = concept.outcomes[action].signed_value(self.config.min_action_support);
                    if score <= 0.0 {
                        continue;
                    }
                    match best {
                        None => best = Some((action, score, concept.support, concept.id)),
                        Some((best_action, best_score, best_support, best_id)) => {
                            let better = score > best_score + 1.0e-6
                                || ((score - best_score).abs() <= 1.0e-6
                                    && (concept.support > best_support
                                        || (concept.support == best_support
                                            && (concept.id < best_id
                                                || (concept.id == best_id
                                                    && action < best_action)))));
                            if better {
                                best = Some((action, score, concept.support, concept.id));
                            }
                        }
                    }
                }
            }
        }
        best.map(|(action, _, _, _)| action)
    }

    pub fn choose_atom_only_action(&self, raster: &[f32]) -> Option<usize> {
        let active = self.active_atom_ids(raster);
        if active.is_empty() {
            return None;
        }
        let mut scores = vec![0.0f32; self.config.motor_cells];
        for atom_id in active {
            let Some(atom) = self.atoms.iter().find(|atom| atom.id == atom_id) else {
                continue;
            };
            for (action, stat) in atom.outcomes.iter().enumerate() {
                scores[action] += stat.signed_value(self.config.min_action_support);
            }
        }

        (0..self.config.motor_cells)
            .max_by(|a, b| {
                scores[*a]
                    .partial_cmp(&scores[*b])
                    .unwrap()
                    .then_with(|| b.cmp(a))
            })
    }

    fn recruit_atom(&mut self, prototype: PhaseVector) -> usize {
        let id = self.next_atom_id;
        self.next_atom_id = self.next_atom_id.saturating_add(1);
        self.atoms.push(ConceptAtom {
            id,
            prototype,
            support: 0,
            outcomes: vec![ConceptOutcomeStat::default(); self.config.motor_cells],
        });
        self.atoms.len() - 1
    }

    fn promote_ready_candidates(&mut self) {
        if !self.config.composite_formation_enabled {
            return;
        }

        let mut idx = 0usize;
        while idx < self.candidates.len() {
            let candidate = &self.candidates[idx];
            if candidate.support < self.config.min_composite_support {
                idx += 1;
                continue;
            }

            let Some((action, evidence)) = candidate
                .outcomes
                .iter()
                .enumerate()
                .map(|(action, stat)| {
                    (
                        action,
                        stat.signed_value(self.config.min_action_support).abs(),
                    )
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            else {
                idx += 1;
                continue;
            };

            if evidence < self.config.composite_promotion_threshold {
                idx += 1;
                continue;
            }

            let children_weak = candidate.child_ids.iter().all(|child_id| {
                self.atom_action_evidence(*child_id, action)
                    .map(|value| value.abs() <= self.config.child_predictiveness_ceiling)
                    .unwrap_or(false)
            });
            if !children_weak {
                idx += 1;
                continue;
            }

            let candidate = self.candidates.remove(idx);
            let id = self.next_composite_id;
            self.next_composite_id = self.next_composite_id.saturating_add(1);
            let utility = self.max_abs_outcome(&candidate.outcomes);
            self.composites.push(CompositeConcept {
                id,
                child_ids: candidate.child_ids,
                outcomes: candidate.outcomes,
                support: candidate.support,
                utility,
                revision: 0,
            });
        }
    }

    fn max_abs_outcome(&self, outcomes: &[ConceptOutcomeStat]) -> f32 {
        outcomes
            .iter()
            .map(|stat| stat.signed_value(self.config.min_action_support).abs())
            .fold(0.0f32, f32::max)
    }

    fn best_atom(&self, trace: &PhaseVector) -> Option<(usize, f32)> {
        self.atoms
            .iter()
            .enumerate()
            .map(|(idx, atom)| (idx, atom.prototype.similarity(trace)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    fn local_relation_traces(&self, raster: &[f32]) -> Vec<PhaseVector> {
        let mut points = raster
            .iter()
            .enumerate()
            .filter(|(_, value)| **value >= 0.5)
            .map(|(idx, _)| (idx % self.config.width, idx / self.config.width))
            .collect::<Vec<_>>();
        points.sort_unstable();

        let mut traces = Vec::new();
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let a = points[i];
                let b = points[j];
                let dx = a.0.abs_diff(b.0);
                let dy = a.1.abs_diff(b.1);
                if dx.max(dy) > self.config.local_radius {
                    continue;
                }

                let (from, to) = if a <= b { (a, b) } else { (b, a) };
                let from_role = &self.position_roles[from.1 * self.config.width + from.0];
                let to_role = &self.position_roles[to.1 * self.config.width + to.0];
                traces.push(to_role.unbind(from_role));
            }
        }
        traces
    }

    fn assert_raster(&self, raster: &[f32]) {
        assert_eq!(raster.len(), self.config.width * self.config.height);
        assert!(raster
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0 && *value <= 1.0));
    }
}

fn ordered_pair(a: u64, b: u64) -> [u64; 2] {
    if a <= b { [a, b] } else { [b, a] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_promotes_only_after_children_are_individually_weak() {
        let mut memory = EvoConceptMemory::new(ConceptConfig::for_raster(12, 12, 2, 128));
        let mut scene = vec![0.0f32; 144];
        scene[1 * 12 + 1] = 1.0;
        scene[1 * 12 + 2] = 1.0;
        scene[8 * 12 + 8] = 1.0;
        scene[9 * 12 + 8] = 1.0;

        for _ in 0..4 {
            memory.observe_factual(&scene, 0, true);
            memory.observe_factual(&scene, 1, false);
        }

        assert_eq!(memory.atoms().len(), 2);
        assert_eq!(
            memory.composites().len(),
            0,
            "children remain strongly predictive when only one pair class exists"
        );
    }
}
