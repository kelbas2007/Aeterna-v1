//! RSP3 policy-level experience study, **not** a recursive adoption or AGI test.
//! Train and examine use distinct fixed cohorts; the frozen report locks the
//! selected pure-VM policy before examining any reserved family.
mod rsp1_support;
use rsp1_support::{cases,loss_size};
use aeterna_v1::construction::{experiential::*,search_policy::*,vm::{Expr,Op,Library,Context,Meter}};
use serde_json::{json,Value};
use std::{fs,path::PathBuf};
type E=Box<dyn std::error::Error>;
fn config()->StudyConfig{StudyConfig{limits:SearchLimits{work:35_000,evaluations:96,max_nodes:11,frontier:128,edits_per_expansion:16},candidate_limit:128,shortlist:12,train_seed:0xEC0100,validation_seed:0xEC0200}}
fn group(seed:u64,count:usize,first:usize)->Vec<ValidationCase>{cases(seed,count,first)}
fn curriculum()->(Vec<ValidationCase>,Vec<ValidationCase>){
 let mut train=group(0xE110,24,0);train.extend(group(0xE120,24,4));
 let mut dev=group(0xE210,24,0);dev.extend(group(0xE220,24,4));(train,dev)
}
// Lock the actual search, mutation, VM and family generator implementations,
// not just the new candidate learner. This snapshot is written BEFORE exams.
fn sources()->Value{json!({
 "experiential":include_str!("../src/construction/experiential.rs"),
 "search_policy":include_str!("../src/construction/search_policy.rs"),
 "vm":include_str!("../src/construction/vm.rs"),
 "generator":include_str!("rsp1_support/mod.rs"),
 "exam_harness":include_str!("rsp3_experience.rs"),
})}
fn source_lock_valid(protocol:&Value,report:&StudyReport)->bool {
 protocol["sources"]==sources() && report.config==config()
}
fn train(root:PathBuf)->Result<(),E>{
 if root.exists(){return Err("Refusing to overwrite previous results".into())}fs::create_dir_all(&root)?;
 let (train,dev)=curriculum();let cfg=config();
 fs::write(root.join("protocol.json"),serde_json::to_vec_pretty(&json!({"kind":"RSP3_DEVELOPMENT_PROPOSAL","test_families_not_seen":[8,9,10,11],
 "train_seed":[0xE110,0xE120],"internal_dev_seed":[0xE210,0xE220],"test_seeds":[0xE310,0xE320,0xE330],"config":cfg,
 "baseline":["cold","loss_plus_size"],"ablation":"scrambled residual association","adoption":"NONE; producer gate still mandatory",
 "sources":sources()}))?)?;
 let r=study(&train,&dev,cfg)?;
 fs::write(root.join("report.json"),serde_json::to_vec_pretty(&r)?)?;
 let cold=evaluate_policy(&SearchPolicy::default(),&dev,&r.config.limits,r.config.validation_seed,false)?;
 let strong=evaluate_policy(&loss_size(),&dev,&r.config.limits,r.config.validation_seed,false)?;
 let chosen=evaluate_policy(&r.proposed,&dev,&r.config.limits,r.config.validation_seed,false)?;
 println!("{}",serde_json::to_string_pretty(&json!({"selected":r.proposed,"changed":r.policy_changed,"feedback_read":r.feedback_read,
 "candidate_count":r.candidate_count,"train_cases":r.training_cases,"dev_cases":r.validation_cases,"study_work":r.total_metered_work,
 "dev":{"learned":chosen.verified,"cold":cold.verified,"loss_size":strong.verified},"frozen":true,"rsp2_adoption":false}))?);Ok(())
}
fn examine(root:PathBuf)->Result<(),E>{
 if root.join("exam.json").exists(){return Err("Reserved exam already used".into())}
 let report:StudyReport=serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
 let protocol:Value=serde_json::from_slice(&fs::read(root.join("protocol.json"))?)?;
 if !source_lock_valid(&protocol,&report){return Err("Unfrozen generator/search/VM/exam code or config".into())}
 let mut result=serde_json::Map::new();
 for (name,first,seed) in [("related",0usize,0xE310u64),("transfer",4usize,0xE320),("reserved",8usize,0xE330)]{
   let cases=group(seed,64,first);let mut variants=serde_json::Map::new();
   for (label,p,scramble) in [("learned",report.proposed.clone(),false),("scrambled_residual",report.proposed.clone(),true),
       ("cold",SearchPolicy::default(),false),("loss_size",loss_size(),false)]{
     let s=evaluate_policy(&p,&cases,&report.config.limits,0xB000+seed,scramble)?;
     variants.insert(label.into(),serde_json::to_value(s)?);
   }
   result.insert(name.into(),json!({"family_start":first,"cases":64,"arms":variants}));
 }
 fs::write(root.join("exam.json"),serde_json::to_vec_pretty(&json!({"kind":"RSP3_RESERVED_DEVELOPMENT_EXAM","fresh_uncurated_external_benchmark":false,
 "frozen":true,"rsp2_adopted":false,"groups":result,"source_lock_valid":true}))?)?;
 println!("{}",serde_json::to_string_pretty(&result.iter().map(|(n,v)|{(n.clone(),json!({"learned":v["arms"]["learned"]["verified"],
 "scrambled":v["arms"]["scrambled_residual"]["verified"],"cold":v["arms"]["cold"]["verified"],
 "strong":v["arms"]["loss_size"]["verified"]}))}).collect::<serde_json::Map<String,Value>>())?);Ok(())
}
fn confirm(root:PathBuf)->Result<(),E>{
 if !root.join("exam.json").exists(){return Err("Primary reserved exam has not completed".into())}
 if root.join("confirmation.json").exists(){return Err("Confirmation already consumed".into())}
 let r:StudyReport=serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
 let frozen:Value=serde_json::from_slice(&fs::read(root.join("protocol.json"))?)?;
 if !source_lock_valid(&frozen,&r){return Err("Policy or relevant code is not frozen".into())}
 let mut batches=Vec::new();
 for cohort in 0..8u64 {
     let mut groups=serde_json::Map::new();
     for (name,first,offset) in [("related",0usize,0u64),("transfer",4,1),("reserved",8,2)]{
         let seed=0xF00000+cohort*0x10000+offset*0x1000;
         let cases=group(seed,24,first);
         let mut scores=serde_json::Map::new();
         for (label,policy,scramble) in [("learned",r.proposed.clone(),false),("cold",SearchPolicy::default(),false),
           ("loss_size",loss_size(),false),("scrambled",r.proposed.clone(),true)] {
             let score=evaluate_policy(&policy,&cases,&r.config.limits,seed^0xB000,scramble)?;
             scores.insert(label.to_owned(),json!({"verified":score.verified,"value":score.value,"work":score.work,"cases":score.verified_cases}));
         }
         groups.insert(name.to_owned(),json!({"seed":seed,"scores":scores}));
     }
     batches.push(json!({"cohort":cohort,"groups":groups}));
 }
 let receipt=json!({"kind":"POST_EXAM_REPLICATION_NOT_NEW_INDEPENDENT_QUALIFICATION","frozen_policy":r.proposed,
 "uses_same_policy_and_code_after_primary_exam":true,"batches":batches,"source_lock_valid":true});
 fs::write(root.join("confirmation.json"),serde_json::to_vec_pretty(&receipt)?)?;
 for name in ["related","transfer","reserved"]{
     let sums=|arm:&str|batches.iter().map(|x|x["groups"][name]["scores"][arm]["verified"].as_u64().unwrap()).sum::<u64>();
     println!("{name}: learned {}/192, cold {}, loss_size {}, scrambled {}",sums("learned"),sums("cold"),sums("loss_size"),sums("scrambled"));
 }
 Ok(())
}
fn producer(root:PathBuf)->Result<(),E>{
 if !root.join("exam.json").exists(){return Err("Freeze before producer check".into())}
 if root.join("producer.json").exists(){return Err("Producer check already consumed".into())}
 let r:StudyReport=serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
 let frozen:Value=serde_json::from_slice(&fs::read(root.join("protocol.json"))?)?;
 if !source_lock_valid(&frozen,&r){return Err("Policy or relevant code is not frozen".into())}
 let object=&r.config.limits;
 let meta=SearchLimits{work:8_000_000,evaluations:24,max_nodes:23,frontier:128,edits_per_expansion:12};
 let cold=SearchPolicy::default();let mut comparisons=Vec::new();
 for pair in 0..2u64{
     let (sup,_)=rsp1_support::family_tasks(0x712000+pair*10000,4,if pair==0{0}else{4});
     let cases=group(0x713000+pair*10000,16,if pair==0{4}else{8});
     let seed=0x715000+pair*1000;
     let mut arms=serde_json::Map::new();
     for (name,policy) in [("trained",&r.proposed),("cold",&cold)]{
         let created=produce(policy,&cold,&sup,object,&meta,seed)?;
         let child=SearchPolicy::from_code(&created.program)?;
         let outcome=evaluate_policy(&child,&cases,object,seed^0xC00D,false)?;
         arms.insert(name.to_string(),json!({"creator":policy,"creation_work":created.work,"child":child,
             "child_verified":outcome.verified,"child_value":outcome.value,"verified_cases":outcome.verified_cases}));
     }
     comparisons.push(json!({"pair":pair,"arms":arms}));
 }
 let report=json!({"kind":"EXPLORATORY_PRODUCER_SWAP_NOT_RSP2_ADMISSION","independent_creator_increase_established":false,
     "comparisons":comparisons});
 fs::write(root.join("producer.json"),serde_json::to_vec_pretty(&report)?)?;
 println!("{}",serde_json::to_string_pretty(&report)?);Ok(())
}
fn main()->Result<(),E>{let args:Vec<_>=std::env::args().collect();if args.len()!=3{return Err("Usage: rsp3_experience train|examine DIRECTORY".into())}
 match args[1].as_str(){"train"=>train(PathBuf::from(&args[2])),"examine"=>examine(PathBuf::from(&args[2])),"confirm"=>confirm(PathBuf::from(&args[2])),"producer"=>producer(PathBuf::from(&args[2])),_=>Err("Unknown mode".into())}}
