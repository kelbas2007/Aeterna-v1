//! JSONL transport for a trusted VIRTUAL environment adapter.
//! No browser, network, filesystem mutation, or physical-action driver.
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,SensorPost};
use aeterna_v1::construction::{Config,Error};
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::construction::epistemic::EvidencePolicy;
use aeterna_v1::HumanProtectionEvidence;
use serde_json::{Value,json};
use std::io::{self,BufRead,Read,Write};
type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;
fn read<R:BufRead>(input:&mut R)->Result<Value>{
    // A 32 MiB checkpoint can double when escaped into a JSONL string.
    let mut line=String::new();input.take(66*1024*1024).read_line(&mut line)?;
    if line.is_empty(){return Err("transport closed".into());}
    if !line.ends_with('\n'){return Err("unterminated or oversized request".into());}
    Ok(serde_json::from_str(&line)?)
}
fn write<W:Write>(out:&mut W,v:&Value)->Result<()>{serde_json::to_writer(&mut *out,v)?;out.write_all(b"\n")?;out.flush()?;Ok(())}
fn safe(_:usize)->HumanProtectionEvidence{HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
fn decode_sensor_post(value:Value)->Result<SensorPost>{
    // Recover the known cost before decoding sensor-specific content. An
    // invalid frame/reward must reach the ordinary cost-bearing reject path.
    let cost=value.get("cost").and_then(Value::as_u64).ok_or("unknown or invalid actual sensor cost")?;
    let reward=serde_json::from_value::<f32>(value["reward"].clone());
    let frame=serde_json::from_value(value["frame"].clone());
    match (reward,frame) {
        (Ok(reward),Ok(frame))=>Ok(SensorPost{frame,cost,reward}),
        _=>Ok(SensorPost{cost,reward:f32::NAN,
            frame:aeterna_v1::construction::sensors::SensorFrame::Image{namespace:0,pixels:Some(Vec::new())}}),
    }
}
fn configuration()->(Config,AutonomyPolicy){
    let cfg=Config{width:32,actions:4,search:SearchConfig{work:100_000,max_expr_nodes:3,max_frontier:512,max_updates:4,max_state:0,deterministic:false},minimum_training:32,search_every:16,shadow_steps:12,journal_capacity:1024,planning_depth:2,planning_work:50_000};
    let policy=AutonomyPolicy{ceiling:SearchConfig{work:300_000,max_expr_nodes:5,max_frontier:2048,max_updates:8,max_state:1,deterministic:false},failures_per_expansion:2,max_external_cost:10000,evidence:Some(EvidencePolicy::default())};
    (cfg,policy)
}
pub(crate) fn main()->Result<()>{
    use std::io::IsTerminal;
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--help" | "-h") if args.len()==2 => { print!("{}", PROJECT_HELP); return Ok(()); },
        Some("--version") if args.len()==2 => { println!("AETERNA-v1 {} / project-20261020", env!("CARGO_PKG_VERSION")); return Ok(()); },
        Some("--status") if args.len()==2 => { println!("{}", serde_json::to_string_pretty(&project_status())?); return Ok(()); },
        Some("--inspect") if args.len()==3 => {
            let bytes=std::fs::read(&args[2])?;
            let rt=ScientificRuntime::from_constructing_checkpoint(&bytes)?;
            let recursive=rt.construction_learner().and_then(|l|l.recursive_search()).map(|s|json!({
                "generation":s.generation(),"work":s.work(),"attempts":s.audits().len(),
                "retained_cases":s.retained().len(),"frozen":s.is_frozen()
            }));
            println!("{}",serde_json::to_string_pretty(&json!({"type":"checkpoint_inspection",
                "organism":rt.unified_report(),"school":rt.school_report(),"recursive_search":recursive,
                "learned_requirements":rt.requirements_report(),
                "read_only":true,"external_actions_executed":0}))?);
            return Ok(());
        },
        Some("--school" | "--relations" | "--word-problems") if args.len()==3 => {},
        Some("--jsonl") if args.len()==2 => {},
        None if io::stdin().is_terminal() => { print!("{}",PROJECT_HELP); return Ok(()); },
        None => {}, // Retain the original piped JSONL adapter contract.
        _ => return Err("invalid arguments; run aeterna-v1 --help".into()),
    }

    if std::env::args().nth(1).as_deref()==Some("--relations"){return relation_console();}
    if std::env::args().nth(1).as_deref()==Some("--school"){return school_console();}
    if std::env::args().nth(1).as_deref()==Some("--word-problems"){return word_console();}
    let stdin=io::stdin();let stdout=io::stdout();let mut input=stdin.lock();let mut out=stdout.lock();
    let mut runtime:Option<ScientificRuntime>=None;
    loop {
        let request=read(&mut input)?;
        let response:Result<Value>=(||{
            match request["op"].as_str().ok_or("missing op")? {
                "project_status"=>Ok(json!({"type":"project_status","project":project_status()})),
                "external_init"=>{
                    if runtime.is_some(){return Err("already initialized".into());}
                    let frame=serde_json::from_value(request["frame"].clone())?;
                    let (mut cfg,mut policy)=configuration();
                    cfg.actions=16;cfg.planning_depth=usize::try_from(request.get("planning_depth").map_or(Some(32),Value::as_u64).ok_or("invalid planning_depth")?)?;
                    cfg.planning_work=1_000_000;cfg.search_every=1_000_000;cfg.journal_capacity=512;
                    policy.max_external_cost=1_000_000;
                    let mut rt=ScientificRuntime::new_constructing(cfg,policy,vec![0;32],"external-continuous-v5")?;
                    rt.enable_constructing_self_directed_control()?;rt.enable_constructing_agency_aware_credit()?;
                    rt.enable_constructing_causal_memory(aeterna_v1::construction::causal_memory::CausalConfig {
                        history_depth:3,model_entries:256,model_bytes:512*1024,outcomes_per_entry:4,planning_states:64})?;
                    rt.enable_constructing_external_reward_memory(request.get("reward_memory").map_or(Some(true),Value::as_bool).ok_or("invalid reward_memory")?,
                        request.get("adaptive").map_or(Some(true),Value::as_bool).ok_or("invalid adaptive")?)?;
                    rt.observe_external_sensor_restart(&frame)?;runtime=Some(rt);
                    Ok(json!({"type":"initialized","action_authority":"protected_u1","learner":"one_joint_checkpoint"}))
                },
                "external_reset"=>{
                    let frame=serde_json::from_value(request["frame"].clone())?;
                    runtime.as_mut().ok_or("not loaded")?.observe_external_sensor_restart(&frame)?;
                    Ok(json!({"type":"external_restarted"}))
                },
                "external_episode_end"=>{
                    runtime.as_mut().ok_or("not loaded")?.finish_external_episode()?;
                    Ok(json!({"type":"external_episode_finished"}))
                },
                "external_reward_report"=>Ok(json!({"type":"external_reward_report","report":runtime.as_ref().ok_or("not loaded")?
                    .construction_learner().ok_or("not constructing")?.external_reward_report()})),
                "external_step"=>{
                    let offers:Vec<ActionOffer>=serde_json::from_value(request["offers"].clone())?;
                    let remaining=request.get("remaining_actions").map(|v|v.as_u64().ok_or("invalid remaining_actions").and_then(|n|usize::try_from(n).map_err(|_|"remaining_actions overflow"))).transpose()?;
                    let step=runtime.as_mut().ok_or("not loaded")?.step_external_sensors(&offers,remaining,safe,|action|{
                        let result:Result<SensorPost>=(||{
                            write(&mut out,&json!({"type":"action","action":action}))?;
                            let post=read(&mut input)?;
                            if post["op"]!="post"{return Err("expected actual sensor POST, no retry permitted".into());}
                            decode_sensor_post(post["result"].clone())
                        })();
                        result.map_err(|e|Error::Execution(e.to_string()))
                    })?;
                    Ok(json!({"type":"committed","step":step}))
                },
                "init"=>{
                    if runtime.is_some(){return Err("already initialized".into());}
                    let (mut cfg,mut policy)=configuration();
                    // Backward-compatible opt-in sensor width. The ordinary
                    // JSONL interface still defaults to 32 channels; a
                    // virtual world may expose 1..32 factual sensor values for
                    // bounded self-directed exploration without motor tuition.
                    if let Some(w)=request.get("width") {
                        cfg.width=usize::try_from(w.as_u64().ok_or("width must be an integer")?)?;
                    }
                    for (field,value) in [("actions",&mut cfg.actions),("planning_depth",&mut cfg.planning_depth),
                        ("journal_capacity",&mut cfg.journal_capacity)] {
                        if let Some(v)=request.get(field) {*value=usize::try_from(v.as_u64().ok_or("configuration must be an unsigned integer")?)?;}
                    }
                    if let Some(v)=request.get("planning_work"){cfg.planning_work=v.as_u64().ok_or("invalid planning_work")?;}
                    if let Some(v)=request.get("max_external_cost"){policy.max_external_cost=v.as_u64().ok_or("invalid max_external_cost")?;}
                    // Validate width BEFORE allocating a default observation.
                    // Untrusted u64::MAX must produce Capacity, not OOM/abort.
                    cfg.validate()?;
                    let initial=if let Some(v)=request.get("observation"){
                        serde_json::from_value::<Vec<i64>>(v.clone())?
                    }else{vec![0;cfg.width]};
                    runtime=Some(ScientificRuntime::new_constructing(cfg,policy,initial,"interactive-v1")?);
                    Ok(json!({"type":"initialized"}))
                },
                "enable_self_directed_control"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_self_directed_control()?;
                    Ok(json!({"type":"self_directed_enabled","action_authority":"protected_u1"}))
                },
                "enable_causal_memory"=>{
                    let config=serde_json::from_value(request.get("config").cloned().unwrap_or(json!({})))?;
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_causal_memory(config)?;
                    Ok(json!({"type":"causal_memory_enabled"}))
                },
                "causal_report"=>Ok(json!({"type":"causal_report","report":runtime.as_ref().ok_or("not loaded")?
                    .construction_learner().ok_or("not constructing")?.causal_report()})),
                "enable_agency_aware_credit"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_agency_aware_credit()?;
                    Ok(json!({"type":"agency_aware_credit_enabled",
                        "credit_rule":"only_if_more_than_one_feasible_motor",
                        "causal_observations":"always_record_real_post"}))
                },
                "enable_instrumental_credit"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_instrumental_credit()?;
                    Ok(json!({"type":"instrumental_credit_enabled",
                        "credit":"actual_goal_progress_only",
                        "epistemic_facts":"remain_in_real_journal"}))
                },
                "enable_predictive_history_states"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_predictive_history_states()?;
                    Ok(json!({"type":"predictive_history_states_enabled",
                        "proposal_authority":"u1_only","evidence":"real_executed_history"}))
                },
                "enable_adaptive_history_suffix"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_adaptive_history_suffix()?;
                    Ok(json!({"type":"adaptive_history_suffix_enabled",
                        "max_predecessor_actions":2,"context":"real_protected_journal",
                        "proposal_authority":"native_U1_only"}))
                },
                "enable_temporal_options"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_temporal_options()?;
                    Ok(json!({"type":"temporal_options_enabled",
                        "authority":"one_native_U1_choice_then_bounded_continuation",
                        "every_physical_step":"fresh_HumanProtection_and_real_POST"}))
                },
                "enable_autobiographical_options"=>{
                    runtime.as_mut().ok_or("not loaded")?
                        .enable_constructing_autobiographical_options()?;
                    Ok(json!({"type":"autobiographical_options_enabled",
                        "memory":"own_chronological_protected_POST_only",
                        "authority":"one_native_U1_choice_then_freshly_protected_steps",
                        "confidence":"tentative_until_real_replay"}))
                },
                "enable_variable_depth_options"=>{
                    runtime.as_mut().ok_or("not loaded")?
                        .enable_constructing_variable_depth_options()?;
                    Ok(json!({"type":"variable_depth_options_enabled",
                        "memory":"minimum_observed_contrast_supported_suffix",
                        "depth_bound":"2_to_planning_depth_max_5",
                        "authority":"native_U1_and_fresh_HumanProtection"}))
                },
                "enable_complexity_need_gate"=>{
                    runtime.as_mut().ok_or("not loaded")?
                        .enable_constructing_complexity_need_gate()?;
                    Ok(json!({"type":"complexity_need_gate_enabled",
                        "decision":"retain_simpler_investigate_or_extend",
                        "source":"real_matched_history_consequences",
                        "admission":"incremental_prediction_gain_exceeds_action_complexity_tax",
                        "authority":"same_U1_fresh_HumanProtection"}))
                },
                "enable_contextual_experiments"=>{
                    runtime.as_mut().ok_or("not loaded")?
                        .enable_constructing_contextual_experiments()?;
                    Ok(json!({"type":"contextual_experiments_enabled",
                        "source":"connected_real_PRE_action_POST_history",
                        "decision":"only_when_complexity_need_investigate",
                        "authority":"native_U1_fresh_HumanProtection_each_motor"}))
                },
                "enable_active_causal_probes"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_active_causal_probes()?;
                    Ok(json!({"type":"active_causal_probes_enabled",
                        "proposal_authority":"native_U1_only",
                        "outcomes":"real_protected_POST_only"}))
                },
                "enable_trajectory_self_revision"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_constructing_trajectory_self_revision()?;
                    Ok(json!({"type":"trajectory_self_revision_enabled",
                        "program_authority":"evaluation_only","action_authority":"human_protection"}))
                },
                "trajectory_self_revision_report"=>Ok(json!({"type":"trajectory_self_revision_report",
                    "report":runtime.as_ref().ok_or("not loaded")?.trajectory_self_revision_report()})),
                "enable_recursive_search"=>{
                    let config=serde_json::from_value(request["config"].clone())?;
                    runtime.as_mut().ok_or("not loaded")?.enable_recursive_search(config)?;
                    Ok(json!({"type":"recursive_search_enabled"}))
                },
                "enable_missing_computation_learning"=>{
                    runtime.as_mut().ok_or("not loaded")?.enable_missing_computation_learning()?;
                    Ok(json!({"type":"requirements_enabled"}))
                },
                "observe_executed_computations"=>{
                    let rows:Vec<aeterna_v1::construction::requirements::ExecutedOperation>=serde_json::from_value(request["observations"].clone())?;
                    let report=runtime.as_mut().ok_or("not loaded")?.observe_executed_computations(&rows)?;
                    Ok(json!({"type":"requirements_learned","report":report}))
                },
                "missing_computation_report"=>Ok(json!({"type":"requirements_report","report":runtime.as_ref().ok_or("not loaded")?.requirements_report()})),
                "complete_missing_computation"=>{
                    let task=serde_json::from_value(request["task"].clone())?;
                    let limits=serde_json::from_value(request.get("limits").cloned().unwrap_or(
                        serde_json::to_value(aeterna_v1::construction::requirements::CompletionLimits::default())?))?;
                    let report=if let Some(s)=request.get("sketch") {
                        let sketch=serde_json::from_value(s.clone())?;
                        runtime.as_mut().ok_or("not loaded")?.complete_missing_computation(&task,&sketch,&limits)?
                    }else{runtime.as_mut().ok_or("not loaded")?.discover_missing_computation(&task,&limits)?};
                    Ok(json!({"type":"missing_computation_result","result":report}))
                },
                "improve_recursive_search"=>{
                    let training:Vec<aeterna_v1::construction::search_policy::SynthesisTask>=serde_json::from_value(request["training"].clone())?;
                    let gate:aeterna_v1::construction::search_policy::AdmissionSuite=serde_json::from_value(request["gate"].clone())?;
                    let seed=request["seed"].as_u64().ok_or("missing seed")?;
                    let audit=runtime.as_mut().ok_or("not loaded")?.improve_recursive_search(&training,&gate,seed)?;
                    Ok(json!({"type":"recursive_search_audit","audit":audit}))
                },
                "solve_synthesis"=>{
                    let task=serde_json::from_value(request["task"].clone())?;
                    let limits=serde_json::from_value(request["limits"].clone())?;
                    let seed=request["seed"].as_u64().ok_or("missing seed")?;
                    let result=runtime.as_mut().ok_or("not loaded")?.solve_by_recursive_search(&task,&limits,seed)?;
                    Ok(json!({"type":"synthesis_result","result":result}))
                },
                "recursive_search_report"=>{
                    let rt=runtime.as_ref().ok_or("not loaded")?;
                    Ok(json!({"type":"recursive_search_report","state":rt.construction_learner().and_then(|l|l.recursive_search())}))
                },
                "freeze_recursive_search"=>{
                    runtime.as_mut().ok_or("not loaded")?.freeze_recursive_search()?;
                    Ok(json!({"type":"recursive_search_frozen"}))
                },
                "begin_education_extension"=>{runtime.as_mut().ok_or("not loaded")?.begin_education_extension()?;Ok(json!({"type":"education_open"}))},
                "install_quantity_relations"=>{
                    let model=serde_json::from_value(request.get("model").ok_or("missing model")?.clone())?;
                    runtime.as_mut().ok_or("not loaded")?.install_quantity_relations(model)?;
                    Ok(json!({"type":"quantity_relations_installed"}))
                }
                "ask_relational"=>{
                    let question=request.get("question").and_then(Value::as_str).ok_or("missing question")?;
                    Ok(json!({"type":"relational_answer","result":runtime.as_ref().ok_or("not loaded")?.answer_relational(question)?}))
                }
                "inspect_relational"=>{
                    let question=request.get("question").and_then(Value::as_str).ok_or("missing question")?;
                    let g=aeterna_v1::word_problems::ground(question)?;
                    let a=request.get("a").and_then(Value::as_u64).ok_or("missing a")? as usize;
                    let b=request.get("b").and_then(Value::as_u64).map(|v|v as usize);
                    Ok(json!({"features":aeterna_v1::quantity_relations::features(&g,a,b)?}))
                }
                "install_word_problems"=>{
                    let model=serde_json::from_value(request["model"].clone())?;
                    runtime.as_mut().ok_or("not loaded")?.install_word_problem_knowledge(model)?;
                    Ok(json!({"type":"word_problem_knowledge_installed"}))
                },
                "inspect_word_problem"=>{
                    let q=request["question"].as_str().ok_or("missing question")?;
                    let g=aeterna_v1::word_problems::ground(q)?;
                    let kind=request.get("kind").and_then(Value::as_str).unwrap_or("role");
                    Ok(json!({"type":"word_problem_inspection","features":aeterna_v1::word_problems::features(&g,kind),"ground":g}))
                },
                "ask_word_problem"=>{
                    let question=request["question"].as_str().ok_or("missing question")?;
                    let prior=request.get("prior_only").and_then(Value::as_bool).unwrap_or(false);
                    Ok(json!({"type":"word_problem_answer","result":runtime.as_ref().ok_or("not loaded")?.answer_word_problem(question,prior)?}))
                },
                "teach"=>{
                    let examples=serde_json::from_value(request["examples"].clone())?;
                    Ok(json!({"type":"lesson","result":runtime.as_mut().ok_or("not loaded")?.teach_school(examples)?}))
                },
                "facts"=>{
                    let facts=serde_json::from_value(request["facts"].clone())?;
                    Ok(json!({"type":"facts","inserted":runtime.as_mut().ok_or("not loaded")?.teach_school_facts(facts)?}))
                },
                "ask"=>{
                    let mode=serde_json::from_value(request.get("mode").cloned().unwrap_or(json!("All")))?;
                    let question=request["question"].as_str().ok_or("missing question")?;
                    Ok(json!({"type":"answer","result":runtime.as_ref().ok_or("not loaded")?.answer_school(question,mode)?}))
                },
                "enable_composition"=>{runtime.as_mut().ok_or("not loaded")?.enable_school_composition()?;Ok(json!({"type":"composition_enabled"}))},
                "understand"=>{
                    let question=request["question"].as_str().ok_or("missing question")?;
                    let limits=serde_json::from_value(request.get("limits").cloned().unwrap_or(serde_json::to_value(aeterna_v1::education::GroundingLimits::default())?))?;
                    Ok(json!({"type":"understanding","result":runtime.as_ref().ok_or("not loaded")?.understand_school(question,limits)?}))
                },
                "ask_surface"=>{
                    let mode=serde_json::from_value(request.get("mode").cloned().unwrap_or(json!("All")))?;
                    let question=request["question"].as_str().ok_or("missing question")?;
                    Ok(json!({"type":"answer","result":runtime.as_ref().ok_or("not loaded")?.answer_school_surface(question,mode)?}))
                },
                "clarify"=>{
                    let question=request["question"].as_str().ok_or("missing question")?;
                    let reply=request["reply"].as_i64().ok_or("reply must answer the generated distinguishing question")?;
                    Ok(json!({"type":"answer","result":runtime.as_ref().ok_or("not loaded")?.clarify_school(question,reply)?}))
                },
                "school_report"=>Ok(json!({"type":"school_report","report":runtime.as_ref().ok_or("not loaded")?.school_report(),"lessons":runtime.as_ref().unwrap().school_lessons()})),
                "freeze"=>{runtime.as_mut().ok_or("not loaded")?.freeze_constructing()?;Ok(json!({"type":"frozen"}))},
                "load"=>{
                    if runtime.is_some(){return Err("already initialized".into());}
                    let bytes=request["checkpoint"].as_str().ok_or("checkpoint must be a string")?.as_bytes();
                    runtime=Some(ScientificRuntime::from_constructing_checkpoint(bytes)?);
                    Ok(json!({"type":"loaded","report":runtime.as_ref().unwrap().unified_report()}))
                },
                "enter"=>{
                    // Retain the carrier dimensions and explicit limits,
                    // including configurations restored from a checkpoint.
                    let learner=runtime.as_ref().ok_or("not loaded")?.construction_learner().ok_or("no constructing learner")?;
                    let cfg=learner.configuration().clone();
                    let policy=learner.autonomy_policy().ok_or("no autonomy policy")?.clone();
                    let initial=serde_json::from_value(request["observation"].clone())?;
                    let name=request["name"].as_str().ok_or("missing sensor contract")?;
                    runtime.as_mut().ok_or("not loaded")?.enter_constructing_domain(name,cfg,policy,initial)?;
                    Ok(json!({"type":"entered","report":runtime.as_ref().unwrap().unified_report()}))
                },
                "step"=>{
                    let offers:Vec<ActionOffer>=if let Some(value)=request.get("offers") {
                        if request.get("available").is_some(){return Err("use offers or available, not both".into())}
                        serde_json::from_value(value.clone())?
                    }else{
                        let available:Vec<bool>=serde_json::from_value(request["available"].clone())?;
                        available.into_iter().map(|available|ActionOffer{available,max_cost:1}).collect()
                    };
                    if offers.len()!=runtime.as_ref().ok_or("not loaded")?.construction_learner()
                        .ok_or("not constructing")?.action_count(){return Err("incorrect primitive action channels".into());}
                    let goal:Option<Vec<Option<i64>>>=serde_json::from_value(request.get("goal").cloned().unwrap_or(Value::Null))?;
                    let remaining_actions=request.get("remaining_actions").map(|v|v.as_u64().ok_or("remaining_actions must be an unsigned integer").and_then(|n|usize::try_from(n).map_err(|_|"remaining_actions overflow"))).transpose()?;
                    let step=runtime.as_mut().ok_or("not loaded")?.step_constructing_with_action_limit(goal.as_deref(),&offers,remaining_actions,safe,|action|{
                        let result:Result<UnifiedPost>=(||{
                            write(&mut out,&json!({"type":"action","action":action}))?;
                            let post=read(&mut input)?;
                            if post["op"]!="post"{return Err("expected actual POST, no retry permitted".into());}
                            Ok(serde_json::from_value(post["result"].clone())?)
                        })();
                        result.map_err(|e|Error::Execution(e.to_string()))
                    })?;
                    Ok(json!({"type":"committed","step":step}))
                },
                "report"=>Ok(json!({"type":"report","report":runtime.as_ref().ok_or("not loaded")?.unified_report()})),
                "checkpoint"=>{
                    let bytes=runtime.as_ref().ok_or("not loaded")?.checkpoint_constructing()?;
                    Ok(json!({"type":"checkpoint","checkpoint":String::from_utf8(bytes)?}))
                },
                "quit"=>Ok(json!({"type":"bye"})),
                _=>Err("unknown operation".into())
            }
        })();
        match response {
            Ok(v)=>{write(&mut out,&v)?;if v["type"]=="bye"{break;}},
            Err(e)=>{
                let mut diagnostic=json!({"type":"error","message":e.to_string()});
                if request["op"]=="external_step" {
                    diagnostic["report"]=json!(runtime.as_ref().and_then(|rt|rt.unified_report()));
                    diagnostic["external_reward_report"]=json!(runtime.as_ref().and_then(|rt|rt.construction_learner()).and_then(|l|l.external_reward_report()));
                }
                write(&mut out,&diagnostic)?;return Err(e);
            }
        }
    }
    Ok(())
}

fn school_console()->Result<()> {
    let path=std::env::args().nth(2).ok_or("usage: aeterna-unified --school learned-school.json")?;
    let bytes=std::fs::read(path)?;let runtime=ScientificRuntime::from_constructing_checkpoint(&bytes)?;
    println!("AETERNA: acquired lessons and teacher memory. Type a question; /exit to stop.");
    println!("{:?}",runtime.school_report());
    let stdin=io::stdin();
    loop {
        print!("> ");io::stdout().flush()?;let mut line=String::new();if stdin.read_line(&mut line)?==0 || line.trim()=="/exit" {break;}
        if let Some(q)=line.trim().strip_prefix("/solve ") {
            println!("{}",serde_json::to_string_pretty(&runtime.answer_word_problem(q,false)?)?);continue;
        }
        if let Some(q)=line.trim().strip_prefix("/why ") {
            println!("{}",serde_json::to_string_pretty(&runtime.understand_school(q,aeterna_v1::education::GroundingLimits::default())?)?);continue;
        }
        match runtime.answer_school(line.trim(),aeterna_v1::education::AnswerMode::All) {
            Ok(result)=>{
                println!("{}",result.answer.as_deref().unwrap_or("Не могу однозначно связать вопрос с приобретёнными знаниями."));println!("[{}; {}]",result.method,result.reason);
                if result.reason=="ambiguous_program_meaning" {
                    if let Some(probe)=runtime.understand_school(line.trim(),aeterna_v1::education::GroundingLimits::default())?.distinguishing_probe {
                        println!("Уточни смысл на примере: {}. Какой ответ ты ожидаешь? (Enter — пропустить)",probe.question);
                        let mut reply=String::new();stdin.read_line(&mut reply)?;
                        if let Ok(n)=reply.trim().parse::<i64>() {let clarified=runtime.clarify_school(line.trim(),n)?;println!("{} [{}]",clarified.answer.as_deref().unwrap_or("Уточнение не выбрало единственный смысл."),clarified.reason);}
                    }
                }
            },
            Err(e)=>println!("Error: {e}"),
        }
    }Ok(())
}

fn word_console()->Result<()> {
    let path=std::env::args().nth(2).ok_or("usage: aeterna-unified --word-problems acquired-checkpoint.json")?;
    let runtime=ScientificRuntime::from_constructing_checkpoint(&std::fs::read(path)?)?;
    println!("AETERNA: learned English word problems. /exit to stop. Answers are hypotheses, not guaranteed facts.");
    let stdin=io::stdin();
    loop {
        print!("> ");io::stdout().flush()?;let mut line=String::new();
        if stdin.read_line(&mut line)?==0||line.trim()=="/exit"{break;}
        match runtime.answer_word_problem(line.trim(),false) {
            Ok(r)=>{
                println!("{} [{}]",r.answer.as_deref().unwrap_or("Unsupported input"),r.reason);
                if let Some(p)=r.interpretations.first(){println!("{} = {}",p.equation,p.answer);}
                if let Some(m)=r.margin{println!("Interpretation margin: {:.3}; not a probability.",m);}
            },Err(e)=>println!("Error: {e}")
        }
    }Ok(())
}

fn relation_console()->Result<()> {
    let path=std::env::args().nth(2).ok_or("usage: aeterna-unified --relations learned-relations.json")?;
    let runtime=ScientificRuntime::from_constructing_checkpoint(&std::fs::read(path)?)?;
    println!("AETERNA REL-1: learned quantity relations. English arithmetic questions; /exit to stop.");
    println!("These are hypotheses, not verified semantic understanding. Original Russian lessons: /school <question>.");
    let stdin=io::stdin();
    loop {
        print!("> ");io::stdout().flush()?;let mut line=String::new();
        if stdin.read_line(&mut line)?==0 || line.trim()=="/exit"{break;}
        if let Some(q)=line.trim().strip_prefix("/school ") {
            let a=runtime.answer_school(q,aeterna_v1::education::AnswerMode::All)?;
            println!("{}",a.answer.as_deref().unwrap_or("No supported interpretation"));continue;
        }
        match runtime.answer_relational(line.trim()) {
            Ok(a)=>{println!("{} [{}]",a.answer.as_deref().unwrap_or("No supported interpretation"),a.reason);
                for c in a.interpretations.iter().take(1){println!("{} = {}; used quantities {:?}",c.equation,c.answer,c.used);}},
            Err(e)=>println!("Rejected: {e}"),
        }
    }
    Ok(())
}


fn project_status()->Value {
    serde_json::from_str(include_str!("../../docs/PROJECT_STATE.json")).expect("checked static project manifest")
}
const PROJECT_HELP:&str = "AETERNA-v1 — one application, explicit research capabilities

  aeterna-v1 --status                 Implementation inventory; no state loaded
  aeterna-v1 --inspect STATE.json     Read-only checkpoint report
  aeterna-v1 --school STATE.json      Acquired Russian/English school expressions
  aeterna-v1 --relations STATE.json   Learned English quantity relations
  aeterna-v1 --word-problems STATE.json  Existing word-problem interface
  aeterna-v1 --jsonl                  Existing ScientificRuntime JSONL protocol
  aeterna-v1 --version

The aeterna-unified binary is a compatibility alias of this same code.
A learned checkpoint must be supplied explicitly; no pretrained state is invented.
RSP2: init, enable_recursive_search, improve_recursive_search(training, gate, seed),
solve_synthesis, recursive_search_report, checkpoint, quit.
Self-directed factual control: init(width<=32, actions<=16), enable_self_directed_control,
then step(goal,available). Optional opt-in credit:
enable_agency_aware_credit BEFORE collecting experience; the organism
still learns world effects from forced actions, not native choice weights.
Optional instrument-only utility: enable_instrumental_credit (after agency,
before first factual action); invisible history preparation gets credit
only after later REAL POST shows goal progress.
Optional predictive history states
(planning_work>=32000): enable_predictive_history_states.
Optional unforced causal pair experiments: enable_active_causal_probes.
Additional conditional past-length inference (opt-in): enable_adaptive_history_suffix.
For a certified two-action-suffix route, opt into persisted multi-step
native choice with enable_temporal_options; each physical step remains protected.
Own-history options (research opt-ins): enable_autobiographical_options,
then enable_variable_depth_options, then enable_complexity_need_gate,
all before the first physical experience. The gate may retain a simpler
model, investigate an unresolved contradiction, or offer a longer option
only when observed evidence pays its complexity cost.
JSONL init accepts planning_depth (1..32), planning_work, journal_capacity,
actions and max_external_cost. Legacy default planning_depth=2. Published cold 3..5-action
skills used a custom Rust ScientificRuntime Config with greater depth;
the flags alone do NOT reproduce the longer experiments.
Optional own evaluator synthesis:
enable_trajectory_self_revision, step(goal,available),
trajectory_self_revision_report. The evaluator changes U1 valuation only,
not the protected actuator boundary or task objectives.
LC1: enable_missing_computation_learning, observe_executed_computations,
complete_missing_computation, missing_computation_report; learned rules share
one checkpoint. Both LC1 and RSP3 remain candidate-only without RSP2 adoption.
Legacy RSP1 states remain readable; old admission history is not upgraded silently.
Virtual environments only. General intelligence and reliable self-improvement
have not been demonstrated. See docs/PROJECT_OVERVIEW.md.
";
