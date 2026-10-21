/// The amount charged cannot depend on Ok versus Err. A metered computation
/// reports the work completed, bounded by the reservation on all exits.
fn settle_reserved_work(account:&mut u64,reserved:u64,spent:u64)->UResult<()> {
    if spent>reserved{return Err(UError::InvalidData)}
    *account=account.checked_sub(reserved-spent).ok_or(UError::InvalidData)?;
    Ok(())
}

// No new organism or external-action commander. These are cognitive operations
// in the already existing ScientificRuntime. The host supplies labelled lessons;
// candidate policy code has no permission to evaluate hidden tasks or do I/O.
impl ScientificRuntime {
    pub fn enable_recursive_search(&mut self,config:crate::construction::search_policy::EvolutionConfig)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked);}
        u.current.learner.enable_recursive_search(config)
    }
    pub fn improve_recursive_search(&mut self,training:&[crate::construction::search_policy::SynthesisTask],gate:&crate::construction::search_policy::AdmissionSuite,seed:u64)->UResult<crate::construction::search_policy::PolicyAudit>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || u.development.is_some(){return Err(UError::Blocked);}
        let before=u.current.learner.report().construction_work;
        let out=u.current.learner.improve_recursive_search(training,gate,seed);
        u.computation_work=u.computation_work.saturating_add(u.current.learner.report().construction_work-before);out
    }
    pub fn solve_by_recursive_search(&mut self,task:&crate::construction::search_policy::SynthesisTask,limits:&crate::construction::search_policy::SearchLimits,seed:u64)->UResult<crate::construction::search_policy::SearchResult>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        limits.validate()?;task.validate()?;
        u.computation_work=u.computation_work.checked_add(limits.work).ok_or(UError::Capacity)?;
        let r=u.current.learner.solve_by_recursive_search(task,limits,seed)?;
        settle_reserved_work(&mut u.computation_work,limits.work,r.work)?;Ok(r)
    }
    pub fn freeze_recursive_search(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.development.is_some(){return Err(UError::Blocked);}
        u.current.learner.freeze_recursive_search();Ok(())
    }
}

// LC1 shares the exact same Learner and combined EvoPhase checkpoint as RSP2.
impl ScientificRuntime {
    pub fn enable_missing_computation_learning(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        u.current.learner.enable_requirements()
    }
    pub fn observe_executed_computations(&mut self,rows:&[crate::construction::requirements::ExecutedOperation])->UResult<crate::construction::requirements::RequirementReport>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        // Both values must come from the same cumulative Learner counter:
        // RequirementReport.work counts LC1 alone, while construction_work
        // also includes work performed by RSP2 and ordinary construction.
        let before=u.current.learner.report().construction_work;
        let out=u.current.learner.teach_executed_operations(rows)?;
        let after=u.current.learner.report().construction_work;
        u.computation_work=u.computation_work.saturating_add(after.saturating_sub(before));Ok(out)
    }
    pub fn requirements_report(&self)->Option<crate::construction::requirements::RequirementReport>{
        self.construction_learner()?.requirements_report()
    }
    pub fn complete_missing_computation(&mut self,
        task:&crate::construction::search_policy::SynthesisTask,
        sketch:&crate::construction::vm::Expr,
        limits:&crate::construction::requirements::CompletionLimits
    )->UResult<crate::construction::requirements::CompletionReport>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        // Both induction shape constraints AND the explicit sketch must pass
        // before reserving work in the organism's shared computation ledger.
        crate::construction::requirements::validate_completion_sketch(task,sketch,limits)?;
        // LC1 is opt-in. No acquired-requirement memory means the solver cannot
        // execute: fail before reserving or mutating the organism work ledger.
        if u.current.learner.requirements_report().is_none(){return Err(UError::NoEvidence)}
        // Reserve the entire budget before execution; only actual spent work
        // remains charged on successful return. Do not expose free failures.
        u.computation_work=u.computation_work.checked_add(limits.work).ok_or(UError::Capacity)?;
        let (result,spent)=u.current.learner.complete_missing_computation_measured(task,sketch,limits);
        // Settle even an unsuccessful attempt using its actual metered work.
        // A malformed preflight spends zero; an arithmetic failure spends only
        // the VM instructions reached before the failure.
        settle_reserved_work(&mut u.computation_work,limits.work,spent)?;
        result
    }

    pub fn discover_missing_computation(&mut self,task:&crate::construction::search_policy::SynthesisTask,
        limits:&crate::construction::requirements::CompletionLimits
    )->UResult<crate::construction::requirements::CompletionReport>{
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        limits.validate()?;crate::construction::requirements::validate_completion_task(task)?;
        // An unenabled LC1 cannot search; rejecting before budget reservation
        // preserves the joint checkpoint and existing RSP2 work accounting.
        if u.current.learner.requirements_report().is_none(){return Err(UError::NoEvidence)}
        u.computation_work=u.computation_work.checked_add(limits.work).ok_or(UError::Capacity)?;
        let (result,spent)=u.current.learner.discover_missing_computation_measured(task,limits);
        settle_reserved_work(&mut u.computation_work,limits.work,spent)?;
        result
    }
}
