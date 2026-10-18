//! Prospective DEVELOPMENT stress test, not independent intelligence proof.
//! All laws coexist from time zero. The world alone owns channel roles, random
//! permutations, exogenous streams and hidden state. The learner receives only
//! PRE/action/POST. The three paired arms share exactly the same solver budgets.
use aeterna_v1::construction::{Config, ExternalPost, ProtectedConstructionRuntime};
use aeterna_v1::construction::curriculum::SelectionMode;
use aeterna_v1::construction::synthesis::{SearchConfig,replay};
use aeterna_v1::construction::vm::Meter;
use aeterna_v1::{EvoConfig,EvoPhase,HumanProtectionEvidence};

#[derive(Clone)]
struct World { state: u64, hidden: i64, obs: Vec<i64>, map: Vec<usize> }
fn draw(s: &mut u64) -> u64 {
    *s = s.wrapping_add(0x9e3779b97f4a7c15);
    let mut x = *s; x = (x^(x>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x^(x>>27)).wrapping_mul(0x94d049bb133111eb); x^(x>>31)
}
impl World {
    fn new(seed: u64) -> Self {
        let mut state = seed; let mut map: Vec<usize> = (0..8).collect();
        for j in (1..8).rev() { let k = draw(&mut state) as usize%(j+1); map.swap(j,k); }
        let mut w = Self { state,hidden: 0,obs: vec![0;8],map };
        for j in 4..7 { w.obs[w.map[j]] = (draw(&mut w.state)%5) as i64-2; }
        w.obs[w.map[7]] = (draw(&mut w.state)%20001) as i64-10000; w
    }
    fn step(&mut self,action: usize) -> Vec<i64> {
        let (x,y,z) = (self.obs[self.map[4]],self.obs[self.map[5]],self.obs[self.map[6]]);
        let h = self.hidden; self.hidden = action as i64;
        for (j,v) in [h*x,h*y,h*(x+y),h*(x+z)].into_iter().enumerate() { self.obs[self.map[j]] = v; }
        for j in 4..7 { self.obs[self.map[j]] = (draw(&mut self.state)%5) as i64-2; }
        self.obs[self.map[7]] = (draw(&mut self.state)%20001) as i64-10000;
        self.obs.clone()
    }
}
fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence { human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false }
}
fn run(seed: u64,mode: SelectionMode,steps: usize) -> Result<serde_json::Value,Box<dyn std::error::Error>> {
    let mut world = World::new(seed);
    let cfg = Config { width:8,actions:3,minimum_training:8,shadow_steps:12,search_every:4,journal_capacity:512,planning_depth:2,planning_work:20000,
        search: SearchConfig { work:2_000_000,max_expr_nodes:9,max_frontier:32768,max_updates:12,max_state:4,deterministic:false } };
    let mut r = ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig { sensory_cells:8,motor_cells:3,..Default::default() }),cfg,world.obs.clone())?;
    r.enable_curriculum(mode)?;
    for _ in 0..steps { r.step(None,safe,|a| Ok(ExternalPost { observation:world.step(a),cost:1 }))?; }
    let snapshot = r.checkpoint()?; r.restore_cognition(&snapshot)?;
    let before = r.learner().report();
    let p = r.learner().program().clone(); let lib = r.learner().library().clone();
    let history = r.learner().records().iter().map(|r| r.transition.clone()).collect::<Vec<_>>();
    if history.iter().skip(1).any(|t| t.restart) { return Err("unexpected curriculum/environment reset".into()); }
    let mut meter = Meter::new(10_000_000); let mut state = replay(&p,&history,&lib,&mut meter)?.1;
    let mut probe = world.clone(); let mut errors = vec![0usize;4]; let mut erased = vec![0usize;4]; let mut cold = vec![0usize;4];
    let mut arng = seed^0x636f756e74657266;
    for _ in 0..48 {
        let a = (draw(&mut arng)%3) as usize;
        let (prediction,next) = p.step(&probe.obs,a,&state,&lib,&mut meter)?;
        let erased_prediction = p.step(&probe.obs,a,&p.initial_state(),&lib,&mut meter)?.0;
        let old = probe.obs.clone(); let actual = probe.step(a);
        for j in 0..4 { let k=world.map[j]; errors[j]+=usize::from(prediction[k]!=actual[k]); erased[j]+=usize::from(erased_prediction[k]!=actual[k]); cold[j]+=usize::from(old[k]!=actual[k]); }
        state=next;
    }
    if r.checkpoint()? != snapshot { return Err("frozen probe changed cognition".into()); }
    // Goal is declared only AFTER the frozen probe. Its visible setpoints expose
    // no transition law; the real action still passes the existing safety gate.
    let mut goal = vec![None;8]; for j in 0..4 { goal[world.map[j]]=Some(0); }
    let mut goal_actions=Vec::new(); let mut reached=goal.iter().zip(&world.obs).all(|(g,&x)|g.is_none_or(|v|v==x));
    for _ in 0..3 {
        if reached { break; }
        let a = r.step_masked(&goal,safe,|a| Ok(ExternalPost { observation:world.step(a),cost:1 }))?;
        goal_actions.push(a); reached=goal.iter().zip(&world.obs).all(|(g,&x)|g.is_none_or(|v|v==x));
    }
    let learned=errors.iter().filter(|&&n|n==0).count();
    eprintln!("seed={seed} mode={mode:?} learned={learned}/4 constructors={} generation={} work={} error={:?}",before.constructors,before.constructor_generation,before.construction_work,before.last_error);
    Ok(serde_json::json!({"seed":seed,"mode":mode,"steps":steps,"mapping_evaluator_only":world.map,
        "frozen_errors":errors,"erased_memory_errors":erased,"cold_errors":cold,"learned_laws":learned,
        "before_probe":before,"curriculum":r.learner().curriculum(),"goal_actions":goal_actions,"goal_reached":reached,
        "checkpoint_bytes":snapshot.len(),"checkpoint_round_trip":true,"training_restarts":0,
        "program":p,"recursive_reports":r.learner().registry().meta_ledger().reports()}))
}
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args().collect();
    let seed=args.get(1).map(|x|x.parse()).transpose()?.unwrap_or(917u64);
    let worlds=args.get(2).map(|x|x.parse()).transpose()?.unwrap_or(4usize);
    let steps=args.get(3).map(|x|x.parse()).transpose()?.unwrap_or(160usize);
    if worlds>32 || !(32..=384).contains(&steps) { return Err("bounded experiment configuration required".into()); }
    let mut rows=Vec::new();
    for j in 0..worlds { for mode in [SelectionMode::Progress,SelectionMode::Fair,SelectionMode::LargestResidual] { rows.push(run(seed.wrapping_add(j as u64*1009),mode,steps)?); } }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"kind":"dc2-autonomy-development","independent_qualification":false,
        "no_task_schedule":true,"no_injected_programs_or_constructors":true,"worlds":worlds,"seed":seed,"steps":steps,"rows":rows}))?);
    Ok(()) // Scientific negative outcomes are results, not suppressed artifacts.
}
