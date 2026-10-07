use aeterna_v1::{
    EvoConfig, EvoPhase, HumanProtection, HumanProtectionDecision, HumanProtectionEvidence,
    HumanProtectionReason, HumanProtectionVerdict, HUMAN_HARM_BLOCK_THRESHOLD,
    HUMAN_HAZARD_CONFIDENCE_MIN,
};
use aeterna_v1::carrier::PhaseNativeConfig;

fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: true,
        physical_effect_possible: true,
        predicted_harm_probability: 0.001,
        hazard_confidence: 0.99,
        emergency_stop: false,
    }
}

#[test]
fn human_protection_blocks_harm_uncertainty_invalid_and_latches_emergency_stop() {
    assert_eq!(HUMAN_HARM_BLOCK_THRESHOLD, 0.01);
    assert_eq!(HUMAN_HAZARD_CONFIDENCE_MIN, 0.90);

    let mut gate = HumanProtection::new();

    let allow = gate.screen(2, safe());
    let permit = allow.permit().expect("safe proposal must receive sealed permit");
    assert_eq!(permit.action(), 2);
    assert_eq!(permit.sequence(), 1);
    assert_eq!(allow.record().verdict, HumanProtectionVerdict::Allow);
    assert_eq!(allow.record().reason, HumanProtectionReason::SafeEvidence);

    let high_risk = gate.screen(
        2,
        HumanProtectionEvidence {
            predicted_harm_probability: 0.011,
            ..safe()
        },
    );
    assert!(high_risk.permit().is_none());
    assert_eq!(high_risk.record().verdict, HumanProtectionVerdict::Block);
    assert_eq!(
        high_risk.record().reason,
        HumanProtectionReason::ExcessHumanHarmRisk
    );

    let uncertain = gate.screen(
        2,
        HumanProtectionEvidence {
            predicted_harm_probability: 0.0,
            hazard_confidence: 0.899,
            ..safe()
        },
    );
    assert!(uncertain.permit().is_none());
    assert_eq!(
        uncertain.record().reason,
        HumanProtectionReason::InsufficientHazardConfidence
    );

    let invalid = gate.screen(
        2,
        HumanProtectionEvidence {
            predicted_harm_probability: f32::NAN,
            ..safe()
        },
    );
    assert!(invalid.permit().is_none());
    assert_eq!(
        invalid.record().reason,
        HumanProtectionReason::InvalidEvidence
    );

    let emergency = gate.screen(
        2,
        HumanProtectionEvidence {
            emergency_stop: true,
            ..safe()
        },
    );
    assert!(emergency.permit().is_none());
    assert_eq!(
        emergency.record().verdict,
        HumanProtectionVerdict::EmergencyStop
    );
    assert!(gate.emergency_stop_latched());

    let safe_but_latched = gate.screen(2, safe());
    assert!(safe_but_latched.permit().is_none());
    assert_eq!(
        safe_but_latched.record().reason,
        HumanProtectionReason::EmergencyStopLatched
    );
    assert_eq!(safe_but_latched.record().sequence, 6);

    gate.external_human_emergency_reset();
    assert!(!gate.emergency_stop_latched());

    let allow_after_external_reset = gate.screen(2, safe());
    assert!(allow_after_external_reset.permit().is_some());
    assert_eq!(allow_after_external_reset.record().sequence, 7);
}

#[test]
fn human_protection_is_deterministic_and_cognitive_checkpoint_cannot_clear_latch() {
    let evidence = [
        safe(),
        HumanProtectionEvidence {
            predicted_harm_probability: 0.02,
            ..safe()
        },
        HumanProtectionEvidence {
            human_present: false,
            predicted_harm_probability: 0.9,
            hazard_confidence: 0.1,
            ..safe()
        },
    ];

    let mut a = HumanProtection::new();
    let mut b = HumanProtection::new();
    for (index, item) in evidence.into_iter().enumerate() {
        let ra = a.screen(index, item);
        let rb = b.screen(index, item);
        assert_eq!(ra.record(), rb.record());
    }

    let mut evo = EvoPhase::new(EvoConfig::default());
    evo.enable_phase_native_planning(PhaseNativeConfig::default());

    let emergency = evo.screen_physical_action(
        0,
        HumanProtectionEvidence {
            emergency_stop: true,
            ..safe()
        },
    );
    assert!(matches!(emergency, HumanProtectionDecision::Blocked(_)));
    assert!(evo.human_protection_emergency_latched());

    let checkpoint = evo
        .phase_native_checkpoint()
        .expect("cognitive checkpoint exists");

    // Restore cognitive state into the SAME protected runtime. The protection
    // latch is intentionally not part of the cognitive checkpoint.
    assert!(evo.restore_phase_native_checkpoint(checkpoint));
    assert!(evo.human_protection_emergency_latched());

    let still_blocked = evo.screen_physical_action(0, safe());
    assert!(still_blocked.permit().is_none());
    assert_eq!(
        still_blocked.record().reason,
        HumanProtectionReason::EmergencyStopLatched
    );

    evo.external_human_emergency_reset();
    assert!(!evo.human_protection_emergency_latched());
    assert!(evo.screen_physical_action(0, safe()).permit().is_some());
}

#[test]
fn human_protection_source_has_no_cognitive_override_dependency() {
    let source = include_str!("../src/human_protection.rs");

    for forbidden in [
        "use crate::",
        "PlanDecision",
        "PhaseDrive",
        "EvoConcept",
        "EvoImaginationPlanner",
        "predicted_value",
        "reward_override",
        "utility_override",
        "set_harm_threshold",
        "set_confidence_threshold",
    ] {
        assert!(
            !source.contains(forbidden),
            "Human Protection contains forbidden cognition/override token {forbidden}"
        );
    }

    for required in [
        "HUMAN_HARM_BLOCK_THRESHOLD",
        "HUMAN_HAZARD_CONFIDENCE_MIN",
        "emergency_stop_latched",
        "external_human_emergency_reset",
        "InvalidEvidence",
        "ExcessHumanHarmRisk",
        "InsufficientHazardConfidence",
    ] {
        assert!(
            source.contains(required),
            "Human Protection missing frozen dependency {required}"
        );
    }
}
