use crate::hdc::PhaseVector;
use crate::trace::ShapeTrace;

#[derive(Debug, Clone)]
pub struct RasterFieldConfig {
    pub width: usize,
    pub height: usize,
    pub motor_cells: usize,
    pub min_active: usize,
    pub max_units: usize,
    pub match_threshold: f32,
    pub min_readout_support: u32,
    pub formation_enabled: bool,
    pub readout_enabled: bool,
    pub learning_enabled: bool,
}

impl RasterFieldConfig {
    pub fn for_raster(width: usize, height: usize, motor_cells: usize) -> Self {
        Self {
            width,
            height,
            motor_cells,
            min_active: 2,
            max_units: 64,
            match_threshold: 0.97,
            min_readout_support: 2,
            formation_enabled: true,
            readout_enabled: false,
            learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OutcomeStat {
    pub success: u32,
    pub failure: u32,
}

impl OutcomeStat {
    fn observe(&mut self, need: bool) {
        if need {
            self.success = self.success.saturating_add(1);
        } else {
            self.failure = self.failure.saturating_add(1);
        }
    }

    fn support(&self) -> u32 {
        self.success.saturating_add(self.failure)
    }

    fn signed_value(&self, min_support: u32) -> f32 {
        let support = self.support();
        if support < min_support || support == 0 {
            return 0.0;
        }

        let success = self.success as f32;
        let failure = self.failure as f32;
        let mean = (success - failure) / support as f32;
        let confidence = support as f32 / (support as f32 + 2.0);
        mean * confidence
    }
}

#[derive(Debug, Clone)]
pub struct PhaseFieldUnit {
    pub id: u64,
    pub prototype: PhaseVector,
    pub support: u32,
    pub utility: f32,
    pub revision: u64,
    pub outcomes: Vec<OutcomeStat>,
}

#[derive(Debug, Clone)]
pub struct EvoRasterField {
    config: RasterFieldConfig,
    position_roles: Vec<PhaseVector>,
    units: Vec<PhaseFieldUnit>,
    next_id: u64,
}

impl EvoRasterField {
    pub fn new(config: RasterFieldConfig, hdc_dim: usize) -> Self {
        assert!(config.width > 0 && config.height > 0);
        assert!(config.motor_cells > 0);
        assert!(config.min_active >= 2);
        assert!(config.max_units > 0);

        // Generic retinotopic phase algebra. The two basis vectors do not encode
        // task semantics. Absolute translation is represented as a common phase
        // factor and cancels when one position role is unbound from another.
        let axis_x = PhaseVector::from_seed(hdc_dim, 0xA11C_0001);
        let axis_y = PhaseVector::from_seed(hdc_dim, 0xA11C_0002);

        let mut position_roles = Vec::with_capacity(config.width * config.height);
        for y in 0..config.height {
            for x in 0..config.width {
                position_roles.push(axis_x.powi(x as i32).bind(&axis_y.powi(y as i32)));
            }
        }

        Self {
            config,
            position_roles,
            units: Vec::new(),
            next_id: 1,
        }
    }

    pub fn set_readout_enabled(&mut self, enabled: bool) {
        self.config.readout_enabled = enabled;
    }

    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.config.learning_enabled = enabled;
    }

    pub fn formation_enabled(&self) -> bool {
        self.config.formation_enabled
    }

    pub fn readout_enabled(&self) -> bool {
        self.config.readout_enabled
    }

    pub fn units(&self) -> &[PhaseFieldUnit] {
        &self.units
    }

    pub fn encode_relational_trace(&self, raster: &[f32]) -> Option<PhaseVector> {
        self.assert_raster(raster);
        self.relational_trace(raster)
    }

    pub fn encode_robust_shape_trace(&self, raster: &[f32]) -> Option<ShapeTrace> {
        self.assert_raster(raster);
        self.robust_shape_trace(raster)
    }

    pub fn active_unit_ids(&self, raster: &[f32]) -> Vec<u64> {
        self.assert_raster(raster);
        let Some(trace) = self.relational_trace(raster) else {
            return Vec::new();
        };

        self.best_matching_unit(&trace)
            .filter(|(_, similarity)| *similarity >= self.config.match_threshold)
            .map(|(idx, _)| vec![self.units[idx].id])
            .unwrap_or_default()
    }

    pub fn observe_factual(&mut self, raster: &[f32], action: usize, need: bool) {
        assert!(action < self.config.motor_cells);
        self.assert_raster(raster);

        if !self.config.learning_enabled {
            return;
        }

        let Some(trace) = self.relational_trace(raster) else {
            return;
        };

        let unit_idx = if self.config.formation_enabled {
            self.match_or_recruit(&trace)
        } else {
            self.best_matching_unit(&trace)
                .filter(|(_, similarity)| *similarity >= self.config.match_threshold)
                .map(|(idx, _)| idx)
        };

        let Some(idx) = unit_idx else {
            return;
        };

        let stat = &mut self.units[idx].outcomes[action];
        let before = stat.signed_value(self.config.min_readout_support);
        stat.observe(need);
        let after = stat.signed_value(self.config.min_readout_support);

        let unit = &mut self.units[idx];
        unit.support = unit.support.saturating_add(1);
        unit.utility = (unit.utility + after * 0.05).clamp(-1.0, 1.0);

        if before.signum() != 0.0 && after.signum() != before.signum() {
            unit.revision = unit.revision.saturating_add(1);
        }
    }

    pub fn motor_evidence(&self, raster: &[f32]) -> Vec<f32> {
        self.assert_raster(raster);
        let mut evidence = vec![0.0; self.config.motor_cells];

        if !self.config.readout_enabled || self.units.is_empty() {
            return evidence;
        }

        let Some(trace) = self.relational_trace(raster) else {
            return evidence;
        };

        let Some((idx, similarity)) = self.best_matching_unit(&trace) else {
            return evidence;
        };

        if similarity < self.config.match_threshold {
            return evidence;
        }

        let unit = &self.units[idx];
        for (action, stat) in unit.outcomes.iter().enumerate() {
            evidence[action] = stat.signed_value(self.config.min_readout_support)
                * (0.5 + 0.5 * unit.utility.max(0.0));
        }

        evidence
    }

    fn match_or_recruit(&mut self, trace: &PhaseVector) -> Option<usize> {
        if let Some((idx, similarity)) = self.best_matching_unit(trace) {
            if similarity >= self.config.match_threshold {
                self.units[idx].support = self.units[idx].support.saturating_add(1);
                return Some(idx);
            }
        }

        if self.units.len() >= self.config.max_units {
            return None;
        }

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        self.units.push(PhaseFieldUnit {
            id,
            prototype: trace.clone(),
            support: 1,
            utility: 0.0,
            revision: 0,
            outcomes: vec![OutcomeStat::default(); self.config.motor_cells],
        });

        Some(self.units.len() - 1)
    }

    fn best_matching_unit(&self, trace: &PhaseVector) -> Option<(usize, f32)> {
        let mut best: Option<(usize, f32)> = None;

        for (idx, unit) in self.units.iter().enumerate() {
            let similarity = unit.prototype.similarity(trace);
            if best.map(|(_, score)| similarity > score).unwrap_or(true) {
                best = Some((idx, similarity));
            }
        }

        best
    }

    fn robust_shape_trace(&self, raster: &[f32]) -> Option<ShapeTrace> {
        let points = raster
            .iter()
            .enumerate()
            .filter(|(_, value)| **value >= 0.5)
            .map(|(idx, _)| ((idx % self.config.width) as i32, (idx / self.config.width) as i32))
            .collect::<Vec<_>>();

        if points.len() < 3 {
            return None;
        }

        // Translation-equivariant oriented triangle fragments. Every visible
        // triple is retained; no task-specific offset is selected. A one-point
        // dropout leaves one exact learned triple, while a distractor preserves
        // all original triples among additional ones. Keeping the two bound
        // displacements together avoids the aliasing of an unordered pair bag.
        let mut oriented = Vec::new();
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                for k in (j + 1)..points.len() {
                    let dx1 = points[j].0 - points[i].0;
                    let dy1 = points[j].1 - points[i].1;
                    let dx2 = points[k].0 - points[i].0;
                    let dy2 = points[k].1 - points[i].1;

                    let fields = [
                        (dx1 + 16) as u64,
                        (dy1 + 16) as u64,
                        (dx2 + 16) as u64,
                        (dy2 + 16) as u64,
                    ];

                    let token = 0x4f52_5452_4900_0000u64
                        ^ (fields[0] << 18)
                        ^ (fields[1] << 12)
                        ^ (fields[2] << 6)
                        ^ fields[3];
                    oriented.push(token);
                }
            }
        }

        // Normalized triangle fragments. These carry no orientation or absolute
        // scale and therefore provide a second, generic shape-equivalence view.
        let mut invariant = Vec::new();
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                for k in (j + 1)..points.len() {
                    let mut d2 = [
                        Self::distance_squared(points[i], points[j]),
                        Self::distance_squared(points[i], points[k]),
                        Self::distance_squared(points[j], points[k]),
                    ];
                    d2.sort_unstable();
                    let max = d2[2];
                    if max == 0 {
                        continue;
                    }

                    let q0 = ((d2[0] * 255 + max / 2) / max).min(255) as u64;
                    let q1 = ((d2[1] * 255 + max / 2) / max).min(255) as u64;
                    let token = 0x5348_4150_4500_0000u64 ^ (q0 << 8) ^ q1;
                    invariant.push(token);
                }
            }
        }

        if oriented.is_empty() && invariant.is_empty() {
            None
        } else {
            Some(ShapeTrace::from_fragments(oriented, invariant))
        }
    }

    fn distance_squared(a: (i32, i32), b: (i32, i32)) -> u64 {
        let dx = (a.0 - b.0) as i64;
        let dy = (a.1 - b.1) as i64;
        (dx * dx + dy * dy) as u64
    }

    fn relational_trace(&self, raster: &[f32]) -> Option<PhaseVector> {
        let active: Vec<usize> = raster
            .iter()
            .enumerate()
            .filter(|(_, value)| **value >= 0.5)
            .map(|(idx, _)| idx)
            .collect();

        if active.len() < self.config.min_active {
            return None;
        }

        let mut relations = Vec::new();
        for i in 0..active.len() {
            for j in (i + 1)..active.len() {
                let from = &self.position_roles[active[i]];
                let to = &self.position_roles[active[j]];
                relations.push(to.unbind(from));
            }
        }

        if relations.is_empty() {
            return None;
        }

        let refs: Vec<&PhaseVector> = relations.iter().collect();
        Some(PhaseVector::bundle(&refs))
    }

    fn assert_raster(&self, raster: &[f32]) {
        assert_eq!(raster.len(), self.config.width * self.config.height);
        assert!(raster
            .iter()
            .all(|x| x.is_finite() && *x >= 0.0 && *x <= 1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raster(points: &[(usize, usize)]) -> Vec<f32> {
        let mut values = vec![0.0; 12 * 12];
        for (x, y) in points {
            values[y * 12 + x] = 1.0;
        }
        values
    }

    #[test]
    fn relational_phase_trace_reuses_same_unit_after_translation() {
        let mut field = EvoRasterField::new(RasterFieldConfig::for_raster(12, 12, 2), 192);

        let first = raster(&[(1, 1), (2, 1), (3, 1)]);
        field.observe_factual(&first, 0, true);
        assert_eq!(field.units().len(), 1);
        let first_id = field.active_unit_ids(&first);
        assert_eq!(first_id.len(), 1);

        let shifted = raster(&[(7, 8), (8, 8), (9, 8)]);
        field.observe_factual(&shifted, 0, true);

        assert_eq!(
            field.units().len(),
            1,
            "translation must reuse the same relational carrier unit"
        );
        assert_eq!(field.active_unit_ids(&shifted), first_id);
    }

    #[test]
    fn different_relations_recruit_different_units() {
        let mut field = EvoRasterField::new(RasterFieldConfig::for_raster(12, 12, 2), 192);

        let horizontal = raster(&[(2, 2), (3, 2), (4, 2)]);
        let vertical = raster(&[(2, 2), (2, 3), (2, 4)]);
        field.observe_factual(&horizontal, 0, true);
        field.observe_factual(&vertical, 1, true);

        assert_eq!(field.units().len(), 2);
        assert_ne!(
            field.active_unit_ids(&horizontal),
            field.active_unit_ids(&vertical)
        );
    }

    #[test]
    fn robust_shape_trace_survives_generic_geometric_nuisances() {
        let field = EvoRasterField::new(RasterFieldConfig::for_raster(12, 12, 2), 192);
        let clean = raster(&[(1, 1), (3, 1), (1, 3), (4, 4)]);
        let rotation = raster(&[(8, 1), (8, 3), (6, 1), (5, 4)]);
        let scale = raster(&[(1, 1), (5, 1), (1, 5), (7, 7)]);
        let dropout = raster(&[(1, 1), (3, 1), (1, 3)]);
        let distractor = raster(&[(1, 1), (3, 1), (1, 3), (4, 4), (9, 9)]);

        let base = field.encode_robust_shape_trace(&clean).unwrap();
        assert_eq!(base.similarity(&field.encode_robust_shape_trace(&rotation).unwrap()), 1.0);
        assert_eq!(base.similarity(&field.encode_robust_shape_trace(&scale).unwrap()), 1.0);
        assert!(base.similarity(&field.encode_robust_shape_trace(&dropout).unwrap()) >= 0.99);
        assert!(base.similarity(&field.encode_robust_shape_trace(&distractor).unwrap()) >= 0.99);
    }
}
