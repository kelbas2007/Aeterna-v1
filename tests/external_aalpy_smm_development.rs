//! EXTERNAL-BLACKBOX-SMM0: execute pinned AALpy stochastic SUL through
//! a live Python process; NEVER feed authored hidden states to AETERNA.
//! Preregistration: docs/EXTERNAL_BLACKBOX_SMM0_PROTOCOL.md.
//! DISCLOSED DEVELOPMENT ONLY, not a secret blind AGI benchmark.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::{json,Value};
use std::io::{BufRead,BufReader,Write};
use std::process::{Child,ChildStdin,ChildStdout,Command,Stdio};

const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};
const SEEDS:[u64;8]=[0xA110,0xB221,0xC332,0xD443,
                     0xE554,0xF665,0x1776,0x2887];
const EPISODES:usize=24;
const HORIZON:usize=24;

fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}

struct ThirdPartyBlackBox{
    process:Child,
    input:ChildStdin,
    output:BufReader<ChildStdout>,
}
impl ThirdPartyBlackBox{
    fn new()->Self{
        let mut process=Command::new("python")
            .arg("-u").arg("scripts/aalpy_blackbox_sul.py")
            .stdin(Stdio::piped()).stdout(Stdio::piped())
            .stderr(Stdio::inherit()).spawn()
            .expect("AALpy Python SUL must start");
        let input=process.stdin.take().unwrap();
        let output=BufReader::new(process.stdout.take().unwrap());
        Self{process,input,output}
    }
    fn request(&mut self,msg:Value)->Value{
        let mut serialized=serde_json::to_vec(&msg).unwrap();
        serialized.push(b'\n');
        self.input.write_all(&serialized).unwrap();
        self.input.flush().unwrap();
        let mut line=String::new();
        assert!(self.output.read_line(&mut line).unwrap()>0,
            "external stochastic black-box SUL unexpectedly terminated");
        let reply:Value=serde_json::from_str(&line).unwrap();
        assert_eq!(reply["ok"],json!(true),
            "third-party SUL failed: {reply}");
        reply
    }
    fn reset(&mut self,seed:u64,role_map:[usize;2]){
        let reply=self.request(json!({"op":"reset","seed":seed,"map":role_map}));
        assert_eq!(reply["observation"],json!([0,0]));
    }
    fn step(&mut self,action:usize)->Value{
        let result=self.request(json!({"op":"step","motor":action}));
        let s=result["output"].as_str().unwrap();
        assert!(matches!(s,"init"|"beep"|"coffee"));
        let observed=result["observation"].as_array().unwrap();
        assert_eq!(observed.len(),2);
        assert_eq!(result["cost"],json!(1));
        assert_eq!(observed[0],json!(i32::from(s=="coffee")));
        assert_eq!(observed[1],json!(i32::from(s=="beep")));
        result
    }
}
impl Drop for ThirdPartyBlackBox{
    fn drop(&mut self){
        let _=self.process.kill();
        let _=self.process.wait();
    }
}

fn cold(graph:bool)->ScientificRuntime{
    let cfg=Config{width:2,actions:2,minimum_training:128,
        search_every:4096,planning_depth:4,planning_work:120_000,
        journal_capacity:2048,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0,0],"external-aalpy-smm").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.enable_constructing_adaptive_history_suffix().unwrap();
    rt.enable_constructing_temporal_options().unwrap();
    if graph{rt.enable_constructing_observable_factual_routes().unwrap();}
    rt.enable_constructing_autobiographical_options().unwrap();
    rt.enable_constructing_variable_depth_options().unwrap();
    rt.enable_constructing_complexity_need_gate().unwrap();
    rt.enable_constructing_active_causal_probes().unwrap();
    rt.enable_constructing_contextual_experiments().unwrap();
    assert!(rt.construction_learner().unwrap().records().is_empty());
    assert_eq!(rt.unified_report().unwrap().native_meta_observations,0);
    rt
}
fn lifetime(seed:u64,role_map:[usize;2],graph:bool)->Value{
    let mut host=ThirdPartyBlackBox::new();
    let mut rt=cold(graph);
    let mut total_actions=0usize;
    let mut goal_successes=0usize;
    let mut first=0usize;
    let mut last=0usize;
    let mut native_graph_selected=0usize;
    let mut completed=Vec::new();
    let mut episodes=Vec::new();
    for episode in 0..EPISODES{
        let episode_seed=seed ^ (episode as u64)
            .wrapping_mul(0x9e3779b97f4a7c15);
        host.reset(episode_seed,role_map);
        if episode>0 {
            let old_len=rt.construction_learner().unwrap().records().len();
            rt.observe_constructing_external_restart(vec![0,0]).unwrap();
            assert_eq!(rt.construction_learner().unwrap().records().len(),old_len);
        }
        let mut trace=Vec::new();
        let mut hit=false;
        for _ in 0..HORIZON{
            let mut physical=None::<Value>;
            let step=rt.step_constructing(Some(&[Some(1),None]),
                &[OFFER;2],safe,|action|{
                    let response=host.step(action);
                    let post=response["observation"].as_array().unwrap().iter()
                        .map(|x|x.as_i64().unwrap()).collect::<Vec<_>>();
                    physical=Some(response);
                    Ok(UnifiedPost{observation:post,cost:1,reward:0.0})
                }).unwrap();
            let actual=physical.expect("physical callback must execute ONCE");
            total_actions+=1;
            native_graph_selected+=
                usize::from(step.source=="observable-empirical-route");
            let goal=actual["observation"][0]==json!(1);
            hit|=goal;
            trace.push(json!({"action":step.action,"source":step.source,
                "external_model_output":actual["output"],
                "physical_post":actual["observation"],"cost":1,
                "goal":goal}));
            if goal{break}
        }
        goal_successes+=usize::from(hit);
        if episode<6{first+=usize::from(hit);}
        if episode>=EPISODES-6{last+=usize::from(hit);}
        episodes.push(json!({"episode":episode,"source_seed":episode_seed,
            "solved":hit,"physical_steps":trace.len(),"real_SUL_steps":trace}));
    }
    let report=rt.unified_report().unwrap();
    assert_eq!(report.total_external_cost,total_actions as u64);
    let credit=report.agency_credit.unwrap();
    assert_eq!(credit.externally_forced_actions,0);
    assert_eq!(credit.self_selected_actions+
        credit.option_continuation_actions,total_actions as u64);
    let records=rt.construction_learner().unwrap().records();
    assert_eq!(records.len(),total_actions);
    assert_eq!(records.iter().filter(|r|r.transition.restart).count(),EPISODES);
    let lane=report.temporal_options.unwrap();
    assert_eq!(lane.observable_graph_enabled,graph);
    completed.extend_from_slice(&lane.learned_depth_completed);
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(cp,ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap());
    json!({"role_map":role_map,"seed":seed,"graph":graph,
        "goals":goal_successes,"actual_actions":total_actions,
        "first_six":first,"last_six":last,
        "native_graph_selected":native_graph_selected,
        "full_option_depths":completed,
        "episodes":episodes})
}

#[test]
fn external_stochastic_smm_real_behavior_vs_same_native_u1_without_graph(){
    let mut raw=Vec::new();
    let mut successes=[0usize;2];
    let mut actions=[0usize;2];
    let mut per_map=[[0usize;2];2];
    let mut graph_selections=0usize;
    let mut early=[0usize;2];
    let mut late=[0usize;2];
    for (map_i,role_map) in [[0,1],[1,0]].into_iter().enumerate(){
        for seed in SEEDS{
            let old=lifetime(seed,role_map,false);
            let new=lifetime(seed,role_map,true);
            for (arm,v) in [&old,&new].into_iter().enumerate(){
                let goals=v["goals"].as_u64().unwrap() as usize;
                successes[arm]+=goals;
                actions[arm]+=v["actual_actions"].as_u64().unwrap() as usize;
                per_map[map_i][arm]+=goals;
                early[arm]+=v["first_six"].as_u64().unwrap() as usize;
                late[arm]+=v["last_six"].as_u64().unwrap() as usize;
            }
            graph_selections+=new["native_graph_selected"].as_u64().unwrap()
                as usize;
            raw.push(json!({"role_map":role_map,"seed":seed,
                "baseline":old,"graph":new}));
        }
    }
    let relative_goals=successes[1].saturating_mul(100)>=
        successes[0].saturating_mul(105);
    let same_goals_cheaper=successes[1]==successes[0] &&
        actions[1].saturating_mul(10)<=actions[0].saturating_mul(9);
    let no_mapping_regression=(0..2).all(|map|
        per_map[map][1].saturating_add(5)>=per_map[map][0]);
    let cost_safe=actions[1].saturating_mul(10)<=
        actions[0].saturating_mul(11);
    let positive=(relative_goals||same_goals_cheaper) &&
        no_mapping_regression && cost_safe;
    let summary=json!({
        "verdict":if positive{"POSITIVE_EXTERNAL_SMM_ENGINEERING"}else{
            "NEGATIVE_EXTERNAL_STOCHASTIC_GRAPH_TRANSFER"
        },
        "source":"AALpy pinned 335c50a1b4b5a830d788f86b7c30a8cb6c73ecc0",
        "externally_authored_smm":true,
        "source_disclosed":true,"independent_one_use":false,
        "worlds":raw.len(),"episodes_per_arm":16*EPISODES,
        "role_maps":[[0,1],[1,0]],"goal_successes":successes,
        "actual_actions":actions,"goal_successes_by_role_map":per_map,
        "actual_graph_action_selections":graph_selections,
        "first_six":early,"last_six":late,
        "no_tutor_actions":true,"bounded_real_observation":true,
        "goal_pass":relative_goals,"same_goal_cost_pass":same_goals_cheaper,
        "mapping_retention_pass":no_mapping_regression,
        "action_cost_pass":cost_safe,
        "criteria_passed":positive
    });
    std::fs::create_dir_all("external-smm-evidence").unwrap();
    std::fs::write("external-smm-evidence/physical_sul_traces.json",
        serde_json::to_vec_pretty(&json!({"summary":summary,"paired_lives":raw}))
            .unwrap()).unwrap();
    eprintln!("EXTERNAL_SMM0 {}",summary);
    assert_eq!(raw.len(),16);
    assert!(positive,"source-disclosed stochastic transfer failed preregistered gate");
}
