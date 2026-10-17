use aeterna_v1::construction::{Config,ExternalPost,ProtectedConstructionRuntime,Learner};
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::vm::{Error,Expr};
use aeterna_v1::{EvoConfig,EvoPhase,HumanProtectionEvidence};
fn safe(_:usize)->HumanProtectionEvidence {HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
fn make(budget:u64)->ProtectedConstructionRuntime {
    let cfg=Config{minimum_training:8,search_every:4,shadow_steps:4,journal_capacity:1024,search:SearchConfig{work:200_000,max_expr_nodes:1,max_state:0,deterministic:true,..Default::default()},..Default::default()};
    let mut r=ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig{sensory_cells:1,motor_cells:2,..Default::default()}),cfg,vec![0]).unwrap();
    r.enable_autonomy(AutonomyPolicy{ceiling:SearchConfig{work:2_000_000,max_expr_nodes:7,max_frontier:32768,max_updates:24,max_state:2,deterministic:true},failures_per_expansion:1,max_external_cost:budget,evidence:None}).unwrap();r
}
fn offers()->Vec<ActionOffer>{vec![ActionOffer{available:true,max_cost:1};2]}
#[test] fn cold_lifetime_expands_itself_and_acquires_missing_memory(){
    let mut r=make(1000);let mut hidden=0;
    for _ in 0..128{r.step_autonomous(None,&offers(),safe,|a|{let y=hidden;hidden=a as i64;Ok(ExternalPost{observation:vec![y],cost:1})}).unwrap();}
    assert!(r.learner().autonomy_report().unwrap().levels[0]>0);assert!(r.learner().report().state_cells>0);assert!(r.learner().report().promotions>0);
    let b=r.checkpoint().unwrap();assert_eq!(Learner::from_checkpoint(&b).unwrap().checkpoint().unwrap(),b);
}
#[test] fn no_available_action_means_no_side_effect_and_no_experience(){
    let mut r=make(10);let before=r.checkpoint().unwrap();assert_eq!(r.step_autonomous(None,&[ActionOffer{available:false,max_cost:1};2],safe,|_|panic!("forbidden")),Err(Error::Blocked));assert_eq!(before,r.checkpoint().unwrap());
}
#[test] fn model_independent_exploration_respects_the_action_mask(){
    let mut r=make(100);let os=[ActionOffer{available:false,max_cost:0},ActionOffer{available:true,max_cost:1}];
    for _ in 0..40{assert_eq!(r.step_autonomous(None,&os,safe,|a|{assert_eq!(a,1);Ok(ExternalPost{observation:vec![0],cost:1})}).unwrap(),1);}
}
#[test] fn actual_external_spending_cannot_be_refunded_by_a_checkpoint(){
    let mut r=make(2);let old=r.checkpoint().unwrap();for _ in 0..2{r.step_autonomous(None,&offers(),safe,|_|Ok(ExternalPost{observation:vec![0],cost:1})).unwrap();}
    assert_eq!(r.restore_cognition(&old),Err(Error::Stale));assert_eq!(r.step_autonomous(None,&offers(),safe,|_|panic!("budget")),Err(Error::Blocked));assert_eq!(r.learner().autonomy_report().unwrap().external_cost,2);
}
#[test] fn dishonest_cost_is_recorded_and_stops_further_execution(){
    let mut r=make(100);assert!(r.step_autonomous(None,&offers(),safe,|_|Ok(ExternalPost{observation:vec![1],cost:5})).is_err());assert!(r.learner().report().poisoned);assert_eq!(r.learner().records().len(),1);assert_eq!(r.learner().records()[0].external_cost,5);assert!(r.step_autonomous(None,&offers(),safe,|_|panic!("poisoned")).is_err());
}
#[test] fn emergency_stop_is_not_an_exploration_exception(){
    let mut r=make(100);assert_eq!(r.step_autonomous(None,&offers(),|a|{let mut e=safe(a);e.emergency_stop=true;e},|_|panic!("stop")),Err(Error::Blocked));assert!(r.organism().human_protection_emergency_latched());assert!(r.learner().records().is_empty());
}
#[test] fn frozen_probe_cannot_change_model_library_or_acquired_search(){
    let mut r=make(1000);for _ in 0..32{r.step_autonomous(None,&offers(),safe,|a|Ok(ExternalPost{observation:vec![a as i64],cost:1})).unwrap();}
    r.freeze_learning().unwrap();let p=r.learner().program().clone();let l=r.learner().library().clone();let g=serde_json::to_vec(r.learner().registry()).unwrap();let c=serde_json::to_vec(&r.learner().curriculum()).unwrap();
    for _ in 0..16{r.step_autonomous(None,&offers(),safe,|_|Ok(ExternalPost{observation:vec![-9],cost:1})).unwrap();}
    assert_eq!(r.learner().program(),&p);assert_eq!(r.learner().library(),&l);assert_eq!(serde_json::to_vec(r.learner().registry()).unwrap(),g);assert_eq!(serde_json::to_vec(&r.learner().curriculum()).unwrap(),c);assert_eq!(r.learner().autonomy_report().unwrap().frozen_at,Some(32));let b=r.checkpoint().unwrap();r.restore_cognition(&b).unwrap();assert_eq!(b,r.checkpoint().unwrap());
}
#[test] fn policy_cannot_be_reset_or_partially_enabled_after_error(){
    let mut r=make(100);let before=r.checkpoint().unwrap();assert_eq!(r.enable_autonomy(AutonomyPolicy::default()),Err(Error::Busy));assert_eq!(before,r.checkpoint().unwrap());
    let mut r=ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig{sensory_cells:1,motor_cells:2,..Default::default()}),Config::default(),vec![0]).unwrap();let before=r.checkpoint().unwrap();let mut bad=AutonomyPolicy::default();bad.max_external_cost=0;assert!(r.enable_autonomy(bad).is_err());assert_eq!(before,r.checkpoint().unwrap());r.enable_autonomy(AutonomyPolicy::default()).unwrap();
}
#[test] fn protected_noisy_output_never_becomes_a_search_obligation(){
    let mut r=ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig{sensory_cells:1,motor_cells:2,..Default::default()}),Config::default(),vec![0]).unwrap();let mut p=AutonomyPolicy::default();p.evidence.as_mut().unwrap().protected_outputs=vec![0];r.enable_autonomy(p).unwrap();
    for t in 0..32{r.step_autonomous(None,&offers(),safe,|_|Ok(ExternalPost{observation:vec![t],cost:1})).unwrap();}
    assert!(r.learner().curriculum().unwrap().history.is_empty());assert_eq!(r.learner().program().outputs,[Expr::Input(0)]);
}
#[test] fn joint_state_search_can_cross_a_useless_single_register(){
    use aeterna_v1::construction::synthesis::{Transition,synthesize_requested};use aeterna_v1::construction::vm::{Library,Program};
    let mut data=Vec::new();let mut input=vec![0];let mut memory=[0,0];
    for (i,a) in [1,1,0,0,1,0,1,1,1,0,1,0,0,0,1,1].into_iter().cycle().take(64).enumerate(){let post=vec![memory[1]];memory=[a as i64,memory[0]];data.push(Transition{id:i as u64,restart:i==0,pre:input,action:a,post:post.clone()});input=post;}
    let cfg=SearchConfig{work:8_000_000,max_expr_nodes:1,max_state:2,max_updates:16,deterministic:true,..Default::default()};let trace=synthesize_requested(&Program::identity(1),&data,&Library::default(),&cfg,&[],Some(0)).unwrap();assert_eq!(trace.training_loss,0.0,"{trace:?}");let patch=trace.winner.unwrap();assert_eq!(patch.cells.len(),2);assert!(patch.cells.iter().any(|c|c.update==Expr::Action));
}
#[test] fn older_step_api_cannot_bypass_the_lifetime_cost_contract(){let mut r=make(100);assert!(r.step_action(0,safe,|_|panic!("no quote")).is_err());assert!(r.learner().records().is_empty());}
