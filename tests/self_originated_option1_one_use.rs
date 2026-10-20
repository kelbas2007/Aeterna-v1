//! ONE-USE source-frozen synthetic cold transfer. Protocol BEFORE this source:
//! docs/SELF_ORIGINATED_OPTION1_ONE_USE_PROTOCOL.md at 914aa0e1.
//! Each action selected by the SAME native U1; all POSTs are real protected.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::{json,Value};
const EPISODES:usize=24;
const HORIZON:usize=18;
const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};
fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
fn mixed(mut n:u64)->u64{
    n=n.wrapping_add(0x9e3779b97f4a7c15);
    n=(n^(n>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n=(n^(n>>27)).wrapping_mul(0x94d049bb133111eb);
    n^(n>>31)
}
fn role_maps(run_id:u64,family:usize)->Vec<[usize;4]>{
    let mut maps=Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            if a==b{continue}
            for c in 0..4 {
                if c==a||c==b{continue}
                let d=(0..4).find(|&x|x!=a&&x!=b&&x!=c).unwrap();
                maps.push([a,b,c,d]);
            }
        }
    }
    assert_eq!(maps.len(),24);
    let mut seed=mixed(run_id ^ ((family as u64+1)
        .wrapping_mul(0xa24baed4963ee407)));
    for index in (1..maps.len()).rev(){
        seed=mixed(seed);
        maps.swap(index,(seed%(index as u64+1)) as usize);
    }
    maps.truncate(8);maps
}
#[derive(Clone)]
struct World{
    family:usize,roles:[usize;4],progress:i64,
    flags:u8,stage:u8
}
impl World{
    fn new(family:usize,roles:[usize;4])->Self{
        Self{family,roles,progress:0,flags:0,stage:0}
    }
    fn act(&mut self,motor:usize){
        let role=self.roles.iter().position(|&r|r==motor).unwrap();
        match (self.family,role){
            // F0: two INDEPENDENT persistent affordance flags; either
            // order is effective, and C consumes both.
            (0,0)=>self.flags|=1,
            (0,1)=>self.flags|=2,
            (0,2)=>{
                if self.flags==3{self.progress+=1}
                self.flags=0;
            },
            (0,3)=>self.flags=0,
            // F1: a repeated action A,A instead of the known A,B.
            (1,0)=>self.stage=(self.stage+1).min(2),
            (1,1)=>self.stage=0,
            (1,2)=>{
                if self.stage==2{self.progress+=1}
                self.stage=0;
            },
            (1,3)=>self.stage=0,
            // F2: toggling A can undo its own preparation. B bridges
            // state 1 to state 2; C consumes; D clears.
            (2,0)=>self.stage=if self.stage==1{0}else{1},
            (2,1)=>self.stage=if self.stage==1{2}else{0},
            (2,2)=>{
                if self.stage==2{self.progress+=1}
                self.stage=0;
            },
            (2,3)=>self.stage=0,
            // F3 negative control: goal motor acts immediately.
            (3,0)|(3,1)=>{},
            (3,2)=>self.progress+=1,
            (3,3)=>self.progress=0,
            _=>unreachable!()
        }
    }
}
fn cold(memory:bool)->ScientificRuntime{
    let cfg=Config{width:1,actions:4,minimum_training:128,
        search_every:4096,planning_depth:4,planning_work:100_000,
        journal_capacity:2048,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"cold-one-use-options").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.enable_constructing_adaptive_history_suffix().unwrap();
    rt.enable_constructing_temporal_options().unwrap();
    rt.enable_constructing_active_causal_probes().unwrap();
    if memory {rt.enable_constructing_autobiographical_options().unwrap();}
    assert!(rt.construction_learner().unwrap().records().is_empty());
    assert!(rt.unified_report().unwrap().native_weights.iter().all(|&w|w==0.0));
    rt
}
fn lifetime(family:usize,roles:[usize;4],memory:bool)->Value{
    let mut rt=cold(memory);
    let mut total_actions=0usize;
    let mut successes=0usize;
    let mut early=0usize;
    let mut late=0usize;
    let mut selected=0usize;
    let mut completed_memory=0usize;
    let mut pending_memory=0usize;
    let mut full=Vec::new();
    for episode in 0..EPISODES{
        let mut world=World::new(family,roles);
        if episode>0{
            let before=rt.construction_learner().unwrap().records().len();
            let cost=rt.unified_report().unwrap().total_external_cost;
            rt.observe_constructing_external_restart(vec![0]).unwrap();
            assert_eq!(before,rt.construction_learner().unwrap().records().len());
            assert_eq!(cost,rt.unified_report().unwrap().total_external_cost);
        }
        pending_memory=0;
        let mut trace=Vec::new();
        for _ in 0..HORIZON{
            if world.progress>=1{break}
            let r=rt.step_constructing(Some(&[Some(1)]),&[OFFER;4],
                safe,|a|{
                    world.act(a);
                    Ok(UnifiedPost{observation:vec![world.progress],
                        cost:1,reward:0.0})
                }).unwrap();
            total_actions+=1;
            if r.source=="autobiographical-option"{
                selected+=1;
                pending_memory=2;
            }else if pending_memory>0{
                if r.source=="temporal-option-continuation"{
                    pending_memory-=1;
                    if pending_memory==0 && world.progress>=1{
                        completed_memory+=1;
                    }
                }else{
                    pending_memory=0;
                }
            }
            trace.push(json!({"a":r.action,"source":r.source,
                "physical_post":world.progress,"cost":1,
                "autobiographical_offered":r.autobiographical_option_proposed}));
        }
        successes+=usize::from(world.progress>=1);
        if episode<6{early+=usize::from(world.progress>=1)}
        if episode>=EPISODES-6{late+=usize::from(world.progress>=1)}
        full.push(json!({"episode":episode,"success":world.progress>=1,
            "actions":trace.len(),"actual_trace":trace}));
    }
    let report=rt.unified_report().unwrap();
    let agency=report.agency_credit.as_ref().unwrap();
    assert_eq!(agency.externally_forced_actions,0);
    assert_eq!(agency.self_selected_actions+
        agency.option_continuation_actions,total_actions as u64);
    let rows=rt.construction_learner().unwrap().records();
    assert_eq!(rows.len(),total_actions);
    assert_eq!(rows.iter().filter(|r|r.transition.restart).count(),EPISODES);
    assert!(rows.windows(2).all(|w|
        w[0].transition.id.checked_add(1)==Some(w[1].transition.id)));
    assert_eq!(report.total_external_cost,total_actions as u64);
    let saved=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&saved)
        .unwrap().checkpoint_constructing().unwrap(),saved);
    json!({"family":family,"roles":roles,"candidate":memory,
        "successes":successes,"actions":total_actions,
        "early_six":early,"late_six":late,"memory_native_selected":selected,
        "completed_memory_routes":completed_memory,
        "memory_started":report.temporal_options.unwrap().autobiographical_started,
        "entire_actual_episode_traces":full})
}
#[test]
#[ignore="PREREGISTERED ONE-USE first GitHub RUN_ID only; never reseed after result"]
fn one_use_cold_self_originated_option_transfer(){
    let run_id=std::env::var("GITHUB_RUN_ID").expect("one-use CI id")
        .parse::<u64>().expect("CI id must be integer");
    let sha=std::env::var("GITHUB_SHA").expect("source merge SHA");
    let mut all=Vec::new();
    let mut groups=Vec::new();
    let mut total_success=[0usize;2];
    let mut total_actions=[0usize;2];
    let mut early=0usize;
    let mut late=0usize;
    let mut family_with_completions=0usize;
    for family in 0..4{
        let mut good=[0usize;2];
        let mut work=[0usize;2];
        let mut selected=0usize;
        let mut completed=0usize;
        let maps=role_maps(run_id,family);
        assert_eq!(maps.len(),8);
        for roles in maps{
            let baseline=lifetime(family,roles,false);
            let candidate=lifetime(family,roles,true);
            for (arm,out) in [baseline.clone(),candidate.clone()].iter().enumerate(){
                let solved=out["successes"].as_u64().unwrap() as usize;
                let actions=out["actions"].as_u64().unwrap() as usize;
                good[arm]+=solved;work[arm]+=actions;
                total_success[arm]+=solved;total_actions[arm]+=actions;
            }
            selected+=candidate["memory_native_selected"].as_u64().unwrap() as usize;
            completed+=candidate["completed_memory_routes"].as_u64().unwrap() as usize;
            early+=candidate["early_six"].as_u64().unwrap() as usize;
            late+=candidate["late_six"].as_u64().unwrap() as usize;
            all.push(json!({"family":family,"roles":roles,
                "baseline":baseline,"candidate":candidate}));
        }
        if family<3 && completed>0 && selected>0 {family_with_completions+=1;}
        groups.push(json!({"family":family,
            "baseline_goals":good[0],"candidate_goals":good[1],
            "baseline_actions":work[0],"candidate_actions":work[1],
            "memory_selected":selected,"memory_completed":completed}));
    }
    let no_family_regression=groups.iter().all(|g|
        g["candidate_goals"].as_u64().unwrap()>=
        g["baseline_goals"].as_u64().unwrap());
    let direct_negative_control=groups[3]["memory_selected"]==json!(0);
    let qualified=no_family_regression &&
        total_success[1].saturating_mul(100)>=total_success[0].saturating_mul(115) &&
        total_actions[1].saturating_mul(10)<=total_actions[0].saturating_mul(9) &&
        late>early && family_with_completions>=2 && direct_negative_control;
    let verdict=if qualified{
        "QUALIFIED_SYNTHETIC_SELF_ORIGINATED_OPTION_TRANSFER"
    }else{"NEGATIVE_SELF_ORIGINATED_OPTION_TRANSFER"};
    let summary=json!({
        "source_disclosed":true,"run_id":run_id,"github_merge_sha":sha,
        "protocol":"docs/SELF_ORIGINATED_OPTION1_ONE_USE_PROTOCOL.md",
        "verdict":verdict,"qualified":qualified,
        "worlds":all.len(),"episodes_per_world":EPISODES,
        "action_horizon":HORIZON,"tutor_motor_actions":0,
        "baseline_goals":total_success[0],"candidate_goals":total_success[1],
        "baseline_actions":total_actions[0],"candidate_actions":total_actions[1],
        "candidate_early_six":early,"candidate_late_six":late,
        "no_family_regression":no_family_regression,
        "families_with_complete_memory_replays":family_with_completions,
        "direct_negative_control":direct_negative_control,
        "families":groups
    });
    std::fs::create_dir_all("self-option-science").unwrap();
    std::fs::write("self-option-science/all_real_actions.json",
        serde_json::to_vec_pretty(&json!({"summary":summary,"cases":all})).unwrap())
        .unwrap();
    eprintln!("SELF_ORIGINATED_OPTION1_ONE_USE {}",summary);
    assert!(qualified,"first pre-registered one-use synthetic cold option criterion failed");
}
