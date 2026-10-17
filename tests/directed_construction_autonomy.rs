use aeterna_v1::construction::{Config, ExternalPost, ProtectedConstructionRuntime};
use aeterna_v1::construction::curriculum::SelectionMode;
use aeterna_v1::construction::synthesis::*;
use aeterna_v1::construction::vm::*;
use aeterna_v1::{EvoConfig,EvoPhase,HumanProtectionEvidence};
fn safe(_:usize)->HumanProtectionEvidence { HumanProtectionEvidence { human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false } }
#[test]
fn target_perfect_search_terminates_despite_unrelated_error() {
    let data:Vec<_>=(0..16).map(|i|Transition{id:i,restart:true,pre:vec![i as i64,0],action:0,post:vec![i as i64+7,if i%2==0 {10000}else{-10000}]}).collect();
    let p=Program::identity(2);let lib=Library::default();let cfg=SearchConfig::default();
    let trace=synthesize_requested(&p,&data,&lib,&cfg,&[],Some(0)).unwrap();
    let q=trace.winner.unwrap().apply(&p,&lib).unwrap();let rows=replay(&q,&data,&lib,&mut Meter::new(100000)).unwrap().0;
    assert!(rows.iter().zip(&data).all(|(p,t)|p[0]==t.post[0]));
    assert!(trace.training_loss>0.0);assert!(trace.work<cfg.work/4);
    assert_eq!(q.outputs[1],p.outputs[1]);assert_eq!(q.cells,p.cells);
}
#[test]
fn focused_hint_cannot_overwrite_existing_state_meaning() {
    let p=Program{outputs:vec![Expr::State(0)],cells:vec![StateCell{initial:0,update:Expr::Action}]};
    let data=vec![Transition{id:0,restart:true,pre:vec![0],action:1,post:vec![7]},Transition{id:1,restart:false,pre:vec![7],action:0,post:vec![7]}];
    let cfg=SearchConfig{work:10000,max_state:1,..Default::default()};let lib=Library::default();
    let hint=Hint{constructor_id:9,output:Expr::integer(7),cells:vec![StateCell{initial:7,update:Expr::integer(7)}]};
    let t=synthesize_requested(&p,&data,&lib,&cfg,&[hint],Some(0)).unwrap();
    assert_ne!(t.constructor_id,Some(9));if let Some(patch)=t.winner { assert_eq!(patch.cells,p.cells); }
}
#[test]
fn target_selection_is_shared_by_counterfactual_search_arms() {
    let data:Vec<_>=(0..8).map(|i|Transition{id:i,restart:true,pre:vec![i as i64,0],action:0,post:vec![i as i64+7,10000]}).collect();
    let mut registry=aeterna_v1::construction::constructors::Registry::default();
    let trial=registry.begin_requested_trial(&Program::identity(2),&data,&Library::default(),&SearchConfig::default(),Some(0)).unwrap();
    assert_eq!(trial.trace().job.target,0);registry.cancel_trial(trial).unwrap();
}
#[test]
fn autonomous_curriculum_is_preserved_with_pending_trials() {
    let cfg=Config{width:2,actions:2,minimum_training:4,shadow_steps:4,search_every:2,search:SearchConfig{work:100000,..Default::default()},..Default::default()};
    let mut r=ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig{sensory_cells:2,motor_cells:2,..Default::default()}),cfg,vec![0,0]).unwrap();
    r.enable_curriculum(SelectionMode::Progress).unwrap();
    for t in 0..24 {
        r.step(None,safe,|a|Ok(ExternalPost{observation:vec![a as i64,if t%3==0 {10000}else{-10000}],cost:1})).unwrap();
        let b=r.checkpoint().unwrap();r.restore_cognition(&b).unwrap();assert_eq!(b,r.checkpoint().unwrap());
    }
    assert_eq!(r.learner().records().len(),24);assert!(r.learner().curriculum().unwrap().history.len()>1);
}
#[test]
fn masked_goal_respects_dimensions_and_existing_permission_boundary() {
    let mut r=ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig{sensory_cells:1,motor_cells:2,..Default::default()}),Config::default(),vec![0]).unwrap();
    assert!(r.learner().choose_masked(Some(&[None,None])).is_err());
    let mut calls=0;
    let result=r.step_masked(&[Some(1)],|a|{let mut h=safe(a);h.emergency_stop=true;h},|_|{calls+=1;Ok(ExternalPost{observation:vec![1],cost:1})});
    assert_eq!(result,Err(Error::Blocked));assert_eq!(calls,0);assert!(r.learner().records().is_empty());
}
