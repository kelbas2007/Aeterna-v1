//! Persistent orchestration of acquired native reasoning, not a second planner.
//!
//! The host supplies raw goals, hazard evidence and actual observations. Native
//! EvoPhase selectors own action choice; the opt-in learned-rule mode uses its
//! explicitly documented bounded software search. The private protection gate owns the
//! single-use permission to invoke the external action callback. This wrapper
//! has no world labels, route table or task-conditioned learning rules.

use crate::carrier::{
    EvoPhase, PhaseNativeCheckpoint, PhaseCognitiveProposal,
    PhaseMetaControlCheckpoint, PhaseMetaDecision,
    PhaseHypothesisEcologyConfig, PhaseUnifiedDecision, PhaseUnifiedCognitiveProposal,
    PhaseUnifiedActionTrace, PhaseTemporalEvidenceReadout,
    PhaseTemporalSensingDecision, PhaseTemporalOutcomeDecision, PhaseTemporalEvidenceConfig,
    PhasePartialDecisionKind,
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
    LearnedRulePlanning,
    RuleExperiment,
    AdaptiveRulePlanning,
    AdaptiveRuleExperiment,
    PartialGoalPlanning,
    PartialInformationGathering,
    PartialMaskExploration,
    VectorPrediction,
    InducedProgramPrediction,
    VectorMeasurement,
    VectorExperiment,
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
    RepresentationCapacity,
    GoalRequired,
    FreshObservationRequired,
    NoSupportedAction,
    InvalidCheckpoint,
    PartialModeRequired,
    VectorModeRequired,
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
    /// Bounded, opt-in factual evidence of U1 competition and its consequence.
    UnifiedDecisionTrace(UnifiedDecisionTrace),
    CognitiveRestart,
    OperatorReset,
}

#[derive(Debug, Clone)]
pub struct LifetimeEvent {
    pub sequence: u64,
    pub goal_epoch: u64,
    pub kind: LifetimeEventKind,
}

/// Physical U1 arbitration before action and an optional factual consequence.
/// No evaluator labels or privileged actor categories enter this audit.
#[derive(Debug, Clone)]
pub struct UnifiedDecisionTrace {
    pub selected_action: usize,
    pub actions: Vec<PhaseUnifiedActionTrace>,
    pub belief_before: Option<PhaseTemporalEvidenceReadout>,
    pub sensing_before: Option<PhaseTemporalSensingDecision>,
    pub outcome_before: Option<PhaseTemporalOutcomeDecision>,
    pub factual_outcome: Option<f32>,
    pub acquired_sample: bool,
    pub online_utility: Option<f32>,
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
    partial_goal: Option<Vec<Option<f32>>>,
    outcome_goal: Option<f32>,
    goal_epoch: u64,
    sequence: u64,
    audit: VecDeque<LifetimeEvent>,
    fresh_observation_required: bool,
    model_learning_enabled: bool,
    online_unified_learning_enabled: bool,
    unified_trace_enabled: bool,
}

impl ScientificRuntime {
    pub fn new(organism: EvoPhase) -> Result<Self, RuntimeError> {
        if organism.phase_native_checkpoint().is_none() {
            return Err(RuntimeError::NativeModelRequired);
        }
        let model_learning_enabled = organism.native_model_learning_enabled();
        Ok(Self {
            organism,
            protection: HumanProtection::new(),
            goal: None,
            partial_goal: None,
            outcome_goal: None,
            goal_epoch: 0,
            sequence: 0,
            audit: VecDeque::new(),
            fresh_observation_required: true,
            model_learning_enabled,
            online_unified_learning_enabled: false,
            unified_trace_enabled: false,
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

    /// Opt-in, factual U1 plasticity. Unlike the historical frozen checkpoint,
    /// this trains the EXISTING physical meta-synapses from executed POSTs.
    /// It never selects or permits an action and is disabled during holdout.
    pub fn set_unified_online_learning(&mut self, enabled: bool) -> bool {
        if !self.organism.phase_native_meta_control_enabled() { return false; }
        self.online_unified_learning_enabled = enabled;
        self.organism.set_phase_native_meta_learning_enabled(
            enabled && self.model_learning_enabled
        );
        true
    }

    /// Extend the existing phase-native temporal affordance to physically
    /// acquired two-action sequences with a factual intermediate state.
    pub fn set_temporal_chain_learning(&mut self, enabled: bool) -> bool {
        self.organism.set_phase_native_temporal_chain_learning(enabled)
    }

    /// Opt-in generic investigation of under-tested opaque motors when
    /// physical belief remains insufficient. EvoPhase supplies the candidate;
    /// the unchanged U1 and Human Protection still choose and permit actions.
    pub fn set_temporal_autonomous_probe(&mut self, enabled: bool) -> bool {
        self.organism.set_phase_native_temporal_autonomous_probe(enabled)
    }

    /// Change only the U1 scoring equation, not candidate classes or safety.
    /// Default remains the historical INTEL-4 compatible normalized policy.
    /// Once enabled, the policy is stored in the physical meta checkpoint.
    pub fn set_unified_monotone_evidence_scoring(&mut self, enabled: bool) -> bool {
        self.organism.set_phase_native_meta_monotone_evidence(enabled)
    }

    /// Opt-in bounded evidence trace. Turning on diagnostics cannot change U1.
    pub fn set_unified_decision_tracing(&mut self, enabled: bool) {
        self.unified_trace_enabled = enabled;
    }

    /// Read-only view of the precise coalesced fields which U1 will score.
    pub fn inspect_unified_competition(&self) -> Option<Vec<PhaseUnifiedActionTrace>> {
        let goal = self.goal.as_ref()?;
        let proposals = self.organism.collect_phase_native_unified_proposals(goal);
        self.organism.trace_phase_native_unified_competition(&proposals)
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
            mode: if self.organism.phase_adaptive_rules_enabled() {
                if proposals.iter().any(|p|p.proposal.action == decision.action && p.proposal.fields[0] > 0.0) {
                    ReasoningMode::AdaptiveRulePlanning
                } else { ReasoningMode::AdaptiveRuleExperiment }
            } else if self.organism.phase_rules_enabled() {
                if proposals.iter().any(|p|p.proposal.action == decision.action && p.proposal.fields[0] > 0.0) {
                    ReasoningMode::LearnedRulePlanning
                } else { ReasoningMode::RuleExperiment }
            } else { ReasoningMode::UnifiedCompetition },
            goal_epoch: self.goal_epoch,
            learned_fingerprint: self.organism.phase_native_learned_fingerprint(),
        };
        self.record(LifetimeEventKind::Proposal(proposal.clone()));
        Ok(Some((proposal,decision,proposals)))
    }

    pub fn propose_unified(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {
        if self.organism.phase_vector_enabled() { return self.propose_vector(); }
        if self.organism.phase_partial_enabled() { return self.propose_partial(); }
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

    fn validate_sensor_values(&self, raster: &[f32]) -> Result<(), RuntimeError> {
        if raster.len() != self.organism.config().sensory_cells
            || raster.iter().any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
            || (self.organism.phase_rules_enabled() && raster.contains(&1.0))
        {
            return Err(RuntimeError::InvalidRaster);
        }
        Ok(())
    }

    fn validate_raster(&self, raster: &[f32]) -> Result<(), RuntimeError> {
        self.validate_sensor_values(raster)?;
        if self.organism.phase_rules_enabled() {
            // A feature vector need not be a previously memorized state. Its
            // transition remains unknown until factual support confirms a rule.
            return Ok(());
        }
        if self.organism.phase_native_abstract_state(raster).is_none() {
            return Err(RuntimeError::UnknownRepresentation);
        }
        Ok(())
    }

    fn prepare_factual_observation(&mut self, raster: &[f32]) -> Result<(), RuntimeError> {
        self.validate_sensor_values(raster)?;
        if self.organism.phase_rules_enabled() { return Ok(()); }
        if self.organism.phase_native_online_enabled()
            && !self.organism.acquire_phase_native_online_observation(raster)
        {
            return Err(if self.model_learning_enabled {
                RuntimeError::RepresentationCapacity
            } else {
                RuntimeError::UnknownRepresentation
            });
        }
        self.validate_raster(raster)
    }

    /// Initial sensing or fresh sensing after restart/fault. The observation
    /// must come from outside cognition. Online mode may acquire a sensory
    /// receptor, but no action consequence is invented at this boundary.
    pub fn observe_external(&mut self, raster: &[f32]) -> Result<(), RuntimeError> {
        if self.organism.phase_partial_enabled() || self.organism.phase_vector_enabled() {
            let observation = raster.iter().copied().map(Some).collect::<Vec<_>>();
            return self.observe_external_partial(&observation);
        }
        self.prepare_factual_observation(raster)?;
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
        if self.organism.phase_vector_enabled() { return Err(RuntimeError::VectorModeRequired); }
        if self.organism.phase_partial_enabled() {
            let goal = raster.iter().copied().map(Some).collect::<Vec<_>>();
            return self.set_partial_goal(&goal);
        }
        if self.organism.phase_native_online_enabled() {
            // A desired observation is not factual experience. Recognition
            // and a route may be acquired later from actual consequences.
            self.validate_sensor_values(raster)?;
        } else {
            self.validate_raster(raster)?;
        }
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
        if self.online_unified_learning_enabled {
            self.organism.set_phase_native_meta_learning_enabled(enabled);
        }
    }

    /// Multivariate responses require actual bounded external outcomes. A
    /// desired image or an externally supplied response label is not a goal.
    pub fn set_outcome_goal(&mut self, target: f32) -> Result<(),RuntimeError> {
        if !self.organism.phase_vector_enabled() { return Err(RuntimeError::VectorModeRequired); }
        if !target.is_finite() || !(0.5..=1.0).contains(&target) { return Err(RuntimeError::InvalidRaster); }
        self.goal_epoch=self.goal_epoch.checked_add(1).expect("goal epoch exhausted");
        self.outcome_goal=Some(target); self.goal=None; self.partial_goal=None;
        self.record(LifetimeEventKind::GoalChanged); Ok(())
    }

    pub fn propose_vector(&mut self) -> Result<Option<ActionProposal>,RuntimeError> {
        if !self.organism.phase_vector_enabled() { return Err(RuntimeError::VectorModeRequired); }
        if self.goal_reached()? { return Ok(None); }
        let decision=self.organism.phase_vector_decision().ok_or(RuntimeError::NoSupportedAction)?;
        let mode=match decision.kind {
            crate::carrier::PhaseVectorDecisionKind::Prediction=>ReasoningMode::VectorPrediction,
            crate::carrier::PhaseVectorDecisionKind::InducedProgram=>ReasoningMode::InducedProgramPrediction,
            crate::carrier::PhaseVectorDecisionKind::Measurement=>ReasoningMode::VectorMeasurement,
            crate::carrier::PhaseVectorDecisionKind::Experiment=>ReasoningMode::VectorExperiment,
        };
        let proposal=ActionProposal { action:decision.action,mode,goal_epoch:self.goal_epoch,learned_fingerprint:self.organism.phase_native_learned_fingerprint() };
        self.record(LifetimeEventKind::Proposal(proposal.clone())); Ok(Some(proposal))
    }

    pub fn goal_reached(&self) -> Result<bool, RuntimeError> {
        if self.fresh_observation_required {
            return Err(RuntimeError::FreshObservationRequired);
        }
        if self.organism.phase_vector_enabled() {
            let target=self.outcome_goal.ok_or(RuntimeError::GoalRequired)?;
            return Ok(self.organism.phase_vector_last_outcome().is_some_and(|value| value>=target));
        }
        if self.organism.phase_partial_enabled() {
            let goal = self.partial_goal.as_ref().ok_or(RuntimeError::GoalRequired)?;
            return Ok(self.organism.phase_partial_goal_reached(goal));
        }
        let goal = self.goal.as_ref().ok_or(RuntimeError::GoalRequired)?;
        let real = self.organism.current_real()
            .ok_or(RuntimeError::FreshObservationRequired)?;
        if self.organism.phase_rules_enabled() {
            if self.organism.phase_adaptive_rules_enabled() {
                return Ok(self.organism.phase_adaptive_goal_matches(&real.sensory, goal));
            }
            return Ok(self.organism.phase_rule_goal_matches(&real.sensory,goal));
        }
        let current = self.organism.phase_native_abstract_state(&real.sensory)
            .ok_or(RuntimeError::UnknownRepresentation)?;
        let target = match self.organism.phase_native_abstract_state(goal) {
            Some(target) => target,
            None if self.organism.phase_native_online_enabled() => return Ok(false),
            None => return Err(RuntimeError::UnknownRepresentation),
        };
        Ok(current.level == target.level && current.cell == target.cell)
    }

    /// Fixed orchestration of existing native mechanisms. Task-level scoring
    /// remains in EvoPhase; this function does not construct or search a model.
    pub fn propose(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {
        if self.organism.phase_vector_enabled() { return self.propose_vector(); }
        if self.organism.phase_native_online_enabled() {
            return self.propose_unified();
        }
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

    /// The step methods invoke the action callback through the same mandatory
    /// authorization boundary. Hazard assessment and the
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
        if self.organism.phase_vector_enabled() { return Err(RuntimeError::VectorModeRequired); }
        if self.organism.phase_native_online_enabled() {
            return self.step_unified(assess, |action| execute(action).map(|post| (post, 0.0)));
        }
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
        if let Err(error) = self.prepare_factual_observation(&post) {
            return Ok(self.latch_fault(format!("unusable factual POST: {}", error)));
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
        if self.organism.phase_partial_enabled() || self.organism.phase_vector_enabled() {
            return self.step_dense_partial(assess, execute);
        }
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

        // Capture physical competition BEFORE the permitted external action;
        // this is the only feature vector eligible for subsequent U1 credit.
        let competition = if self.online_unified_learning_enabled
            || self.unified_trace_enabled
        {
            self.organism.trace_phase_native_unified_competition(&pre_proposals)
                .unwrap_or_default()
        } else { Vec::new() };
        let selected_fields = competition.iter()
            .find(|item| item.action == action).map(|item| item.fields);
        let belief_before = if self.online_unified_learning_enabled
            || self.unified_trace_enabled
        { self.organism.phase_native_temporal_evidence() } else { None };
        let sensing_before = if self.unified_trace_enabled {
            self.organism.choose_phase_native_temporal_sensing_action()
        } else { None };
        let outcome_before = if self.unified_trace_enabled {
            self.organism.choose_phase_native_temporal_outcome_action()
        } else { None };

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
        if let Err(error) = self.prepare_factual_observation(&post) {
            return Ok(self.latch_fault(format!("unusable factual POST: {}", error)));
        }

        // Commit sensory affordance only after one permitted and executed
        // motor returned a validated FACTUAL POST. Non-sensing no-op actions
        // do NOT count another copy of the same cue as new information.
        if self.organism.phase_native_temporal_evidence_enabled() {
            if self.model_learning_enabled {
                // TE5: real terminal-like POST + factual bounded outcome is
                // credited against PRE-action carrier belief, never to a
                // host-given hidden class or a prepared correct-motor table.
                let _=self.organism.observe_phase_native_temporal_outcome(
                    action,&post,task_outcome
                );
                let _=self.organism.observe_phase_native_sensing_affordance(
                    action,&factual_pre,&post
                );
            }
            if self.organism.phase_native_temporal_source_count()==2
                && self.organism.phase_native_temporal_factual_sample_from(
                    action,&factual_pre,&post
                )
            {
                let _=self.organism.observe_phase_native_temporal_signal(&post);
            }
        }

        let any_refinement =
            self.organism.phase_native_compositional_enabled()
                || self.organism.phase_native_perceptual_enabled()
                || self.organism.phase_native_context_enabled();
        let suppressed = if self.organism.phase_rules_enabled() {
            let count = if self.model_learning_enabled {
                match self.organism.observe_phase_rule_result(action,&factual_pre,&post) {
                    Some(count) => count,
                    None => return Ok(self.latch_fault("factual rule update rejected".into())),
                }
            } else { 0 };
            self.organism.observe_initial_real(&post,false);
            count
        } else if any_refinement {
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

        // Credit only genuinely acquired evidence, never an unchanged no-op
        // or predicted/imagined cue. An actual sensory sample is informative
        // only if the *pre-action* physical belief still needed information.
        let acquired_sample = belief_before.as_ref().map(|prior| {
            self.organism.phase_native_temporal_evidence()
                .map(|after| after.observations > prior.observations)
                .unwrap_or(false)
        }).unwrap_or(false);
        let sensor_information = if acquired_sample
            && belief_before.as_ref().is_some_and(|prior| prior.needs_more)
        {
            self.organism.phase_native_temporal_action_affordance(action)
                .unwrap_or(0.0).clamp(0.0,1.0) * 0.5
        } else { 0.0 };
        let online_utility = if self.online_unified_learning_enabled
            && self.model_learning_enabled
        {
            let value = task_outcome.max(sensor_information)
                .max(if info_gain { 0.5 } else { 0.0 });
            if let Some(fields) = selected_fields {
                if self.organism.observe_phase_native_meta_utility(fields,value) {
                    Some(value)
                } else { None }
            } else { None }
        } else { None };

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

        if self.unified_trace_enabled {
            let mut actions = competition;
            // Bounded action diagnostics even if the external actuator has
            // an unusually large action space. Keep high-score rivals first.
            actions.sort_by(|a,b| b.score.total_cmp(&a.score)
                .then_with(||a.action.cmp(&b.action)));
            actions.truncate(16);
            self.record(LifetimeEventKind::UnifiedDecisionTrace(
                UnifiedDecisionTrace {
                    selected_action: action,
                    actions,
                    belief_before,
                    sensing_before,
                    outcome_before,
                    factual_outcome: Some(task_outcome),
                    acquired_sample,
                    online_utility,
                }
            ));
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

    fn validate_partial_values(&self, values: &[Option<f32>]) -> Result<(), RuntimeError> {
        if !self.organism.phase_partial_enabled() && !self.organism.phase_vector_enabled() {
            return Err(RuntimeError::PartialModeRequired);
        }
        if values.len() != self.organism.config().sensory_cells
            || values
                .iter()
                .flatten()
                .any(|x| !x.is_finite() || if self.organism.phase_vector_enabled() { !(0.0..=1.0).contains(x) } else { !(0.0..1.0).contains(x) })
        {
            return Err(RuntimeError::InvalidRaster);
        }
        Ok(())
    }

    /// A fresh sparse observation resets episodic beliefs. None carries no
    /// factual value and is never materialized as a synthetic zero channel.
    pub fn observe_external_partial(&mut self, values: &[Option<f32>]) -> Result<(), RuntimeError> {
        self.validate_partial_values(values)?;
        let accepted=if self.organism.phase_vector_enabled() { self.organism.observe_phase_vector_initial(values) } else { self.organism.observe_phase_partial_initial(values) };
        if !accepted {
            return Err(RuntimeError::InvalidRaster);
        }
        self.fresh_observation_required = false;
        self.record(LifetimeEventKind::ExternalObservation);
        Ok(())
    }

    /// None in a goal is an unconstrained field. At least one constraint is
    /// required. Setting it does not supply hidden observations or any tuition.
    pub fn set_partial_goal(&mut self, values: &[Option<f32>]) -> Result<(), RuntimeError> {
        if self.organism.phase_vector_enabled() { return Err(RuntimeError::VectorModeRequired); }
        self.validate_partial_values(values)?;
        if !values.iter().any(Option::is_some) {
            return Err(RuntimeError::InvalidRaster);
        }
        self.goal_epoch = self.goal_epoch.checked_add(1).expect("goal epoch exhausted");
        self.partial_goal = Some(values.to_vec());
        self.goal = None;
        self.record(LifetimeEventKind::GoalChanged);
        Ok(())
    }

    pub fn propose_partial(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {
        if self.organism.phase_vector_enabled() { return self.propose_vector(); }
        if !self.organism.phase_partial_enabled() {
            return Err(RuntimeError::PartialModeRequired);
        }
        if self.goal_reached()? {
            return Ok(None);
        }
        let goal = self.partial_goal.as_ref().ok_or(RuntimeError::GoalRequired)?;
        let decision = self
            .organism
            .phase_partial_decision(goal)
            .ok_or(RuntimeError::NoSupportedAction)?;
        let mode = match decision.kind {
            PhasePartialDecisionKind::GoalPlan => ReasoningMode::PartialGoalPlanning,
            PhasePartialDecisionKind::InformationGathering => ReasoningMode::PartialInformationGathering,
            PhasePartialDecisionKind::MaskExploration => ReasoningMode::PartialMaskExploration,
        };
        let proposal = ActionProposal {
            action: decision.action,
            mode,
            goal_epoch: self.goal_epoch,
            learned_fingerprint: self.organism.phase_native_learned_fingerprint(),
        };
        self.record(LifetimeEventKind::Proposal(proposal.clone()));
        Ok(Some(proposal))
    }

    /// Partial POST is produced only by the external permitted actuator.
    /// State inference remains available while learned parameters are frozen.
    pub fn step_partial<Assess, Execute>(
        &mut self,
        assess: Assess,
        execute: Execute,
    ) -> Result<StepOutcome, RuntimeError>
    where
        Assess: FnOnce(&ActionProposal) -> Option<HumanProtectionEvidence>,
        Execute: FnOnce(usize) -> Result<(Vec<Option<f32>>, f32), String>,
    {
        if !self.organism.phase_partial_enabled() && !self.organism.phase_vector_enabled() {
            return Err(RuntimeError::PartialModeRequired);
        }
        if self.protection.emergency_stop_latched() {
            let record = self.protection.screen(0, Self::absent_evidence()).record().clone();
            self.record(LifetimeEventKind::Blocked(record.clone()));
            return Ok(StepOutcome::Blocked(record));
        }
        let Some(proposal) = self.propose_partial()? else {
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
            Err(error) => return Ok(self.latch_fault(format!("permit consumption: {error:?}"))),
        };
        let (post, outcome) = match execute(action) {
            Ok(result) => result,
            Err(error) => return Ok(self.latch_fault(error)),
        };
        if !outcome.is_finite() || !(0.0..=1.0).contains(&outcome) {
            return Ok(self.latch_fault("invalid bounded task outcome".into()));
        }
        if let Err(error) = self.validate_partial_values(&post) {
            return Ok(self.latch_fault(format!("unusable factual partial POST: {error}")));
        }
        let update=if self.organism.phase_vector_enabled() { self.organism.observe_phase_vector_result(action,&post,outcome) } else { self.organism.observe_phase_partial_result(action,&post) };
        let report = match update {
            Some(report) => report,
            None => return Ok(self.latch_fault("factual partial update rejected".into())),
        };
        self.record(LifetimeEventKind::Executed {
            action,
            suppressed: report.suppressed,
            learned: report.learned,
        });
        Ok(StepOutcome::Executed {
            proposal,
            suppressed: report.suppressed,
            learned: report.learned,
        })
    }

    /// Adapt dense sensors to the partial execution seam. Conversion happens
    /// inside the executor callback, after that seam consumes its permit.
    fn step_dense_partial<Assess, Execute>(
        &mut self,
        assess: Assess,
        execute: Execute,
    ) -> Result<StepOutcome, RuntimeError>
    where
        Assess: FnOnce(&ActionProposal) -> Option<HumanProtectionEvidence>,
        Execute: FnOnce(usize) -> Result<(Vec<f32>, f32), String>,
    {
        self.step_partial(assess, |action| {
            execute(action).map(|(post, outcome)| (post.into_iter().map(Some).collect(), outcome))
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

    /// Restore persisted online cognition atomically while keeping the current
    /// protection latch, goal and audit. Checkpoint data has no actuator authority.
    pub fn restore_online_checkpoint(&mut self, bytes: &[u8]) -> Result<(), RuntimeError> {
        let mut replacement = EvoPhase::from_online_checkpoint(bytes)
            .map_err(|_| RuntimeError::InvalidCheckpoint)?;
        if replacement.config().sensory_cells != self.organism.config().sensory_cells
            || replacement.config().motor_cells != self.organism.config().motor_cells
            || replacement.phase_vector_enabled()!=self.organism.phase_vector_enabled()
        {
            return Err(RuntimeError::InvalidCheckpoint);
        }
        // Translate a full goal exactly when switching observation modes.
        // A sparse goal has no exact full-vector equivalent, so reject that
        // switch atomically rather than invent values for unconstrained fields.
        let mut next_goal = self.goal.clone();
        let mut next_partial_goal = self.partial_goal.clone();
        if replacement.phase_partial_enabled() && !self.organism.phase_partial_enabled() {
            next_partial_goal = next_goal.take().map(|goal| goal.into_iter().map(Some).collect());
        } else if !replacement.phase_partial_enabled() && self.organism.phase_partial_enabled() {
            if let Some(goal) = next_partial_goal.take() {
                next_goal = Some(
                    goal.into_iter().collect::<Option<Vec<_>>>().ok_or(RuntimeError::InvalidCheckpoint)?
                );
            }
        }
        replacement.set_planning_learning_enabled(self.model_learning_enabled);
        self.organism = replacement;
        self.goal = next_goal;
        self.partial_goal = next_partial_goal;
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
