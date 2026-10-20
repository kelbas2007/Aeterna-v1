//! Exposed protected runtime contract test: no invented learner labels or
//! future observations are supplied. The environment executes opaque motors.
//! This tests provenance/checkpoint wiring, not evaluator advantage or AGI.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence { human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false }
}
fn step(rt:&mut ScientificRuntime,world:&mut i64,allowed:usize,goal:i64){
    let offers=(0..2).map(|i|ActionOffer{available:i==allowed,max_cost:1})
        .collect::<Vec<_>>();
    let outcome=rt.step_constructing(Some(&[Some(goal)]),&offers,safe,|action|{
        if action==0 {*world+=1;}
        Ok(UnifiedPost{observation:vec![*world],cost:1,reward:0.0})
    }).unwrap();
    assert_eq!(outcome.action,allowed);
}
#[test]
fn only_protected_executed_post_builds_trajectory_and_checkpoint(){
    let cfg=Config{width:1,actions:2,minimum_training:128,
        search_every:4096,planning_work:4000,..Default::default()};
    let mut runtime=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),
        vec![0],"real-episode").unwrap();
    runtime.enable_constructing_trajectory_self_revision().unwrap();
    let mut world=0_i64;
    for _ in 0..8 {step(&mut runtime,&mut world,1,2);}
    assert_eq!(world,0);
    let r=runtime.trajectory_self_revision_report().unwrap();
    assert_eq!((r.episodes,r.steps),(1,8));
    assert_eq!(r.generations,0,"no approval from failure-only evidence");
    step(&mut runtime,&mut world,0,2);
    step(&mut runtime,&mut world,0,2);
    assert_eq!(world,2);
    let r=runtime.trajectory_self_revision_report().unwrap();
    assert_eq!((r.episodes,r.steps),(2,10));
    let checkpoint=runtime.checkpoint_constructing().unwrap();
    let mut restored=ScientificRuntime::from_constructing_checkpoint(&checkpoint).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(),checkpoint);
    let restored_report=restored.trajectory_self_revision_report().unwrap();
    assert_eq!((restored_report.episodes,restored_report.steps),(2,10));
    step(&mut restored,&mut world,0,4);
    step(&mut restored,&mut world,0,4);
    let new_report=restored.trajectory_self_revision_report().unwrap();
    assert_eq!((new_report.episodes,new_report.steps),(3,12));
    assert_eq!(world,4);
}
#[test]
fn human_protection_veto_never_enters_revision_experience(){
    let cfg=Config{width:1,actions:2,minimum_training:128,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),
        vec![0],"safety-veto").unwrap();
    rt.enable_constructing_trajectory_self_revision().unwrap();
    let offers=vec![ActionOffer{available:true,max_cost:1};2];
    let mut called=false;
    let result=rt.step_constructing(Some(&[Some(1)]),&offers,|_|HumanProtectionEvidence{
        human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:0.0,emergency_stop:true,
    },|_|{
        called=true;
        Ok(UnifiedPost{observation:vec![1],cost:1,reward:0.0})
    });
    assert!(result.is_err());assert!(!called);
    let r=rt.trajectory_self_revision_report().unwrap();
    assert_eq!((r.episodes,r.steps),(0,0));
}
