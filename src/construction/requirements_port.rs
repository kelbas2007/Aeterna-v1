// An acquired computational skill, not a separate action authority or organism.
impl Learner {
    pub fn enable_requirements(&mut self)->Result<()> {
        if self.pending||self.poisoned||self.shadow.is_some()||self.frozen_at.is_some()||self.requirements.is_some(){return Err(Error::Busy)}
        self.requirements=Some(super::requirements::RequirementMemory::default());Ok(())
    }
    pub fn requirements_report(&self)->Option<super::requirements::RequirementReport>{
        self.requirements.as_ref().map(|b|b.report())
    }
    pub fn teach_executed_operations(&mut self,observations:&[super::requirements::ExecutedOperation])->Result<super::requirements::RequirementReport>{
        if self.pending||self.poisoned||self.shadow.is_some()||self.frozen_at.is_some(){return Err(Error::Busy)}
        let b=self.requirements.as_mut().ok_or(Error::NoEvidence)?;
        let before=b.report().work;
        let out=b.observe(observations)?;
        self.construction_work=self.construction_work.saturating_add(out.work-before);
        Ok(out)
    }
    pub fn discover_missing_computation(&self,task:&super::search_policy::SynthesisTask,limits:&super::requirements::CompletionLimits)->Result<super::requirements::CompletionReport>{
        self.requirements.as_ref().ok_or(Error::NoEvidence)?.discover(task,limits)
    }
    pub fn complete_missing_computation(&self,task:&super::search_policy::SynthesisTask,
        sketch:&Expr,limits:&super::requirements::CompletionLimits)->Result<super::requirements::CompletionReport>{
        let b=self.requirements.as_ref().ok_or(Error::NoEvidence)?;
        b.complete(task,sketch,limits)
    }
}
