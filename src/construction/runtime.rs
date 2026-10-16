//! Persistent, opt-in construction runtime using EvoPhase's sealed permission
//! boundary. Authored software synthesis is NOT physical SNN-owned computation.
use super::constructors::*;
use super::synthesis::*;
use super::vm::*;
use crate::{EvoPhase, HumanProtectionDecision, HumanProtectionEvidence};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub width: usize, pub actions: usize, pub search: SearchConfig,
    pub minimum_training: usize, pub search_every: usize, pub shadow_steps: usize,
    pub journal_capacity: usize, pub planning_depth: usize, pub planning_work: u64,
}
impl Default for Config {
    fn default() -> Self { Self { width: 1,actions: 2,search: SearchConfig::default(),minimum_training: 8,search_every: 4,shadow_steps: 6,journal_capacity: 512,planning_depth: 3,planning_work: 20000 } }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        self.search.validate()?;
        if !(1..=32).contains(&self.width) || !(1..=16).contains(&self.actions)
            || !(2..=128).contains(&self.minimum_training) || self.search_every == 0
            || !(2..=128).contains(&self.shadow_steps) || self.journal_capacity < self.minimum_training+self.shadow_steps
            || self.journal_capacity > 4096 || !(1..=5).contains(&self.planning_depth)
            || !(100..=1000000).contains(&self.planning_work) { return Err(Error::Capacity); }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast { pub center: Vec<i64>, pub scale: Vec<f64> }
impl Forecast {
    /// Discrete Laplace likelihood; past-fitted scale stays frozen in a trial.
    pub fn nll(&self,post: &[i64]) -> Result<f64> {
        if post.len() != self.center.len() || self.scale.len() != post.len() { return Err(Error::InvalidData); }
        let mut nll = 0.0;
        for ((&mu,&scale),&actual) in self.center.iter().zip(&self.scale).zip(post) {
            if !scale.is_finite() || !(0.25..=1e9).contains(&scale) { return Err(Error::InvalidData); }
            let r = (-1.0/scale).exp();
            nll += ((actual as i128-mu as i128).abs() as f64)/scale + ((1.0+r)/(1.0-r)).ln();
        }
        Ok(nll)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Model { program: Program,state: Vec<i64>,scale: Vec<f64> }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Shadow {
    model: Model, trace: SearchTrace, training_len: usize, start_id: u64,
    samples: usize, incumbent_nll: f64,candidate_nll: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub transition: Transition, pub model: u64,pub prediction: Forecast,
    pub candidate: Option<u64>,pub candidate_prediction: Option<Forecast>,
    pub permit_sequence: u64,pub external_cost: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    pub first_id: u64,pub last_id: u64,pub parent: u64,pub candidate: u64,
    pub accepted: bool,pub retention_ok: bool,pub incumbent_nll: f64,pub candidate_nll: f64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Learner {
    config: Config,active: Model,previous: Option<Program>,shadow: Option<Shadow>,
    library: Library,registry: Registry,records: Vec<Record>,promotions: Vec<Promotion>,
    current: Vec<i64>,next_id: u64,pending: bool,poisoned: bool,restart_next: bool,
    last_trace: Option<SearchTrace>,construction_work: u64,last_error: Option<String>,
}
/// Non-cloneable and non-serializable PRE-action commitment.
struct Commitment {
    id: u64,pre: Vec<i64>,action: usize,model: u64,prediction: Forecast,next_state: Vec<i64>,
    candidate: Option<u64>,candidate_prediction: Option<Forecast>,candidate_state: Option<Vec<i64>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotReport {
    pub events: usize,pub active_program: u64,pub state_cells: usize,pub program_nodes: usize,
    pub library_nodes: usize,pub constructors: usize,pub promotions: usize,
    pub construction_work: u64,pub shadow_pending: bool,pub poisoned: bool,
    pub last_error: Option<String>,
}
fn calibrate(program: &Program,data: &[Transition],lib: &Library,meter: &mut Meter) -> Result<(Vec<i64>,Vec<f64>)> {
    let (predicted,state) = replay(program,data,lib,meter)?;
    let mut errors = vec![0.0; program.outputs.len()];
    for (p,t) in predicted.iter().zip(data) {
        for (j,(&x,&y)) in p.iter().zip(&t.post).enumerate() { errors[j] += ((x as i128-y as i128).abs()) as f64; }
    }
    let scale = errors.into_iter().map(|e| (e/(data.len().max(1) as f64)).clamp(0.25,1e9)).collect();
    Ok((state,scale))
}
/// Deterministic model-independent exploration reserve. Only factual event count
/// enters this mixer, never an environment seed, task identity or hidden state.
fn exploration_word(event: u64) -> u64 {
    let mut z = event.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
impl Learner {
    pub fn new(config: Config,initial: Vec<i64>) -> Result<Self> {
        config.validate()?;
        if initial.len() != config.width { return Err(Error::InvalidData); }
        Ok(Self { active: Model { program: Program::identity(config.width),state: vec![],scale: vec![1.0;config.width] },
            config,previous: None,shadow: None,library: Library::default(),registry: Registry::default(),records: vec![],promotions: vec![],
            current: initial,next_id: 0,pending: false,poisoned: false,restart_next: true,last_trace: None,construction_work: 0,last_error: None })
    }
    pub fn program(&self) -> &Program { &self.active.program }
    pub fn library(&self) -> &Library { &self.library }
    pub fn registry(&self) -> &Registry { &self.registry }
    pub fn records(&self) -> &[Record] { &self.records }
    pub fn promotions(&self) -> &[Promotion] { &self.promotions }
    pub fn last_search(&self) -> Option<&SearchTrace> { self.last_trace.as_ref() }
    pub fn current(&self) -> &[i64] { &self.current }
    pub fn report(&self) -> SnapshotReport {
        SnapshotReport { events: self.records.len(),active_program: self.active.program.fingerprint(),state_cells: self.active.program.cells.len(),program_nodes: self.active.program.nodes(),library_nodes: self.library.nodes(),constructors: self.registry.constructors().len(),promotions: self.promotions.iter().filter(|p| p.accepted).count(),construction_work: self.construction_work,shadow_pending: self.shadow.is_some(),poisoned: self.poisoned,last_error: self.last_error.clone() }
    }
    fn data(&self) -> Vec<Transition> { self.records.iter().map(|r| r.transition.clone()).collect() }
    fn prepare(&mut self,action: usize) -> Result<Commitment> {
        if self.poisoned { return Err(Error::Execution("unknown external state after callback fault".into())); }
        if self.pending { return Err(Error::Busy); }
        if action >= self.config.actions { return Err(Error::Index); }
        if self.records.len() >= self.config.journal_capacity || self.next_id == u64::MAX { return Err(Error::Capacity); }
        let mut meter = Meter::new(100000);
        let (center,next_state) = self.active.program.step(&self.current,action,&self.active.state,&self.library,&mut meter)?;
        let mut candidate = None; let mut candidate_prediction = None; let mut candidate_state = None;
        if let Some(shadow) = &self.shadow {
            match shadow.model.program.step(&self.current,action,&shadow.model.state,&self.library,&mut meter) {
                Ok((p,s)) => { candidate = Some(shadow.model.program.fingerprint()); candidate_prediction = Some(Forecast { center: p,scale: shadow.model.scale.clone() }); candidate_state = Some(s); },
                Err(e) => { self.last_error = Some(format!("shadow execution rejected: {e}")); self.shadow = None; }
            }
        }
        self.pending = true;
        Ok(Commitment { id: self.next_id,pre: self.current.clone(),action,model: self.active.program.fingerprint(),
            prediction: Forecast { center,scale: self.active.scale.clone() },next_state,candidate,candidate_prediction,candidate_state })
    }
    fn cancel(&mut self,token: Commitment) -> Result<()> {
        if !self.pending || token.id != self.next_id { return Err(Error::Stale); }
        self.pending = false; Ok(())
    }
    fn fault(&mut self,token: Commitment,message: String) {
        let _ = self.cancel(token); self.poisoned = true; self.last_error = Some(message);
    }
    fn commit(&mut self,token: Commitment,post: Vec<i64>,permit_sequence: u64,external_cost: u64) -> Result<()> {
        if !self.pending || token.id != self.next_id || token.pre != self.current || token.model != self.active.program.fingerprint() { return Err(Error::Stale); }
        if post.len() != self.config.width {
            self.fault(token,"invalid POST after possible external execution".into()); return Err(Error::InvalidData);
        }
        let incumbent_nll = token.prediction.nll(&post)?;
        let candidate_nll = token.candidate_prediction.as_ref().map(|p| p.nll(&post)).transpose()?;
        self.active.state = token.next_state;
        if let (Some(shadow),Some(nll),Some(state)) = (&mut self.shadow,candidate_nll,token.candidate_state) {
            if token.candidate != Some(shadow.model.program.fingerprint()) || token.id < shadow.start_id { return Err(Error::Stale); }
            shadow.model.state = state; shadow.samples += 1; shadow.incumbent_nll += incumbent_nll; shadow.candidate_nll += nll;
        }
        let transition = Transition { id: token.id,restart: self.restart_next,pre: token.pre,action: token.action,post: post.clone() };
        self.records.push(Record { transition,model: token.model,prediction: token.prediction,candidate: token.candidate,candidate_prediction: token.candidate_prediction,permit_sequence,external_cost });
        self.current = post; self.next_id += 1; self.restart_next = false; self.pending = false;
        if self.shadow.as_ref().is_some_and(|s| s.samples >= self.config.shadow_steps) {
            if let Err(e) = self.finish_shadow() { self.last_error = Some(format!("validation: {e}")); self.shadow = None; }
        }
        if self.shadow.is_none() && self.records.len() >= self.config.minimum_training && self.records.len() % self.config.search_every == 0 {
            if let Err(e) = self.construct() {
                if e != Error::NoEvidence { self.last_error = Some(format!("construction: {e}")); }
            }
        }
        Ok(())
    }
    /// Synchronous learning; no external action or synthetic factual record.
    pub fn construct(&mut self) -> Result<()> {
        if self.pending || self.shadow.is_some() { return Err(Error::Busy); }
        if self.poisoned || self.records.len() < self.config.minimum_training { return Err(Error::NoEvidence); }
        let data = self.data();
        let mut overhead = Meter::new(self.config.search.work/8);
        let job_result = discover_job(&self.active.program,&data,&self.library,self.config.search.deterministic,&mut overhead);
        let job = match job_result { Ok(Some(j)) => j,Ok(None) => { self.construction_work += overhead.spent; return Err(Error::NoEvidence); },Err(e) => { self.construction_work += overhead.spent; return Err(e); } };
        let hints_result = self.registry.suggestions(&job,self.config.width,&self.config.search,&self.library,&mut overhead);
        self.construction_work += overhead.spent;
        let hints = match hints_result { Ok(h) => h,Err(Error::Budget) => vec![],Err(e) => return Err(e) };
        let mut config = self.config.search.clone(); config.work -= overhead.spent;
        let trace = synthesize(&self.active.program,&data,&self.library,&config,&hints)?;
        self.construction_work += trace.work;
        self.last_trace = Some(trace.clone());
        let Some(patch) = &trace.winner else { return Ok(()); };
        let program = patch.apply(&self.active.program,&self.library)?;
        if program == self.active.program { return Ok(()); }
        let mut meter = Meter::new(1000000);
        let (state,scale) = calibrate(&program,&data,&self.library,&mut meter)?;
        let (_,incumbent_scale) = calibrate(&self.active.program,&data,&self.library,&mut meter)?;
        self.construction_work += meter.spent;
        self.active.scale = incumbent_scale;
        self.shadow = Some(Shadow { model: Model { program,state,scale },trace,training_len: data.len(),start_id: self.next_id,samples: 0,incumbent_nll: 0.0,candidate_nll: 0.0 });
        Ok(())
    }
    fn finish_shadow(&mut self) -> Result<()> {
        let mut shadow = self.shadow.take().ok_or(Error::NoEvidence)?;
        let data = self.data(); let training = &data[..shadow.training_len];
        let mut meter = Meter::new(2000000);
        let (old,_) = replay(&self.active.program,training,&self.library,&mut meter)?;
        let (new,_) = replay(&shadow.model.program,training,&self.library,&mut meter)?;
        let retention_ok = old.iter().zip(new.iter()).zip(training).all(|((a,b),t)| {
            let cost = |p: &[i64]| p.iter().zip(&t.post).map(|(&x,&y)| (x as i128-y as i128).abs()).sum::<i128>(); cost(b) <= cost(a)
        });
        let complexity = 0.01 * (shadow.model.program.nodes() as f64-self.active.program.nodes() as f64);
        let accepted = retention_ok && shadow.candidate_nll+complexity < shadow.incumbent_nll;
        let promotion = Promotion { first_id: shadow.start_id,last_id: self.next_id-1,parent: self.active.program.fingerprint(),candidate: shadow.model.program.fingerprint(),accepted,retention_ok,incumbent_nll: shadow.incumbent_nll,candidate_nll: shadow.candidate_nll };
        if self.promotions.len() >= 128 { return Err(Error::Capacity); }
        self.promotions.push(promotion);
        if accepted {
            self.previous = Some(self.active.program.clone()); self.active = shadow.model;
            shadow.trace.prospectively_accepted = true;
            self.last_trace = Some(shadow.trace.clone());
            if let Some(patch) = &shadow.trace.winner {
                if let Err(e) = acquire_function(&patch.replacement,&mut self.library,self.config.width) { self.last_error = Some(format!("library: {e}")); }
            }
            if let Err(e) = self.registry.learn(shadow.trace,&self.library,&mut meter) { self.last_error = Some(format!("constructor acquisition: {e}")); }
        }
        self.construction_work += meter.spent;
        Ok(())
    }
    /// Roll back computation, NOT history, costs, environment or permissions.
    pub fn rollback(&mut self) -> Result<()> {
        if self.pending || self.poisoned { return Err(Error::Busy); }
        let program = self.previous.clone().ok_or(Error::NoEvidence)?;
        let mut meter = Meter::new(2000000);
        let (state,scale) = calibrate(&program,&self.data(),&self.library,&mut meter)?;
        self.previous = Some(self.active.program.clone()); self.active = Model { program,state,scale }; self.shadow = None;
        self.construction_work += meter.spent; Ok(())
    }
    /// Records a real host-observed reset, retaining one lifetime journal.
    pub fn record_external_restart(&mut self,observation: Vec<i64>) -> Result<()> {
        if self.pending || self.poisoned { return Err(Error::Busy); }
        if observation.len() != self.config.width { return Err(Error::InvalidData); }
        self.current = observation; self.restart_next = true; self.active.state = self.active.program.initial_state();
        if let Some(s) = &mut self.shadow { s.model.state = s.model.program.initial_state(); }
        Ok(())
    }
    /// Ephemeral imagination plus a model-independent exploration reserve.
    /// No simulated POST is inserted into the factual journal.
    pub fn choose_action(&self,goal: Option<&[i64]>) -> Result<usize> {
        if self.pending || self.poisoned { return Err(Error::Busy); }
        if goal.is_some_and(|g| g.len() != self.config.width) { return Err(Error::InvalidData); }
        let word = exploration_word(self.next_id);
        if goal.is_none() && word & 3 == 0 {
            return Ok(((word >> 2) % self.config.actions as u64) as usize);
        }
        let mut models = vec![&self.active]; if let Some(s) = &self.shadow { models.push(&s.model); }
        let mut meter = Meter::new(self.config.planning_work);
        let mut usage = vec![0usize;self.config.actions];
        for r in &self.records { usage[r.transition.action] += 1; }
        let fallback = (0..self.config.actions).min_by_key(|&a| (usage[a],a)).ok_or(Error::Index)?;
        let mut best = (f64::NEG_INFINITY,fallback); let mut paths = vec![vec![]];
        for depth in 1..=self.config.planning_depth {
            let mut next_paths = Vec::new();
            for prefix in paths {
                for action in 0..self.config.actions {
                    if meter.remaining() < 10 { return Ok(best.1); }
                    meter.charge(1)?;
                    let mut path = prefix.clone(); path.push(action);
                    let mut predictions = Vec::new(); let mut valid = true;
                    for model in &models {
                        let mut input = self.current.clone(); let mut state = model.state.clone();
                        for &a in &path {
                            match model.program.step(&input,a,&state,&self.library,&mut meter) {
                                Ok((o,s)) => { input = o; state = s; },
                                Err(Error::Budget) => return Ok(best.1),
                                Err(_) => { valid = false; break; }
                            }
                        }
                        if valid { predictions.push(input); } else { break; }
                    }
                    if !valid { continue; }
                    let information = if predictions.len() > 1 && predictions[0] != predictions[1] { 1.0 } else { 0.0 };
                    let utility = goal.map(|g| predictions.iter().map(|p| {
                        if p == g { 5.0 } else { -p.iter().zip(g).map(|(&a,&b)| ((a as i128-b as i128).abs()) as f64).sum::<f64>() }
                    }).sum::<f64>()/predictions.len() as f64).unwrap_or(0.0);
                    let score = utility+information+0.1/(1.0+usage[path[0]] as f64)-0.01*depth as f64;
                    if score > best.0 { best = (score,path[0]); }
                    if next_paths.len() < 4096 { next_paths.push(path); }
                }
            }
            paths = next_paths;
        }
        Ok(best.1)
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        if self.pending { return Err(Error::Busy); }
        let payload = serde_json::to_vec(self).map_err(|_| Error::InvalidData)?;
        if payload.len() > 4*1024*1024 { return Err(Error::Capacity); }
        serde_json::to_vec(&(1u32,fingerprint(&payload),self)).map_err(|_| Error::InvalidData)
    }
    pub fn from_checkpoint(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 4*1024*1024 { return Err(Error::Capacity); }
        let (version,hash,mut state): (u32,u64,Self) = serde_json::from_slice(bytes).map_err(|_| Error::InvalidData)?;
        if version != 1 || hash != fingerprint(&serde_json::to_vec(&state).map_err(|_| Error::InvalidData)?) || state.pending { return Err(Error::InvalidData); }
        state.config.validate()?; state.library.validate(state.config.width)?; state.registry.validate(&state.library)?;
        if state.records.len() > state.config.journal_capacity || state.promotions.len() > 128 || state.current.len() != state.config.width || state.next_id != state.records.len() as u64 { return Err(Error::InvalidData); }
        state.active.program.validate(state.config.width,&state.library)?;
        if let Some(p) = &state.previous { p.validate(state.config.width,&state.library)?; }
        let mut meter = Meter::new(10000000);
        if !state.records.is_empty() {
            validate_data(&state.data(),state.config.width,state.config.actions)?;
            if !state.restart_next && state.records.last().unwrap().transition.post != state.current { return Err(Error::InvalidData); }
            for (i,r) in state.records.iter().enumerate() {
                if r.transition.id != i as u64 { return Err(Error::InvalidData); }
                r.prediction.nll(&r.transition.post)?;
                if let Some(p) = &r.candidate_prediction { p.nll(&r.transition.post)?; }
            }
            state.active.state = replay(&state.active.program,&state.data(),&state.library,&mut meter)?.1;
        } else { state.active.state = state.active.program.initial_state(); }
        if state.restart_next { state.active.state = state.active.program.initial_state(); }
        Forecast { center: state.current.clone(),scale: state.active.scale.clone() }.nll(&state.current)?;
        let data = state.data();
        if let Some(s) = &mut state.shadow {
            s.model.program.validate(state.config.width,&state.library)?;
            if s.training_len > data.len() || s.start_id != s.training_len as u64 || s.samples != data.len()-s.training_len || s.samples >= state.config.shadow_steps || !s.incumbent_nll.is_finite() || !s.candidate_nll.is_finite() { return Err(Error::InvalidData); }
            s.model.state = replay(&s.model.program,&data,&state.library,&mut meter)?.1;
            if state.restart_next { s.model.state = s.model.program.initial_state(); }
            Forecast { center: state.current.clone(),scale: s.model.scale.clone() }.nll(&state.current)?;
        }
        Ok(state)
    }
}
#[derive(Debug)]
pub struct ExternalPost { pub observation: Vec<i64>,pub cost: u64 }
/// Does not clone or serialize the live action authority.
#[derive(Debug)]
pub struct ProtectedConstructionRuntime { organism: EvoPhase,learner: Learner }
impl ProtectedConstructionRuntime {
    pub fn new(organism: EvoPhase,config: Config,initial: Vec<i64>) -> Result<Self> {
        if organism.config().motor_cells != config.actions || organism.config().sensory_cells != config.width { return Err(Error::InvalidData); }
        Ok(Self { organism,learner: Learner::new(config,initial)? })
    }
    pub fn learner(&self) -> &Learner { &self.learner }
    pub fn organism(&self) -> &EvoPhase { &self.organism }
    pub fn checkpoint(&self) -> Result<Vec<u8>> { self.learner.checkpoint() }
    /// Live round-trip requires the identical cognitive generation, including
    /// history, costs, consumed trials and fault state. Equal observations alone
    /// are insufficient. Use rollback_cognition to revert only computation.
    pub fn restore_cognition(&mut self,bytes: &[u8]) -> Result<()> {
        if self.learner.pending { return Err(Error::Busy); }
        let restored = Learner::from_checkpoint(bytes)?;
        if restored.checkpoint()? != self.learner.checkpoint()? { return Err(Error::Stale); }
        self.learner = restored; Ok(())
    }
    pub fn rollback_cognition(&mut self) -> Result<()> { self.learner.rollback() }
    pub fn step<H,F>(&mut self,goal: Option<&[i64]>,hazard: H,execute: F) -> Result<usize>
    where H: FnOnce(usize)->HumanProtectionEvidence,F: FnOnce(usize)->Result<ExternalPost> {
        let action = self.learner.choose_action(goal)?; self.step_action(action,hazard,execute)
    }
    /// Explicitly supervised interventions, still protected and fact-recorded.
    pub fn step_action<H,F>(&mut self,action: usize,hazard: H,execute: F) -> Result<usize>
    where H: FnOnce(usize)->HumanProtectionEvidence,F: FnOnce(usize)->Result<ExternalPost> {
        let token = self.learner.prepare(action)?;
        match self.organism.screen_physical_action(action,hazard(action)) {
            HumanProtectionDecision::Blocked(_) => { self.learner.cancel(token)?; Err(Error::Blocked) },
            HumanProtectionDecision::Allowed { permit,.. } => {
                let sequence = permit.sequence();
                let permitted = match self.organism.consume_physical_action_permit(permit) {
                    Ok(a) => a,Err(e) => { self.learner.cancel(token)?; return Err(Error::Execution(format!("permit: {e:?}"))); }
                };
                match execute(permitted) {
                    Ok(post) => { self.learner.commit(token,post.observation,sequence,post.cost)?; Ok(permitted) },
                    Err(e) => { self.learner.fault(token,format!("external callback: {e}")); Err(e) }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvoConfig;
    fn safe(_: usize) -> HumanProtectionEvidence { HumanProtectionEvidence { human_present: false,physical_effect_possible: false,predicted_harm_probability: 0.0,hazard_confidence: 1.0,emergency_stop: false } }
    fn runtime() -> ProtectedConstructionRuntime {
        let cfg = Config { minimum_training: 6,shadow_steps: 4,search_every: 2,search: SearchConfig { deterministic: true,..Default::default() },..Default::default() };
        ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig { sensory_cells: 1,motor_cells: 2,..Default::default() }),cfg,vec![0]).unwrap()
    }
    fn train(r: &mut ProtectedConstructionRuntime,n: usize) -> i64 {
        let mut hidden = 0;
        for i in 0..n {
            let action = [1,1,0,1,0,0,1,0][i%8];
            r.step_action(action,safe,|a| { let post = hidden; hidden = a as i64; Ok(ExternalPost { observation: vec![post],cost: 1 }) }).unwrap();
        }
        hidden
    }
    #[test] fn closed_loop_acquires_state_then_uses_fresh_shadow_observations() {
        let mut r = runtime(); train(&mut r,32);
        assert!(!r.learner.program().cells.is_empty(),"{:?}",r.learner.report());
        assert!(r.learner.promotions().iter().any(|p| p.accepted));
        for p in r.learner.promotions() { assert!(p.last_id >= p.first_id+3); }
        for pair in r.learner.records().windows(2) { assert!(pair[0].permit_sequence < pair[1].permit_sequence); }
    }
    #[test] fn autonomous_acquisition_survives_frozen_non_alternating_probe() {
        let mut r = runtime(); let mut hidden = 0;
        for _ in 0..64 {
            r.step(None,safe,|a| { let post = hidden; hidden = a as i64; Ok(ExternalPost { observation: vec![post],cost: 1 }) }).unwrap();
        }
        let p = r.learner.program().clone(); let lib = r.learner.library().clone();
        let mut meter = Meter::new(100000);
        let mut state = replay(&p,&r.learner.data(),&lib,&mut meter).unwrap().1;
        let mut input = r.learner.current().to_vec(); let mut wrong = 0; let mut confound_wrong = 0;
        for a in [0,0,0,1,0,1,1,1].into_iter().cycle().take(24) {
            let (prediction,next) = p.step(&input,a,&state,&lib,&mut meter).unwrap();
            let actual = hidden; hidden = a as i64;
            wrong += usize::from(prediction != [actual]);
            confound_wrong += usize::from(i64::from(a == 0) != actual);
            state = next; input = vec![actual];
        }
        assert_eq!(wrong,0,"frozen model: {p:?}; {:?}",r.learner.report());
        assert!(confound_wrong > 0); assert_eq!(r.learner.records().len(),64);
    }
    #[test] fn simulation_does_not_manufacture_factual_experience() {
        let mut r = runtime(); train(&mut r,24); let n = r.learner.records().len();
        for _ in 0..5 { r.learner.choose_action(Some(&[1])).unwrap(); }
        assert_eq!(r.learner.records().len(),n);
    }
    #[test] fn blocked_callback_and_checkpoint_cannot_clear_emergency_stop() {
        let mut r = runtime(); let snapshot = r.checkpoint().unwrap(); let mut calls = 0;
        let blocked = r.step_action(0,|a| { let mut evidence = safe(a); evidence.emergency_stop = true; evidence },|_| { calls += 1; Ok(ExternalPost { observation: vec![0],cost: 1 }) });
        assert_eq!(blocked,Err(Error::Blocked)); assert_eq!(calls,0); assert!(r.learner.records().is_empty());
        r.restore_cognition(&snapshot).unwrap(); assert!(r.organism().human_protection_emergency_latched());
        assert_eq!(r.step_action(0,safe,|_| { calls += 1; Ok(ExternalPost { observation: vec![0],cost: 1 }) }),Err(Error::Blocked));
        assert_eq!(calls,0);
    }
    #[test] fn callback_fault_poisoned_not_replayed() {
        let mut r = runtime(); assert!(r.step_action(1,safe,|_| Err(Error::Execution("connection lost".into()))).is_err());
        assert!(r.learner.report().poisoned); let mut called = false;
        assert!(r.step_action(1,safe,|_| { called = true; Ok(ExternalPost { observation: vec![0],cost: 1 }) }).is_err()); assert!(!called);
    }
    #[test] fn older_snapshot_cannot_erase_a_same_observation_event() {
        let mut r = runtime(); let bytes = r.checkpoint().unwrap();
        r.step_action(0,safe,|_| Ok(ExternalPost { observation: vec![0],cost: 7 })).unwrap();
        assert_eq!(r.restore_cognition(&bytes),Err(Error::Stale));
        assert_eq!(r.learner.records().len(),1); assert_eq!(r.learner.records()[0].external_cost,7);
    }
    #[test] fn snapshot_cannot_clear_unknown_external_state() {
        let mut r = runtime(); let bytes = r.checkpoint().unwrap();
        let _ = r.step_action(1,safe,|_| Err(Error::Execution("unknown result".into())));
        assert_eq!(r.restore_cognition(&bytes),Err(Error::Stale)); assert!(r.learner.report().poisoned);
    }
    #[test] fn checkpoint_round_trip_and_corruption() {
        let mut r = runtime(); train(&mut r,32); let bytes = r.checkpoint().unwrap(); let restored = Learner::from_checkpoint(&bytes).unwrap();
        assert_eq!(restored.program(),r.learner.program()); assert_eq!(restored.current(),r.learner.current()); assert_eq!(restored.records().len(),32);
        r.restore_cognition(&bytes).unwrap();
        let mut corrupt = bytes; corrupt[3] ^= 1; assert!(Learner::from_checkpoint(&corrupt).is_err());
    }
    #[test] fn capacity_is_checked_before_external_execution() {
        let mut r = runtime(); r.learner.config.journal_capacity = 10; train(&mut r,10);
        let mut called = false;
        assert_eq!(r.step_action(1,safe,|_| { called = true; Ok(ExternalPost { observation: vec![0],cost: 1 }) }),Err(Error::Capacity)); assert!(!called);
    }
    #[test] fn probability_handles_extreme_integer_difference() {
        let f = Forecast { center: vec![i64::MIN],scale: vec![0.25] };
        assert!(f.nll(&[i64::MAX]).unwrap().is_finite());
    }
    #[test] fn rollback_does_not_delete_experience() {
        let mut r = runtime(); train(&mut r,32); let n = r.learner.records().len();
        r.rollback_cognition().unwrap(); assert_eq!(r.learner.records().len(),n); assert_eq!(r.learner.next_id,n as u64);
    }
}
