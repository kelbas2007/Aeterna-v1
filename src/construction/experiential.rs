//! RSP3 development: acquire a VM search policy from completed support-only
//! searches. This is a separate *proposal*, never a bypass of RSP2 admission.
//! All grammar operators/feature meanings are authored. Training pairs may
//! contain teacher probes; the final reserved exam must be held outside this API.
use super::search_policy::{search, Objective, Observation, SearchLimits, SearchPolicy, ValidationCase, FEATURES};
use super::vm::{Context, Error, Expr, Library, Meter, Op, Result, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyConfig {
    pub limits: SearchLimits,
    pub candidate_limit: usize,
    pub shortlist: usize,
    pub train_seed: u64,
    pub validation_seed: u64,
}
impl StudyConfig {
    pub fn validate(&self)->Result<()> {
        self.limits.validate()?;
        if !(2..=256).contains(&self.candidate_limit) || !(2..=64).contains(&self.shortlist) {return Err(Error::Capacity)}
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperienceScore {
    pub tasks: usize,
    pub verified: usize,
    pub support_fits: usize,
    pub false_fits: usize,
    pub work: u64,
    pub value: u64,
    pub verified_cases: Vec<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyObservation {
    pub index: usize, pub policy: SearchPolicy, pub training: ExperienceScore,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub validation: Option<ExperienceScore>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudyReport {
    pub initial: SearchPolicy,
    pub proposed: SearchPolicy,
    pub adopted_by_rsp2: bool,
    pub feedback_read: bool,
    pub policy_changed: bool,
    pub training_cases: usize,
    pub validation_cases: usize,
    pub candidate_count: usize,
    pub observations: Vec<PolicyObservation>,
    pub total_metered_work: u64,
    pub training_commitment: u64,
    pub validation_commitment: u64,
    pub config: StudyConfig,
}

// Interpreter-public data only, evaluated by the *actual* RSP2 VM.
fn bin(op:Op,a:Expr,b:Expr)->Expr{Expr::bin(op,a,b)}
fn directed(feature:usize,edit:i64,arith:i64,width:usize,gate:usize)->Expr {
    let detail=bin(Op::Add,Expr::Input(feature),Expr::integer((width+7) as i64*arith));
    let encoded=bin(Op::Sub,Expr::integer(0),bin(Op::Add,Expr::integer(1+edit),bin(Op::Mul,Expr::integer(8),detail)));
    let fallback=Expr::Input(8);
    let conditional=if gate==0 {encoded}else {
        let gate_expr=bin(Op::Lt,Expr::Input(1),Expr::integer(gate as i64));
        Expr::If(Box::new(gate_expr),Box::new(encoded),Box::new(fallback.clone()))
    };
    let available=bin(Op::Lt,Expr::integer(-1),Expr::Input(feature));
    Expr::If(Box::new(available),Box::new(conditional),Box::new(fallback))
}

/// An authored *grammar* of strategies, not a list of solved tasks or a target
/// policy. Which executable expression survives is determined by real searches.
pub fn policy_grammar(width:usize,limit:usize)->Result<Vec<SearchPolicy>> {
    if !(1..=8).contains(&width) || !(2..=256).contains(&limit) {return Err(Error::Capacity)}
    let default=SearchPolicy::default();
    let mut out=vec![default.clone(),SearchPolicy{priority:bin(Op::Add,Expr::Input(0),Expr::Input(1)),..default.clone()}];
    let mut seen=BTreeSet::new();for p in &out {seen.insert(p.fingerprint());}
    // Grammar enumeration order is frozen independently of the study outcome.
    // Include baseline with varied location and broad random variations.
    let priorities=[Expr::Input(0),bin(Op::Add,Expr::Input(0),Expr::Input(1))];
    let locations=[Expr::Input(9),Expr::integer(0),Expr::Input(7)];
    let mutations=[Expr::Input(8),Expr::integer(0),Expr::integer(1),Expr::integer(3)];
    let mut variants=mutations.to_vec();
    for feature in [14usize,16,17] {
        for edit in [0,1,3] {for arithmetic in [0,2] {for gate in [0,2,4] {
            variants.push(directed(feature,edit,arithmetic,width,gate));
        }}}
    }
    // Mix all variations rather than hand-selecting the prior best.
    for variation in &variants {
        for location in &locations {for priority in &priorities {
            let p=SearchPolicy{priority:priority.clone(),location:location.clone(),variation:variation.clone()};
            if p.validate().is_ok() && seen.insert(p.fingerprint()) {out.push(p);if out.len()>=limit{return Ok(out)}}
        }}
    }
    Ok(out)
}

// The observation provided to each search comes ONLY from its support rows.
// The probe set is evaluated afterward. The optimizer knows this development
// score, but never receives the final external exam, which is caller-owned.
pub fn evaluate_policy(policy:&SearchPolicy,cases:&[ValidationCase],limits:&SearchLimits,seed:u64,scramble_feedback:bool)->Result<ExperienceScore> {
    limits.validate()?;
    if cases.is_empty() || cases.len()>128 {return Err(Error::Capacity)}
    let mut score=ExperienceScore{tasks:cases.len(),verified:0,support_fits:0,false_fits:0,work:0,value:0,verified_cases:Vec::new()};
    for (i,c) in cases.iter().enumerate(){
        c.validate()?;
        let salt=seed.wrapping_add(i as u64*0x10001);
        let run=if scramble_feedback {
            let mut task=ScrambledObjective{task:c.support.clone()};
            search(policy,Expr::integer(0),&mut task,limits,salt)?
        }else {
            search(policy,Expr::integer(0),&mut c.support.clone(),limits,salt)?
        };
        let mut meter=Meter::new(limits.work);
        let mut pass=run.solved;
        if run.solved {
            for row in &c.probes.examples {
                let response=meter.charge(1).and_then(|_|run.program.eval(Context{input:&row.input,action:0,state:&[],args:&[]},&c.probes.library,&mut meter)).and_then(Value::int);
                if response!=Ok(row.output){pass=false;break;}
            }
        }
        score.support_fits+=usize::from(run.solved);
        score.verified+=usize::from(pass);
        score.false_fits+=usize::from(run.solved&&!pass);
        score.work=score.work.saturating_add(run.work+meter.spent);
        score.value=score.value.saturating_add(if pass {run.work+meter.spent} else {4*limits.work+run.loss.min(limits.work)});
        score.verified_cases.push(pass);
    }
    Ok(score)
}
struct ScrambledObjective {task:super::search_policy::SynthesisTask}
impl Objective for ScrambledObjective {
    fn input_width(&self)->usize{self.task.examples[0].input.len()}
    fn library(&self)->&Library{&self.task.library}
    fn score(&mut self,c:&Expr,m:&mut Meter)->Result<u64>{self.task.score(c,m)}
    fn observe(&mut self,c:&Expr,m:&mut Meter,mask:u32)->Result<Observation>{
        let mut observed=self.task.observe(c,m,mask)?;
        // A deliberately wrong association, not a zeroed signal or free work.
        // Other statistics and all support labels remain untouched.
        let width=self.input_width() as i64;
        for f in [4usize,6,7] {
            if observed.feedback[f]>=0 && width>1 {observed.feedback[f]=(observed.feedback[f]+1)%width;}
        }
        Ok(observed)
    }
}
fn commitment(cases:&[ValidationCase])->u64{super::vm::fingerprint(&serde_json::to_vec(cases).unwrap())}
fn better(a:&ExperienceScore,b:&ExperienceScore)->bool{
    a.verified>b.verified || (a.verified==b.verified && a.value<b.value)
}

/// Produces a *candidate*. Does not touch the active RSP2 search-policy state.
/// Only RecursiveSearch::improve with real RSP2 producer/retention gates can
/// perform a lifecycle adoption; this avoids manufacturing a generation count.
pub fn study(train:&[ValidationCase],validation:&[ValidationCase],config:StudyConfig)->Result<StudyReport>{
    config.validate()?;
    if !(8..=128).contains(&train.len())||!(8..=128).contains(&validation.len()){return Err(Error::Capacity)}
    let width=train[0].support.examples[0].input.len();
    let mut keys=BTreeSet::new();
    for c in train.iter().chain(validation){c.validate()?;
        if c.support.examples[0].input.len()!=width{return Err(Error::InvalidData)}
        // Disjoint support/probe rows inside each case and no exact case reuse.
        if !keys.insert(super::vm::fingerprint(&serde_json::to_vec(c).unwrap())){return Err(Error::Stale)}
    }
    let policies=policy_grammar(width,config.candidate_limit)?;
    let mut observations=Vec::new();let mut cost=0u64;
    for (index,p) in policies.into_iter().enumerate(){
        let training=evaluate_policy(&p,train,&config.limits,config.train_seed,false)?;
        cost=cost.saturating_add(training.work);
        observations.push(PolicyObservation{index,policy:p,training,validation:None});
    }
    // A fair shortlist is formed WITHOUT reading the development validation.
    let mut order:Vec<_>=(0..observations.len()).collect();
    order.sort_by(|&a,&b|observations[b].training.verified.cmp(&observations[a].training.verified)
        .then_with(||observations[a].training.value.cmp(&observations[b].training.value)).then_with(||a.cmp(&b)));
    let mut chosen:BTreeSet<usize>=order.into_iter().take(config.shortlist).collect();
    chosen.insert(0);chosen.insert(1);
    for index in chosen {
        let v=evaluate_policy(&observations[index].policy,validation,&config.limits,config.validation_seed,false)?;
        cost=cost.saturating_add(v.work);observations[index].validation=Some(v);
    }
    let initial=observations[0].policy.clone();let incumbent=observations[0].validation.as_ref().ok_or(Error::InvalidData)?;
    let mut best=0;
    for (i,o) in observations.iter().enumerate() {
        let Some(s)=&o.validation else {continue};
        // Every previously solved internal-validation case must be retained.
        if !incumbent.verified_cases.iter().zip(&s.verified_cases).all(|(&a,&b)|!a||b){continue}
        let winner=observations[best].validation.as_ref().ok_or(Error::InvalidData)?;
        if better(s,winner){best=i;}
    }
    let proposed=observations[best].policy.clone();
    let feedback_read=[&proposed.priority,&proposed.variation,&proposed.location].iter().any(|e|{
        let mut found=false;e.map(&mut |x|{if let Expr::Input(j)=x{if (10..FEATURES).contains(j){found=true}}None});found
    });
    Ok(StudyReport{initial,policy_changed:best!=0,proposed,adopted_by_rsp2:false,feedback_read,training_cases:train.len(),
        validation_cases:validation.len(),candidate_count:observations.len(),observations,total_metered_work:cost,
        training_commitment:commitment(train),validation_commitment:commitment(validation),config})
}
