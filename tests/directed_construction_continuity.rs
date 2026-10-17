//! Cognitive persistence controls in a deterministic simulated device.
//! These tests never copy live external-action authority. The second runtime
//! owns a separate simulated device and independently issued action permits.
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::{Config, Error, ExternalPost, Learner, ProtectedConstructionRuntime};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};

fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}

fn runtime() -> ProtectedConstructionRuntime {
    let config = Config {
        minimum_training: 6,
        shadow_steps: 4,
        search_every: 2,
        search: SearchConfig { deterministic: true, ..Default::default() },
        ..Default::default()
    };
    let organism = EvoPhase::new(EvoConfig {
        sensory_cells: 1,
        motor_cells: 2,
        ..Default::default()
    });
    ProtectedConstructionRuntime::new(organism, config, vec![0]).unwrap()
}

fn advance(hidden: &mut i64, action: usize) -> ExternalPost {
    let observed = *hidden;
    *hidden = action as i64;
    ExternalPost { observation: vec![observed], cost: 1 }
}

fn with_pending_trial() -> (ProtectedConstructionRuntime, i64) {
    let mut rt = runtime();
    let mut hidden = 0;
    for i in 0..64 {
        let action = [1, 1, 0, 1, 0, 0, 1, 0][i % 8];
        rt.step_action(action, safe, |a| Ok(advance(&mut hidden, a))).unwrap();
        if rt.learner().report().shadow_pending {
            return (rt, hidden);
        }
    }
    panic!("fixture never created an audited prospective trial");
}

#[test]
fn pending_trials_and_final_decisions_survive_every_step_round_trip() {
    // Vary the observed action history, not a hard-coded learned program.
    let mut pending_snapshots = 0;
    let mut accepted_models = 0;
    for shift in 0..4 {
        let mut uninterrupted = runtime();
        let mut restored = runtime();
        let (mut first_device, mut second_device) = (0, 0);
        for i in 0..48 {
            let action = [1, 1, 0, 1, 0, 0, 1, 0][(i + shift) % 8];
            uninterrupted.step_action(action, safe, |a| Ok(advance(&mut first_device, a))).unwrap();
            restored.step_action(action, safe, |a| Ok(advance(&mut second_device, a))).unwrap();
            let checkpoint = restored.checkpoint().unwrap();
            pending_snapshots += usize::from(restored.learner().report().shadow_pending);
            let decoded = Learner::from_checkpoint(&checkpoint).unwrap();
            assert_eq!(decoded.checkpoint().unwrap(), checkpoint);
            restored.restore_cognition(&checkpoint).unwrap();
            assert_eq!(uninterrupted.checkpoint().unwrap(), restored.checkpoint().unwrap(),
                "restoration changed cognition at shift={shift}, step={i}");
            assert_eq!(first_device, second_device);
        }
        accepted_models += restored.learner().report().promotions;
    }
    assert!(pending_snapshots > 0, "test must include incomplete audits");
    assert!(accepted_models > 0, "test must cross actual model installations");
}

#[test]
fn autonomous_actions_and_forecasts_are_identical_after_round_trips() {
    let mut uninterrupted = runtime();
    let mut restored = runtime();
    let (mut first_device, mut second_device) = (0, 0);
    let mut pending_snapshots = 0;
    for i in 0..64 {
        let first = uninterrupted.step(None, safe, |a| Ok(advance(&mut first_device, a))).unwrap();
        let second = restored.step(None, safe, |a| Ok(advance(&mut second_device, a))).unwrap();
        assert_eq!(first, second, "autonomous action diverged at step={i}");
        pending_snapshots += usize::from(restored.learner().report().shadow_pending);
        let checkpoint = restored.checkpoint().unwrap();
        restored.restore_cognition(&checkpoint).unwrap();
        // Full equality includes PRE forecasts, costs, consumed trials, learning
        // traces, accepted/rejected promotions and the learned program state.
        assert_eq!(uninterrupted.checkpoint().unwrap(), restored.checkpoint().unwrap(),
            "autonomous cognition diverged at step={i}");
    }
    assert!(pending_snapshots > 0);
    assert!(restored.learner().report().promotions > 0);
    assert!(!restored.learner().program().cells.is_empty());
    assert!(restored.learner().report().last_error.is_none());
}

#[test]
fn pending_trial_checkpoint_cannot_clear_external_fault_or_replay_action() {
    let (mut rt, _) = with_pending_trial();
    let before_fault = rt.checkpoint().unwrap();
    let events = rt.learner().records().len();
    let mut calls = 0;
    let result = rt.step_action(1, safe, |_| {
        calls += 1;
        Err(Error::Execution("simulated unknown external result".into()))
    });
    assert!(result.is_err());
    assert_eq!(calls, 1);
    assert!(rt.learner().report().poisoned);
    assert_eq!(rt.learner().records().len(), events);
    assert_eq!(rt.restore_cognition(&before_fault), Err(Error::Stale));
    let after_fault = rt.checkpoint().unwrap();
    assert!(Learner::from_checkpoint(&after_fault).unwrap().report().poisoned);
    rt.restore_cognition(&after_fault).unwrap();
    assert!(rt.step_action(1, safe, |_| {
        calls += 1;
        Ok(ExternalPost { observation: vec![0], cost: 1 })
    }).is_err());
    assert_eq!(calls, 1, "an uncertain external action must never be replayed");
    assert_eq!(rt.learner().records().len(), events);
}

#[test]
fn pending_cognitive_trial_does_not_restore_emergency_stop_authority() {
    let (mut rt, _) = with_pending_trial();
    let checkpoint = rt.checkpoint().unwrap();
    let events = rt.learner().records().len();
    let mut calls = 0;
    assert_eq!(rt.step_action(0, |a| {
        let mut evidence = safe(a);
        evidence.emergency_stop = true;
        evidence
    }, |_| {
        calls += 1;
        Ok(ExternalPost { observation: vec![0], cost: 1 })
    }), Err(Error::Blocked));
    rt.restore_cognition(&checkpoint).unwrap();
    assert!(rt.organism().human_protection_emergency_latched());
    assert!(rt.learner().report().shadow_pending);
    assert_eq!(rt.step_action(0, safe, |_| {
        calls += 1;
        Ok(ExternalPost { observation: vec![0], cost: 1 })
    }), Err(Error::Blocked));
    assert_eq!(calls, 0);
    assert_eq!(rt.learner().records().len(), events);
}
