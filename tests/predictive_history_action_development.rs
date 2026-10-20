//! Exposed engineering regression: does the learned HISTORY route execute?
//! No independent generalization or recursive self-improvement claim.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,UnifiedStep};
use serde_json::json;

fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
#[derive(Clone)]
struct World{value:i64,primed:bool,roles:[usize;4],broken:bool}
impl World {
    fn act(&mut self,a:usize){
        match self.roles.iter().position(|&id|id==a).unwrap(){
            0=>self.primed=!self.broken,
            1=>{if self.primed&&!self.broken{self.value+=1}self.primed=false;},
            2=>self.primed=false,
            3=>{self.value=0;self.primed=false;},
            _=>unreachable!(),
        }
    }
}
fn step(rt:&mut ScientificRuntime,w:&mut World,forced:Option<usize>,goal:Option<i64>)->UnifiedStep{
    let offers=(0..4).map(|a|ActionOffer{available:forced.is_none_or(|id|id==a),max_cost:1}).collect::<Vec<_>>();
    let goal=goal.map(|g|vec![Some(g)]);
    rt.step_constructing(goal.as_deref(),&offers,safe,|a|{
        w.act(a);Ok(UnifiedPost{observation:vec![w.value],cost:1,reward:0.0})
    }).unwrap()
}
fn acquire(roles:[usize;4],agency_aware:bool)->(ScientificRuntime,World){
    let cfg=Config{width:1,actions:4,minimum_training:128,search_every:4096,
        planning_depth:3,planning_work:120_000,journal_capacity:1024,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0],"history-action-regression").unwrap();
    if agency_aware {
        rt.enable_constructing_agency_aware_credit().unwrap();
    }
    let mut w=World{value:0,primed:false,roles,broken:false};
    // Exactly the already-exposed identification fixture, motor-permuted.
    // The world is only a real-POST callback, never a learner-side simulator.
    for i in 0..44 {
        step(&mut rt,&mut w,Some(roles[3]),Some(1));
        step(&mut rt,&mut w,Some(roles[if i%2==0{0}else{2}]),Some(1));
        step(&mut rt,&mut w,Some(roles[1]),Some(1));
    }
    step(&mut rt,&mut w,Some(roles[3]),Some(1));
    (rt,w)
}
fn enable(rt:&mut ScientificRuntime){
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
}
#[test]
fn prepared_context_executes_the_remaining_step_after_checkpoint(){
    // Agency credit is enabled ONCE before real factual identification.
    // This corrects policy-credit provenance, not the causal-rule grammar.
    let (mut rt,mut w)=acquire([0,1,2,3],true);
    // Disclosed controlled preparation through the normal protection gate.
    // This is not an autonomous-acquisition or start-to-finish claim.
    step(&mut rt,&mut w,Some(0),Some(1));
    enable(&mut rt);rt.freeze_constructing().unwrap();
    let cp=rt.checkpoint_constructing().unwrap();
    let mut rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    assert_eq!(rt.checkpoint_constructing().unwrap(),cp);
    let actual=step(&mut rt,&mut w,None,Some(1));
    eprintln!("HISTORY_REMAINDER {}",json!({"action":actual.action,"source":actual.source,
        "offered":actual.predictive_history_plan_proposed,"post":w.value,"fields":actual.selected_fields}));
    assert_eq!(actual.action,1,"history remainder must execute use, not prepare again");
    assert_eq!(actual.source,"predictive-history","history remainder must win native U1");
    assert_eq!(w.value,1,"history remainder must reach the real external goal");
}
#[test]
fn real_contradiction_retires_previously_supported_history_route(){
    let (mut rt,mut w)=acquire([0,1,2,3],false);
    enable(&mut rt);
    w.broken=true; // evaluator-only world change, not a learner instruction
    step(&mut rt,&mut w,Some(0),Some(1));
    step(&mut rt,&mut w,Some(1),Some(1)); // genuine contradictory POST
    assert_eq!(w.value,0);
    let next=step(&mut rt,&mut w,None,Some(1));
    assert!(!next.predictive_history_plan_proposed,
        "observed contradiction must prevent reissuing the disproved history plan");
    assert_ne!(next.source,"predictive-history");
}
#[test]
fn actual_denial_does_not_advance_the_history_or_execute_a_plan(){
    let (mut rt,mut w)=acquire([0,1,2,3],false);
    enable(&mut rt);
    let events=rt.construction_learner().unwrap().records().len();
    let offers=vec![ActionOffer{available:true,max_cost:1};4];
    let result=rt.step_constructing(Some(&[Some(1)]),&offers,|_|{
        let mut evidence=safe(0);evidence.emergency_stop=true;evidence
    },|a|{w.act(a);panic!("a blocked history proposal must not execute")});
    assert!(result.is_err());
    assert_eq!(rt.construction_learner().unwrap().records().len(),events);
    assert_eq!(w.value,0);
}
#[test]
fn report_unforced_closed_loop_for_all_opaque_role_permutations(){
    // Descriptive development cohort; a printed aggregate is not admission.
    // This test intentionally does NOT weaken a qualification threshold or
    // claim victory when candidate selections fail: complete traces are kept.
    let mut cases=Vec::new();let mut success=[0usize;2];let mut actions=[0usize;2];
    let mut selected_history=0usize;
    for a in 0..4 {for b in 0..4 {if b==a{continue}for c in 0..4 {
        if c==a||c==b{continue}let d=(0..4).find(|&d|d!=a&&d!=b&&d!=c).unwrap();
        let roles=[a,b,c,d];let (rt,w)=acquire(roles,false);let cp=rt.checkpoint_constructing().unwrap();
        let mut pair=Vec::new();
        for arm in 0..2 {
            let mut rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
            rt.enable_constructing_self_directed_control().unwrap();
            if arm==1 {rt.enable_constructing_predictive_history_states().unwrap();}
            let mut world=w.clone();let mut trace=Vec::new();
            // LIVE U1, not a hardcoded plan execution schedule or frozen motor.
            for _ in 0..24 {
                if world.value==1{break}
                let out=step(&mut rt,&mut world,None,Some(1));
                if arm==1&&out.source=="predictive-history"{selected_history+=1;}
                trace.push(json!({"action":out.action,"source":out.source,"post":world.value,
                    "history_offered":out.predictive_history_plan_proposed}));
            }
            success[arm]+=usize::from(world.value==1);actions[arm]+=trace.len();
            pair.push(json!({"solved":world.value==1,"actions":trace.len(),"trace":trace}));
        }
        cases.push(json!({"roles":roles,"baseline":pair[0],"history":pair[1]}));
    }}}
    eprintln!("HISTORY_ACTION_DEVELOPMENT {}",json!({"cases":cases,"case_count":24,
        "baseline_solved":success[0],"history_solved":success[1],"baseline_actions":actions[0],
        "history_actions":actions[1],"selected_history":selected_history,
        "independent_qualification":false}));
    assert_eq!(cases.len(),24);
}
