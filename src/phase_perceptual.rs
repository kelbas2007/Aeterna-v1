// G22. Evidence-gated invention of a bounded raw perceptual variable.
// Included inside phase_native.rs and therefore shares the same private carrier
// cells/synapses and ordinary native transition learner as P1-G21.

const PERCEPT_CANDIDATE_CAP: usize = 16;
const PERCEPT_DISCOVERY_CAP: usize = 128;
const PERCEPT_MAX_RADIUS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhasePerceptFeature {
    WeakAmplitudeBin(u8),
    WideOffset { dx: u8, dy: u8 },
}

#[derive(Debug, Clone)]
struct PerceptFact {
    base: usize,
    action: usize,
    post: usize,
    features: Vec<PhasePerceptFeature>,
}

#[derive(Debug, Clone)]
pub struct PhasePerceptWitness {
    pub base_cell: usize,
    pub features: [PhasePerceptFeature; 2],
    pub feature_cells: [usize; 2],
    pub successor_cells: [usize; 2],
    pub state_cells: [usize; 2],
    pub input_synapses: [[usize; 2]; 2],
    pub anchor_action: usize,
    pub promoted: bool,
    pub retired: bool,
    pub born_fact: u64,
    pub eligible_observations: u64,
    pub log_evidence: f64,
    pub side_switches: u64,
    last_side: Option<usize>,
    gate_observations: [u32; 2],
}

#[derive(Debug, Clone, Default)]
pub(super) struct PhasePerceptState {
    discovery: Vec<PerceptFact>,
    candidates: Vec<PhasePerceptWitness>,
    factual_events: u64,
}

impl EvoPhase {
    pub fn enable_phase_native_perceptual_refinement(&mut self) -> bool {
        let Some(native) = self.phase_native.as_mut() else { return false; };
        if native.perceptual.is_some() { return false; }
        if self.concept_memory.is_none() { return false; }
        native.perceptual = Some(PhasePerceptState::default());
        true
    }

    pub fn phase_native_perceptual_enabled(&self) -> bool {
        self.phase_native.as_ref().map(|s| s.perceptual.is_some()).unwrap_or(false)
    }

    pub fn phase_native_perceptual_witnesses(&self) -> Vec<PhasePerceptWitness> {
        self.phase_native.as_ref().and_then(|s| s.perceptual.as_ref())
            .map(|p| p.candidates.clone()).unwrap_or_default()
    }

    pub(super) fn is_native_perceptual_synapse(&self, index: usize) -> bool {
        self.phase_native.as_ref().and_then(|s| s.perceptual.as_ref())
            .map(|p| p.candidates.iter().any(|w|
                w.input_synapses.iter().any(|pair| pair.contains(&index))))
            .unwrap_or(false)
    }

    fn phase_raw_features(&self, raster: &[f32]) -> Option<Vec<PhasePerceptFeature>> {
        let memory = self.concept_memory.as_ref()?;
        let (width, height) = memory.raster_shape();
        if raster.len() != width * height { return None; }
        let inherited = memory.inherited_local_radius();
        let mut features = std::collections::BTreeSet::new();

        // New intensity vocabulary below the inherited binary concept threshold.
        // Position is deliberately absent so a learned bin transfers across marker
        // translations.
        for &value in raster {
            if value.is_finite() && value > 0.0 && value < 0.5 {
                let bin = ((value * 16.0).floor() as u8).min(7);
                features.insert(PhasePerceptFeature::WeakAmplitudeBin(bin));
            }
        }

        // Wider translation-invariant geometry not visible to the inherited
        // local-relation radius.
        let points = raster.iter().enumerate()
            .filter(|(_, value)| **value >= 0.5)
            .map(|(index, _)| (index % width, index / width))
            .collect::<Vec<_>>();
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let dx = points[i].0.abs_diff(points[j].0);
                let dy = points[i].1.abs_diff(points[j].1);
                let radius = dx.max(dy);
                if radius > inherited && radius <= PERCEPT_MAX_RADIUS {
                    features.insert(PhasePerceptFeature::WideOffset {
                        dx: dx as u8, dy: dy as u8,
                    });
                }
            }
        }
        Some(features.into_iter().collect())
    }

    fn percept_side(features: &[PhasePerceptFeature], w: &PhasePerceptWitness)
        -> Option<usize>
    {
        let a = features.binary_search(&w.features[0]).is_ok();
        let b = features.binary_search(&w.features[1]).is_ok();
        match (a, b) {
            (true, false) => Some(0),
            (false, true) => Some(1),
            _ => None,
        }
    }

    fn percept_counts(&self, native: &PhaseNativeState, w: &PhasePerceptWitness)
        -> [[u64; 2]; 2]
    {
        let mut counts = [[0u64; 2]; 2];
        let motor = self.motor_cell(w.anchor_action);
        for circuit in &native.circuits {
            if self.synapses[circuit.motor_synapse].to != motor { continue; }
            let from = self.synapses[circuit.afferent_synapse].from;
            let to = self.synapses[circuit.successor_synapse].to;
            for side in 0..2 {
                for outcome in 0..2 {
                    if from == w.state_cells[side] && to == w.successor_cells[outcome] {
                        counts[side][outcome] += circuit.support;
                    }
                }
            }
        }
        counts
    }

    fn percept_candidate_pair(
        old: &[PhasePerceptFeature],
        new: &[PhasePerceptFeature],
    ) -> Option<[PhasePerceptFeature; 2]> {
        let old_only = old.iter().copied().find(|f| new.binary_search(f).is_err())?;
        let new_only = new.iter().copied().find(|f| old.binary_search(f).is_err())?;
        if old_only == new_only { None } else { Some([old_only, new_only]) }
    }

    /// Return (applicable, action). A promoted current-sensory refinement takes
    /// priority over its ambiguous inherited parent representation.
    pub fn phase_native_perceptual_action(&mut self, goal_sensory: &[f32])
        -> (bool, Option<usize>)
    {
        let Some(real) = self.current_real.as_ref() else { return (false, None); };
        let sensory = real.sensory.clone();
        let Some(base) = self.phase_native_abstract_state(&sensory)
            else { return (false, None); };
        let Some(goal) = self.phase_native_abstract_state(goal_sensory)
            else { return (false, None); };
        let Some(raw) = self.phase_raw_features(&sensory) else { return (false, None); };
        let Some(native) = self.phase_native.as_ref() else { return (false, None); };
        let Some(perceptual) = native.perceptual.as_ref() else { return (false, None); };
        let Some(w) = perceptual.candidates.iter()
            .find(|w| w.base_cell == base.cell && !w.retired)
            else { return (false, None); };

        if !w.promoted {
            if native.config.learning_enabled { return (true, Some(w.anchor_action)); }
            return (false, None);
        }

        let Some(side) = Self::percept_side(&raw, w) else { return (true, None); };
        let floor = native.config.coherence_floor;
        let base_link = &self.synapses[w.input_synapses[side][0]];
        let feature_link = &self.synapses[w.input_synapses[side][1]];
        let base_current = if base_link.from == base.cell {
            conductance(&self.cells, base_link, floor)
        } else { 0.0 };
        // The generic raw descriptor encoder supplies current to the acquired
        // physical feature cell only when that learned descriptor is present.
        let feature_current = if feature_link.from == w.feature_cells[side]
            && raw.binary_search(&w.features[side]).is_ok()
        {
            conductance(&self.cells, feature_link, floor)
        } else { 0.0 };
        if base_current.min(feature_current) <= 1.0e-8 {
            return (true, None);
        }
        let entry = w.state_cells[side];

        if native.config.learning_enabled {
            if let Some(action) = (0..self.config.motor_cells)
                .find(|a| !self.phase_drive_action_known_at(native, entry, *a))
            {
                return (true, Some(action));
            }
        }

        (true, self.phase_native_goal_decision_from_cells(entry, goal.cell, None)
            .map(|d| d.first_action))
    }

    /// Factual current-sensory refinement update. Candidate selection may use
    /// the discovery collision, but validation starts strictly after birth.
    pub fn observe_phase_native_perceptual_result(
        &mut self,
        action: usize,
        post: &[f32],
    ) -> Option<usize> {
        if action >= self.config.motor_cells { return None; }
        let pre_sensory = self.current_real.as_ref()?.sensory.clone();
        let pre = self.phase_native_abstract_state(&pre_sensory)?;
        let after = self.phase_native_abstract_state(post)?;
        let raw = self.phase_raw_features(&pre_sensory)?;

        let mut native = self.phase_native.take()?;
        let Some(mut perceptual) = native.perceptual.take() else {
            self.phase_native = Some(native);
            return self.observe_phase_native_rival_probe_result(action, post);
        };
        let learning = native.config.learning_enabled;
        let mut born_now = false;

        if learning {
            perceptual.factual_events = perceptual.factual_events.saturating_add(1);

            let no_candidate = !perceptual.candidates.iter()
                .any(|w| w.base_cell == pre.cell);
            if no_candidate
                && perceptual.candidates.len() < PERCEPT_CANDIDATE_CAP
                && self.config.structural_growth_enabled
            {
                let opposing = perceptual.discovery.iter().rev().find(|fact|
                    fact.base == pre.cell && fact.action == action
                        && fact.post != after.cell).cloned();
                if let Some(old) = opposing {
                    if let Some(feature_pair) =
                        Self::percept_candidate_pair(&old.features, &raw)
                    {
                        let free = self.dormant_range()
                            .filter(|i| !self.cells[*i].recruited)
                            .take(4).collect::<Vec<_>>();
                        if free.len() == 4 {
                            for &cell in &free { self.cells[cell].recruited = true; }
                            let feature_cells = [free[0], free[1]];
                            let states = [free[2], free[3]];
                            let inputs = [
                                [self.native_synapse(pre.cell, states[0]),
                                 self.native_synapse(feature_cells[0], states[0])],
                                [self.native_synapse(pre.cell, states[1]),
                                 self.native_synapse(feature_cells[1], states[1])],
                            ];
                            perceptual.candidates.push(PhasePerceptWitness {
                                base_cell: pre.cell,
                                features: feature_pair,
                                feature_cells,
                                successor_cells: [old.post, after.cell],
                                state_cells: states,
                                input_synapses: inputs,
                                anchor_action: action,
                                promoted: false,
                                retired: false,
                                born_fact: perceptual.factual_events,
                                eligible_observations: 0,
                                log_evidence: 0.0,
                                side_switches: 0,
                                last_side: None,
                                gate_observations: [0; 2],
                            });
                            born_now = true;
                        }
                    }
                }
            }

            if !born_now {
                if let Some(index) = perceptual.candidates.iter()
                    .position(|w| w.base_cell == pre.cell && !w.retired)
                {
                    let snapshot = perceptual.candidates[index].clone();
                    if let Some(side) = Self::percept_side(&raw, &snapshot) {
                        for synapse in snapshot.input_synapses[side] {
                            self.learn_phase_concept_running_mean_synapse(
                                synapse, 1.0, snapshot.gate_observations[side]);
                        }
                        perceptual.candidates[index].gate_observations[side] =
                            snapshot.gate_observations[side].saturating_add(1);

                        let accepted = self.native_cell_observation(
                            &mut native, snapshot.state_cells[side],
                            action, after.cell, 0.0
                        ).is_some();
                        if !accepted { perceptual.candidates[index].retired = true; }

                        if action == snapshot.anchor_action && !snapshot.promoted {
                            if !snapshot.successor_cells.contains(&after.cell) {
                                perceptual.candidates[index].retired = true;
                            } else if accepted {
                                let counts = self.percept_counts(&native, &snapshot);
                                let w = &mut perceptual.candidates[index];
                                w.eligible_observations =
                                    w.eligible_observations.saturating_add(1);
                                if w.last_side.map(|last| last != side).unwrap_or(false) {
                                    w.side_switches = w.side_switches.saturating_add(1);
                                }
                                w.last_side = Some(side);
                                w.log_evidence = context_log_evidence(counts);
                                w.promoted = context_gate(counts, w.side_switches);
                                if !w.promoted
                                    && w.eligible_observations >= CONTEXT_MAX_SAMPLES
                                {
                                    w.retired = true;
                                }
                            }
                        }
                    }
                }
            }

            if perceptual.discovery.len() == PERCEPT_DISCOVERY_CAP {
                perceptual.discovery.remove(0);
            }
            perceptual.discovery.push(PerceptFact {
                base: pre.cell, action, post: after.cell, features: raw,
            });
        }

        let preserve_rivals = perceptual.candidates.iter()
            .any(|w| w.base_cell == pre.cell && !w.retired);
        native.perceptual = Some(perceptual);
        self.phase_native = Some(native);

        if !learning {
            self.observe_initial_real(post, false);
            return Some(0);
        }
        if preserve_rivals {
            if !self.observe_phase_native_abstract_transition(
                &pre_sensory, action, post, 0.0)
            {
                return None;
            }
            self.observe_initial_real(post, false);
            Some(0)
        } else {
            self.observe_phase_native_rival_probe_result(action, post)
        }
    }
}
