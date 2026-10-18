use aeterna_v1::construction::search_policy::*;
use aeterna_v1::construction::vm::{Context,Expr,Library,Meter,Op};
use serde_json::{Value,json};
pub fn limits()->SearchLimits{SearchLimits{work:100_000,evaluations:384,max_nodes:11,frontier:512,edits_per_expansion:24}}
pub fn meta()->SearchLimits{SearchLimits{work:100_000_000,evaluations:120,max_nodes:23,frontier:256,edits_per_expansion:12}}
pub fn config()->EvolutionConfig{EvolutionConfig{object:limits(),meta:meta(),max_lifetime_work:5_000_000_000}}
fn outputs(family:usize,a:i64,b:i64,c:i64,k:i64)->i64{match family{
 0=>a+b,1=>a*b+c,2=>(a-b)*c,3=>a+k,
 4=>a*b-c,5=>a*(b+c),6=>if a<b{a}else{b},_=>a^b,
}}
/// Only this teacher/test harness owns the generative rule. The policy gets
/// numerical examples and observed search losses, never family IDs or code.
pub fn tasks(seed:u64,n:usize,transfer:bool)->(Vec<SynthesisTask>,Vec<SynthesisTask>){
 let mut rng=Random(seed);let mut teaching=Vec::new();let mut probes=Vec::new();
 for index in 0..n{
  let k=(rng.index(7)+1) as i64;let j=rng.index(3);let f=index%4+if transfer{4}else{0};
  let mut make=|rows:usize,range:usize|{
   let examples=(0..rows).map(|_|{
    let input:Vec<_>=(0..3).map(|_|rng.index(range) as i64-(range/2) as i64).collect();
    let y=outputs(f,input[j],input[(j+1)%3],input[(j+2)%3],k);Example{input,output:y}
   }).collect();SynthesisTask{examples,library:Library::default()}
  };
  teaching.push(make(12,15));probes.push(make(32,63));
 }(teaching,probes)
}
pub fn loss_size()->SearchPolicy{SearchPolicy{priority:Expr::bin(Op::Add,Expr::Input(0),Expr::Input(1)),..Default::default()}}
pub fn fifo()->SearchPolicy{SearchPolicy{priority:Expr::Input(2),..Default::default()}}
pub fn probe(program:&Expr,task:&SynthesisTask)->Value{
 let mut m=Meter::new(1_000_000);let mut errors=0;let mut predictions=Vec::new();
 for row in &task.examples{
  let result=program.eval(Context{input:&row.input,action:0,state:&[],args:&[]},&task.library,&mut m).and_then(|v|v.int());
  let y=result.ok();if y!=Some(row.output){errors+=1;}predictions.push(y);
 }
 json!({"rows":task.examples.len(),"errors":errors,"predictions":predictions,"work":m.spent})
}
pub fn tested(policy:&SearchPolicy,tasks:&[SynthesisTask],probes:&[SynthesisTask],seed:u64)->Result<Value,Box<dyn std::error::Error>>{
 let now=std::time::Instant::now();let score=evaluate(policy,tasks,&limits(),seed)?;
 let measured:Vec<_>=score.runs.iter().zip(probes).map(|(r,t)|probe(&r.program,t)).collect();
 let generalizing=score.runs.iter().zip(&measured).filter(|(r,p)|r.solved && p["errors"]==0).count();
 let false_fits=score.solved-generalizing;
 Ok(json!({"policy":policy,"score":score,"generalizing":generalizing,"false_fits":false_fits,"probes":measured,"wall_seconds":now.elapsed().as_secs_f64()}))
}
