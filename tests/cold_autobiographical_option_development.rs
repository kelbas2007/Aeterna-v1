//! SELF-ORIGINATED OPTION-0: EXPOSED COLD developmental comparison.
//! NO teacher action schedule, role tags, fabricated POST or learner-row insert.
//! The prior frozen COLD-FRESH1 negative is NOT reinterpreted by this test.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::{json,Value};

const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};
const EPISODES:usize=24;
const MAX_STEPS:usize=18;
fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
#[derive(Clone)]
struct World {roles:[usize;4],stage:u8,progress:i64,direct:bool}
impl World{
    fn new(roles:[usize;4],direct:bool)->Self{
        Self{roles,stage:0,progress:0,direct}
    }
    fn act(&mut self,motor:usize){
        let role=self.roles.iter().position(|&a|a==motor).unwrap();
        if self.direct {
            match role{2=>self.progress+=1,3=>self.progress=0,_=>{}}
        }else{
            match role {
                0=>self.stage=1,
                1=>self.stage=if self.stage==1{2}else{0},
                2=>{
                    if self.stage==2 {self.progress+=1;}
                    self.stage=0;
                },
                3=>self.stage=0,
                _=>unreachable!()
            }
        }
    }
}
fn new_cold(with_memory:bool)->ScientificRuntime{
    let cfg=Config{width:1,actions:4,minimum_training:128,
        search_every:4096,planning_depth:4,planning_work:100_000,
        journal_capacity:2048,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"cold-autobiographical").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.enable_constructing_adaptive_history_suffix().unwrap();
    rt.enable_constructing_temporal_options().unwrap();
    rt.enable_constructing_active_causal_probes().unwrap();
    if with_memory{rt.enable_constructing_autobiographical_options().unwrap();}
    assert!(rt.construction_learner().unwrap().records().is_empty());
    assert!(rt.unified_report().unwrap().native_weights.iter().all(|&v|v==0.0));
    rt
}
fn lifetime(roles:[usize;4],direct:bool,memory:bool)->Value{
    let mut rt=new_cold(memory);
    let mut successes=0usize;
    let mut early=0usize;
    let mut late=0usize;
    let mut actions=0usize;
    let mut proposed=0usize;
    let mut selected=0usize;
    let mut continuations=0usize;
    let mut episodes=Vec::new();
    for episode in 0..EPISODES{
        let mut world=World::new(roles,direct);
        if episode>0{
            let before=rt.construction_learner().unwrap().records().len();
            rt.observe_constructing_external_restart(vec![0]).unwrap();
            assert_eq!(before,rt.construction_learner().unwrap().records().len());
        }
        let mut sequence=Vec::new();
        for _ in 0..MAX_STEPS{
            if world.progress>=1{break}
            let out=rt.step_constructing(Some(&[Some(1)]),&[OFFER;4],safe,
                |motor|{
                    world.act(motor);
                    Ok(UnifiedPost{observation:vec![world.progress],
                        cost:1,reward:0.0})
                }).unwrap();
            proposed+=usize::from(out.autobiographical_option_proposed);
            selected+=usize::from(out.source=="autobiographical-option");
            continuations+=usize::from(out.source=="temporal-option-continuation");
            actions+=1;
            sequence.push(json!({"action":out.action,
                "source":out.source,"real_post":world.progress,
                "memory_proposed":out.autobiographical_option_proposed}));
        }
        successes+=usize::from(world.progress>=1);
        if episode<6{early+=usize::from(world.progress>=1);}
        if episode>=EPISODES-6{late+=usize::from(world.progress>=1);}
        episodes.push(json!({"ep":episode,"solved":world.progress>=1,
            "actions":sequence.len(),"trace":sequence}));
    }
    let report=rt.unified_report().unwrap();
    let a=report.agency_credit.as_ref().unwrap();
    assert_eq!(a.externally_forced_actions,0,"no scripted motor ever");
    assert_eq!(a.self_selected_actions+
        a.option_continuation_actions,actions as u64);
    assert_eq!(a.option_continuation_actions,continuations as u64);
    assert_eq!(rt.construction_learner().unwrap().records().len(),actions);
    assert_eq!(rt.construction_learner().unwrap().records().iter()
        .filter(|r|r.transition.restart).count(),EPISODES);
    let option=report.temporal_options.as_ref().unwrap();
    assert_eq!(option.autobiographical_enabled,memory);
    assert_eq!(option.autobiographical_started,selected as u64);
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp).unwrap()
        .checkpoint_constructing().unwrap(),cp);
    json!({"roles":roles,"memory":memory,"direct":direct,
        "successes":successes,"first_six":early,"last_six":late,
        "actions":actions,"proposed":proposed,"selected":selected,
        "continuations":continuations,
        "completed_options":option.completed,
        "native_weights":report.native_weights,
        "terminal_credits":option.terminal_credits,
        "failed_real_outcomes":option.contradictory_credits,
        "full_real_episode_traces":episodes})
}
fn permutations()->Vec<[usize;4]>{
    let mut out=Vec::new();
    for a in 0..4{for b in 0..4{if a==b{continue}
        for c in 0..4{if c==a||c==b{continue}
            let d=(0..4).find(|&x|x!=a&&x!=b&&x!=c).unwrap();
            out.push([a,b,c,d]);
        }
    }}}
    assert_eq!(out.len(),24);
    // Fixed disclosed developmental shuffle, never a one-use scientific seed.
    let mut seed=0x7e51_13ae_09df_d410_u64;
    for i in (1..out.len()).rev(){
        seed^=seed>>12;seed^=seed<<25;seed^=seed>>27;
        seed=seed.wrapping_mul(0x2545f4914f6cdd1d);
        out.swap(i,(seed % (i as u64+1)) as usize);
    }
    out.truncate(8);out
}
#[test]
fn cold_self_originated_3_action_route_vs_same_native_u1_without_memory(){
    let mut records=Vec::new();
    let mut sum=[0usize;2];
    let mut actions=[0usize;2];
    let mut offered=0usize;
    let mut selected=0usize;
    let mut completed=0usize;
    let mut early=0usize;
    let mut late=0usize;
    for roles in permutations(){
        let off=lifetime(roles,false,false);
        let on=lifetime(roles,false,true);
        for (i,v) in [off.clone(),on.clone()].iter().enumerate(){
            sum[i]+=v["successes"].as_u64().unwrap() as usize;
            actions[i]+=v["actions"].as_u64().unwrap() as usize;
        }
        offered+=on["proposed"].as_u64().unwrap() as usize;
        selected+=on["selected"].as_u64().unwrap() as usize;
        completed+=on["completed_options"].as_u64().unwrap() as usize;
        early+=on["first_six"].as_u64().unwrap() as usize;
        late+=on["last_six"].as_u64().unwrap() as usize;
        records.push(json!({"roles":roles,"control":off,"candidate":on}));
    }
    let summary=json!({"cold_zero_tuition":true,
        "worlds":records.len(),"total_goal_episodes":EPISODES*records.len(),
        "actions_per_episode_max":MAX_STEPS,
        "baseline_goal_successes":sum[0],"memory_goal_successes":sum[1],
        "baseline_real_actions":actions[0],"memory_real_actions":actions[1],
        "memory_proposed":offered,"memory_native_selected":selected,
        "memory_completed_real_options":completed,
        "candidate_early_goals":early,"candidate_late_goals":late,
        "independent_scientific_qualification":false});
    std::fs::create_dir_all("cold-memory-evidence").unwrap();
    std::fs::write("cold-memory-evidence/all_real_traces.json",
        serde_json::to_vec_pretty(&json!({"summary":summary,"cases":records}))
            .unwrap()).unwrap();
    eprintln!("SELF_ORIGINATED_OPTION0 {}",summary);
    assert!(offered>0,
        "self-collected successes must generate a genuine optional plan");
    assert!(selected>0 && completed>0,
        "native U1 must select and complete at least one earned, physically verified option");
}
#[test]
fn direct_progress_world_does_not_invent_hidden_causal_preparations(){
    let result=lifetime([2,0,3,1],true,true);
    assert_eq!(result["proposed"].as_u64(),Some(0),
        "a last motor that always makes visible progress has no contrasted hidden dependency");
    assert_eq!(result["selected"].as_u64(),Some(0));
}
