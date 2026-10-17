//! Prospective recursive improvement of executable search schemas.
//!
//! A refinement is a program that substitutes typed syntax into the holes of an
//! existing constructor. It changes what the SEARCH enumerates, not merely an
//! answer. Its accepted descendants may induce the next refinement through the
//! same Registry::learn path. The interpreter, evidence gate and action authority
//! are never writable by a refinement. This is bounded schema metaprogramming,
//! not arbitrary native-code self-modification or a claim of general intelligence.
use super::constructors::{ConstructorStatus, Registry};
use super::synthesis::*;
use super::vm::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refinement {
    pub parent: u64,
    /// Parallel substitution avoids capture when the child has its own holes.
    pub bindings: BTreeMap<u16, Expr>,
}

pub fn instantiate(pattern: &Expr, bindings: &BTreeMap<u16, Expr>) -> Expr {
    pattern.map(&mut |e| match e {
        Expr::Hole { id, .. } => bindings.get(id).cloned(),
        _ => None,
    })
}

/// Match syntax, preserving repeated-hole equality and types. An empirical
/// equality of outputs does not establish generator ancestry.
pub fn match_schema(pattern: &Expr, value: &Expr, lib: &Library) -> Result<Option<BTreeMap<u16, Expr>>> {
    // The explicit traversal below also supports call and fold syntax. Keep
    // library calls as syntax; do not silently inline them to manufacture lineage.
    fn go(p: &Expr,v: &Expr,lib: &Library,b: &mut BTreeMap<u16,Expr>,depth: usize) -> Result<bool> {
        if depth > 48 { return Err(Error::Capacity); }
        if let Expr::Hole { id,ty } = p {
            if v.ty(lib)? != *ty { return Ok(false); }
            if let Some(old) = b.get(id) { return Ok(old == v); }
            b.insert(*id,v.clone()); return Ok(true);
        }
        Ok(match (p,v) {
            (Expr::Bin(op,a,c),Expr::Bin(oq,x,z)) if op == oq => go(a,x,lib,b,depth+1)? && go(c,z,lib,b,depth+1)?,
            (Expr::Not(a),Expr::Not(x)) | (Expr::AsInt(a),Expr::AsInt(x)) => go(a,x,lib,b,depth+1)?,
            (Expr::If(a,c,d),Expr::If(x,z,w)) => go(a,x,lib,b,depth+1)? && go(c,z,lib,b,depth+1)? && go(d,w,lib,b,depth+1)?,
            (Expr::Call(id,args),Expr::Call(jd,values)) if id == jd && args.len() == values.len() => {
                let mut valid = true;
                for (a,x) in args.iter().zip(values) { if !go(a,x,lib,b,depth+1)? { valid = false; break; } }
                valid
            }
            (Expr::Fold { count:a,seed:c,body:d },Expr::Fold { count:x,seed:z,body:w }) => go(a,x,lib,b,depth+1)? && go(c,z,lib,b,depth+1)? && go(d,w,lib,b,depth+1)?,
            _ => p == v,
        })
    }
    let mut bindings = BTreeMap::new();
    Ok(if go(pattern,value,lib,&mut bindings,0)? { Some(bindings) } else { None })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMeasurement {
    pub work: u64,
    pub training_loss: f64,
    pub future_loss: f64,
    pub program_nodes: usize,
    pub library_nodes: usize,
    pub program_fingerprint: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveVerdict {
    pub ticket: u64,
    pub constructor: Option<u64>,
    pub generation: u32,
    pub parent_constructor: Option<u64>,
    pub candidate: SearchMeasurement,
    pub same_library: SearchMeasurement,
    /// All the same library and previous constructors, except the selected
    /// winner and its descendants. This is stricter than comparing to no skills.
    pub without_winner_subtree: SearchMeasurement,
    pub model_accepted: bool,
    pub quality_retained: bool,
    pub additional_search_value: bool,
    pub recursive_search_value: bool,
    pub constructors_acquired: usize,
    pub full_construction_work: u64,
    pub check_work: u64,
    pub independent_qualification: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaLedger {
    pub construction_work: u64,
    /// Actual computation includes the counterfactual arms. Those arms are not
    /// free online work and are not disguised as external interactions.
    pub search_work: u64,
    pub check_work: u64,
    pub abandoned: u64,
    pub(super) pending: Option<u64>,
    consumed: BTreeSet<u64>,
    reports: Vec<RecursiveVerdict>,
}
impl MetaLedger {
    pub fn is_empty(&self) -> bool {
        self.construction_work == 0 && self.search_work == 0 && self.check_work == 0
            && self.abandoned == 0 && self.pending.is_none() && self.consumed.is_empty() && self.reports.is_empty()
    }
    pub fn reports(&self) -> &[RecursiveVerdict] { &self.reports }
    pub fn validate(&self) -> Result<()> {
        if self.consumed.len() > 256 || self.reports.len() > 128 { return Err(Error::Capacity); }
        if self.pending.is_some_and(|id| !self.consumed.contains(&id)) { return Err(Error::InvalidData); }
        let mut ids = BTreeSet::new();
        for r in &self.reports {
            if !ids.insert(r.ticket) || !self.consumed.contains(&r.ticket) || r.independent_qualification { return Err(Error::InvalidData); }
            for a in [&r.candidate,&r.same_library,&r.without_winner_subtree] {
                if !a.training_loss.is_finite() || !a.future_loss.is_finite() || a.training_loss < 0.0 || a.future_loss < 0.0 { return Err(Error::InvalidData); }
            }
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct FrozenArm { trace: SearchTrace, program: Program, work: u64 }

/// Fields are private and the live registry owns a single-use commitment. The
/// ticket is serialized only for cognitive checkpoints, not external authority.
/// Neither future labels nor an evaluator callback exist in this object.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTrial {
    ticket: u64, registry_stamp: u64,
    parent: Program, acquisition: Vec<Transition>, library: Library,
    candidate: FrozenArm, baseline: FrozenArm, ancestor: Option<FrozenArm>,
    generation: u32, parent_constructor: Option<u64>, learning_budget: u64,
}
impl SearchTrial {
    pub(super) fn training_len(&self) -> usize { self.acquisition.len() }
    pub fn trace(&self) -> &SearchTrace { &self.candidate.trace }
    pub fn program(&self) -> &Program { &self.candidate.program }
    pub fn fingerprint(&self) -> u64 { self.ticket }
    pub fn frozen_search_work(&self) -> (u64,u64,u64) {
        (self.candidate.work,self.baseline.work,self.ancestor.as_ref().unwrap_or(&self.baseline).work)
    }
    pub fn validate(&self) -> Result<()> {
        let width = self.parent.outputs.len();
        self.library.validate(width)?; self.parent.validate(width,&self.library)?;
        validate_data(&self.acquisition,width,32)?;
        for a in [&self.candidate,&self.baseline].into_iter().chain(self.ancestor.iter()) {
            a.program.validate(width,&self.library)?;
            if !a.trace.training_loss.is_finite() || a.trace.prospectively_accepted { return Err(Error::InvalidData); }
        }
        if self.learning_budget > 100_000_000 { return Err(Error::Capacity); }
        Ok(())
    }
}

fn run_arm(parent: &Program,data: &[Transition],lib: &Library,cfg: &SearchConfig,registry: Option<&Registry>) -> Result<FrozenArm> {
    let mut meter = Meter::new(cfg.work/8);
    let hints = if let Some(registry) = registry {
        let result = (|| {
            let job = discover_job(parent,data,lib,cfg.deterministic,&mut meter)?.ok_or(Error::NoEvidence)?;
            registry.suggestions(&job,parent.outputs.len(),cfg,lib,&mut meter)
        })();
        match result { Ok(h) => h,Err(Error::Budget) => vec![],Err(e) => return Err(e) }
    } else { vec![] };
    let mut adjusted = cfg.clone(); adjusted.work = cfg.work.checked_sub(meter.spent).ok_or(Error::Budget)?;
    if adjusted.work < 100 { return Err(Error::Budget); }
    let trace = synthesize(parent,data,lib,&adjusted,&hints)?;
    let program = trace.winner.as_ref().map(|p| p.apply(parent,lib)).transpose()?.unwrap_or_else(|| parent.clone());
    let work = trace.work+meter.spent;
    Ok(FrozenArm { trace,program,work })
}
fn semantic_data_id(parent: &Program,data: &[Transition],lib: &Library) -> Result<u64> {
    // Relabelling row IDs must not reopen the same acquired problem for another
    // attempt against different future labels. IDs are still validated separately.
    let rows: Vec<_> = data.iter().map(|t| (t.restart,&t.pre,t.action,&t.post)).collect();
    Ok(fingerprint(&serde_json::to_vec(&(parent,&rows,lib)).map_err(|_| Error::InvalidData)?))
}
fn registry_stamp(registry: &Registry) -> Result<u64> {
    let mut snapshot = registry.clone(); snapshot.meta.pending = None;
    Ok(fingerprint(&serde_json::to_vec(&snapshot).map_err(|_| Error::InvalidData)?))
}
fn measure(arm: &FrozenArm,all: &[Transition],split: usize,lib: &Library,meter: &mut Meter) -> Result<(SearchMeasurement,Vec<Vec<i64>>)> {
    let start = meter.spent;
    let predictions = replay(&arm.program,all,lib,meter)?.0;
    let result = SearchMeasurement {
        work: arm.work+meter.spent-start,
        training_loss: absolute_loss(&predictions[..split],&all[..split])?,
        future_loss: absolute_loss(&predictions[split..],&all[split..])?,
        program_nodes: arm.program.nodes(),library_nodes: lib.nodes(),program_fingerprint: arm.program.fingerprint(),
    };
    Ok((result,predictions))
}
fn no_worse(a: &[Vec<i64>],b: &[Vec<i64>],data: &[Transition]) -> bool {
    a.iter().zip(b).zip(data).all(|((a,b),t)| {
        let loss = |row: &[i64]| row.iter().zip(&t.post).map(|(&x,&y)| (x as i128-y as i128).abs()).sum::<i128>();
        loss(a) <= loss(b)
    })
}
impl Registry {
    pub fn meta_ledger(&self) -> &MetaLedger { &self.meta }
    fn descendant_of(&self,mut id: u64,ancestor: u64) -> bool {
        for _ in 0..=self.constructors.len() {
            if id == ancestor { return true; }
            match self.constructors.iter().find(|c| c.id == id).and_then(|c| c.refinement.as_ref()) {
                Some(edit) => id = edit.parent,
                None => return false,
            }
        }
        false
    }
    /// Freeze every search arm BEFORE the environment supplies a validation
    /// transition. Same observations, parent, library, grammar and budget.
    pub fn begin_trial(&mut self,parent: &Program,acquisition: &[Transition],lib: &Library,cfg: &SearchConfig) -> Result<SearchTrial> {
        if self.meta.pending.is_some() { return Err(Error::Busy); }
        if self.meta.consumed.len() >= 256 || self.meta.reports.len() >= 128 { return Err(Error::Capacity); }
        self.validate(lib)?; cfg.validate()?; parent.validate(parent.outputs.len(),lib)?;
        validate_data(acquisition,parent.outputs.len(),32)?;
        let mut preflight = Meter::new(cfg.work/8);
        let discovery = discover_job(parent,acquisition,lib,cfg.deterministic,&mut preflight);
        self.meta.search_work = self.meta.search_work.saturating_add(preflight.spent);
        if discovery?.is_none() { return Err(Error::NoEvidence); }
        let ticket = semantic_data_id(parent,acquisition,lib)?;
        if !self.meta.consumed.insert(ticket) { return Err(Error::Stale); }
        let result: Result<SearchTrial> = (|| {
            let baseline = run_arm(parent,acquisition,lib,cfg,None)?;
            self.meta.search_work = self.meta.search_work.saturating_add(baseline.work);
            let candidate = run_arm(parent,acquisition,lib,cfg,Some(self))?;
            self.meta.search_work = self.meta.search_work.saturating_add(candidate.work);
            let selected = candidate.trace.constructor_id.and_then(|id| self.constructors.iter().find(|c| c.id == id));
            let generation = selected.map(|c| c.generation).unwrap_or(0);
            let parent_constructor = selected.and_then(|c| c.refinement.as_ref()).map(|e| e.parent);
            let ancestor = if let Some(cid) = candidate.trace.constructor_id {
                if parent_constructor.is_some() {
                    let mut without = self.clone();
                    without.constructors.retain(|c| !self.descendant_of(c.id,cid));
                    let arm = run_arm(parent,acquisition,lib,cfg,Some(&without))?;
                    self.meta.search_work = self.meta.search_work.saturating_add(arm.work);
                    Some(arm)
                } else { None }
            } else { None };
            let trial = SearchTrial { ticket,registry_stamp: registry_stamp(self)?,parent: parent.clone(),acquisition: acquisition.to_vec(),library: lib.clone(),candidate,baseline,ancestor,generation,parent_constructor,learning_budget: cfg.work/4 };
            trial.validate()?;
            Ok(trial)
        })();
        match result {
            Ok(trial) => { self.meta.pending = Some(ticket); Ok(trial) },
            Err(e) => {
                self.meta.abandoned = self.meta.abandoned.saturating_add(1);
                // A failed arm cannot give free unrecorded search; conservative
                // upper-bound accounting is used when no completed trace exists.
                self.meta.search_work = self.meta.search_work.saturating_add(cfg.work);
                Err(e)
            }
        }
    }
    pub fn cancel_trial(&mut self,trial: SearchTrial) -> Result<()> {
        if self.meta.pending != Some(trial.ticket) { return Err(Error::Stale); }
        self.meta.pending = None; self.meta.abandoned = self.meta.abandoned.saturating_add(1); Ok(())
    }
    /// The caller may veto installation, never force acceptance. Even a true
    /// allow_model cannot bypass frozen prediction, novelty or retention checks.
    pub fn complete_trial(&mut self,trial: SearchTrial,future: &[Transition],allow_model: bool) -> Result<RecursiveVerdict> {
        if self.meta.pending != Some(trial.ticket) { return Err(Error::Stale); }
        self.meta.pending = None; // consumed even if malformed validation fails
        let result = self.complete_inner(trial,future,allow_model);
        if result.is_err() { self.meta.abandoned = self.meta.abandoned.saturating_add(1); }
        result
    }
    fn complete_inner(&mut self,mut trial: SearchTrial,future: &[Transition],allow_model: bool) -> Result<RecursiveVerdict> {
        if registry_stamp(self)? != trial.registry_stamp { return Err(Error::Stale); }
        trial.validate()?;
        if future.len() < 2 { return Err(Error::NoEvidence); }
        let split = trial.acquisition.len();
        let mut all = trial.acquisition.clone(); all.extend_from_slice(future);
        validate_data(&all,trial.parent.outputs.len(),32)?;
        let mut meter = Meter::new(100_000_000);
        let checked: Result<_> = (|| {
            let (candidate,cp) = measure(&trial.candidate,&all,split,&trial.library,&mut meter)?;
            let (same_library,bp) = measure(&trial.baseline,&all,split,&trial.library,&mut meter)?;
            let (without_winner_subtree,ap) = measure(trial.ancestor.as_ref().unwrap_or(&trial.baseline),&all,split,&trial.library,&mut meter)?;
            let old = replay(&trial.parent,&all,&trial.library,&mut meter)?.0;
            let model_accepted = allow_model && trial.candidate.trace.winner.is_some()
                && no_worse(&cp[..split],&old[..split],&all[..split])
                && candidate.training_loss < absolute_loss(&old[..split],&all[..split])?
                && candidate.future_loss < absolute_loss(&old[split..],future)?;
            let quality_retained = no_worse(&cp,&bp,&all) && no_worse(&cp,&ap,&all);
            Ok((candidate,same_library,without_winner_subtree,model_accepted,quality_retained))
        })();
        self.meta.check_work = self.meta.check_work.saturating_add(meter.spent);
        let (candidate,same_library,without_winner_subtree,model_accepted,quality_retained) = checked?;
        let cid = trial.candidate.trace.constructor_id;
        let mut verdict = RecursiveVerdict { ticket: trial.ticket,constructor: cid,generation: trial.generation,parent_constructor: trial.parent_constructor,candidate,same_library,without_winner_subtree,model_accepted,quality_retained,additional_search_value: false,recursive_search_value: false,constructors_acquired: 0,full_construction_work: self.meta.construction_work,check_work: meter.spent,independent_qualification: false };
        if model_accepted {
            trial.candidate.trace.prospectively_accepted = true;
            let mut learning = Meter::new(trial.learning_budget.max(100));
            // Budget exhaustion is a reported absence of a new constructor,
            // never grounds for inventing a schema or a successful revision.
            match self.learn(trial.candidate.trace,&trial.library,&mut learning) {
                Ok(n) => verdict.constructors_acquired = n,
                Err(Error::Budget) | Err(Error::Capacity) => (),
                Err(e) => return Err(e),
            }
        }
        verdict.full_construction_work = self.meta.construction_work;
        if let Some(id) = cid {
            let history = self.meta.reports.iter().filter(|r| r.constructor == Some(id)).chain(std::iter::once(&verdict));
            let mut count = 0usize; let mut good = true; let mut saving = 0i128; let mut parent_saving = 0i128;
            for r in history {
                count += 1; good &= r.quality_retained && r.model_accepted;
                saving += r.same_library.work as i128-r.candidate.work as i128;
                parent_saving += r.without_winner_subtree.work as i128-r.candidate.work as i128;
            }
            let debt = self.meta.construction_work as i128;
            verdict.additional_search_value = count >= 2 && good && saving > debt;
            verdict.recursive_search_value = verdict.additional_search_value && trial.parent_constructor.is_some() && parent_saving > debt;
            if let Some(c) = self.constructors.iter_mut().find(|c| c.id == id) {
                c.status = development_status(quality_retained,verdict.additional_search_value,
                    verdict.recursive_search_value,c.refinement.is_some());
            }
        }
        self.meta.reports.push(verdict.clone());
        Ok(verdict)
    }
}

/// Evidence status is reversible: an earlier speedup is not a permanent title.
fn development_status(quality: bool,additional: bool,recursive: bool,has_parent: bool) -> ConstructorStatus {
    if !quality { ConstructorStatus::RejectedOnDevelopmentBatch }
    else if additional && (!has_parent || recursive) { ConstructorStatus::UsefulOnDevelopmentBatch }
    else { ConstructorStatus::Proposed }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn economic_regression_revokes_previous_usefulness() {
        assert_eq!(development_status(true,true,true,true),ConstructorStatus::UsefulOnDevelopmentBatch);
        assert_eq!(development_status(true,true,false,true),ConstructorStatus::Proposed);
        assert_eq!(development_status(true,false,false,false),ConstructorStatus::Proposed);
        assert_eq!(development_status(false,true,true,true),ConstructorStatus::RejectedOnDevelopmentBatch);
    }
}
