//! Persistent orchestration of acquired native reasoning, not a second planner.
//!
//! The host supplies raw goals, hazard evidence and actual observations. Native
//! EvoPhase selectors own action choice. The private protection gate owns the
//! single-use permission to invoke the external action callback. This wrapper
//! has no world labels, route table or task-conditioned learning rules.

use crate::carrier::{EvoPhase, PhaseNativeCheckpoint};
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
        let (action, mode) = if let Some(action) =
            self.organism.choose_phase_native_goal_rival_probe(&goal)
        {
            (action, ReasoningMode::RivalDiscrimination)
        } else {
            let action = self.organism.choose_phase_native_goal_active_action(&goal)
                .ok_or(RuntimeError::NoSupportedAction)?;
            (action, ReasoningMode::GoalDirectedAction)
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

        let suppressed = if self.model_learning_enabled {
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
