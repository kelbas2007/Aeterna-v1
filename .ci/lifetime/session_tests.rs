//! Tests cross real disk and OS process boundaries; no imagined POSTs.
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::construction::epistemic::EvidencePolicy;
use aeterna_v1::construction::session::PersistentSession;
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::vm::Error;
use aeterna_v1::construction::{Config,ExternalPost};use aeterna_v1::HumanProtectionEvidence;
use serde_json::{json,Value};
use std::{fs,io::{BufRead,BufReader,Write},path::PathBuf,process::{Child,ChildStdin,ChildStdout,Command,Stdio},sync::atomic::{AtomicU64,Ordering}};
static NEXT:AtomicU64=AtomicU64::new(0);
struct Folder(PathBuf);
impl Folder{fn new()->Self{Self(std::env::temp_dir().join(format!("aeterna-session-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed))))}}
impl Drop for Folder{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn safe(_:usize)->HumanProtectionEvidence{HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
fn config()->Config{Config{minimum_training:32,search_every:32,shadow_steps:4,journal_capacity:256,search:SearchConfig{work:50_000,max_expr_nodes:1,max_state:0,max_updates:1,..Default::default()},..Default::default()}}
fn policy(budget:u64)->AutonomyPolicy{AutonomyPolicy{max_external_cost:budget,evidence:Some(EvidencePolicy::default()),..Default::default()}}
fn offers()->[ActionOffer;2]{[ActionOffer{available:true,max_cost:1};2]}
fn feed(s:&mut PersistentSession,begin:usize,end:usize){for i in begin..end{s.step(None,&offers(),safe,|_|Ok(ExternalPost{observation:vec![i64::from(i%5!=0)],cost:1})).unwrap();}}
#[test] fn single_writer_lock_is_released_by_drop_not_by_a_second_open(){let d=Folder::new();let first=PersistentSession::open(&d.0).unwrap();assert!(PersistentSession::open(&d.0).is_err());drop(first);let _second=PersistentSession::open(&d.0).unwrap();}
#[test] fn persistent_live_trial_is_not_restarted_or_refunded(){
    let d=Folder::new();let mut s=PersistentSession::open(&d.0).unwrap();s.start(config(),policy(200),vec![0]).unwrap();feed(&mut s,0,48);let before=s.runtime().unwrap().checkpoint().unwrap();let evidence=s.runtime().unwrap().learner().epistemic_report().unwrap();assert_eq!(evidence.pending.as_ref().unwrap().samples,16);let c=s.continuity().unwrap().unwrap();drop(s);
    let mut s=PersistentSession::open(&d.0).unwrap();assert!(s.runtime().is_err());let mut wrong=c.clone();wrong.next_event+=1;assert_eq!(s.resume(&wrong),Err(Error::Stale));s.resume(&c).unwrap();assert_eq!(s.runtime().unwrap().checkpoint().unwrap(),before);feed(&mut s,48,160);assert!(s.runtime().unwrap().learner().report().promotions>0);
    let records=s.runtime().unwrap().learner().records();assert!(records.windows(2).all(|p|p[1].permit_sequence>p[0].permit_sequence));assert_eq!(s.runtime().unwrap().learner().autonomy_report().unwrap().external_cost,160);assert_eq!(fs::read_dir(&d.0).unwrap().count(),2);
}
#[test] fn spent_external_budget_survives_a_new_runtime(){let d=Folder::new();let mut s=PersistentSession::open(&d.0).unwrap();s.start(config(),policy(1),vec![0]).unwrap();feed(&mut s,0,1);let c=s.continuity().unwrap().unwrap();drop(s);let mut s=PersistentSession::open(&d.0).unwrap();s.resume(&c).unwrap();assert_eq!(s.step(None,&offers(),safe,|_|panic!("budget")),Err(Error::Blocked));assert_eq!(s.continuity().unwrap().unwrap(),c);}
#[test] fn emergency_latch_survives_process_store_and_cannot_be_cleared_by_start(){let d=Folder::new();let mut s=PersistentSession::open(&d.0).unwrap();s.start(config(),policy(20),vec![0]).unwrap();assert_eq!(s.step(None,&offers(),|a|{let mut e=safe(a);e.emergency_stop=true;e},|_|panic!("blocked")),Err(Error::Blocked));let c=s.continuity().unwrap().unwrap();drop(s);let mut s=PersistentSession::open(&d.0).unwrap();assert_eq!(s.resume(&c),Err(Error::Blocked));assert_eq!(s.start(config(),policy(20),vec![0]),Err(Error::Busy));}
#[test] fn bad_receipt_is_durable_and_not_retryable_after_reopen(){let d=Folder::new();let mut s=PersistentSession::open(&d.0).unwrap();s.start(config(),policy(20),vec![0]).unwrap();assert!(s.step(None,&offers(),safe,|_|Err(Error::Execution("unknown external effect".into()))).is_err());let c=s.continuity().unwrap().unwrap();drop(s);let mut s=PersistentSession::open(&d.0).unwrap();assert_eq!(s.resume(&c),Err(Error::Blocked));}
#[test] fn freeze_is_persistent_and_cannot_be_undone_by_rollback(){let d=Folder::new();let mut s=PersistentSession::open(&d.0).unwrap();s.start(config(),policy(200),vec![0]).unwrap();feed(&mut s,0,128);s.freeze().unwrap();let b=s.runtime().unwrap().checkpoint().unwrap();let c=s.continuity().unwrap().unwrap();drop(s);let mut s=PersistentSession::open(&d.0).unwrap();s.resume(&c).unwrap();assert_eq!(s.rollback(),Err(Error::Busy));assert_eq!(b,s.runtime().unwrap().checkpoint().unwrap());feed(&mut s,128,144);assert_eq!(s.runtime().unwrap().learner().autonomy_report().unwrap().frozen_at,Some(128));}
#[test] fn leftover_intent_requires_reconciliation_even_with_a_valid_saved_state(){let d=Folder::new();let mut s=PersistentSession::open(&d.0).unwrap();s.start(config(),policy(20),vec![0]).unwrap();drop(s);fs::write(d.0.join("pending.json"),b"{}").unwrap();assert!(PersistentSession::open(&d.0).is_err());assert!(d.0.join("pending.json").exists());}
#[test] fn broken_state_is_not_silently_replaced_with_a_cold_lifetime(){let d=Folder::new();fs::create_dir_all(&d.0).unwrap();fs::write(d.0.join("state.json"),b"{}").unwrap();assert!(PersistentSession::open(&d.0).is_err());assert_eq!(fs::read(d.0.join("state.json")).unwrap(),b"{}");}
struct Host{child:Child,input:ChildStdin,output:BufReader<ChildStdout>}
impl Host{
    fn launch(d:&Folder)->(Self,Value){let mut child=Command::new(env!("CARGO_BIN_EXE_aeterna-construction")).arg(&d.0).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();let input=child.stdin.take().unwrap();let output=BufReader::new(child.stdout.take().unwrap());let mut h=Self{child,input,output};let ready=h.read();(h,ready)}
    fn send(&mut self,v:Value){serde_json::to_writer(&mut self.input,&v).unwrap();writeln!(&mut self.input).unwrap();self.input.flush().unwrap();}
    fn read(&mut self)->Value{let mut line=String::new();assert!(self.output.read_line(&mut line).unwrap()>0,"child exited without receipt");serde_json::from_str(&line).unwrap()}
    fn step_request()->Value{json!({"op":"step","offers":offers(),"hazards":[{"human_present":false,"physical_effect_possible":false,"predicted_harm_probability":0.0,"hazard_confidence":1.0,"emergency_stop":false},{"human_present":false,"physical_effect_possible":false,"predicted_harm_probability":0.0,"hazard_confidence":1.0,"emergency_stop":false}]})}
    fn start(&mut self){self.send(json!({"op":"start","config":config(),"policy":policy(200),"observation":[0]}));assert_eq!(self.read()["event"],"status");}
    fn step(&mut self,i:usize)->Value{self.send(Self::step_request());let a=self.read();assert_eq!(a["event"],"act");assert_eq!(a["id"],i);self.send(json!({"op":"post","event":i,"action":a["action"],"observation":[i64::from(i%5!=0)],"cost":1}));self.read()}
}
impl Drop for Host{fn drop(&mut self){let _=self.child.kill();let _=self.child.wait();}}
#[test] fn actual_process_exit_and_reattachment_preserve_unfinished_evidence(){
    let d=Folder::new();let(mut h,ready)=Host::launch(&d);assert!(ready["continuity"].is_null());h.start();let mut state=Value::Null;for i in 0..48{state=h.step(i);assert_eq!(state["event"],"status");}assert_eq!(state["evidence"]["pending"]["samples"],16);let c=state["continuity"].clone();h.send(json!({"op":"close"}));assert!(h.child.wait().unwrap().success());drop(h);
    let(mut h,ready)=Host::launch(&d);assert_eq!(ready["continuity"],c);h.send(json!({"op":"resume","continuity":c}));assert_eq!(h.read()["evidence"],state["evidence"]);for i in 48..160{state=h.step(i);assert_eq!(state["event"],"status");}assert!(state["learner"]["promotions"].as_u64().unwrap()>0);assert_eq!(state["autonomy"]["external_cost"],160);
}
#[test] fn killed_process_after_act_never_replays_an_uncertain_external_action(){let d=Folder::new();let(mut h,_)=Host::launch(&d);h.start();h.send(Host::step_request());assert_eq!(h.read()["event"],"act");h.child.kill().unwrap();h.child.wait().unwrap();drop(h);assert!(d.0.join("pending.json").exists());let output=Command::new(env!("CARGO_BIN_EXE_aeterna-construction")).arg(&d.0).output().unwrap();assert!(!output.status.success());assert!(output.stdout.is_empty());assert!(String::from_utf8(output.stderr).unwrap().contains("no automatic retry"));}
#[test] fn mismatched_process_receipt_poison_is_saved_before_the_error_response(){let d=Folder::new();let(mut h,_)=Host::launch(&d);h.start();h.send(Host::step_request());let a=h.read();h.send(json!({"op":"post","event":99,"action":a["action"],"observation":[1],"cost":1}));assert_eq!(h.read()["event"],"error");h.send(json!({"op":"status"}));let s=h.read();assert_eq!(s["learner"]["poisoned"],true);drop(h);let(mut h,ready)=Host::launch(&d);h.send(json!({"op":"resume","continuity":ready["continuity"]}));assert_eq!(h.read()["event"],"error");}
