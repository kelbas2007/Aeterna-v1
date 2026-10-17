from pathlib import Path
import hashlib, shutil
root=Path.cwd()
expected={'src/construction/mod.rs':'6e59525d3baeb5e5b713605f12d1c8016aa192828cc9edaa85ceab4782a56f28','src/construction/runtime.rs':'950b6bc3428030e5dcef17c97bcc8ebd336f8b1f7ea5761a55814c12b8033e46','src/construction/synthesis.rs':'5c408a91d2d665756e1c97a630df7c8ef07bca283e0fe442ab87c31ed090ce19','src/construction/constructors.rs':'16a31548d9d67402ef7066bcf07296ab2a5365cf9697c9b55ebad11cb6af41e6'}
for path,digest in expected.items():
    if hashlib.sha256((root/path).read_bytes()).hexdigest()!=digest: raise RuntimeError('Stale source: '+path)
for source,target in [('autonomy.rs','src/construction/autonomy.rs'),('session.rs','src/construction/session.rs'),('bin.rs','src/bin/aeterna-construction.rs'),('unified.rs','tests/directed_construction_unified.rs'),('session_tests.rs','tests/directed_construction_session.rs'),('lifetime.rs','examples/directed_construction_lifetime.rs'),('adapter.py','scripts/construction_adapter_demo.py')]:
    p=root/target
    if p.exists(): raise RuntimeError('Refusing to overwrite '+target)
    p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(root/'.ci/lifetime'/source,p)
p=root/'src/construction/mod.rs';p.write_text(p.read_text()+'pub mod autonomy;\npub mod session;\n')
p=root/'src/construction/runtime.rs';s=p.read_text()
def sub(a,b,n=1):
    global s
    if s.count(a)!=n: raise RuntimeError('Unexpected source occurrence: '+a[:100])
    s=s.replace(a,b)
sub('use super::constructors::*;','use super::constructors::*;\nuse super::autonomy::{ActionOffer,AutonomyPolicy,AutonomyReport};')
sub('    epistemic: Option<RiskLedger>,','''    epistemic: Option<RiskLedger>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    autonomy: Option<AutonomyPolicy>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    frozen_at: Option<u64>,''')
sub('curriculum: None,epistemic: None })','curriculum: None,epistemic: None,autonomy: None,frozen_at: None })')
sub('    pub fn program(&self) -> &Program {','''    /// Enable the entire bounded loop atomically before experience.
    pub fn enable_autonomy(&mut self,policy:AutonomyPolicy)->Result<()> {
        if self.pending || self.poisoned || self.shadow.is_some() || !self.records.is_empty()
            || self.curriculum.is_some() || self.epistemic.is_some() || self.autonomy.is_some()
            || self.frozen_at.is_some() {return Err(Error::Busy);}
        policy.validate(&self.config)?;
        let curriculum=Curriculum::new(self.config.width,SelectionMode::Fair)?;
        let risk=policy.evidence.as_ref().map(|e|RiskLedger::new(e.clone(),self.config.width)).transpose()?;
        self.curriculum=Some(curriculum);self.epistemic=risk;self.autonomy=Some(policy);Ok(())
    }
    pub fn autonomy_report(&self)->Option<AutonomyReport> {
        let policy=self.autonomy.as_ref()?;let c=self.curriculum.as_ref()?;
        let levels:Vec<_>=(0..self.config.width).map(|j|policy.level(c,j)).collect();
        let effective_limits=levels.iter().map(|&n|policy.limits(&self.config.search,n)).collect();
        let external_cost=self.records.iter().fold(0_u64,|s,r|s.saturating_add(r.external_cost));
        Some(AutonomyReport{levels,effective_limits,external_cost,remaining_cost:policy.max_external_cost.saturating_sub(external_cost),frozen_at:self.frozen_at,events:self.records.len()})
    }
    /// Irreversible freeze; facts and action costs continue, learning does not.
    pub fn freeze_learning(&mut self)->Result<()> {
        if self.pending || self.poisoned {return Err(Error::Busy);}
        if self.frozen_at.is_none(){self.cancel_shadow();self.frozen_at=Some(self.next_id);}Ok(())
    }
    pub fn program(&self) -> &Program {''')
sub('        self.current = post; self.next_id += 1; self.restart_next = false; self.pending = false;','''        self.current = post; self.next_id += 1; self.restart_next = false; self.pending = false;
        // A costly contract violation stays in history, never promotes a model.
        if self.poisoned {self.cancel_shadow();return Ok(());}''')
sub('        if self.shadow.is_none() && self.records.len() >= self.config.minimum_training','        if self.frozen_at.is_none() && self.shadow.is_none() && self.records.len() >= self.config.minimum_training')
sub('        if self.poisoned || self.records.len() < self.config.minimum_training','        if self.frozen_at.is_some() || self.poisoned || self.records.len() < self.config.minimum_training')
sub('''                Ok(errors)
            })();''','''                if let Some(risk)=&self.epistemic {
                    for j in risk.report(None).policy.protected_outputs {errors[j]=0.0;}
                }
                Ok(errors)
            })();''')
sub('''        let before = self.registry.meta_ledger().search_work;
        let controls_result = self.registry.begin_requested_trial(&self.active.program,&data,&self.library,&self.config.search,requested);''','''        let search=match (&self.autonomy,&self.curriculum,requested) {
            (Some(policy),Some(c),Some(j))=>policy.limits(&self.config.search,policy.level(c,j)),
            _=>self.config.search.clone(),
        };
        let before = self.registry.meta_ledger().search_work;
        let controls_result = self.registry.begin_requested_trial(&self.active.program,&data,&self.library,&search,requested);''')
sub('    pub fn rollback(&mut self) -> Result<()> {\n        if self.pending || self.poisoned','    pub fn rollback(&mut self) -> Result<()> {\n        if self.frozen_at.is_some() || self.pending || self.poisoned')
sub('''    pub fn choose_masked(&self,goal: Option<&[Option<i64>]>) -> Result<usize> {
        if self.pending''','''    pub fn choose_masked(&self,goal: Option<&[Option<i64>]>) -> Result<usize> {self.choose_offered(goal,None)}
    pub fn choose_offered(&self,goal:Option<&[Option<i64>]>,offers:Option<&[ActionOffer]>)->Result<usize> {
        if self.pending''')
sub('''        let word = exploration_word(self.next_id);
        if goal.is_none() && word & 3 == 0 {
            return Ok(((word >> 2) % self.config.actions as u64) as usize);
        }''','''        let remaining=self.autonomy_report().map(|r|r.remaining_cost).unwrap_or(u64::MAX);
        if offers.is_some_and(|o|o.len()!=self.config.actions){return Err(Error::InvalidData);}
        let allowed:Vec<_>=(0..self.config.actions).filter(|&a|offers.is_none_or(|o|o[a].available && o[a].max_cost<=remaining)).collect();
        if allowed.is_empty(){return Err(Error::Blocked);}
        let word=exploration_word(self.next_id);
        if goal.is_none() && word & 3 == 0 {return Ok(allowed[((word>>2)%allowed.len() as u64) as usize]);}''')
sub('        let fallback = (0..self.config.actions).min_by_key(|&a| (usage[a],a)).ok_or(Error::Index)?;','        let fallback=allowed.iter().copied().min_by_key(|&a|(usage[a],a)).ok_or(Error::Blocked)?;')
sub('''                for action in 0..self.config.actions {
                    if meter.remaining()''','''                for action in allowed.iter().copied() {
                    if meter.remaining()''')
sub('''                    let mut path = prefix.clone(); path.push(action);
                    let mut predictions''','''                    let mut path = prefix.clone(); path.push(action);
                    let quoted=offers.map(|o|path.iter().try_fold(0_u64,|sum,&a|sum.checked_add(o[a].max_cost))).unwrap_or(Some(0));
                    let Some(quoted)=quoted.filter(|&cost|cost<=remaining) else {continue};
                    let mut predictions''')
sub('                    let score = utility+information+0.1/(1.0+usage[path[0]] as f64)-0.01*depth as f64;','''                    let penalty=if offers.is_some(){0.01*(quoted as f64).ln_1p()}else{0.0};
                    let score = utility+information+0.1/(1.0+usage[path[0]] as f64)-0.01*depth as f64-penalty;''')
sub('        state.config.validate()?; state.library.validate(state.config.width)?; state.registry.validate(&state.library)?;','''        state.config.validate()?; state.library.validate(state.config.width)?; state.registry.validate(&state.library)?;
        if state.frozen_at.is_some_and(|id|id>state.next_id || state.shadow.is_some()){return Err(Error::InvalidData);}
        if let Some(policy)=&state.autonomy {
            policy.validate(&state.config)?;
            if state.curriculum.as_ref().is_none_or(|c|c.mode!=SelectionMode::Fair)
                || policy.evidence!=state.epistemic.as_ref().map(|r|r.report(None).policy){return Err(Error::InvalidData);}
        }''')
sub('pub struct ProtectedConstructionRuntime { organism: EvoPhase,learner: Learner }','pub struct ProtectedConstructionRuntime { organism: EvoPhase,learner: Learner,permit_offset:u64 }')
sub('Ok(Self { organism,learner: Learner::new(config,initial)? })','Ok(Self { organism,learner: Learner::new(config,initial)?,permit_offset:0 })')
sub('    pub fn enable_curriculum(&mut self,mode: SelectionMode) -> Result<()> { self.learner.enable_curriculum(mode) }','''    pub fn enable_curriculum(&mut self,mode: SelectionMode) -> Result<()> { self.learner.enable_curriculum(mode) }
    pub fn enable_autonomy(&mut self,policy:AutonomyPolicy)->Result<()> {self.learner.enable_autonomy(policy)}
    pub fn freeze_learning(&mut self)->Result<()> {self.learner.freeze_learning()}
    pub fn step_autonomous<H,F>(&mut self,goal:Option<&[Option<i64>]>,offers:&[ActionOffer],hazard:H,execute:F)->Result<usize>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->Result<ExternalPost> {
        if self.learner.autonomy.is_none(){return Err(Error::InvalidData);}
        let action=self.learner.choose_offered(goal,Some(offers))?;
        self.step_action_inner(action,hazard,execute,Some(offers[action].max_cost))
    }
    /// Restores software cognition only, never the old SNN or a live permit.
    pub(super) fn resume_session(bytes:&[u8],emergency_latched:bool)->Result<Self> {
        let learner=Learner::from_checkpoint(bytes)?;
        if learner.poisoned || emergency_latched || learner.autonomy.is_none(){return Err(Error::Blocked);}
        let permit_offset=learner.records.iter().map(|r|r.permit_sequence).max().unwrap_or(0);
        let organism=EvoPhase::new(crate::EvoConfig{sensory_cells:learner.config.width,motor_cells:learner.config.actions,..Default::default()});
        Ok(Self{organism,learner,permit_offset})
    }''')
sub('''    pub fn step_action<H,F>(&mut self,action: usize,hazard: H,execute: F) -> Result<usize>
    where H: FnOnce(usize)->HumanProtectionEvidence,F: FnOnce(usize)->Result<ExternalPost> {
        let token''','''    pub fn step_action<H,F>(&mut self,action:usize,hazard:H,execute:F)->Result<usize>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->Result<ExternalPost> {
        if self.learner.autonomy.is_some(){return Err(Error::Execution("autonomous lifetime requires PRE action offers".into()));}
        self.step_action_inner(action,hazard,execute,None)
    }
    fn step_action_inner<H,F>(&mut self,action:usize,hazard:H,execute:F,max_cost:Option<u64>)->Result<usize>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->Result<ExternalPost> {
        let token''')
sub('                let sequence = permit.sequence();','                let sequence=self.permit_offset.checked_add(permit.sequence()).ok_or(Error::Capacity)?;')
sub('                    Ok(post) => { self.learner.commit(token,post.observation,sequence,post.cost)?; Ok(permitted) },','''                    Ok(post)=>{
                        let exceeded=max_cost.is_some_and(|cap|post.cost>cap);
                        if exceeded {self.learner.poisoned=true;self.learner.last_error=Some("external cost exceeded PRE quote".into());}
                        self.learner.commit(token,post.observation,sequence,post.cost)?;
                        if exceeded{Err(Error::Execution("external cost exceeded PRE quote".into()))}else{Ok(permitted)}
                    },''')
p.write_text(s)
p=root/'src/construction/synthesis.rs';s=p.read_text()
sub('struct Best { patch: Option<Patch>,loss:','struct Best { patch: Option<Patch>,target_exact:bool,loss:')
sub('best.loss = loss; best.nodes = model.nodes(); best.patch = Some(patch); best.constructor = constructor;','best.target_exact=target_exact; best.loss = loss; best.nodes = model.nodes(); best.patch = Some(patch); best.constructor = constructor;')
sub('fn baseline(focused: bool,','fn baseline_single(focused: bool,')
sub('Best { patch: None,loss:','Best { patch: None,target_exact:false,loss:',2)
sub('/// Hints change search order, never acceptance or authority.','''/// A finite joint lane can cross individually useless memory steps. No domain
/// delay/counter operator is inserted; complete programs receive validation.
fn baseline(focused:bool,parent:&Program,data:&[Transition],job:&Job,cfg:&SearchConfig,lib:&Library,meter:&mut Meter,best:&mut Best)->Result<()> {
    let room=cfg.max_state.saturating_sub(parent.cells.len());
    if !focused || room<2{return baseline_single(focused,parent,data,job,cfg,lib,meter,best);}
    let mut single=Meter::new(meter.remaining()*3/4);
    let result=baseline_single(focused,parent,data,job,cfg,lib,&mut single,best);meter.charge(single.spent)?;
    if let Err(e)=result{if e!=Error::Budget{return Err(e);}}
    if best.target_exact{return Ok(());}
    let mut leaves:Vec<_>=[0,1,-1,2].into_iter().map(Expr::integer).collect();leaves.push(Expr::Action);
    leaves.extend((0..parent.outputs.len()).map(Expr::Input));leaves.extend((0..parent.cells.len()+2).map(Expr::State));
    let total=leaves.len().saturating_mul(leaves.len()).saturating_mul(3);
    for index in 0..total {
        let initial=[0,1,-1][index%3];let code=index/3;let mut cells=parent.cells.clone();
        cells.push(StateCell{initial,update:leaves[code/leaves.len()].clone()});cells.push(StateCell{initial,update:leaves[code%leaves.len()].clone()});
        let mut local=Meter::new((meter.remaining()/(total-index) as u64).max(1));
        let found=search_output_focused(true,parent,data,job,cells,Expr::Hole{id:0,ty:Ty::Int},None,cfg,lib,&mut local,best);meter.charge(local.spent)?;
        if matches!(found,Ok(true)){return Ok(());}if let Err(e)=found{if e!=Error::Budget{return Err(e);}}
        if meter.remaining()==0{return Err(Error::Budget);}
    }
    Ok(())
}
/// Hints change search order, never acceptance or authority.''')
p.write_text(s)
p=root/'src/construction/constructors.rs';s=p.read_text()
sub('            if old.constructor_id != trace.constructor_id { continue; }','''            // Cross-origin evidence can teach shared structure, never recursive
            // credit: that still requires two descendants of one generator.''')
sub('            ordered.push(((discouraged,holes.len(),std::cmp::Reverse(c.generation),c.id),c));','''            let discouraged=discouraged || c.status==ConstructorStatus::RejectedOnDevelopmentBatch;
            ordered.push(((discouraged,holes.len(),std::cmp::Reverse(c.generation),c.id),c));''')
sub('            if c.status == ConstructorStatus::RejectedOnDevelopmentBatch { continue; }','            // Failed-batch status is an advisory priority, not a permanent ban.')
sub('    #[test] fn training_success_is_not_promotion() {','''    #[test] fn failed_batch_does_not_permanently_hide_a_constructor() {
        let mut r=Registry::default();let lib=Library::default();let mut m=Meter::new(100000);
        r.learn(trace(0),&lib,&mut m).unwrap();r.learn(trace(1),&lib,&mut m).unwrap();
        r.constructors[0].status=ConstructorStatus::RejectedOnDevelopmentBatch;
        assert!(!r.suggestions(&trace(0).job,2,&SearchConfig::default(),&lib,&mut m).unwrap().is_empty());
    }
    #[test] fn cross_origin_structure_is_learnable_without_inventing_recursive_credit() {
        let mut r=Registry::default();let lib=Library::default();let mut m=Meter::new(1_000_000);
        let mut a=trace(0);a.winner.as_mut().unwrap().replacement=Expr::State(0);a.winner.as_mut().unwrap().cells=vec![StateCell{initial:0,update:Expr::Action}];
        let mut b=a.clone();b.job.target=1;b.winner.as_mut().unwrap().target=1;b.winner.as_mut().unwrap().replacement=Expr::bin(Op::Mul,Expr::Input(0),Expr::State(0));
        r.learn(a,&lib,&mut m).unwrap();r.learn(b,&lib,&mut m).unwrap();let parent=r.constructors[0].id;
        let mut c=trace(1);c.job.target=2;c.constructor_id=Some(parent);c.winner=Some(Patch{parent:7,target:2,replacement:Expr::bin(Op::Mul,Expr::Input(1),Expr::State(0)),cells:vec![StateCell{initial:0,update:Expr::Action}]});
        r.learn(c,&lib,&mut m).unwrap();
        let specific=r.constructors.iter().find(|c|matches!(unpack(&c.code,c.cell_count),Ok((Expr::Bin(Op::Mul,_,_),_)))).unwrap();
        assert_eq!(specific.generation,1);assert!(specific.refinement.is_none());
    }
    #[test] fn training_success_is_not_promotion() {''')
p.write_text(s)
print('Integrated autonomous lifetime, joint construction and durable session')
