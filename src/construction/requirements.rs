//! LC1: executable *learned* backward requirements for missing computations.
//!
//! A right-operand inverse is selected from pure VM expressions using observed
//! completed executions (left, right, result), not an authored inverse table.
//! It proposes a missing subcomputation; ordinary forward execution is the
//! only correctness authority. No learner ever sees hidden probe labels.
use super::induction::{self, InductionConfig};
use super::search_policy::SynthesisTask;
use super::vm::{Context, Error, Expr, Library, Meter, Op, Result, Ty, Value};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutedOperation {
    pub op: Op,
    pub left: i64,
    pub right: i64,
    pub output: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InverseRule {
    pub op: Op,
    /// Input(0) = known left, Input(1) = desired operation result.
    pub right: Expr,
    pub witnessed: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RequirementMemory {
    observations: Vec<ExecutedOperation>,
    rules: Vec<InverseRule>,
    #[serde(default)]
    frozen: bool,
    #[serde(default)]
    work: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementReport {pub observations: usize,pub learned_rules: usize,pub work: u64,pub frozen: bool}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionLimits {pub work: u64,pub nodes: usize,pub terms: usize}
impl Default for CompletionLimits {
    fn default()->Self {Self{work:1_000_000,nodes:7,terms:2048}}
}
impl CompletionLimits {
    pub fn validate(&self)->Result<()> {
        if !(10_000..=20_000_000).contains(&self.work)||!(3..=11).contains(&self.nodes)||!(16..=32768).contains(&self.terms){return Err(Error::Capacity)}
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionReport {
    pub program: Option<Expr>,pub required_values: Vec<i64>,pub work: u64,
    pub tried_rules: usize,pub forward_verified: bool,pub used_acquired_inverse: bool,
}
// LC1 delegates missing-fragment inference to induction::infer, which accepts
// at most 128 support rows. SynthesisTask itself permits up to 256, so LC1
// must enforce its own narrower contract BEFORE reserving runtime work.
pub const MAX_LC1_SUPPORT_ROWS: usize = 128;
pub fn validate_completion_task(task: &SynthesisTask) -> Result<()> {
    task.validate()?;
    if task.examples.len() < 4 { return Err(Error::InvalidData); }
    if task.examples.len() > MAX_LC1_SUPPORT_ROWS { return Err(Error::Capacity); }
    Ok(())
}
fn inputs(op: Op)->bool {op.input()==Ty::Int && op.output()==Ty::Int}
fn candidates()->Vec<Expr>{
    let mut leaves=vec![Expr::Input(0),Expr::Input(1)];
    leaves.extend([-2,-1,0,1,2].map(Expr::integer));
    let mut result=leaves.clone();
    for op in [Op::Add,Op::Sub,Op::Mul,Op::DivExact,Op::Xor] {
        for a in &leaves {for b in &leaves {
            result.push(Expr::bin(op,a.clone(),b.clone()));
        }}
    }
    result.sort_by(|a,b|a.nodes().cmp(&b.nodes()).then_with(||a.cmp(b)));
    result.dedup();result
}
impl RequirementMemory {
    pub fn report(&self)->RequirementReport {
        RequirementReport{observations:self.observations.len(),learned_rules:self.rules.len(),work:self.work,frozen:self.frozen}
    }
    pub fn rules(&self)->&[InverseRule] {&self.rules}
    pub fn freeze(&mut self){self.frozen=true;}
    pub fn validate(&self)->Result<()> {
        if self.observations.len()>1024||self.rules.len()>8 {return Err(Error::Capacity)}
        for r in &self.observations {
            if !inputs(r.op)||r.op.apply(Value::Int(r.left),Value::Int(r.right))!=Ok(Value::Int(r.output)) {return Err(Error::InvalidData)}
        }
        let mut seen=BTreeSet::new();
        for rule in &self.rules {
            if !seen.insert(rule.op)||!inputs(rule.op)||rule.right.nodes()>3||rule.witnessed>self.observations.len()||rule.witnessed<12 {
                return Err(Error::InvalidData)
            }
            let rows:Vec<_>=self.observations.iter().filter(|r|r.op==rule.op).collect();
            if rows.len()!=rule.witnessed{return Err(Error::InvalidData)}
            for r in rows {
                let mut meter=Meter::new(100);
                let predicted=rule.right.eval(Context{input:&[r.left,r.output],action:0,state:&[],args:&[]},&Library::default(),&mut meter);
                if predicted!=Ok(Value::Int(r.right)){return Err(Error::InvalidData)}
            }
        }
        Ok(())
    }
    /// Atomic intake: all new records must be real, coherent *forward* executions.
    /// The inverse expression is inferred from the records without labels of
    /// expected inverse programs or handcrafted operator-to-inverse lookup.
    pub fn observe(&mut self,new:&[ExecutedOperation])->Result<RequirementReport>{
        if self.frozen{return Err(Error::Stale)}
        if new.is_empty()||new.len()>256||self.observations.len()+new.len()>1024{return Err(Error::Capacity)}
        for r in new {
            if !inputs(r.op)||r.op.apply(Value::Int(r.left),Value::Int(r.right))!=Ok(Value::Int(r.output)){return Err(Error::InvalidData)}
        }
        let mut next=self.clone();next.observations.extend_from_slice(new);
        let mut selected=Vec::new();
        for op in [Op::Add,Op::Sub,Op::Mul,Op::DivExact,Op::Xor,Op::Rem] {
            let rows:Vec<_>=next.observations.iter().filter(|r|r.op==op).collect();
            if rows.len()<12 {continue}
            let distinct_l:BTreeSet<_>=rows.iter().map(|r|r.left).collect();
            let distinct_r:BTreeSet<_>=rows.iter().map(|r|r.right).collect();
            let distinct_z:BTreeSet<_>=rows.iter().map(|r|r.output).collect();
            if distinct_l.len()<5||distinct_r.len()<5||distinct_z.len()<5{continue}
            let mut labels=BTreeMap::new();let mut identifiable=true;
            for r in &rows {
                if labels.insert((r.left,r.output),r.right).is_some_and(|old|old!=r.right){identifiable=false;break}
            }
            if !identifiable{continue}
            for candidate in candidates(){
                let mut correct=true;
                for r in &rows {
                    next.work=next.work.saturating_add(1);
                    let mut meter=Meter::new(50);
                    let predicted=candidate.eval(Context{input:&[r.left,r.output],action:0,state:&[],args:&[]},&Library::default(),&mut meter);
                    next.work=next.work.saturating_add(meter.spent);
                    if predicted!=Ok(Value::Int(r.right)) {correct=false;break;}
                }
                if correct{selected.push(InverseRule{op,right:candidate,witnessed:rows.len()});break}
            }
        }
        next.rules=selected;next.validate()?;*self=next;Ok(self.report())
    }
    /// The learned rule is only a hint; it must reproduce the desired output
    /// under the *actual* forward VM, otherwise refuse this hint.
    pub fn requirement(&self,op:Op,left:i64,goal:i64,meter:&mut Meter)->Result<Option<i64>> {
        let Some(rule)=self.rules.iter().find(|r|r.op==op) else {return Ok(None)};
        let guess=rule.right.eval(Context{input:&[left,goal],action:0,state:&[],args:&[]},&Library::default(),meter);
        let guess=match guess{Ok(Value::Int(x))=>x,Err(Error::Budget)=>return Err(Error::Budget),_=>return Ok(None)};
        meter.charge(1)?;
        Ok(if op.apply(Value::Int(left),Value::Int(guess))==Ok(Value::Int(goal)){Some(guess)}else{None})
    }
    /// Complete an explicitly identified missing right subcomputation. Support
    /// outcomes are shown to the solver; withheld probes are never passed in.
    pub fn complete(&self,task:&SynthesisTask,sketch:&Expr,limits:&CompletionLimits)->Result<CompletionReport>{
        validate_completion_task(task)?;limits.validate()?;
        let Expr::Bin(op,known,missing)=sketch else {return Err(Error::InvalidProgram)};
        if !matches!(missing.as_ref(),Expr::Hole{id:0,ty:Ty::Int}) || !inputs(*op)||sketch.nodes()>limits.nodes+4{return Err(Error::InvalidProgram)}
        if known.first_hole().is_some()||known.ty(&task.library)!=Ok(Ty::Int){return Err(Error::InvalidProgram)}
        if self.rules.iter().all(|r|r.op!=*op) {return Ok(CompletionReport{program:None,required_values:vec![],work:0,tried_rules:0,forward_verified:false,used_acquired_inverse:false})}
        let mut meter=Meter::new(limits.work);
        let mut needed=Vec::with_capacity(task.examples.len());
        for row in &task.examples {
            let lhs=known.eval(Context{input:&row.input,action:0,state:&[],args:&[]},&task.library,&mut meter)?.int()?;
            let Some(right)=self.requirement(*op,lhs,row.output,&mut meter)? else {
                return Ok(CompletionReport{program:None,required_values:needed,work:meter.spent,tried_rules:1,forward_verified:false,used_acquired_inverse:false});
            };
            needed.push(right);
        }
        let demand=task.examples.iter().zip(&needed).map(|(r,&y)|induction::Example{input:r.input.clone(),output:y}).collect::<Vec<_>>();
        let remaining=meter.remaining();
        if remaining<100 {return Ok(CompletionReport{program:None,required_values:needed,work:meter.spent,tried_rules:1,forward_verified:false,used_acquired_inverse:true})}
        let induction=induction::infer(&demand,&task.library,&InductionConfig{max_nodes:limits.nodes.saturating_sub(known.nodes()+1).max(3),max_terms:limits.terms,work:remaining})?;
        meter.charge(induction.work)?;
        let Some(fragment)=induction.expression else {return Ok(CompletionReport{program:None,required_values:needed,work:meter.spent,tried_rules:1,forward_verified:false,used_acquired_inverse:true})};
        let completed=sketch.substitute(0,&fragment);
        let verified=completed.nodes()<=limits.nodes && task.examples.iter().all(|r| {
            completed.eval(Context{input:&r.input,action:0,state:&[],args:&[]},&task.library,&mut meter)==Ok(Value::Int(r.output))
        });
        Ok(CompletionReport{program:verified.then_some(completed),required_values:needed,work:meter.spent,tried_rules:1,forward_verified:verified,used_acquired_inverse:true})
    }

    /// Find the first missing intermediate expression without a caller-supplied
    /// skeleton. The search *generates* typed one-hole hypotheses from learned
    /// operator relations and input compositions; the VM alone verifies a fit.
    /// Ordinary RSP2 induction is left available and never pruned by this hint.
    pub fn discover(&self,task:&SynthesisTask,limits:&CompletionLimits)->Result<CompletionReport>{
        validate_completion_task(task)?;limits.validate()?;
        if self.rules.is_empty(){return Ok(CompletionReport{program:None,required_values:vec![],work:0,tried_rules:0,forward_verified:false,used_acquired_inverse:false})}
        let width=task.examples[0].input.len().min(6);
        let mut known=(0..width).map(Expr::Input).collect::<Vec<_>>();
        for a in 0..width {for b in a+1..width {
            for op in [Op::Add,Op::Sub,Op::Mul] {
                known.push(Expr::bin(op,Expr::Input(a),Expr::Input(b)));
            }
        }}
        let mut total=0u64;let mut tried=0;
        for stem in known {
            for rule in &self.rules {
                if total+10_000>limits.work {break}
                let sub=CompletionLimits{work:limits.work.saturating_sub(total).min(120_000),nodes:limits.nodes,terms:limits.terms};
                let sketch=Expr::bin(rule.op,stem.clone(),Expr::Hole{id:0,ty:Ty::Int});
                let result=self.complete(task,&sketch,&sub);
                tried+=1;
                match result {
                    Ok(mut r)=>{
                        total=total.saturating_add(r.work);
                        if r.forward_verified {
                            r.work=total;r.tried_rules=tried;return Ok(r)
                        }
                    },
                    Err(Error::Budget)=>{total=total.saturating_add(sub.work);},
                    Err(Error::Overflow)|Err(Error::InvalidData)=>{total=total.saturating_add(sub.work);},
                    Err(error)=>return Err(error),
                }
            }
        }
        Ok(CompletionReport{program:None,required_values:vec![],work:total,tried_rules:tried,forward_verified:false,used_acquired_inverse:tried>0})
    }
}
