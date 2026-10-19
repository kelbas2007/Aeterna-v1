//! Persistent, opt-in construction runtime using EvoPhase's sealed permission
//! boundary. Authored software synthesis is NOT physical SNN-owned computation.
use super::constructors::*;
use super::autonomy::{ActionOffer,AutonomyPolicy,AutonomyReport};
use super::synthesis::*;
use super::vm::*;
use super::recursive::SearchTrial;
use super::curriculum::{Curriculum, SelectionMode};
use super::epistemic::{EvidencePolicy, EvidenceReport, EvidenceTrial, RiskLedger};
use crate::{EvoPhase, HumanProtectionDecision, HumanProtectionEvidence};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub width: usize, pub actions: usize, pub search: SearchConfig,
    pub minimum_training: usize, pub search_every: usize, pub shadow_steps: usize,
    pub journal_capacity: usize, pub planning_depth: usize, pub planning_work: u64,
}
impl Default for Config {
    fn default() -> Self { Self { width: 1,actions: 2,search: SearchConfig::default(),minimum_training: 8,search_every: 4,shadow_steps: 6,journal_capacity: 512,planning_depth: 3,planning_work: 20000 } }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        self.search.validate()?;
        if !(1..=32).contains(&self.width) || !(1..=16).contains(&self.actions)
            || !(2..=128).contains(&self.minimum_training) || self.search_every == 0
            || !(2..=128).contains(&self.shadow_steps) || self.journal_capacity < self.minimum_training+self.shadow_steps
            || self.journal_capacity > 4096 || !(1..=5).contains(&self.planning_depth)
            || !(100..=1000000).contains(&self.planning_work) { return Err(Error::Capacity); }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast { pub center: Vec<i64>, pub scale: Vec<f64> }
impl Forecast {
    /// Discrete Laplace likelihood; past-fitted scale stays frozen in a trial.
    pub fn nll(&self,post: &[i64]) -> Result<f64> {
        if post.len() != self.center.len() || self.scale.len() != post.len() { return Err(Error::InvalidData); }
        let mut nll = 0.0;
        for ((&mu,&scale),&actual) in self.center.iter().zip(&self.scale).zip(post) {
            if !scale.is_finite() || !(0.25..=1e9).contains(&scale) { return Err(Error::InvalidData); }
            let r = (-1.0/scale).exp();
            nll += ((actual as i128-mu as i128).abs() as f64)/scale + ((1.0+r)/(1.0-r)).ln();
        }
        Ok(nll)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Model { program: Program,state: Vec<i64>,scale: Vec<f64> }
#[derive(Debug, Serialize, Deserialize)]
struct Shadow {
    model: Model, trace: SearchTrace, training_len: usize, start_id: u64,
    samples: usize, incumbent_nll: f64,candidate_nll: f64,
    #[serde(default, skip_serializing_if="Option::is_none")]
    controls: Option<SearchTrial>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    evidence: Option<EvidenceTrial>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub transition: Transition, pub model: u64,pub prediction: Forecast,
    pub candidate: Option<u64>,pub candidate_prediction: Option<Forecast>,
    pub permit_sequence: u64,pub external_cost: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    pub first_id: u64,pub last_id: u64,pub parent: u64,pub candidate: u64,
    pub accepted: bool,pub retention_ok: bool,pub incumbent_nll: f64,pub candidate_nll: f64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Learner {
    config: Config,active: Model,previous: Option<Program>,shadow: Option<Shadow>,
    library: Library,registry: Registry,records: Vec<Record>,promotions: Vec<Promotion>,
    current: Vec<i64>,next_id: u64,pending: bool,poisoned: bool,restart_next: bool,
    last_trace: Option<SearchTrace>,construction_work: u64,last_error: Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    curriculum: Option<Curriculum>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    epistemic: Option<RiskLedger>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    autonomy: Option<AutonomyPolicy>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    frozen_at: Option<u64>,
    #[serde(default,skip_serializing_if="carrier_unmanaged")]
    carrier_managed: bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    requirements: Option<super::requirements::RequirementMemory>,
    /// Opt-in factual control; no extra teacher/model/independent checkpoint.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    self_directed: bool,
}
/// Non-cloneable and non-serializable PRE-action commitment.
pub(crate) struct Commitment {
    id: u64,pre: Vec<i64>,action: usize,model: u64,prediction: Forecast,next_state: Vec<i64>,
    candidate: Option<u64>,candidate_prediction: Option<Forecast>,candidate_state: Option<Vec<i64>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotReport {
    pub events: usize,pub active_program: u64,pub state_cells: usize,pub program_nodes: usize,
    pub library_nodes: usize,pub constructors: usize,pub promotions: usize,
    pub construction_work: u64,pub shadow_pending: bool,pub poisoned: bool,
    pub last_error: Option<String>,
    pub constructor_generation: u32,
    pub recursive_trials: usize,
    pub recursive_improvements: usize,
}
fn calibrate(program: &Program,data: &[Transition],lib: &Library,meter: &mut Meter) -> Result<(Vec<i64>,Vec<f64>)> {
    let (predicted,state) = replay(program,data,lib,meter)?;
    let mut errors = vec![0.0; program.outputs.len()];
    for (p,t) in predicted.iter().zip(data) {
        for (j,(&x,&y)) in p.iter().zip(&t.post).enumerate() { errors[j] += ((x as i128-y as i128).abs()) as f64; }
    }
    let scale = errors.into_iter().map(|e| (e/(data.len().max(1) as f64)).clamp(0.25,1e9)).collect();
    Ok((state,scale))
}
/// Deterministic model-independent exploration reserve. Only factual event count
/// enters this mixer, never an environment seed, task identity or hidden state.
fn exploration_word(event: u64) -> u64 {
    let mut z = event.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// A tiny bounded grammar fitted to *executed* transitions in Learner's journal.
/// The same facts, not a separately installed world model, are used after
/// restart. These are authored mathematical operators, not invented by SNN.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum ObservedEffect { Keep, Delta(i64), Set(i64) }
impl ObservedEffect {
    fn apply(self,x:i64)->Option<i64>{ match self { Self::Keep=>Some(x), Self::Delta(d)=>x.checked_add(d),Self::Set(v)=>Some(v) } }
}
// Explicitly metered factual fitting. The journal is bounded by the original
// runtime policy, but every row/axis/guard considered here ALSO consumes the
// same planning meter. No hidden O(journal_size * candidates) scans.
fn effect(rows:&[&Transition],axis:usize,meter:&mut Meter)->Option<ObservedEffect>{
    meter.charge((rows.len() as u64).saturating_mul(4)).ok()?;
    if rows.len()<2 {return None;}
    if rows.iter().all(|r|r.pre[axis]==r.post[axis]){return Some(ObservedEffect::Keep)}
    let difference=rows[0].post[axis] as i128 - rows[0].pre[axis] as i128;
    if let Ok(d)=i64::try_from(difference) {
        if rows.iter().all(|r|r.post[axis] as i128-r.pre[axis] as i128==difference){return Some(ObservedEffect::Delta(d))}
    }
    let output=rows[0].post[axis];
    if rows.iter().all(|r|r.post[axis]==output){return Some(ObservedEffect::Set(output))}
    None
}
fn predict_axis(rows:&[&Transition],axis:usize,current:&[i64],meter:&mut Meter)->Option<i64>{
    if let Some(rule)=effect(rows,axis,meter){return rule.apply(current[axis]);}
    // Test an independently contrasted joint effect before sweeping cheaper
    // but insufficient single-guard hypotheses: prospective re-fitting must
    // fit inside the SAME finite planning budget, not demand extra compute.
    if let Some(joint)=predict_conjunctive_axis(rows,axis,current,meter){return Some(joint)}
    for guard in 0..current.len() {
        if guard==axis{continue}
        // Upfront charge covers collection, sorting and de-duplication of at
        // most 96 numeric witnesses; exact CPU cycles are NOT claimed.
        meter.charge((rows.len() as u64).saturating_mul(9)).ok()?;
        let mut values:Vec<i64>=rows.iter().map(|r|r.pre[guard]).collect();
        values.sort_unstable();values.dedup();
        if values.len()>4 {continue;}
        for value in values {
            meter.charge((rows.len() as u64).saturating_mul(2)).ok()?;
            let yes=rows.iter().copied().filter(|r|r.pre[guard]==value).collect::<Vec<_>>();
            let no=rows.iter().copied().filter(|r|r.pre[guard]!=value).collect::<Vec<_>>();
            if yes.len()>=2 && !no.is_empty() && current[guard]==value {
                if let Some(rule)=effect(&yes,axis,meter) {
                    meter.charge(no.len() as u64).ok()?;
                    if no.iter().any(|r|rule.apply(r.pre[axis])!=Some(r.post[axis]))
                        && controlled_guard_contrast(&yes,&no,axis,guard,meter)? {
                        return rule.apply(current[axis]);
                    }
                }
            }
            if no.len()>=2 && !yes.is_empty() && current[guard]!=value {
                if let Some(rule)=effect(&no,axis,meter) {
                    meter.charge(yes.len() as u64).ok()?;
                    if yes.iter().any(|r|rule.apply(r.pre[axis])!=Some(r.post[axis]))
                        && controlled_guard_contrast(&no,&yes,axis,guard,meter)? {
                        return rule.apply(current[axis]);
                    }
                }
            }
        }
    }

    // An unidentifiable conditional effect is unknown; do not invent a
    // proposed consequence when no controlled explanation survives.
    None
}
/// Find an observed contrast in which the candidate guard changes but all
/// OTHER pre-action sensors (excluding the predicted output's own input)
/// remain equal. This rejects simple co-varying-sensor confounding.
/// Stop immediately if the existing computation budget is exhausted.
fn controlled_guard_contrast(positive:&[&Transition],negative:&[&Transition],
                             axis:usize,guard:usize,meter:&mut Meter)->Option<bool> {
    for &a in positive {
        for &b in negative {
            meter.charge(a.pre.len() as u64+1).ok()?;
            if a.pre.iter().zip(&b.pre).enumerate()
                .all(|(i,(x,y))|i==axis||i==guard||x==y) {return Some(true)}
        }
    }
    Some(false)
}
/// Discover a two-condition effect from real BEFORE/ACTION/AFTER samples.
/// No caller supplies an action meaning, prerequisite indices, thresholds or
/// a learned outcome. Both necessary conditions must have independent,
/// controlled negative witnesses and the positive examples must agree.
/// A rule learned from this prefix is *still untrusted* until the separate
/// chronological prospective validator accepts later real transitions.
fn predict_conjunctive_axis(rows:&[&Transition],axis:usize,current:&[i64],
                            meter:&mut Meter)->Option<i64> {
    let mut accepted:Option<i64>=None;
    for left in 0..current.len(){
        if left==axis{continue}
        for right in (left+1)..current.len(){
            if right==axis{continue}
            meter.charge((rows.len() as u64).saturating_mul(4).saturating_add(8)).ok()?;
            let mut both=Vec::new();
            let mut without_left=Vec::new();
            let mut without_right=Vec::new();
            for &row in rows {
                let l=row.pre[left]==current[left];
                let r=row.pre[right]==current[right];
                if l&&r {both.push(row)}
                else if !l&&r {without_left.push(row)}
                else if l&&!r {without_right.push(row)}
            }
            if both.len()<3||without_left.len()<2||without_right.len()<2 {continue}
            let Some(rule)=effect(&both,axis,meter) else {
                if meter.remaining()==0{return None}
                continue
            };
            let Some(predicted)=rule.apply(current[axis]) else {continue};
            if predicted==current[axis] {continue} // no uninformative no-op rule
            meter.charge((without_left.len()+without_right.len()) as u64).ok()?;
            if !without_left.iter().any(|r|rule.apply(r.pre[axis])!=Some(r.post[axis]))
                || !without_right.iter().any(|r|rule.apply(r.pre[axis])!=Some(r.post[axis])) {continue}
            if !controlled_guard_contrast(&both,&without_left,axis,left,meter)?
                || !controlled_guard_contrast(&both,&without_right,axis,right,meter)? {continue}
            if accepted.is_some_and(|other|other!=predicted){return None}
            accepted=Some(predicted);
        }
    }
    accepted
}
fn factual_prediction(records:&[Record],pre:&[i64],action:usize,
                      frozen_at:Option<u64>,meter:&mut Meter)->Option<Vec<i64>> {
    // A frozen Learner may still commit REAL external observations for safety
    // and accounting, but they must not become new training examples.
    let end=frozen_at.map_or(records.len(),|id|records.partition_point(|r|r.transition.id<id));
    meter.charge(16).ok()?; // upper bound on binary selection of <=4096 rows
    let mut observations=Vec::new();
    for r in records[..end].iter().rev().take(96) {
        meter.charge(1).ok()?;
        if r.transition.action==action {observations.push(&r.transition)}
    }
    if observations.len()<2 {return None;}
    (0..pre.len()).map(|axis|predict_axis(&observations,axis,pre,meter)).collect()
}
/// A conservative, prospective belief check inside the existing Learner.
/// The latest three REAL transitions of an action are evidence held out from
/// the older prefix used to infer the effect. A fact is offered to U1 only if
/// all three consequences were predictable, at least one actually changed the
/// environment, and the inferred rule did not change after those events.
/// This is not proof of causality or a learned SNN rule: ambiguous/contextual
/// effects are UNKNOWN, and U1 remains responsible for exploration.
fn calibrated_factual_prediction(records:&[Record],pre:&[i64],action:usize,
                                  frozen_at:Option<u64>,meter:&mut Meter)->Option<Vec<i64>> {
    let end=frozen_at.map_or(records.len(),|id|
        records.partition_point(|r|r.transition.id<id));
    // This partition and sample itself have bounded, charged cost; there is no
    // post-freeze leakage into prediction or the confidence assessment.
    meter.charge(16).ok()?;
    let mut seen=Vec::new();
    for (idx,r) in records[..end].iter().enumerate().rev().take(96) {
        meter.charge(1).ok()?;
        if r.transition.action==action {
            seen.push(idx);
            if seen.len()==32{break}
        }
    }
    // Five+ earlier training instances, three chronological holdouts.
    if seen.len()<8{return None}
    let held=&seen[..3];
    let cutoff=seen[2];
    let training=&records[..cutoff];
    let mut effect_seen=false;
    for &idx in held {
        let row=&records[idx].transition;
        let predicted=factual_prediction(training,&row.pre,action,None,meter)?;
        if predicted!=row.post {return None}
        effect_seen|=row.pre!=row.post;
    }
    if !effect_seen{return None}
    let predicted_from_training=factual_prediction(training,pre,action,None,meter)?;
    let predicted_from_all=factual_prediction(records,pre,action,frozen_at,meter)?;
    (predicted_from_training==predicted_from_all).then_some(predicted_from_all)
}
fn goal_distance(state:&[i64],goal:&[Option<i64>])->u128{
    state.iter().zip(goal).filter_map(|(&x,g)|g.map(|v|(i128::from(x)-i128::from(v)).unsigned_abs())).sum()
}
impl Learner {
    pub fn new(config: Config,initial: Vec<i64>) -> Result<Self> {
        config.validate()?;
        if initial.len() != config.width { return Err(Error::InvalidData); }
        Ok(Self { active: Model { program: Program::identity(config.width),state: vec![],scale: vec![1.0;config.width] },
            config,previous: None,shadow: None,library: Library::default(),registry: Registry::default(),records: vec![],promotions: vec![],
            current: initial,next_id: 0,pending: false,poisoned: false,restart_next: true,last_trace: None,construction_work: 0,last_error: None,curriculum: None,epistemic: None,autonomy: None,frozen_at: None,carrier_managed:false,requirements:None,self_directed:false })
    }
    /// Opt-in, no prefilled task list or preferred sensor indices.
    pub fn enable_curriculum(&mut self,mode: SelectionMode) -> Result<()> {
        if self.pending || self.shadow.is_some() || self.curriculum.is_some() { return Err(Error::Busy); }
        self.curriculum = Some(Curriculum::new(self.config.width,mode)?); Ok(())
    }
    pub fn curriculum(&self) -> Option<&Curriculum> { self.curriculum.as_ref() }
    /// Explicit lifetime policy, fixed before any observations. Cannot relax a
    /// deterministic contract or reset previously allocated statistical risk.
    pub fn enable_epistemic(&mut self,policy: EvidencePolicy) -> Result<()> {
        if self.pending || self.shadow.is_some() || self.epistemic.is_some() || !self.records.is_empty() || self.poisoned {
            return Err(Error::Busy);
        }
        if self.config.search.deterministic { return Err(Error::InvalidData); }
        self.epistemic = Some(RiskLedger::new(policy,self.config.width)?); Ok(())
    }
    pub fn epistemic_report(&self) -> Option<EvidenceReport> {
        self.epistemic.as_ref().map(|risk| risk.report(self.shadow.as_ref().and_then(|s| s.evidence.as_ref())))
    }
    /// Enable the entire bounded loop atomically before experience.
    pub fn enable_autonomy(&mut self,policy:AutonomyPolicy)->Result<()> {
        if self.pending || self.poisoned || self.shadow.is_some() || !self.records.is_empty()
            || self.curriculum.is_some() || self.epistemic.is_some() || self.autonomy.is_some()
            || self.frozen_at.is_some() {return Err(Error::Busy);}
        policy.validate(&self.config)?;
        let curriculum=Curriculum::new(self.config.width,SelectionMode::Fair)?;
        let risk=policy.evidence.as_ref().map(|e|RiskLedger::new(e.clone(),self.config.width)).transpose()?;
        self.curriculum=Some(curriculum);self.epistemic=risk;self.autonomy=Some(policy);Ok(())
    }
    pub fn autonomy_report(&self)->Option<AutonomyReport> {
        let policy=self.autonomy.as_ref()?;let c=self.curriculum.as_ref()?;
        let levels:Vec<_>=(0..self.config.width).map(|j|policy.level(c,j)).collect();
        let effective_limits=levels.iter().map(|&n|policy.limits(&self.config.search,n)).collect();
        let external_cost=self.records.iter().fold(0_u64,|s,r|s.saturating_add(r.external_cost));
        Some(AutonomyReport{levels,effective_limits,external_cost,remaining_cost:policy.max_external_cost.saturating_sub(external_cost),frozen_at:self.frozen_at,events:self.records.len()})
    }
    /// Irreversible freeze; facts and action costs continue, learning does not.
    pub fn freeze_learning(&mut self)->Result<()> {
        if self.pending || self.poisoned {return Err(Error::Busy);}
        if self.frozen_at.is_none(){self.cancel_shadow();self.frozen_at=Some(self.next_id);}
        self.freeze_recursive_search();if let Some(b)=&mut self.requirements{b.freeze();}Ok(())
    }
    pub fn program(&self) -> &Program { &self.active.program }
    pub fn library(&self) -> &Library { &self.library }
    pub fn registry(&self) -> &Registry { &self.registry }
    pub fn records(&self) -> &[Record] { &self.records }
    pub fn promotions(&self) -> &[Promotion] { &self.promotions }
    pub fn last_search(&self) -> Option<&SearchTrace> { self.last_trace.as_ref() }
    pub fn current(&self) -> &[i64] { &self.current }
    pub fn report(&self) -> SnapshotReport {
        SnapshotReport { events: self.records.len(),active_program: self.active.program.fingerprint(),state_cells: self.active.program.cells.len(),program_nodes: self.active.program.nodes(),library_nodes: self.library.nodes(),constructors: self.registry.constructors().len(),promotions: self.promotions.iter().filter(|p| p.accepted).count(),construction_work: self.construction_work,shadow_pending: self.shadow.is_some(),poisoned: self.poisoned,last_error: self.last_error.clone(),
            constructor_generation: self.registry.constructors().iter().map(|c| c.generation).max().unwrap_or(0),
            recursive_trials: self.registry.meta_ledger().reports().len(),
            recursive_improvements: self.registry.meta_ledger().reports().iter().filter(|r| r.recursive_search_value).count() }
    }
    fn data(&self) -> Vec<Transition> { self.records.iter().map(|r| r.transition.clone()).collect() }
    fn prepare(&mut self,action: usize) -> Result<Commitment> {
        if self.poisoned { return Err(Error::Execution("unknown external state after callback fault".into())); }
        if self.pending { return Err(Error::Busy); }
        if action >= self.config.actions { return Err(Error::Index); }
        if self.records.len() >= self.config.journal_capacity || self.next_id == u64::MAX { return Err(Error::Capacity); }
        if let (Some(risk),Some(shadow)) = (&self.epistemic,&self.shadow) {
            let bound = shadow.evidence.as_ref().ok_or(Error::Stale).and_then(|e|
                e.binding(risk,&self.active.program,&shadow.model.program,&self.library,shadow.trace.job.target,shadow.start_id));
            if let Err(e) = bound { self.last_error=Some(format!("epistemic binding: {e}")); self.cancel_shadow(); }
        }
        let mut meter = Meter::new(100000);
        let (center,next_state) = self.active.program.step(&self.current,action,&self.active.state,&self.library,&mut meter)?;
        let mut candidate = None; let mut candidate_prediction = None; let mut candidate_state = None;
        if let Some(shadow) = &self.shadow {
            match shadow.model.program.step(&self.current,action,&shadow.model.state,&self.library,&mut meter) {
                Ok((p,s)) => { candidate = Some(shadow.model.program.fingerprint()); candidate_prediction = Some(Forecast { center: p,scale: shadow.model.scale.clone() }); candidate_state = Some(s); },
                Err(e) => { self.last_error = Some(format!("shadow execution rejected: {e}")); self.cancel_shadow(); }
            }
        }
        self.pending = true;
        Ok(Commitment { id: self.next_id,pre: self.current.clone(),action,model: self.active.program.fingerprint(),
            prediction: Forecast { center,scale: self.active.scale.clone() },next_state,candidate,candidate_prediction,candidate_state })
    }
    fn cancel(&mut self,token: Commitment) -> Result<()> {
        if !self.pending || token.id != self.next_id { return Err(Error::Stale); }
        self.pending = false; Ok(())
    }
    fn fault(&mut self,token: Commitment,message: String) {
        let _ = self.cancel(token); self.poisoned = true; self.last_error = Some(message);
    }
    fn commit(&mut self,token: Commitment,post: Vec<i64>,permit_sequence: u64,external_cost: u64) -> Result<()> {
        if !self.pending || token.id != self.next_id || token.pre != self.current || token.model != self.active.program.fingerprint() { return Err(Error::Stale); }
        if post.len() != self.config.width {
            self.fault(token,"invalid POST after possible external execution".into()); return Err(Error::InvalidData);
        }
        let incumbent_nll = token.prediction.nll(&post)?;
        let candidate_nll = token.candidate_prediction.as_ref().map(|p| p.nll(&post)).transpose()?;
        let mut evidence_error = None;
        self.active.state = token.next_state;
        if let (Some(shadow),Some(nll),Some(state)) = (&mut self.shadow,candidate_nll,token.candidate_state) {
            if token.candidate != Some(shadow.model.program.fingerprint()) || token.id < shadow.start_id { return Err(Error::Stale); }
            if let Some(e) = &mut shadow.evidence {
                let update = token.candidate_prediction.as_ref().ok_or(Error::Stale)
                    .and_then(|p| e.observe(token.id,&token.prediction.center,&p.center,&post));
                if let Err(error) = update { evidence_error = Some(error); }
            }
            shadow.model.state = state; shadow.samples += 1; shadow.incumbent_nll += incumbent_nll; shadow.candidate_nll += nll;
        }
        let transition = Transition { id: token.id,restart: self.restart_next,pre: token.pre,action: token.action,post: post.clone() };
        self.records.push(Record { transition,model: token.model,prediction: token.prediction,candidate: token.candidate,candidate_prediction: token.candidate_prediction,permit_sequence,external_cost });
        self.current = post; self.next_id += 1; self.restart_next = false; self.pending = false;
        // A costly contract violation stays in history, never promotes a model.
        if self.poisoned {self.cancel_shadow();return Ok(());}
        if let Some(error) = evidence_error {
            self.last_error=Some(format!("epistemic observation: {error}")); self.cancel_shadow();
        }
        if self.shadow.as_ref().is_some_and(|s| s.evidence.as_ref().map_or(s.samples >= self.config.shadow_steps,|e| e.ready())) {
            if let Err(e) = self.finish_shadow() { self.last_error = Some(format!("validation: {e}")); self.cancel_shadow(); }
        }
        if !self.carrier_managed && self.frozen_at.is_none() && self.shadow.is_none() && self.records.len() >= self.config.minimum_training && self.records.len() % self.config.search_every == 0 {
            if let Err(e) = self.construct() {
                if e != Error::NoEvidence { self.last_error = Some(format!("construction: {e}")); }
            }
        }
        Ok(())
    }
    /// Synchronous learning; no external action or synthetic factual record.
    pub fn construct(&mut self) -> Result<()> {
        let result = self.construct_inner();
        if self.shadow.is_none() {
            if let Some(c) = &mut self.curriculum { c.finish(false,0.0,self.construction_work)?; }
        }
        result
    }
    fn construct_inner(&mut self) -> Result<()> {
        if self.pending || self.shadow.is_some() { return Err(Error::Busy); }
        if self.frozen_at.is_some() || self.poisoned || self.records.len() < self.config.minimum_training { return Err(Error::NoEvidence); }
        let data = self.data();
        let requested = if let Some(c) = &mut self.curriculum {
            let mut meter = Meter::new(2_000_000);
            let discovery = (|| -> Result<_> {
                let (predictions,_) = replay(&self.active.program,&data,&self.library,&mut meter)?;
                let mut errors = vec![0.0; self.config.width];
                for (p,t) in predictions.iter().zip(&data) {
                    for j in 0..errors.len() { meter.charge(1)?; errors[j] += ((p[j] as i128-t.post[j] as i128).abs()) as f64; }
                }
                if let Some(risk)=&self.epistemic {
                    for j in risk.report(None).policy.protected_outputs {errors[j]=0.0;}
                }
                Ok(errors)
            })();
            self.construction_work = self.construction_work.saturating_add(meter.spent);
            c.select(&discovery?,self.next_id,self.construction_work)?
        } else { None };
        if self.curriculum.is_some() && requested.is_none() { return Err(Error::NoEvidence); }
        let search=match (&self.autonomy,&self.curriculum,requested) {
            (Some(policy),Some(c),Some(j))=>policy.limits(&self.config.search,policy.level(c,j)),
            _=>self.config.search.clone(),
        };
        let before = self.registry.meta_ledger().search_work;
        let controls_result = self.registry.begin_requested_trial(&self.active.program,&data,&self.library,&search,requested);
        self.construction_work += self.registry.meta_ledger().search_work.saturating_sub(before);
        let controls = controls_result?;
        let trace = controls.trace().clone();
        self.last_trace = Some(trace.clone());
        if trace.winner.is_none() || controls.program() == &self.active.program {
            self.registry.cancel_trial(controls)?; return Ok(());
        }
        let program = controls.program().clone();
        let mut meter = Meter::new(1000000);
        let calibration = (|| -> Result<_> {
            let (state,scale) = calibrate(&program,&data,&self.library,&mut meter)?;
            let (_,old_scale) = calibrate(&self.active.program,&data,&self.library,&mut meter)?;
            Ok((state,scale,old_scale))
        })();
        let (state,scale,incumbent_scale) = match calibration {
            Ok(value) => value,
            Err(e) => { self.construction_work += meter.spent; self.registry.cancel_trial(controls)?; return Err(e); }
        };
        self.construction_work += meter.spent;
        let evidence = if let Some(risk) = &mut self.epistemic {
            match risk.allocate(&self.active.program,&program,&self.library,trace.job.target,self.next_id,
                                self.config.journal_capacity-self.records.len()) {
                Ok(e) => Some(e),
                Err(e) => { self.registry.cancel_trial(controls)?; return Err(e); }
            }
        } else { None };
        self.active.scale = incumbent_scale;
        self.shadow = Some(Shadow { model: Model { program,state,scale },trace,training_len: data.len(),start_id: self.next_id,samples: 0,incumbent_nll: 0.0,candidate_nll: 0.0,controls: Some(controls),evidence });
        Ok(())
    }
    fn finish_shadow(&mut self) -> Result<()> {
        let shadow = self.shadow.as_ref().ok_or(Error::NoEvidence)?;
        let data = self.data(); let training = &data[..shadow.training_len];
        let mut meter = Meter::new(2000000);
        let (old,_) = replay(&self.active.program,training,&self.library,&mut meter)?;
        let (new,_) = replay(&shadow.model.program,training,&self.library,&mut meter)?;
        let pointwise_ok = old.iter().zip(new.iter()).zip(training).all(|((a,b),t)| {
            let cost = |p: &[i64]| p.iter().zip(&t.post).map(|(&x,&y)| (x as i128-y as i128).abs()).sum::<i128>(); cost(b) <= cost(a)
        });
        let complexity = 0.01 * (shadow.model.program.nodes() as f64-self.active.program.nodes() as f64);
        let (retention_ok,local_acceptance) = if let Some(evidence) = &shadow.evidence {
            let risk = self.epistemic.as_ref().ok_or(Error::Stale)?;
            evidence.binding(risk,&self.active.program,&shadow.model.program,&self.library,shadow.trace.job.target,shadow.start_id)?;
            let retained = risk.footprint(&self.active.program,&shadow.model.program,shadow.trace.job.target);
            (retained,retained && evidence.accepted())
        } else {
            if self.epistemic.is_some() { return Err(Error::Stale); }
            (pointwise_ok,pointwise_ok && shadow.candidate_nll+complexity < shadow.incumbent_nll)
        };
        if self.promotions.len() >= 128 { return Err(Error::Capacity); }
        let mut shadow = self.shadow.take().ok_or(Error::NoEvidence)?;
        let mut accepted = false;
        if let Some(controls) = shadow.controls.take() {
            let ledger = self.registry.meta_ledger();
            let before = ledger.check_work.saturating_add(ledger.construction_work);
            let result = if let Some(evidence) = &shadow.evidence {
                match self.epistemic.as_ref() {
                    Some(risk) => self.registry.complete_trial_epistemic(controls,&data[shadow.training_len..],local_acceptance,evidence,risk),
                    None => { let _=self.registry.cancel_trial(controls); Err(Error::Stale) }
                }
            } else { self.registry.complete_trial(controls,&data[shadow.training_len..],local_acceptance) };
            let ledger = self.registry.meta_ledger();
            self.construction_work += ledger.check_work.saturating_add(ledger.construction_work).saturating_sub(before);
            // Local likelihood is a proposal, never authority to bypass the
            // prospective gate. An audit error must also fail closed.
            match result {
                Ok(verdict) => {
                    accepted = local_acceptance && verdict.model_accepted;
                    if local_acceptance && !verdict.model_accepted {
                        self.last_error = Some("recursive validation: model installation vetoed".into());
                    }
                },
                Err(e) => {
                    accepted = false;
                    self.last_error = Some(format!("recursive validation: {e}"));
                },
            }
        } else {
            // Old checkpoints may contain an unaudited pending shadow. Retain
            // their factual history, but never install it through a local-only
            // compatibility path. The next search creates a fresh audited trial.
            self.last_error = Some("recursive validation: missing frozen trial".into());
        }
        // Publish one FINAL decision only after every applicable gate. Rejected
        // trials cannot become successful promotions or accepted search traces.
        if let Some(c) = &mut self.curriculum {
            let target = shadow.trace.job.target;
            let mut before = 0.0; let mut after = 0.0;
            for r in &self.records[shadow.training_len..] {
                let y = r.transition.post[target];
                before += ((r.prediction.center[target] as i128-y as i128).abs()) as f64;
                if let Some(p) = &r.candidate_prediction { after += ((p.center[target] as i128-y as i128).abs()) as f64; }
                else { after = f64::INFINITY; }
            }
            let gain = if accepted { ((before-after)/(1.0+before)).clamp(0.0,1.0) } else { 0.0 };
            c.finish(accepted,gain,self.construction_work.saturating_add(meter.spent))?;
        }
        if let Some(e) = &shadow.evidence {
            if let Some(risk) = &mut self.epistemic { risk.record(e,accepted)?; }
            else { return Err(Error::Stale); }
        }
        shadow.trace.prospectively_accepted = accepted;
        self.last_trace = Some(shadow.trace.clone());
        self.promotions.push(Promotion { first_id: shadow.start_id,last_id: self.next_id-1,parent: self.active.program.fingerprint(),candidate: shadow.model.program.fingerprint(),accepted,retention_ok,incumbent_nll: shadow.incumbent_nll,candidate_nll: shadow.candidate_nll });
        if accepted {
            self.previous = Some(self.active.program.clone()); self.active = shadow.model;
            if let Some(patch) = &shadow.trace.winner {
                if let Err(e) = acquire_function(&patch.replacement,&mut self.library,self.config.width) { self.last_error = Some(format!("library: {e}")); }
            }
        }
        self.construction_work += meter.spent;
        Ok(())
    }
    fn cancel_shadow(&mut self) {
        if let Some(c) = &mut self.curriculum { let _ = c.finish(false,0.0,self.construction_work); }
        if let Some(mut shadow) = self.shadow.take() {
            if let (Some(risk),Some(e)) = (&mut self.epistemic,&shadow.evidence) { let _=risk.record(e,false); }
            if let Some(controls) = shadow.controls.take() { let _ = self.registry.cancel_trial(controls); }
        }
    }
    /// Roll back computation, NOT history, costs, environment or permissions.
    pub fn rollback(&mut self) -> Result<()> {
        if self.frozen_at.is_some() || self.pending || self.poisoned { return Err(Error::Busy); }
        let program = self.previous.clone().ok_or(Error::NoEvidence)?;
        let mut meter = Meter::new(2000000);
        let (state,scale) = calibrate(&program,&self.data(),&self.library,&mut meter)?;
        self.previous = Some(self.active.program.clone()); self.active = Model { program,state,scale }; self.cancel_shadow();
        self.construction_work += meter.spent; Ok(())
    }
    /// Records a real host-observed reset, retaining one lifetime journal.
    pub fn record_external_restart(&mut self,observation: Vec<i64>) -> Result<()> {
        if self.pending || self.poisoned { return Err(Error::Busy); }
        if observation.len() != self.config.width { return Err(Error::InvalidData); }
        self.current = observation; self.restart_next = true; self.active.state = self.active.program.initial_state();
        if let Some(s) = &mut self.shadow { s.model.state = s.model.program.initial_state(); }
        Ok(())
    }

    /// Repair the existing Learner loop: factual uncertainty -> an information
    /// probe -> observed consequence -> updated law -> goal action. All motors
    /// still pass through ProtectedConstructionRuntime and HumanProtection.
    /// No schedules, reward labels, privileged motor roles, or synthetic POSTs.
    pub fn enable_self_directed_control(&mut self)->Result<()> {
        if self.config.width>6 || self.config.actions>8 {return Err(Error::Capacity);}
        if self.autonomy.is_none() || self.pending || self.poisoned || self.self_directed ||
           self.frozen_at.is_some() {return Err(Error::Busy);}
        self.self_directed=true;
        Ok(())
    }
    pub fn self_directed_control_enabled(&self)->bool {self.self_directed}
    /// Returns both the factual choice and its metered reasoning cost. All
    /// loops stop as soon as the configured work budget is spent. The candidate
    /// is still only a U1 proposal, not authority to perform an action.
    fn factual_control_metered(&self,goal:&[Option<i64>],allowed:&[usize],budget:u64)->(usize,u64){
        let mut meter=Meter::new(budget);
        let baseline=goal_distance(&self.current,goal);
        let mut best:Option<(u128,usize,usize)>=None;
        let mut frontier=VecDeque::from([(self.current.clone(),None::<usize>,0usize)]);
        let mut visited=BTreeSet::new();visited.insert(self.current.clone());
        let mut expanded=0usize;
        let cap=budget.min(512) as usize;
        'search: while let Some((state,first,depth))=frontier.pop_front() {
            if depth>=self.config.planning_depth || expanded>=cap {continue}
            for &action in allowed {
                if meter.charge(1+state.len() as u64).is_err(){break 'search}
                expanded+=1;
                let Some(next)=factual_prediction(&self.records,&state,action,self.frozen_at,&mut meter) else {
                    if meter.remaining()==0 {break 'search}
                    continue
                };
                if next==state{continue}
                let origin=first.unwrap_or(action);
                let distance=goal_distance(&next,goal);
                if distance<baseline {
                    let candidate=(distance,depth+1,origin);
                    if best.as_ref().is_none_or(|b|candidate<*b){best=Some(candidate)}
                }
                if visited.len()<2048 {
                    if meter.charge(12+next.len() as u64).is_err(){break 'search}
                    if visited.insert(next.clone()) {frontier.push_back((next,Some(origin),depth+1))}
                }
            }
        }
        if let Some((_,_,action))=best {return (action,meter.spent)}
        // No proven model improvement. Probe less-observed actions based only
        // on PRE-FREEZE evidence; post-freeze action logs cannot retrain it.
        let end=self.frozen_at.map_or(self.records.len(),|id|
            self.records.partition_point(|r|r.transition.id<id));
        let mut selected=allowed[0];let mut rank=(usize::MAX,usize::MAX,u64::MAX);
        for &action in allowed {
            let mut local=0usize;let mut global=0usize;
            for r in self.records[..end].iter().rev().take(96) {
                if meter.charge(self.current.len() as u64+1).is_err(){break}
                if r.transition.action==action {
                    global+=1;
                    if r.transition.pre==self.current{local+=1}
                }
            }
            let tie_event=self.frozen_at.unwrap_or(self.next_id);
            let score=(local,global,exploration_word(tie_event^action as u64));
            if score<rank {rank=score;selected=action}
            if meter.remaining()==0{break}
        }
        (selected,meter.spent)
    }
    /// Explicitly report factual planning work to its caller so both the
    /// standalone protected executor and U1 can charge their own ledgers.
    pub(crate) fn choose_offered_metered_budget(&self,goal:Option<&[Option<i64>]>,offers:Option<&[ActionOffer]>,budget:u64)->Result<(usize,u64)> {
        if !self.self_directed {return self.choose_offered(goal,offers).map(|action|(action,0))}
        if self.pending||self.poisoned{return Err(Error::Busy)}
        if goal.is_some_and(|g|g.len()!=self.config.width) || offers.is_some_and(|o|o.len()!=self.config.actions){return Err(Error::InvalidData)}
        let remaining=self.autonomy_report().map(|r|r.remaining_cost).unwrap_or(u64::MAX);
        let allowed=(0..self.config.actions).filter(|&a|offers.is_none_or(|o|o[a].available&&o[a].max_cost<=remaining)).collect::<Vec<_>>();
        if allowed.is_empty(){return Err(Error::Blocked)}
        if goal.is_some_and(|g|g.iter().any(Option::is_some)&&goal_distance(&self.current,g)==0){return Err(Error::NoEvidence)}
        Ok(self.factual_control_metered(goal.unwrap_or(&[]),&allowed,budget))
    }
    /// Ephemeral imagination plus a model-independent exploration reserve.
    /// No simulated POST is inserted into the factual journal.
    pub fn choose_action(&self,goal: Option<&[i64]>) -> Result<usize> {
        let owned = goal.map(|g| g.iter().copied().map(Some).collect::<Vec<_>>());
        self.choose_masked(owned.as_deref())
    }
    /// Explicit setpoints on a subset of observable coordinates. Unspecified
    /// nuisance channels are not silently assigned a goal or a semantic label.
    pub fn choose_masked(&self,goal: Option<&[Option<i64>]>) -> Result<usize> {self.choose_offered(goal,None)}
    pub fn choose_offered(&self,goal:Option<&[Option<i64>]>,offers:Option<&[ActionOffer]>)->Result<usize> {
        if self.pending || self.poisoned { return Err(Error::Busy); }
        if goal.is_some_and(|g| g.len() != self.config.width) { return Err(Error::InvalidData); }
        let remaining=self.autonomy_report().map(|r|r.remaining_cost).unwrap_or(u64::MAX);
        if offers.is_some_and(|o|o.len()!=self.config.actions){return Err(Error::InvalidData);}
        let allowed:Vec<_>=(0..self.config.actions).filter(|&a|offers.is_none_or(|o|o[a].available && o[a].max_cost<=remaining)).collect();
        if allowed.is_empty(){return Err(Error::Blocked);}
        if self.self_directed {
            if let Some(goal)=goal {
                if goal.iter().any(Option::is_some) && goal_distance(&self.current,goal)==0 {
                    return Err(Error::NoEvidence); // no unnecessary protected action
                }
            }
            return Ok(self.factual_control_metered(goal.unwrap_or(&[]),&allowed,self.config.planning_work).0);
        }
        let word=exploration_word(self.next_id);
        if goal.is_none() && word & 3 == 0 {return Ok(allowed[((word>>2)%allowed.len() as u64) as usize]);}
        let mut models = vec![&self.active]; if let Some(s) = &self.shadow { models.push(&s.model); }
        let mut meter = Meter::new(self.config.planning_work);
        let mut usage = vec![0usize;self.config.actions];
        for r in &self.records { usage[r.transition.action] += 1; }
        let fallback=allowed.iter().copied().min_by_key(|&a|(usage[a],a)).ok_or(Error::Blocked)?;
        let mut best = (f64::NEG_INFINITY,fallback); let mut paths = vec![vec![]];
        for depth in 1..=self.config.planning_depth {
            let mut next_paths = Vec::new();
            for prefix in paths {
                for action in allowed.iter().copied() {
                    if meter.remaining() < 10 { return Ok(best.1); }
                    meter.charge(1)?;
                    let mut path = prefix.clone(); path.push(action);
                    let quoted=offers.map(|o|path.iter().try_fold(0_u64,|sum,&a|sum.checked_add(o[a].max_cost))).unwrap_or(Some(0));
                    let Some(quoted)=quoted.filter(|&cost|cost<=remaining) else {continue};
                    let mut predictions = Vec::new(); let mut valid = true;
                    for model in &models {
                        let mut input = self.current.clone(); let mut state = model.state.clone();
                        for &a in &path {
                            match model.program.step(&input,a,&state,&self.library,&mut meter) {
                                Ok((o,s)) => { input = o; state = s; },
                                Err(Error::Budget) => return Ok(best.1),
                                Err(_) => { valid = false; break; }
                            }
                        }
                        if valid { predictions.push(input); } else { break; }
                    }
                    if !valid { continue; }
                    let information = if predictions.len() > 1 && predictions[0] != predictions[1] { 1.0 } else { 0.0 };
                    let utility = goal.map(|g| predictions.iter().map(|p| {
                        if p.iter().zip(g).all(|(&a,b)| b.is_none_or(|v| a == v)) { 5.0 }
                        else { -p.iter().zip(g).filter_map(|(&a,b)| b.map(|v| ((a as i128-v as i128).abs()) as f64)).sum::<f64>() }
                    }).sum::<f64>()/predictions.len() as f64).unwrap_or(0.0);
                    let penalty=if offers.is_some(){0.01*(quoted as f64).ln_1p()}else{0.0};
                    let score = utility+information+0.1/(1.0+usage[path[0]] as f64)-0.01*depth as f64-penalty;
                    if score > best.0 { best = (score,path[0]); }
                    if next_paths.len() < 4096 { next_paths.push(path); }
                }
            }
            paths = next_paths;
        }
        Ok(best.1)
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        if self.pending { return Err(Error::Busy); }
        let payload = serde_json::to_vec(self).map_err(|_| Error::InvalidData)?;
        if payload.len() > 4*1024*1024 { return Err(Error::Capacity); }
        serde_json::to_vec(&(1u32,fingerprint(&payload),self)).map_err(|_| Error::InvalidData)
    }
    pub fn from_checkpoint(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 4*1024*1024 { return Err(Error::Capacity); }
        let (version,hash,mut state): (u32,u64,Self) = serde_json::from_slice(bytes).map_err(|_| Error::InvalidData)?;
        if version != 1 || hash != fingerprint(&serde_json::to_vec(&state).map_err(|_| Error::InvalidData)?) || state.pending { return Err(Error::InvalidData); }
        if let Some(c) = &state.curriculum {
            c.validate(state.config.width)?;
            if c.pending_target() != state.shadow.as_ref().map(|s| s.trace.job.target) { return Err(Error::InvalidData); }
        }
        state.config.validate()?; state.library.validate(state.config.width)?; state.registry.validate(&state.library)?;
        if let Some(book)=&state.requirements {
            book.validate()?;
            let learned=book.report();
            // LC1-only spent work is part of the all-mechanism Learner ledger.
            if learned.work>state.construction_work{return Err(Error::InvalidData)}
            // Both freeze directions must match, even after re-envelope.
            if learned.frozen!=state.frozen_at.is_some(){return Err(Error::InvalidData)}
        }
        if state.frozen_at.is_some_and(|id|id>state.next_id || state.shadow.is_some()){return Err(Error::InvalidData);}
        if state.self_directed && (state.autonomy.is_none() || state.config.width>6 || state.config.actions>8) {
            return Err(Error::InvalidData);
        }
        if let Some(policy)=&state.autonomy {
            policy.validate(&state.config)?;
            if state.curriculum.as_ref().is_none_or(|c|c.mode!=SelectionMode::Fair)
                || policy.evidence!=state.epistemic.as_ref().map(|r|r.report(None).policy){return Err(Error::InvalidData);}
        }
        if let Some(risk) = &state.epistemic {
            risk.validate(state.config.width)?;
            if state.config.search.deterministic { return Err(Error::InvalidData); }
        }
        if state.records.len() > state.config.journal_capacity || state.promotions.len() > 128 || state.current.len() != state.config.width || state.next_id != state.records.len() as u64 { return Err(Error::InvalidData); }
        state.active.program.validate(state.config.width,&state.library)?;
        if let Some(p) = &state.previous { p.validate(state.config.width,&state.library)?; }
        let mut meter = Meter::new(10000000);
        if !state.records.is_empty() {
            validate_data(&state.data(),state.config.width,state.config.actions)?;
            if !state.restart_next && state.records.last().unwrap().transition.post != state.current { return Err(Error::InvalidData); }
            for (i,r) in state.records.iter().enumerate() {
                if r.transition.id != i as u64 { return Err(Error::InvalidData); }
                r.prediction.nll(&r.transition.post)?;
                if let Some(p) = &r.candidate_prediction { p.nll(&r.transition.post)?; }
            }
            state.active.state = replay(&state.active.program,&state.data(),&state.library,&mut meter)?.1;
        } else { state.active.state = state.active.program.initial_state(); }
        if state.restart_next { state.active.state = state.active.program.initial_state(); }
        Forecast { center: state.current.clone(),scale: state.active.scale.clone() }.nll(&state.current)?;
        let data = state.data();
        if let Some(s) = &mut state.shadow {
            if let Some(trial) = &s.controls {
                trial.validate()?;
                if trial.training_len() != s.training_len || trial.program() != &s.model.program
                    || state.registry.meta.pending != Some(trial.fingerprint()) { return Err(Error::InvalidData); }
            } else if state.registry.meta.pending.is_some() { return Err(Error::InvalidData); }
            s.model.program.validate(state.config.width,&state.library)?;
            if s.training_len > data.len() || s.start_id != s.training_len as u64 || s.samples != data.len()-s.training_len || (s.evidence.is_none() && s.samples >= state.config.shadow_steps) || !s.incumbent_nll.is_finite() || !s.candidate_nll.is_finite() { return Err(Error::InvalidData); }
            if let Some(e) = &s.evidence {
                let risk=state.epistemic.as_ref().ok_or(Error::InvalidData)?;
                e.binding(risk,&state.active.program,&s.model.program,&state.library,s.trace.job.target,s.start_id)?;
                if e.samples()!=s.samples || e.ready() { return Err(Error::InvalidData); }
                let future=&state.records[s.training_len..];
                if future.iter().any(|r| r.model!=state.active.program.fingerprint() || r.candidate!=Some(s.model.program.fingerprint())) { return Err(Error::Stale); }
                let old:Vec<_>=future.iter().map(|r|r.prediction.center.clone()).collect();
                let new:Vec<_>=future.iter().map(|r|r.candidate_prediction.as_ref().map(|p|p.center.clone()).ok_or(Error::Stale)).collect::<Result<_>>()?;
                e.verify_replay(&old,&new,&data[s.training_len..])?;
                // Check stored forecasts against the frozen executable models,
                // not only against another editable presentation of evidence.
                let old_vm=replay(&state.active.program,&data,&state.library,&mut meter)?.0;
                let new_vm=replay(&s.model.program,&data,&state.library,&mut meter)?.0;
                if old_vm[s.training_len..]!=old || new_vm[s.training_len..]!=new {return Err(Error::InvalidData);}
            } else if state.epistemic.is_some() { return Err(Error::InvalidData); }
            s.model.state = replay(&s.model.program,&data,&state.library,&mut meter)?.1;
            if state.restart_next { s.model.state = s.model.program.initial_state(); }
            Forecast { center: state.current.clone(),scale: s.model.scale.clone() }.nll(&state.current)?;
        }
        if state.shadow.is_none() && state.registry.meta.pending.is_some() { return Err(Error::InvalidData); }
        Ok(state)
    }
}
#[derive(Debug)]
pub struct ExternalPost { pub observation: Vec<i64>,pub cost: u64 }
/// Does not clone or serialize the live action authority.
#[derive(Debug)]
pub struct ProtectedConstructionRuntime { organism: EvoPhase,learner: Learner,permit_offset:u64 }
impl ProtectedConstructionRuntime {
    pub fn new(organism: EvoPhase,config: Config,initial: Vec<i64>) -> Result<Self> {
        if organism.config().motor_cells != config.actions || organism.config().sensory_cells != config.width { return Err(Error::InvalidData); }
        Ok(Self { organism,learner: Learner::new(config,initial)?,permit_offset:0 })
    }
    pub fn enable_curriculum(&mut self,mode: SelectionMode) -> Result<()> { self.learner.enable_curriculum(mode) }
    pub fn enable_autonomy(&mut self,policy:AutonomyPolicy)->Result<()> {self.learner.enable_autonomy(policy)}
    pub fn freeze_learning(&mut self)->Result<()> {self.learner.freeze_learning()}
    pub fn enable_self_directed_control(&mut self)->Result<()> {self.learner.enable_self_directed_control()}
    pub fn step_autonomous<H,F>(&mut self,goal:Option<&[Option<i64>]>,offers:&[ActionOffer],hazard:H,execute:F)->Result<usize>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->Result<ExternalPost> {
        if self.learner.autonomy.is_none(){return Err(Error::InvalidData);}
        let (action,work)=self.learner.choose_offered_metered(goal,Some(offers))?;
        self.learner.construction_work=self.learner.construction_work.checked_add(work).ok_or(Error::Capacity)?;
        self.step_action_inner(action,hazard,execute,Some(offers[action].max_cost))
    }
    /// Restores software cognition only, never the old SNN or a live permit.
    pub(super) fn resume_session(bytes:&[u8],emergency_latched:bool)->Result<Self> {
        let learner=Learner::from_checkpoint(bytes)?;
        if learner.poisoned || emergency_latched || learner.autonomy.is_none(){return Err(Error::Blocked);}
        let permit_offset=learner.records.iter().map(|r|r.permit_sequence).max().unwrap_or(0);
        let organism=EvoPhase::new(crate::EvoConfig{sensory_cells:learner.config.width,motor_cells:learner.config.actions,..Default::default()});
        Ok(Self{organism,learner,permit_offset})
    }
    pub fn enable_epistemic(&mut self,policy: EvidencePolicy) -> Result<()> { self.learner.enable_epistemic(policy) }
    pub fn learner(&self) -> &Learner { &self.learner }
    pub fn organism(&self) -> &EvoPhase { &self.organism }
    pub fn checkpoint(&self) -> Result<Vec<u8>> { self.learner.checkpoint() }
    /// Environment-reported restart, not an imagined reset or action. It
    /// leaves the cognitive journal and lifetime's physical permits intact.
    pub fn observe_factual_restart(&mut self,observation:Vec<i64>)->Result<()> {
        self.learner.record_external_restart(observation)
    }
    /// Live round-trip requires the identical cognitive generation, including
    /// history, costs, consumed trials and fault state. Equal observations alone
    /// are insufficient. Use rollback_cognition to revert only computation.
    pub fn restore_cognition(&mut self,bytes: &[u8]) -> Result<()> {
        if self.learner.pending { return Err(Error::Busy); }
        let restored = Learner::from_checkpoint(bytes)?;
        if restored.checkpoint()? != self.learner.checkpoint()? { return Err(Error::Stale); }
        self.learner = restored; Ok(())
    }
    pub fn rollback_cognition(&mut self) -> Result<()> { self.learner.rollback() }
    pub fn step<H,F>(&mut self,goal: Option<&[i64]>,hazard: H,execute: F) -> Result<usize>
    where H: FnOnce(usize)->HumanProtectionEvidence,F: FnOnce(usize)->Result<ExternalPost> {
        let action = self.learner.choose_action(goal)?; self.step_action(action,hazard,execute)
    }
    pub fn step_masked<H,F>(&mut self,goal: &[Option<i64>],hazard: H,execute: F) -> Result<usize>
    where H: FnOnce(usize)->HumanProtectionEvidence,F: FnOnce(usize)->Result<ExternalPost> {
        let action = self.learner.choose_masked(Some(goal))?; self.step_action(action,hazard,execute)
    }
    /// Explicitly supervised interventions, still protected and fact-recorded.
    pub fn step_action<H,F>(&mut self,action:usize,hazard:H,execute:F)->Result<usize>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->Result<ExternalPost> {
        if self.learner.autonomy.is_some(){return Err(Error::Execution("autonomous lifetime requires PRE action offers".into()));}
        self.step_action_inner(action,hazard,execute,None)
    }
    fn step_action_inner<H,F>(&mut self,action:usize,hazard:H,execute:F,max_cost:Option<u64>)->Result<usize>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->Result<ExternalPost> {
        let token = self.learner.prepare(action)?;
        match self.organism.screen_physical_action(action,hazard(action)) {
            HumanProtectionDecision::Blocked(_) => { self.learner.cancel(token)?; Err(Error::Blocked) },
            HumanProtectionDecision::Allowed { permit,.. } => {
                let sequence=self.permit_offset.checked_add(permit.sequence()).ok_or(Error::Capacity)?;
                let permitted = match self.organism.consume_physical_action_permit(permit) {
                    Ok(a) => a,Err(e) => { self.learner.cancel(token)?; return Err(Error::Execution(format!("permit: {e:?}"))); }
                };
                match execute(permitted) {
                    Ok(post)=>{
                        let exceeded=max_cost.is_some_and(|cap|post.cost>cap);
                        if exceeded {self.learner.poisoned=true;self.learner.last_error=Some("external cost exceeded PRE quote".into());}
                        self.learner.commit(token,post.observation,sequence,post.cost)?;
                        if exceeded{Err(Error::Execution("external cost exceeded PRE quote".into()))}else{Ok(permitted)}
                    },
                    Err(e) => { self.learner.fault(token,format!("external callback: {e}")); Err(e) }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvoConfig;
    fn safe(_: usize) -> HumanProtectionEvidence { HumanProtectionEvidence { human_present: false,physical_effect_possible: false,predicted_harm_probability: 0.0,hazard_confidence: 1.0,emergency_stop: false } }
    fn runtime() -> ProtectedConstructionRuntime {
        let cfg = Config { minimum_training: 6,shadow_steps: 4,search_every: 2,search: SearchConfig { deterministic: true,..Default::default() },..Default::default() };
        ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig { sensory_cells: 1,motor_cells: 2,..Default::default() }),cfg,vec![0]).unwrap()
    }
    fn train(r: &mut ProtectedConstructionRuntime,n: usize) -> i64 {
        let mut hidden = 0;
        for i in 0..n {
            let action = [1,1,0,1,0,0,1,0][i%8];
            r.step_action(action,safe,|a| { let post = hidden; hidden = a as i64; Ok(ExternalPost { observation: vec![post],cost: 1 }) }).unwrap();
        }
        hidden
    }
    #[test] fn closed_loop_acquires_state_then_uses_fresh_shadow_observations() {
        let mut r = runtime(); train(&mut r,32);
        assert!(!r.learner.program().cells.is_empty(),"{:?}",r.learner.report());
        assert!(r.learner.promotions().iter().any(|p| p.accepted));
        for p in r.learner.promotions() { assert!(p.last_id >= p.first_id+3); }
        for pair in r.learner.records().windows(2) { assert!(pair[0].permit_sequence < pair[1].permit_sequence); }
    }
    #[test] fn autonomous_acquisition_survives_frozen_non_alternating_probe() {
        let mut r = runtime(); let mut hidden = 0;
        for _ in 0..64 {
            r.step(None,safe,|a| { let post = hidden; hidden = a as i64; Ok(ExternalPost { observation: vec![post],cost: 1 }) }).unwrap();
        }
        let p = r.learner.program().clone(); let lib = r.learner.library().clone();
        let mut meter = Meter::new(100000);
        let mut state = replay(&p,&r.learner.data(),&lib,&mut meter).unwrap().1;
        let mut input = r.learner.current().to_vec(); let mut wrong = 0; let mut confound_wrong = 0;
        for a in [0,0,0,1,0,1,1,1].into_iter().cycle().take(24) {
            let (prediction,next) = p.step(&input,a,&state,&lib,&mut meter).unwrap();
            let actual = hidden; hidden = a as i64;
            wrong += usize::from(prediction != [actual]);
            confound_wrong += usize::from(i64::from(a == 0) != actual);
            state = next; input = vec![actual];
        }
        assert_eq!(wrong,0,"frozen model: {p:?}; {:?}",r.learner.report());
        assert!(confound_wrong > 0); assert_eq!(r.learner.records().len(),64);
    }
    #[test] fn simulation_does_not_manufacture_factual_experience() {
        let mut r = runtime(); train(&mut r,24); let n = r.learner.records().len();
        for _ in 0..5 { r.learner.choose_action(Some(&[1])).unwrap(); }
        assert_eq!(r.learner.records().len(),n);
    }
    #[test] fn blocked_callback_and_checkpoint_cannot_clear_emergency_stop() {
        let mut r = runtime(); let snapshot = r.checkpoint().unwrap(); let mut calls = 0;
        let blocked = r.step_action(0,|a| { let mut evidence = safe(a); evidence.emergency_stop = true; evidence },|_| { calls += 1; Ok(ExternalPost { observation: vec![0],cost: 1 }) });
        assert_eq!(blocked,Err(Error::Blocked)); assert_eq!(calls,0); assert!(r.learner.records().is_empty());
        r.restore_cognition(&snapshot).unwrap(); assert!(r.organism().human_protection_emergency_latched());
        assert_eq!(r.step_action(0,safe,|_| { calls += 1; Ok(ExternalPost { observation: vec![0],cost: 1 }) }),Err(Error::Blocked));
        assert_eq!(calls,0);
    }
    #[test] fn callback_fault_poisoned_not_replayed() {
        let mut r = runtime(); assert!(r.step_action(1,safe,|_| Err(Error::Execution("connection lost".into()))).is_err());
        assert!(r.learner.report().poisoned); let mut called = false;
        assert!(r.step_action(1,safe,|_| { called = true; Ok(ExternalPost { observation: vec![0],cost: 1 }) }).is_err()); assert!(!called);
    }
    #[test] fn older_snapshot_cannot_erase_a_same_observation_event() {
        let mut r = runtime(); let bytes = r.checkpoint().unwrap();
        r.step_action(0,safe,|_| Ok(ExternalPost { observation: vec![0],cost: 7 })).unwrap();
        assert_eq!(r.restore_cognition(&bytes),Err(Error::Stale));
        assert_eq!(r.learner.records().len(),1); assert_eq!(r.learner.records()[0].external_cost,7);
    }
    #[test] fn snapshot_cannot_clear_unknown_external_state() {
        let mut r = runtime(); let bytes = r.checkpoint().unwrap();
        let _ = r.step_action(1,safe,|_| Err(Error::Execution("unknown result".into())));
        assert_eq!(r.restore_cognition(&bytes),Err(Error::Stale)); assert!(r.learner.report().poisoned);
    }
    #[test] fn checkpoint_round_trip_and_corruption() {
        let mut r = runtime(); train(&mut r,32); let bytes = r.checkpoint().unwrap(); let restored = Learner::from_checkpoint(&bytes).unwrap();
        assert_eq!(restored.program(),r.learner.program()); assert_eq!(restored.current(),r.learner.current()); assert_eq!(restored.records().len(),32);
        r.restore_cognition(&bytes).unwrap();
        let mut corrupt = bytes; corrupt[3] ^= 1; assert!(Learner::from_checkpoint(&corrupt).is_err());
    }
    #[test] fn capacity_is_checked_before_external_execution() {
        let mut r = runtime(); r.learner.config.journal_capacity = 10; train(&mut r,10);
        let mut called = false;
        assert_eq!(r.step_action(1,safe,|_| { called = true; Ok(ExternalPost { observation: vec![0],cost: 1 }) }),Err(Error::Capacity)); assert!(!called);
    }
    #[test] fn probability_handles_extreme_integer_difference() {
        let f = Forecast { center: vec![i64::MIN],scale: vec![0.25] };
        assert!(f.nll(&[i64::MAX]).unwrap().is_finite());
    }
    #[test] fn rollback_does_not_delete_experience() {
        let mut r = runtime(); train(&mut r,32); let n = r.learner.records().len();
        r.rollback_cognition().unwrap(); assert_eq!(r.learner.records().len(),n); assert_eq!(r.learner.next_id,n as u64);
    }
    // Fault-injection fixture for authority precedence, not a learning benchmark.
    // Deliberately control the local likelihood vote while keeping the real
    // begin_trial/complete_trial implementation and its frozen candidate.
    fn audit_shadow_fixture(future_offset: i64,invalid_future: bool,local_vote: bool) -> (Learner,Program) {
        let config = Config { minimum_training: 6,shadow_steps: 6,..Default::default() };
        let mut learner = Learner::new(config,vec![0]).unwrap();
        let acquisition: Vec<_> = (0..6u64).map(|i| Transition {
            id: i,restart: true,pre: vec![i as i64],action: 0,post: vec![i as i64+17],
        }).collect();
        let controls = learner.registry.begin_trial(&learner.active.program,&acquisition,&learner.library,&learner.config.search).unwrap();
        let candidate = controls.program().clone();
        assert_ne!(candidate,learner.active.program);
        let trace = controls.trace().clone();
        let (state,scale) = calibrate(&candidate,&acquisition,&learner.library,&mut Meter::new(1_000_000)).unwrap();
        let mut future: Vec<_> = (6..12u64).map(|i| Transition {
            id: i,restart: true,pre: vec![i as i64],action: 0,post: vec![i as i64+future_offset],
        }).collect();
        if invalid_future { future[0].id = 0; } // duplicate evidence, not a new trial
        let mut records = acquisition.clone(); records.extend(future);
        let parent = learner.active.program.fingerprint();
        learner.records = records.into_iter().enumerate().map(|(index,transition)| Record {
            prediction: Forecast { center: transition.pre.clone(),scale: vec![1.0] },
            transition,model: parent,candidate: None,candidate_prediction: None,
            permit_sequence: index as u64+1,external_cost: 1,
        }).collect();
        learner.current = learner.records.last().unwrap().transition.post.clone();
        learner.next_id = learner.records.len() as u64;
        learner.restart_next = false;
        learner.shadow = Some(Shadow {
            model: Model { program: candidate.clone(),state,scale },trace,
            training_len: acquisition.len(),start_id: 6,samples: 6,
            incumbent_nll: if local_vote { 1_000_000.0 } else { 0.0 },
            candidate_nll: if local_vote { 0.0 } else { 1_000_000.0 },
            controls: Some(controls),evidence: None,
        });
        (learner,candidate)
    }
    fn assert_audit_rejection(learner: &Learner,parent: &Program,library: &Library,events: usize) {
        assert_eq!(learner.program(),parent);
        assert_eq!(learner.library(),library);
        assert_eq!(learner.records.len(),events);
        assert_eq!(learner.next_id,events as u64);
        assert!(learner.previous.is_none());
        assert!(learner.shadow.is_none());
        assert!(learner.registry.traces().is_empty());
        assert!(learner.registry.constructors().is_empty());
        assert_eq!(learner.promotions.len(),1);
        assert!(!learner.promotions[0].accepted);
        assert!(!learner.last_search().unwrap().prospectively_accepted);
    }
    #[test] fn shadow_audit_error_cannot_install_locally_preferred_model() {
        let (mut learner,_) = audit_shadow_fixture(17,true,true);
        let parent = learner.program().clone(); let library = learner.library().clone();
        let events = learner.records.len();
        learner.finish_shadow().unwrap();
        assert_audit_rejection(&learner,&parent,&library,events);
        assert_eq!(learner.registry.meta_ledger().abandoned,1);
        assert!(learner.registry.meta_ledger().reports().is_empty());
        assert!(learner.last_error.as_deref().unwrap().contains("InvalidData"));
    }
    #[test] fn shadow_audit_veto_cannot_install_locally_preferred_model() {
        let (mut learner,_) = audit_shadow_fixture(0,false,true);
        let parent = learner.program().clone(); let library = learner.library().clone();
        let events = learner.records.len();
        learner.finish_shadow().unwrap();
        assert_audit_rejection(&learner,&parent,&library,events);
        assert!(!learner.registry.meta_ledger().reports().last().unwrap().model_accepted);
        assert!(learner.last_error.as_deref().unwrap().contains("vetoed"));
    }
    #[test] fn shadow_audit_acceptance_still_requires_local_vote() {
        let (mut learner,_) = audit_shadow_fixture(17,false,false);
        let parent = learner.program().clone(); let library = learner.library().clone();
        let events = learner.records.len();
        learner.finish_shadow().unwrap();
        assert_audit_rejection(&learner,&parent,&library,events);
        assert!(!learner.registry.meta_ledger().reports().last().unwrap().model_accepted);
    }
    #[test] fn shadow_joint_acceptance_installs_exact_frozen_candidate() {
        let (mut learner,candidate) = audit_shadow_fixture(17,false,true);
        let parent = learner.program().clone(); let events = learner.records.len();
        learner.finish_shadow().unwrap();
        assert_eq!(learner.program(),&candidate);
        assert_eq!(learner.previous.as_ref(),Some(&parent));
        assert_eq!(learner.records.len(),events);
        assert!(learner.shadow.is_none());
        assert!(learner.promotions.last().unwrap().accepted);
        assert!(learner.registry.meta_ledger().reports().last().unwrap().model_accepted);
        assert!(learner.last_search().unwrap().prospectively_accepted);
        assert_eq!(learner.registry.traces().len(),1);
    }

    #[test] fn shadow_missing_audit_cannot_use_legacy_installation_path() {
        let (mut learner,_) = audit_shadow_fixture(17,false,true);
        let controls = learner.shadow.as_mut().unwrap().controls.take().unwrap();
        learner.registry.cancel_trial(controls).unwrap();
        let parent = learner.program().clone(); let library = learner.library().clone();
        let events = learner.records.len();
        learner.finish_shadow().unwrap();
        assert_audit_rejection(&learner,&parent,&library,events);
        assert!(learner.last_error.as_deref().unwrap().contains("missing frozen trial"));
        assert!(learner.registry.meta_ledger().reports().is_empty());
    }

}

fn carrier_unmanaged(v:&bool)->bool{!*v}
include!("unified_port.rs");

include!("recursive_search_port.rs");

include!("requirements_port.rs");

#[cfg(test)]
mod factual_control_integrity_tests {
    use super::*;

    fn record(id:u64, action:usize, start:i64, end:i64)->Record {
        Record {
            transition:Transition{id,restart:id==0,pre:vec![start,1],action,post:vec![end,1]},
            model:0,prediction:Forecast{center:vec![end,1],scale:vec![1.0,1.0]},
            candidate:None,candidate_prediction:None,permit_sequence:id+1,external_cost:1,
        }
    }

    #[test]
    fn prospective_factual_validation_uses_real_heldout_consequences() {
        // Twelve true observations of the same affine effect, with THREE
        // latest examples withheld from rule fitting, are enough to offer a
        // prediction. We have never given the learner an authored action role.
        let mut data=(0..12).map(|i|record(i,0,i as i64,i as i64+1)).collect::<Vec<_>>();
        let mut meter=Meter::new(40_000);
        assert_eq!(calibrated_factual_prediction(&data,&[30,1],0,None,&mut meter),Some(vec![31,1]));
        assert!(meter.spent>0 && meter.spent<=40_000);

        // A once-consistent prediction that encounters three observed
        // contradictions MUST become unknown rather than trusted, even if
        // the original model would still confidently extrapolate.
        for i in 12..15 {data.push(record(i,0,i as i64,i as i64+10))}
        let mut meter=Meter::new(40_000);
        assert_eq!(calibrated_factual_prediction(&data,&[30,1],0,None,&mut meter),None);

        // Freeze binds BOTH fitting and confidence to the same old prefix.
        let mut meter=Meter::new(40_000);
        assert_eq!(calibrated_factual_prediction(&data,&[30,1],0,Some(12),&mut meter),Some(vec![31,1]));
    }

    #[test]
    fn never_certify_uninformative_repetition_or_unmetered_guess() {
        let data=(0..12).map(|i|record(i,0,0,0)).collect::<Vec<_>>();
        let mut meter=Meter::new(40_000);
        assert_eq!(calibrated_factual_prediction(&data,&[10,1],0,None,&mut meter),None,
            "many no-op repetitions must not certify a useful consequence");
        let data=(0..12).map(|i|record(i,0,i as i64,i as i64+1)).collect::<Vec<_>>();
        let mut meter=Meter::new(100);
        assert_eq!(calibrated_factual_prediction(&data,&[30,1],0,None,&mut meter),None,
            "incomplete validation due to budget exhaustion must fail closed");
        assert!(meter.spent<=100);
    }

    #[test]
    fn freeze_excludes_actual_later_transitions_from_acquired_effects() {
        let cfg=Config{width:2,actions:2,minimum_training:128,search_every:4096,
            planning_work:4000,..Default::default()};
        let mut learner=Learner::new(cfg,vec![5,1]).unwrap();
        learner.enable_autonomy(AutonomyPolicy::default()).unwrap();
        learner.enable_self_directed_control().unwrap();
        for i in 0..8 {learner.records.push(record(i,0,i as i64,i as i64+1))}
        learner.next_id=8;
        let mut before_meter=Meter::new(4000);
        let before=factual_prediction(&learner.records,&[11,1],0,None,&mut before_meter);
        assert_eq!(before,Some(vec![12,1]));
        learner.freeze_learning().unwrap();
        assert_eq!(learner.frozen_at,Some(8));
        // Later real consequences contradict the acquired law, but frozen
        // inference remains pinned to id < frozen_at, even across a restart.
        for i in 8..18 {learner.records.push(record(i,0,i as i64,i as i64+10))}
        learner.next_id=18;
        let mut frozen_meter=Meter::new(4000);
        let after=factual_prediction(&learner.records,&[11,1],0,learner.frozen_at,&mut frozen_meter);
        assert_eq!(after,before);
        let mut unrestricted_meter=Meter::new(4000);
        let unrestricted=factual_prediction(&learner.records,&[11,1],0,None,&mut unrestricted_meter);
        assert_ne!(unrestricted,before,"post-freeze evidence must be excluded, not ignored by chance");
        assert!(frozen_meter.spent<=learner.config.planning_work);
    }


    fn joint_record(id:u64,left:i64,right:i64,effect:i64)->Record {
        let x=id as i64*3;
        let pre=vec![x,left,right];
        let post=vec![x+effect,left,right];
        Record {
            transition:Transition{id,restart:id==0,pre,action:0,post:post.clone()},
            model:0,prediction:Forecast{center:post,scale:vec![1.0;3]},
            candidate:None,candidate_prediction:None,permit_sequence:id+1,external_cost:1,
        }
    }

    #[test]
    fn independently_contrasted_double_guard_survives_later_real_evidence() {
        // Vary both opaque sensors independently. Neither alone explains
        // progress; the conjunction is inferred from PRE/action/POST only.
        let mut data=Vec::new();
        for i in 0..28_u64 {
            let (a,b)=[(0,0),(1,0),(0,1),(1,1)][i as usize%4];
            data.push(joint_record(i,a,b,i64::from(a==1 && b==1)));
        }
        let prior=&data[..25];
        let mut meter=Meter::new(100_000);
        assert_eq!(factual_prediction(prior,&[99,1,1],0,None,&mut meter),Some(vec![100,1,1]));
        let mut meter=Meter::new(4_000);
        assert_eq!(calibrated_factual_prediction(&data,&[99,1,1],0,None,&mut meter),
            Some(vec![100,1,1]),"verified conjunction must fit the existing U1 planning budget");
        assert!(meter.spent<=4_000);

        // Three newest contradictory real POSTs revoke the original effect,
        // rather than treating the first acquired conjunction as immutable.
        for i in 28..31_u64 {data.push(joint_record(i,1,1,10))}
        let mut meter=Meter::new(100_000);
        assert_eq!(calibrated_factual_prediction(&data,&[99,1,1],0,None,&mut meter),None);
    }

    #[test]
    fn correlated_sensors_cannot_fake_independent_causal_guards() {
        // The same apparent joint effect without either unilateral contrast
        // is ambiguous: guard A and guard B might be redundant/confounded.
        let data=(0..28_u64).map(|i| {
            let v=(i%2) as i64;
            joint_record(i,v,v,v)
        }).collect::<Vec<_>>();
        let mut meter=Meter::new(100_000);
        assert_eq!(calibrated_factual_prediction(&data,&[99,1,1],0,None,&mut meter),None,
            "do not certify a conditional cause from perfectly correlated sensors");
    }


    #[test]
    fn certified_multistep_chain_uses_only_heldout_validated_factual_edges() {
        // SYNTHETIC journal fixture, not an autonomous acquisition benchmark.
        // The learner is never given motor role labels or prerequisite indices.
        // Action 0 sets the first sensor, action 1 the second; action 2
        // increments progress iff BOTH sensors were set. No one-step move
        // from [0,0,0] reaches the masked goal.
        let cfg=Config{width:3,actions:3,minimum_training:128,search_every:4096,
            planning_depth:4,planning_work:120_000,..Default::default()};
        let mut learner=Learner::new(cfg,vec![0,0,0]).unwrap();
        learner.enable_autonomy(AutonomyPolicy::default()).unwrap();
        learner.enable_self_directed_control().unwrap();
        let mut id=0_u64;
        for i in 0..48_usize {
            let (left,right)=[(0,0),(1,0),(0,1),(1,1)][i%4];
            for action in 0..3 {
                let pre=vec![0,left,right];
                let mut post=pre.clone();
                match action {
                    0=>post[1]=1,
                    1=>post[2]=1,
                    2=>{if left==1&&right==1{post[0]+=1}},
                    _=>unreachable!(),
                }
                learner.records.push(Record{
                    transition:Transition{id,restart:id==0,pre,action,post:post.clone()},
                    model:0,prediction:Forecast{center:post,scale:vec![1.0;3]},
                    candidate:None,candidate_prediction:None,
                    permit_sequence:id+1,external_cost:1,
                });
                id+=1;
            }
        }
        learner.next_id=id;
        let goal=[Some(1),None,None];
        let (plan,spent)=learner.carrier_certified_goal_chain_metered(&goal,&[true;3],120_000);
        let plan=plan.expect("a two-prerequisite acquired chain must be constructible");
        assert_eq!(plan.depth,3);
        assert!(plan.first_action==0||plan.first_action==1);
        assert_eq!(plan.first_consequence[0],0,"first action must establish a prerequisite, not fake goal progress");
        assert_eq!(plan.endpoint[0],1);
        assert!(spent<=120_000);
        let (disabled,spent)=learner.carrier_certified_goal_chain_metered(&goal,&[true,false,true],120_000);
        assert!(disabled.is_none(),"an unavailable prerequisite must prevent a fabricated plan");
        assert!(spent<=120_000);
        let (unfunded,spent)=learner.carrier_certified_goal_chain_metered(&goal,&[true;3],100);
        assert!(unfunded.is_none(),"insufficient proof budget must abstain");
        assert!(spent<=100);
        // Three genuine contradictory consequences of the same last action
        // must revoke the previously accepted causal edge on recomputation.
        for i in 0..3 {
            let pre=vec![i,1,1];let post=vec![i+10,1,1];
            learner.records.push(Record{
                transition:Transition{id,restart:false,pre,action:2,post:post.clone()},
                model:0,prediction:Forecast{center:post,scale:vec![1.0;3]},
                candidate:None,candidate_prediction:None,permit_sequence:id+1,external_cost:1,
            });id+=1;
        }
        learner.next_id=id;
        let (revoked,_) = learner.carrier_certified_goal_chain_metered(&goal,&[true;3],120_000);
        assert!(revoked.is_none(),"contradicting real consequences must not leave a trusted old chain");
    }

    #[test]
    fn double_guard_fails_closed_on_insufficient_meter() {
        let data=(0..28_u64).map(|i|{
            let (a,b)=[(0,0),(1,0),(0,1),(1,1)][i as usize%4];
            joint_record(i,a,b,i64::from(a==1&&b==1))
        }).collect::<Vec<_>>();
        let mut meter=Meter::new(100);
        assert_eq!(calibrated_factual_prediction(&data,&[99,1,1],0,None,&mut meter),None);
        assert!(meter.spent<=100);
    }

    #[test]
    fn factual_search_cost_is_bounded_even_with_large_journal() {
        let cfg=Config{width:2,actions:2,minimum_training:128,search_every:4096,
            journal_capacity:4096,planning_work:100,..Default::default()};
        let mut learner=Learner::new(cfg,vec![5,1]).unwrap();
        learner.enable_autonomy(AutonomyPolicy::default()).unwrap();
        learner.enable_self_directed_control().unwrap();
        for i in 0..4000 {learner.records.push(record(i,0,i as i64,i as i64+1))}
        learner.next_id=4000;
        let (_,cost)=learner.choose_offered_metered(Some(&[Some(10),None]),None).unwrap();
        assert!(cost<=100,"measured factual work must never exceed planning_work");
        let mut meter=Meter::new(100);
        let _=factual_prediction(&learner.records,&[5,1],0,None,&mut meter);
        assert!(meter.spent<=100);
    }
}
