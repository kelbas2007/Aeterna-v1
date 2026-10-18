//! Deterministic development exercise, not an independent intelligence test.
//! The environment alone owns the hidden register and observation law. The
//! persistent registry receives PRE/action/POST only. No solution or constructor
//! is inserted by this fixture, and future records are obtained after begin_trial.
use aeterna_v1::construction::constructors::Registry;
use aeterna_v1::construction::synthesis::{SearchConfig,Transition};
use aeterna_v1::construction::vm::{Library,Program};

fn observations(offset: u64,seed: u64,sources: &[usize],episodes: usize) -> Vec<Transition> {
    let mut data = Vec::new(); let mut rng = seed;
    for _ in 0..episodes {
        let mut input = vec![0];
        for _ in 0..4 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            input.push(((rng >> 32)%17) as i64-8);
        }
        let factor: i64 = sources.iter().map(|&i| input[i]).sum();
        let mut hidden = 0;
        for (i,action) in [1usize,1,0,2,0,1].into_iter().enumerate() {
            let mut post = input.clone(); post[0] = hidden*factor; hidden = action as i64;
            data.push(Transition { id: offset+data.len() as u64,restart: i == 0,pre: input,action,post: post.clone() });
            input = post;
        }
    }
    data
}
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let mut registry = Registry::default();
    let library = Library::default(); // same exact library in all three arms
    let config = SearchConfig { work: 8_000_000,max_expr_nodes: 9,max_frontier: 32768,max_updates: 16,max_state: 2,deterministic: true };
    let mut reports = Vec::new();
    let tasks: &[&[usize]] = &[&[1],&[2],&[1,2],&[3,4],&[1,3],&[2,4],&[1,4],&[2,3],
        &[1,2,3],&[1,2,4],&[1,3,4],&[2,3,4],&[1,2,3],&[1,2,4],&[1,3,4],&[2,3,4]];
    for (index,sources) in tasks.iter().enumerate() {
        let acquisition = observations(0,31+index as u64*101,sources,4);
        let trial = registry.begin_trial(&Program::identity(5),&acquisition,&library,&config)?;
        // The program and all counterfactual arms are already frozen here.
        let future = observations(acquisition.len() as u64,901+index as u64*103,sources,3);
        let report = registry.complete_trial(trial,&future,true)?;
        eprintln!("task {index}: generation={} accepted={} future={} base_work={} ancestor_work={} learned_work={} acquired={} recursive_value={}",report.generation,report.model_accepted,report.candidate.future_loss,report.same_library.work,report.without_winner_subtree.work,report.candidate.work,report.constructors_acquired,report.recursive_search_value);
        reports.push(report);
    }
    registry.validate(&library)?;
    let guard_outcomes: usize = registry.constructors().iter().map(|c| registry.guard_evidence(c.id).len()).sum();
    let executed_constructors = reports.iter().filter(|r| r.constructor.is_some()).count();
    if guard_outcomes != executed_constructors { return Err("prospective constructor outcomes did not reach applicability learning".into()); }
    let saved = serde_json::to_vec(&registry)?;
    let restored: Registry = serde_json::from_slice(&saved)?;
    restored.validate(&library)?;
    if serde_json::to_vec(&restored)? != saved { return Err("registry checkpoint did not preserve exact state".into()); }
    let generation = registry.constructors().iter().map(|c| c.generation).max().unwrap_or(0);
    let mut latest = std::collections::BTreeMap::new();
    for report in &reports { if let Some(id) = report.constructor { latest.insert(id,report); } }
    let improved = reports.last().is_some_and(|r| r.recursive_search_value)
        && latest.values().any(|r| r.generation >= 2 && r.recursive_search_value)
        && latest.values().any(|r| r.generation >= 3 && r.recursive_search_value)
        && reports.iter().all(|r| r.model_accepted && r.quality_retained && r.candidate.future_loss == 0.0);
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
        "kind":"recursive-construction-development","independent_qualification":false,
        "hand_injected_solutions":0,"hand_injected_constructors":0,
        "same_library_in_all_arms":true,"max_generation":generation,
        "prospective_guard_outcomes":guard_outcomes,
        "nontrivial_applicability_guards":registry.constructors().iter().filter(|c| c.guard != aeterna_v1::construction::vm::Expr::Lit(aeterna_v1::construction::vm::Value::Bool(true))).count(),
        "recursive_improvement_observed":improved,"reports":reports,
        "constructors":registry.constructors(),"ledger":registry.meta_ledger(),
        "checkpoint_bytes":saved.len(),"checkpoint_round_trip":true
    }))?);
    if generation < 3 || !improved { return Err("retained multi-generation recursion criterion not met; inspect the entire sequence".into()); }
    Ok(())
}
