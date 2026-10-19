//! INSTRUMENTAL-CREDIT1 disclosed causal-credit control. NO science claim.
//! Every action remains protected, and every world POST is physically
//! executed in a toy external callback; never synthesize learner journal.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::json;
fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};
fn rt(grounded:bool)->ScientificRuntime{
    let cfg=Config{width:1,actions:4,minimum_training:128,search_every:4096,
        journal_capacity:1024,planning_work:80_000,planning_depth:3,
        ..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,
        AutonomyPolicy::default(),vec![0],"grounded-instrumental-credit").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    if grounded{rt.enable_constructing_instrumental_credit().unwrap();}
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.enable_constructing_active_causal_probes().unwrap();
    rt
}
#[test]
fn unsolved_information_may_teach_facts_but_not_utility_until_actual_progress(){
    let mut old=rt(false);let mut guarded=rt(true);
    for i in 0..12{
        // Same non-progressing real virtual world, all motors freely offered.
        for organism in [&mut old,&mut guarded]{
            let output=organism.step_constructing(
                Some(&[Some(100)]),&[OFFER;4],safe,|_action|{
                    Ok(UnifiedPost{observation:vec![0],cost:1,reward:0.0})
                }).unwrap();
            assert!(output.action<4);
        }
        assert_eq!(guarded.construction_learner().unwrap().records().len(),i+1);
    }
    assert_eq!(guarded.unified_report().unwrap().native_meta_observations,0);
    assert!(guarded.unified_report().unwrap().native_weights.iter()
        .all(|&v|v==0.0));
    let old_report=old.unified_report().unwrap();
    assert!(old_report.native_meta_observations>0,
        "historical default should still train U1 from immediate novelty/uncertainty");
    let c=guarded.unified_report().unwrap().instrumental_credit.unwrap();
    assert_eq!(c.facts_without_progress,12);
    assert_eq!(c.actual_progress_credited,0);
    assert!(!c.pending_preparation);
    // Physical progress is now actually observed, with no evaluator label.
    guarded.step_constructing(Some(&[Some(100)]),&[OFFER;4],safe,|_action|{
        Ok(UnifiedPost{observation:vec![1],cost:1,reward:0.0})
    }).unwrap();
    let report=guarded.unified_report().unwrap();
    let c=report.instrumental_credit.unwrap();
    assert_eq!(c.facts_without_progress,12);
    assert_eq!(c.actual_progress_credited,1);
    assert!(report.native_meta_observations>0);
    let bytes=guarded.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&bytes)
        .unwrap().checkpoint_constructing().unwrap(),bytes);
}
#[test]
fn refused_action_or_late_optin_cannot_credit_virtual_progress(){
    let mut guarded=rt(true);let mut callback_called=false;
    let result=guarded.step_constructing(
        Some(&[Some(2)]),&[OFFER;4],|_action|{
            let mut e=safe(0);e.emergency_stop=true;e
        },|_|{
            callback_called=true;
            Ok(UnifiedPost{observation:vec![1],cost:1,reward:0.0})
        });
    assert!(result.is_err());assert!(!callback_called);
    assert!(guarded.construction_learner().unwrap().records().is_empty());
    assert_eq!(guarded.unified_report().unwrap().native_meta_observations,0);
    assert_eq!(guarded.unified_report().unwrap().instrumental_credit.unwrap().actual_progress_credited,0);
    let mut no_agency=ScientificRuntime::new_constructing(
        Config{width:1,actions:4,minimum_training:128,..Default::default()},
        AutonomyPolicy::default(),vec![0],"no-agency").unwrap();
    assert!(no_agency.enable_constructing_instrumental_credit().is_err());
    guarded.enable_constructing_instrumental_credit().unwrap_err();
}
#[derive(Clone)]
struct World{progress:i64,ready:bool,roles:[usize;4]}
impl World{
    fn act(&mut self,a:usize){
        let role=self.roles.iter().position(|&x|x==a).unwrap();
        match role{
            0=>self.ready=true,
            1=>{if self.ready{self.progress+=1};self.ready=false},
            2=>self.ready=false,
            3=>{self.progress=0;self.ready=false},
            _=>unreachable!(),
        }
    }
}
fn run(active:bool,roles:[usize;4])->serde_json::Value{
    let mut rt=rt(active);
    let mut w=World{progress:0,ready:false,roles};
    let mut probe_selected=0;
    let mut progress_actions=0;
    // Every identification motor is selected by U1 with no prescribed order.
    for _ in 0..192{
        let before=w.progress;
        let out=rt.step_constructing(Some(&[Some(2000)]),&[OFFER;4],safe,|a|{
            w.act(a);
            Ok(UnifiedPost{observation:vec![w.progress],cost:1,reward:0.0})
        }).unwrap();
        probe_selected+=usize::from(out.source=="causal-pair-probe");
        progress_actions+=usize::from(w.progress>before);
    }
    let learned=rt.unified_report().unwrap();
    let train_state=w.clone();
    rt.freeze_constructing().unwrap();
    let cp=rt.checkpoint_constructing().unwrap();
    let mut rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    w=train_state;
    let target=w.progress+1;
    let mut test_actions=0;let mut selected_history=0;
    for _ in 0..24{
        if w.progress>=target{break}
        let out=rt.step_constructing(Some(&[Some(target)]),&[OFFER;4],safe,|a|{
            w.act(a);Ok(UnifiedPost{observation:vec![w.progress],cost:1,reward:0.0})
        }).unwrap();
        selected_history+=usize::from(out.source=="predictive-history");
        test_actions+=1;
    }
    json!({"roles":roles,"instrumental":active,
        "real_cold_actions":192,"forced_actions":0,
        "cold_probe_selected":probe_selected,"progress_actions":progress_actions,
        "initial_native_weights":learned.native_weights,
        "credit":learned.instrumental_credit,
        "test_solved":w.progress>=target,"test_actions":test_actions,
        "history_selected":selected_history,"source_checkpoint_size":cp.len()})
}
#[test]
fn disclosed_unscheduled_lifetimes_show_goal_vs_exploration_credit_separation(){
    // Different from earlier *two* SELF-PROBE role maps, but still authored
    // engineering worlds. Do not call this independent transfer.
    let mut data=Vec::new();
    for roles in [[1,0,3,2],[3,2,1,0]]{
        data.push(run(false,roles));
        data.push(run(true,roles));
    }
    eprintln!("INSTRUMENTAL_CREDIT_DEVELOPMENT {}",json!({
        "cases":data,"forced_training_actions":0,
        "independent_qualification":false
    }));
    assert_eq!(data.len(),4);
}
