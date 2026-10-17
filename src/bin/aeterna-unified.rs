//! JSONL transport for a trusted VIRTUAL environment adapter.
//! No browser, network, filesystem mutation, or physical-action driver.
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use aeterna_v1::construction::{Config,Error};
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::construction::epistemic::EvidencePolicy;
use aeterna_v1::HumanProtectionEvidence;
use serde_json::{Value,json};
use std::io::{self,BufRead,Read,Write};
type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;
fn read<R:BufRead>(input:&mut R)->Result<Value>{
    let mut line=String::new();input.take(34*1024*1024).read_line(&mut line)?;
    if line.is_empty(){return Err("transport closed".into());}
    if !line.ends_with('\n'){return Err("unterminated or oversized request".into());}
    Ok(serde_json::from_str(&line)?)
}
fn write<W:Write>(out:&mut W,v:&Value)->Result<()>{serde_json::to_writer(&mut *out,v)?;out.write_all(b"\n")?;out.flush()?;Ok(())}
fn safe(_:usize)->HumanProtectionEvidence{HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
fn configuration()->(Config,AutonomyPolicy){
    let cfg=Config{width:32,actions:4,search:SearchConfig{work:100_000,max_expr_nodes:3,max_frontier:512,max_updates:4,max_state:0,deterministic:false},minimum_training:32,search_every:16,shadow_steps:12,journal_capacity:1024,planning_depth:2,planning_work:50_000};
    let policy=AutonomyPolicy{ceiling:SearchConfig{work:300_000,max_expr_nodes:5,max_frontier:2048,max_updates:8,max_state:1,deterministic:false},failures_per_expansion:2,max_external_cost:10000,evidence:Some(EvidencePolicy::default())};
    (cfg,policy)
}
fn main()->Result<()>{
    let stdin=io::stdin();let stdout=io::stdout();let mut input=stdin.lock();let mut out=stdout.lock();
    let mut runtime:Option<ScientificRuntime>=None;
    loop {
        let request=read(&mut input)?;
        let response:Result<Value>=(||{
            match request["op"].as_str().ok_or("missing op")? {
                "load"=>{
                    if runtime.is_some(){return Err("already initialized".into());}
                    let bytes=request["checkpoint"].as_str().ok_or("checkpoint must be a string")?.as_bytes();
                    runtime=Some(ScientificRuntime::from_constructing_checkpoint(bytes)?);
                    Ok(json!({"type":"loaded","report":runtime.as_ref().unwrap().unified_report()}))
                },
                "enter"=>{
                    let (cfg,policy)=configuration();let initial=serde_json::from_value(request["observation"].clone())?;
                    let name=request["name"].as_str().ok_or("missing sensor contract")?;
                    runtime.as_mut().ok_or("not loaded")?.enter_constructing_domain(name,cfg,policy,initial)?;
                    Ok(json!({"type":"entered","report":runtime.as_ref().unwrap().unified_report()}))
                },
                "step"=>{
                    let available:Vec<bool>=serde_json::from_value(request["available"].clone())?;
                    if available.len()!=4{return Err("requires four primitive action channels".into());}
                    let offers:Vec<_>=available.into_iter().map(|available|ActionOffer{available,max_cost:1}).collect();
                    let goal:Option<Vec<Option<i64>>>=serde_json::from_value(request.get("goal").cloned().unwrap_or(Value::Null))?;
                    let step=runtime.as_mut().ok_or("not loaded")?.step_constructing(goal.as_deref(),&offers,safe,|action|{
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
            Err(e)=>{write(&mut out,&json!({"type":"error","message":e.to_string()}))?;return Err(e);}
        }
    }
    Ok(())
}
