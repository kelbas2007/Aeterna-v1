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
    pub fn answer_school(&self,question:&str,mode:crate::education::AnswerMode)->UResult<crate::education::SchoolAnswer> {
        use crate::education::SchoolAnswer;
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let Some(school)=&u.education else{return Ok(SchoolAnswer::unknown("no_teacher_experience"));};
        let mut candidates=school.propose(question,mode.clone())?;
        if candidates.is_empty() && mode!=crate::education::AnswerMode::RecallOnly {candidates=school.lexical_compute(question)?;}
        if candidates.is_empty() && mode!=crate::education::AnswerMode::ComputeOnly {candidates=school.related(question)?;}
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
