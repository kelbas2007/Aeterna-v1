use aeterna_v1::construction::search_policy::*;
use aeterna_v1::construction::vm::{Expr,Error,Library,Op,Value,Meter};
use aeterna_v1::scientific_runtime::ScientificRuntime;
use aeterna_v1::construction::{Config,autonomy::AutonomyPolicy};
fn limits()->SearchLimits{SearchLimits{work:20_000,evaluations:64,max_nodes:9,frontier:64,edits_per_expansion:12}}
fn task(offset:i64)->SynthesisTask{SynthesisTask{examples:(1..=6).map(|x|Example{input:vec![x+offset,2*x-offset],output:3*x}).collect(),library:Library::default()}}
fn config()->EvolutionConfig{EvolutionConfig{object:limits(),meta:SearchLimits{work:5_000_000,evaluations:12,max_nodes:15,frontier:32,edits_per_expansion:8},max_lifetime_work:10_000_000}}
fn gate()->Vec<SynthesisTask>{(1..=8).map(|x|task(x*20)).collect()}
#[test] fn code_roundtrip_includes_rank_operator_and_location(){let p=SearchPolicy::default();assert_eq!(SearchPolicy::from_code(&p.code()).unwrap(),p);}
#[test] fn arbitrary_code_record_is_not_a_policy(){assert!(SearchPolicy::from_code(&Expr::integer(1)).is_err());}
#[test] fn priority_cannot_read_outside_public_features(){let p=SearchPolicy{priority:Expr::Input(FEATURES),..Default::default()};assert!(p.validate().is_err());}
#[test] fn all_three_functions_are_validated(){for i in 0..2{let mut p=SearchPolicy::default();if i==0{p.variation=Expr::State(0)}else{p.location=Expr::Hole{id:0,ty:aeterna_v1::construction::vm::Ty::Int}};assert!(p.validate().is_err());}}
#[test] fn policy_cannot_embed_unknown_library_or_action_authority(){for e in [Expr::Call(99,vec![]),Expr::Action,Expr::Arg(0)]{let p=SearchPolicy{priority:e,..Default::default()};assert!(p.validate().is_err());}}
#[test] fn policy_requires_integer_outputs(){let p=SearchPolicy{priority:Expr::Lit(Value::Bool(true)),..Default::default()};assert_eq!(p.validate(),Err(Error::Type));}
#[test] fn shape_growth_is_bounded(){let mut p=SearchPolicy::default();for _ in 0..40{p.priority=Expr::bin(Op::Add,p.priority,Expr::integer(1));}assert!(p.validate().is_err());}
#[test] fn oversized_literals_rejected_without_abs_overflow(){let p=SearchPolicy{priority:Expr::integer(i64::MIN),..Default::default()};assert!(p.validate().is_err());}
#[test] fn all_work_caps_checked(){for work in [0,100_000_001]{let l=SearchLimits{work,..limits()};assert!(l.validate().is_err());}}
#[test] fn no_zero_sized_search_or_bad_width(){let mut t=task(0);t.examples[0].input.clear();assert!(t.validate().is_err());let l=SearchLimits{frontier:0,..limits()};assert!(l.validate().is_err());}
#[test] fn search_is_deterministic_and_reports_its_actual_producer(){let p=SearchPolicy::default();let a=search(&p,Expr::integer(0),&mut task(0),&limits(),51).unwrap();let b=search(&p,Expr::integer(0),&mut task(0),&limits(),51).unwrap();assert_eq!(a,b);assert_eq!(a.policy_id,p.fingerprint());assert!(a.work<=limits().work);assert!(a.objective_work+a.priority_work<=a.work);}
#[test] fn root_replacement_is_executable_not_hole_specialization(){let old=Expr::bin(Op::Add,Expr::Input(0),Expr::integer(2));let edit=TreeEdit{site:0,replacement:Expr::bin(Op::Mul,Expr::Input(1),Expr::integer(3))};assert_eq!(edit.apply(&old).unwrap(),edit.replacement);}
#[test] fn every_winner_replays_its_full_edit_chain(){let r=search(&SearchPolicy::default(),Expr::integer(0),&mut task(0),&limits(),8).unwrap();let mut e=Expr::integer(0);for edit in &r.edits_to_winner{e=edit.apply(&e).unwrap();}assert_eq!(e,r.program);}
#[test] fn invalid_edit_location_is_not_silently_accepted(){assert!(TreeEdit{site:2,replacement:Expr::integer(1)}.apply(&Expr::integer(0)).is_err());}
#[test] fn reexecuted_candidate_matches_recorded_loss(){let mut t=task(0);let r=search(&SearchPolicy::default(),Expr::integer(0),&mut t,&limits(),18).unwrap();assert_eq!(t.score(&r.program,&mut Meter::new(10000)).unwrap(),r.loss);}
#[test] fn meta_search_really_executes_policy_as_code(){let p=SearchPolicy::default();let r=produce(&p,&p,&[task(0)],&limits(),&config().meta,7).unwrap();assert!(SearchPolicy::from_code(&r.program).is_ok());assert!(r.evaluations>1);assert!(r.objective_work>0);assert_eq!(r.policy_id,p.fingerprint());}
#[test] fn proposal_serialization_and_generation_validation(){let mut e=RecursiveSearch::new(config()).unwrap();e.improve(&[task(0)],&gate(),19).unwrap();e.validate().unwrap();let b=serde_json::to_vec(&e).unwrap();let r:RecursiveSearch=serde_json::from_slice(&b).unwrap();r.validate().unwrap();assert_eq!(b,serde_json::to_vec(&r).unwrap());}
#[test] fn fabricated_generation_fails_even_after_reserialization(){let e=RecursiveSearch::new(config()).unwrap();let mut j=serde_json::to_value(e).unwrap();j["generation"]=serde_json::json!(9);let f:RecursiveSearch=serde_json::from_value(j).unwrap();assert!(f.validate().is_err());}
#[test] fn acceptance_requires_actual_execution_not_caller_boolean(){let mut e=RecursiveSearch::new(config()).unwrap();e.improve(&[task(0)],&gate(),17).unwrap();let mut j=serde_json::to_value(&e).unwrap();let old=j["audits"][0]["accepted"].as_bool().unwrap();j["audits"][0]["accepted"]=serde_json::json!(!old);let f:RecursiveSearch=serde_json::from_value(j).unwrap();assert!(f.validate().is_err());}
#[test] fn gate_cannot_be_reused_or_become_training(){let mut e=RecursiveSearch::new(config()).unwrap();e.improve(&[task(0)],&gate(),1).unwrap();let before=e.clone();assert_eq!(e.improve(&[task(0)],&gate(),2).unwrap_err(),Error::Stale);assert_eq!(e,before);let other:Vec<_>=(1..=8).map(|x|task(300+x)).collect();assert_eq!(e.improve(&[task(20)],&other,3).unwrap_err(),Error::Stale);}
#[test] fn changing_labels_does_not_reopen_gate(){let mut e=RecursiveSearch::new(config()).unwrap();let mut g=gate();e.improve(&[task(0)],&g,1).unwrap();for t in &mut g{t.examples[0].output+=1;}assert_eq!(e.improve(&[task(0)],&g,2).unwrap_err(),Error::Stale);}
#[test] fn duplicate_or_overlapping_validation_inputs_rejected(){let mut e=RecursiveSearch::new(config()).unwrap();let g=vec![task(0);8];assert_eq!(e.improve(&[task(0)],&g,1).unwrap_err(),Error::Stale);}
#[test] fn freeze_prevents_generation_or_risk_reset(){let mut e=RecursiveSearch::new(config()).unwrap();e.freeze();let before=e.clone();assert_eq!(e.improve(&[task(0)],&gate(),1).unwrap_err(),Error::Blocked);assert_eq!(e,before);}
#[test] fn insufficient_lifetime_budget_cannot_install_or_spend(){let mut c=config();c.max_lifetime_work=1;let mut e=RecursiveSearch::new(c).unwrap();let before=e.clone();assert_eq!(e.improve(&[task(0)],&gate(),1).unwrap_err(),Error::Budget);assert_eq!(e,before);}
#[test] fn changing_mutation_or_location_changes_real_search(){let p=SearchPolicy::default();let mut q=p.clone();q.variation=Expr::integer(1);q.location=Expr::integer(0);let a=search(&p,Expr::integer(0),&mut task(0),&limits(),42).unwrap();let b=search(&q,Expr::integer(0),&mut task(0),&limits(),42).unwrap();assert_ne!(a.trace_hash,b.trace_hash);}
fn runtime()->ScientificRuntime{ScientificRuntime::new_constructing(Config::default(),AutonomyPolicy::default(),vec![0],"RSP1-test").unwrap()}
#[test] fn runtime_owns_search_state_and_preserves_prior_cognition(){let mut r=runtime();let prior=r.construction_learner().unwrap().program().clone();r.enable_recursive_search(config()).unwrap();r.improve_recursive_search(&[task(0)],&gate(),4).unwrap();assert_eq!(&prior,r.construction_learner().unwrap().program());assert!(r.unified_report().unwrap().computation_work>0);let bytes=r.checkpoint_constructing().unwrap();let s=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();assert_eq!(bytes,s.checkpoint_constructing().unwrap());}
#[test] fn frozen_runtime_blocks_retraining(){let mut r=runtime();r.enable_recursive_search(config()).unwrap();r.freeze_constructing().unwrap();assert!(r.improve_recursive_search(&[task(0)],&gate(),3).is_err());assert!(r.construction_learner().unwrap().recursive_search().unwrap().is_frozen());}
#[test] fn pure_search_does_not_create_external_events_or_permissions(){let mut r=runtime();r.enable_recursive_search(config()).unwrap();let before=r.construction_learner().unwrap().records().len();r.solve_by_recursive_search(&task(0),&limits(),1).unwrap();assert_eq!(before,r.construction_learner().unwrap().records().len());assert_eq!(r.unified_report().unwrap().total_external_cost,0);}
#[test] fn live_restore_cannot_refund_pure_search_work(){let mut r=runtime();r.enable_recursive_search(config()).unwrap();let bytes=r.checkpoint_constructing().unwrap();r.solve_by_recursive_search(&task(0),&limits(),1).unwrap();assert_eq!(r.restore_constructing(&bytes),Err(Error::Stale));}
#[test] fn legacy_checkpoint_with_no_recursive_state_is_unchanged(){let r=runtime();let b=r.checkpoint_constructing().unwrap();let text=String::from_utf8(b.clone()).unwrap();assert!(!text.contains("recursive_search"));assert_eq!(b,ScientificRuntime::from_constructing_checkpoint(&b).unwrap().checkpoint_constructing().unwrap());}

#[test] fn existing_constructor_executes_the_opted_in_policy_and_keeps_prospective_gate(){
    use aeterna_v1::construction::Learner;
    use aeterna_v1::construction::synthesis::Transition;
    let mut cfg=Config::default();cfg.minimum_training=8;cfg.search.max_expr_nodes=9;
    let mut learner=Learner::new(cfg.clone(),vec![0]).unwrap();learner.enable_recursive_search(config()).unwrap();
    // The registry's public trial path is the same path used by construct().
    let mut registry=learner.registry().clone();
    let data:Vec<_>=(0..16).map(|i|Transition{id:i,restart:true,pre:vec![i as i64],action:(i%2) as usize,post:vec![i as i64+(i%2) as i64]}).collect();
    let trial=registry.begin_trial(learner.program(),&data,learner.library(),&cfg.search).unwrap();
    assert_eq!(trial.trace().search_policy_id,Some(SearchPolicy::default().fingerprint()));
    assert!(!trial.trace().prospectively_accepted);
    registry.cancel_trial(trial).unwrap();
}
#[test] fn empty_objective_is_rejected_before_indexing(){let mut t=SynthesisTask{examples:vec![],library:Library::default()};assert!(search(&SearchPolicy::default(),Expr::integer(0),&mut t,&limits(),0).is_err());}
