// A context-conditioned projection of the SAME U1 fields. Visible interface
// context never supplies motor/class roles. No separate chooser is created.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ContextualMetaRow {
    synapses: [PhaseSynapse; META_FIELD_COUNT],
    draws: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ContextualMetaProjection {
    prior: [PhaseSynapse; META_FIELD_COUNT],
    rows: std::collections::BTreeMap<i64, ContextualMetaRow>,
    current: Option<i64>,
}

impl PhaseMetaControlState {
    fn projection_synapse<'a>(
        &'a self,
        physical: &'a [PhaseSynapse],
        index: usize,
    ) -> &'a PhaseSynapse {
        if let Some(c) = &self.contextual {
            c.current
                .and_then(|key| c.rows.get(&key))
                .map_or(&c.prior[index], |r| &r.synapses[index])
        } else {
            &physical[self.weight_synapses[index]]
        }
    }
}

impl EvoPhase {
    pub(crate) fn enable_contextual_meta_projection(&mut self) -> bool {
        let Some(meta) = self
            .phase_native
            .as_mut()
            .and_then(|n| n.meta_control.as_mut())
        else {
            return false;
        };
        if meta.contextual.is_some() || !meta.config.learning_enabled {
            return false;
        }
        meta.contextual = Some(ContextualMetaProjection {
            prior: std::array::from_fn(|i| self.synapses[meta.weight_synapses[i]].clone()),
            rows: Default::default(),
            current: None,
        });
        true
    }

    pub(crate) fn can_select_meta_context(&self, context: i64) -> bool {
        if self
            .phase_native
            .as_ref()
            .and_then(|n| n.meta_control.as_ref())
            .is_none_or(|m| m.contextual.is_none())
        {
            return true;
        }
        if !(0..=262143).contains(&context) {
            return false;
        }
        self.phase_native
            .as_ref()
            .and_then(|n| n.meta_control.as_ref())
            .is_none_or(|m| {
                m.contextual.as_ref().is_none_or(|c| {
                    !m.config.learning_enabled || c.rows.contains_key(&context) || c.rows.len() < 8
                })
            })
    }

    pub(crate) fn select_meta_context(&mut self, context: i64) -> bool {
        if !self.can_select_meta_context(context) {
            return false;
        }
        let Some(meta) = self
            .phase_native
            .as_mut()
            .and_then(|n| n.meta_control.as_mut())
        else {
            return true;
        };
        if let Some(c) = &mut meta.contextual {
            if meta.config.learning_enabled {
                c.rows.entry(context).or_insert_with(|| ContextualMetaRow {
                    synapses: c.prior.clone(),
                    draws: 0,
                });
            }
            c.current = Some(context);
        }
        true
    }

    pub(crate) fn contextual_meta_entropy(&self) -> Option<u64> {
        let c = self
            .phase_native
            .as_ref()?
            .meta_control
            .as_ref()?
            .contextual
            .as_ref()?;
        let context = c.current?;
        let draws = c.rows.get(&context).map_or(0, |r| r.draws);
        Some(
            (context as u64).wrapping_mul(0x9e3779b97f4a7c15)
                ^ draws.wrapping_mul(0xbf58476d1ce4e5b9),
        )
    }

    pub(crate) fn note_contextual_meta_action(&mut self) {
        if let Some(c) = self
            .phase_native
            .as_mut()
            .and_then(|n| n.meta_control.as_mut())
            .and_then(|m| m.contextual.as_mut())
        {
            if let Some(row) = c.current.and_then(|key| c.rows.get_mut(&key)) {
                row.draws = row.draws.saturating_add(1);
            }
        }
    }

    pub(crate) fn contextual_meta_domains(&self) -> Option<Vec<i64>> {
        Some(
            self.phase_native
                .as_ref()?
                .meta_control
                .as_ref()?
                .contextual
                .as_ref()?
                .rows
                .keys()
                .copied()
                .collect(),
        )
    }

    pub(crate) fn valid_current_meta_context(&self, context: i64, common: bool) -> bool {
        self.phase_native
            .as_ref()
            .and_then(|n| n.meta_control.as_ref())
            .and_then(|m| m.contextual.as_ref())
            .is_none_or(|c| common && c.current.map_or(c.rows.is_empty(), |key| key == context))
    }
}

impl ContextualMetaProjection {
    fn validate(&self, physical: &[PhaseSynapse]) -> bool {
        let unit = |x: f32| x.is_finite() && (0.0..=1.0).contains(&x);
        let valid = |row: &[PhaseSynapse; META_FIELD_COUNT]| {
            row.iter().zip(physical).all(|(a, b)| {
                a.from == b.from
                    && a.to == b.to
                    && unit(a.weight)
                    && unit(a.confidence)
                    && unit(a.eligibility)
                    && a.phase_offset.is_finite()
            })
        };
        physical.len() == META_FIELD_COUNT
            && self.rows.len() <= 8
            && valid(&self.prior)
            && self.current.is_none_or(|k| (0..=262143).contains(&k))
            && self
                .rows
                .iter()
                .all(|(&key, r)| (0..=262143).contains(&key) && valid(&r.synapses))
    }
}

#[cfg(test)]
mod contextual_meta_tests {
    use super::*;

    #[test]
    fn one_u1_keeps_contextual_credit_and_exploration_across_restore_and_freeze() {
        let mut evo = EvoPhase::new(super::super::EvoConfig {
            sensory_cells: 32,
            motor_cells: 16,
            dormant_cells: 32,
            hdc_dim: 32,
            ..Default::default()
        });
        evo.enable_phase_native_planning(PhaseNativeConfig::default());
        assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig::default()));
        assert!(evo.enable_contextual_meta_projection());
        assert!(evo.select_meta_context(9));
        let prior = evo.phase_native_meta_weights().unwrap();
        let untouched_entropy = evo.contextual_meta_entropy();
        assert!(evo.select_meta_context(4));
        for _ in 0..100 {
            assert!(evo.observe_phase_native_meta_utility([1.0, 0.0, 0.0, 0.0, 0.0], 1.0));
            evo.note_contextual_meta_action();
        }
        let learned = evo.phase_native_meta_weights().unwrap();
        assert_ne!(learned, prior);
        assert!(evo.select_meta_context(9));
        assert_eq!(evo.phase_native_meta_weights().unwrap(), prior);
        assert_eq!(evo.contextual_meta_entropy(), untouched_entropy);
        let bytes = evo.constructing_carrier_checkpoint().unwrap();
        let mut evo = EvoPhase::from_constructing_carrier_checkpoint(&bytes).unwrap();
        assert_eq!(bytes, evo.constructing_carrier_checkpoint().unwrap());
        assert!(evo.select_meta_context(4));
        assert_eq!(evo.phase_native_meta_weights().unwrap(), learned);
        evo.set_phase_native_meta_learning_enabled(false);
        evo.observe_phase_native_meta_utility([1.0, 0.0, 0.0, 0.0, 0.0], 0.0);
        assert_eq!(evo.phase_native_meta_weights().unwrap(), learned);
        assert!(evo.select_meta_context(123));
        assert_eq!(evo.phase_native_meta_weights().unwrap(), prior);
    }
}
