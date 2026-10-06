// Included by phase_native.rs: access is to the SAME private carrier arrays.
// P2 uses synchronous phasor propagation, not a biological oscillator claim.
// Decoder weights learn sensory patterns in their tuition coordinate frame.
// Recognition is inherited; no future receptor prototype is copied to output.

impl EvoPhase {
    /// Addresses only. Pixel values remain in actual shared synapse weights.
    pub fn phase_native_decoder_synapses(&self) -> Vec<usize> {
        let Some(state) = self.phase_native.as_ref() else { return Vec::new(); };
        self.synapses.iter().enumerate().filter_map(|(i, syn)| {
            (syn.to < self.config.sensory_cells
                && state.receptors.iter().any(|r| r.cell == syn.from)).then_some(i)
        }).collect()
    }

    fn is_native_decoder_synapse(&self, index: usize) -> bool {
        let Some(state) = self.phase_native.as_ref() else { return false; };
        self.synapses.get(index).map(|syn| {
            syn.to < self.config.sensory_cells
                && state.receptors.iter().any(|r| r.cell == syn.from)
        }).unwrap_or(false)
    }

    fn learn_native_sensory_decoder(&mut self, receptor: usize, sensory: &[f32]) {
        for (channel, target) in sensory.iter().copied().enumerate() {
            let index = match self.synapses.iter().position(|syn| {
                syn.from == receptor && syn.to == channel
            }) {
                Some(index) => index,
                None => self.native_synapse(receptor, channel),
            };
            let syn = &mut self.synapses[index];
            if !syn.plastic { continue; }
            syn.eligibility = 1.0;
            syn.weight += self.config.weight_learning_rate * (target - syn.weight);
            let relative = wrap_phase(self.cells[channel].phase - self.cells[receptor].phase);
            syn.phase_offset = wrap_phase(syn.phase_offset
                + self.config.phase_learning_rate * signed_phase_error(relative, syn.phase_offset));
            syn.confidence += self.config.weight_learning_rate * (1.0 - syn.confidence);
        }
    }

    /// Factual API for P2. It measures the pre-update prediction error first.
    /// Tuples must come from actual interaction, never from imagined output.
    /// In P2's fully observed deterministic scope, conflicting successors
    /// suppress contradicted afferents locally; old addresses/evidence remain.
    pub fn observe_phase_native_forward_transition(
        &mut self, pre: &[f32], action: usize, post: &[f32], value: f32,
    ) -> Option<f32> {
        self.assert_sensory(pre);
        self.assert_sensory(post);
        assert!(action < self.config.motor_cells);
        assert!(value.is_finite() && (0.0..=1.0).contains(&value));
        let pre_trace = self.encode_high_level_trace(pre)?;
        let post_trace = self.encode_high_level_trace(post)?;
        let prior = self.imagine_phase_native_actions(pre, &[action]);
        let error = prior.first()
            .and_then(|p| self.encode_high_level_trace(&p.sensory))
            .map(|p| (1.0 - p.similarity(&post_trace)).clamp(0.0, 2.0))
            .unwrap_or(1.0);
        if !self.phase_native.as_ref()?.config.learning_enabled { return Some(error); }
        self.observe_planning_transition(pre, action, post, value);
        let mut state = self.phase_native.take()?;
        let from = state.receptors.iter().find(|r| {
            r.trace.similarity(&pre_trace) >= state.config.match_threshold
        }).map(|r| r.cell);
        let to = state.receptors.iter().find(|r| {
            r.trace.similarity(&post_trace) >= state.config.match_threshold
        }).map(|r| r.cell);
        if let (Some(from), Some(to)) = (from, to) {
            let motor = self.motor_cell(action);
            // Suppress only after the factual successor circuit exists.
            let acquired = state.circuits.iter().any(|c| {
                self.synapses[c.afferent_synapse].from == from
                    && self.synapses[c.motor_synapse].to == motor
                    && self.synapses[c.successor_synapse].to == to
            });
            if acquired {
                for c in &mut state.circuits {
                    if self.synapses[c.afferent_synapse].from == from
                        && self.synapses[c.motor_synapse].to == motor
                        && self.synapses[c.successor_synapse].to != to {
                        let syn = &mut self.synapses[c.afferent_synapse];
                        if syn.plastic {
                            syn.eligibility = 1.0;
                            syn.weight += self.config.weight_learning_rate * (0.0 - syn.weight);
                            c.revision = c.revision.saturating_add(1);
                            c.counterexamples.push((c.support, value));
                        }
                    }
                }
                self.learn_native_sensory_decoder(from, pre);
                self.learn_native_sensory_decoder(to, post);
            }
        }
        self.phase_native = Some(state);
        Some(error)
    }

    /// Forward continuation receives physical action tokens, NOT intermediate
    /// sensory states. Only the initial observation uses prototype recognition.
    /// Subsequent state is carried by mode-isolated membrane activity.
    pub fn imagine_phase_native_actions(
        &self, pre: &[f32], actions: &[usize],
    ) -> Vec<super::Prediction> {
        let Some(state) = self.phase_native.as_ref() else { return Vec::new(); };
        if actions.len() > state.config.horizon || actions.iter().any(|a| *a >= self.config.motor_cells) {
            return Vec::new();
        }
        let Some(trace) = self.encode_high_level_trace(pre) else { return Vec::new(); };
        let Some(entry) = state.receptors.iter().find(|r| {
            r.trace.similarity(&trace) >= state.config.match_threshold
        }).map(|r| r.cell) else { return Vec::new(); };
        let decoder = self.phase_native_decoder_synapses();
        if decoder.is_empty() { return Vec::new(); }
        let mut activity = self.cells.clone();
        for c in &mut activity { c.charge = 0.0; }
        activity[entry].charge = 1.0;
        let mut outputs = Vec::new();
        for &action in actions {
            let Some((next, prediction)) = native_forward_tick(
                &self.cells, &self.synapses, &state.circuits, &decoder,
                &activity, self.motor_cell(action), self.config.sensory_cells,
                state.config.coherence_floor, u64::from(self.config.min_recruit_support),
            ) else { break; };
            // Decoder failure is abstention, never a stored-template fallback.
            if self.encode_high_level_trace(&prediction.sensory).is_none() { break; }
            activity = next;
            outputs.push(prediction);
        }
        outputs
    }

    /// Ordinary bounded action loop: P1 selects; P2 predicts BEFORE execution.
    /// No legacy graph backend or unpredicted fallback action is permitted.
    pub fn choose_phase_native_action_with_prediction(
        &mut self,
    ) -> Option<(PlanDecision, super::Prediction)> {
        self.phase_native.as_ref()?;
        let pre = self.current_real.as_ref()?.sensory.clone();
        let decision = self.plan_imagined(&pre)?;
        let prediction = self.imagine_phase_native_actions(&pre, &[decision.first_action])
            .into_iter().next()?;
        Some((decision, prediction))
    }

    /// Commit external factual POST and return the PRE-update sensory error.
    /// Learning is controlled by the same native freeze flag as ordinary P1.
    pub fn observe_phase_native_action_result(
        &mut self, action: usize, post: &[f32], value: f32,
    ) -> Option<f32> {
        let pre = self.current_real.as_ref()?.sensory.clone();
        let error = self.observe_phase_native_forward_transition(&pre, action, post, value)?;
        self.observe_initial_real(post, value >= 1.0);
        Some(error)
    }
}

// One inherited action-gated physical propagation step. No receptor trace,
// state label, route, expected output or evaluator callback is available here.
fn native_forward_tick(
    base: &[PhaseCell], synapses: &[PhaseSynapse], circuits: &[PhaseCircuitInfo],
    decoder: &[usize], active: &[PhaseCell], motor: usize, sensory_cells: usize,
    floor: f32, min_support: u64,
) -> Option<(Vec<PhaseCell>, super::Prediction)> {
    let mut relay = base.to_vec();
    for c in &mut relay { c.charge = 0.0; }
    for c in circuits {
        if c.support < min_support { continue; }
        let gate = &synapses[c.motor_synapse];
        if gate.to != motor { continue; }
        let afferent = &synapses[c.afferent_synapse];
        let (amplitude, phase) = native_forward_signal(base, active, afferent, floor);
        relay[c.relay_cell].charge = amplitude * conductance(base, gate, floor);
        relay[c.relay_cell].phase = phase;
    }
    let mut re = vec![0.0_f32; base.len()];
    let mut im = vec![0.0_f32; base.len()];
    let mut outcome = 0.0;
    let mut outcome_mass = 0.0;
    for c in circuits {
        let syn = &synapses[c.successor_synapse];
        let (amplitude, phase) = native_forward_signal(base, &relay, syn, floor);
        re[syn.to] += amplitude * phase.cos();
        im[syn.to] += amplitude * phase.sin();
        let reward = &synapses[c.outcome_synapse];
        let (reward_current, _) = native_forward_signal(base, &relay, reward, floor);
        outcome += reward_current;
        outcome_mass += relay[c.relay_cell].charge;
    }
    let mut next = base.to_vec();
    let mut peak = 0.0_f32;
    let mut mass = 0.0_f32;
    for i in 0..next.len() {
        let amplitude = re[i].hypot(im[i]);
        next[i].charge = amplitude;
        if amplitude > 1.0e-8 { next[i].phase = wrap_phase(im[i].atan2(re[i])); }
        peak = peak.max(amplitude);
        mass += amplitude;
    }
    // Fully observed deterministic scope: competing unresolved successors
    // must abstain, rather than silently presenting a mixture as a fact.
    if peak < 0.25 || peak / mass.max(1.0e-8) < 0.95 { return None; }
    let mut pixels_re = vec![0.0_f32; sensory_cells];
    let mut pixels_im = vec![0.0_f32; sensory_cells];
    for &index in decoder {
        let syn = &synapses[index];
        let (amplitude, phase) = native_forward_signal(base, &next, syn, floor);
        pixels_re[syn.to] += amplitude * phase.cos();
        pixels_im[syn.to] += amplitude * phase.sin();
    }
    let sensory: Vec<f32> = pixels_re.iter().zip(&pixels_im)
        .map(|(x, y)| (x.hypot(*y) / peak).clamp(0.0, 1.0)).collect();
    Some((next, super::Prediction {
        sensory,
        need: (outcome / outcome_mass.max(1.0e-8)).clamp(0.0, 1.0),
        confidence: (peak / mass.max(1.0e-8)).clamp(0.0, 1.0),
        authority: Authority::Imagined,
    }))
}

fn native_forward_signal(
    base: &[PhaseCell], active: &[PhaseCell], syn: &PhaseSynapse, floor: f32,
) -> (f32, f32) {
    let arrived = wrap_phase(active[syn.from].phase + syn.phase_offset);
    if !base[syn.from].recruited || !base[syn.to].recruited {
        return (0.0, arrived);
    }
    let agreement = signed_phase_error(base[syn.to].phase, arrived).cos();
    let gate = ((agreement - floor) / (1.0 - floor)).clamp(0.0, 1.0);
    (active[syn.from].charge * syn.weight.clamp(0.0, 1.0) * gate, arrived)
}
