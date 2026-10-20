//! OBSERVABLE-GRAPH-1: source-disclosed actual transition graph qualification.
//! Preregistration committed BEFORE this evaluator: docs/OBSERVABLE_GRAPH1_PROTOCOL.md
//! Every PRE/action/POST is physically executed by a protected virtual world.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::{json,Value};

const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};
const EPISODES:usize=24;
const HORIZON:usize=18;
fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
#[derive(Clone)]
struct World{family:usize,roles:[usize;4],stage:u8,progress:i64}
impl World{
    fn new(family:usize,roles:[usize;4])->Self{Self{family,roles,stage:0,progress:0}}
    fn act(&mut self,motor:usize){
        let role=self.roles.iter().position(|&m|m==motor).unwrap();
        match (self.family,role){
            (0,0)=>self.stage=1,
            (0,1)=>self.stage=if self.stage==1{2}else{0},
            (0,2)=>self.stage=if self.stage==2{3}else{0},
            (0,3)=>{if self.stage==3{self.progress+=1}self.stage=0},
            (1,0)=>self.stage=match self.stage{0=>1,2=>3,_=>0},
            (1,1)=>self.stage=match self.stage{1=>2,3=>4,_=>0},
            (1,2)=>{if self.stage==4{self.progress+=1}self.stage=0},
            (1,3)=>self.stage=0,
            (2,0)=>self.stage=1,
            (2,1)=>self.stage=if self.stage==1{2}else{0},
            (2,2)=>{if self.stage==2{self.progress+=1}self.stage=0},
            (2,3)=>self.stage=0,
            (3,0)|(3,1)=>{},
            (3,2)=>self.progress+=1,
            (3,3)=>self.progress=0,
            _=>unreachable!()
        }
    }
    // Real world sensor, not a modeled prediction. BLIND: physical stage
    // unavailable to cognition. RICH: the exact current world stage can be
    // measured by the host. Both arms have identical 2-channel configs.
    fn observation(&self,rich:bool)->Vec<i64>{
        vec![self.progress,if rich{self.stage as i64}else{0}]
    }
}
fn organism(graph:bool)->ScientificRuntime{
    let cfg=Config{width:2,actions:4,minimum_training:128,
        search_every:4096,planning_depth:5,planning_work:120_000,
        journal_capacity:2048,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0,0],"sensor-causal-ablation").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.enable_constructing_adaptive_history_suffix().unwrap();
    rt.enable_constructing_temporal_options().unwrap();
    if graph {rt.enable_constructing_observable_factual_routes().unwrap();}
    rt.enable_constructing_autobiographical_options().unwrap();
    rt.enable_constructing_variable_depth_options().unwrap();
    rt.enable_constructing_complexity_need_gate().unwrap();
    rt.enable_constructing_active_causal_probes().unwrap();
    rt.enable_constructing_contextual_experiments().unwrap();
    assert!(rt.construction_learner().unwrap().records().is_empty());
    assert_eq!(rt.unified_report().unwrap().native_meta_observations,0);
    rt
}
fn lifetime(family:usize,roles:[usize;4],rich:bool,graph:bool)->Value{
    let mut rt=organism(graph);
    let mut successes=0usize;
    let mut actions=0usize;
    let mut first_six=0usize;
    let mut last_six=0usize;
    let mut context=0usize;
    let mut graph_selected=0usize;
    let mut graph_offered=0usize;
    let mut graph_completed=0usize;
    let mut graph_pending=false;
    let mut episodes=Vec::new();
    for ep in 0..EPISODES{
        graph_pending=false;
        let mut world=World::new(family,roles);
        if ep>0 {
            let prior=rt.construction_learner().unwrap().records().len();
            rt.observe_constructing_external_restart(vec![0,0]).unwrap();
            assert_eq!(prior,rt.construction_learner().unwrap().records().len());
        }
        let mut trace=Vec::new();
        for _ in 0..HORIZON{
            if world.progress>=1{break}
            let step=rt.step_constructing(
                Some(&[Some(1),None]),&[OFFER;4],safe,|motor|{
                    world.act(motor);
                    Ok(UnifiedPost{observation:world.observation(rich),
                        cost:1,reward:0.0})
                }).unwrap();
            actions+=1;
            context+=usize::from(step.source=="contextual-causal-experiment");
            graph_selected+=usize::from(step.source=="observable-empirical-route");
            graph_offered+=usize::from(step.observable_empirical_route_proposed);
            let option=rt.unified_report().unwrap().temporal_options.unwrap();
            if step.source=="observable-empirical-route"{
                graph_pending=option.pending_steps>0;
            }else if graph_pending &&
                step.source=="temporal-option-continuation"{
                if option.pending_steps==0{
                    if world.progress>=1{graph_completed+=1;}
                    graph_pending=false;
                }
            }else if step.source!="temporal-option-continuation"{
                graph_pending=false;
            }
            trace.push(json!({"motor":step.action,"source":step.source,
                "physical_POST":world.observation(rich),
                "goal_met":world.progress>=1,
                "graph_proposed":step.observable_empirical_route_proposed,
                "epistemic_proposed":step.contextual_experiment_proposed}));
        }
        let solved=world.progress>=1;
        successes+=usize::from(solved);
        if ep<6{first_six+=usize::from(solved);}
        if ep>=EPISODES-6{last_six+=usize::from(solved);}
        episodes.push(json!({"ep":ep,"solved":solved,
            "actions":trace.len(),"actual_actions":trace}));
    }
    let report=rt.unified_report().unwrap();
    assert_eq!(report.total_external_cost,actions as u64);
    let agency=report.agency_credit.unwrap();
    assert_eq!(agency.externally_forced_actions,0);
    assert_eq!(agency.self_selected_actions+agency.option_continuation_actions,
        actions as u64);
    let journal=rt.construction_learner().unwrap().records();
    assert_eq!(journal.len(),actions);
    assert_eq!(journal.iter().filter(|r|r.transition.restart).count(),EPISODES);
    let option=report.temporal_options.unwrap();
    assert_eq!(option.observable_graph_enabled,graph);
    assert_eq!(option.contextual_experiment_selected,context as u64);
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap(),cp);
    json!({"family":family,"roles":roles,"rich_actual_sensor":rich,
        "graph_enabled":graph,
        "successes":successes,"real_actions":actions,
        "first_six":first_six,"last_six":last_six,
        "context_selected":context,
        "graph_native_selected":graph_selected,
        "graph_offered":graph_offered,
        "graph_actually_completed":graph_completed,
        "completed_depths":option.learned_depth_completed,
        "episodes":episodes})
}
fn mix(mut n:u64)->u64{
    n=n.wrapping_add(0x9e3779b97f4a7c15);
    n=(n^(n>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n=(n^(n>>27)).wrapping_mul(0x94d049bb133111eb);
    n^(n>>31)
}
fn role_maps(family:usize,count:usize)->Vec<[usize;4]>{
    let mut maps=Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            if a==b{continue}
            for c in 0..4 {
                if c==a||c==b{continue}
                let d=(0..4).find(|&m|m!=a&&m!=b&&m!=c).unwrap();
                maps.push([a,b,c,d]);
            }
        }
    }
    assert_eq!(maps.len(),24);
    let mut seed=mix(0x6ba4_6c4d_1701_32f0 ^ family as u64);
    for i in (1..maps.len()).rev(){
        seed=mix(seed);maps.swap(i,(seed%(i as u64+1)) as usize);
    }
    maps.truncate(count);maps
}

#[test]
fn native_u1_reuses_real_observable_transition_paths_vs_blind_and_unadapted(){
    let mut cases=Vec::new();
    let mut goals=[0usize;3];
    let mut physical_actions=[0usize;3];
    let mut family_goals=[[0usize;3];4];
    let mut family_actions=[[0usize;3];4];
    let mut graph_selected=[0usize;4];
    let mut graph_proposed=[0usize;4];
    let mut graph_completed=[0usize;4];
    let mut early=[0usize;3];
    let mut late=[0usize;3];
    let mut worlds_regressed=0usize;
    for family in 0..4{
        for roles in role_maps(family,if family<2{12}else{6}){
            let out=[
                lifetime(family,roles,false,false),
                lifetime(family,roles,true,false),
                lifetime(family,roles,true,true),
            ];
            if out[2]["successes"].as_u64().unwrap()<
                out[1]["successes"].as_u64().unwrap(){worlds_regressed+=1;}
            for (arm,row) in out.iter().enumerate(){
                let n=row["successes"].as_u64().unwrap() as usize;
                let cost=row["real_actions"].as_u64().unwrap() as usize;
                goals[arm]+=n;physical_actions[arm]+=cost;
                family_goals[family][arm]+=n;
                family_actions[family][arm]+=cost;
                early[arm]+=row["first_six"].as_u64().unwrap() as usize;
                late[arm]+=row["last_six"].as_u64().unwrap() as usize;
            }
            for (list,name) in [
                (&mut graph_selected,"graph_native_selected"),
                (&mut graph_proposed,"graph_offered"),
                (&mut graph_completed,"graph_actually_completed"),
            ]{
                list[family]+=out[2][name].as_u64().unwrap() as usize;
            }
            cases.push(json!({"family":family,"roles":roles,
                "blind":out[0],"rich_no_graph":out[1],
                "rich_factual_graph":out[2]}));
        }
    }
    let pass=goals[2]>=goals[1]+100 &&
        physical_actions[2].saturating_mul(10)<=
            physical_actions[1].saturating_mul(11) &&
        family_goals[2][2]>=100 &&
        family_goals[0][2]>=100 &&
        family_goals[1][2]>=4 &&
        family_goals[3][2]>=144 &&
        graph_selected.iter().sum::<usize>()>0 &&
        graph_completed.iter().sum::<usize>()>0;
    let summary=json!({
        "verdict":if pass{"OBSERVABLE_GRAPH1_ENGINEERING_PASS"}else{
            "NEGATIVE_OBSERVABLE_GRAPH_DEVELOPMENT"
        },
        "precommitted_gate_passed":pass,
        "source_disclosed":true,"independent_science":false,
        "worlds":cases.len(),"episodes_per_arm":36*EPISODES,
        "configurations":"blind;rich_no_graph;rich_real_graph",
        "goals":goals,"physical_actions":physical_actions,
        "family_goals":family_goals,"family_actions":family_actions,
        "native_U1_selected_real_graph_options":graph_selected,
        "real_graph_options_proposed":graph_proposed,
        "actual_completed_real_graph_options":graph_completed,
        "early_six":early,"late_six":late,
        "rich_graph_per_world_regressions":worlds_regressed,
        "forced_motor_tuition":0,"real_POST_for_each_motor":true
    });
    std::fs::create_dir_all("observable-graph-evidence").unwrap();
    std::fs::write("observable-graph-evidence/physical_episode_traces.json",
        serde_json::to_vec_pretty(&json!({"summary":summary,"worlds":cases}))
            .unwrap()).unwrap();
    eprintln!("OBSERVABLE_GRAPH1 {}",summary);
    assert_eq!(cases.len(),36);
    assert!(pass,"precommitted behavior criteria not met");
}
#[test]
fn pre_graph_v1_joint_checkpoint_keeps_unset_fields_omitted(){
    let rt=organism(false);
    let cp=rt.checkpoint_constructing().unwrap();
    let json:serde_json::Value=serde_json::from_slice(&cp).unwrap();
    let lane=json["cognition"]["temporal_options"].as_object().unwrap();
    assert!(!lane.contains_key("observable_graph_enabled"),
        "new disabled flag must not change legacy version-1 checkpoint hash");
    assert_eq!(cp,ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap());
    let graph=organism(true);
    assert!(graph.unified_report().unwrap().temporal_options.unwrap()
        .observable_graph_enabled);
    let cp=graph.checkpoint_constructing().unwrap();
    assert_eq!(cp,ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap());
}
