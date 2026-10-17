// Included into ScientificRuntime: no second executable decision owner.
impl ScientificRuntime {
    pub fn teach_school(&mut self,examples:Vec<crate::education::TeachingExample>)->UResult<crate::education::LessonReport> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || self.protection.emergency_stop_latched(){return Err(UError::Busy);}
        let school=u.education.get_or_insert_with(Default::default);
        let report=school.teach(examples)?;
        u.computation_work=u.computation_work.saturating_add(report.work);
        // Actual held-out TEACHER feedback, never the later examination labels.
        // Counted as teaching feedback rather than a physical environment action.
        if report.expression.is_some() {
            for correct in &report.validation_outcomes {
                self.organism.observe_phase_native_meta_utility([0.5,0.0,0.0,0.95,0.5],if *correct{1.0}else{0.0});
            }
        }
        Ok(report)
    }
    pub fn teach_school_facts(&mut self,facts:Vec<crate::education::Fact>)->UResult<usize> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || self.protection.emergency_stop_latched(){return Err(UError::Busy);}
        u.education.get_or_insert_with(Default::default).teach_facts(facts)
    }
    pub fn school_report(&self)->Option<crate::education::SchoolReport>{Some(self.constructing.as_ref()?.education.as_ref()?.report())}
    pub fn school_lessons(&self)->Option<&[crate::education::LessonReport]>{Some(self.constructing.as_ref()?.education.as_ref()?.lessons())}
    pub fn enable_school_composition(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller || self.protection.emergency_stop_latched(){return Err(UError::Busy);}
        u.education.get_or_insert_with(Default::default).enable_composition()
    }
    pub fn understand_school(&self,question:&str,limits:crate::education::GroundingLimits)->UResult<crate::education::Understanding> {
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        u.education.as_ref().ok_or(UError::NoEvidence)?.understand(question,limits)
    }
    pub fn answer_school(&self,question:&str,mode:crate::education::AnswerMode)->UResult<crate::education::SchoolAnswer> {
        self.answer_school_impl(question,mode,false)
    }
    /// Matched ablation: same acquired programs and lessons, only composition disabled.
    pub fn answer_school_surface(&self,question:&str,mode:crate::education::AnswerMode)->UResult<crate::education::SchoolAnswer> {
        self.answer_school_impl(question,mode,true)
    }
    fn answer_school_impl(&self,question:&str,mode:crate::education::AnswerMode,surface:bool)->UResult<crate::education::SchoolAnswer> {
        use crate::education::SchoolAnswer;
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let Some(school)=&u.education else{return Ok(SchoolAnswer::unknown("no_teacher_experience"));};
        let mut candidates=school.propose(question,mode.clone())?;
        if let Some(c)=candidates.iter().find(|c|c.answer.answer.is_none()){return Ok(c.answer.clone());}
        if school.composition_enabled() && !surface && mode!=crate::education::AnswerMode::RecallOnly {
            // Never allow an approximate surface answer to override ambiguity,
            // partial coverage, an arithmetic error, or an exhausted parse search.
            let interpreted=school.composed_proposals(question)?;
            let reason=&interpreted[0].answer.reason;
            let no_grounding=reason=="unlearned_language"||reason=="missing_arguments_or_unlearned_structure";
            // Exact attributed factual recall remains available; lexical guesses do not.
            if !no_grounding || candidates.is_empty() || candidates.iter().any(|c|c.answer.method!="teacher_recall") {
                candidates=interpreted;
            }
        } else {
            if candidates.is_empty() && mode!=crate::education::AnswerMode::RecallOnly {candidates=school.lexical_compute(question)?;}
            if candidates.is_empty() && mode!=crate::education::AnswerMode::ComputeOnly {candidates=school.related(question)?;}
        }
        if candidates.is_empty(){return Ok(SchoolAnswer::unknown("no_learned_grounding"));}
        let first=&candidates[0].answer.answer;
        if first.is_none(){return Ok(candidates.remove(0).answer);}
        if candidates.iter().any(|c|c.answer.answer!=*first){return Ok(SchoolAnswer::unknown("conflicting_acquired_answers"));}
        // Epistemic gates cannot be overridden by U1. Among supported answers,
        // the same organism readout selects the evidence-bearing proposal.
        let proposals:Vec<_>=candidates.iter().enumerate().map(|(i,c)|PhaseCognitiveProposal{proposal_id:i as u64,action:0,fields:c.fields}).collect();
        let choice=self.organism.choose_phase_meta_exploratory(&proposals,uhash(question.as_bytes()),false).ok_or(UError::NoEvidence)?;
        let mut answer=candidates.remove(choice.proposal_id as usize).answer;answer.controller_score=Some(choice.score);Ok(answer)
    }
}
impl ScientificRuntime {
    pub fn clarify_school(&self,question:&str,reply:i64)->UResult<crate::education::SchoolAnswer> {
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let school=u.education.as_ref().ok_or(UError::NoEvidence)?;
        if !school.composition_enabled(){return Err(UError::NoEvidence);}
        for p in school.propose(question,crate::education::AnswerMode::All)? {
            if p.answer.answer.is_none(){return Ok(p.answer);}
        }
        let mut answer=school.clarify(question,reply)?;
        if answer.answer.is_some(){
            let p=PhaseCognitiveProposal{proposal_id:0,action:0,fields:[0.5,0.0,0.0,0.95,0.5]};
            let c=self.organism.choose_phase_meta_exploratory(&[p],uhash(question.as_bytes()),false).ok_or(UError::NoEvidence)?;
            answer.controller_score=Some(c.score);
        }
        Ok(answer)
    }
}

impl ScientificRuntime {
    /// Explicit teacher-opened epoch. Preserves programs, controller weights,
    /// factual history, pending world trials and risk debt; no authority reset.
    pub fn begin_education_extension(&mut self)->UResult<()> {
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.current.learner.report().poisoned{return Err(UError::Stale);}
        u.education.get_or_insert_with(Default::default).begin_extension();
        u.freeze_controller=false;
        self.organism.set_phase_native_meta_learning_enabled(true);
        Ok(())
    }
    pub fn install_word_problem_knowledge(&mut self,model:crate::word_problems::WordProblemModel)->UResult<()> {
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Busy);}
        model.validate()?; // Reject before even creating an empty School.
        u.education.get_or_insert_with(Default::default).install_word_problems(model)
    }
    pub fn answer_word_problem(&self,question:&str,prior_only:bool)->UResult<crate::word_problems::WordProblemAnswer> {
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;
        u.education.as_ref().ok_or(UError::NoEvidence)?.word_problem_answer(question,prior_only)
    }
}
