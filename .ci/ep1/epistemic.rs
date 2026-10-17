//! EP1: persistent prospective evidence for revising a noisy prediction.
//!
//! A fixed mixture of products (1 + bet * paired_gain), gain in [-1, 1].
//! Under E[gain_t | PRE information] <= 0 each product is a nonnegative
//! supermartingale. Alpha spending bounds false crossings of these nulls;
//! it does NOT certify safety, distribution-free future accuracy, or AGI.
//! The runtime commits both predictions before POST. This module has no action
//! authority. All non-target computations and declared hard outputs are retained.
use super::synthesis::Transition;
use super::vm::{fingerprint, Error, Library, Program, Result};
use serde::{Deserialize, Serialize};

const BETS: [f64; 4] = [0.0625, 0.125, 0.25, 0.5];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidencePolicy {
    pub total_alpha: f64,
    pub max_samples: usize,
    /// These existing output computations may not be revised by EP1.
    pub protected_outputs: Vec<usize>,
}
impl Default for EvidencePolicy {
    fn default() -> Self {
        Self { total_alpha: 0.01, max_samples: 1024, protected_outputs: vec![] }
    }
}
impl EvidencePolicy {
    pub fn validate(&self, width: usize) -> Result<()> {
        if !(1..=32).contains(&width) || !self.total_alpha.is_finite()
            || !(0.0..1.0).contains(&self.total_alpha) || self.total_alpha == 0.0
            || !(2..=4096).contains(&self.max_samples) { return Err(Error::InvalidData); }
        let mut outputs = self.protected_outputs.clone();
        outputs.sort_unstable(); outputs.dedup();
        if outputs.len() != self.protected_outputs.len() || outputs.iter().any(|&j| j >= width) {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub target: usize, pub attempt: u64, pub alpha: f64,
    pub parent: u64, pub candidate: u64, pub first_id: u64,
    pub samples: usize, pub max_samples: usize, pub total_gain: f64,
    pub log_e: f64, pub crossed: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceOutcome { pub evidence: EvidenceSummary, pub installed: bool }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceReport {
    pub policy: EvidencePolicy, pub attempts: Vec<u64>, pub allocated_alpha: f64,
    pub pending: Option<EvidenceSummary>, pub outcomes: Vec<EvidenceOutcome>,
}

/// Durable risk debt. Cancellation, rollback and restart never refund attempts.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct RiskLedger {
    policy: EvidencePolicy, attempts: Vec<u64>, outcomes: Vec<EvidenceOutcome>,
}
impl RiskLedger {
    pub(super) fn new(policy: EvidencePolicy, width: usize) -> Result<Self> {
        policy.validate(width)?;
        Ok(Self { policy, attempts: vec![0; width], outcomes: vec![] })
    }
    fn alpha(&self, target: usize, attempt: u64) -> Result<f64> {
        if target >= self.attempts.len() || attempt == 0 || attempt > 256 { return Err(Error::InvalidData); }
        Ok(self.policy.total_alpha / self.attempts.len() as f64 / attempt as f64 / (attempt + 1) as f64)
    }
    pub(super) fn validate(&self, width: usize) -> Result<()> {
        self.policy.validate(width)?;
        if self.attempts.len() != width || self.attempts.iter().any(|&n| n > 256)
            || self.outcomes.len() > 128 { return Err(Error::InvalidData); }
        let mut seen = std::collections::BTreeSet::new();
        for outcome in &self.outcomes {
            let e = &outcome.evidence;
            if !seen.insert((e.target,e.attempt)) || e.target >= width || e.attempt > self.attempts[e.target]
                || e.alpha.to_bits() != self.alpha(e.target,e.attempt)?.to_bits()
                || e.samples > e.max_samples || e.max_samples > self.policy.max_samples
                || !e.total_gain.is_finite() || !e.log_e.is_finite()
                || (outcome.installed && (!e.crossed || e.samples < 2)) { return Err(Error::InvalidData); }
        }
        Ok(())
    }
    pub(super) fn footprint(&self, parent: &Program, candidate: &Program, target: usize) -> bool {
        target < parent.outputs.len() && parent.outputs.len() == candidate.outputs.len()
            && !self.policy.protected_outputs.contains(&target)
            && candidate.cells.starts_with(&parent.cells)
            && parent.outputs.iter().zip(&candidate.outputs).enumerate().all(|(j,(a,b))| j == target || a == b)
    }
    pub(super) fn allocate(&mut self, parent: &Program, candidate: &Program, library: &Library,
                          target: usize, first_id: u64, available: usize) -> Result<EvidenceTrial> {
        if !self.footprint(parent,candidate,target) || parent == candidate { return Err(Error::NoEvidence); }
        if self.outcomes.len() >= 128 || available < 2 { return Err(Error::Capacity); }
        let previous = *self.attempts.get(target).ok_or(Error::Index)?;
        let attempt = previous.checked_add(1).ok_or(Error::Capacity)?;
        let alpha = self.alpha(target,attempt)?;
        let library = library_stamp(library)?;
        self.attempts[target] = attempt;
        Ok(EvidenceTrial { target,attempt,alpha,parent: parent.fingerprint(),candidate: candidate.fingerprint(),library,
            first_id,max_samples: self.policy.max_samples.min(available),samples: 0,wealth: [0.0;4],total_gain: 0.0,crossed: false })
    }
    pub(super) fn record(&mut self, trial: &EvidenceTrial, installed: bool) -> Result<()> {
        self.validate_ticket(trial)?;
        if self.outcomes.len() >= 128 || self.outcomes.iter().any(|x| x.evidence.target == trial.target && x.evidence.attempt == trial.attempt) {
            return Err(Error::Stale);
        }
        if installed && !trial.accepted() { return Err(Error::NoEvidence); }
        self.outcomes.push(EvidenceOutcome { evidence: trial.summary(), installed }); Ok(())
    }
    fn validate_ticket(&self, trial: &EvidenceTrial) -> Result<()> {
        if self.attempts.get(trial.target) != Some(&trial.attempt)
            || trial.alpha.to_bits() != self.alpha(trial.target,trial.attempt)?.to_bits()
            || trial.max_samples < 2 || trial.max_samples > self.policy.max_samples
            || self.outcomes.iter().any(|o| o.evidence.target == trial.target && o.evidence.attempt == trial.attempt) {
            return Err(Error::Stale);
        }
        Ok(())
    }
    pub(super) fn report(&self, pending: Option<&EvidenceTrial>) -> EvidenceReport {
        // Sum_{k=1..n} 1/[k(k+1)] = n/(n+1). No floating-point refund ledger.
        let allocated_alpha = self.attempts.iter().map(|&n| self.policy.total_alpha / self.attempts.len() as f64
            * n as f64 / (n + 1) as f64).sum();
        EvidenceReport { policy: self.policy.clone(), attempts: self.attempts.clone(), allocated_alpha,
            pending: pending.map(EvidenceTrial::summary), outcomes: self.outcomes.clone() }
    }
}
fn library_stamp(library: &Library) -> Result<u64> {
    Ok(fingerprint(&serde_json::to_vec(library).map_err(|_| Error::InvalidData)?))
}

/// Frozen question; only the parent runtime can supply actual PRE commitments.
/// Serialized inside the same atomic cognitive checkpoint as its risk ledger.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct EvidenceTrial {
    target: usize, attempt: u64, alpha: f64, parent: u64, candidate: u64, library: u64,
    first_id: u64, max_samples: usize, samples: usize, wealth: [f64;4], total_gain: f64, crossed: bool,
}
impl EvidenceTrial {
    pub(super) fn log_e(&self) -> f64 {
        let high = self.wealth.iter().copied().fold(f64::NEG_INFINITY,f64::max);
        high + (self.wealth.iter().map(|x| (x-high).exp()).sum::<f64>() / BETS.len() as f64).ln()
    }
    pub(super) fn accepted(&self) -> bool { self.crossed && self.samples >= 2 }
    pub(super) fn ready(&self) -> bool { self.accepted() || self.samples >= self.max_samples }
    pub(super) fn samples(&self) -> usize { self.samples }
    pub(super) fn summary(&self) -> EvidenceSummary {
        EvidenceSummary { target:self.target,attempt:self.attempt,alpha:self.alpha,parent:self.parent,candidate:self.candidate,
            first_id:self.first_id,samples:self.samples,max_samples:self.max_samples,total_gain:self.total_gain,
            log_e:self.log_e(),crossed:self.crossed }
    }
    pub(super) fn binding(&self, ledger: &RiskLedger, parent: &Program, candidate: &Program,
                         library: &Library, target: usize, first_id: u64) -> Result<()> {
        ledger.validate_ticket(self)?;
        if self.parent != parent.fingerprint() || self.candidate != candidate.fingerprint()
            || self.library != library_stamp(library)? || self.target != target || self.first_id != first_id
            || !ledger.footprint(parent,candidate,target) { return Err(Error::Stale); }
        Ok(())
    }
    pub(super) fn observe(&mut self, id: u64, old: &[i64], new: &[i64], actual: &[i64]) -> Result<()> {
        if self.ready() || id != self.first_id.checked_add(self.samples as u64).ok_or(Error::Capacity)? {
            return Err(Error::Stale);
        }
        if old.len() != actual.len() || new.len() != actual.len() || self.target >= actual.len() { return Err(Error::InvalidData); }
        let gain = usize::from(old[self.target] != actual[self.target]) as f64 - usize::from(new[self.target] != actual[self.target]) as f64;
        for (w,bet) in self.wealth.iter_mut().zip(BETS) { *w += (bet*gain).ln_1p(); }
        self.samples += 1; self.total_gain += gain; self.crossed |= self.log_e() >= -self.alpha.ln();
        Ok(())
    }
    /// Recompute from authenticated committed predictions or frozen VM replay.
    /// This checks persistence and binds a registry audit to the same evidence.
    pub(super) fn verify_replay(&self, old: &[Vec<i64>], new: &[Vec<i64>], future: &[Transition]) -> Result<()> {
        if old.len() != future.len() || new.len() != future.len() || future.len() != self.samples { return Err(Error::InvalidData); }
        let mut copy = Self { target:self.target,attempt:self.attempt,alpha:self.alpha,parent:self.parent,candidate:self.candidate,
            library:self.library,first_id:self.first_id,max_samples:self.max_samples,samples:0,wealth:[0.0;4],total_gain:0.0,crossed:false };
        for ((a,b),t) in old.iter().zip(new).zip(future) { copy.observe(t.id,a,b,&t.post)?; }
        if copy.wealth.iter().zip(self.wealth).any(|(a,b)| a.to_bits() != b.to_bits())
            || copy.total_gain.to_bits() != self.total_gain.to_bits() || copy.crossed != self.crossed { return Err(Error::InvalidData); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vm::{Expr,StateCell};
    fn pair() -> (Program,Program,Library) {
        let p=Program::identity(1); let mut q=p.clone(); q.outputs[0]=Expr::integer(1);
        (p,q,Library::default())
    }
    fn trial(max:usize) -> (RiskLedger,EvidenceTrial) {
        let (p,q,l)=pair(); let mut risk=RiskLedger::new(EvidencePolicy{max_samples:max,..Default::default()},1).unwrap();
        let e=risk.allocate(&p,&q,&l,0,10,max).unwrap(); (risk,e)
    }
    #[test] fn invalid_policy_rejected() {
        for alpha in [0.0,-0.1,1.0,f64::NAN,f64::INFINITY] { assert!(RiskLedger::new(EvidencePolicy{total_alpha:alpha,..Default::default()},1).is_err()); }
        assert!(RiskLedger::new(EvidencePolicy::default(),0).is_err());
        assert!(RiskLedger::new(EvidencePolicy{protected_outputs:vec![0,0],..Default::default()},1).is_err());
    }
    #[test] fn twenty_sample_timeout_is_inconclusive_not_rejection_of_world() {
        let (_,mut e)=trial(20); for i in 10..30 { e.observe(i,&[0],&[0],&[0]).unwrap(); }
        assert!(e.ready()); assert!(!e.accepted()); assert_eq!(e.log_e(),0.0);
    }
    #[test] fn noisy_superior_model_can_cross_despite_old_lucky_observation() {
        let (_,mut e)=trial(512);
        for i in 0..512 { let y=if i%5==0 {0} else {1}; e.observe(10+i,&[0],&[1],&[y]).unwrap(); if e.ready() {break;} }
        assert!(e.accepted()); assert!(e.samples()>6);
    }
    #[test] fn null_and_inferior_candidates_do_not_cross_on_balanced_control() {
        for bad in [false,true] { let (_,mut e)=trial(1000);
            for i in 0..1000 { let y=if bad || i%2==0 {0} else {1};e.observe(10+i,&[0],&[1],&[y]).unwrap(); }
            assert!(!e.accepted());
        }
    }
    #[test] fn repeated_or_skipped_events_are_not_evidence() {
        let (_,mut e)=trial(64); assert!(e.observe(9,&[0],&[1],&[1]).is_err());
        assert!(e.observe(11,&[0],&[1],&[1]).is_err());e.observe(10,&[0],&[1],&[1]).unwrap();
        assert!(e.observe(10,&[0],&[1],&[1]).is_err());assert_eq!(e.samples(),1);
    }
    #[test] fn shape_failure_does_not_spend_an_observation() {
        let (_,mut e)=trial(64);assert!(e.observe(10,&[],&[1],&[1]).is_err());assert_eq!(e.samples(),0);
    }
    #[test] fn repeated_inspection_is_inert() {
        let (_,mut e)=trial(512); e.observe(10,&[0],&[1],&[1]).unwrap();let bytes=serde_json::to_vec(&e).unwrap();
        for _ in 0..1000 { let _=e.log_e();let _=e.ready();let _=e.summary(); }
        assert_eq!(bytes,serde_json::to_vec(&e).unwrap());
    }
    #[test] fn spending_is_summable_and_cancellation_does_not_refund() {
        let (p,q,l)=pair();let mut r=RiskLedger::new(EvidencePolicy::default(),1).unwrap();let mut sum=0.0;
        for _ in 0..128 { let e=r.allocate(&p,&q,&l,0,10,512).unwrap();sum+=e.alpha; }
        assert!(sum<0.01);assert!((sum-r.report(None).allocated_alpha).abs()<1e-14);
        assert_eq!(r.report(None).attempts,[128]);
    }
    #[test] fn stale_ticket_cannot_be_reused() {
        let (mut r,e)=trial(64);let (p,q,l)=pair();let _=r.allocate(&p,&q,&l,0,12,64).unwrap();
        assert!(e.binding(&r,&p,&q,&l,0,10).is_err());
    }
    #[test] fn changed_program_or_library_cannot_use_old_evidence() {
        let (r,e)=trial(64);let (p,mut q,mut l)=pair();q.outputs[0]=Expr::integer(2);
        assert!(e.binding(&r,&p,&q,&l,0,10).is_err());
        l.functions.push(super::super::vm::Function { id:4,arity:0,body:Expr::integer(4) });
        assert!(e.binding(&r,&p,&pair().1,&l,0,10).is_err());
    }
    #[test] fn protected_outputs_and_existing_state_updates_cannot_change() {
        let p=Program::identity(2);let mut q=p.clone();q.outputs[1]=Expr::integer(1);
        let r=RiskLedger::new(EvidencePolicy{protected_outputs:vec![1],..Default::default()},2).unwrap();
        assert!(!r.footprint(&p,&q,1));assert!(!r.footprint(&p,&q,0));
        let mut p=Program::identity(1);p.cells.push(StateCell{initial:0,update:Expr::Action});let mut q=p.clone();q.cells[0].update=Expr::integer(0);
        assert!(!RiskLedger::new(EvidencePolicy::default(),1).unwrap().footprint(&p,&q,0));
    }
    #[test] fn evidence_and_risk_roundtrip_bit_exactly() {
        let (r,mut e)=trial(512);for i in 10..43 {e.observe(i,&[0],&[1],&[(i%2) as i64]).unwrap();}
        let bytes=serde_json::to_vec(&(r,e)).unwrap();let (r,e):(RiskLedger,EvidenceTrial)=serde_json::from_slice(&bytes).unwrap();
        r.validate(1).unwrap();assert_eq!(bytes,serde_json::to_vec(&(r,e)).unwrap());
    }
    #[test] fn replay_rejects_modified_future() {
        let (_,mut e)=trial(64); e.observe(10,&[0],&[1],&[1]).unwrap();
        let t=Transition{id:10,restart:false,pre:vec![0],action:0,post:vec![0]};
        assert!(e.verify_replay(&[vec![0]],&[vec![1]],&[t]).is_err());
    }
    #[test] fn installed_outcome_requires_crossing_and_is_single_use() {
        let (mut r,mut e)=trial(128); assert!(r.record(&e,true).is_err());
        for i in 10..138 { e.observe(i,&[0],&[1],&[1]).unwrap();if e.ready(){break;} }
        r.record(&e,true).unwrap();assert!(r.record(&e,true).is_err());r.validate(1).unwrap();
    }
}
