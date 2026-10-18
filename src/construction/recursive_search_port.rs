// Included by Learner. Preserves the existing owner and shadow-trial gates.
impl Learner {
    pub fn enable_recursive_search(&mut self,cfg:super::search_policy::EvolutionConfig)->Result<()> {
        if self.pending||self.shadow.is_some()||self.poisoned||self.frozen_at.is_some()||self.registry.recursive_search.is_some(){return Err(Error::Busy);}
        self.registry.recursive_search=Some(super::search_policy::RecursiveSearch::new(cfg)?);Ok(())
    }
    pub fn recursive_search(&self)->Option<&super::search_policy::RecursiveSearch>{self.registry.recursive_search.as_ref()}
    pub(crate) fn improve_recursive_search(&mut self,training:&[super::search_policy::SynthesisTask],gate:&[super::search_policy::SynthesisTask],seed:u64)->Result<super::search_policy::PolicyAudit>{
        if self.pending||self.shadow.is_some()||self.poisoned||self.frozen_at.is_some(){return Err(Error::Busy);}
        let state=self.registry.recursive_search.as_mut().ok_or(Error::NoEvidence)?;
        let before=state.work();let result=state.improve(training,gate,seed).cloned();
        self.construction_work=self.construction_work.saturating_add(state.work()-before);result
    }
    pub(crate) fn freeze_recursive_search(&mut self){if let Some(s)=&mut self.registry.recursive_search{s.freeze();}}
    pub(crate) fn solve_by_recursive_search(&self,task:&super::search_policy::SynthesisTask,limits:&super::search_policy::SearchLimits,seed:u64)->Result<super::search_policy::SearchResult>{
        task.validate()?;
        let state=self.registry.recursive_search.as_ref().ok_or(Error::NoEvidence)?;
        super::search_policy::search(state.policy(),Expr::integer(0),&mut task.clone(),limits,seed)
    }
}
