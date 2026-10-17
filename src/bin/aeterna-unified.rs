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
    if std::env::args().nth(1).as_deref()==Some("--school"){return school_console();}
    if std::env::args().nth(1).as_deref()==Some("--word-problems"){return word_console();}
    let stdin=io::stdin();let stdout=io::stdout();let mut input=stdin.lock();let mut out=stdout.lock();
    let mut runtime:Option<ScientificRuntime>=None;
    loop {
        let request=read(&mut input)?;
        let response:Result<Value>=(||{
            match request["op"].as_str().ok_or("missing op")? {
                "init"=>{
                    if runtime.is_some(){return Err("already initialized".into());}
                    let (cfg,policy)=configuration();runtime=Some(ScientificRuntime::new_constructing(cfg,policy,vec![0;32],"interactive-v1")?);
                    Ok(json!({"type":"initialized"}))
                },
                "begin_education_extension"=>{runtime.as_mut().ok_or("not loaded")?.begin_education_extension()?;Ok(json!({"type":"education_open"}))},
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
