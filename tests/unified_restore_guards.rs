use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,RuntimeError};
use aeterna_v1::construction::{Config,Error};
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::HumanProtectionEvidence;
fn safe(_:usize)->HumanProtectionEvidence {HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
fn config()->Config{Config{minimum_training:32,journal_capacity:512,..Default::default()}}
fn make()->ScientificRuntime{ScientificRuntime::new_constructing(config(),AutonomyPolicy::default(),vec![0],"numeric-v1").unwrap()}
fn step(r:&mut ScientificRuntime){let x=r.construction_learner().unwrap().current()[0];r.step_constructing(None,&[ActionOffer{available:true,max_cost:1};2],safe,|a|Ok(UnifiedPost{observation:vec![x+a as i64],cost:1,reward:0.25})).unwrap();}

#[test]
fn generic_restart_rejected_without_mutating_combined_cognition() {
    let mut r = make();
    for _ in 0..8 { step(&mut r); }
    let before = r.checkpoint_constructing().unwrap();
    assert_eq!(r.restart_cognition(), Err(RuntimeError::ConstructionModeRequired));
    assert_eq!(r.checkpoint_constructing().unwrap(), before);
    ScientificRuntime::from_constructing_checkpoint(&before).unwrap();
    step(&mut r);
    assert_eq!(r.construction_learner().unwrap().records().len(), 9);
}

#[test]
fn generic_native_restore_rejected_before_replacing_carrier() {
    let mut r = make();
    for _ in 0..8 { step(&mut r); }
    let native = r.organism().phase_native_checkpoint().unwrap();
    let before = r.checkpoint_constructing().unwrap();
    assert_eq!(r.restore_native_checkpoint(native), Err(RuntimeError::ConstructionModeRequired));
    assert_eq!(r.checkpoint_constructing().unwrap(), before);
    let mut restored = ScientificRuntime::from_constructing_checkpoint(&before).unwrap();
    step(&mut r); step(&mut restored);
    assert_eq!(r.checkpoint_constructing().unwrap(), restored.checkpoint_constructing().unwrap());
}

#[test]
fn generic_online_restore_rejected_at_mode_boundary() {
    let mut r = make();
    step(&mut r);
    let before = r.checkpoint_constructing().unwrap();
    assert_eq!(r.restore_online_checkpoint(b"{}"), Err(RuntimeError::ConstructionModeRequired));
    assert_eq!(r.checkpoint_constructing().unwrap(), before);
    assert_eq!(r.restore_online_checkpoint(b"not a checkpoint"), Err(RuntimeError::ConstructionModeRequired));
    assert_eq!(r.checkpoint_constructing().unwrap(), before);
}

#[test]
fn generic_restore_cannot_clear_a_constructing_emergency_latch() {
    let mut r = make();
    let native = r.organism().phase_native_checkpoint().unwrap();
    let result = r.step_constructing(None, &[ActionOffer { available: true, max_cost: 1 }; 2],
        |a| { let mut h=safe(a); h.emergency_stop=true; h }, |_| panic!("blocked"));
    assert!(matches!(result, Err(Error::Blocked)));
    let before = r.checkpoint_constructing().unwrap();
    assert_eq!(r.restart_cognition(), Err(RuntimeError::ConstructionModeRequired));
    assert_eq!(r.restore_native_checkpoint(native), Err(RuntimeError::ConstructionModeRequired));
    assert_eq!(r.restore_online_checkpoint(b"{}"), Err(RuntimeError::ConstructionModeRequired));
    assert!(r.emergency_latched());
    assert_eq!(r.checkpoint_constructing().unwrap(), before);
    assert!(ScientificRuntime::from_constructing_checkpoint(&before).unwrap().emergency_latched());
}
