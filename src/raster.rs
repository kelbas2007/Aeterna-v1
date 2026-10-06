use std::collections::BTreeSet;

use crate::hdc::PhaseVector;

#[derive(Debug, Clone)]
pub struct RasterFieldConfig {
    pub width: usize,
    pub height: usize,
    pub motor_cells: usize,
    pub patch_side: usize,
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
            patch_side: 3,
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
    slot_roles: Vec<PhaseVector>,
    units: Vec<PhaseFieldUnit>,
    next_id: u64,
}

impl EvoRasterField {
    pub fn new(config: RasterFieldConfig, hdc_dim: usize) -> Self {
        assert!(config.width > 0 && config.height > 0);
        assert!(config.motor_cells > 0);
        assert!(config.patch_side > 0);
        assert!(config.patch_side <= config.width && config.patch_side <= config.height);
        assert!(config.min_active > 0);
        assert!(config.max_units > 0);

        // These are generic local receptive-field slots, shared at every raster
        // translation. They do not encode world coordinates or task semantics.
        let slots = config.patch_side * config.patch_side;
        let slot_roles = (0..slots)
            .map(|i| PhaseVector::from_seed(hdc_dim, 0xC011_AE00 + i as u64))
            .collect();

        Self {
            config,
            slot_roles,
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

    pub fn active_unit_ids(&self, raster: &[f32]) -> Vec<u64> {
        self.assert_raster(raster);
        let traces = self.patch_traces(raster);
        self.active_units_from_traces(&traces)
            .into_iter()
            .map(|idx| self.units[idx].id)
            .collect()
    }

    pub fn observe_factual(&mut self, raster: &[f32], action: usize, need: bool) {
        assert!(action < self.config.motor_cells);
        self.assert_raster(raster);

        if !self.config.learning_enabled {
            return;
        }

        let traces = self.patch_traces(raster);

        if self.config.formation_enabled {
            for trace in &traces {
                self.match_or_recruit(trace);
            }
        }

        let active = self.active_units_from_traces(&traces);
        for idx in active {
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
    }

    pub fn motor_evidence(&self, raster: &[f32]) -> Vec<f32> {
        self.assert_raster(raster);
        let mut evidence = vec![0.0; self.config.motor_cells];

        if !self.config.readout_enabled || self.units.is_empty() {
            return evidence;
        }

        let traces = self.patch_traces(raster);
        let active = self.active_units_from_traces(&traces);

        for idx in active {
            let unit = &self.units[idx];
            for (action, stat) in unit.outcomes.iter().enumerate() {
                evidence[action] += stat.signed_value(self.config.min_readout_support)
                    * (0.5 + 0.5 * unit.utility.max(0.0));
            }
        }

        evidence
    }

    fn match_or_recruit(&mut self, trace: &PhaseVector) {
        let mut best: Option<(usize, f32)> = None;
        for (idx, unit) in self.units.iter().enumerate() {
            let similarity = unit.prototype.similarity(trace);
            if best.map(|(_, score)| similarity > score).unwrap_or(true) {
                best = Some((idx, similarity));
            }
        }

        if let Some((idx, similarity)) = best {
            if similarity >= self.config.match_threshold {
                self.units[idx].support = self.units[idx].support.saturating_add(1);
                return;
            }
        }

        if self.units.len() >= self.config.max_units {
            return;
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
    }

    fn active_units_from_traces(&self, traces: &[PhaseVector]) -> Vec<usize> {
        let mut active = BTreeSet::new();

        for trace in traces {
            let mut best: Option<(usize, f32)> = None;
            for (idx, unit) in self.units.iter().enumerate() {
                let similarity = unit.prototype.similarity(trace);
                if best.map(|(_, score)| similarity > score).unwrap_or(true) {
                    best = Some((idx, similarity));
                }
            }

            if let Some((idx, similarity)) = best {
                if similarity >= self.config.match_threshold {
                    active.insert(idx);
                }
            }
        }

        active.into_iter().collect()
    }

    fn patch_traces(&self, raster: &[f32]) -> Vec<PhaseVector> {
        let mut traces = Vec::new();
        let side = self.config.patch_side;

        for top in 0..=(self.config.height - side) {
            for left in 0..=(self.config.width - side) {
                let mut active_roles = Vec::new();

                for local_y in 0..side {
                    for local_x in 0..side {
                        let global_x = left + local_x;
                        let global_y = top + local_y;
                        let value = raster[global_y * self.config.width + global_x];
                        if value >= 0.5 {
                            let slot = local_y * side + local_x;
                            active_roles.push(&self.slot_roles[slot]);
                        }
                    }
                }

                if active_roles.len() >= self.config.min_active {
                    traces.push(PhaseVector::bundle(&active_roles));
                }
            }
        }

        traces
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
    fn learned_local_phase_motif_reappears_after_translation() {
        let mut field = EvoRasterField::new(RasterFieldConfig::for_raster(12, 12, 2), 128);

        let first = raster(&[(1, 1), (2, 1), (3, 1)]);
        field.observe_factual(&first, 0, true);
        assert!(!field.units().is_empty());

        let learned = field.units().len();
        let shifted = raster(&[(7, 8), (8, 8), (9, 8)]);
        field.observe_factual(&shifted, 0, true);

        // Shared local receptive-field roles mean the translated relation reuses
        // existing carrier motifs rather than requiring one absolute-address unit
        // for every translation.
        assert!(field.units().len() <= learned + 2);
    }
}
