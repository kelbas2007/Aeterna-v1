//! Persistent orchestration of acquired native reasoning, not a second planner.
//!
//! The host supplies raw goals, hazard evidence and actual observations. Native
//! EvoPhase selectors own action choice. The private protection gate owns the
//! single-use permission to invoke the external action callback. This wrapper
//! has no world labels, route table or task-conditioned learning rules.

use crate::carrier::{
    EvoPhase, PhaseNativeCheckpoint, PhaseCognitiveProposal,
    PhaseMetaControlCheckpoint, PhaseMetaDecision,
    PhaseHypothesisEcologyConfig, PhaseUnifiedDecision, PhaseUnifiedCognitiveProposal,
    PhaseTemporalEvidenceConfig,
};
use crate::human_protection::{
    HumanProtection, HumanProtectionEvidence, HumanProtectionReason,
    HumanProtectionRecord,
};
use std::collections::VecDeque;
use std::fmt;

const AUDIT_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningMode {
    RivalDiscrimination,
    GoalDirectedAction,
    ContextualRefinement,
    PerceptualRefinement,
    CompositionalRefinement,
    GeneralEpistemic,
    UnifiedCompetition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProposal {
    pub action: usize,
    pub mode: ReasoningMode,
    pub goal_epoch: u64,
    pub learned_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    NativeModelRequired,
    InvalidRaster,
    UnknownRepresentation,
    GoalRequired,
    FreshObservationRequired,
    NoSupportedAction,
    InvalidCheckpoint,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone)]
pub enum LifetimeEventKind {
    ExternalObservation,
    GoalChanged,
    Proposal(ActionProposal),
    Blocked(HumanProtectionRecord),
    Executed { action: usize, suppressed: usize, learned: bool },
    ExecutionFault(String),
    CognitiveRestart,
    OperatorReset,
}

#[derive(Debug, Clone)]
pub struct LifetimeEvent {
    pub sequence: u64,
    pub goal_epoch: u64,
    pub kind: LifetimeEventKind,
}

#[derive(Debug, Clone)]
pub enum StepOutcome {
    GoalReached,
    Blocked(HumanProtectionRecord),
    Executed { proposal: ActionProposal, suppressed: usize, learned: bool },
    ExecutionFault(String),
}

/// This host boundary is deliberately not Clone. A native checkpoint contains
/// cognition only and cannot duplicate or roll back this boundary's authority.
#[derive(Debug)]
pub struct ScientificRuntime {
    organism: EvoPhase,
    protection: HumanProtection,
    goal: Option<Vec<f32>>,
    goal_epoch: u64,
    sequence: u64,
    audit: VecDeque<LifetimeEvent>,
    fresh_observation_required: bool,
    model_learning_enabled: bool,
}

impl ScientificRuntime {
    pub fn new(organism: EvoPhase) -> Result<Self, RuntimeError> {
        if organism.phase_native_checkpoint().is_none() {
            return Err(RuntimeError::NativeModelRequired);
        }
        Ok(Self {
            organism,
            protection: HumanProtection::new(),
            goal: None,
            goal_epoch: 0,
            sequence: 0,
            audit: VecDeque::new(),
            fresh_observation_required: true,
            model_learning_enabled: true,
        })
    }

    /// Read-only diagnostics. No mutable carrier/protection handle is exposed.
    pub fn organism(&self) -> &EvoPhase { &self.organism }
    pub fn audit(&self) -> &VecDeque<LifetimeEvent> { &self.audit }
    pub fn sequence(&self) -> u64 { self.sequence }
    pub fn emergency_latched(&self) -> bool {
        self.protection.emergency_stop_latched()
    }

    /// Opt-in unified-cognition transport. This wrapper does not rank proposal
    /// sources; the native physical meta-control circuit sees only opaque
    /// proposal IDs/actions and numeric carrier-derived fields.
    pub fn restore_unified_meta_control(
        &mut self,
        checkpoint: PhaseMetaControlCheckpoint,
    ) -> bool {
        self.organism.restore_phase_native_meta_checkpoint(checkpoint)
    }

    pub fn select_unified_proposal(
        &self,
        proposals: &[PhaseCognitiveProposal],
    ) -> Option<PhaseMetaDecision> {
        self.organism.choose_phase_native_meta_proposal(proposals)
    }

    /// INTEL-2 assembly: restore the learned U1 meta-control and attach a cold
    /// U2 hypothesis ecology. Meta weights are frozen during the target
    /// lifetime; hypothesis authority remains plastic from factual usefulness.
    pub fn enable_unified_cognition(
        &mut self,
        checkpoint: PhaseMetaControlCheckpoint,
        ecology: PhaseHypothesisEcologyConfig,
    ) -> bool {
        if !self.organism.restore_phase_native_meta_checkpoint(checkpoint) {
            return false;
        }
        if !self.organism.enable_phase_native_hypothesis_ecology(ecology) {
            return false;
        }
        self.organism.set_phase_native_meta_learning_enabled(false);
        true
    }

    /// TE3 optional native temporal-evidence project. Source/goal/answer
    /// identities stay entirely inside EvoPhase and are never supplied here.
    pub fn enable_temporal_evidence(
        &mut self, config: PhaseTemporalEvidenceConfig
    )->bool{
        self.organism.enable_phase_native_temporal_evidence(config)
    }

    fn select_unified_internal(
        &mut self,
    ) -> Result<Option<(ActionProposal, PhaseUnifiedDecision, Vec<PhaseUnifiedCognitiveProposal>)>, RuntimeError> {
        if self.goal_reached()? { return Ok(None); }
        let goal = self.goal.as_ref().ok_or(RuntimeError::GoalRequired)?.clone();
        let proposals = self.organism.collect_phase_native_unified_proposals(&goal);
        let decision = self.organism.choose_phase_native_unified_proposal(&proposals)
            .ok_or(RuntimeError::NoSupportedAction)?;
        let proposal = ActionProposal {
            action: decision.action,
            mode: ReasoningMode::UnifiedCompetition,
            goal_epoch: self.goal_epoch,
            learned_fingerprint: self.organism.phase_native_learned_fingerprint(),
        };
        self.record(LifetimeEventKind::Proposal(proposal.clone()));
        Ok(Some((proposal,decision,proposals)))
    }

    pub fn propose_unified(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {
        Ok(self.select_unified_internal()?.map(|(proposal,_,_)|proposal))
    }

    /// Enable the native residual-driven representation learner. No context,
    /// split location, outcome mapping or threshold is accepted from the host.
    pub fn enable_context_refinement(&mut self) -> bool {
        self.organism.enable_phase_native_context_refinement()
    }

    /// Enable current-sensory evidence-gated feature refinement. The host
    /// supplies no feature identity, split location or answer mapping.
    pub fn enable_perceptual_refinement(&mut self) -> bool {
        self.organism.enable_phase_native_perceptual_refinement()
    }

    /// Enable bounded compositional perceptual function synthesis. The host
    /// supplies no operator, descriptor identity or answer mapping.
    pub fn enable_compositional_refinement(&mut self) -> bool {
        self.organism.enable_phase_native_compositional_refinement()
    }

    fn record(&mut self, kind: LifetimeEventKind) {
        self.sequence = self.sequence.checked_add(1)
            .expect("lifetime audit sequence exhausted");
        if self.audit.len() == AUDIT_CAPACITY { self.audit.pop_front(); }
        self.audit.push_back(LifetimeEvent {
            sequence: self.sequence, goal_epoch: self.goal_epoch, kind,
        });
    }

    fn validate_raster(&self, raster: &[f32]) -> Result<(), RuntimeError> {
        if raster.len() != self.organism.config().sensory_cells
            || raster.iter().any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        {
            return Err(RuntimeError::InvalidRaster);
        }
        if self.organism.phase_native_abstract_state(raster).is_none() {
            return Err(RuntimeError::UnknownRepresentation);
        }
        Ok(())
    }

    /// Initial sensing or fresh sensing after restart/fault. The observation
    /// must come from outside cognition; this method performs no model tuition.
    pub fn observe_external(&mut self, raster: &[f32]) -> Result<(), RuntimeError> {
        self.validate_raster(raster)?;
        self.organism.clear_phase_native_context_history();
        // A new external real observation begins a fresh evidence episode,
        // while preserving acquired cue and motor synapses across the lifetime.
        if self.organism.phase_native_temporal_evidence_enabled() {
            let _=self.organism.begin_phase_native_temporal_episode();
            let _=self.organism.observe_phase_native_temporal_signal(raster);
        }
        self.organism.observe_initial_real(raster, false);
        self.fresh_observation_required = false;
        self.record(LifetimeEventKind::ExternalObservation);
        Ok(())
    }

    pub fn set_goal(&mut self, raster: &[f32]) -> Result<(), RuntimeError> {
        self.validate_raster(raster)?;
        self.goal_epoch = self.goal_epoch.checked_add(1)
            .expect("goal epoch exhausted");
        self.goal = Some(raster.to_vec());
        self.record(LifetimeEventKind::GoalChanged);
        Ok(())
    }

    /// Host-controlled freeze for evaluation or read-only operation. No world
    /// identity or desired outcome is accepted by this switch.
    pub fn set_model_learning_enabled(&mut self, enabled: bool) {
        self.model_learning_enabled = enabled;
        self.organism.set_planning_learning_enabled(enabled);
    }

    pub fn goal_reached(&self) -> Result<bool, RuntimeError> {
        if self.fresh_observation_required {
            return Err(RuntimeError::FreshObservationRequired);
        }
        let goal = self.goal.as_ref().ok_or(RuntimeError::GoalRequired)?;
        let real = self.organism.current_real()
            .ok_or(RuntimeError::FreshObservationRequired)?;
        let current = self.organism.phase_native_abstract_state(&real.sensory)
            .ok_or(RuntimeError::UnknownRepresentation)?;
        let target = self.organism.phase_native_abstract_state(goal)
            .ok_or(RuntimeError::UnknownRepresentation)?;
        Ok(current.level == target.level && current.cell == target.cell)
    }

    /// Fixed orchestration of existing native mechanisms. Task-level scoring
    /// remains in EvoPhase; this function does not construct or search a model.
    pub fn propose(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {
        if self.goal_reached()? { return Ok(None); }
        let goal = self.goal.as_ref().ok_or(RuntimeError::GoalRequired)?.clone();
        let compositional = self.organism.phase_native_compositional_action(&goal);
        let perceptual = self.organism.phase_native_perceptual_action(&goal);
        let contextual = self.organism.phase_native_context_action(&goal);
        let (action, mode) = if compositional.0 {
            (compositional.1.ok_or(RuntimeError::NoSupportedAction)?,
                ReasoningMode::CompositionalRefinement)
        } else if perceptual.0 {
            // A learned current-sensory distinction must not be bypassed using
            // the inherited ambiguous parent representation.
            (perceptual.1.ok_or(RuntimeError::NoSupportedAction)?,
                ReasoningMode::PerceptualRefinement)
        } else if contextual.0 {
            // A missing/damaged required context must not be bypassed using
            // the old ambiguous parent model.
            (contextual.1.ok_or(RuntimeError::NoSupportedAction)?,
                ReasoningMode::ContextualRefinement)
        } else if let Some(action) =
            self.organism.choose_phase_native_goal_rival_probe(&goal)
        {
            (action, ReasoningMode::RivalDiscrimination)
        } else if let Some(action) =
            self.organism.choose_phase_native_goal_active_action(&goal)
        {
            (action, ReasoningMode::GoalDirectedAction)
        } else {
            // INTEL-1 Repair-1: a goal that is not yet connected to the known
            // physical model cannot create a goal-relevance gradient. Bootstrap
            // only with the already-qualified G16 general epistemic drive.
            let action = self.organism
                .choose_phase_native_abstract_learned_drive_action()
                .ok_or(RuntimeError::NoSupportedAction)?;
            (action, ReasoningMode::GeneralEpistemic)
        };
        let proposal = ActionProposal {
            action, mode, goal_epoch: self.goal_epoch,
            learned_fingerprint: self.organism.phase_native_learned_fingerprint(),
        };
        self.record(LifetimeEventKind::Proposal(proposal.clone()));
        Ok(Some(proposal))
    }

    fn absent_evidence() -> HumanProtectionEvidence {
        HumanProtectionEvidence {
            human_present: true, physical_effect_possible: true,
            predicted_harm_probability: f32::NAN,
            hazard_confidence: 0.0, emergency_stop: false,
        }
    }

    fn latch_fault(&mut self, message: String) -> StepOutcome {
        let mut evidence = Self::absent_evidence();
        evidence.emergency_stop = true;
        self.protection.screen(0, evidence);
        self.fresh_observation_required = true;
        self.record(LifetimeEventKind::ExecutionFault(message.clone()));
        StepOutcome::ExecutionFault(message)
    }

    /// Only this method invokes the action callback. Hazard assessment and the
    /// callback are supplied by the trusted execution adapter, not cognition.
    /// A missing assessment is fail-closed; the permit never leaves this gate.
    pub fn step<Assess, Execute>(
        &mut self,
        assess: Assess,
        execute: Execute,
    ) -> Result<StepOutcome, RuntimeError>
    where
        Assess: FnOnce(&ActionProposal) -> Option<HumanProtectionEvidence>,
        Execute: FnOnce(usize) -> Result<Vec<f32>, String>,
    {
        if self.protection.emergency_stop_latched() {
            let decision = self.protection.screen(0, Self::absent_evidence());
            let record = decision.record().clone();
            self.record(LifetimeEventKind::Blocked(record.clone()));
            return Ok(StepOutcome::Blocked(record));
        }
        let Some(proposal) = self.propose()? else {
            return Ok(StepOutcome::GoalReached);
        };
        let evidence = assess(&proposal).unwrap_or_else(Self::absent_evidence);
        let decision = self.protection.screen(proposal.action, evidence);
        let screening = decision.record().clone();
        let Some(permit) = decision.into_permit() else {
            self.record(LifetimeEventKind::Blocked(screening.clone()));
            return Ok(StepOutcome::Blocked(screening));
        };
        let action = match self.protection.consume_permit(permit) {
            Ok(action) => action,
            Err(error) => {
                return Ok(self.latch_fault(format!("permit consumption: {:?}", error)));
            }
        };
        // The action has now been authorized and its permission consumed.
        let post = match execute(action) {
            Ok(post) => post,
            Err(error) => return Ok(self.latch_fault(error)),
        };
        if let Err(error) = self.validate_raster(&post) {
            return Ok(self.latch_fault(format!("unusable factual POST: {}", error)));
        }

        // Commit sensory affordance only after one permitted and executed
        // motor returned a validated FACTUAL POST. Non-sensing no-op actions
        // do NOT count another copy of the same cue as new information.
        if self.organism.phase_native_temporal_evidence_enabled() {
            if self.model_learning_enabled {
                let _=self.organism.observe_phase_native_sensing_affordance(
                    action,&factual_pre,&post
                );
            }
            if self.organism.phase_native_temporal_source_count()==2
                && self.organism.phase_native_temporal_action_affordance(action)
                    .unwrap_or(0.0)>1.0e-8
            {
                let _=self.organism.observe_phase_native_temporal_signal(&post);
            }
        }

        let any_refinement =
            self.organism.phase_native_compositional_enabled()
                || self.organism.phase_native_perceptual_enabled()
                || self.organism.phase_native_context_enabled();
        let suppressed = if any_refinement {
            // Repair-3: one factual action/POST is inspected by every enabled
            // representation learner, while the shared parent transition and
            // REAL frame are committed exactly once.
            match self.organism.observe_phase_native_refinement_fanout_result(action, &post) {
                Some(count) => count,
                None => return Ok(self.latch_fault("refinement factual fanout rejected".into())),
            }
        } else if self.model_learning_enabled {
            match self.organism.observe_phase_native_rival_probe_result(action, &post) {
                Some(count) => count,
                None => {
                    return Ok(self.latch_fault("native factual update rejected".into()));
                }
            }
        } else {
            // Frozen models still receive the actual observation; they are not
            // silently handed a repaired prediction or a fictitious reset.
            self.organism.observe_initial_real(&post, false);
            0
        };
        self.record(LifetimeEventKind::Executed {
            action, suppressed, learned: self.model_learning_enabled,
        });
        Ok(StepOutcome::Executed {
            proposal, suppressed, learned: self.model_learning_enabled,
        })
    }

    /// Unified INTEL-2 execution path. The evaluator supplies only raw factual
    /// POST plus bounded task outcome. No reasoning-class credit label exists.
    pub fn step_unified<Assess, Execute>(
        &mut self,
        assess: Assess,
        execute: Execute,
    ) -> Result<StepOutcome, RuntimeError>
    where
        Assess: FnOnce(&ActionProposal) -> Option<HumanProtectionEvidence>,
        Execute: FnOnce(usize) -> Result<(Vec<f32>, f32), String>,
    {
        if self.protection.emergency_stop_latched() {
            let decision = self.protection.screen(0, Self::absent_evidence());
            let record = decision.record().clone();
            self.record(LifetimeEventKind::Blocked(record.clone()));
            return Ok(StepOutcome::Blocked(record));
        }

        let Some((proposal, unified, pre_proposals)) = self.select_unified_internal()? else {
            return Ok(StepOutcome::GoalReached);
        };

        let evidence = assess(&proposal).unwrap_or_else(Self::absent_evidence);
        let decision = self.protection.screen(proposal.action, evidence);
        let screening = decision.record().clone();
        let Some(permit) = decision.into_permit() else {
            self.record(LifetimeEventKind::Blocked(screening.clone()));
            return Ok(StepOutcome::Blocked(screening));
        };
        let action = match self.protection.consume_permit(permit) {
            Ok(action) => action,
            Err(error) => {
                return Ok(self.latch_fault(format!("permit consumption: {:?}", error)));
            }
        };

        let before_knowledge = self.organism.phase_native_unified_knowledge_snapshot();
        let factual_pre = self.organism.current_real()
            .ok_or(RuntimeError::FreshObservationRequired)?
            .sensory.clone();
        let (post, task_outcome) = match execute(action) {
            Ok(result) => result,
            Err(error) => return Ok(self.latch_fault(error)),
        };
        if !task_outcome.is_finite() || !(0.0..=1.0).contains(&task_outcome) {
            return Ok(self.latch_fault("invalid bounded task outcome".into()));
        }
        if let Err(error) = self.validate_raster(&post) {
            return Ok(self.latch_fault(format!("unusable factual POST: {}", error)));
        }

        let any_refinement =
            self.organism.phase_native_compositional_enabled()
                || self.organism.phase_native_perceptual_enabled()
                || self.organism.phase_native_context_enabled();
        let suppressed = if any_refinement {
            match self.organism.observe_phase_native_refinement_fanout_result(action, &post) {
                Some(count) => count,
                None => return Ok(self.latch_fault(
                    "unified factual refinement fanout rejected".into()
                )),
            }
        } else if self.model_learning_enabled {
            match self.organism.observe_phase_native_rival_probe_result(action, &post) {
                Some(count) => count,
                None => return Ok(self.latch_fault(
                    "unified native factual update rejected".into()
                )),
            }
        } else {
            self.organism.observe_initial_real(&post, false);
            0
        };

        let after_knowledge = self.organism.phase_native_unified_knowledge_snapshot();
        let info_gain = after_knowledge.gained_since(before_knowledge);

        // U2 ecology receives generic factual credit for every persistent
        // proposal that was applicable around this fact. No reasoning-class
        // label is present. Increased structure applicability is passive
        // supporting evidence; selected proposal also receives task/info gain.
        let goal_now = self.goal.as_ref().ok_or(RuntimeError::GoalRequired)?.clone();
        let post_proposals = self.organism.collect_phase_native_unified_proposals(&goal_now);
        let mut candidate_ids = pre_proposals.iter()
            .chain(post_proposals.iter())
            .filter_map(|p|p.persistent_candidate_id)
            .collect::<Vec<_>>();
        candidate_ids.sort_unstable();
        candidate_ids.dedup();

        for candidate_id in candidate_ids {
            let before_app = pre_proposals.iter()
                .filter(|p|p.persistent_candidate_id==Some(candidate_id))
                .map(|p|p.applicability)
                .fold(0.0_f32,f32::max);
            let after_app = post_proposals.iter()
                .filter(|p|p.persistent_candidate_id==Some(candidate_id))
                .map(|p|p.applicability)
                .fold(0.0_f32,f32::max);
            let evidence_gain = after_app > before_app + 1.0e-6;
            let selected = unified.supporting_candidate_ids.contains(&candidate_id);
            let selected_gain = if selected {
                task_outcome.max(if info_gain {1.0}else{0.0})
            } else {
                0.0
            };
            let usefulness = selected_gain.max(if evidence_gain {1.0}else{0.0});

            if (selected || evidence_gain)
                && !self.organism.phase_native_hypothesis_registered(candidate_id)
            {
                let _ = self.organism.register_phase_native_hypothesis(candidate_id);
            }
            if self.organism.phase_native_hypothesis_registered(candidate_id) {
                let _ = self.organism.observe_phase_native_hypothesis_utility(
                    candidate_id,
                    usefulness,
                );
            }
        }

        self.record(LifetimeEventKind::Executed {
            action,
            suppressed,
            learned: self.model_learning_enabled,
        });
        Ok(StepOutcome::Executed {
            proposal,
            suppressed,
            learned: self.model_learning_enabled,
        })
    }

    /// In-memory cognitive restart only. External safety authority, goal and
    /// lifetime audit survive. No saved observation is replayed as fresh fact.
    pub fn restart_cognition(&mut self) -> Result<(), RuntimeError> {
        let checkpoint: PhaseNativeCheckpoint = self.organism.phase_native_checkpoint()
            .ok_or(RuntimeError::NativeModelRequired)?;
        let mut replacement = EvoPhase::new(self.organism.config().clone());
        if !replacement.restore_phase_native_checkpoint(checkpoint) {
            return Err(RuntimeError::InvalidCheckpoint);
        }
        self.organism = replacement;
        self.fresh_observation_required = true;
        self.record(LifetimeEventKind::CognitiveRestart);
        Ok(())
    }

    /// External operator command; a deployment must authenticate its caller.
    /// It is not part of the cognitive action vocabulary.
    pub fn external_operator_reset(&mut self) {
        self.protection.external_human_emergency_reset();
        self.record(LifetimeEventKind::OperatorReset);
    }

    pub fn last_protection_reason(&self) -> Option<HumanProtectionReason> {
        self.protection.last_record().map(|record| record.reason)
    }
}
