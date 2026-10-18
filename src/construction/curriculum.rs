//! Self-selection of unresolved prediction obligations, in a single lifetime.
//! Selection sees residuals and PAST prospective outcomes, never environment
//! identities, target programs or future labels. This is authored scheduling,
//! not a claim that the scheduler itself has been synthesized.
use super::vm::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode { Progress, Fair, LargestResidual }
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Opportunity {
    pub attempts: u64, pub accepted: u64, pub verified_gain: f64, pub paid_work: u64,
    pub last_turn: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub target: usize, pub evidence_end: u64, pub work_start: u64,
    pub accepted: bool, pub verified_gain: f64, pub paid_work: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curriculum {
    pub mode: SelectionMode,
    pub opportunities: Vec<Opportunity>,
    pub history: Vec<Attempt>,
    pending: Option<Attempt>,
}
impl Curriculum {
    pub fn new(width: usize,mode: SelectionMode) -> Result<Self> {
        if !(1..=32).contains(&width) { return Err(Error::Capacity); }
        Ok(Self { mode,opportunities: vec![Opportunity::default();width],history: vec![],pending: None })
    }
    pub fn pending_target(&self) -> Option<usize> { self.pending.as_ref().map(|a| a.target) }
    /// Every fourth selection reserves fair access. Failed search never becomes
    /// evidence of impossibility. Once-resolved residuals may reopen naturally.
    pub fn select(&mut self,errors: &[f64],evidence_end: u64,work_start: u64) -> Result<Option<usize>> {
        if self.pending.is_some() { return Err(Error::Busy); }
        if self.history.len() >= 256 { return Err(Error::Capacity); }
        if errors.len() != self.opportunities.len() || errors.iter().any(|x| !x.is_finite() || *x < 0.0) { return Err(Error::InvalidData); }
        let available: Vec<_> = (0..errors.len()).filter(|&j| errors[j] > 0.0).collect();
        if available.is_empty() { return Ok(None); }
        let fair = || *available.iter().min_by_key(|&&j| (self.opportunities[j].attempts,self.opportunities[j].last_turn,j)).unwrap();
        let target = match self.mode {
            SelectionMode::LargestResidual => *available.iter().max_by(|&&a,&&b| errors[a].total_cmp(&errors[b]).then_with(|| b.cmp(&a))).unwrap(),
            SelectionMode::Fair => fair(),
            SelectionMode::Progress => {
                if self.history.len() % 4 == 0 || available.iter().any(|&j| self.opportunities[j].attempts == 0) { fair() }
                else {
                    let score = |j: usize| {
                        let o = &self.opportunities[j];
                        let probability = (1.0+o.verified_gain)/(2.0+o.attempts as f64);
                        let cost = 1.0+o.paid_work as f64/(o.attempts.max(1) as f64*1_000_000.0);
                        probability/cost
                    };
                    *available.iter().max_by(|&&a,&&b| score(a).total_cmp(&score(b)).then_with(|| b.cmp(&a))).unwrap()
                }
            }
        };
        self.opportunities[target].attempts += 1;
        self.opportunities[target].last_turn = self.history.len() as u64+1;
        self.pending = Some(Attempt { target,evidence_end,work_start,accepted: false,verified_gain: 0.0,paid_work: 0 });
        Ok(Some(target))
    }
    /// A search training fit supplies zero reward. Only the final frozen audit
    /// can provide a positive, normalized paired future improvement.
    pub fn finish(&mut self,accepted: bool,gain: f64,work_end: u64) -> Result<()> {
        if !gain.is_finite() || !(0.0..=1.0).contains(&gain) || (!accepted && gain != 0.0) { return Err(Error::InvalidData); }
        let Some(mut attempt) = self.pending.take() else { return Ok(()); };
        attempt.accepted = accepted; attempt.verified_gain = gain;
        attempt.paid_work = work_end.saturating_sub(attempt.work_start);
        let o = &mut self.opportunities[attempt.target];
        o.accepted += u64::from(accepted); o.verified_gain += gain;
        o.paid_work = o.paid_work.saturating_add(attempt.paid_work);
        self.history.push(attempt); Ok(())
    }
    pub fn validate(&self,width: usize) -> Result<()> {
        if width != self.opportunities.len() || !(1..=32).contains(&width) || self.history.len() > 256 { return Err(Error::InvalidData); }
        let mut counts = vec![0u64;width]; let mut accepts = vec![0u64;width];
        let mut gain = vec![0.0;width]; let mut work = vec![0u64;width];
        for a in &self.history {
            if a.target >= width || !a.verified_gain.is_finite() || !(0.0..=1.0).contains(&a.verified_gain) || (!a.accepted && a.verified_gain != 0.0) { return Err(Error::InvalidData); }
            counts[a.target] += 1; accepts[a.target] += u64::from(a.accepted);
            gain[a.target] += a.verified_gain; work[a.target] = work[a.target].saturating_add(a.paid_work);
        }
        if let Some(a) = &self.pending {
            if a.target >= width || a.accepted || a.verified_gain != 0.0 || a.paid_work != 0 { return Err(Error::InvalidData); }
            counts[a.target] += 1;
        }
        for (j,o) in self.opportunities.iter().enumerate() {
            if o.attempts != counts[j] || o.accepted != accepts[j] || o.verified_gain.to_bits() != gain[j].to_bits() || o.paid_work != work[j] || o.last_turn > self.history.len() as u64+1 { return Err(Error::InvalidData); }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn arbitrarily_large_distractor_does_not_starve_other_residuals() {
        let mut c = Curriculum::new(3,SelectionMode::Progress).unwrap();
        for t in 0..24 { c.select(&[1e18,1.0,2.0],t,0).unwrap(); c.finish(false,0.0,1000).unwrap(); }
        assert!(c.opportunities.iter().all(|o| o.attempts >= 2)); c.validate(3).unwrap();
    }
    #[test] fn largest_residual_control_exhibits_starvation() {
        let mut c = Curriculum::new(3,SelectionMode::LargestResidual).unwrap();
        for t in 0..24 { assert_eq!(c.select(&[1e18,1.0,2.0],t,0).unwrap(),Some(0)); c.finish(false,0.0,1000).unwrap(); }
        assert_eq!(c.opportunities[1].attempts,0);
    }
    #[test] fn solved_targets_are_not_scheduled_and_can_reopen() {
        let mut c = Curriculum::new(2,SelectionMode::Progress).unwrap();
        assert_eq!(c.select(&[0.0,3.0],4,0).unwrap(),Some(1)); c.finish(true,0.8,100).unwrap();
        assert_eq!(c.select(&[0.0,0.0],8,100).unwrap(),None);
        assert_eq!(c.select(&[2.0,0.0],12,100).unwrap(),Some(0));
    }
    #[test] fn rejected_fit_cannot_buy_positive_priority() {
        let mut c = Curriculum::new(1,SelectionMode::Progress).unwrap(); c.select(&[1.0],1,0).unwrap();
        assert_eq!(c.finish(false,1.0,100),Err(Error::InvalidData));
        assert_eq!(c.pending_target(),Some(0)); c.finish(false,0.0,100).unwrap(); c.validate(1).unwrap();
    }
    #[test] fn pending_selection_round_trips_and_history_is_checked() {
        let mut c = Curriculum::new(2,SelectionMode::Progress).unwrap(); c.select(&[1.0,2.0],4,3).unwrap();
        let b = serde_json::to_vec(&c).unwrap(); let mut d: Curriculum = serde_json::from_slice(&b).unwrap();
        d.validate(2).unwrap(); d.finish(true,0.25,103).unwrap(); d.validate(2).unwrap();
        d.opportunities[0].accepted += 1; assert!(d.validate(2).is_err());
    }
}
