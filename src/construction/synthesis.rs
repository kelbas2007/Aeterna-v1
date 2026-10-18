//! Bounded synthesis from factual PRE/action/POST records.
//! Matching signatures are never treated as semantic equivalence.
use super::vm::*;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub id: u64,
    /// Externally recorded start/reset, never a simulated reset.
    pub restart: bool,
    pub pre: Vec<i64>, pub action: usize, pub post: Vec<i64>,
}
pub fn validate_data(data: &[Transition], width: usize, actions: usize) -> Result<()> {
    if data.is_empty() || !data[0].restart || data.len() > 4096 || width == 0 || width > 32 || actions == 0 || actions > 32 { return Err(Error::InvalidData); }
    let mut last = None;
    for (i,t) in data.iter().enumerate() {
        if t.pre.len() != width || t.post.len() != width || t.action >= actions || last.is_some_and(|id| id >= t.id) { return Err(Error::InvalidData); }
        if i > 0 && !t.restart && data[i-1].post != t.pre { return Err(Error::InvalidData); }
        last = Some(t.id);
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub parent: u64, pub target: usize,
    pub witness_ids: Vec<(u64,u64)>,
    pub deterministic_contract: bool,
    pub stateless_impossible: bool,
    pub exposed_sources: Vec<usize>,
    /// Observable features only; no task name, seed or correct program.
    pub features: Vec<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub work: u64, pub max_expr_nodes: usize, pub max_frontier: usize,
    pub max_updates: usize, pub max_state: usize, pub deterministic: bool,
}
impl Default for SearchConfig {
    fn default() -> Self { Self { work: 300_000, max_expr_nodes: 7, max_frontier: 2048, max_updates: 24, max_state: 4, deterministic: false } }
}
impl SearchConfig {
    pub fn validate(&self) -> Result<()> {
        if self.work < 100 || self.work > 100_000_000 || !(1..=15).contains(&self.max_expr_nodes)
            || !(8..=32768).contains(&self.max_frontier) || !(1..=512).contains(&self.max_updates) || self.max_state > 8 { return Err(Error::Capacity); }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    pub parent: u64, pub target: usize, pub replacement: Expr,
    pub cells: Vec<StateCell>,
}
impl Patch {
    pub fn apply(&self, parent: &Program, lib: &Library) -> Result<Program> {
        if self.parent != parent.fingerprint() || self.target >= parent.outputs.len() { return Err(Error::Stale); }
        let mut p = parent.clone(); p.outputs[self.target] = self.replacement.clone(); p.cells = self.cells.clone();
        p.validate(parent.outputs.len(),lib)?; Ok(p)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTrace {
    pub job: Job, pub candidates: u64, pub updates_tested: u64, pub work: u64,
    pub baseline_work: u64, pub constructor_work: u64,
    pub exhausted: bool, pub frontier_limited: bool,
    pub winner: Option<Patch>, pub training_loss: f64,
    pub constructor_id: Option<u64>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub search_policy_id: Option<u64>,
    pub prospectively_accepted: bool,
}
#[derive(Debug, Clone)]
pub struct Hint { pub constructor_id: u64, pub output: Expr, pub cells: Vec<StateCell> }

pub fn replay(program: &Program,data: &[Transition],lib: &Library,meter: &mut Meter) -> Result<(Vec<Vec<i64>>,Vec<i64>)> {
    let mut state = program.initial_state(); let mut predictions = Vec::with_capacity(data.len());
    for t in data {
        meter.charge(1)?;
        if t.restart { state = program.initial_state(); }
        let (p,s) = program.step(&t.pre,t.action,&state,lib,meter)?;
        predictions.push(p); state = s;
    }
    Ok((predictions,state))
}
pub fn absolute_loss(predictions: &[Vec<i64>],data: &[Transition]) -> Result<f64> {
    if predictions.len() != data.len() { return Err(Error::InvalidData); }
    let mut sum = 0.0;
    for (p,t) in predictions.iter().zip(data) {
        if p.len() != t.post.len() { return Err(Error::InvalidData); }
        for (&a,&b) in p.iter().zip(&t.post) { sum += ((a as i128-b as i128).abs()) as f64; }
    }
    Ok(sum)
}
pub fn discover_job(program: &Program,data: &[Transition],lib: &Library,deterministic: bool,meter: &mut Meter) -> Result<Option<Job>> {
    discover_requested_job(program,data,lib,deterministic,None,meter)
}
pub fn discover_requested_job(program: &Program,data: &[Transition],lib: &Library,deterministic: bool,requested: Option<usize>,meter: &mut Meter) -> Result<Option<Job>> {
    let width = program.outputs.len();
    let (predicted,_) = replay(program,data,lib,meter)?;
    let mut errors = vec![0.0; width];
    for (p,t) in predicted.iter().zip(data) {
        for j in 0..width { errors[j] += ((p[j] as i128-t.post[j] as i128).abs()) as f64; }
    }
    let target = match requested {
        Some(j) if j < width => j,
        Some(_) => return Err(Error::Index),
        None => (0..width).max_by(|&a,&b| errors[a].total_cmp(&errors[b]).then_with(|| b.cmp(&a))).ok_or(Error::InvalidData)?,
    };
    if errors[target] == 0.0 { return Ok(None); }
    let mut seen: BTreeMap<(Vec<i64>,usize),(i64,u64)> = BTreeMap::new();
    let mut witnesses = Vec::new();
    for t in data {
        meter.charge(1 + width as u64)?;
        let key = (t.pre.clone(),t.action);
        if let Some(&(y,id)) = seen.get(&key) {
            if y != t.post[target] && witnesses.len() < 64 { witnesses.push((id,t.id)); }
        } else { seen.insert(key,(t.post[target],t.id)); }
    }
    let features = vec![width as i64,data.len() as i64,witnesses.len() as i64,program.cells.len() as i64];
    Ok(Some(Job { parent: program.fingerprint(),target,
        stateless_impossible: deterministic && !witnesses.is_empty(),deterministic_contract: deterministic,
        witness_ids: witnesses,exposed_sources: (0..width).collect(),features }))
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pub width: usize, pub states: usize, pub max_nodes: usize, pub max_frontier: usize,
}
pub struct Enumerator {
    grammar: Grammar, queue: BinaryHeap<Reverse<(usize,u64,Expr)>>,
    serial: u64, pub limited: bool,
}
impl Enumerator {
    pub fn new(grammar: Grammar, root: Expr) -> Self {
        let mut queue = BinaryHeap::new(); queue.push(Reverse((root.nodes(),0,root)));
        Self { grammar,queue,serial: 0,limited: false }
    }
    fn productions(&self, ty: Ty, next: u16, lib: &Library) -> Vec<Expr> {
        let hole = |offset: u16,t| Expr::Hole { id: next+offset,ty: t };
        let mut p = match ty {
            Ty::Int => {
                let mut a = vec![Expr::integer(0),Expr::integer(1),Expr::integer(-1),Expr::integer(2),Expr::Action];
                a.extend((0..self.grammar.width).map(Expr::Input));
                a.extend((0..self.grammar.states).map(Expr::State)); a
            }
            Ty::Bool => vec![Expr::Lit(Value::Bool(false)),Expr::Lit(Value::Bool(true))],
        };
        for op in [Op::Add,Op::Sub,Op::Mul,Op::Xor,Op::Rem,Op::Eq,Op::Lt,Op::And,Op::Or] {
            if op.output() == ty { p.push(Expr::bin(op,hole(0,op.input()),hole(1,op.input()))); }
        }
        if ty == Ty::Int { p.push(Expr::AsInt(Box::new(hole(0,Ty::Bool)))); }
        else { p.push(Expr::Not(Box::new(hole(0,Ty::Bool)))); }
        p.push(Expr::If(Box::new(hole(0,Ty::Bool)),Box::new(hole(1,ty)),Box::new(hole(2,ty))));
        for f in &lib.functions {
            if f.body.ty(lib) == Ok(ty) {
                p.push(Expr::Call(f.id,(0..f.arity).map(|i| hole(i as u16,Ty::Int)).collect()));
            }
        }
        p
    }
    pub fn next(&mut self,lib: &Library,meter: &mut Meter) -> Result<Option<Expr>> {
        while let Some(Reverse((_,_,e))) = self.queue.pop() {
            meter.charge(e.nodes() as u64)?;
            if let Some((id,ty)) = e.first_hole() {
                let mut high = id;
                e.map(&mut |x| { if let Expr::Hole { id,.. } = x { high = high.max(*id); } None });
                let next = high.checked_add(1).filter(|n| *n < u16::MAX-16).ok_or(Error::Capacity)?;
                for production in self.productions(ty,next,lib) {
                    let child = e.substitute(id,&production); let size = child.nodes();
                    meter.charge(size as u64)?;
                    if size > self.grammar.max_nodes { continue; }
                    if self.queue.len() >= self.grammar.max_frontier { self.limited = true; continue; }
                    self.serial += 1; self.queue.push(Reverse((size,self.serial,child)));
                }
            } else { return Ok(Some(e)); }
        }
        Ok(None)
    }
}

fn state_contexts(cells: &[StateCell],data: &[Transition],lib: &Library,meter: &mut Meter) -> Result<Vec<Vec<i64>>> {
    let initial: Vec<_> = cells.iter().map(|c| c.initial).collect();
    let mut state = initial.clone(); let mut contexts = Vec::with_capacity(data.len());
    for t in data {
        meter.charge(1)?;
        if t.restart { state = initial.clone(); }
        contexts.push(state.clone());
        let ctx = Context { input: &t.pre,action: t.action,state: &state,args: &[] };
        state = cells.iter().map(|c| c.update.eval(ctx,lib,meter)?.int()).collect::<Result<Vec<_>>>()?;
    }
    Ok(contexts)
}
fn column(e: &Expr,data: &[Transition],states: &[Vec<i64>],lib: &Library,meter: &mut Meter) -> Result<Vec<i64>> {
    data.iter().zip(states).map(|(t,s)| e.eval(Context { input: &t.pre,action: t.action,state: s,args: &[] },lib,meter)?.int()).collect()
}
/// Exact backward requirement x OP hole = y. Ordinary enumeration remains.
fn inverse_candidates(data: &[Transition],states: &[Vec<i64>],target: usize,width: usize,nstate: usize,lib: &Library,meter: &mut Meter) -> Result<Vec<Expr>> {
    let leaves: Vec<_> = (0..width).map(Expr::Input).chain((0..nstate).map(Expr::State)).chain([Expr::Action]).collect();
    let mut answers = Vec::new();
    for leaf in leaves {
        let values = column(&leaf,data,states,lib,meter)?;
        for op in [Op::Add,Op::Sub,Op::Xor] {
            let required: Option<Vec<_>> = values.iter().zip(data).map(|(&x,t)| op.right_requirement(x,t.post[target])).collect();
            if let Some(r) = required {
                if !r.is_empty() && r.iter().all(|x| *x == r[0]) { answers.push(Expr::bin(op,leaf.clone(),Expr::integer(r[0]))); }
            }
        }
    }
    Ok(answers)
}
struct Best { patch: Option<Patch>,target_exact:bool,loss: f64,nodes: usize,candidates: u64,updates: u64,limited: bool,constructor: Option<u64> }
#[cfg(test)]
fn search_output(parent: &Program,data: &[Transition],job: &Job,cells: Vec<StateCell>,root: Expr,constructor: Option<u64>,cfg: &SearchConfig,lib: &Library,meter: &mut Meter,best: &mut Best) -> Result<bool> {
    search_output_focused(false,parent,data,job,cells,root,constructor,cfg,lib,meter,best)
}
fn search_output_focused(focused: bool,parent: &Program,data: &[Transition],job: &Job,cells: Vec<StateCell>,root: Expr,constructor: Option<u64>,cfg: &SearchConfig,lib: &Library,meter: &mut Meter,best: &mut Best) -> Result<bool> {
    best.updates += 1;
    let states = match state_contexts(&cells,data,lib,meter) { Ok(x) => x,Err(Error::Budget) => return Err(Error::Budget),Err(_) => return Ok(false) };
    if job.deterministic_contract {
        let mut seen = BTreeMap::new();
        for (t,s) in data.iter().zip(&states) {
            meter.charge((t.pre.len()+s.len()+1) as u64)?;
            let key = (t.pre.clone(),t.action,s.clone());
            if seen.insert(key,t.post[job.target]).is_some_and(|y| y != t.post[job.target]) { return Ok(false); }
        }
    }
    // Existing registers must retain their meanings. A learned schema from a
    // separate lifetime cannot overwrite live memory just because indices fit.
    if focused && !cells.starts_with(&parent.cells) { return Ok(false); }
    let other_loss = if focused {
        let mut unchanged = parent.clone(); unchanged.cells = cells.clone();
        unchanged.validate(parent.outputs.len(),lib)?;
        let rows = replay(&unchanged,data,lib,meter)?.0;
        rows.iter().zip(data).map(|(p,t)| p.iter().zip(&t.post).enumerate()
            .filter(|(j,_)| *j != job.target)
            .map(|(_,(&x,&y))| ((x as i128-y as i128).abs()) as f64).sum::<f64>()).sum()
    } else { 0.0 };
    let grammar = Grammar { width: parent.outputs.len(),states: cells.len(),max_nodes: cfg.max_expr_nodes,max_frontier: cfg.max_frontier };
    let mut enumeration = Enumerator::new(grammar,root.clone());
    // Generic inverse solving belongs to the BASELINE, not an acquired schema.
    // A constructor must actually instantiate its own root to receive credit.
    let mut preferred: std::collections::VecDeque<_> = if constructor.is_none() {
        inverse_candidates(data,&states,job.target,parent.outputs.len(),cells.len(),lib,meter)?.into()
    } else { std::collections::VecDeque::new() };
    if focused {
        let mut fast = Meter::new(meter.remaining()/2);
        let result = super::exact_completion::complete(&root,data,&states,job.target,parent.outputs.len(),cfg.max_expr_nodes,lib,&mut fast);
        meter.charge(fast.spent)?;
        match result {
            Ok(Some(e)) => preferred.push_front(e),
            Ok(None) | Err(Error::Budget) => {},
            Err(e) => return Err(e),
        }
    }
    loop {
        let expression = if let Some(e) = preferred.pop_front() { Some(e) } else { enumeration.next(lib,meter)? };
        let Some(expression) = expression else { best.limited |= enumeration.limited; return Ok(false); };
        meter.charge(expression.nodes() as u64)?;
        if expression.nodes() > cfg.max_expr_nodes { continue; }
        let patch = Patch { parent: job.parent,target: job.target,replacement: expression,cells: cells.clone() };
        let model = match patch.apply(parent,lib) { Ok(p) => p,Err(_) => continue };
        best.candidates += 1;
        let loss_result = if focused {
            column(&patch.replacement,data,&states,lib,meter).map(|values| {
                let target_loss: i128 = values.iter().zip(data).map(|(&x,t)| (x as i128-t.post[job.target] as i128).abs()).sum();
                (other_loss+target_loss as f64,target_loss==0)
            })
        } else {
            replay(&model,data,lib,meter).and_then(|(p,_)| absolute_loss(&p,data)).map(|v| (v,v==0.0))
        };
        let (loss,target_exact) = match loss_result { Ok(v) => v,Err(Error::Budget) => return Err(Error::Budget),Err(_) => continue };
        if loss < best.loss || (loss == best.loss && model.nodes() < best.nodes) {
            best.target_exact=target_exact; best.loss = loss; best.nodes = model.nodes(); best.patch = Some(patch); best.constructor = constructor;
        }
        best.limited |= enumeration.limited;
        if target_exact && (!focused || best.patch.is_some()) { return Ok(true); }
    }
}
fn baseline_single(focused: bool,parent: &Program,data: &[Transition],job: &Job,cfg: &SearchConfig,lib: &Library,meter: &mut Meter,best: &mut Best) -> Result<()> {
    let root = || Expr::Hole { id: 0,ty: Ty::Int };
    if !job.stateless_impossible || !parent.cells.is_empty() {
        let cap = (meter.remaining()/4).max(1);
        let mut local = Meter::new(cap);
        let found = search_output_focused(focused,parent,data,job,parent.cells.clone(),root(),None,cfg,lib,&mut local,best);
        meter.charge(local.spent)?;
        if matches!(found,Ok(true)) { return Ok(()); }
        if let Err(e) = found { if e != Error::Budget { return Err(e); } }
    }
    if parent.cells.len() >= cfg.max_state { return Ok(()); }
    let mut updates = Enumerator::new(Grammar { width: parent.outputs.len(),states: parent.cells.len()+1,max_nodes: cfg.max_expr_nodes.min(5),max_frontier: cfg.max_frontier },root());
    for update_index in 0..cfg.max_updates {
        let Some(update) = updates.next(lib,meter)? else { break; };
        for (initial_index,initial) in [0,1,-1].into_iter().enumerate() {
            let mut cells = parent.cells.clone(); cells.push(StateCell { initial,update: update.clone() });
            // In focused mode every remaining memory-update hypothesis gets
            // an equal reservation. Geometric allocation silently starves late
            // grammar leaves when noisy inputs prevent collision pruning.
            let alternatives = (cfg.max_updates-update_index)*3-initial_index;
            let cap = if focused { (meter.remaining()/alternatives as u64).max(1) }
                else { (meter.remaining()/4).max(1) };
            let mut local = Meter::new(cap);
            let found = search_output_focused(focused,parent,data,job,cells,root(),None,cfg,lib,&mut local,&mut *best);
            meter.charge(local.spent)?;
            if matches!(found,Ok(true)) { return Ok(()); }
            if let Err(e) = found { if e != Error::Budget { return Err(e); } }
            if meter.remaining() < 10 { return Err(Error::Budget); }
        }
    }
    best.limited |= updates.limited;
    Ok(())
}
/// A finite joint lane can cross individually useless memory steps. No domain
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
/// Hints change search order, never acceptance or authority. A separate broad
/// lane retains at least one quarter of the budget if learned search fails.
/// Finding a zero-training-loss candidate stops search, not prospective checking.
pub fn synthesize(parent: &Program,data: &[Transition],lib: &Library,cfg: &SearchConfig,hints: &[Hint]) -> Result<SearchTrace> {
    synthesize_requested(parent,data,lib,cfg,hints,None)
}
pub fn synthesize_requested(parent: &Program,data: &[Transition],lib: &Library,cfg: &SearchConfig,hints: &[Hint],requested: Option<usize>) -> Result<SearchTrace> {
    cfg.validate()?; lib.validate(parent.outputs.len())?; parent.validate(parent.outputs.len(),lib)?;
    let actions = data.iter().map(|t| t.action).max().unwrap_or(0).checked_add(1).ok_or(Error::Capacity)?;
    validate_data(data,parent.outputs.len(),actions)?;
    let mut meter = Meter::new(cfg.work);
    let job = discover_requested_job(parent,data,lib,cfg.deterministic,requested,&mut meter)?.ok_or(Error::NoEvidence)?;
    let (old,_) = replay(parent,data,lib,&mut meter)?;
    let old_loss = absolute_loss(&old,data)?;
    let mut best = Best { patch: None,target_exact:false,loss: old_loss,nodes: parent.nodes(),candidates: 0,updates: 0,limited: false,constructor: None };
    // Search in increasing structural extensions, not huge per-hint blocks.
    // A narrow proposal first tries leaf bindings; larger computations get a
    // turn only after every other schema had that same opportunity. This keeps
    // a new over-specialization from consuming the entire learned-search lane.
    // The cost of all unsuccessful rounds is charged, including repeated setup.
    let reserve = if hints.is_empty() { meter.remaining() } else { meter.remaining()/4 };
    let mut constructor_work = 0;
    let usable = hints.len().min(32);
    let mut solved_target = false;
    'rounds: for extension in 0..=cfg.max_expr_nodes {
        for hint in hints.iter().take(usable) {
            let nodes = hint.output.nodes().saturating_add(extension);
            if nodes > cfg.max_expr_nodes || (extension > 0 && hint.output.first_hole().is_none()) { continue; }
            let available = meter.remaining().saturating_sub(reserve);
            let cap = available / usable.max(1) as u64;
            if cap == 0 { break 'rounds; }
            let mut local = Meter::new(cap);
            let mut bounded = cfg.clone(); bounded.max_expr_nodes = nodes;
            let result = search_output_focused(requested.is_some(),parent,data,&job,hint.cells.clone(),hint.output.clone(),Some(hint.constructor_id),&bounded,lib,&mut local,&mut best);
            constructor_work += local.spent;
            meter.charge(local.spent)?;
            if matches!(result,Ok(true)) { solved_target = true; break 'rounds; }
            if let Err(e) = result { if e != Error::Budget { return Err(e); } }
        }
    }
    let mut baseline_work = 0;
    if best.loss != 0.0 && !solved_target {
        let mut broad = Meter::new(meter.remaining());
        let result = baseline(requested.is_some(),parent,data,&job,cfg,lib,&mut broad,&mut best);
        baseline_work = broad.spent;
        meter.charge(broad.spent)?;
        if let Err(e) = result { if e != Error::Budget { return Err(e); } }
    }
    Ok(SearchTrace { job,candidates: best.candidates,updates_tested: best.updates,work: meter.spent,
        baseline_work,constructor_work,exhausted: meter.remaining() == 0,frontier_limited: best.limited,
        winner: best.patch,training_loss: best.loss,constructor_id: best.constructor,search_policy_id:None,prospectively_accepted: false })
}
#[cfg(test)]
mod tests {
    use super::*;
    pub fn delayed() -> Vec<Transition> {
        let mut hidden = 0; let mut obs = 0; let mut data = Vec::new();
        for (i,a) in [1,1,0,1,0,0,1,0,1,1,0,0].into_iter().enumerate() {
            let post = hidden; hidden = a as i64;
            data.push(Transition { id: i as u64,restart: i == 0,pre: vec![obs],action: a,post: vec![post] }); obs = post;
        }
        data
    }
    #[test] fn collision_requires_available_history_only_under_contract() {
        let data = delayed(); let p = Program::identity(1); let lib = Library::default();
        let d = discover_job(&p,&data,&lib,true,&mut Meter::new(10000)).unwrap().unwrap();
        assert!(d.stateless_impossible); assert!(!d.witness_ids.is_empty());
        let s = discover_job(&p,&data,&lib,false,&mut Meter::new(10000)).unwrap().unwrap(); assert!(!s.stateless_impossible);
    }
    #[test] fn discovers_state_update_without_delay_operator() {
        let p = Program::identity(1); let lib = Library::default();
        let result = synthesize(&p,&delayed(),&lib,&SearchConfig { deterministic: true,..Default::default() },&[]).unwrap();
        let learned = result.winner.unwrap().apply(&p,&lib).unwrap();
        assert_eq!(result.training_loss,0.0); assert!(!learned.cells.is_empty());
        assert_eq!(absolute_loss(&replay(&learned,&delayed(),&lib,&mut Meter::new(10000)).unwrap().0,&delayed()).unwrap(),0.0);
    }
    #[test] fn missing_raw_source_and_inverse_requirement() {
        let data: Vec<_> = (0..8).map(|i| Transition { id: i as u64,restart: true,pre: vec![0,i],action: 0,post: vec![i+17,i] }).collect();
        let p = Program::identity(2); let lib = Library::default();
        let trace = synthesize(&p,&data,&lib,&SearchConfig::default(),&[]).unwrap();
        assert_eq!(trace.training_loss,0.0); assert_eq!(trace.job.exposed_sources,vec![0,1]);
        assert!(trace.winner.unwrap().replacement.nodes() <= 3);
    }
    #[test] fn constructor_does_not_receive_credit_for_unrelated_inverse_solver() {
        let data: Vec<_> = (0..8).map(|i| Transition { id: i as u64,restart: true,pre: vec![i],action: 0,post: vec![i+17] }).collect();
        let p = Program::identity(1); let lib = Library::default(); let mut meter = Meter::new(10000);
        let job = discover_job(&p,&data,&lib,true,&mut meter).unwrap().unwrap();
        let mut best = Best { patch: None,target_exact:false,loss: 136.0,nodes: p.nodes(),candidates: 0,updates: 0,limited: false,constructor: None };
        let solved = search_output(&p,&data,&job,vec![],Expr::integer(0),Some(9),&SearchConfig::default(),&lib,&mut meter,&mut best).unwrap();
        assert!(!solved); assert!(best.patch.is_none()); assert!(best.constructor.is_none());
    }
    #[test] fn stale_patch_cannot_replace_new_parent() {
        let p = Program::identity(1); let patch = Patch { parent: 0,target: 0,replacement: Expr::integer(1),cells: vec![] };
        assert_eq!(patch.apply(&p,&Library::default()),Err(Error::Stale));
    }
    #[test] fn reject_unrecorded_reset_and_duplicate_evidence() {
        let mut data = delayed(); data[1].pre = vec![42]; assert!(validate_data(&data,1,2).is_err());
        data = delayed(); data[1].id = data[0].id; assert!(validate_data(&data,1,2).is_err());
    }
    #[test] fn finite_queue_does_not_claim_exhaustive_search() {
        let mut e = Enumerator::new(Grammar { width: 4,states: 0,max_nodes: 7,max_frontier: 8 },Expr::Hole { id: 0,ty: Ty::Int });
        let mut m = Meter::new(2000);
        for _ in 0..20 { if e.next(&Library::default(),&mut m).is_err() { break; } }
        assert!(e.limited); assert!(m.spent <= m.limit);
    }
}
