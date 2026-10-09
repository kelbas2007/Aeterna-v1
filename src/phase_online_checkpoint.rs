// Versioned, data-only persistence for the online mode. Neither REAL input nor
// actuator authority is serialized. No code or external paths are evaluated.

const ONLINE_CHECKPOINT_VERSION: u32 = 8;
const ONLINE_CHECKPOINT_MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OnlineObservationSnapshot {
    cell: usize,
    sensory: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OnlineSnapshot {
    version: u32,
    config: super::EvoConfig,
    native: PhaseNativeConfig,
    online: PhaseOnlineConfig,
    cells: Vec<PhaseCell>,
    synapses: Vec<PhaseSynapse>,
    observations: Vec<OnlineObservationSnapshot>,
    circuits: Vec<PhaseCircuitInfo>,
    #[serde(default)]
    rules: Option<PhaseRuleState>,
    #[serde(default)]
    partial: Option<PhasePartialState>,
    #[serde(default)]
    vector: Option<PhaseVectorState>,
}

fn invalid_online_checkpoint(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

impl OnlineSnapshot {
    fn validate(&self) -> std::io::Result<()> {
        let fail = || invalid_online_checkpoint("incompatible or invalid online checkpoint");
        let cfg = &self.config;
        let native = &self.native;
        let unit = |x: f32| x.is_finite() && (0.0..=1.0).contains(&x);
        let role_count = cfg
            .sensory_cells
            .checked_add(cfg.motor_cells)
            .ok_or_else(fail)?;
        let dormant_start = role_count.checked_add(1).ok_or_else(fail)?;
        let total = dormant_start
            .checked_add(cfg.dormant_cells)
            .ok_or_else(fail)?;
        let role_dimensions = role_count.checked_mul(cfg.hdc_dim).ok_or_else(fail)?;
        if !(1..=ONLINE_CHECKPOINT_VERSION).contains(&self.version)
            || (self.version == 1 && self.rules.is_some())
            || (self.version < 3 && self.partial.is_some())
            || cfg.sensory_cells == 0
            || cfg.motor_cells == 0
            || cfg.hdc_dim == 0
            || role_dimensions > ONLINE_CHECKPOINT_MAX_BYTES / std::mem::size_of::<f32>()
            || total != self.cells.len()
            || cfg.min_recruit_support == 0
            || !unit(cfg.weight_learning_rate)
            || !unit(cfg.phase_learning_rate)
            || !unit(cfg.eligibility_decay)
            || !unit(cfg.residual_recruit_threshold)
            || !(1..=1024).contains(&native.horizon)
            || !unit(native.match_threshold)
            || native.match_threshold == 0.0
            || !unit(native.coherence_floor)
            || native.coherence_floor >= 1.0
            || !unit(native.discount)
            || native.discount >= 1.0
            || self.online.max_states == 0
            || self.online.max_states > cfg.dormant_cells
            || self.observations.len() > self.online.max_states
        {
            return Err(fail());
        }
        if self.cells.iter().enumerate().any(|(i, c)| {
            !c.phase.is_finite()
                || !c.charge.is_finite()
                || c.charge != 0.0
                || !c.threshold.is_finite()
                || !c.utility.is_finite()
                || (i < dormant_start && !c.recruited)
        }) || self.synapses.iter().any(|s| {
            s.from >= total
                || s.to >= total
                || !unit(s.weight)
                || !s.phase_offset.is_finite()
                || !unit(s.eligibility)
                || !unit(s.confidence)
        }) {
            return Err(fail());
        }
        let mut allocated = std::collections::BTreeSet::new();
        for observation in &self.observations {
            if !(dormant_start..total).contains(&observation.cell)
                || !self.cells[observation.cell].recruited
                || !allocated.insert(observation.cell)
                || observation.sensory.len() != cfg.sensory_cells
                || observation.sensory.iter().any(|&x| !unit(x))
            {
                return Err(fail());
            }
        }
        let receptors = allocated.clone();
        let mut used_synapses = std::collections::BTreeSet::new();
        for circuit in &self.circuits {
            if !(dormant_start..total).contains(&circuit.relay_cell)
                || !self.cells[circuit.relay_cell].recruited
                || !allocated.insert(circuit.relay_cell)
                || circuit.support == 0
                || circuit.counterexamples.iter().any(|(_, x)| !unit(*x))
                || circuit
                    .indices()
                    .iter()
                    .any(|&i| i >= self.synapses.len() || !used_synapses.insert(i))
            {
                return Err(fail());
            }
            let aff = &self.synapses[circuit.afferent_synapse];
            let succ = &self.synapses[circuit.successor_synapse];
            let outcome = &self.synapses[circuit.outcome_synapse];
            let motor = &self.synapses[circuit.motor_synapse];
            if !receptors.contains(&aff.from)
                || aff.to != circuit.relay_cell
                || succ.from != circuit.relay_cell
                || !receptors.contains(&succ.to)
                || outcome.from != circuit.relay_cell
                || outcome.to != role_count
                || motor.from != circuit.relay_cell
                || !(cfg.sensory_cells..role_count).contains(&motor.to)
            {
                return Err(fail());
            }
        }
        if let Some(rules) = &self.rules {
            if !rules.config.valid()
                || (self.version < 4 && rules.language != PhaseRuleLanguage::SingleChannel)
                || (self.version < 5 && rules.adaptive.is_some())
                || cfg.sensory_cells > 16
                || cfg.motor_cells > 32
                || rules.actions.len() != cfg.motor_cells
                || !self.observations.is_empty()
                || !self.circuits.is_empty()
                || rules.factual_sequence == u64::MAX
            {
                return Err(fail());
            }
            let templates = rule_templates(rules.language, cfg.sensory_cells);
            if rule_synapse_budget(&templates, cfg.sensory_cells, cfg.motor_cells)
                .is_none_or(|n| n > MAX_PHASE_RULE_SYNAPSES)
            {
                return Err(fail());
            }
            let mut sources = std::collections::BTreeSet::new();
            let mut observation_total = 0_u64;
            for action in &rules.actions {
                observation_total = observation_total
                    .checked_add(action.observations)
                    .ok_or_else(fail)?;
                if action.outputs.len() != cfg.sensory_cells
                    || action.evidence.len() > rules.config.evidence_capacity
                    || action.observations < action.evidence.len() as u64
                    || action.revision > action.observations
                    || action.revision == u64::MAX
                    || (action.observations == 0) != action.evidence.is_empty()
                {
                    return Err(fail());
                }
                for (i, evidence) in action.evidence.iter().enumerate() {
                    if evidence.sequence == 0
                        || evidence.sequence > rules.factual_sequence
                        || !sources.insert(evidence.sequence)
                        || !rule_vector_valid(&evidence.pre, cfg.sensory_cells)
                        || !rule_vector_valid(&evidence.post, cfg.sensory_cells)
                        || (i > 0 && action.evidence[i - 1].sequence >= evidence.sequence)
                        || action.evidence[..i]
                            .iter()
                            .any(|e| rule_distance(&e.pre, &evidence.pre) <= rules.config.tolerance)
                    {
                        return Err(fail());
                    }
                }
                for output in &action.outputs {
                    if !(dormant_start..total).contains(&output.cell)
                        || !self.cells[output.cell].recruited
                        || !allocated.insert(output.cell)
                        || output.candidates.len() != templates.len()
                        || (!action.evidence.is_empty()
                            && !output.candidates.iter().any(|c| c.active))
                    {
                        return Err(fail());
                    }
                    for (candidate, template) in output.candidates.iter().zip(&templates) {
                        if candidate.source != template.source
                            || candidate.orientation != template.orientation
                            || candidate.synapse >= self.synapses.len()
                            || !used_synapses.insert(candidate.synapse)
                            || (action.evidence.is_empty() && candidate.active)
                        {
                            return Err(fail());
                        }
                        let synapse = &self.synapses[candidate.synapse];
                        let from = if template.orientation == 0 {
                            role_count
                        } else {
                            template.source
                        };
                        if synapse.from != from
                            || synapse.to != output.cell
                            || !(0.0..std::f32::consts::TAU).contains(&synapse.phase_offset)
                        {
                            return Err(fail());
                        }
                        match (&candidate.secondary, template.secondary) {
                            (None, None) => {}
                            (Some(other), Some((source, orientation))) => {
                                if other.source != source
                                    || other.orientation != orientation
                                    || other.synapse >= self.synapses.len()
                                    || !used_synapses.insert(other.synapse)
                                {
                                    return Err(fail());
                                }
                                let synapse = &self.synapses[other.synapse];
                                if synapse.from != source
                                    || synapse.to != output.cell
                                    || !(0.0..std::f32::consts::TAU).contains(&synapse.phase_offset)
                                {
                                    return Err(fail());
                                }
                            }
                            _ => return Err(fail()),
                        }
                    }
                }
            }
            if observation_total != rules.factual_sequence {
                return Err(fail());
            }
            if let Some(adaptive) = &rules.adaptive {
                if rules.factual_sequence != 0
                    || self.partial.as_ref().is_some_and(|p| p.uncertainty.is_none())
                    || !adaptive.config.valid()
                    || adaptive.actions.len() != cfg.motor_cells
                    || adaptive.factual_sequence == u64::MAX
                    || self.synapses.len() > MAX_PHASE_RULE_SYNAPSES
                {
                    return Err(fail());
                }
                let mut events = std::collections::BTreeSet::new();
                let mut observations = 0_u64;
                for action in &adaptive.actions {
                    observations = observations
                        .checked_add(action.observations)
                        .ok_or_else(fail)?;
                    if action.slots.len() != adaptive.config.model_slots
                        || action.evidence.len() > adaptive.config.evidence_capacity
                        || action.pending.len() >= adaptive.config.change_support
                        || action.archives.len() > adaptive.config.model_slots
                        || action.revision == u64::MAX
                        || action.evictions == u64::MAX
                        || action.reactivations > action.observations
                        || action.observations < action.evidence.len() as u64
                    {
                        return Err(fail());
                    }
                    for (i, e) in action.evidence.iter().enumerate() {
                        if e.sequence == 0
                            || e.sequence > adaptive.factual_sequence
                            || !events.insert(e.sequence)
                            || !rule_vector_valid(&e.pre, cfg.sensory_cells)
                            || !rule_vector_valid(&e.post, cfg.sensory_cells)
                            || (i > 0 && action.evidence[i - 1].sequence >= e.sequence)
                            || action.evidence[..i].iter().any(|old| {
                                rule_distance(&old.pre, &e.pre)
                                    <= 2.0 * adaptive.config.residual_tolerance
                            })
                        {
                            return Err(fail());
                        }
                    }
                    for (i, e) in action.pending.iter().enumerate() {
                        if e.sequence == 0
                            || e.sequence > adaptive.factual_sequence
                            || !rule_vector_valid(&e.pre, cfg.sensory_cells)
                            || !rule_vector_valid(&e.post, cfg.sensory_cells)
                            || (i > 0 && action.pending[i - 1].sequence >= e.sequence)
                            || action.pending[..i].iter().any(|old| {
                                rule_distance(&old.pre, &e.pre)
                                    <= 2.0 * adaptive.config.residual_tolerance
                            })
                        {
                            return Err(fail());
                        }
                    }
                    let mut occupied = std::collections::BTreeSet::new();
                    for model in action.active.iter().chain(&action.archives) {
                        if model.slots.len() != if model.condition.is_some() { 2 } else { 1 }
                            || (model.condition.is_some() && !adaptive.config.allow_conditions)
                            || model.supports.len() != model.slots.len()
                            || model.supports.iter().any(|&n| {
                                n < adaptive.config.min_support
                                    || n > adaptive.config.evidence_capacity
                            })
                            || model.evidence_sources.len() < adaptive.config.min_support
                            || model.evidence_sources.len() > adaptive.config.evidence_capacity
                            || model.supports.iter().sum::<usize>() > model.evidence_sources.len()
                            || model.evidence_sources.windows(2).any(|p| p[0] >= p[1])
                            || model
                                .evidence_sources
                                .iter()
                                .any(|&s| s == 0 || s > adaptive.factual_sequence)
                            || model
                                .slots
                                .iter()
                                .any(|&s| s >= action.slots.len() || !occupied.insert(s))
                            || model.condition.as_ref().is_some_and(|c| {
                                c.source >= cfg.sensory_cells
                                    || !c.lower.is_finite()
                                    || !c.upper.is_finite()
                                    || !(0.0..1.0).contains(&c.lower)
                                    || !(0.0..1.0).contains(&c.upper)
                                    || c.upper - c.lower <= 2.0 * adaptive.config.residual_tolerance
                            })
                        {
                            return Err(fail());
                        }
                    }
                    for (s, slot) in action.slots.iter().enumerate() {
                        if slot.outputs.len() != cfg.sensory_cells {
                            return Err(fail());
                        }
                        for output in &slot.outputs {
                            if !(dormant_start..total).contains(&output.cell)
                                || !self.cells[output.cell].recruited
                                || !allocated.insert(output.cell)
                                || output.candidates.len() != templates.len()
                                || output.candidates.iter().filter(|c| c.active).count()
                                    != usize::from(occupied.contains(&s))
                            {
                                return Err(fail());
                            }
                            for (c, t) in output.candidates.iter().zip(&templates) {
                                if c.source != t.source
                                    || c.orientation != t.orientation
                                    || c.synapse >= self.synapses.len()
                                    || !used_synapses.insert(c.synapse)
                                {
                                    return Err(fail());
                                }
                                let link = &self.synapses[c.synapse];
                                if link.from
                                    != if t.orientation == 0 {
                                        role_count
                                    } else {
                                        t.source
                                    }
                                    || link.to != output.cell
                                    || !(0.0..std::f32::consts::TAU).contains(&link.phase_offset)
                                {
                                    return Err(fail());
                                }
                                match (&c.secondary, t.secondary) {
                                    (None, None) => {}
                                    (Some(other), Some((source, orientation))) => {
                                        if other.source != source
                                            || other.orientation != orientation
                                            || other.synapse >= self.synapses.len()
                                            || !used_synapses.insert(other.synapse)
                                        {
                                            return Err(fail());
                                        }
                                        let link = &self.synapses[other.synapse];
                                        if link.from != source
                                            || link.to != output.cell
                                            || !(0.0..std::f32::consts::TAU)
                                                .contains(&link.phase_offset)
                                        {
                                            return Err(fail());
                                        }
                                    }
                                    _ => return Err(fail()),
                                }
                            }
                        }
                    }
                }
                if observations != adaptive.factual_sequence {
                    return Err(fail());
                }
            }
        }
        if let Some(partial) = &self.partial {
            if self.rules.is_none()
                || (self.version < 5 && partial.inverse_enabled)
                || (self.version < 6 && partial.uncertainty.is_some())
                || partial.uncertainty.as_ref().is_some_and(|c| {
                    !c.valid()
                        || !self.rules.as_ref().is_some_and(|r| r.adaptive.is_some())
                })
                || !partial.config.valid()
                || partial.masks.len() != cfg.motor_cells
                || partial.factual_sequence == u64::MAX
                || partial.episode.is_some()
            {
                return Err(fail());
            }
            let mut sources = std::collections::BTreeSet::new();
            let mut observations = 0_u64;
            for mask in &partial.masks {
                observations = observations
                    .checked_add(mask.observations)
                    .ok_or_else(fail)?;
                if mask.visible.len() != cfg.sensory_cells
                    || mask.revision > mask.observations
                    || mask.source_ids.len() > partial.config.min_mask_support
                    || mask.source_ids.len() as u64 > mask.observations
                    || (mask.observations == 0) != mask.source_ids.is_empty()
                    || mask.source_ids.windows(2).any(|p| p[0] >= p[1])
                    || mask
                        .source_ids
                        .iter()
                        .any(|&id| id == 0 || id > partial.factual_sequence || !sources.insert(id))
                {
                    return Err(fail());
                }
            }
            if observations != partial.factual_sequence {
                return Err(fail());
            }
        }
        if let Some(vector) = &self.vector {
            if self.version<7 || (self.version<8 && vector.induction.is_some()) || self.rules.is_some() || self.partial.is_some() || !self.observations.is_empty() || !self.circuits.is_empty()
                || !vector.validate_snapshot(cfg,&self.cells,&self.synapses,&mut allocated,&mut used_synapses) { return Err(fail()); }
        }
        if used_synapses.len() != self.synapses.len()
            || (dormant_start..total).any(|i| self.cells[i].recruited != allocated.contains(&i))
        {
            return Err(fail());
        }
        Ok(())
    }
}

impl EvoPhase {
    /// Encode acquired online knowledge. Callers can write these bytes to disk.
    /// Unsupported mixed modes are rejected instead of silently losing state.
    pub fn online_checkpoint_bytes(&self) -> std::io::Result<Vec<u8>> {
        let state = self
            .phase_native
            .as_ref()
            .ok_or_else(|| invalid_online_checkpoint("native model required"))?;
        let online = state
            .online
            .clone()
            .ok_or_else(|| invalid_online_checkpoint("online mode required"))?;
        if state.drive.is_some()
            || state.concepts.is_some()
            || state.deep.is_some()
            || state.contextual.is_some()
            || state.perceptual.is_some()
            || state.compositional.is_some()
            || state.meta_control.is_some()
            || state.temporal_evidence.is_some()
            || self.concept_memory.is_some()
        {
            return Err(invalid_online_checkpoint(
                "mixed cognitive modes cannot be serialized as online state",
            ));
        }
        let observations = state
            .receptors
            .iter()
            .map(|r| {
                let CarrierTrace::Sensory(sensory) = &r.trace else {
                    return Err(invalid_online_checkpoint("unsupported receptor encoding"));
                };
                Ok(OnlineObservationSnapshot {
                    cell: r.cell,
                    sensory: sensory.clone(),
                })
            })
            .collect::<std::io::Result<Vec<_>>>()?;
        let mut cells = self.cells.clone();
        for cell in &mut cells {
            cell.charge = 0.0;
        }
        let mut partial = state.partial.clone();
        if let Some(partial) = partial.as_mut() {
            partial.episode = None;
        }
        let mut vector=state.vector.clone();
        if let Some(vector)=vector.as_mut() { vector.episode=None; }
        let snapshot = OnlineSnapshot {
            version: ONLINE_CHECKPOINT_VERSION,
            config: self.config.clone(),
            native: state.config.clone(),
            online,
            cells,
            synapses: self.synapses.clone(),
            observations,
            circuits: state.circuits.clone(),
            rules: state.rules.clone(),
            partial,
            vector,
        };
        snapshot.validate()?;
        let bytes = serde_json::to_vec(&snapshot).map_err(std::io::Error::other)?;
        if bytes.len() > ONLINE_CHECKPOINT_MAX_BYTES {
            return Err(invalid_online_checkpoint(
                "online checkpoint exceeds byte limit",
            ));
        }
        Ok(bytes)
    }

    /// Load a versioned snapshot with bounds/index validation before allocation
    /// of a carrier. A new factual observation is required before any action.
    pub fn from_online_checkpoint(bytes: &[u8]) -> std::io::Result<Self> {
        if bytes.len() > ONLINE_CHECKPOINT_MAX_BYTES {
            return Err(invalid_online_checkpoint(
                "online checkpoint exceeds byte limit",
            ));
        }
        let snapshot: OnlineSnapshot =
            serde_json::from_slice(bytes).map_err(|e| invalid_online_checkpoint(&e.to_string()))?;
        snapshot.validate()?;
        let mut evo = Self::new(snapshot.config);
        evo.enable_phase_native_planning(snapshot.native);
        if !evo.enable_phase_native_online_learning(snapshot.online) {
            return Err(invalid_online_checkpoint(
                "could not initialize online mode",
            ));
        }
        evo.cells = snapshot.cells;
        evo.synapses = snapshot.synapses;
        let state = evo.phase_native.as_mut().expect("initialized native state");
        state.receptors = snapshot
            .observations
            .into_iter()
            .map(|o| Receptor {
                trace: CarrierTrace::Sensory(o.sensory),
                cell: o.cell,
            })
            .collect();
        state.circuits = snapshot.circuits;
        state.rules = snapshot.rules;
        state.partial = snapshot.partial;
        state.vector = snapshot.vector;
        Ok(evo)
    }
}
