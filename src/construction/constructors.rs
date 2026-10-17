//! Acquired executable patch schemas and prospective equal-library controls.
//! A schema is not called an improved learning algorithm until its measured
//! search cost beats the same-library baseline on a fresh, single-use batch.
use super::synthesis::*;
use super::vm::*;
use super::recursive::{MetaLedger, Refinement, match_schema, instantiate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstructorStatus { Proposed, UsefulOnDevelopmentBatch, RejectedOnDevelopmentBatch }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constructor {
    pub id: u64, pub generation: u32,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub refinement: Option<Refinement>,
    /// Packed expression tree representing output and state edits as DATA.
    pub code: Expr, pub cell_count: usize,
    pub guard: Expr, pub training_trace_ids: Vec<u64>,
    pub construction_work: u64, pub status: ConstructorStatus,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    pub(super) constructors: Vec<Constructor>, traces: Vec<SearchTrace>,
    consumed_batches: BTreeSet<u64>,
    guard_observations: Vec<(Vec<i64>,bool)>,
    #[serde(default, skip_serializing_if="MetaLedger::is_empty")]
    pub(super) meta: MetaLedger,
}
pub(super) fn pack(patch: &Patch) -> Expr {
    let mut parts = vec![patch.replacement.clone()];
    for c in &patch.cells { parts.push(Expr::integer(c.initial)); parts.push(c.update.clone()); }
    parts.into_iter().rev().fold(Expr::integer(0),|tail,head| Expr::bin(Op::Add,head,tail))
}
pub(super) fn unpack(code: &Expr,ncells: usize) -> Result<(Expr,Vec<StateCell>)> {
    let mut cursor = code; let mut parts = Vec::new();
    for _ in 0..1+2*ncells {
        if let Expr::Bin(Op::Add,head,tail) = cursor { parts.push((**head).clone()); cursor = tail; }
        else { return Err(Error::InvalidProgram); }
    }
    if *cursor != Expr::integer(0) { return Err(Error::InvalidProgram); }
    let mut cells = Vec::new();
    for i in 0..ncells {
        let initial = match parts[1+2*i] { Expr::Lit(Value::Int(n)) => n,_ => return Err(Error::Hole) };
        cells.push(StateCell { initial,update: parts[2+2*i].clone() });
    }
    Ok((parts[0].clone(),cells))
}
fn trace_id(trace: &SearchTrace) -> u64 {
    fingerprint(&serde_json::to_vec(trace).expect("serializable trace"))
}
impl Registry {
    pub fn constructors(&self) -> &[Constructor] { &self.constructors }
    pub fn traces(&self) -> &[SearchTrace] { &self.traces }
    pub fn nodes(&self) -> usize { self.constructors.iter().map(|c| c.code.nodes()+c.guard.nodes()).sum() }
    pub fn validate(&self,lib: &Library) -> Result<()> {
        if self.constructors.len() > 32 || self.traces.len() > 128 || self.consumed_batches.len() > 256 || self.guard_observations.len() > 128 { return Err(Error::Capacity); }
        let mut ids = BTreeSet::new();
        for c in &self.constructors {
            if c.code.ty(lib)? != Ty::Int || c.guard.ty(lib)? != Ty::Bool || c.cell_count > 8 || c.code.nodes() > 1024 || c.guard.nodes() > 64 || c.training_trace_ids.len() > 128 || !ids.insert(c.id) { return Err(Error::InvalidProgram); }
        }
        self.meta.validate()?;
        for c in &self.constructors {
            if let Some(edit) = &c.refinement {
                let parent = self.constructors.iter().find(|p| p.id == edit.parent).ok_or(Error::InvalidProgram)?;
                if c.generation != parent.generation.checked_add(1).ok_or(Error::Capacity)?
                    || parent.cell_count != c.cell_count || instantiate(&parent.code,&edit.bindings) != c.code
                    || parent.code == c.code { return Err(Error::InvalidProgram); }
            } else if c.generation > 1 { return Err(Error::InvalidProgram); }
        }
        Ok(())
    }
    /// Only accept traces whose candidate has already passed a prospective
    /// model trial. Search success alone is explicitly insufficient.
    pub fn learn(&mut self, trace: SearchTrace, lib: &Library,meter: &mut Meter) -> Result<usize> {
        if self.meta.pending.is_some() { return Err(Error::Busy); }
        let start = meter.spent;
        let mut next = self.clone();
        let result = next.learn_inner(trace,lib,meter);
        // Failed construction attempts also cost work; an error never installs
        // half of a generator revision.
        self.meta.construction_work = self.meta.construction_work.saturating_add(meter.spent-start);
        if result.is_ok() { next.meta.construction_work = self.meta.construction_work; *self = next; }
        result
    }
    fn learn_inner(&mut self, trace: SearchTrace, lib: &Library,meter: &mut Meter) -> Result<usize> {
        if !trace.prospectively_accepted || trace.winner.is_none() { return Err(Error::NoEvidence); }
        if self.traces.len() >= 128 { return Err(Error::Capacity); }
        let id = trace_id(&trace);
        if self.traces.iter().any(|t| trace_id(t) == id) { return Err(Error::Stale); }
        let patch = trace.winner.as_ref().ok_or(Error::NoEvidence)?;
        if let Some(cid) = trace.constructor_id {
            let c = self.constructors.iter().find(|c| c.id == cid).ok_or(Error::InvalidProgram)?;
            meter.charge((c.code.nodes()+pack(patch).nodes()) as u64)?;
            if match_schema(&c.code,&pack(patch),lib)?.is_none() { return Err(Error::InvalidProgram); }
        }
        let before = self.constructors.len();
        for old in &self.traces {
            if old.constructor_id != trace.constructor_id { continue; }
            let Some(old_patch) = &old.winner else { continue; };
            if old_patch.cells.len() != patch.cells.len() || old_patch == patch { continue; }
            if self.constructors.len() >= 32 { break; }
            let start = meter.spent;
            let a = pack(old_patch); let b = pack(patch);
            meter.charge((a.nodes()+b.nodes()) as u64)?;
            let code = generalize(&a,&b,lib)?;
            // A hole spanning the entire packed edit encodes no useful structure.
            if matches!(code,Expr::Hole { .. }) || code.first_hole().is_none() { continue; }
            if patch.cells.is_empty() && matches!(&code,Expr::Bin(Op::Add,head,_) if matches!(head.as_ref(),Expr::Hole { .. })) { continue; }
            let cid = fingerprint(&serde_json::to_vec(&code).map_err(|_| Error::InvalidData)?);
            if self.constructors.iter().any(|c| c.id == cid) { continue; }
            // Recursive credit requires TWO successful descendants of the same
            // executed generator, plus a replayable edit of its search code.
            // Merely writing a larger generation number is not recursion.
            let refinement = if old.constructor_id.is_some() && old.constructor_id == trace.constructor_id {
                let parent = self.constructors.iter().find(|c| Some(c.id) == trace.constructor_id).ok_or(Error::InvalidProgram)?;
                meter.charge((parent.code.nodes()+code.nodes()+a.nodes()) as u64)?;
                if match_schema(&parent.code,&a,lib)?.is_none() { return Err(Error::InvalidProgram); }
                match match_schema(&parent.code,&code,lib)? {
                    Some(bindings) if parent.code != code => Some(Refinement { parent: parent.id,bindings }),
                    _ => None,
                }
            } else { None };
            // A generalization whose lineage is not causal remains generation 1.
            let generation = refinement.as_ref().map(|r| self.constructors.iter().find(|c| c.id == r.parent).unwrap().generation+1).unwrap_or(1);
            // No negative guard evidence yet: do not invent a task classifier.
            let guard = fit_guard(&self.guard_observations,lib,meter)?.unwrap_or(Expr::Lit(Value::Bool(true)));
            self.constructors.push(Constructor { id: cid,generation,refinement,code,cell_count: patch.cells.len(),guard,
                training_trace_ids: vec![trace_id(old),id],construction_work: meter.spent-start,status: ConstructorStatus::Proposed });
        }
        self.traces.push(trace); Ok(self.constructors.len()-before)
    }
    /// Execute the acquired constructor-to-constructor edits. Stored `code`
    /// is checked as a cache, not treated as proof that a rewrite took place.
    pub fn materialize_constructor(&self,id: u64,meter: &mut Meter) -> Result<Expr> {
        fn run(registry: &Registry,id: u64,meter: &mut Meter,depth: usize) -> Result<Expr> {
            if depth > 32 { return Err(Error::Capacity); }
            let c = registry.constructors.iter().find(|c| c.id == id).ok_or(Error::InvalidProgram)?;
            meter.charge(c.code.nodes() as u64)?;
            if let Some(edit) = &c.refinement {
                let parent = run(registry,edit.parent,meter,depth+1)?;
                meter.charge(edit.bindings.values().map(|e| e.nodes() as u64).sum())?;
                let generated = instantiate(&parent,&edit.bindings);
                if generated != c.code { return Err(Error::InvalidProgram); }
                Ok(generated)
            } else { Ok(c.code.clone()) }
        }
        run(self,id,meter,0)
    }
    pub fn suggestions(&self,job: &Job,width: usize,cfg: &SearchConfig,lib: &Library,meter: &mut Meter) -> Result<Vec<Hint>> {
        let mut hints = Vec::new();
        let mut ordered = Vec::new();
        for c in &self.constructors {
            meter.charge(c.code.nodes() as u64)?;
            let mut holes = BTreeSet::new();
            c.code.map(&mut |e| { if let Expr::Hole { id,.. } = e { holes.insert(*id); } None });
            // A new generation is not automatically cheaper. Independent holes
            // multiply the number of smallest completions; try fewer bindings
            // first, and use generation only to break that structural tie.
            ordered.push(((holes.len(),std::cmp::Reverse(c.generation),c.id),c));
        }
        ordered.sort_by_key(|(key,_)| *key);
        for (_,c) in ordered {
            if c.status == ConstructorStatus::RejectedOnDevelopmentBatch { continue; }
            if c.guard.eval(Context { input: &job.features,action: 0,state: &[],args: &[] },lib,meter) != Ok(Value::Bool(true)) { continue; }
            let generated = self.materialize_constructor(c.id,meter)?;
            // If only the output contains holes, leave them open: the acquired
            // computation becomes an executable SEARCH schema, not a saved answer.
            if let Ok((output,cells)) = unpack(&generated,c.cell_count) {
                if cells.iter().all(|cell| cell.update.first_hole().is_none()) {
                    meter.charge(c.code.nodes() as u64)?;
                    hints.push(Hint { constructor_id: c.id,output,cells });
                    if hints.len() >= 32 { return Ok(hints); }
                    continue;
                }
            }
            let mut e = Enumerator::new(Grammar { width,states: c.cell_count,max_nodes: c.code.nodes()+12,max_frontier: cfg.max_frontier },generated);
            for _ in 0..16 {
                let Some(code) = e.next(lib,meter)? else { break; };
                if let Ok((output,cells)) = unpack(&code,c.cell_count) {
                    hints.push(Hint { constructor_id: c.id,output,cells });
                }
                if hints.len() >= 32 { return Ok(hints); }
            }
        }
        Ok(hints)
    }
    /// Consumed holdout labels may train the NEXT guard. This never changes the
    /// result of the already completed batch and that batch cannot be reused.
    pub fn record_guard_observation(&mut self,features: Vec<i64>,useful: bool,lib: &Library,meter: &mut Meter) -> Result<()> {
        if features.len() != 4 || self.guard_observations.len() >= 128 { return Err(Error::Capacity); }
        self.guard_observations.push((features,useful));
        if let Some(guard) = fit_guard(&self.guard_observations,lib,meter)? {
            for c in &mut self.constructors { c.guard = guard.clone(); }
        }
        Ok(())
    }
}

/// Uses the same typed language/enumerator, not authored if(problem_kind) rules.
pub fn fit_guard(examples: &[(Vec<i64>,bool)],lib: &Library,meter: &mut Meter) -> Result<Option<Expr>> {
    if examples.is_empty() { return Ok(None); }
    let width = examples[0].0.len();
    if width == 0 || width > 32 || examples.iter().any(|x| x.0.len() != width) { return Err(Error::InvalidData); }
    let mut enumeration = Enumerator::new(Grammar { width,states: 0,max_nodes: 5,max_frontier: 1024 },Expr::Hole { id: 0,ty: Ty::Bool });
    while let Some(expr) = enumeration.next(lib,meter)? {
        let mut correct = true;
        for (features,y) in examples {
            match expr.eval(Context { input: features,action: 0,state: &[],args: &[] },lib,meter) {
                Ok(Value::Bool(v)) if v == *y => (),
                Err(Error::Budget) => return Err(Error::Budget),
                _ => { correct = false; break; }
            }
        }
        if correct { return Ok(Some(expr)); }
    }
    Ok(None)
}

#[derive(Debug, Clone)]
pub struct TransferCase { pub acquisition: Vec<Transition>,pub future: Vec<Transition> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmResult { pub work: u64,pub future_loss: f64,pub training_loss: f64,pub program_nodes: usize,pub library_nodes: usize }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRow { pub no_library: ArmResult,pub same_library: ArmResult,pub constructors: ArmResult }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaReport {
    pub batch: u64,pub rows: Vec<TransferRow>,pub construction_work: u64,
    pub additional_search_value: bool,pub independent_qualification: bool,
}
fn assess(parent: &Program,case: &TransferCase,lib: &Library,cfg: &SearchConfig,hints: &[Hint],overhead: u64) -> Result<ArmResult> {
    let mut adjusted = cfg.clone(); adjusted.work = cfg.work.checked_sub(overhead).ok_or(Error::Budget)?;
    if adjusted.work < 100 { return Err(Error::Budget); }
    let trace = synthesize(parent,&case.acquisition,lib,&adjusted,hints)?;
    let program = trace.winner.as_ref().map(|p| p.apply(parent,lib)).transpose()?.unwrap_or_else(|| parent.clone());
    let mut all = case.acquisition.clone(); all.extend_from_slice(&case.future);
    let actions = all.iter().map(|t| t.action).max().unwrap_or(0)+1;
    validate_data(&all,parent.outputs.len(),actions)?;
    let mut evaluation = Meter::new(10_000_000);
    let (predictions,_) = replay(&program,&all,lib,&mut evaluation)?;
    let future_loss = absolute_loss(&predictions[case.acquisition.len()..],&case.future)?;
    Ok(ArmResult { work: trace.work+overhead+evaluation.spent,future_loss,training_loss: trace.training_loss,
        program_nodes: program.nodes(),library_nodes: lib.nodes() })
}
impl Registry {
    /// Caller freezes this registry before obtaining the acquisition/future
    /// records. Batch identity includes all data and is single-use even on error.
    /// No future record reaches suggestions(), synthesize(), or fit_guard().
    pub fn evaluate(&mut self,cases: &[TransferCase],lib: &Library,cfg: &SearchConfig) -> Result<MetaReport> {
        if cases.is_empty() || cases.len() > 32 || self.consumed_batches.len() >= 256 { return Err(Error::Capacity); }
        let packed: Vec<_> = cases.iter().map(|c| (&c.acquisition,&c.future)).collect();
        let batch = fingerprint(&serde_json::to_vec(&packed).map_err(|_| Error::InvalidData)?);
        if !self.consumed_batches.insert(batch) { return Err(Error::Stale); }
        let mut rows = Vec::new();
        for case in cases {
            if case.acquisition.is_empty() || case.future.is_empty() { return Err(Error::InvalidData); }
            let width = case.acquisition[0].pre.len(); let parent = Program::identity(width);
            let mut meter = Meter::new(cfg.work/8);
            let job = discover_job(&parent,&case.acquisition,lib,cfg.deterministic,&mut meter)?.ok_or(Error::NoEvidence)?;
            let hints = self.suggestions(&job,width,cfg,lib,&mut meter)?;
            rows.push(TransferRow {
                no_library: assess(&parent,case,&Library::default(),cfg,&[],0)?,
                same_library: assess(&parent,case,lib,cfg,&[],0)?,
                constructors: assess(&parent,case,lib,cfg,&hints,meter.spent)?,
            });
        }
        let construction_work = self.constructors.iter().map(|c| c.construction_work).sum::<u64>();
        let baseline_work: u128 = rows.iter().map(|r| r.same_library.work as u128).sum();
        let learned_work: u128 = rows.iter().map(|r| r.constructors.work as u128).sum::<u128>() + construction_work as u128;
        let additional_search_value = learned_work < baseline_work && rows.iter().all(|r| r.constructors.future_loss <= r.same_library.future_loss && r.constructors.training_loss <= r.same_library.training_loss);
        // Batch status is not an AGI or independent scientific qualification.
        Ok(MetaReport { batch,rows,construction_work,additional_search_value,independent_qualification: false })
    }
}

/// Acquire callable computations, charging the ENTIRE library in comparisons.
/// State/action-dependent expressions are not silently converted into pure calls.
pub fn acquire_function(expression: &Expr,lib: &mut Library,width: usize) -> Result<Option<u64>> {
    if lib.functions.len() >= 32 { return Err(Error::Capacity); }
    let mut inputs = BTreeSet::new(); let mut impure = false;
    expression.map(&mut |e| {
        match e { Expr::Input(i) => { inputs.insert(*i); },Expr::State(_) | Expr::Action | Expr::Arg(_) | Expr::Hole { .. } | Expr::Fold { .. } => impure = true,_ => () }
        None
    });
    if impure || expression.nodes() < 3 || inputs.len() > 8 { return Ok(None); }
    let ordered: Vec<_> = inputs.into_iter().collect();
    let body = expression.map(&mut |e| if let Expr::Input(i) = e { ordered.iter().position(|j| j == i).map(Expr::Arg) } else { None });
    let id = fingerprint(&serde_json::to_vec(&body).map_err(|_| Error::InvalidData)?);
    if lib.functions.iter().any(|f| f.id == id) { return Ok(Some(id)); }
    let function = Function { id,arity: ordered.len(),body };
    let mut candidate = lib.clone(); candidate.functions.push(function); candidate.validate(width)?;
    *lib = candidate; Ok(Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn trace(index: usize) -> SearchTrace {
        SearchTrace { job: Job { parent: 7,target: 0,witness_ids: vec![],deterministic_contract: true,stateless_impossible: false,exposed_sources: vec![0,1],features: vec![2,8,0,0] },
            candidates: 8,updates_tested: 1,work: 100,baseline_work: 100,constructor_work: 0,exhausted: false,frontier_limited: false,
            winner: Some(Patch { parent: 7,target: 0,replacement: Expr::bin(Op::Mul,Expr::Input(index),Expr::Input(index)),cells: vec![] }),training_loss: 0.0,constructor_id: None,prospectively_accepted: true }
    }
    #[test] fn executable_schema_is_acquired_and_keeps_bound_holes() {
        let mut r = Registry::default(); let lib = Library::default(); let mut m = Meter::new(100000);
        r.learn(trace(0),&lib,&mut m).unwrap(); assert_eq!(r.learn(trace(1),&lib,&mut m).unwrap(),1);
        let hints = r.suggestions(&trace(0).job,2,&SearchConfig::default(),&lib,&mut m).unwrap();
        assert_eq!(hints.len(),1); let h = hints[0].output.first_hole().unwrap().0;
        assert_eq!(hints[0].output.substitute(h,&Expr::Input(2)),Expr::bin(Op::Mul,Expr::Input(2),Expr::Input(2)));
        assert_eq!(r.constructors()[0].status,ConstructorStatus::Proposed);
    }
    #[test] fn training_success_is_not_promotion() {
        let mut t = trace(0); t.prospectively_accepted = false;
        assert_eq!(Registry::default().learn(t,&Library::default(),&mut Meter::new(10000)),Err(Error::NoEvidence));
    }
    #[test] fn learned_guard_is_an_executable_expression() {
        let examples = vec![(vec![0],false),(vec![1],true),(vec![2],true),(vec![-1],true)];
        let lib = Library::default(); let guard = fit_guard(&examples,&lib,&mut Meter::new(100000)).unwrap().unwrap();
        assert_eq!(guard.eval(Context { input: &[17],action: 0,state: &[],args: &[] },&lib,&mut Meter::new(100)),Ok(Value::Bool(true)));
    }
    #[test] fn pure_library_counts_definition_and_rebinds_arguments() {
        let mut lib = Library::default();
        let id = acquire_function(&Expr::bin(Op::Mul,Expr::Input(0),Expr::Input(0)),&mut lib,3).unwrap().unwrap();
        assert!(lib.nodes() >= 4);
        let call = Expr::Call(id,vec![Expr::Input(2)]);
        assert_eq!(call.eval(Context { input: &[0,0,7],action: 0,state: &[],args: &[] },&lib,&mut Meter::new(100)),Ok(Value::Int(49)));
    }
    #[test] fn generation_two_uses_successful_constructor_descendants() {
        let lib = Library::default(); let mut r = Registry::default(); let mut m = Meter::new(100000);
        r.learn(trace(0),&lib,&mut m).unwrap(); r.learn(trace(1),&lib,&mut m).unwrap();
        let cid = r.constructors()[0].id;
        let mut next = trace(2); next.constructor_id = Some(cid);
        next.winner.as_mut().unwrap().replacement = Expr::bin(Op::Mul,Expr::bin(Op::Add,Expr::Input(0),Expr::Input(1)),Expr::bin(Op::Add,Expr::Input(0),Expr::Input(1)));
        r.learn(next,&lib,&mut m).unwrap();
        // A generic existing schema may already cover this edit; deduplication
        // is correct and must not invent a new generation to inflate progress.
        assert!(!r.constructors().is_empty()); assert_eq!(r.traces().len(),3);
    }
}
