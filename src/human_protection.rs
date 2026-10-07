//! Non-bypassable Human Protection boundary for physical action proposals.
//!
//! This module is intentionally independent of learned cognition. It accepts
//! externally supplied hazard evidence and returns an authorization verdict.
//! Learned rewards, planning values, concepts and drive weights are absent.

pub const HUMAN_HARM_BLOCK_THRESHOLD: f32 = 0.01;
pub const HUMAN_HAZARD_CONFIDENCE_MIN: f32 = 0.90;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HumanProtectionEvidence {
    pub human_present: bool,
    pub physical_effect_possible: bool,
    pub predicted_harm_probability: f32,
    pub hazard_confidence: f32,
    pub emergency_stop: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanProtectionVerdict {
    Allow,
    Block,
    EmergencyStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanProtectionReason {
    SafeEvidence,
    EmergencyStopSignal,
    EmergencyStopLatched,
    InvalidEvidence,
    ExcessHumanHarmRisk,
    InsufficientHazardConfidence,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HumanProtectionRecord {
    pub sequence: u64,
    pub proposed_action: usize,
    pub verdict: HumanProtectionVerdict,
    pub reason: HumanProtectionReason,
    pub evidence: HumanProtectionEvidence,
    pub emergency_stop_latched: bool,
}

/// Opaque, single-use authorization proof returned only for an allowed
/// proposal. It is deliberately neither Clone nor Copy.
#[derive(Debug, PartialEq, Eq)]
pub struct HumanProtectionPermit {
    action: usize,
    sequence: u64,
    _sealed: (),
}

impl HumanProtectionPermit {
    pub fn action(&self) -> usize {
        self.action
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

#[derive(Debug, PartialEq)]
pub enum HumanProtectionDecision {
    Allowed {
        permit: HumanProtectionPermit,
        record: HumanProtectionRecord,
    },
    Blocked(HumanProtectionRecord),
}

impl HumanProtectionDecision {
    pub fn record(&self) -> &HumanProtectionRecord {
        match self {
            Self::Allowed { record, .. } => record,
            Self::Blocked(record) => record,
        }
    }

    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }

    /// Consume the decision object to extract its non-cloneable permit.
    pub fn into_permit(self) -> Option<HumanProtectionPermit> {
        match self {
            Self::Allowed { permit, .. } => Some(permit),
            Self::Blocked(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanProtectionPermitError {
    EmergencyStopLatched,
    StaleOrUnknownPermit,
    AlreadyConsumed,
    RecordMismatch,
}

#[derive(Debug, Clone)]
pub struct HumanProtection {
    sequence: u64,
    emergency_stop_latched: bool,
    last_record: Option<HumanProtectionRecord>,
    last_issued_sequence: Option<u64>,
    last_consumed_sequence: Option<u64>,
}

impl Default for HumanProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl HumanProtection {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            emergency_stop_latched: false,
            last_record: None,
            last_issued_sequence: None,
            last_consumed_sequence: None,
        }
    }

    pub fn emergency_stop_latched(&self) -> bool {
        self.emergency_stop_latched
    }

    pub fn last_record(&self) -> Option<&HumanProtectionRecord> {
        self.last_record.as_ref()
    }

    pub fn screen(
        &mut self,
        proposed_action: usize,
        evidence: HumanProtectionEvidence,
    ) -> HumanProtectionDecision {
        self.sequence = self.sequence.saturating_add(1);

        let (verdict, reason) = if evidence.emergency_stop {
            self.emergency_stop_latched = true;
            (
                HumanProtectionVerdict::EmergencyStop,
                HumanProtectionReason::EmergencyStopSignal,
            )
        } else if self.emergency_stop_latched {
            (
                HumanProtectionVerdict::Block,
                HumanProtectionReason::EmergencyStopLatched,
            )
        } else if !valid_probability(evidence.predicted_harm_probability)
            || !valid_probability(evidence.hazard_confidence)
        {
            (
                HumanProtectionVerdict::Block,
                HumanProtectionReason::InvalidEvidence,
            )
        } else if evidence.human_present
            && evidence.physical_effect_possible
            && evidence.predicted_harm_probability > HUMAN_HARM_BLOCK_THRESHOLD
        {
            (
                HumanProtectionVerdict::Block,
                HumanProtectionReason::ExcessHumanHarmRisk,
            )
        } else if evidence.physical_effect_possible
            && evidence.hazard_confidence < HUMAN_HAZARD_CONFIDENCE_MIN
        {
            // Fail closed even when the external presence estimate says false:
            // low evidence confidence means absence itself is not trusted.
            (
                HumanProtectionVerdict::Block,
                HumanProtectionReason::InsufficientHazardConfidence,
            )
        } else {
            (
                HumanProtectionVerdict::Allow,
                HumanProtectionReason::SafeEvidence,
            )
        };

        let record = HumanProtectionRecord {
            sequence: self.sequence,
            proposed_action,
            verdict,
            reason,
            evidence,
            emergency_stop_latched: self.emergency_stop_latched,
        };
        self.last_record = Some(record.clone());

        if verdict == HumanProtectionVerdict::Allow {
            self.last_issued_sequence = Some(self.sequence);
            HumanProtectionDecision::Allowed {
                permit: HumanProtectionPermit {
                    action: proposed_action,
                    sequence: self.sequence,
                    _sealed: (),
                },
                record,
            }
        } else {
            // Any later screen invalidates every older permit.
            self.last_issued_sequence = None;
            HumanProtectionDecision::Blocked(record)
        }
    }

    /// Mandatory actuator-side consumption of a screened action permit.
    ///
    /// Safe Rust ownership makes the permit itself single-use, while sequence
    /// validation prevents a previously issued but unconsumed permit from being
    /// used after any later screening event.
    pub fn consume_permit(
        &mut self,
        permit: HumanProtectionPermit,
    ) -> Result<usize, HumanProtectionPermitError> {
        if self.emergency_stop_latched {
            return Err(HumanProtectionPermitError::EmergencyStopLatched);
        }
        if self.last_consumed_sequence == Some(permit.sequence) {
            return Err(HumanProtectionPermitError::AlreadyConsumed);
        }
        if self.sequence != permit.sequence
            || self.last_issued_sequence != Some(permit.sequence)
        {
            return Err(HumanProtectionPermitError::StaleOrUnknownPermit);
        }
        let Some(record) = self.last_record.as_ref() else {
            return Err(HumanProtectionPermitError::RecordMismatch);
        };
        if record.sequence != permit.sequence
            || record.proposed_action != permit.action
            || record.verdict != HumanProtectionVerdict::Allow
        {
            return Err(HumanProtectionPermitError::RecordMismatch);
        }

        self.last_consumed_sequence = Some(permit.sequence);
        self.last_issued_sequence = None;
        Ok(permit.action)
    }

    /// Explicit external operator path.
    ///
    /// This is deliberately not named as a generic reset and is not called by
    /// learning/planning code. Real deployments must authenticate the caller at
    /// the actuator/runtime boundary.
    pub fn external_human_emergency_reset(&mut self) {
        self.emergency_stop_latched = false;
        self.last_issued_sequence = None;
    }
}

fn valid_probability(value: f32) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
