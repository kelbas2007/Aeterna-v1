//! OPEN/EXPOSED developmental diagnostic of cold, unscheduled causal probing.
//! Not an independent qualified result or unlimited meta-self-improvement.
//! During identification ALL four opaque motors are genuinely available;
//! every choice is native U1 and every POST comes from protected execution.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::json;

fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{
        human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false,
    }
}
const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};
#[derive(Clone)]
struct World{
    progress:i64,ready:bool,roles:[usize;4],hidden_matters:bool,
}
impl World{
    fn new(roles:[usize;4],hidden_matters:bool)->Self{
        Self{progress:0,ready:false,roles,hidden_matters}
    }
    fn act(&mut self,action:usize){
        let role=self.roles.iter().position(|&a|a==action).unwrap();
        match role{
            0=>self.ready=true,
            1=>{
                if !self.hidden_matters || self.ready{self.progress+=1;}
                self.ready=false;
            },
            2=>self.ready=false,
            3=>{self.progress=0;self.ready=false},
            _=>unreachable!(),
        }
    }
}
fn new_runtime(active:bool)->ScientificRuntime{
    let cfg=Config{width:1,actions:4,minimum_training:128,
        search_every:4096,journal_capacity:1024,planning_depth:3,
        planning_work:80_000,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"cold-unscheduled-pair-probing").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    if active{rt.enable_constructing_active_causal_probes().unwrap();}
    rt
}
fn execute(rt:&mut ScientificRuntime,world:&mut World,goal:i64)
    ->aeterna_v1::scientific_runtime::UnifiedStep{
    rt.step_constructing(Some(&[Some(goal)]),&[OFFER;4],safe,|action|{
        world.act(action);
        Ok(UnifiedPost{observation:vec![world.progress],cost:1,reward:0.0})
    }).unwrap()
}
#[test]
fn cold_u1_chooses_its_own_information_experiments_without_tutor(){
    let mut data=Vec::new();
    let mut total_selected=0usize;
    let mut total_offered=0usize;
    for roles in [[0,1,2,3],[2,0,3,1]]{
        for probing in [false,true] {
            let mut world=World::new(roles,true);
            let mut rt=new_runtime(probing);
            let mut counts=[0usize;4];
            let mut offered=0usize;
            let mut selected=0usize;
            let mut observed_progress=0usize;
            for _ in 0..192 {
                let before=world.progress;
                let result=execute(&mut rt,&mut world,2000);
                counts[result.action]+=1;
                offered+=usize::from(result.active_pair_probe_proposed);
                selected+=usize::from(result.source=="causal-pair-probe");
                observed_progress+=usize::from(world.progress>before);
            }
            assert_eq!(rt.construction_learner().unwrap().records().len(),192,
                "each actual unscheduled physical POST must appear in same journal");
            let agency=rt.unified_report().unwrap().agency_credit.unwrap();
            assert_eq!(agency.externally_forced_actions,0,
                "NEVER force any identified learning motor");
            assert_eq!(agency.self_selected_actions,192);
            assert_eq!(counts.iter().sum::<usize>(),192);
            if !probing{assert_eq!((selected,offered),(0,0));}
            let source_checkpoint=rt.checkpoint_constructing().unwrap();
            rt.freeze_constructing().unwrap();
            let checkpoint=rt.checkpoint_constructing().unwrap();
            assert_eq!(ScientificRuntime::from_constructing_checkpoint(&checkpoint)
                .unwrap().checkpoint_constructing().unwrap(),checkpoint);
            let mut rt=ScientificRuntime::from_constructing_checkpoint(&checkpoint).unwrap();
            let mut world=world.clone();
            let target=world.progress+1;
            let mut history_selected=0usize;
            let mut goal_steps=0usize;
            let mut history_offered=0usize;
            for _ in 0..24{
                if world.progress>=target{break}
                let step=execute(&mut rt,&mut world,target);
                goal_steps+=1;
                history_selected+=usize::from(step.source=="predictive-history");
                history_offered+=usize::from(step.predictive_history_plan_proposed);
            }
            data.push(json!({
                "roles":roles,"active_probe":probing,
                "training_actions":192,"training_motor_counts":counts,
                "training_probe_offered":offered,
                "training_probe_selected":selected,
                "training_actual_progress_steps":observed_progress,
                "training_world_progress":target-1,
                "source_checkpoint_bytes":source_checkpoint.len(),
                "target":target,"test_solved":world.progress>=target,
                "test_actions":goal_steps,
                "test_history_offered":history_offered,
                "test_history_selected":history_selected,
                "frozen_check":true,
            }));
            if probing{total_selected+=selected;total_offered+=offered;}
        }
    }
    eprintln!("COLD_PAIR_PROBE_DEVELOPMENT {}",json!({
        "cases":data,"selected_probe":total_selected,
        "offered_probe":total_offered,
        "no_forced_training":true,"independent_qualification":false
    }));
    assert!(total_offered>0,
        "the organism must notice unknown causal pair experiments");
    assert!(total_selected>0,
        "an offered thought is not autonomous exploration: U1 must SELECT the probe");
}
#[test]
fn safety_veto_preserves_cold_epistemic_journal_and_motors(){
    let mut rt=new_runtime(true);
    let mut called=false;
    let out=rt.step_constructing(Some(&[Some(1000)]),&[OFFER;4],
        |_|{
            let mut e=safe(0);e.emergency_stop=true;e
        },
        |_|{
            called=true;
            Ok(UnifiedPost{observation:vec![10],cost:1,reward:0.0})
        });
    assert!(out.is_err());assert!(!called);
    assert!(rt.construction_learner().unwrap().records().is_empty());
    let agency=rt.unified_report().unwrap().agency_credit.unwrap();
    assert_eq!((agency.externally_forced_actions,agency.self_selected_actions),(0,0));
}
#[test]
fn one_legacy_budget_and_incorrect_opt_in_order_cannot_enable_probe_lane(){
    let cfg=Config{width:1,actions:4,minimum_training:128,
        search_every:4096,planning_work:4000,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"underspecified").unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    assert!(rt.enable_constructing_active_causal_probes().is_err());
    assert!(rt.enable_constructing_predictive_history_states().is_err());
}
