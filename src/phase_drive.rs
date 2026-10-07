// P4 learned exploration drive. Included by phase_native.rs so it operates on
// the same private EvoPhase cells/synapses as P1-P3.

#[derive(Debug, Clone)]
pub struct PhaseDriveConfig {
    pub learning_rate: f32,
    pub discount: f32,
    pub learning_enabled: bool,
    pub readout_enabled: bool,
    pub phase_learning_enabled: bool,
}

impl Default for PhaseDriveConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.35,
            discount: 0.90,
            learning_enabled: true,
            readout_enabled: true,
            phase_learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct PhaseDriveState {
    config: PhaseDriveConfig,
    feature_cells: [usize; 2],
    drive_cell: usize,
    weight_synapses: [usize; 2],
    pending_features: Option<[f32; 2]>,
    observations: u64,
}

#[derive(Debug, Clone)]
pub struct PhaseDriveCheckpoint {
    config: PhaseDriveConfig,
    learned_synapses: [PhaseSynapse; 2],
    observations: u64,
}

impl EvoPhase {
    /// P4 installs two generic epistemic feature channels into the SAME
    /// physical arrays. Both learned weights begin exactly at zero.
    pub fn enable_phase_native_learned_drive(&mut self, config: PhaseDriveConfig) -> bool {
        assert!(config.learning_rate > 0.0 && config.learning_rate <= 1.0);
        assert!(config.discount >= 0.0 && config.discount < 1.0);

        let mut state = self.phase_native.take().expect("enable native mode first");
        if state.drive.is_some() || !state.receptors.is_empty() || !state.circuits.is_empty() {
            self.phase_native = Some(state);
            return false;
        }

        let free = self
            .dormant_range()
            .filter(|index| !self.cells[*index].recruited)
            .take(3)
            .collect::<Vec<_>>();
        if free.len() != 3 {
            self.phase_native = Some(state);
            return false;
        }

        let feature_cells = [free[0], free[1]];
        let drive_cell = free[2];
        for index in [feature_cells[0], feature_cells[1], drive_cell] {
            self.cells[index].recruited = true;
        }

        let weight_synapses = [
            self.native_synapse(feature_cells[0], drive_cell),
            self.native_synapse(feature_cells[1], drive_cell),
        ];

        state.drive = Some(PhaseDriveState {
            config,
            feature_cells,
            drive_cell,
            weight_synapses,
            pending_features: None,
            observations: 0,
        });
        self.phase_native = Some(state);
        true
    }

    pub fn phase_native_drive_weights(&self) -> Option<[f32; 2]> {
        let drive = self.phase_native.as_ref()?.drive.as_ref()?;
        Some([
            self.synapses[drive.weight_synapses[0]].weight,
            self.synapses[drive.weight_synapses[1]].weight,
        ])
    }

    pub fn phase_native_drive_synapses(&self) -> Option<[usize; 2]> {
        self.phase_native
            .as_ref()?
            .drive
            .as_ref()
            .map(|drive| drive.weight_synapses)
    }

    pub fn phase_native_drive_observations(&self) -> u64 {
        self.phase_native
            .as_ref()
            .and_then(|state| state.drive.as_ref())
            .map(|drive| drive.observations)
            .unwrap_or(0)
    }

    pub fn set_phase_native_drive_learning_enabled(&mut self, enabled: bool) {
        if let Some(drive) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.drive.as_mut())
        {
            drive.config.learning_enabled = enabled;
            if !enabled {
                drive.pending_features = None;
            }
        }
    }

    pub fn set_phase_native_drive_readout_enabled(&mut self, enabled: bool) {
        if let Some(drive) = self
            .phase_native
            .as_mut()
            .and_then(|state| state.drive.as_mut())
        {
            drive.config.readout_enabled = enabled;
        }
    }

    /// Source-world transfer carries only the learned drive synapse parameters.
    pub fn phase_native_drive_checkpoint(&self) -> Option<PhaseDriveCheckpoint> {
        let drive = self.phase_native.as_ref()?.drive.as_ref()?;
        Some(PhaseDriveCheckpoint {
            config: drive.config.clone(),
            learned_synapses: [
                self.synapses[drive.weight_synapses[0]].clone(),
                self.synapses[drive.weight_synapses[1]].clone(),
            ],
            observations: drive.observations,
        })
    }

    /// Restore only the meta-drive into a cold native carrier. World receptors,
    /// transition circuits and REAL state are deliberately absent.
    pub fn restore_phase_native_drive_checkpoint(
        &mut self,
        checkpoint: PhaseDriveCheckpoint,
    ) -> bool {
        let cold = self
            .phase_native
            .as_ref()
            .map(|state| {
                state.drive.is_none()
                    && state.receptors.is_empty()
                    && state.circuits.is_empty()
            })
            .unwrap_or(false);
        if !cold || !self.enable_phase_native_learned_drive(checkpoint.config.clone()) {
            return false;
        }

        let mut state = self.phase_native.take().expect("native state");
        let drive = state.drive.as_mut().expect("drive allocated");
        for i in 0..2 {
            let index = drive.weight_synapses[i];
            let learned = &checkpoint.learned_synapses[i];
            let synapse = &mut self.synapses[index];
            synapse.weight = learned.weight;
            synapse.phase_offset = learned.phase_offset;
            synapse.eligibility = 0.0;
            synapse.confidence = learned.confidence;
            synapse.plastic = learned.plastic;
        }
        drive.observations = checkpoint.observations;
        drive.pending_features = None;
        self.phase_native = Some(state);
        true
    }

    fn phase_drive_action_known_at(
        &self,
        state: &PhaseNativeState,
        cell: usize,
        action: usize,
    ) -> bool {
        let motor = self.config.sensory_cells + action;
        let min_support = u64::from(self.config.min_recruit_support);
        state.circuits.iter().any(|circuit| {
            circuit.support >= min_support
                && self.synapses[circuit.afferent_synapse].from == cell
                && self.synapses[circuit.motor_synapse].to == motor
                && self.synapses[circuit.afferent_synapse].weight > 0.25
                && self.synapses[circuit.successor_synapse].weight > 0.25
        })
    }

    /// Generic physical frontier field over an explicit set of state cells.
    /// P4 passes receptor cells; G16 passes acquired abstract cells. The
    /// propagation rule and learned feature weights are identical.
    pub(super) fn phase_drive_frontier_activity_for_cells(
        &self,
        state: &PhaseNativeState,
        state_cells: &[usize],
    ) -> Vec<f32> {
        let motor_cells = self.config.motor_cells;
        let min_support = u64::from(self.config.min_recruit_support);
        let floor = state.config.coherence_floor;

        let mut membranes = self.cells.clone();
        for cell in &mut membranes {
            cell.charge = 0.0;
        }
        let mut intrinsic = vec![0.0_f32; membranes.len()];
        for &state_cell in state_cells {
            let unknown = (0..motor_cells)
                .filter(|action| {
                    !self.phase_drive_action_known_at(state, state_cell, *action)
                })
                .count();
            intrinsic[state_cell] = unknown as f32 / motor_cells as f32;
            membranes[state_cell].charge = intrinsic[state_cell];
        }

        for _ in 0..state.config.horizon {
            let old = membranes.clone();
            for (index, cell) in membranes.iter_mut().enumerate() {
                cell.charge = intrinsic[index];
            }
            for circuit in &state.circuits {
                if circuit.support < min_support {
                    continue;
                }
                let afferent = &self.synapses[circuit.afferent_synapse];
                let successor = &self.synapses[circuit.successor_synapse];
                if afferent.weight <= 0.25 || successor.weight <= 0.25 {
                    continue;
                }
                let propagated = state.config.discount
                    * conductance(&self.cells, afferent, floor)
                    * conductance(&self.cells, successor, floor)
                    * old[successor.to].charge;
                if propagated > membranes[afferent.from].charge {
                    membranes[afferent.from].charge = propagated;
                }
            }
        }

        membranes.iter().map(|cell| cell.charge).collect()
    }

    fn phase_drive_frontier_activity(&self, state: &PhaseNativeState) -> Vec<f32> {
        let receptor_cells = state
            .receptors
            .iter()
            .map(|receptor| receptor.cell)
            .collect::<Vec<_>>();
        self.phase_drive_frontier_activity_for_cells(state, &receptor_cells)
    }

    pub(super) fn phase_drive_features(
        &self,
        state: &PhaseNativeState,
        entry: usize,
        action: usize,
        frontier: &[f32],
    ) -> [f32; 2] {
        if !self.phase_drive_action_known_at(state, entry, action) {
            return [1.0, 0.0];
        }

        let motor = self.config.sensory_cells + action;
        let floor = state.config.coherence_floor;
        let min_support = u64::from(self.config.min_recruit_support);
        let reachable = state
            .circuits
            .iter()
            .filter(|circuit| {
                circuit.support >= min_support
                    && self.synapses[circuit.afferent_synapse].from == entry
                    && self.synapses[circuit.motor_synapse].to == motor
            })
            .map(|circuit| {
                let afferent = &self.synapses[circuit.afferent_synapse];
                let successor = &self.synapses[circuit.successor_synapse];
                conductance(&self.cells, afferent, floor)
                    * conductance(&self.cells, successor, floor)
                    * frontier[successor.to]
            })
            .fold(0.0_f32, f32::max);

        [0.0, reachable.clamp(0.0, 1.0)]
    }

    pub(super) fn phase_drive_score(
        &self,
        state: &PhaseNativeState,
        features: [f32; 2],
    ) -> Option<f32> {
        let drive = state.drive.as_ref()?;
        if !drive.config.readout_enabled {
            return None;
        }
        let floor = state.config.coherence_floor;
        let mut score = 0.0_f32;
        for i in 0..2 {
            score += features[i]
                * conductance(&self.cells, &self.synapses[drive.weight_synapses[i]], floor);
        }
        Some(score.clamp(0.0, 2.0))
    }

    pub(super) fn phase_native_stage_drive_action(&mut self, action: usize) {
        let Some(sensory) = self.current_real.as_ref().map(|real| real.sensory.clone()) else {
            return;
        };
        let Some(trace) = self.encode_high_level_trace(&sensory) else {
            return;
        };

        let mut state = match self.phase_native.take() {
            Some(state) => state,
            None => return,
        };
        if state.drive.is_none() {
            self.phase_native = Some(state);
            return;
        }
        let Some(entry) = state
            .receptors
            .iter()
            .find(|receptor| {
                receptor.trace.similarity(&trace) >= state.config.match_threshold
            })
            .map(|receptor| receptor.cell)
        else {
            self.phase_native = Some(state);
            return;
        };

        let frontier = self.phase_drive_frontier_activity(&state);
        let features = self.phase_drive_features(&state, entry, action, &frontier);
        state.drive.as_mut().expect("drive").pending_features = Some(features);
        self.phase_native = Some(state);
    }

    /// Target-world P4 selector. There is no call to the P3 fixed selector.
    pub fn choose_phase_native_learned_drive_action(&mut self) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let trace = self.encode_high_level_trace(&sensory)?;
        let mut state = self.phase_native.take()?;

        if state
            .drive
            .as_ref()
            .map(|drive| drive.config.readout_enabled)
            != Some(true)
        {
            self.phase_native = Some(state);
            return None;
        }

        let entry = match self.native_receptor(&mut state, trace) {
            Some(cell) => cell,
            None => {
                self.phase_native = Some(state);
                return None;
            }
        };
        let frontier = self.phase_drive_frontier_activity(&state);

        let mut best: Option<(usize, f32, [f32; 2])> = None;
        for action in 0..self.config.motor_cells {
            let features = self.phase_drive_features(&state, entry, action, &frontier);
            let score = self.phase_drive_score(&state, features)?;
            match best {
                None => best = Some((action, score, features)),
                Some((best_action, best_score, _)) => {
                    if score > best_score + 1.0e-6
                        || ((score - best_score).abs() <= 1.0e-6
                            && action < best_action)
                    {
                        best = Some((action, score, features));
                    }
                }
            }
        }

        let (action, _, features) = best?;
        state.drive.as_mut().expect("drive").pending_features = Some(features);
        self.phase_native = Some(state);
        Some(action)
    }

    pub(super) fn phase_native_drive_after_cell_fact(
        &mut self,
        post_cell: usize,
        state_cells: &[usize],
        structural_gain: bool,
    ) {
        let mut state = match self.phase_native.take() {
            Some(state) => state,
            None => return,
        };

        let Some(drive) = state.drive.as_ref() else {
            self.phase_native = Some(state);
            return;
        };
        let Some(features) = drive.pending_features else {
            self.phase_native = Some(state);
            return;
        };
        let config = drive.config.clone();
        let weight_synapses = drive.weight_synapses;
        state.drive.as_mut().expect("drive").pending_features = None;

        if !config.learning_enabled {
            self.phase_native = Some(state);
            return;
        }

        let predicted = self.phase_drive_score(&state, features).unwrap_or(0.0);
        let next_value = if state_cells.contains(&post_cell) {
            let frontier =
                self.phase_drive_frontier_activity_for_cells(&state, state_cells);
            (0..self.config.motor_cells)
                .filter_map(|action| {
                    let next_features =
                        self.phase_drive_features(&state, post_cell, action, &frontier);
                    self.phase_drive_score(&state, next_features)
                })
                .fold(0.0_f32, f32::max)
        } else {
            0.0
        };

        let target = ((if structural_gain { 1.0 } else { 0.0 })
            + config.discount * next_value)
            .clamp(0.0, 1.0);
        let error = target - predicted;

        for i in 0..2 {
            if features[i] <= 0.0 {
                continue;
            }
            let synapse = &mut self.synapses[weight_synapses[i]];
            if !synapse.plastic {
                continue;
            }
            synapse.eligibility = features[i];
            synapse.weight = (synapse.weight
                + config.learning_rate * error * features[i])
                .clamp(0.0, 1.0);
            if config.phase_learning_enabled {
                let target_phase =
                    wrap_phase(self.cells[synapse.to].phase - self.cells[synapse.from].phase);
                synapse.phase_offset = wrap_phase(
                    synapse.phase_offset
                        + self.config.phase_learning_rate
                            * features[i]
                            * signed_phase_error(target_phase, synapse.phase_offset),
                );
            }
            synapse.confidence = (synapse.confidence
                + config.learning_rate * (1.0 - synapse.confidence))
                .clamp(0.0, 1.0);
        }

        let observations = state.drive.as_ref().expect("drive").observations;
        state.drive.as_mut().expect("drive").observations = observations.saturating_add(1);
        self.phase_native = Some(state);
    }

    /// Local TD update from internal structural information gain after a real
    /// P2 transition. Hidden world labels and evaluator route data are absent.
    pub(super) fn phase_native_drive_after_fact(
        &mut self,
        post_sensory: &[f32],
        structural_gain: bool,
    ) {
        let Some(post_trace) = self.encode_high_level_trace(post_sensory) else {
            return;
        };
        let Some(state) = self.phase_native.as_ref() else {
            return;
        };
        let receptor_cells = state
            .receptors
            .iter()
            .map(|receptor| receptor.cell)
            .collect::<Vec<_>>();
        let Some(post_cell) = state
            .receptors
            .iter()
            .find(|receptor| {
                receptor.trace.similarity(&post_trace) >= state.config.match_threshold
            })
            .map(|receptor| receptor.cell)
        else {
            // Still clear any staged feature through the generic path.
            self.phase_native_drive_after_cell_fact(
                usize::MAX,
                &receptor_cells,
                structural_gain,
            );
            return;
        };
        self.phase_native_drive_after_cell_fact(
            post_cell,
            &receptor_cells,
            structural_gain,
        );
    }
}
