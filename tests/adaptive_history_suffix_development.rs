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
fn a_self_selected_macro_learns_from_the_entire_executed_trajectory(){
    let (mut rt,mut world)=acquired(true);
    enable(&mut rt,true);
    rt.enable_constructing_temporal_options().unwrap();
    let first=step(&mut rt,&mut world,None);
    assert_eq!(first.source,"predictive-suffix");
    let first_report=rt.unified_report().unwrap();
    assert_eq!(first_report.temporal_options.as_ref().unwrap().terminal_credits,0,
        "the first invisible preparation is not a successful whole route");
    let meta_after_first=first_report.native_meta_observations;
    let checkpoint=rt.checkpoint_constructing().unwrap();
    let mut resumed=ScientificRuntime::from_constructing_checkpoint(&checkpoint).unwrap();
    assert_eq!(resumed.checkpoint_constructing().unwrap(),checkpoint);
    let middle=step(&mut resumed,&mut world,None);
    assert_eq!(middle.source,"temporal-option-continuation");
    assert_eq!(resumed.unified_report().unwrap().native_meta_observations,
        meta_after_first,"the middle no-op cannot mint positive U1 utility");
    let last=step(&mut resumed,&mut world,None);
    assert_eq!(last.source,"temporal-option-continuation");
    assert_eq!(world.progress,1,"only the real final physical POST is success");
    let report=resumed.unified_report().unwrap();
    assert_eq!(report.native_meta_observations,meta_after_first+1,
        "one U1 decision gets one terminal outcome, not three action votes");
    let lane=report.temporal_options.unwrap();
    assert_eq!((lane.started,lane.completed,lane.terminal_credits,
        lane.contradictory_credits,lane.completed_real_cost,lane.pending_steps),
        (1,1,1,0,3,0));
    let saved=resumed.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&saved).unwrap()
        .checkpoint_constructing().unwrap(),saved);
}

#[test]
fn matched_physical_option_outcomes_causally_diverge_native_u1_weights(){
    // Exposed MECHANISM check, not cold generalization: one identical
    // acquired checkpoint, identical native choices/POSTs through two steps.
    // The third REAL physical POST alone differs. No synthetic learner rows.
    let (trained,original_world)=acquired(true);
    let bytes=trained.checkpoint_constructing().unwrap();
    let mut solved=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    let mut failed=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    enable(&mut solved,true);enable(&mut failed,true);
    solved.enable_constructing_temporal_options().unwrap();
    failed.enable_constructing_temporal_options().unwrap();
    let mut good_world=original_world.clone();
    let mut bad_world=original_world.clone();
    for expected in ["predictive-suffix","temporal-option-continuation"] {
        let good=step(&mut solved,&mut good_world,None);
        let bad=step(&mut failed,&mut bad_world,None);
        assert_eq!((good.action,good.source.as_str()),
            (bad.action,bad.source.as_str()));
        assert_eq!(good.source,expected);
        assert_eq!(solved.unified_report().unwrap().native_weights,
            failed.unified_report().unwrap().native_weights,
            "prior selected physical actions and native learning must match");
    }
    let before=solved.unified_report().unwrap().native_meta_observations;
    bad_world.broken=true; // evaluator-only latent change, no privileged input
    let good=step(&mut solved,&mut good_world,None);
    let bad=step(&mut failed,&mut bad_world,None);
    assert_eq!((good.action,good.source.as_str()),
        (bad.action,bad.source.as_str()));
    assert_eq!((good_world.progress,bad_world.progress),(1,0));
    let good=solved.unified_report().unwrap();
    let bad=failed.unified_report().unwrap();
    assert_eq!((good.native_meta_observations,bad.native_meta_observations),
        (before+1,before+1),"both real terminal outcomes update exactly once");
    assert_eq!(good.temporal_options.as_ref().unwrap().terminal_credits,1);
    assert_eq!(bad.temporal_options.as_ref().unwrap().contradictory_credits,1);
    let change: f32=good.native_weights.iter().zip(bad.native_weights.iter())
        .map(|(a,b)|(a-b).abs()).sum();
    assert!(change>1.0e-6,
        "successful vs failed complete actions must alter physical U1 synaptic evaluation");
    for rt in [&solved,&failed]{
        let cp=rt.checkpoint_constructing().unwrap();
        let restored=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
        assert_eq!(restored.checkpoint_constructing().unwrap(),cp);
    }
    eprintln!("TEMPORAL_CREDIT_MATCHED_POST {}",json!({
        "same_earned_votes":true,
        "actual_goal_good":good_world.progress,
        "actual_goal_bad":bad_world.progress,
        "native_weight_l1_divergence":change,
        "independent_qualification":false
    }));
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


#[test]
fn causal_option_survives_checkpoint_but_fresh_protection_veto_revokes_it(){
    let (mut rt,mut w)=acquired(true);
    enable(&mut rt,true);
    rt.enable_constructing_temporal_options().unwrap();
    let first=step(&mut rt,&mut w,None);
    assert_eq!(first.source,"predictive-suffix");
    let r=rt.unified_report().unwrap().temporal_options.unwrap();
    assert_eq!((r.started,r.pending_steps),(1,2));
    let bytes=rt.checkpoint_constructing().unwrap();
    let mut restored=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(),bytes);
    let before=restored.construction_learner().unwrap().records().len();
    let mut called=false;
    let offers=[ActionOffer{available:true,max_cost:1};5];
    let blocked=restored.step_constructing(
        Some(&[Some(1)]),&offers,
        |_|{let mut h=safe(0);h.emergency_stop=true;h},
        |_motor|{
            called=true;
            Ok(UnifiedPost{observation:vec![9],cost:1,reward:0.0})
        }
    );
    assert!(blocked.is_err());
    assert!(!called,"every subordinate physical step needs fresh protection");
    assert_eq!(restored.construction_learner().unwrap().records().len(),before);
    let r=restored.unified_report().unwrap().temporal_options.unwrap();
    assert_eq!((r.revoked,r.completed,r.pending_steps),(1,0,0));
    let saved=restored.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&saved)
        .unwrap().checkpoint_constructing().unwrap(),saved);
}
#[test]
fn real_unexpected_option_post_rejects_remaining_plan_without_forging_goal(){
    let (mut rt,mut w)=acquired(true);
    enable(&mut rt,true);rt.enable_constructing_temporal_options().unwrap();
    let first=step(&mut rt,&mut w,None);
    assert_eq!(first.source,"predictive-suffix");
    let middle=step(&mut rt,&mut w,None);
    assert_eq!((middle.action,middle.source.as_str()),(1,"temporal-option-continuation"));
    w.broken=true; // Evaluator-only change; hidden from the organism
    let meta_before_last=rt.unified_report().unwrap().native_meta_observations;
    let last=step(&mut rt,&mut w,None);
    assert_eq!((last.action,last.source.as_str()),(2,"temporal-option-continuation"));
    assert_eq!(w.progress,0,"the physical world did NOT reach the goal");
    let r=rt.unified_report().unwrap();
    let lane=r.temporal_options.unwrap();
    assert_eq!((lane.started,lane.continued,lane.completed,lane.revoked,lane.pending_steps),
        (1,1,0,1,0));
    assert_eq!((lane.terminal_credits,lane.contradictory_credits,lane.completed_real_cost),
        (0,1,0),"failed real POST must be credited as failure, not goal");
    assert_eq!(r.native_meta_observations,meta_before_last+1,
        "only the original U1 macro choice receives the observed failure");
    let agency=r.agency_credit.unwrap();
    assert_eq!((agency.self_selected_actions,agency.option_continuation_actions),(1,2));
    assert_eq!(rt.construction_learner().unwrap().records().len(),180);
    let saved=rt.checkpoint_constructing().unwrap();
    let restored=ScientificRuntime::from_constructing_checkpoint(&saved).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(),saved);
}
#[test]
fn unavailable_option_motor_is_not_executed_and_cancels_macro(){
    let (mut rt,mut w)=acquired(true);
    enable(&mut rt,true);rt.enable_constructing_temporal_options().unwrap();
    assert_eq!(step(&mut rt,&mut w,None).source,"predictive-suffix");
    let before=rt.construction_learner().unwrap().records().len();
    let mut actually_executed=false;
    let offers=(0..5).map(|a|ActionOffer{available:a!=1,max_cost:1})
        .collect::<Vec<_>>();
    let denied=rt.step_constructing(Some(&[Some(1)]),&offers,safe,|_motor|{
        actually_executed=true;
        Ok(UnifiedPost{observation:vec![1],cost:1,reward:0.0})
    });
    assert!(denied.is_err());assert!(!actually_executed);
    assert_eq!(rt.construction_learner().unwrap().records().len(),before);
    let option=rt.unified_report().unwrap().temporal_options.unwrap();
    assert_eq!((option.started,option.revoked,option.pending_steps),(1,1,0));
}
