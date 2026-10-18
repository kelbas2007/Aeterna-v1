//! RSP2 acquisition and engineering check. Historical RSP1 evidence is unchanged.
//! Does not call an LLM, read a target implementation, or execute external actions.
mod rsp1_support;
use rsp1_support::*;
use aeterna_v1::construction::{Config,autonomy::AutonomyPolicy};
use aeterna_v1::construction::search_policy::*;
use aeterna_v1::scientific_runtime::ScientificRuntime;
use serde_json::{json,Value};
use std::path::{Path,PathBuf};
use std::fs;
type Result<T>=std::result::Result<T,Box<dyn std::error::Error>>;
fn save(path:impl AsRef<Path>,value:&Value)->Result<()>{let bytes=serde_json::to_vec(value)?;fs::write(path,bytes)?;Ok(())}
fn train(root:&Path)->Result<()> {
 if root.exists(){return Err("Refusing to overwrite a previous experiment. Choose a new directory.".into());}
 fs::create_dir_all(root)?;let mut summaries=Vec::new();
 for seed in 101..=108u64 {
  let clock=std::time::Instant::now();
  let mut rt=ScientificRuntime::new_constructing(Config{width:3,..Default::default()},AutonomyPolicy::default(),vec![0;3],"RSP1-numeric-teaching")?;
  rt.enable_recursive_search(config())?;
  let (lessons,_)=tasks(seed*100,8,false);let mut actual_gates=Vec::new();
  for attempt in 0..24u64 {
   let gate=admission(seed*80000+attempt*100_000);
   let a=rt.improve_recursive_search(&lessons,&gate,seed*1000+attempt)?;
   eprintln!("seed {seed} attempt {attempt} generation {} accepted {} gate solved {} -> {}",a.generation_after,a.accepted,a.before.solved,a.after.solved);
   actual_gates.push(gate);
  }
  rt.freeze_recursive_search()?;
  let bytes=rt.checkpoint_constructing()?;
  let restored=ScientificRuntime::from_constructing_checkpoint(&bytes)?;
  if restored.checkpoint_constructing()?!=bytes{return Err("checkpoint failed exact replay".into());}
  fs::write(root.join(format!("trained-{seed}.json")),&bytes)?;
  let state=rt.construction_learner().unwrap().recursive_search().unwrap();state.validate()?;
  save(root.join(format!("development-{seed}.json")),&json!({"seed":seed,"lessons":lessons,"gates":actual_gates,"evolution":state,"runtime_report":rt.unified_report()}))?;
  summaries.push(json!({"seed":seed,"generation":state.generation(),"work":state.work(),"policy":state.policy(),"checkpoint_bytes":bytes.len(),"checkpoint_fingerprint":aeterna_v1::construction::vm::fingerprint(&bytes),"wall_seconds":clock.elapsed().as_secs_f64()}));
 }
  save(root.join("training-summary.json"),&json!({"kind":"RSP2_DEVELOPMENT_ONLY","seeds":summaries,"attempts_per_seed":24,"object_limits":limits(),"meta_limits":meta(),"test_outcomes_read":false}))?;Ok(())
}
fn examine(root:&Path)->Result<()> {
 if root.join("exam-summary.json").exists(){return Err("Exam already consumed; replay into a separate copy, never retune on it.".into());}
 let mut summaries=Vec::new();
 for seed in 101..=108u64 {
  let bytes=fs::read(root.join(format!("trained-{seed}.json")))?;
  let rt=ScientificRuntime::from_constructing_checkpoint(&bytes)?;let state=rt.construction_learner().unwrap().recursive_search().unwrap();
  if !state.is_frozen(){return Err("policy must be frozen".into());}
  let learned=state.policy().clone();let cold=SearchPolicy::default();let strong=loss_size();let fi=fifo();
  let mut families=serde_json::Map::new();
  for (name,transfer,offset) in [("related",false,0u64),("transfer",true,1)] {
   let (sup,probes)=tasks(0xDEC000+seed*4000+offset*1000,64,transfer);let testseed=0xA77000+seed*100+offset;
   let mut arms=serde_json::Map::new();
   for (n,p) in [("learned",&learned),("cold",&cold),("loss_size",&strong),("fifo",&fi)] {
    arms.insert(n.into(),tested(p,&sup,&probes,testseed)?);
   }
   save(root.join(format!("frozen-{seed}-{name}.json")),&json!({"seed":seed,"family":name,"support":sup,"probe_tasks":probes,"arms":arms,"search_seed":testseed}))?;
   let counts:serde_json::Map<String,Value>=arms.iter().map(|(k,v)|(k.clone(),json!({"fit":v["score"]["solved"],"generalizing":v["generalizing"],"false_fits":v["false_fits"],"value":v["score"]["value"],"work":v["score"]["work"]}))).collect();
   families.insert(name.into(),Value::Object(counts));
  }
  // Producer swap: identical cold TARGET, known library and data. Only the
  // producer changes. Children are scored on fresh tasks unavailable in their
  // creation objective. No child replaces the frozen deployment policy.
  let mut ignition=Vec::new();
  for rep in 0..8u64 {
   let (sup,_)=tasks(0x510000+seed*9000+rep*64,8,false);
   let (gate,probes)=tasks(0x910000+seed*10000+rep*128,32,false);
   let searchseed=0x718000+seed*100+rep;let testseed=0xA10000+seed*200+rep;
   let mut arms=serde_json::Map::new();
   for (name,producer) in [("learned",&learned),("cold",&cold),("loss_size",&strong)] {
    let now=std::time::Instant::now();
    let result=produce(producer,&cold,&sup,&limits(),&meta(),searchseed)?;
    let child=SearchPolicy::from_code(&result.program)?;
    let trial=tested(&child,&gate,&probes,testseed)?;
    arms.insert(name.into(),json!({"producer":producer,"same_target":cold,"production":result,"child":child,"next_tasks":trial,"wall_seconds":now.elapsed().as_secs_f64()}));
   }
   eprintln!("producer swap seed {seed}, pair {rep}: {} vs {} child solves",arms["learned"]["next_tasks"]["generalizing"],arms["cold"]["next_tasks"]["generalizing"]);
   ignition.push(json!({"pair":rep,"support":sup,"test_tasks":gate,"probe_tasks":probes,"search_seed":searchseed,"test_seed":testseed,"arms":arms}));
  }
  let summary_pairs:Vec<_>=ignition.iter().map(|x|{
   let a=&x["arms"]["learned"];let b=&x["arms"]["cold"];let c=&x["arms"]["loss_size"];
   json!({"pair":x["pair"],"learned_child_value":a["next_tasks"]["score"]["value"],"cold_child_value":b["next_tasks"]["score"]["value"],"strong_child_value":c["next_tasks"]["score"]["value"],"learned_child_solves":a["next_tasks"]["generalizing"],"cold_child_solves":b["next_tasks"]["generalizing"],"strong_child_solves":c["next_tasks"]["generalizing"],"learned_creation_cost":a["production"]["work"],"cold_creation_cost":b["production"]["work"],"strong_creation_cost":c["production"]["work"]})
  }).collect();
  save(root.join(format!("producer-swap-{seed}.json")),&json!({"seed":seed,"pairs":ignition}))?;
  summaries.push(json!({"seed":seed,"generation":state.generation(),"acquisition_work":state.work(),"frozen":families,"producer_swap":summary_pairs}));
  if bytes!=rt.checkpoint_constructing()?{return Err("evaluation altered frozen state".into());}
 }
  save(root.join("exam-summary.json"),&json!({"kind":"RSP2_FROZEN_DEVELOPMENT_DISTRIBUTION_TEST","independent_external_benchmark":false,"seeds":summaries,"immutable_after_exam":true,"solver_and_producer_roles_separate":true}))?;Ok(())
}
fn repair_check(root:&Path)->Result<()> {
 if root.exists(){return Err("Refusing to overwrite a previous repair check".into());}
 fs::create_dir_all(root)?;
 let mut cfg=config();cfg.meta.work=20_000_000;cfg.meta.evaluations=32;
 save(root.join("protocol.json"),&json!({
  "kind":"RSP2_ENGINEERING_CHECK","seeds":[201,202,203,204],"attempts_per_seed":8,
  "config":cfg,"admission_families":[0,1,2,3,4,5,6,7],"new_exam_families":[8,9,10,11],
  "exam_tasks_per_group":32,"controls":["cold","loss_size"],"all_metered_search_and_validation_work_charged":true,
  "not_a_wall_clock_speedup_claim":true,"source":include_str!("../src/construction/search_policy.rs")
 }))?;
 let whole=std::time::Instant::now();let mut summaries=Vec::new();
 for seed in 201..=204u64 {
  let clock=std::time::Instant::now();
  let mut rt=ScientificRuntime::new_constructing(Config{width:3,..Default::default()},AutonomyPolicy::default(),vec![0;3],"RSP2-repair-check")?;
  rt.enable_recursive_search(cfg.clone())?;
  let lessons=tasks(seed*1000,8,false).0;let mut attempts=Vec::new();
  for attempt in 0..8u64 {
   let gate=admission(10_000_000+seed*1_000_000+attempt*100_000);
   let a=rt.improve_recursive_search(&lessons,&gate,seed*1000+attempt)?;
   let q=a.qualification.as_ref().ok_or("missing qualification")?;
   eprintln!("repair seed {seed} attempt {attempt}: accepted={} direct={} producer={} generation={}",a.accepted,q.direct_pass(&cfg.object),q.producer_pass(&cfg.object),a.generation_after);
   attempts.push(json!({"attempt":attempt,"accepted":a.accepted,"direct_pass":q.direct_pass(&cfg.object),"producer_pass":q.producer_pass(&cfg.object),
    "producer_pairs":q.producers.len(),"work":a.consumed_work,"retention_cases":q.retention.after.search.tasks,
    "retention_solved":q.retention.after.solved()}));
  }
  rt.freeze_recursive_search()?;
  let bytes=rt.checkpoint_constructing()?;let restored=ScientificRuntime::from_constructing_checkpoint(&bytes)?;
  if bytes!=restored.checkpoint_constructing()?{return Err("repair checkpoint was not exact".into());}
  fs::write(root.join(format!("trained-{seed}.json")),&bytes)?;
  let state=rt.construction_learner().unwrap().recursive_search().unwrap();state.validate()?;
  let mut groups=serde_json::Map::new();
  for (name,first) in [("related",0),("admission_transfer",4),("new_structure",8)] {
   let (support,probes)=family_tasks(0xC200000+seed*10000+first as u64*1000,32,first);
   let mut arms=serde_json::Map::new();
   for (label,policy) in [("learned",state.policy().clone()),("cold",SearchPolicy::default()),("loss_size",loss_size())] {
    arms.insert(label.into(),tested(&policy,&support,&probes,0xA200000+seed*100+first as u64)?);
   }
   groups.insert(name.into(),json!({"support":support,"probes":probes,"arms":arms}));
  }
  save(root.join(format!("seed-{seed}.json")),&json!({"seed":seed,"attempts":attempts,"groups":groups,"state":state,"exact_checkpoint":true}))?;
  let counts:serde_json::Map<String,Value>=groups.iter().map(|(k,v)|{
   let arms:serde_json::Map<String,Value>=v["arms"].as_object().unwrap().iter().map(|(a,s)|(a.clone(),json!({"solves":s["generalizing"],"work":s["score"]["work"],"false_fits":s["false_fits"]}))).collect();
   (k.clone(),Value::Object(arms))
  }).collect();
  summaries.push(json!({"seed":seed,"generation":state.generation(),"retained":state.retained().len(),"acquisition_work":state.work(),
   "attempts":attempts,"groups":counts,"checkpoint_bytes":bytes.len(),"wall_seconds":clock.elapsed().as_secs_f64()}));
 }
 save(root.join("summary.json"),&json!({"kind":"RSP2_ENGINEERING_CHECK","seeds":summaries,"wall_seconds":whole.elapsed().as_secs_f64()}))?;
 Ok(())
}
fn main()->Result<()> {
 let args:Vec<_>=std::env::args().collect();let mode=args.get(1).map(String::as_str).unwrap_or("help");
 let root=PathBuf::from(args.get(2).map(String::as_str).unwrap_or("rsp1-session"));
  match mode {"train"=>train(&root),"examine"=>examine(&root),"repair-check"=>repair_check(&root),_=>{println!("AETERNA recursive_search: train DIR, examine DIR, or repair-check DIR.\nUse a NEW directory. All seeds and failures remain in the report. This executable is not a conversational AI.");Ok(())}}
}
