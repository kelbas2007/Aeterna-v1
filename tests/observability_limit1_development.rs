//! OBSERVABILITY-LIMIT1: source-disclosed SAME-U1 true-sensor ablation.
//! Preregistration committed BEFORE this evaluator: docs/OBSERVABILITY_LIMIT1_PROTOCOL.md
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
fn organism()->ScientificRuntime{
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
    rt.enable_constructing_autobiographical_options().unwrap();
    rt.enable_constructing_variable_depth_options().unwrap();
    rt.enable_constructing_complexity_need_gate().unwrap();
    rt.enable_constructing_active_causal_probes().unwrap();
    rt.enable_constructing_contextual_experiments().unwrap();
    assert!(rt.construction_learner().unwrap().records().is_empty());
    assert_eq!(rt.unified_report().unwrap().native_meta_observations,0);
    rt
}
fn lifetime(family:usize,roles:[usize;4],rich:bool)->Value{
    let mut rt=organism();
    let mut successes=0usize;
    let mut actions=0usize;
    let mut first_six=0usize;
    let mut last_six=0usize;
    let mut context=0usize;
    let mut episodes=Vec::new();
    for ep in 0..EPISODES{
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
            trace.push(json!({"motor":step.action,"source":step.source,
                "physical_POST":world.observation(rich),
                "goal_met":world.progress>=1,
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
    assert_eq!(option.contextual_experiment_selected,context as u64);
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap(),cp);
    json!({"family":family,"roles":roles,"rich_actual_sensor":rich,
        "successes":successes,"real_actions":actions,
        "first_six":first_six,"last_six":last_six,
        "context_selected":context,
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
fn same_cold_u1_with_flat_versus_truly_informative_stage_sensor(){
    let mut results=Vec::new();
    let mut goals=[0usize;2];
    let mut actions=[0usize;2];
    let mut groups=[[0usize;2];4];
    let mut cost_groups=[[0usize;2];4];
    let mut context=[[0usize;2];4];
    let mut first=[0usize;2];
    let mut last=[0usize;2];
    let mut depths=[[0u64;6];2];
    let mut regressions=0usize;
    for family in 0..4{
        for role in role_maps(family,if family<2{12}else{6}){
            let pair=[lifetime(family,role,false),lifetime(family,role,true)];
            if pair[1]["successes"].as_u64().unwrap()<
                pair[0]["successes"].as_u64().unwrap(){
                regressions+=1;
            }
            for (arm,v) in pair.iter().enumerate(){
                let n=v["successes"].as_u64().unwrap() as usize;
                let c=v["real_actions"].as_u64().unwrap() as usize;
                goals[arm]+=n;actions[arm]+=c;
                groups[family][arm]+=n;cost_groups[family][arm]+=c;
                context[family][arm]+=v["context_selected"].as_u64().unwrap() as usize;
                first[arm]+=v["first_six"].as_u64().unwrap() as usize;
                last[arm]+=v["last_six"].as_u64().unwrap() as usize;
                for depth in 2..=5{
                    depths[arm][depth]+=v["completed_depths"][depth].as_u64().unwrap();
                }
            }
            results.push(json!({"family":family,"role_map":role,
                "blind":pair[0],"rich":pair[1]}));
        }
    }
    let positive=groups[1][1]>groups[1][0] &&
        groups[1][1]>=12 && groups[2][1]>=groups[2][0] &&
        groups[3][1]>=groups[3][0] && goals[1]>goals[0] &&
        actions[1].saturating_mul(10)<=actions[0].saturating_mul(11);
    let summary=json!({
        "verdict":if positive{"POSITIVE_TRUE_SENSOR_UTILITY_DEVELOPMENT"}else{
            "NEGATIVE_SENSOR_ADDITION_UNDER_EXISTING_U1"},
        "source_disclosed":true,"independent_qualification":false,
        "self_created_sensor":false,"actual_motor_tuition":0,
        "worlds":results.len(),"episodes_per_arm":36*EPISODES,
        "blind_vs_rich_goals":goals,"blind_vs_rich_real_actions":actions,
        "family_goals":groups,"family_real_actions":cost_groups,
        "context_selected":context,"first_six":first,"last_six":last,
        "actual_completed_depths":depths,"per_world_regressions":regressions});
    std::fs::create_dir_all("observability-evidence").unwrap();
    std::fs::write("observability-evidence/actual_episode_traces.json",
        serde_json::to_vec_pretty(&json!({"summary":summary,"worlds":results}))
            .unwrap()).unwrap();
    eprintln!("OBSERVABILITY_LIMIT1 {}",summary);
    assert_eq!(results.len(),36);
}
#[test]
fn irrelevant_extra_sensor_is_masked_and_never_a_goal(){
    let mut rt=organism();
    let mut w=World::new(1,[0,1,2,3]);
    let step=rt.step_constructing(Some(&[Some(1),None]),&[OFFER;4],
        safe,|action|{
            w.act(action);
            Ok(UnifiedPost{observation:w.observation(true),cost:1,reward:0.0})
        }).unwrap();
    assert_eq!(step.event,1);
    assert_eq!(rt.unified_report().unwrap().total_external_cost,1);
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(cp,ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap());
}
