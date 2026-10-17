use aeterna_v1::construction::{Config,ExternalPost,Learner,ProtectedConstructionRuntime};
use aeterna_v1::construction::epistemic::EvidencePolicy;
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::vm::{Error,Expr};
use aeterna_v1::{EvoConfig,EvoPhase,HumanProtectionEvidence};

fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
fn runtime(enabled:bool)->ProtectedConstructionRuntime {
    let config=Config {minimum_training:32,search_every:32,shadow_steps:4,journal_capacity:2048,
        search:SearchConfig{work:50_000,max_expr_nodes:1,max_state:0,max_updates:1,..Default::default()},..Default::default()};
    let mut r=ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig{sensory_cells:1,motor_cells:2,..Default::default()}),config,vec![0]).unwrap();
    if enabled {r.enable_epistemic(EvidencePolicy::default()).unwrap();}r
}
fn feed(r:&mut ProtectedConstructionRuntime,start:usize,end:usize) {
    for i in start..end {r.step_action(0,safe,|_|Ok(ExternalPost{observation:vec![i64::from(i%5!=0)],cost:1})).unwrap();}
}
#[test]
fn stochastic_revision_installs_without_erasing_the_incumbents_lucky_zero() {
    let mut old=runtime(false);let mut new=runtime(true);feed(&mut old,0,256);feed(&mut new,0,256);
    assert_eq!(old.learner().report().promotions,0);
    assert!(new.learner().report().promotions>0,"{:?}",new.learner().epistemic_report());
    assert_eq!(new.learner().program().outputs,[Expr::integer(1)]);
    assert_eq!(new.learner().records()[0].transition.post,[0]);
    assert_eq!(new.learner().records().len(),old.learner().records().len());
    let report=new.learner().epistemic_report().unwrap();
    let installed=report.outcomes.iter().find(|o|o.installed).unwrap();
    assert!(installed.evidence.samples>4);assert!(installed.evidence.crossed);
    assert!(report.allocated_alpha<=report.policy.total_alpha);
    assert!(new.learner().registry().meta_ledger().reports().iter().any(|r|r.model_accepted));
}
#[test]
fn unfinished_evidence_survives_beyond_legacy_window_and_roundtrip() {
    let mut r=runtime(true);feed(&mut r,0,48);
    let report=r.learner().epistemic_report().unwrap();assert_eq!(report.pending.as_ref().unwrap().samples,16);
    let bytes=r.checkpoint().unwrap();let restored=Learner::from_checkpoint(&bytes).unwrap();
    assert_eq!(restored.checkpoint().unwrap(),bytes);assert_eq!(restored.epistemic_report(),Some(report));
    r.restore_cognition(&bytes).unwrap();feed(&mut r,48,256);assert!(r.learner().report().promotions>0);
}
#[test]
fn frequent_restores_have_identical_evidence_and_decisions() {
    let mut a=runtime(true);let mut b=runtime(true);
    for i in 0..160 {feed(&mut a,i,i+1);feed(&mut b,i,i+1);let bytes=b.checkpoint().unwrap();b.restore_cognition(&bytes).unwrap();assert_eq!(a.checkpoint().unwrap(),b.checkpoint().unwrap());}
}
#[test]
fn enabling_again_or_after_experience_cannot_reset_risk() {
    let mut r=runtime(true);assert_eq!(r.enable_epistemic(EvidencePolicy::default()),Err(Error::Busy));
    let mut r=runtime(false);feed(&mut r,0,1);assert_eq!(r.enable_epistemic(EvidencePolicy::default()),Err(Error::Busy));
}
#[test]
fn deterministic_contract_cannot_be_relabelled_stochastic() {
    let config=Config{search:SearchConfig{deterministic:true,..Default::default()},..Default::default()};
    let mut r=ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig{sensory_cells:1,motor_cells:2,..Default::default()}),config,vec![0]).unwrap();
    assert_eq!(r.enable_epistemic(EvidencePolicy::default()),Err(Error::InvalidData));
}
#[test]
fn declared_hard_output_is_not_revised() {
    let mut r=runtime(false);r.enable_epistemic(EvidencePolicy{protected_outputs:vec![0],..Default::default()}).unwrap();feed(&mut r,0,128);
    assert_eq!(r.learner().report().promotions,0);assert_eq!(r.learner().program().outputs,[Expr::Input(0)]);
    assert_eq!(r.learner().epistemic_report().unwrap().attempts,[0]);
}
#[test]
fn blocked_action_neither_creates_evidence_nor_clears_the_shield() {
    let mut r=runtime(true);feed(&mut r,0,48);let before=r.checkpoint().unwrap();let mut calls=0;
    let result=r.step_action(0,|a|{let mut e=safe(a);e.emergency_stop=true;e},|_|{calls+=1;Ok(ExternalPost{observation:vec![1],cost:1})});
    assert_eq!(result,Err(Error::Blocked));assert_eq!(calls,0);assert_eq!(before,r.checkpoint().unwrap());
    r.restore_cognition(&before).unwrap();assert!(r.organism().human_protection_emergency_latched());
}
#[test]
fn external_callback_fault_cannot_be_replayed_to_manufacture_evidence() {
    let mut r=runtime(true);feed(&mut r,0,48);let report=r.learner().epistemic_report();let bytes=r.checkpoint().unwrap();
    assert!(r.step_action(0,safe,|_|Err(Error::Execution("unknown external outcome".into()))).is_err());
    assert!(r.learner().report().poisoned);assert_eq!(report,r.learner().epistemic_report());
    assert_eq!(r.restore_cognition(&bytes),Err(Error::Stale));
    assert!(r.step_action(0,safe,|_|panic!("must not execute again")).is_err());
}
#[test]
fn old_checkpoint_cannot_rewind_allocated_risk_or_factual_history() {
    let mut r=runtime(true);let bytes=r.checkpoint().unwrap();feed(&mut r,0,48);
    assert_eq!(r.restore_cognition(&bytes),Err(Error::Stale));
    assert_eq!(r.learner().epistemic_report().unwrap().attempts,[1]);assert_eq!(r.learner().records().len(),48);
}
#[test]
fn short_explicit_budget_records_inconclusive_without_installation() {
    let mut r=runtime(false);r.enable_epistemic(EvidencePolicy{max_samples:4,..Default::default()}).unwrap();feed(&mut r,0,48);
    let report=r.learner().epistemic_report().unwrap();assert_eq!(report.outcomes.len(),1);
    assert!(!report.outcomes[0].installed);assert!(!report.outcomes[0].evidence.crossed);
    assert_eq!(report.outcomes[0].evidence.samples,4);assert_eq!(report.attempts,[1]);
}
