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

/// Opaque authorization proof returned only for an allowed proposal.
///
/// The private field prevents cognition from constructing a valid permit
/// without passing through HumanProtection::screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
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

    pub fn permit(&self) -> Option<HumanProtectionPermit> {
        match self {
            Self::Allowed { permit, .. } => Some(*permit),
            Self::Blocked(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HumanProtection {
    sequence: u64,
    emergency_stop_latched: bool,
    last_record: Option<HumanProtectionRecord>,
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
        } else if evidence.human_present
            && evidence.physical_effect_possible
            && evidence.hazard_confidence < HUMAN_HAZARD_CONFIDENCE_MIN
        {
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
            HumanProtectionDecision::Allowed {
                permit: HumanProtectionPermit {
                    action: proposed_action,
                    sequence: self.sequence,
                    _sealed: (),
                },
                record,
            }
        } else {
            HumanProtectionDecision::Blocked(record)
        }
    }

    /// Explicit external operator path.
    ///
    /// This is deliberately not named as a generic reset and is not called by
    /// learning/planning code. Real deployments must authenticate the caller at
    /// the actuator/runtime boundary.
    pub fn external_human_emergency_reset(&mut self) {
        self.emergency_stop_latched = false;
    }
}

fn valid_probability(value: f32) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
