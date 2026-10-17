//! One explicit, bounded policy for the full autonomous construction loop.
//! This authored resource scheduler is not a synthesized learning algorithm.
use super::curriculum::Curriculum;
use super::epistemic::EvidencePolicy;
use super::runtime::Config;
use super::synthesis::SearchConfig;
use super::vm::{Error,Result};
use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutonomyPolicy {
    pub ceiling:SearchConfig,
    pub failures_per_expansion:usize,
    pub max_external_cost:u64,
    pub evidence:Option<EvidencePolicy>,
}
impl Default for AutonomyPolicy {
    fn default()->Self {Self{ceiling:SearchConfig{work:8_000_000,max_expr_nodes:11,max_frontier:32768,max_updates:64,max_state:8,deterministic:false},failures_per_expansion:2,max_external_cost:1_000_000,evidence:Some(EvidencePolicy::default())}}
}
impl AutonomyPolicy {
    pub fn validate(&self,config:&Config)->Result<()> {
        config.validate()?;self.ceiling.validate()?;
        let base=&config.search;
        if self.ceiling.work<base.work || self.ceiling.max_expr_nodes<base.max_expr_nodes
            || self.ceiling.max_frontier<base.max_frontier || self.ceiling.max_updates<base.max_updates
            || self.ceiling.max_state<base.max_state || self.ceiling.deterministic!=base.deterministic
            || !(1..=64).contains(&self.failures_per_expansion) || self.max_external_cost==0 {return Err(Error::InvalidData);}
        match &self.evidence {
            Some(e) if !base.deterministic=>e.validate(config.width)?,
            None if base.deterministic=>{},
            _=>return Err(Error::InvalidData),
        }
        Ok(())
    }
    pub fn level(&self,curriculum:&Curriculum,target:usize)->usize {
        curriculum.history.iter().filter(|a|a.target==target && !a.accepted).count()/self.failures_per_expansion
    }
    pub fn limits(&self,base:&SearchConfig,level:usize)->SearchConfig {
        let factor=1_u64.checked_shl(level.min(63) as u32).unwrap_or(u64::MAX);
        let mut c=base.clone();
        c.work=base.work.saturating_mul(factor).min(self.ceiling.work);
        c.max_expr_nodes=base.max_expr_nodes.saturating_add(level.saturating_mul(2)).min(self.ceiling.max_expr_nodes);
        c.max_frontier=base.max_frontier.saturating_mul(factor as usize).min(self.ceiling.max_frontier);
        c.max_updates=base.max_updates.saturating_mul(factor as usize).min(self.ceiling.max_updates);
        c.max_state=base.max_state.saturating_add(level).min(self.ceiling.max_state);c
    }
}
/// Availability/price quote, never authority to perform a physical action.
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionOffer {pub available:bool,pub max_cost:u64}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AutonomyReport {pub levels:Vec<usize>,pub effective_limits:Vec<SearchConfig>,pub external_cost:u64,pub remaining_cost:u64,pub frozen_at:Option<u64>,pub events:usize}
#[cfg(test)] mod tests {
    use super::*;use super::super::curriculum::SelectionMode;
    #[test] fn widening_uses_failures_not_target_number_or_training_fit() {
        let p=AutonomyPolicy::default();let mut c=Curriculum::new(2,SelectionMode::Fair).unwrap();
        c.select(&[1.0,0.0],1,0).unwrap();c.finish(true,0.5,100).unwrap();assert_eq!(p.level(&c,0),0);
        for i in 0..2 {c.select(&[0.0,1.0],2+i,100).unwrap();c.finish(false,0.0,200).unwrap();}
        assert_eq!(p.level(&c,0),0);assert_eq!(p.level(&c,1),1);
    }
    #[test] fn expansion_never_changes_contract_or_exceeds_declared_ceiling() {
        let p=AutonomyPolicy::default();let b=SearchConfig{work:100,max_expr_nodes:1,max_state:0,..Default::default()};
        for level in 0..256 {let c=p.limits(&b,level);c.validate().unwrap();assert!(c.work<=p.ceiling.work && c.max_state<=p.ceiling.max_state);assert_eq!(c.deterministic,b.deterministic);}
    }
    #[test] fn deterministic_and_stochastic_contracts_cannot_be_swapped() {
        let mut p=AutonomyPolicy::default();let mut c=Config::default();p.validate(&c).unwrap();
        c.search.deterministic=true;assert!(p.validate(&c).is_err());p.ceiling.deterministic=true;assert!(p.validate(&c).is_err());
        p.evidence=None;p.validate(&c).unwrap();p.failures_per_expansion=0;assert!(p.validate(&c).is_err());
    }
}
