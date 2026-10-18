//! RSP1: metacircular, bounded search over executable expressions.
//!
//! A policy is itself an expression in the existing pure VM. The SAME search
//! function, driven by the accepted policy, can search solution expressions or
//! replacement policy expressions. Candidates cannot access an evaluator, a
//! target program, filesystem, authority or checkpoint mutators.
//!
//! This changes SEARCH ORDER, not the interpreter or acceptance contract.
//! Tree edits and the search/verification envelope remain authored.
use super::vm::{Context, Error, Expr, Library, Meter, Op, Result, Ty, Value};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

pub const FEATURES: usize = 10;
const POLICY_RECORD:u64=0x52535031;
pub const MAX_POLICY_NODES: usize = 31;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchPolicy { pub priority: Expr, pub variation:Expr, pub location:Expr }
impl Default for SearchPolicy {
    fn default() -> Self { Self { priority: Expr::Input(0), variation:Expr::Input(8), location:Expr::Input(9) } }
}
impl SearchPolicy {
    pub fn validate(&self) -> Result<()> {
        // Iterative shape checks before recursion: malformed trees must fail.
        let mut stack = vec![(&self.priority, 0),(&self.variation,0),(&self.location,0)]; let mut count = 1;
        while let Some((e,d)) = stack.pop() {
            count += 1;
            if count > MAX_POLICY_NODES || d > 12 { return Err(Error::Capacity); }
            match e {
                Expr::Input(i) if *i < FEATURES => {},
                Expr::Lit(Value::Int(x)) if x.abs_diff(0) <= 1_000_000 => {},
                Expr::Lit(Value::Bool(_)) => {},
                Expr::Bin(_,a,b) => {stack.push((a,d+1));stack.push((b,d+1));},
                Expr::If(a,b,c) => {stack.push((a,d+1));stack.push((b,d+1));stack.push((c,d+1));},
                Expr::Not(a) | Expr::AsInt(a) => stack.push((a,d+1)),
                _ => return Err(Error::InvalidProgram),
            }
        }
        if [&self.priority,&self.variation,&self.location].iter().any(|e|e.ty(&Library::default())!=Ok(Ty::Int)) { return Err(Error::Type); }
        Ok(())
    }
    /// Typed data record: not an arithmetic call in the active solution.
    pub fn code(&self)->Expr{Expr::Call(POLICY_RECORD,vec![self.priority.clone(),self.variation.clone(),self.location.clone()])}
    pub fn from_code(code:&Expr)->Result<Self>{
        if let Expr::Call(id,a)=code{if *id==POLICY_RECORD&&a.len()==3{
            let p=Self{priority:a[0].clone(),variation:a[1].clone(),location:a[2].clone()};p.validate()?;return Ok(p);
        }}
        Err(Error::InvalidProgram)
    }
    pub fn fingerprint(&self) -> u64 {
        super::vm::fingerprint(&serde_json::to_vec(self).expect("serializable VM expression"))
    }
    fn rank(&self, f: &[i64; FEATURES], meter: &mut Meter) -> Result<i64> {
        // A bad priority may lose its opportunity, but never gain free work.
        match self.priority.eval(Context{input:f,action:0,state:&[],args:&[]},&Library::default(),meter) {
            Ok(Value::Int(n)) => Ok(n.clamp(-1_000_000_000_000,1_000_000_000_000)),
            Err(Error::Budget) => Err(Error::Budget),
            _ => Ok(i64::MAX),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchLimits {
    pub work: u64, pub evaluations: usize, pub max_nodes: usize,
    pub frontier: usize, pub edits_per_expansion: usize,
}
impl Default for SearchLimits {
    fn default()->Self {Self{work:1_000_000,evaluations:400,max_nodes:11,frontier:512,edits_per_expansion:32}}
}
impl SearchLimits {
    pub fn validate(&self)->Result<()> {
        if !(1..=100_000_000).contains(&self.work) || !(1..=100_000).contains(&self.evaluations)
            || !(1..=MAX_POLICY_NODES).contains(&self.max_nodes) || !(1..=8192).contains(&self.frontier)
            || !(1..=512).contains(&self.edits_per_expansion) { return Err(Error::Capacity); }
        Ok(())
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct SearchResult {
    pub program: Expr, pub loss: u64, pub solved: bool, pub evaluations: usize,
    pub work: u64, pub objective_work:u64, pub priority_work:u64,
    pub expanded:usize, pub generated:usize, pub exhausted:bool,
    pub policy_id:u64, pub trace_hash:u64,
    pub selected_origins:Vec<usize>,
    pub edits_to_winner:Vec<TreeEdit>,
}
#[derive(Debug,Clone)]
struct Node { expr:Expr, loss:u64, parent:usize, depth:usize, edit:Option<TreeEdit> }

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeEdit { pub site:usize, pub replacement:Expr }
impl TreeEdit {
    pub fn apply(&self,parent:&Expr)->Result<Expr>{
        if self.site>=parent.nodes()||self.replacement.nodes()>MAX_POLICY_NODES{return Err(Error::InvalidProgram);}
        Ok(replace(parent,self.site,self.replacement.clone()))
    }
}

/// Public task information is held by the host. Only evaluated numerical
/// losses enter the policy. Callback receives a meter it MUST charge.
/// The metacircular experiment charges all nested search costs as well.
pub trait Objective {
    fn input_width(&self)->usize;
    fn library(&self)->&Library;
    fn score(&mut self, candidate:&Expr, meter:&mut Meter)->Result<u64>;
    fn solved(&self, loss:u64)->bool {loss==0}
}

#[derive(Debug,Clone,Copy)]
pub struct Random(pub u64);
impl Random {
    pub fn word(&mut self)->u64 {
        self.0=self.0.wrapping_add(0x9e3779b97f4a7c15);let mut x=self.0;
        x=(x^(x>>30)).wrapping_mul(0xbf58476d1ce4e5b9);x=(x^(x>>27)).wrapping_mul(0x94d049bb133111eb);x^(x>>31)
    }
    pub fn index(&mut self,n:usize)->usize {(self.word()%(n.max(1) as u64)) as usize}
}
fn leaf(r:&mut Random,width:usize)->Expr {
    let k=r.index(width+7);
    if k < width {Expr::Input(k)} else {Expr::integer([-2,-1,0,1,2,4,8][k-width])}
}
fn get(e:&Expr, wanted:usize)->Option<&Expr> {
    fn go<'a>(e:&'a Expr,n:&mut usize,w:usize)->Option<&'a Expr> {
        let index=*n;*n+=1;if index==w{return Some(e);}
        match e {
            Expr::Bin(_,a,b)=>go(a,n,w).or_else(||go(b,n,w)),
            Expr::If(a,b,c)=>go(a,n,w).or_else(||go(b,n,w)).or_else(||go(c,n,w)),
            Expr::Not(a)|Expr::AsInt(a)=>go(a,n,w),
            Expr::Call(_,args)=>args.iter().find_map(|a|go(a,n,w)),
            Expr::Fold{count,seed,body}=>go(count,n,w).or_else(||go(seed,n,w)).or_else(||go(body,n,w)),
            _=>None,
        }
    }
    go(e,&mut 0,wanted)
}
fn replace(e:&Expr,wanted:usize,replacement:Expr)->Expr {
    let mut i=0;
    e.map(&mut |_|{let j=i;i+=1;if j==wanted{Some(replacement.clone())}else{None}})
}
/// Primitive edits are generic syntax changes, not task solutions. Whole-root
/// replacement, growth, deletion and crossover are permitted; not only holes.
fn mutate(e:&Expr, width:usize, library:&Library, archive:&[Node], r:&mut Random, site:usize, operator:usize)->(Expr,TreeEdit) {
    let old=get(e,site).unwrap_or(e);
    let replacement=match operator {
        0=>leaf(r,width),
        1=>{
            let ops=[Op::Add,Op::Sub,Op::Mul,Op::Xor,Op::Rem];let op=ops[r.index(ops.len())];
            let other=leaf(r,width);if r.word()&1==0{Expr::bin(op,old.clone(),other)}else{Expr::bin(op,other,old.clone())}
        },
        2=>match old {Expr::Bin(_,a,b)=>if r.word()&1==0{*a.clone()}else{*b.clone()},_=>leaf(r,width)},
        3=>match old {
            Expr::Bin(_,a,b)=>Expr::bin([Op::Add,Op::Sub,Op::Mul,Op::Xor,Op::Rem][r.index(5)],*a.clone(),*b.clone()),
            _=>Expr::bin(Op::Add,old.clone(),leaf(r,width)),
        },
        4=>match old{Expr::Bin(op,a,b)=>Expr::bin(*op,*b.clone(),*a.clone()),_=>leaf(r,width)},
        5=>{
            if !archive.is_empty(){let a=&archive[r.index(archive.len())].expr;get(a,r.index(a.nodes())).unwrap_or(a).clone()}
            else{leaf(r,width)}
        },
        6=>{
            let condition=Expr::bin(Op::Lt,leaf(r,width),leaf(r,width));
            Expr::If(Box::new(condition),Box::new(old.clone()),Box::new(leaf(r,width)))
        },
        _=>{
            if !library.functions.is_empty() && r.word()&1==0 {
                let f=&library.functions[r.index(library.functions.len())];
                Expr::Call(f.id,(0..f.arity).map(|_|leaf(r,width)).collect())
            } else {Expr::bin([Op::Add,Op::Sub,Op::Mul][r.index(3)],leaf(r,width),leaf(r,width))}
        },
    };
    let next=replace(e,site,replacement.clone());
    (next,TreeEdit{site,replacement})
}

pub fn search<O:Objective>(policy:&SearchPolicy,root:Expr,objective:&mut O,limits:&SearchLimits,seed:u64)->Result<SearchResult> {
    policy.validate()?;limits.validate()?;
    if !(1..=32).contains(&objective.input_width()) || root.nodes()>limits.max_nodes {return Err(Error::InvalidData);}
    let mut meter=Meter::new(limits.work);let mut objective_work=0;let mut priority_work=0;let mut evals=0;
    let score=|e:&Expr,o:&mut O,m:&mut Meter,work:&mut u64,evals:&mut usize|->Result<u64>{
        let before=m.spent;let result=o.score(e,m);*work+=m.spent-before;*evals+=1;result
    };
    let root_loss=score(&root,objective,&mut meter,&mut objective_work,&mut evals)?;
    let mut nodes=vec![Node{expr:root,loss:root_loss,parent:0,depth:0,edit:None}];
    let mut best=0;let mut heap=BinaryHeap::new();heap.push(Reverse((0i64,0usize)));
    let mut seen=BTreeSet::new();seen.insert(nodes[0].expr.clone());
    let mut expanded=0;let mut generated=0;let mut trace=0xcbf29ce484222325u64;let mut origins=Vec::new();
    let mut rng=Random(seed);let mut exhausted=false;
    'outer: while !heap.is_empty() && evals<limits.evaluations && !objective.solved(nodes[best].loss) {
        // 1/8 FIFO reserve is immutable; the learned rank cannot starve the
        // whole remaining space. Every actual scan is charged.
        let index=if expanded%8==7 {
            let mut entries=heap.clone().into_vec();
            if meter.charge(entries.len() as u64).is_err(){exhausted=true;break;}
            entries.sort_by_key(|Reverse((_,id))|*id);let item=entries.remove(0);heap=entries.into_iter().collect();item.0.1
        } else {heap.pop().unwrap().0.1};
        expanded+=1;origins.push(index);trace=(trace^(index as u64)).wrapping_mul(0x100000001b3);
        let parent=nodes[index].clone();
        for _ in 0..limits.edits_per_expansion {
            if evals>=limits.evaluations{break 'outer;}
            let noise_a=(rng.word()&0xFFFF) as i64;let noise_b=(rng.word()&0xFFFF) as i64;
            let f=[parent.loss.min(1_000_000_000) as i64,parent.expr.nodes() as i64,nodes.len() as i64,parent.depth as i64,
                0,nodes[best].loss.min(1_000_000_000) as i64,expanded as i64,(meter.remaining()*1024/limits.work) as i64,noise_a,noise_b];
            let before=meter.spent;
            
            let op=policy.variation.eval(Context{input:&f,action:0,state:&[],args:&[]},&Library::default(),&mut meter).and_then(Value::int);
            let site=policy.location.eval(Context{input:&f,action:0,state:&[],args:&[]},&Library::default(),&mut meter).and_then(Value::int);
            priority_work+=meter.spent-before;
            if op==Err(Error::Budget)||site==Err(Error::Budget){exhausted=true;break 'outer;}
            // Independent 1/8 variation reserve complements the FIFO reserve.
            let (op,site)=if generated%8==7{(noise_a as usize%8,noise_b as usize%parent.expr.nodes())}
                else{(op.unwrap_or(noise_a).unsigned_abs() as usize%8,site.unwrap_or(noise_b).unsigned_abs() as usize%parent.expr.nodes())};
            let (candidate,edit)=mutate(&parent.expr,objective.input_width(),objective.library(),&nodes,&mut rng,site,op);generated+=1;
            trace=(trace^(op as u64)^((site as u64)<<8)).wrapping_mul(0x100000001b3);
            trace=(trace^super::vm::fingerprint(&serde_json::to_vec(&candidate).unwrap())).wrapping_mul(0x100000001b3);
            if meter.charge((candidate.nodes()+parent.expr.nodes()) as u64).is_err(){exhausted=true;break 'outer;}
            if candidate.nodes()>limits.max_nodes || candidate.ty(objective.library())!=Ok(Ty::Int) || !seen.insert(candidate.clone()){continue;}
            if heap.len()>=limits.frontier{continue;}
            let loss=match score(&candidate,objective,&mut meter,&mut objective_work,&mut evals) {
                Ok(n)=>n,Err(Error::Budget)=>{exhausted=true;break 'outer;},Err(_)=>continue,
            };
            let id=nodes.len();
            // Different scales are explicit. No hidden target/schema ID enters.
            let f=[loss.min(1_000_000_000) as i64,candidate.nodes() as i64,
                id as i64, parent.depth as i64+1, parent.loss.min(1_000_000_000) as i64-loss.min(1_000_000_000) as i64,
                nodes[best].loss.min(1_000_000_000) as i64,expanded as i64,
                (meter.remaining()*1024/limits.work) as i64,noise_a,noise_b];
            let before=meter.spent;let rank=policy.rank(&f,&mut meter);priority_work+=meter.spent-before;
            let rank=match rank{Ok(n)=>n,Err(Error::Budget)=>{exhausted=true;break 'outer;},Err(e)=>return Err(e)};
            nodes.push(Node{expr:candidate,loss,parent:index,depth:parent.depth+1,edit:Some(edit)});heap.push(Reverse((rank,id)));
            if (loss,nodes[id].expr.nodes()) < (nodes[best].loss,nodes[best].expr.nodes()){best=id;}
            if objective.solved(nodes[best].loss){break 'outer;}
        }
    }
    let mut edits=Vec::new();let mut cursor=best;
    while cursor!=0{let n=&nodes[cursor];edits.push(n.edit.clone().ok_or(Error::InvalidProgram)?);cursor=n.parent;}
    edits.reverse();
    Ok(SearchResult{program:nodes[best].expr.clone(),loss:nodes[best].loss,solved:objective.solved(nodes[best].loss),
        evaluations:evals,work:meter.spent,objective_work,priority_work,expanded,generated,
        exhausted:exhausted||evals>=limits.evaluations,policy_id:policy.fingerprint(),trace_hash:trace,selected_origins:origins,edits_to_winner:edits})
}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {pub input:Vec<i64>,pub output:i64}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynthesisTask {pub examples:Vec<Example>,pub library:Library}
impl SynthesisTask {
    pub fn validate(&self)->Result<()> {
        if self.examples.is_empty()||self.examples.len()>256{return Err(Error::InvalidData);}
        let width=self.examples[0].input.len();if !(1..=32).contains(&width){return Err(Error::InvalidData);}
        if self.examples.iter().any(|r|r.input.len()!=width||r.input.iter().any(|v|v.abs_diff(0)>1_000_000)||r.output.abs_diff(0)>1_000_000){return Err(Error::InvalidData);}
        self.library.validate(width)?;Ok(())
    }
    pub fn id(&self)->u64{super::vm::fingerprint(&serde_json::to_vec(self).unwrap())}
}
impl Objective for SynthesisTask {
    fn input_width(&self)->usize{self.examples.first().map_or(0,|e|e.input.len())}
    fn library(&self)->&Library{&self.library}
    fn score(&mut self,e:&Expr,m:&mut Meter)->Result<u64>{
        let mut loss=0u64;
        for row in &self.examples{
            m.charge(1)?;
            let y=e.eval(Context{input:&row.input,action:0,state:&[],args:&[]},&self.library,m)?.int()?;
            loss=loss.saturating_add(y.abs_diff(row.output));
        }
        Ok(loss)
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct SuiteScore {pub tasks:usize,pub solved:usize,pub value:u64,pub work:u64,pub runs:Vec<SearchResult>}
/// Every failed search is charged the same failure penalty; solved ones use
/// actual work. This is an instrumented cost, not wall-clock time.
pub fn evaluate(policy:&SearchPolicy,tasks:&[SynthesisTask],limits:&SearchLimits,seed:u64)->Result<SuiteScore>{
    let mut s=SuiteScore{tasks:tasks.len(),solved:0,value:0,work:0,runs:Vec::new()};
    for (i,t) in tasks.iter().enumerate(){
        t.validate()?;let mut task=t.clone();
        let r=search(policy,Expr::integer(0),&mut task,limits,seed.wrapping_add(i as u64*0x10001))?;
        s.solved+=usize::from(r.solved);s.work=s.work.saturating_add(r.work);
        s.value=s.value.saturating_add(if r.solved {r.work} else {limits.work*2+r.loss.min(limits.work)});s.runs.push(r);
    }
    Ok(s)
}

struct PolicyObjective<'a> {tasks:&'a [SynthesisTask],object_limits:&'a SearchLimits,seed:u64,library:Library}
impl Objective for PolicyObjective<'_> {
    fn input_width(&self)->usize{FEATURES}
    fn library(&self)->&Library{&self.library}
    fn solved(&self,_:u64)->bool{false}
    fn score(&mut self,e:&Expr,m:&mut Meter)->Result<u64>{
        let p=SearchPolicy::from_code(e)?;
        // Full worst-case nested cost reserved before evaluation; impossible
        // to silently overrun the host budget by an expensive candidate.
        let max=self.object_limits.work.checked_mul(self.tasks.len() as u64).ok_or(Error::Capacity)?;
        if m.remaining()<max{return Err(Error::Budget);}
        match evaluate(&p,self.tasks,self.object_limits,self.seed){
            Ok(score)=>{m.charge(score.work)?;Ok(score.value)},
            Err(e)=>{m.charge(max)?;Err(e)}
        }
    }
}
/// producer drives an actual invocation of the SAME search routine over the
/// target policy code. It may replace any subtree, not only specialize a hole.
pub fn produce(producer:&SearchPolicy,target:&SearchPolicy,tasks:&[SynthesisTask],object_limits:&SearchLimits,meta_limits:&SearchLimits,seed:u64)->Result<SearchResult>{
    if tasks.is_empty()||tasks.len()>128{return Err(Error::InvalidData);}
    target.validate()?;
    let mut objective=PolicyObjective{tasks,object_limits,seed:seed^0xF00D,library:Library{functions:vec![super::vm::Function{id:POLICY_RECORD,arity:3,body:Expr::Arg(0)}]}};
    search(producer,target.code(),&mut objective,meta_limits,seed)
}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvolutionConfig {
    pub object:SearchLimits,pub meta:SearchLimits,pub max_lifetime_work:u64,
}
impl EvolutionConfig{
    pub fn validate(&self)->Result<()>{
        self.object.validate()?;self.meta.validate()?;
        if self.max_lifetime_work==0||self.max_lifetime_work>100_000_000_000{return Err(Error::Capacity);}
        Ok(())
    }
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct PolicyAudit {
    pub attempt:u64,pub generation_before:u32,pub generation_after:u32,
    pub producer:SearchPolicy,pub candidate:SearchPolicy,pub seed:u64,
    pub training_ids:Vec<u64>,pub gate_ids:Vec<u64>,pub gate_input_keys:Vec<u64>,
    pub production:SearchResult,pub before:SuiteScore,pub after:SuiteScore,
    pub behaviour_changed:bool,
    pub accepted:bool,pub consumed_work:u64,
}
/// Host-owned lifecycle. No method accepts a user-supplied success boolean or
/// a requested generation number. Every adoption follows executed production
/// and a previously unused validation batch.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveSearch {
    config:EvolutionConfig,active:SearchPolicy,generation:u32,work:u64,
    audits:Vec<PolicyAudit>,used_gate_inputs:BTreeSet<u64>,frozen:bool,
}
fn input_key(task:&SynthesisTask)->u64{
    let rows:Vec<_>=task.examples.iter().map(|r|&r.input).collect();
    super::vm::fingerprint(&serde_json::to_vec(&rows).unwrap())
}
impl RecursiveSearch {
    pub fn new(config:EvolutionConfig)->Result<Self>{
        config.validate()?;
        Ok(Self{config,active:SearchPolicy::default(),generation:0,work:0,audits:Vec::new(),used_gate_inputs:BTreeSet::new(),frozen:false})
    }
    pub fn policy(&self)->&SearchPolicy{&self.active}
    pub fn generation(&self)->u32{self.generation}
    pub fn work(&self)->u64{self.work}
    pub fn audits(&self)->&[PolicyAudit]{&self.audits}
    pub fn freeze(&mut self){self.frozen=true;}
    pub fn is_frozen(&self)->bool{self.frozen}
    pub fn validate(&self)->Result<()>{
        self.config.validate()?;self.active.validate()?;
        if self.audits.len()>64||self.used_gate_inputs.len()>8192||self.work>self.config.max_lifetime_work{return Err(Error::InvalidData);}
        let mut policy=SearchPolicy::default();let mut generation=0;let mut sum=0u64;let mut seen=BTreeSet::new();
        for (i,a) in self.audits.iter().enumerate(){
            a.candidate.validate()?;
            if a.attempt!=i as u64||a.producer!=policy||a.generation_before!=generation||a.production.policy_id!=policy.fingerprint()
                ||a.production.program!=a.candidate.code()||a.production.work>a.consumed_work
                ||a.before.tasks!=a.gate_ids.len()||a.after.tasks!=a.gate_ids.len()
                ||a.before.runs.len()!=a.gate_ids.len()||a.after.runs.len()!=a.gate_ids.len()||a.gate_input_keys.len()!=a.gate_ids.len(){return Err(Error::InvalidData);}
            let mut replay=policy.code();
            for edit in &a.production.edits_to_winner{replay=edit.apply(&replay)?;}
            if replay!=a.candidate.code(){return Err(Error::InvalidData);}
            for key in &a.gate_input_keys{if !seen.insert(*key)||!self.used_gate_inputs.contains(key){return Err(Error::InvalidData);}}
            let changed=a.before.runs.iter().zip(&a.after.runs).any(|(x,y)|x.trace_hash!=y.trace_hash);
            if changed!=a.behaviour_changed{return Err(Error::InvalidData);}
            validate_suite(&a.before,&self.config.object)?;validate_suite(&a.after,&self.config.object)?;
            if a.consumed_work!=a.production.work+a.before.work+a.after.work{return Err(Error::InvalidData);}
            let accepted=changed && a.candidate!=policy && a.after.solved>=a.before.solved && a.after.value<a.before.value
                && a.after.value.saturating_mul(100)<=a.before.value.saturating_mul(95);
            if accepted!=a.accepted{return Err(Error::InvalidData);}
            if accepted {policy=a.candidate.clone();generation+=1;}
            if a.generation_after!=generation{return Err(Error::InvalidData);}
            sum=sum.checked_add(a.consumed_work).ok_or(Error::Capacity)?;
        }
        if self.active!=policy||self.generation!=generation||sum>self.work{return Err(Error::InvalidData);}
        Ok(())
    }
    pub fn improve(&mut self,training:&[SynthesisTask],gate:&[SynthesisTask],seed:u64)->Result<&PolicyAudit>{
        if self.frozen{return Err(Error::Blocked);}
        if self.audits.len()>=64||training.is_empty()||gate.len()<8||training.len()>64||gate.len()>64{return Err(Error::Capacity);}
        let mut train_keys=BTreeSet::new();let mut gate_keys=BTreeSet::new();
        for task in training{task.validate()?;let key=input_key(task);if self.used_gate_inputs.contains(&key){return Err(Error::Stale);}train_keys.insert(key);}
        for task in gate{task.validate()?;let key=input_key(task);
            if self.used_gate_inputs.contains(&key)||train_keys.contains(&key)||!gate_keys.insert(key){return Err(Error::Stale);}
        }
        let reservation=self.config.meta.work.checked_add(self.config.object.work*gate.len() as u64*2).ok_or(Error::Capacity)?;
        if self.work.checked_add(reservation).is_none_or(|n|n>self.config.max_lifetime_work){return Err(Error::Budget);}
        // Consume before execution. Errors retain reservation and single-use
        // gate records rather than creating free retries or double spending.
        self.work+=reservation;self.used_gate_inputs.extend(gate_keys);
        let production=produce(&self.active,&self.active,training,&self.config.object,&self.config.meta,seed)?;
        let candidate=SearchPolicy::from_code(&production.program)?;
        let before=evaluate(&self.active,gate,&self.config.object,seed^0xBA5E)?;
        let after=evaluate(&candidate,gate,&self.config.object,seed^0xBA5E)?;
        let spent=production.work+before.work+after.work;
        self.work=self.work.checked_sub(reservation-spent).ok_or(Error::InvalidData)?;
        let changed=before.runs.iter().zip(&after.runs).any(|(x,y)|x.trace_hash!=y.trace_hash);
        let accepted=changed && candidate!=self.active && after.solved>=before.solved && after.value<before.value
            && after.value.saturating_mul(100)<=before.value.saturating_mul(95);
        let producer=self.active.clone();let old_generation=self.generation;
        if accepted{self.active=candidate.clone();self.generation+=1;}
        self.audits.push(PolicyAudit{attempt:self.audits.len() as u64,generation_before:old_generation,generation_after:self.generation,
            producer,candidate,seed,training_ids:training.iter().map(SynthesisTask::id).collect(),gate_ids:gate.iter().map(SynthesisTask::id).collect(),gate_input_keys:gate.iter().map(input_key).collect(),
            production,before,after,behaviour_changed:changed,accepted,consumed_work:spent});
        Ok(self.audits.last().unwrap())
    }
}

fn validate_suite(s:&SuiteScore,l:&SearchLimits)->Result<()>{
    if s.tasks!=s.runs.len()||s.solved!=s.runs.iter().filter(|r|r.solved).count()
        ||s.work!=s.runs.iter().map(|r|r.work).sum::<u64>()
        ||s.value!=s.runs.iter().map(|r|if r.solved{r.work}else{l.work*2+r.loss.min(l.work)}).sum::<u64>()
        ||s.runs.iter().any(|r|r.work>l.work||r.evaluations>l.evaluations||r.objective_work+r.priority_work>r.work||r.solved!=(r.loss==0)){
        return Err(Error::InvalidData);
    }Ok(())
}

pub(super) fn synthesis_lane(parent:&super::vm::Program,data:&[super::synthesis::Transition],lib:&Library,
    cfg:&super::synthesis::SearchConfig,requested:Option<usize>,policy:&SearchPolicy)->Result<super::synthesis::SearchTrace>{
    use super::synthesis::*;
    cfg.validate()?;parent.validate(parent.outputs.len(),lib)?;validate_data(data,parent.outputs.len(),32)?;
    let mut meter=Meter::new(cfg.work);let job=discover_requested_job(parent,data,lib,cfg.deterministic,requested,&mut meter)?.ok_or(Error::NoEvidence)?;
    let width=parent.outputs.len();let full=width+1+parent.cells.len();
    // One quarter always remains available to the prior broad synthesizer.
    if full>32||data.len()>256||meter.remaining()<400{
        let mut adjusted=cfg.clone();adjusted.work=meter.remaining();
        if adjusted.work<100{return Err(Error::Budget);}
        let mut r=synthesize_requested(parent,data,lib,&adjusted,&[],requested)?;r.work+=meter.spent;return Ok(r);
    }
    let mut state=parent.initial_state();let mut examples=Vec::new();let mut other_loss=0.0;
    for t in data {
        if t.restart{state=parent.initial_state();}
        let mut input=t.pre.clone();input.push(t.action as i64);input.extend_from_slice(&state);
        let (old,next)=parent.step(&t.pre,t.action,&state,lib,&mut meter)?;state=next;
        other_loss+=old.iter().zip(&t.post).enumerate().filter(|(j,_)|*j!=job.target).map(|(_,(&x,&y))|x.abs_diff(y) as f64).sum::<f64>();
        examples.push(Example{input,output:t.post[job.target]});
    }
    let mut task=SynthesisTask{examples,library:lib.clone()};task.validate()?;
    let limits=SearchLimits{work:meter.remaining()*3/4,evaluations:4096,max_nodes:cfg.max_expr_nodes,frontier:cfg.max_frontier.min(8192),edits_per_expansion:24};
    let root=parent.outputs[job.target].map(&mut |e|match e{
        Expr::Action=>Some(Expr::Input(width)),Expr::State(i)=>Some(Expr::Input(width+1+i)),_=>None,
    });
    let search_result=search(policy,root,&mut task,&limits,job.parent^data.len() as u64);
    let mut candidate=None;let mut candidates=0;let mut learned_work=0;
    match search_result {
        Ok(r)=>{candidates=r.evaluations as u64;learned_work=r.work;meter.charge(r.work)?;
            if r.loss==0 {
                let replacement=r.program.map(&mut |e|if let Expr::Input(i)=e {
                    if *i==width{Some(Expr::Action)}else if *i>width{Some(Expr::State(i-width-1))}else{None}
                }else{None});
                let patch=Patch{parent:parent.fingerprint(),target:job.target,replacement,cells:parent.cells.clone()};
                patch.apply(parent,lib)?;
                // State rules and every other output are unchanged. Full
                // executable replay remains mandatory in complete_trial.
                candidate=Some((patch,other_loss));
            }
        },Err(Error::Budget)=>{learned_work=limits.work;meter.charge(limits.work)?;},Err(_)=>{learned_work=limits.work;meter.charge(limits.work)?;},
    }
    if let Some((patch,loss))=candidate {
        return Ok(SearchTrace{job,candidates,updates_tested:0,work:meter.spent,baseline_work:0,constructor_work:learned_work,
            exhausted:false,frontier_limited:false,winner:Some(patch),training_loss:loss,constructor_id:None,search_policy_id:Some(policy.fingerprint()),prospectively_accepted:false});
    }
    let mut adjusted=cfg.clone();adjusted.work=meter.remaining();if adjusted.work<100{return Err(Error::Budget);}
    let mut trace=synthesize_requested(parent,data,lib,&adjusted,&[],requested)?;
    trace.search_policy_id=Some(policy.fingerprint());trace.work+=meter.spent;trace.constructor_work+=learned_work;trace.candidates+=candidates;Ok(trace)
}
