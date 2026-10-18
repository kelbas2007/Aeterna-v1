//! Application consolidation regressions, not intelligence benchmarks.
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};
const MAIN: &str = env!("CARGO_BIN_EXE_aeterna-v1");
const ALIAS: &str = env!("CARGO_BIN_EXE_aeterna-unified");
fn protocol(binary: &str, messages: &[Value]) -> Vec<Value> {
 let mut child=Command::new(binary).arg("--jsonl").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
 let mut input=child.stdin.take().unwrap();for message in messages{writeln!(input,"{message}").unwrap();}drop(input);
 let o=child.wait_with_output().unwrap();assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));
 String::from_utf8(o.stdout).unwrap().lines().map(|s|serde_json::from_str(s).unwrap()).collect()
}
fn messages()->Vec<Value>{
 let examples:Vec<_>=(0..32).map(|i|{let(a,b)=(3+i,2+(i*7)%29);json!({"question":format!("sum {a} и {b}?"),"answer":a+b,"source":format!("entrypoint-fixture:{i}")})}).collect();
 let limits=json!({"work":30000,"evaluations":64,"max_nodes":11,"frontier":64,"edits_per_expansion":12});
 vec![json!({"op":"init"}),json!({"op":"teach","examples":examples}),json!({"op":"enable_recursive_search","config":{"object":limits,"meta":limits,"max_lifetime_work":1000000}}),json!({"op":"ask","question":"sum 137 и 58?","mode":"ComputeOnly"}),json!({"op":"recursive_search_report"}),json!({"op":"checkpoint"}),json!({"op":"quit"})]
}
#[test]fn aliases_have_identical_help_and_version(){for flag in ["--help","--version"]{let a=Command::new(MAIN).arg(flag).output().unwrap();let b=Command::new(ALIAS).arg(flag).output().unwrap();assert!(a.status.success()&&b.status.success());assert_eq!(a.stdout,b.stdout);assert!(String::from_utf8(a.stdout).unwrap().contains("AETERNA-v1"));}}
#[test]fn status_distinguishes_code_from_knowledge(){let o=Command::new(MAIN).arg("--status").output().unwrap();assert!(o.status.success());let v:Value=serde_json::from_slice(&o.stdout).unwrap();assert_eq!(v["checkpoint_loaded"],false);assert_eq!(v["recursive_admission"]["new_states"],2);assert_eq!(v["recursive_admission"]["feedback_features"],20);assert_eq!(v["canonical_binary"],"aeterna-v1");}
#[test]fn invalid_arguments_do_not_enter_another_mode(){for args in [vec!["--unknown"],vec!["--school"],vec!["--jsonl","extra"]]{let o=Command::new(MAIN).args(args).stdin(Stdio::null()).output().unwrap();assert!(!o.status.success());assert!(o.stdout.is_empty());}}
#[test]fn original_piped_protocol_still_works(){let mut c=Command::new(MAIN).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();c.stdin.take().unwrap().write_all(b"{\"op\":\"quit\"}\n").unwrap();let o=c.wait_with_output().unwrap();assert!(o.status.success());assert_eq!(serde_json::from_slice::<Value>(&o.stdout).unwrap()["type"],"bye");}
#[test]fn school_and_rsp2_share_one_state_and_restore(){let a=protocol(MAIN,&messages());let b=protocol(ALIAS,&messages());assert_eq!(a,b);assert_eq!(a[3]["result"]["answer"],"195");assert_eq!(a[4]["state"]["admission_version"],2);assert_eq!(a[4]["state"]["generation"],0);let state=a[5]["checkpoint"].as_str().unwrap();let restored=protocol(MAIN,&[json!({"op":"load","checkpoint":state}),json!({"op":"ask","question":"sum 137 и 58?","mode":"ComputeOnly"}),json!({"op":"checkpoint"}),json!({"op":"quit"})]);assert_eq!(restored[1]["result"]["answer"],"195");assert_eq!(restored[2]["checkpoint"],state);}
#[test]fn inspection_is_read_only(){let a=protocol(MAIN,&messages());let state=a[5]["checkpoint"].as_str().unwrap();let p=std::env::temp_dir().join(format!("aeterna-project-inspect-{}.json",std::process::id()));let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(&p).unwrap();f.write_all(state.as_bytes()).unwrap();drop(f);let o=Command::new(MAIN).arg("--inspect").arg(&p).output().unwrap();let after=std::fs::read(&p).unwrap();std::fs::remove_file(p).unwrap();assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));assert_eq!(after,state.as_bytes());let v:Value=serde_json::from_slice(&o.stdout).unwrap();assert_eq!(v["read_only"],true);assert_eq!(v["external_actions_executed"],0);assert!(!v["school"].is_null());assert_eq!(v["recursive_search"]["generation"],0);}
#[test]fn inventory_does_not_initialize_or_change_state(){let out=protocol(MAIN,&[json!({"op":"project_status"}),json!({"op":"project_status"}),json!({"op":"quit"})]);assert_eq!(out[0],out[1]);assert_eq!(out[0]["project"]["checkpoint_loaded"],false);}
