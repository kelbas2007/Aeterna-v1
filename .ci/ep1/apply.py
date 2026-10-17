from pathlib import Path
import hashlib
import shutil
root=Path.cwd()
expected = {'src/construction/mod.rs': 'ba54896b1853e4ae1689da6c525b703104c792dcf2fe1d3e043846178ae04b69', 'src/construction/runtime.rs': '6788e553c03a91b7b909a950a26d493196e0c48d368dcec9637da763a95b51fc', 'src/construction/recursive.rs': '98fdbf7a175455975c558afc4d86f0d3328560bd62de229031d0836b693cd627'}
for path,digest in expected.items():
    if hashlib.sha256((root/path).read_bytes()).hexdigest()!=digest:
        raise RuntimeError('Source changed; refusing stale integration: '+path)
for source,target in [('epistemic.rs','src/construction/epistemic.rs'),('tests.rs','tests/directed_construction_epistemic.rs'),('example.rs','examples/directed_construction_epistemic.rs')]:
    if (root/target).exists():
        raise RuntimeError('Refusing to overwrite existing file: '+target)
    shutil.copyfile(root/'.ci/ep1'/source,root/target)
p=root/'src/construction/mod.rs';p.write_text(p.read_text()+'pub mod epistemic;\n')
p=root/'src/construction/runtime.rs';s=p.read_text()
def sub(old,new,count=1):
 global s
 assert s.count(old)==count,(old[:90],s.count(old),count)
 s=s.replace(old,new)
sub('use super::curriculum::{Curriculum, SelectionMode};','use super::curriculum::{Curriculum, SelectionMode};\nuse super::epistemic::{EvidencePolicy, EvidenceReport, EvidenceTrial, RiskLedger};')
sub('    controls: Option<SearchTrial>,','    controls: Option<SearchTrial>,\n    #[serde(default, skip_serializing_if="Option::is_none")]\n    evidence: Option<EvidenceTrial>,')
sub('    curriculum: Option<Curriculum>,','    curriculum: Option<Curriculum>,\n    #[serde(default,skip_serializing_if="Option::is_none")]\n    epistemic: Option<RiskLedger>,')
sub('last_error: None,curriculum: None })','last_error: None,curriculum: None,epistemic: None })')
sub('    pub fn curriculum(&self) -> Option<&Curriculum> { self.curriculum.as_ref() }','''    pub fn curriculum(&self) -> Option<&Curriculum> { self.curriculum.as_ref() }
    /// Explicit lifetime policy, fixed before any observations. Cannot relax a
    /// deterministic contract or reset previously allocated statistical risk.
    pub fn enable_epistemic(&mut self,policy: EvidencePolicy) -> Result<()> {
        if self.pending || self.shadow.is_some() || self.epistemic.is_some() || !self.records.is_empty() || self.poisoned {
            return Err(Error::Busy);
        }
        if self.config.search.deterministic { return Err(Error::InvalidData); }
        self.epistemic = Some(RiskLedger::new(policy,self.config.width)?); Ok(())
    }
    pub fn epistemic_report(&self) -> Option<EvidenceReport> {
        self.epistemic.as_ref().map(|risk| risk.report(self.shadow.as_ref().and_then(|s| s.evidence.as_ref())))
    }''')
sub('        let mut meter = Meter::new(100000);\n        let (center,next_state)', '''        if let (Some(risk),Some(shadow)) = (&self.epistemic,&self.shadow) {
            let bound = shadow.evidence.as_ref().ok_or(Error::Stale).and_then(|e|
                e.binding(risk,&self.active.program,&shadow.model.program,&self.library,shadow.trace.job.target,shadow.start_id));
            if let Err(e) = bound { self.last_error=Some(format!("epistemic binding: {e}")); self.cancel_shadow(); }
        }
        let mut meter = Meter::new(100000);
        let (center,next_state)''')
sub('        self.active.state = token.next_state;','        let mut evidence_error = None;\n        self.active.state = token.next_state;')
sub('            shadow.model.state = state; shadow.samples += 1; shadow.incumbent_nll += incumbent_nll; shadow.candidate_nll += nll;', '''            if let Some(e) = &mut shadow.evidence {
                let update = token.candidate_prediction.as_ref().ok_or(Error::Stale)
                    .and_then(|p| e.observe(token.id,&token.prediction.center,&p.center,&post));
                if let Err(error) = update { evidence_error = Some(error); }
            }
            shadow.model.state = state; shadow.samples += 1; shadow.incumbent_nll += incumbent_nll; shadow.candidate_nll += nll;''')
sub('        if self.shadow.as_ref().is_some_and(|s| s.samples >= self.config.shadow_steps) {','''        if let Some(error) = evidence_error {
            self.last_error=Some(format!("epistemic observation: {error}")); self.cancel_shadow();
        }
        if self.shadow.as_ref().is_some_and(|s| s.evidence.as_ref().map_or(s.samples >= self.config.shadow_steps,|e| e.ready())) {''')
sub('        self.active.scale = incumbent_scale;\n        self.shadow = Some(Shadow { model: Model { program,state,scale },trace,training_len: data.len(),start_id: self.next_id,samples: 0,incumbent_nll: 0.0,candidate_nll: 0.0,controls: Some(controls) });','''        let evidence = if let Some(risk) = &mut self.epistemic {
            match risk.allocate(&self.active.program,&program,&self.library,trace.job.target,self.next_id,
                                self.config.journal_capacity-self.records.len()) {
                Ok(e) => Some(e),
                Err(e) => { self.registry.cancel_trial(controls)?; return Err(e); }
            }
        } else { None };
        self.active.scale = incumbent_scale;
        self.shadow = Some(Shadow { model: Model { program,state,scale },trace,training_len: data.len(),start_id: self.next_id,samples: 0,incumbent_nll: 0.0,candidate_nll: 0.0,controls: Some(controls),evidence });''')
sub('        let retention_ok = old.iter().zip(new.iter()).zip(training).all(|((a,b),t)| {','        let pointwise_ok = old.iter().zip(new.iter()).zip(training).all(|((a,b),t)| {')
sub('        let local_acceptance = retention_ok && shadow.candidate_nll+complexity < shadow.incumbent_nll;', '''        let (retention_ok,local_acceptance) = if let Some(evidence) = &shadow.evidence {
            let risk = self.epistemic.as_ref().ok_or(Error::Stale)?;
            evidence.binding(risk,&self.active.program,&shadow.model.program,&self.library,shadow.trace.job.target,shadow.start_id)?;
            let retained = risk.footprint(&self.active.program,&shadow.model.program,shadow.trace.job.target);
            (retained,retained && evidence.accepted())
        } else {
            if self.epistemic.is_some() { return Err(Error::Stale); }
            (pointwise_ok,pointwise_ok && shadow.candidate_nll+complexity < shadow.incumbent_nll)
        };''')
sub('            let result = self.registry.complete_trial(controls,&data[shadow.training_len..],local_acceptance);','''            let result = if let Some(evidence) = &shadow.evidence {
                match self.epistemic.as_ref() {
                    Some(risk) => self.registry.complete_trial_epistemic(controls,&data[shadow.training_len..],local_acceptance,evidence,risk),
                    None => { let _=self.registry.cancel_trial(controls); Err(Error::Stale) }
                }
            } else { self.registry.complete_trial(controls,&data[shadow.training_len..],local_acceptance) };''')
sub('        shadow.trace.prospectively_accepted = accepted;','''        if let Some(e) = &shadow.evidence {
            if let Some(risk) = &mut self.epistemic { risk.record(e,accepted)?; }
            else { return Err(Error::Stale); }
        }
        shadow.trace.prospectively_accepted = accepted;''')
sub('        if let Some(mut shadow) = self.shadow.take() {\n            if let Some(controls)', '''        if let Some(mut shadow) = self.shadow.take() {
            if let (Some(risk),Some(e)) = (&mut self.epistemic,&shadow.evidence) { let _=risk.record(e,false); }
            if let Some(controls)''')
sub('        state.config.validate()?; state.library.validate(state.config.width)?; state.registry.validate(&state.library)?;', '''        state.config.validate()?; state.library.validate(state.config.width)?; state.registry.validate(&state.library)?;
        if let Some(risk) = &state.epistemic {
            risk.validate(state.config.width)?;
            if state.config.search.deterministic { return Err(Error::InvalidData); }
        }''')
sub('|| s.samples >= state.config.shadow_steps ||', '|| (s.evidence.is_none() && s.samples >= state.config.shadow_steps) ||')
sub('            s.model.state = replay(&s.model.program,&data,&state.library,&mut meter)?.1;', '''            if let Some(e) = &s.evidence {
                let risk=state.epistemic.as_ref().ok_or(Error::InvalidData)?;
                e.binding(risk,&state.active.program,&s.model.program,&state.library,s.trace.job.target,s.start_id)?;
                if e.samples()!=s.samples || e.ready() { return Err(Error::InvalidData); }
                let future=&state.records[s.training_len..];
                if future.iter().any(|r| r.model!=state.active.program.fingerprint() || r.candidate!=Some(s.model.program.fingerprint())) { return Err(Error::Stale); }
                let old:Vec<_>=future.iter().map(|r|r.prediction.center.clone()).collect();
                let new:Vec<_>=future.iter().map(|r|r.candidate_prediction.as_ref().map(|p|p.center.clone()).ok_or(Error::Stale)).collect::<Result<_>>()?;
                e.verify_replay(&old,&new,&data[s.training_len..])?;
                // Check stored forecasts against the frozen executable models,
                // not only against another editable presentation of evidence.
                let old_vm=replay(&state.active.program,&data,&state.library,&mut meter)?.0;
                let new_vm=replay(&s.model.program,&data,&state.library,&mut meter)?.0;
                if old_vm[s.training_len..]!=old || new_vm[s.training_len..]!=new {return Err(Error::InvalidData);}
            } else if state.epistemic.is_some() { return Err(Error::InvalidData); }
            s.model.state = replay(&s.model.program,&data,&state.library,&mut meter)?.1;''')
sub('    pub fn enable_curriculum(&mut self,mode: SelectionMode) -> Result<()> { self.learner.enable_curriculum(mode) }','''    pub fn enable_curriculum(&mut self,mode: SelectionMode) -> Result<()> { self.learner.enable_curriculum(mode) }
    pub fn enable_epistemic(&mut self,policy: EvidencePolicy) -> Result<()> { self.learner.enable_epistemic(policy) }''')
sub('            controls: Some(controls),\n        });','            controls: Some(controls),evidence: None,\n        });')
p.write_text(s)
p=root/'src/construction/recursive.rs';s=p.read_text()
sub('use super::synthesis::*;','use super::synthesis::*;\nuse super::epistemic::{EvidenceTrial,RiskLedger};')
sub('        let result = self.complete_inner(trial,future,allow_model);','        let result = self.complete_inner(trial,future,allow_model,None);')
sub('    fn complete_inner(&mut self,mut trial: SearchTrial,future: &[Transition],allow_model: bool) -> Result<RecursiveVerdict> {','''    /// Internal EP1 route: evidence is independently replayed against frozen
    /// programs. The caller cannot replace it with a permissive boolean.
    pub(super) fn complete_trial_epistemic(&mut self,trial: SearchTrial,future: &[Transition],allow_model: bool,
                                         evidence: &EvidenceTrial,risk: &RiskLedger) -> Result<RecursiveVerdict> {
        if self.meta.pending != Some(trial.ticket) { return Err(Error::Stale); }
        self.meta.pending=None;
        let result=self.complete_inner(trial,future,allow_model,Some((evidence,risk)));
        if result.is_err() {self.meta.abandoned=self.meta.abandoned.saturating_add(1);}
        result
    }
    fn complete_inner(&mut self,mut trial: SearchTrial,future: &[Transition],allow_model: bool,
                      evidence: Option<(&EvidenceTrial,&RiskLedger)>) -> Result<RecursiveVerdict> {''')
sub('''            let model_accepted = allow_model && trial.candidate.trace.winner.is_some()
                && no_worse(&cp[..split],&old[..split],&all[..split])
                && candidate.training_loss < absolute_loss(&old[..split],&all[..split])?
                && candidate.future_loss < absolute_loss(&old[split..],future)?;''','''            let model_accepted = if let Some((e,risk)) = evidence {
                if trial.candidate.trace.job.deterministic_contract { return Err(Error::InvalidData); }
                e.binding(risk,&trial.parent,&trial.candidate.program,&trial.library,
                          trial.candidate.trace.job.target,future[0].id)?;
                e.verify_replay(&old[split..],&cp[split..],future)?;
                allow_model && trial.candidate.trace.winner.is_some() && e.accepted()
            } else {
                allow_model && trial.candidate.trace.winner.is_some()
                    && no_worse(&cp[..split],&old[..split],&all[..split])
                    && candidate.training_loss < absolute_loss(&old[..split],&all[..split])?
                    && candidate.future_loss < absolute_loss(&old[split..],future)?
            };''')
p.write_text(s)
print('Patched runtime and recursive audit')
