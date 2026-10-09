// Online sensory acquisition reuses the P1/P3 physical transition substrate.
// Recognition is inherited sensor-distance matching; it is not a claim of
// learned invariance. No route, goal outcome or motor role enters acquisition.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhaseOnlineConfig {
    /// Bound acquired observations independently of physical relay capacity.
    pub max_states: usize,
}

impl Default for PhaseOnlineConfig {
    fn default() -> Self {
        Self { max_states: 32 }
    }
}

impl EvoPhase {
    /// Select a cold sensor vocabulary before any target observations. Existing
    /// qualified abstract representations cannot be silently bypassed by it.
    pub fn enable_phase_native_online_learning(&mut self, config: PhaseOnlineConfig) -> bool {
        if config.max_states == 0
            || config.max_states > self.config.dormant_cells
            || self.concept_memory.is_some()
            || self.raster_field.is_some()
        {
            return false;
        }
        let Some(state) = self.phase_native.as_mut() else {
            return false;
        };
        if state.online.is_some()
            || !state.receptors.is_empty()
            || !state.circuits.is_empty()
            || state.concepts.is_some()
            || state.meta_control.is_some()
            || state.temporal_evidence.is_some()
        {
            return false;
        }
        state.online = Some(config);
        true
    }

    pub fn phase_native_online_enabled(&self) -> bool {
        self.phase_native
            .as_ref()
            .is_some_and(|s| s.online.is_some())
    }

    pub(super) fn online_sensory_trace(&self, sensory: &[f32]) -> Option<CarrierTrace> {
        if sensory.len() != self.config.sensory_cells
            || sensory.is_empty()
            || sensory
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        {
            return None;
        }
        Some(CarrierTrace::Sensory(sensory.to_vec()))
    }

    pub fn phase_native_online_state(&self, sensory: &[f32]) -> Option<PhaseAbstractStateRef> {
        let state = self.phase_native.as_ref()?;
        state.online.as_ref()?;
        let trace = self.online_sensory_trace(sensory)?;
        let receptor = state
            .receptors
            .iter()
            .find(|r| r.trace.similarity(&trace) >= state.config.match_threshold)?;
        Some(PhaseAbstractStateRef {
            level: 0,
            id: receptor.cell as u64,
            cell: receptor.cell,
        })
    }

    /// May only be called at a validated external observation boundary. Goals
    /// and imagined predictions never allocate receptors or teach transitions.
    pub fn acquire_phase_native_online_observation(&mut self, sensory: &[f32]) -> bool {
        if !self.phase_native_online_enabled() {
            return false;
        }
        if self.phase_native_online_state(sensory).is_some() {
            return true;
        }
        let Some(trace) = self.online_sensory_trace(sensory) else {
            return false;
        };
        let Some(mut state) = self.phase_native.take() else {
            return false;
        };
        let acquired =
            state.config.learning_enabled && self.native_receptor(&mut state, trace).is_some();
        self.phase_native = Some(state);
        acquired
    }

    fn phase_native_online_proposal(&self, goal: &[f32]) -> Option<PhaseUnifiedCognitiveProposal> {
        if self.phase_rules_enabled() {
            let decision = if self.phase_adaptive_rules_enabled() {
                self.phase_adaptive_decision(goal)?
            } else {
                self.phase_rule_decision(goal)?
            };
            let planning = decision.kind == PhaseRuleDecisionKind::GoalPlan;
            return Some(PhaseUnifiedCognitiveProposal {
                persistent_candidate_id: None,
                applicability: 1.0,
                proposal: PhaseCognitiveProposal {
                    proposal_id: unified_hash(&[
                        0x0A12,
                        decision.action as u64,
                        self.phase_native_learned_fingerprint(),
                    ]),
                    action: decision.action,
                    fields: [
                        if planning { 1.0 } else { 0.0 },
                        if planning { 0.0 } else { 1.0 },
                        decision.expected_disagreement,
                        if planning { 1.0 } else { 0.0 },
                        1.0,
                    ],
                },
            });
        }
        let sensory = &self.current_real.as_ref()?.sensory;
        let entry = self.phase_native_online_state(sensory)?;
        self.online_sensory_trace(goal)?;
        let mut imagined = self.clone();
        let decision = self.phase_native_online_state(goal).and_then(|target| {
            imagined.phase_native_goal_decision_from_cells(entry.cell, target.cell, None)
        });
        let (action, value, novelty) = if let Some(plan) = decision {
            (plan.first_action, plan.predicted_value, 0.0)
        } else {
            // P3 explores unknown local actions and propagates frontier value
            // through learned synapses. It receives no task-specific teacher.
            (imagined.choose_phase_native_autonomous_action()?, 0.0, 1.0)
        };
        Some(PhaseUnifiedCognitiveProposal {
            persistent_candidate_id: None,
            applicability: 1.0,
            proposal: PhaseCognitiveProposal {
                proposal_id: unified_hash(&[0x0A11, entry.cell as u64, action as u64]),
                action,
                fields: [value, novelty, 0.0, 1.0, 1.0],
            },
        })
    }
}
