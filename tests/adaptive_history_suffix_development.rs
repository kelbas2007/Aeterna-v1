//! ADAPTIVE-HISTORY-SUFFIX1: EXPOSED mechanism-only development fixture.
//! All factual transitions are from real HumanProtection-authorized POSTs.
//! The environment's latent stage and motor meanings never enter cognition.
//! This is NOT a new independent transfer or self-improvement qualification.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,UnifiedStep};
use serde_json::json;

fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false}
}
#[derive(Clone)]
struct World{progress:i64,stage:u8,needs_second_order:bool,broken:bool}
impl World{
    fn new(needs_second_order:bool)->Self{
        Self{progress:0,stage:0,needs_second_order,broken:false}
    }
    fn execute(&mut self,motor:usize){
        match motor{
            0=>self.stage=1, // preparation A, invisible sensor outcome
            1=>self.stage=if self.stage==1{2}else{0}, // common final prep B
            2=>{
                if !self.broken && (!self.needs_second_order||self.stage==2){
                    self.progress+=1
                }
                self.stage=0;
            },
            3=>self.stage=0, // matched negative predecessor C
            4=>{self.progress=0;self.stage=0}, // physically executed reset
            _=>unreachable!(),
        }
    }
}
fn step(rt:&mut ScientificRuntime,w:&mut World,forced:Option<usize>)
    ->UnifiedStep{
    let offers=(0..5).map(|a|ActionOffer{
        available:forced.is_none_or(|id|id==a),max_cost:1
    }).collect::<Vec<_>>();
    let chosen=rt.step_constructing(
        Some(&[Some(1)]),&offers,safe,|motor|{
            w.execute(motor);
            Ok(UnifiedPost{observation:vec![w.progress],cost:1,reward:0.0})
        }).unwrap();
    if let Some(expected)=forced{assert_eq!(chosen.action,expected);}
    chosen
}
fn acquired(needs_second_order:bool)->(ScientificRuntime,World){
    let cfg=Config{width:1,actions:5,minimum_training:128,
        search_every:4096,planning_work:120_000,
        planning_depth:4,journal_capacity:1024,
        ..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"adaptive-hidden-suffix").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    let mut world=World::new(needs_second_order);
    // This is deliberately a disclosed IDENTIFIABILITY schedule for an
    // engineering test, NOT autonomous cold acquisition. Both contexts
    // have same visible PRE and same immediately prior action 1 (B).
    for i in 0..44usize{
        step(&mut rt,&mut world,Some(4));
        step(&mut rt,&mut world,Some(if i%2==0{0}else{3}));
        step(&mut rt,&mut world,Some(1));
        step(&mut rt,&mut world,Some(2));
    }
    step(&mut rt,&mut world,Some(4));
    assert_eq!(world.progress,0);
    assert_eq!(rt.construction_learner().unwrap().records().len(),177);
    assert_eq!(rt.unified_report().unwrap().agency_credit.as_ref()
        .unwrap().externally_forced_actions,177);
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp).unwrap()
        .checkpoint_constructing().unwrap(),cp);
    (rt,world)
}
fn enable(rt:&mut ScientificRuntime,deeper:bool){
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    if deeper {rt.enable_constructing_adaptive_history_suffix().unwrap();}
}
#[test]
fn same_sensor_and_same_last_motor_but_different_older_action_are_distinguished(){
    let (trained,w)=acquired(true);
    let cp=trained.checkpoint_constructing().unwrap();
    let mut old=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    let mut learned=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    enable(&mut old,false);
    enable(&mut learned,true);
    let mut x=w.clone();let mut y=w.clone();
    let old_step=step(&mut old,&mut x,None);
    let new_step=step(&mut learned,&mut y,None);
    eprintln!("ADAPTIVE_SUFFIX_FIRST {}",json!({
        "baseline_offered":old_step.predictive_history_plan_proposed,
        "candidate_offered":new_step.adaptive_suffix_plan_proposed,
        "candidate_selected":new_step.source,
        "action":new_step.action,"actual_post":y.progress,
        "old_actual_post":x.progress}));
    assert!(!old_step.adaptive_suffix_plan_proposed);
    assert!(new_step.adaptive_suffix_plan_proposed,
        "the deeper matched-PRE real causal evidence must create a plan");
    // Critically, a proposal is NOT credited as an autonomously selected
    // plan when native U1 chooses something else. That is reported above.
    assert_eq!(learned.construction_learner().unwrap().records().len(),178);
}

#[test]
fn native_u1_attempts_the_full_unforced_hidden_preparation_chain(){
    let (trained,initial_world)=acquired(true);
    let checkpoint=trained.checkpoint_constructing().unwrap();
    let mut all=Vec::new();
    let mut candidate_reached=false;
    let mut candidate_suffix_actions=0usize;
    let mut continued=0usize;
    let mut completed=0u64;
    for adaptive in [false,true]{
        let mut rt=ScientificRuntime::from_constructing_checkpoint(&checkpoint).unwrap();
        enable(&mut rt,adaptive);
        if adaptive{rt.enable_constructing_temporal_options().unwrap();}
        let mut w=initial_world.clone();
        let mut trace=Vec::new();
        for _ in 0..12{
            if w.progress>=1{break}
            let outcome=step(&mut rt,&mut w,None);
            trace.push(json!({
                "motor":outcome.action,"source":outcome.source,
                "real_post":w.progress,
                "suffix_available":outcome.adaptive_suffix_plan_proposed
            }));
        }
        if adaptive{
            candidate_reached=w.progress>=1;
            candidate_suffix_actions=trace.iter()
                .filter(|t|t["source"]=="predictive-suffix").count();
            continued=trace.iter()
                .filter(|t|t["source"]=="temporal-option-continuation").count();
            completed=rt.unified_report().unwrap().temporal_options
                .expect("option lane must report").completed;
        }
        all.push(json!({"adaptive":adaptive,"solved":w.progress>=1,
            "actions":trace.len(),"trace":trace}));
    }
    eprintln!("ADAPTIVE_SUFFIX_CLOSED_LOOP {}",json!({"arms":all,
        "independent_qualification":false,
        "all_test_motors_unforced":true,
        "two_action_context_inferred_from_actual_training_POSTs":true}));
    assert!(candidate_suffix_actions>0,
        "the native controller must actually SELECT a depth-two plan");
    assert!(candidate_reached,
        "the native controller must finish its self-selected multi-step plan with an actual external goal POST");
    assert_eq!(continued,2,
        "native-U1 option has two physical continuations, not new primitive votes");
    assert_eq!(completed,1,
        "macro completes only after a real terminal protected POST");
}

#[test]
fn no_hidden_history_effect_cannot_produce_spurious_suffix_goal_plan(){
    let (mut rt,mut w)=acquired(false);enable(&mut rt,true);
    let chosen=step(&mut rt,&mut w,None);
    assert!(!chosen.adaptive_suffix_plan_proposed,
        "different action histories with identical outcomes cannot split states");
}
#[test]
fn unavailable_preparatory_motor_must_not_be_imagined(){
    let (mut rt,mut w)=acquired(true);enable(&mut rt,true);
    let offers=(0..5).map(|a|ActionOffer{
        available:a!=0,max_cost:1
    }).collect::<Vec<_>>();
    let chosen=rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
        w.execute(a);
        Ok(UnifiedPost{observation:vec![w.progress],cost:1,reward:0.0})
    }).unwrap();
    assert!(!chosen.adaptive_suffix_plan_proposed,
        "unavailable first prerequisite must not survive as a permitted plan");
}
#[test]
fn three_real_later_contradictions_revoke_a_suffix_goal_route(){
    let (mut rt,mut w)=acquired(true);
    enable(&mut rt,true);
    // The evaluator alone changes its hidden dynamics. The organism sees
    // only three contradictory executed POSTs for the same suffix.
    w.broken=true;
    for _ in 0..3{
        for motor in [4_usize,0,1,2]{step(&mut rt,&mut w,Some(motor));}
    }
    step(&mut rt,&mut w,Some(4));
    let check=step(&mut rt,&mut w,None);
    assert!(!check.adaptive_suffix_plan_proposed,
        "later real contradictions must withdraw the acquired explanation");
}
#[test]
fn underfunded_or_wrongly_initialized_suffix_never_gets_action_authority(){
    let cfg=Config{width:1,actions:5,minimum_training:128,
        search_every:4096,planning_depth:2,planning_work:4000,
        ..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"unfunded-adaptive").unwrap();
    assert!(rt.enable_constructing_adaptive_history_suffix().is_err());
    rt.enable_constructing_self_directed_control().unwrap();
    assert!(rt.enable_constructing_predictive_history_states().is_err());
}
#[test]
fn human_protection_veto_cannot_forge_suffix_training(){
    let (mut rt,mut w)=acquired(true);enable(&mut rt,true);
    let n=rt.construction_learner().unwrap().records().len();
    let all=[ActionOffer{available:true,max_cost:1};5];
    let mut called=false;
    let result=rt.step_constructing(Some(&[Some(1)]),&all,|_|{
        let mut evidence=safe(0);evidence.emergency_stop=true;evidence
    },|a|{
        called=true;
        w.execute(a);
        Ok(UnifiedPost{observation:vec![w.progress],cost:1,reward:0.0})
    });
    assert!(result.is_err());assert!(!called);
    assert_eq!(rt.construction_learner().unwrap().records().len(),n);
}
