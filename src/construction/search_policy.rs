//! RSP1: metacircular, bounded search over executable expressions.
//!
//! A policy is itself an expression in the existing pure VM. The SAME search
//! function, driven by the accepted policy, can search solution expressions or
//! replacement policy expressions. Candidates cannot access an evaluator, a
//! target program, filesystem, authority or checkpoint mutators.
//!
//! Tree edits and the interpreter remain authored. Adoption requires held-out
//! predictions, retention, transfer and a matched test of the producer role.
use super::vm::{Context, Error, Expr, Library, Meter, Op, Result, Ty, Value};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

pub const LEGACY_FEATURES: usize = 10;
pub const FEATURES: usize = 20;
/// Observed-example features, never task IDs, reference programs or exam data:
/// 10 count; 11 exact fraction; 12 residual sign balance; 13 mean residual;
/// 14 strongest associated input; 15 signed association; 16/17 order pair;
/// 18 order-dependent residual difference; 19 observation kind (1 = examples).
pub const FEEDBACK_FEATURES: usize = FEATURES - LEGACY_FEATURES;
const POLICY_RECORD:u64=0x52535031;
pub const MAX_POLICY_NODES: usize = 31;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchPolicy { pub priority: Expr, pub variation:Expr, pub location:Expr }
impl Default for SearchPolicy {
    fn default() -> Self { Self { priority: Expr::Input(0), variation:Expr::Input(8), location:Expr::Input(9) } }
}
impl SearchPolicy {
    fn feedback_mask(&self) -> u32 {
        let mut mask = 0;
        for e in [&self.priority, &self.variation, &self.location] {
            e.map(&mut |x| { if let Expr::Input(i) = x {
                if *i >= LEGACY_FEATURES && *i < FEATURES { mask |= 1 << (*i - LEGACY_FEATURES); }
            } None });
        }
        mask
    }
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
struct Node { expr:Expr, loss:u64, feedback:[i64;FEEDBACK_FEATURES], parent:usize, depth:usize, edit:Option<TreeEdit> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation { pub loss:u64, pub feedback:[i64;FEEDBACK_FEATURES] }

fn features(base:[i64;LEGACY_FEATURES], feedback:[i64;FEEDBACK_FEATURES]) -> [i64;FEATURES] {
    let mut out = [0;FEATURES]; out[..LEGACY_FEATURES].copy_from_slice(&base);
    out[LEGACY_FEATURES..].copy_from_slice(&feedback); out
}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeEdit { pub site:usize, pub replacement:Expr }
impl TreeEdit {
    pub fn apply(&self,parent:&Expr)->Result<Expr>{
        if self.site>=parent.nodes()||self.replacement.nodes()>MAX_POLICY_NODES{return Err(Error::InvalidProgram);}
        Ok(replace(parent,self.site,self.replacement.clone()))
    }
}

/// Only public examples and their actual prediction residuals enter the policy.
/// The callback receives a meter it MUST charge, including diagnostics.
/// The metacircular experiment charges all nested search costs as well.
pub trait Objective {
    fn input_width(&self)->usize;
    fn library(&self)->&Library;
    fn score(&mut self, candidate:&Expr, meter:&mut Meter)->Result<u64>;
    fn observe(&mut self, candidate:&Expr, meter:&mut Meter, _mask:u32)->Result<Observation> {
        Ok(Observation { loss:self.score(candidate,meter)?, feedback:[0;FEEDBACK_FEATURES] })
    }
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
fn selected_leaf(r:&mut Random,width:usize,detail:Option<usize>)->Expr {
    match detail {
        None=>leaf(r,width),
        Some(d)=>{let k=d%(width+7);if k<width{Expr::Input(k)}else{Expr::integer([-2,-1,0,1,2,4,8][k-width])}}
    }
}
fn selected_op(r:&mut Random,width:usize,detail:Option<usize>)->Op {
    [Op::Add,Op::Sub,Op::Mul,Op::Xor,Op::Rem][detail.map_or_else(||r.index(5),|d|(d/(width+7))%5)]
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
fn mutate(e:&Expr, width:usize, library:&Library, archive:&[Node], r:&mut Random, site:usize, operator:usize, detail:Option<usize>)->(Expr,TreeEdit) {
    let old=get(e,site).unwrap_or(e);
    let replacement=match operator {
        0=>selected_leaf(r,width,detail),
        1=>{
            let op=selected_op(r,width,detail);
            let other=selected_leaf(r,width,detail);if r.word()&1==0{Expr::bin(op,old.clone(),other)}else{Expr::bin(op,other,old.clone())}
        },
        2=>match old {Expr::Bin(_,a,b)=>if r.word()&1==0{*a.clone()}else{*b.clone()},_=>leaf(r,width)},
        3=>match old {
            Expr::Bin(_,a,b)=>Expr::bin(selected_op(r,width,detail),*a.clone(),*b.clone()),
            _=>Expr::bin(Op::Add,old.clone(),selected_leaf(r,width,detail)),
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
    let feedback_mask=policy.feedback_mask();
    let score=|e:&Expr,o:&mut O,m:&mut Meter,work:&mut u64,evals:&mut usize|->Result<Observation>{
        let before=m.spent;let result=o.observe(e,m,feedback_mask);*work+=m.spent-before;*evals+=1;result
    };
    let root_observation=score(&root,objective,&mut meter,&mut objective_work,&mut evals)?;
    let mut nodes=vec![Node{expr:root,loss:root_observation.loss,feedback:root_observation.feedback,parent:0,depth:0,edit:None}];
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
            let f=features([parent.loss.min(1_000_000_000) as i64,parent.expr.nodes() as i64,nodes.len() as i64,parent.depth as i64,
                0,nodes[best].loss.min(1_000_000_000) as i64,expanded as i64,(meter.remaining()*1024/limits.work) as i64,noise_a,noise_b],parent.feedback);
            let before=meter.spent;
            
            let op=policy.variation.eval(Context{input:&f,action:0,state:&[],args:&[]},&Library::default(),&mut meter).and_then(Value::int);
            let site=policy.location.eval(Context{input:&f,action:0,state:&[],args:&[]},&Library::default(),&mut meter).and_then(Value::int);
            priority_work+=meter.spent-before;
            if op==Err(Error::Budget)||site==Err(Error::Budget){exhausted=true;break 'outer;}
            // Independent 1/8 variation reserve complements the FIFO reserve.
            let raw_op=op.unwrap_or(noise_a);
            // Policies using observed feedback may choose a concrete binding:
            // -(1 + edit + 8*detail), detail = leaf + (width+7)*arithmetic_op.
            // Legacy policies retain their original modulo/random semantics.
            let directed=feedback_mask!=0&&raw_op<0;
            let code=if directed{raw_op.unsigned_abs()-1}else{raw_op.unsigned_abs()};
            let (op,site,detail)=if generated%8==7{(noise_a as usize%8,noise_b as usize%parent.expr.nodes(),None)}
                else{(code as usize%8,site.unwrap_or(noise_b).unsigned_abs() as usize%parent.expr.nodes(),directed.then_some((code/8) as usize))};
            let (candidate,edit)=mutate(&parent.expr,objective.input_width(),objective.library(),&nodes,&mut rng,site,op,detail);generated+=1;
            trace=(trace^(op as u64)^((site as u64)<<8)).wrapping_mul(0x100000001b3);
            trace=(trace^super::vm::fingerprint(&serde_json::to_vec(&candidate).unwrap())).wrapping_mul(0x100000001b3);
            if meter.charge((candidate.nodes()+parent.expr.nodes()) as u64).is_err(){exhausted=true;break 'outer;}
            if candidate.nodes()>limits.max_nodes || candidate.ty(objective.library())!=Ok(Ty::Int) || !seen.insert(candidate.clone()){continue;}
            if heap.len()>=limits.frontier{continue;}
            let observation=match score(&candidate,objective,&mut meter,&mut objective_work,&mut evals) {
                Ok(n)=>n,Err(Error::Budget)=>{exhausted=true;break 'outer;},Err(_)=>continue,
            };
            let loss=observation.loss;
            let id=nodes.len();
            // Different scales are explicit. No hidden target/schema ID enters.
            let f=features([loss.min(1_000_000_000) as i64,candidate.nodes() as i64,
                id as i64, parent.depth as i64+1, parent.loss.min(1_000_000_000) as i64-loss.min(1_000_000_000) as i64,
                nodes[best].loss.min(1_000_000_000) as i64,expanded as i64,
                (meter.remaining()*1024/limits.work) as i64,noise_a,noise_b],observation.feedback);
            let before=meter.spent;let rank=policy.rank(&f,&mut meter);priority_work+=meter.spent-before;
            let rank=match rank{Ok(n)=>n,Err(Error::Budget)=>{exhausted=true;break 'outer;},Err(e)=>return Err(e)};
            nodes.push(Node{expr:candidate,loss,feedback:observation.feedback,parent:index,depth:parent.depth+1,edit:Some(edit)});heap.push(Reverse((rank,id)));
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
    fn observe(&mut self,e:&Expr,m:&mut Meter,mask:u32)->Result<Observation>{
        if mask==0 {return Ok(Observation{loss:self.score(e,m)?,feedback:[0;FEEDBACK_FEATURES]});}
        let width=self.input_width();
        if self.examples.is_empty()||self.examples.len()>256||!(1..=32).contains(&width){return Err(Error::InvalidData);}
        m.charge(self.examples.len() as u64)?;
        if self.examples.iter().any(|r|r.input.len()!=width){return Err(Error::InvalidData);}
        let mut residuals=Vec::with_capacity(self.examples.len());let mut loss=0u64;
        for row in &self.examples {
            m.charge(2)?;
            let y=e.eval(Context{input:&row.input,action:0,state:&[],args:&[]},&self.library,m)?.int()?;
            loss=loss.saturating_add(y.abs_diff(row.output));
            residuals.push((row.output as i128-y as i128).clamp(-1_000_000,1_000_000));
        }
        let n=residuals.len() as i128;if n==0{return Err(Error::InvalidData);}
        let mut f=[0;FEEDBACK_FEATURES];let sum:i128=residuals.iter().sum();
        f[0]=n as i64;f[1]=(1024*residuals.iter().filter(|&&r|r==0).count() as i128/n) as i64;
        f[2]=(1024*residuals.iter().map(|r|r.signum()).sum::<i128>()/n) as i64;
        f[3]=(sum/n) as i64;f[4]=-1;f[6]=-1;f[7]=-1;f[9]=1;
        m.charge((residuals.len()*4) as u64)?;
        if mask & ((1<<4)|(1<<5)) != 0 {
            for j in 0..width {
                m.charge((self.examples.len()*6) as u64)?;
                let sx:i128=self.examples.iter().map(|r|r.input[j] as i128).sum();
                let mut covariance=0i128;let mut magnitude=0i128;
                for (row,r) in self.examples.iter().zip(&residuals) {
                    let product=(row.input[j] as i128*n-sx)*(r*n-sum);
                    covariance+=product;magnitude+=product.abs();
                }
                let association=if magnitude==0{0}else{(1024*covariance/magnitude) as i64};
                if association.abs()>f[5].abs(){f[4]=j as i64;f[5]=association;}
            }
        }
        if mask & ((1<<6)|(1<<7)|(1<<8)) != 0 {
            let scale=residuals.iter().map(|x|x.abs()).sum::<i128>().max(1)*n;
            for a in 0..width {for b in a+1..width {
                m.charge((self.examples.len()*4) as u64)?;
                let mut left=0i128;let mut count=0i128;
                for (row,r) in self.examples.iter().zip(&residuals) {
                    if row.input[a]<row.input[b]{left+=r;count+=1;}
                }
                if count==0 || count==n{continue;}
                let difference=(1024*(left*n-sum*count)/scale).clamp(-1024,1024) as i64;
                if difference.abs()>f[8].abs(){f[6]=a as i64;f[7]=b as i64;f[8]=difference;}
            }}
        }
        Ok(Observation{loss,feedback:f})
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

/// Probe examples are used by admission only; never by the candidate's search.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationCase {pub support:SynthesisTask,pub probes:SynthesisTask}
impl ValidationCase {
    pub fn validate(&self)->Result<()> {
        self.support.validate()?;self.probes.validate()?;
        if self.support.input_width()!=self.probes.input_width() || self.support.library!=self.probes.library
            || self.probes.examples.iter().any(|p|self.support.examples.iter().any(|s|s.input==p.input)) {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    fn id(&self)->u64{super::vm::fingerprint(&serde_json::to_vec(self).unwrap())}
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerCase {pub training:Vec<SynthesisTask>,pub examination:Vec<ValidationCase>}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionSuite {
    pub fresh:Vec<ValidationCase>,
    /// Structurally different tasks supplied by the curriculum, with no family
    /// labels exposed to policy code. The engine cannot infer diversity from IDs.
    pub transfer:Vec<ValidationCase>,
    pub producers:Vec<ProducerCase>,
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct VerifiedRun {pub case_id:u64,pub seed:u64,pub probe_rows:usize,pub probe_errors:usize,pub probe_work:u64}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct VerifiedSuite {pub search:SuiteScore,pub probes:Vec<VerifiedRun>}
impl VerifiedSuite {
    pub fn solved(&self)->usize {self.search.runs.iter().zip(&self.probes).filter(|(r,p)|r.solved&&p.probe_errors==0).count()}
    pub fn work(&self)->u64 {self.search.work+self.probes.iter().map(|p|p.probe_work).sum::<u64>()}
    fn value(&self,limits:&SearchLimits)->u64 {
        self.search.runs.iter().zip(&self.probes).map(|(r,p)|
            if r.solved&&p.probe_errors==0 {r.work+p.probe_work} else {limits.work*4+r.loss.min(limits.work)}).sum()
    }
    fn validate(&self,limits:&SearchLimits)->Result<()> {
        validate_suite(&self.search,limits)?;
        if self.probes.len()!=self.search.runs.len() || self.probes.iter().any(|p|
            p.probe_rows==0||p.probe_errors>p.probe_rows||p.probe_work>limits.work) {return Err(Error::InvalidData);}
        Ok(())
    }
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct PairedSuite {pub before:VerifiedSuite,pub after:VerifiedSuite}
impl PairedSuite {
    fn validate(&self,limits:&SearchLimits)->Result<()> {
        self.before.validate(limits)?;self.after.validate(limits)?;
        if self.before.probes.iter().map(|p|(p.case_id,p.seed,p.probe_rows)).collect::<Vec<_>>()
            !=self.after.probes.iter().map(|p|(p.case_id,p.seed,p.probe_rows)).collect::<Vec<_>>() {return Err(Error::InvalidData);}
        Ok(())
    }
    fn work(&self)->u64{self.before.work()+self.after.work()}
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct ProducerComparison {
    pub before_creation:SearchResult,pub after_creation:SearchResult,pub children:PairedSuite,
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct RetainedCase {pub case:ValidationCase,pub seed:u64}
pub const MAX_RETENTION:usize=128;
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct QualificationAudit {
    pub fresh:PairedSuite,pub transfer:PairedSuite,pub retention:PairedSuite,
    pub producers:Vec<ProducerComparison>,pub added_retention:Vec<RetainedCase>,
}
impl QualificationAudit {
    fn behaviour_changed(&self)->bool {
        [&self.fresh,&self.transfer,&self.retention].iter().any(|p|p.before.search.runs.iter().zip(&p.after.search.runs)
            .any(|(a,b)|a.trace_hash!=b.trace_hash))
    }
    pub fn direct_pass(&self,limits:&SearchLimits)->bool {
        preserves_solved(&self.fresh) && preserves_solved(&self.transfer)
            && self.retention.after.solved()==self.retention.after.search.tasks
            && self.retention.before.solved()==self.retention.before.search.tasks
            && improves(self.fresh.before.solved()+self.transfer.before.solved(),self.fresh.after.solved()+self.transfer.after.solved(),
                self.fresh.before.value(limits)+self.transfer.before.value(limits),self.fresh.after.value(limits)+self.transfer.after.value(limits))
    }
    pub fn producer_pass(&self,limits:&SearchLimits)->bool {
        if self.producers.len()<2 || self.producers.iter().any(|p|p.children.after.solved()<p.children.before.solved()){return false;}
        let before_solved=self.producers.iter().map(|p|p.children.before.solved()).sum();
        let after_solved=self.producers.iter().map(|p|p.children.after.solved()).sum();
        let before_value=self.producers.iter().map(|p|p.before_creation.work+p.children.before.value(limits)).sum();
        let after_value=self.producers.iter().map(|p|p.after_creation.work+p.children.after.value(limits)).sum();
        improves(before_solved,after_solved,before_value,after_value)
    }
    fn work(&self)->u64 {self.fresh.work()+self.transfer.work()+self.retention.work()+self.producers.iter()
        .map(|p|p.before_creation.work+p.after_creation.work+p.children.work()).sum::<u64>()}
}
fn preserves_solved(pair:&PairedSuite)->bool {
    pair.before.search.runs.iter().zip(&pair.before.probes).zip(pair.after.search.runs.iter().zip(&pair.after.probes))
        .all(|((a,pa),(b,pb))|!a.solved||pa.probe_errors!=0||(b.solved&&pb.probe_errors==0))
}
fn improves(before:usize,after:usize,before_value:u64,after_value:u64)->bool {
    after>before || (after==before && after_value<before_value && after_value.saturating_mul(100)<=before_value.saturating_mul(95))
}
fn verify_cases(policy:&SearchPolicy,cases:&[RetainedCase],limits:&SearchLimits)->Result<VerifiedSuite> {
    let mut out=VerifiedSuite{search:SuiteScore{tasks:cases.len(),solved:0,value:0,work:0,runs:Vec::new()},probes:Vec::new()};
    for c in cases {
        let run=search(policy,Expr::integer(0),&mut c.case.support.clone(),limits,c.seed)?;
        let mut meter=Meter::new(limits.work);let mut errors=0;
        for (i,row) in c.case.probes.examples.iter().enumerate() {
            let result=meter.charge(1).and_then(|_|run.program.eval(Context{input:&row.input,action:0,state:&[],args:&[]},&c.case.probes.library,&mut meter)).and_then(Value::int);
            if result==Err(Error::Budget){errors+=c.case.probes.examples.len()-i;break;}
            if result!=Ok(row.output){errors+=1;}
        }
        out.search.solved+=usize::from(run.solved);out.search.work+=run.work;
        out.search.value+=if run.solved{run.work}else{limits.work*2+run.loss.min(limits.work)};
        out.probes.push(VerifiedRun{case_id:c.case.id(),seed:c.seed,probe_rows:c.case.probes.examples.len(),probe_errors:errors,probe_work:meter.spent});
        out.search.runs.push(run);
    }
    Ok(out)
}
fn paired(before:&SearchPolicy,after:&SearchPolicy,cases:&[RetainedCase],limits:&SearchLimits)->Result<PairedSuite>{
    Ok(PairedSuite{before:verify_cases(before,cases,limits)?,after:verify_cases(after,cases,limits)?})
}
fn seeded(cases:&[ValidationCase],seed:u64)->Vec<RetainedCase>{
    cases.iter().enumerate().map(|(i,c)|RetainedCase{case:c.clone(),seed:seed.wrapping_add(i as u64*0x10001)}).collect()
}
/// Same initial target, data, seeds and budgets; only the producer is changed.
pub fn compare_producers(before:&SearchPolicy,after:&SearchPolicy,case:&ProducerCase,
    object:&SearchLimits,meta:&SearchLimits,seed:u64)->Result<ProducerComparison>{
    if case.training.is_empty()||case.examination.is_empty(){return Err(Error::InvalidData);}
    for c in &case.examination{c.validate()?;}
    let target=SearchPolicy::default();
    let before_creation=produce(before,&target,&case.training,object,meta,seed)?;
    let after_creation=produce(after,&target,&case.training,object,meta,seed)?;
    let children=paired(&SearchPolicy::from_code(&before_creation.program)?,&SearchPolicy::from_code(&after_creation.program)?,
        &seeded(&case.examination,seed^0xC41D),object)?;
    Ok(ProducerComparison{before_creation,after_creation,children})
}
fn legacy_admission_version()->u8{1}
fn is_legacy_admission(v:&u8)->bool{*v==1}

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
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub qualification:Option<QualificationAudit>,
}
/// Host-owned lifecycle. No method accepts a user-supplied success boolean or
/// a requested generation number. Every adoption follows executed production
/// and a previously unused validation batch.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveSearch {
    config:EvolutionConfig,active:SearchPolicy,generation:u32,work:u64,
    audits:Vec<PolicyAudit>,used_gate_inputs:BTreeSet<u64>,frozen:bool,
    #[serde(default="legacy_admission_version",skip_serializing_if="is_legacy_admission")]
    admission_version:u8,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    retained:Vec<RetainedCase>,
}
fn input_key(task:&SynthesisTask)->u64{
    let mut rows:Vec<_>=task.examples.iter().map(|r|&r.input).collect();rows.sort();rows.dedup();
    super::vm::fingerprint(&serde_json::to_vec(&rows).unwrap())
}
impl RecursiveSearch {
    pub fn new(config:EvolutionConfig)->Result<Self>{
        config.validate()?;
        Ok(Self{config,active:SearchPolicy::default(),generation:0,work:0,audits:Vec::new(),used_gate_inputs:BTreeSet::new(),frozen:false,admission_version:2,retained:Vec::new()})
    }
    pub fn policy(&self)->&SearchPolicy{&self.active}
    pub fn generation(&self)->u32{self.generation}
    pub fn work(&self)->u64{self.work}
    pub fn audits(&self)->&[PolicyAudit]{&self.audits}
    pub fn retained(&self)->&[RetainedCase]{&self.retained}
    pub fn freeze(&mut self){self.frozen=true;}
    pub fn is_frozen(&self)->bool{self.frozen}
    pub fn validate(&self)->Result<()>{
        self.config.validate()?;self.active.validate()?;
        if !(1..=2).contains(&self.admission_version)||self.retained.len()>MAX_RETENTION||self.audits.len()>64||self.used_gate_inputs.len()>8192||self.work>self.config.max_lifetime_work{return Err(Error::InvalidData);}
        let mut policy=SearchPolicy::default();let mut generation=0;let mut sum=0u64;let mut seen=BTreeSet::new();let mut retained=Vec::<RetainedCase>::new();
        for (i,a) in self.audits.iter().enumerate(){
            a.candidate.validate()?;
            if a.attempt!=i as u64||a.producer!=policy||a.generation_before!=generation||a.production.policy_id!=policy.fingerprint()
                ||a.production.program!=a.candidate.code()||a.production.work>a.consumed_work
                ||a.before.tasks!=a.gate_ids.len()||a.after.tasks!=a.gate_ids.len()
                ||a.before.runs.len()!=a.gate_ids.len()||a.after.runs.len()!=a.gate_ids.len(){return Err(Error::InvalidData);}
            let mut replay=policy.code();
            for edit in &a.production.edits_to_winner{replay=edit.apply(&replay)?;}
            if replay!=a.candidate.code(){return Err(Error::InvalidData);}
            for key in &a.gate_input_keys{if !seen.insert(*key)||!self.used_gate_inputs.contains(key){return Err(Error::InvalidData);}}
            let changed=if self.admission_version==2 {a.qualification.as_ref().ok_or(Error::InvalidData)?.behaviour_changed()}
                else {a.before.runs.iter().zip(&a.after.runs).any(|(x,y)|x.trace_hash!=y.trace_hash)};
            if changed!=a.behaviour_changed{return Err(Error::InvalidData);}
            validate_suite(&a.before,&self.config.object)?;validate_suite(&a.after,&self.config.object)?;
            let accepted=if self.admission_version==1 {
                if a.qualification.is_some()||a.gate_input_keys.len()!=a.gate_ids.len()||a.consumed_work!=a.production.work+a.before.work+a.after.work{return Err(Error::InvalidData);}
                changed && a.candidate!=policy && a.after.solved>=a.before.solved && a.after.value<a.before.value
                    && a.after.value.saturating_mul(100)<=a.before.value.saturating_mul(95)
            } else {
                let q=a.qualification.as_ref().ok_or(Error::InvalidData)?;
                for pair in [&q.fresh,&q.transfer,&q.retention]{
                    pair.validate(&self.config.object)?;
                    if pair.before.search.runs.iter().any(|r|r.policy_id!=policy.fingerprint())
                        ||pair.after.search.runs.iter().any(|r|r.policy_id!=a.candidate.fingerprint()){return Err(Error::InvalidData);}
                }
                if q.fresh.before.search!=a.before||q.fresh.after.search!=a.after
                    ||q.fresh.before.search.tasks<8||q.transfer.before.search.tasks<8
                    ||q.retention.before.probes.iter().map(|p|(p.case_id,p.seed)).collect::<Vec<_>>()
                        !=retained.iter().map(|r|(r.case.id(),r.seed)).collect::<Vec<_>>()
                    ||a.consumed_work!=a.production.work+q.work(){return Err(Error::InvalidData);}
                for p in &q.producers {
                    p.children.validate(&self.config.object)?;
                    for (creation,producer,child) in [(&p.before_creation,&policy,&p.children.before),(&p.after_creation,&a.candidate,&p.children.after)] {
                        if creation.policy_id!=producer.fingerprint()||creation.work>self.config.meta.work{return Err(Error::InvalidData);}
                        let mut body=SearchPolicy::default().code();for edit in &creation.edits_to_winner{body=edit.apply(&body)?;}
                        if body!=creation.program{return Err(Error::InvalidData);}
                        let child_id=SearchPolicy::from_code(&body)?.fingerprint();
                        if child.search.runs.iter().any(|r|r.policy_id!=child_id){return Err(Error::InvalidData);}
                    }
                }
                let ok=changed&&a.candidate!=policy&&q.direct_pass(&self.config.object)&&q.producer_pass(&self.config.object);
                if !ok&&!q.added_retention.is_empty(){return Err(Error::InvalidData);}
                let expected:Vec<_>=if ok { [&q.fresh.after,&q.transfer.after].iter().flat_map(|s|s.search.runs.iter().zip(&s.probes))
                    .filter(|(run,p)|run.solved&&p.probe_errors==0&&!retained.iter().any(|r|r.case.id()==p.case_id))
                    .map(|(_,p)|(p.case_id,p.seed)).collect() } else {Vec::new()};
                if expected!=q.added_retention.iter().map(|r|(r.case.id(),r.seed)).collect::<Vec<_>>(){return Err(Error::InvalidData);}
                for r in &q.added_retention {
                    r.case.validate()?;
                    if retained.iter().any(|old|old.case.id()==r.case.id()) {return Err(Error::InvalidData);}
                    let evidence=[&q.fresh.after,&q.transfer.after].iter().any(|s|s.search.runs.iter().zip(&s.probes)
                        .any(|(run,p)|run.solved&&p.probe_errors==0&&p.case_id==r.case.id()&&p.seed==r.seed));
                    if !evidence{return Err(Error::InvalidData);}retained.push(r.clone());
                }
                ok
            };
            if accepted!=a.accepted{return Err(Error::InvalidData);}
            if accepted {policy=a.candidate.clone();generation+=1;}
            if a.generation_after!=generation{return Err(Error::InvalidData);}
            sum=sum.checked_add(a.consumed_work).ok_or(Error::Capacity)?;
        }
        if self.active!=policy||self.generation!=generation||sum>self.work||retained!=self.retained{return Err(Error::InvalidData);}
        Ok(())
    }
    pub fn improve(&mut self,training:&[SynthesisTask],gate:&AdmissionSuite,seed:u64)->Result<&PolicyAudit>{
        if self.frozen{return Err(Error::Blocked);}
        // Old checkpoints remain readable and executable. Their historical
        // admissions cannot be relabelled as having passed the new contract.
        if self.admission_version!=2{return Err(Error::Stale);}
        if self.audits.len()>=64||training.is_empty()||training.len()>64||!(8..=64).contains(&gate.fresh.len())
            ||!(8..=64).contains(&gate.transfer.len())||!(2..=4).contains(&gate.producers.len()){return Err(Error::Capacity);}
        let mut train_keys=BTreeSet::new();let mut gate_keys=BTreeSet::new();
        for task in training{task.validate()?;let key=input_key(task);if self.used_gate_inputs.contains(&key){return Err(Error::Stale);}train_keys.insert(key);}
        let mut reserve=self.config.meta.work+4*self.config.object.work*(gate.fresh.len()+gate.transfer.len()+self.retained.len()) as u64;
        let mut all_tasks=Vec::new();
        for c in gate.fresh.iter().chain(&gate.transfer){c.validate()?;all_tasks.extend([&c.support,&c.probes]);}
        for p in &gate.producers {
            if !(1..=8).contains(&p.training.len())||!(2..=16).contains(&p.examination.len()){return Err(Error::Capacity);}
            all_tasks.extend(p.training.iter());
            for c in &p.examination {c.validate()?;all_tasks.extend([&c.support,&c.probes]);}
            reserve=reserve.checked_add(2*self.config.meta.work+4*self.config.object.work*p.examination.len() as u64).ok_or(Error::Capacity)?;
        }
        for task in all_tasks{task.validate()?;let key=input_key(task);
            if self.used_gate_inputs.contains(&key)||train_keys.contains(&key)||!gate_keys.insert(key){return Err(Error::Stale);}
        }
        if self.used_gate_inputs.len()+gate_keys.len()>8192{return Err(Error::Capacity);}
        let reservation=reserve;
        if self.work.checked_add(reservation).is_none_or(|n|n>self.config.max_lifetime_work){return Err(Error::Budget);}
        // Consume before execution. Errors retain reservation and single-use
        // gate records rather than creating free retries or double spending.
        self.work+=reservation;self.used_gate_inputs.extend(&gate_keys);
        let production=produce(&self.active,&self.active,training,&self.config.object,&self.config.meta,seed)?;
        let candidate=SearchPolicy::from_code(&production.program)?;
        let fresh=seeded(&gate.fresh,seed^0xBA5E);let transfer=seeded(&gate.transfer,seed^0x7A45);
        let mut q=QualificationAudit{
            fresh:paired(&self.active,&candidate,&fresh,&self.config.object)?,
            transfer:paired(&self.active,&candidate,&transfer,&self.config.object)?,
            retention:paired(&self.active,&candidate,&self.retained,&self.config.object)?,
            producers:Vec::new(),added_retention:Vec::new(),
        };
        let changed=q.behaviour_changed();
        if changed&&candidate!=self.active&&q.direct_pass(&self.config.object) {
            for (i,p) in gate.producers.iter().enumerate() {
                let s=seed.wrapping_add(0x91000+i as u64*0x10001);
                q.producers.push(compare_producers(&self.active,&candidate,p,&self.config.object,&self.config.meta,s)?);
            }
        }
        let accepted=changed&&candidate!=self.active&&q.direct_pass(&self.config.object)&&q.producer_pass(&self.config.object);
        if accepted {
            for (cases,score) in [(&fresh,&q.fresh.after),(&transfer,&q.transfer.after)] {
                for (c,(r,p)) in cases.iter().zip(score.search.runs.iter().zip(&score.probes)) {
                    if r.solved&&p.probe_errors==0&&!self.retained.iter().any(|old|old.case.id()==c.case.id()) {q.added_retention.push(c.clone());}
                }
            }
            if self.retained.len()+q.added_retention.len()>MAX_RETENTION{return Err(Error::Capacity);}
        }
        let spent=production.work+q.work();
        self.work=self.work.checked_sub(reservation.checked_sub(spent).ok_or(Error::InvalidData)?).ok_or(Error::InvalidData)?;
        let before=q.fresh.before.search.clone();let after=q.fresh.after.search.clone();
        let producer=self.active.clone();let old_generation=self.generation;
        if accepted{self.active=candidate.clone();self.generation+=1;self.retained.extend(q.added_retention.iter().cloned());}
        self.audits.push(PolicyAudit{attempt:self.audits.len() as u64,generation_before:old_generation,generation_after:self.generation,
            producer,candidate,seed,training_ids:training.iter().map(SynthesisTask::id).collect(),gate_ids:gate.fresh.iter().map(|c|c.support.id()).collect(),gate_input_keys:gate_keys.into_iter().collect(),
            production,before,after,behaviour_changed:changed,accepted,consumed_work:spent,qualification:Some(q)});
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
