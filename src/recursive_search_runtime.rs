// No new organism or external-action commander. These are cognitive operations
// in the already existing ScientificRuntime. The host supplies labelled lessons;
// candidate policy code has no permission to evaluate hidden tasks or do I/O.
impl ScientificRuntime {
    pub fn enable_recursive_search(&mut self,config:crate::construction::search_policy::EvolutionConfig)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked);}
        u.current.learner.enable_recursive_search(config)
    }
    pub fn improve_recursive_search(&mut self,training:&[crate::construction::search_policy::SynthesisTask],gate:&[crate::construction::search_policy::SynthesisTask],seed:u64)->UResult<crate::construction::search_policy::PolicyAudit>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked);}
        let before=u.current.learner.report().construction_work;
        let out=u.current.learner.improve_recursive_search(training,gate,seed);
        u.computation_work=u.computation_work.saturating_add(u.current.learner.report().construction_work-before);out
    }
    pub fn solve_by_recursive_search(&mut self,task:&crate::construction::search_policy::SynthesisTask,limits:&crate::construction::search_policy::SearchLimits,seed:u64)->UResult<crate::construction::search_policy::SearchResult>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        limits.validate()?;task.validate()?;
        u.computation_work=u.computation_work.checked_add(limits.work).ok_or(UError::Capacity)?;
        let r=u.current.learner.solve_by_recursive_search(task,limits,seed)?;
        u.computation_work-=limits.work-r.work;Ok(r)
    }
    pub fn freeze_recursive_search(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;u.current.learner.freeze_recursive_search();Ok(())
    }
}
