//! COLD-EPISODES1 developmental control. ALL motors offered at EVERY
//! action. No trainer-scheduled motors or hidden-state/world labels enter U1.
//! The evaluator may start a fresh REAL world episode and pass its genuine
//! new sensor observation through the world-restart API.
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
#[derive(Clone)]
struct World{progress:i64,primed:bool,roles:[usize;4]}
impl World{
    fn new(roles:[usize;4])->Self{Self{progress:0,primed:false,roles}}
    fn execute(&mut self,action:usize){
        let role=self.roles.iter().position(|&id|id==action).unwrap();
        match role{
            0=>self.primed=true,
            1=>{if self.primed{self.progress+=1}self.primed=false;},
            2=>self.primed=false,
            3=>{self.progress=0;self.primed=false;},
            _=>unreachable!(),
        }
    }
}
fn cognition(mode:usize)->ScientificRuntime{
    let cfg=Config{width:1,actions:4,minimum_training:128,
        search_every:4096,planning_depth:3,planning_work:80_000,
        journal_capacity:2048,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"autonomous-protected-episodes").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    if mode==3{rt.enable_constructing_instrumental_credit().unwrap();}
    rt.enable_constructing_self_directed_control().unwrap();
    if mode>=1 {rt.enable_constructing_predictive_history_states().unwrap();}
    if mode>=2 {rt.enable_constructing_active_causal_probes().unwrap();}
    rt
}
fn run(roles:[usize;4],mode:usize)->serde_json::Value{
    let mut rt=cognition(mode);
    let mut early=0usize;let mut late=0usize;
    let mut goals=0usize;let mut actions=0usize;
    let mut chosen_history=0usize;let mut chosen_probes=0usize;
    let mut per_episode=Vec::new();
    const EPISODES:usize=24;
    const ACTIONS:usize=18;
    for ep in 0..EPISODES{
        // World instance is evaluator-side only; this is not a U1 motor.
        let mut w=World::new(roles);
        if ep>0{
            let prior_steps=rt.construction_learner().unwrap().records().len();
            let prior_cost=rt.unified_report().unwrap().total_external_cost;
            rt.observe_constructing_external_restart(vec![w.progress]).unwrap();
            assert_eq!(rt.construction_learner().unwrap().records().len(),prior_steps,
                "external reset is factual sensing, never a fabricated POST");
            assert_eq!(rt.unified_report().unwrap().total_external_cost,prior_cost,
                "pure observation does not reset or mint physical-action credits");
        }
        let mut took=0usize;
        for _ in 0..ACTIONS{
            if w.progress>=1{break}
            let next=rt.step_constructing(Some(&[Some(1)]),&[OFFER;4],
                safe,|action|{
                    w.execute(action);
                    Ok(UnifiedPost{observation:vec![w.progress],
                        cost:1,reward:0.0})
                }).unwrap();
            actions+=1;took+=1;
            chosen_history+=usize::from(next.source=="predictive-history");
            chosen_probes+=usize::from(next.source=="causal-pair-probe");
        }
        let success=w.progress>=1;
        goals+=usize::from(success);
        if ep<6{early+=usize::from(success)}
        if ep>=EPISODES-6{late+=usize::from(success)}
        per_episode.push(json!({"episode":ep,"success":success,"actions":took}));
    }
    let ledger=rt.unified_report().unwrap();
    let agency=ledger.agency_credit.as_ref().unwrap();
    assert_eq!(agency.externally_forced_actions,0);
    assert_eq!(agency.self_selected_actions,actions as u64);
    let rows=rt.construction_learner().unwrap().records();
    assert_eq!(rows.len(),actions);
    assert_eq!(rows.iter().filter(|r|r.transition.restart).count(),EPISODES,
        "one reset-sentinel on the first real POST of each episode");
    assert!(rows.windows(2).all(|p|
        p[0].transition.id.checked_add(1)==Some(p[1].transition.id)));
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp).unwrap()
        .checkpoint_constructing().unwrap(),cp);
    json!({"roles":roles,"mode":mode,
        "goal_successes":goals,"total_episodes":EPISODES,
        "early_successes_first6":early,"late_successes_last6":late,
        "executed_actions":actions,"selected_history":chosen_history,
        "selected_causal_probes":chosen_probes,
        "observed_episode_restarts":EPISODES-1,
        "native_weights":ledger.native_weights,
        "native_meta_observations":ledger.native_meta_observations,
        "instrumental_credit":ledger.instrumental_credit,
        "full_episode_trace":per_episode})
}
#[test]
fn cold_protected_u1_reuses_one_lifetime_across_world_initiated_episodes(){
    let mut outcomes=Vec::new();
    for roles in [[0,1,2,3],[2,0,3,1]]{
        for mode in 0..4{
            outcomes.push(run(roles,mode));
        }
    }
    eprintln!("COLD_EPISODES1_RESULT {}",json!({
        "results":outcomes,"independent_qualification":false,
        "tutored_motor_actions":0,
        "world_restarts_are_observed_not_executed":true
    }));
    assert_eq!(outcomes.len(),8);
}
#[test]
fn externally_observed_reset_preserves_true_checkpoint_and_safety_latch(){
    let mut rt=cognition(2);
    // A real world may reset while no action is ongoing. Width errors
    // must reject before altering the joint checkpoint.
    let before=rt.checkpoint_constructing().unwrap();
    assert!(rt.observe_constructing_external_restart(vec![0,7]).is_err());
    assert_eq!(rt.checkpoint_constructing().unwrap(),before);
    rt.observe_constructing_external_restart(vec![0]).unwrap();
    let after=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&after).unwrap()
        .checkpoint_constructing().unwrap(),after);
    let mut called=false;
    let blocked=rt.step_constructing(Some(&[Some(1)]),&[OFFER;4],
        |_a|{let mut h=safe(0);h.emergency_stop=true;h},
        |_a|{called=true;Ok(UnifiedPost{observation:vec![1],cost:1,reward:0.0})});
    assert!(blocked.is_err());assert!(!called);
    rt.observe_constructing_external_restart(vec![0]).unwrap();
    assert!(rt.step_constructing(Some(&[Some(1)]),&[OFFER;4],safe,
        |_a|panic!("external sensory reset cannot clear physical emergency")).is_err());
    assert!(rt.construction_learner().unwrap().records().is_empty());
}
