//! JSONL protocol: no LLM, shell, network or privileged world introspection.
use aeterna_v1::construction::{Config,ExternalPost};
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::construction::session::{Continuity,PersistentSession};
use aeterna_v1::construction::vm::{Error,Result};
use aeterna_v1::HumanProtectionEvidence;
use serde::Deserialize;use serde_json::{json,Value};use std::io::{self,BufRead,Write};
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct Hazard {human_present:bool,physical_effect_possible:bool,predicted_harm_probability:f32,hazard_confidence:f32,emergency_stop:bool}
impl From<&Hazard> for HumanProtectionEvidence {fn from(h:&Hazard)->Self {Self{human_present:h.human_present,physical_effect_possible:h.physical_effect_possible,predicted_harm_probability:h.predicted_harm_probability,hazard_confidence:h.hazard_confidence,emergency_stop:h.emergency_stop}}}
#[derive(Deserialize)] #[serde(tag="op",rename_all="snake_case",deny_unknown_fields)]
enum Command {
    Start{config:Config,policy:AutonomyPolicy,observation:Vec<i64>},Resume{continuity:Continuity},
    Step{goal:Option<Vec<Option<i64>>>,offers:Vec<ActionOffer>,hazards:Vec<Hazard>},
    Post{event:u64,action:usize,observation:Vec<i64>,cost:u64},Status,Freeze,Rollback,Close,
}
fn emit(out:&mut impl Write,value:Value)->Result<()> {
    serde_json::to_writer(&mut *out,&value).map_err(|e|Error::Execution(e.to_string()))?;
    out.write_all(b"\n").and_then(|_|out.flush()).map_err(|e|Error::Execution(e.to_string()))
}
fn read(input:&mut impl BufRead)->Result<Option<Command>> {
    let mut line=Vec::new();loop {
        let buffer=input.fill_buf().map_err(|e|Error::Execution(e.to_string()))?;
        if buffer.is_empty(){if line.is_empty(){return Ok(None)}else{break;}}
        let count=buffer.iter().position(|&b|b==b'\n').map(|i|i+1).unwrap_or(buffer.len());
        if line.len()+count>65536 {return Err(Error::Capacity);}
        let done=buffer[count-1]==b'\n';line.extend_from_slice(&buffer[..count]);input.consume(count);if done{break;}
    }
    serde_json::from_slice(&line).map(Some).map_err(|_|Error::InvalidData)
}
fn status(s:&PersistentSession)->Value {
    match s.runtime(){Ok(r)=>json!({"event":"status","continuity":s.continuity().ok().flatten(),"learner":r.learner().report(),"autonomy":r.learner().autonomy_report(),"evidence":r.learner().epistemic_report()}),Err(_)=>json!({"event":"resume_required","continuity":s.continuity().ok().flatten()})}
}
fn run()->Result<()> {
    let mut args=std::env::args().skip(1);let dir=args.next().ok_or(Error::Execution("usage: aeterna-construction SESSION_DIRECTORY".into()))?;
    if args.next().is_some(){return Err(Error::InvalidData);}
    let mut session=PersistentSession::open(dir)?;let stdin=io::stdin();let mut input=stdin.lock();let stdout=io::stdout();let mut out=stdout.lock();
    emit(&mut out,json!({"event":"ready","protocol":1,"continuity":session.continuity()?}))?;
    while let Some(command)=read(&mut input)? {
        let result=match command {
            Command::Start{config,policy,observation}=>session.start(config,policy,observation),Command::Resume{continuity}=>session.resume(&continuity),
            Command::Freeze=>session.freeze(),Command::Rollback=>session.rollback(),Command::Status=>Ok(()),Command::Close=>break,Command::Post{..}=>Err(Error::Stale),
            Command::Step{goal,offers,hazards}=>{
                if offers.len()!=hazards.len(){Err(Error::InvalidData)}else{
                    let event=session.runtime()?.learner().records().len() as u64;
                    session.step(goal.as_deref(),&offers,|a|HumanProtectionEvidence::from(&hazards[a]),|a|{
                        emit(&mut out,json!({"event":"act","action":a,"id":event}))?;
                        match read(&mut input)? {Some(Command::Post{event:id,action,observation,cost}) if id==event && action==a=>Ok(ExternalPost{observation,cost}),_=>Err(Error::Execution("missing or mismatched external receipt".into()))}
                    }).map(|_|())
                }
            },
        };
        if let Err(e)=result {emit(&mut out,json!({"event":"error","error":e.to_string()}))?;}else{emit(&mut out,status(&session))?;}
    }
    Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("{e}");std::process::exit(1);}}
