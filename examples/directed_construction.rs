//! Repeatable development fixture, not an independent intelligence benchmark.
//! The learner sees one raw integer channel and two opaque actions, not `hidden`.
use aeterna_v1::construction::{Config, ExternalPost, ProtectedConstructionRuntime};
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::{EvoConfig,EvoPhase,HumanProtectionEvidence};
use std::io::Write;
fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence { human_present: false,physical_effect_possible: false,predicted_harm_probability: 0.0,hazard_confidence: 1.0,emergency_stop: false }
}
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let config = Config { minimum_training: 6,shadow_steps: 4,search_every: 2,search: SearchConfig { deterministic: true,..Default::default() },..Default::default() };
    let carrier = EvoPhase::new(EvoConfig { sensory_cells: 1,motor_cells: 2,..Default::default() });
    let mut runtime = ProtectedConstructionRuntime::new(carrier,config,vec![0])?;
    let mut hidden = 0_i64;
    for _ in 0..32 {
        runtime.step(None,safe,|action| {
            let post = hidden; hidden = action as i64;
            Ok(ExternalPost { observation: vec![post],cost: 1 })
        })?;
    }
    let acquired = runtime.learner().report();
    if acquired.state_cells == 0 || acquired.promotions == 0 { return Err(format!("no validated state acquisition: {acquired:?}").into()); }
    let checkpoint = runtime.checkpoint()?;
    runtime.restore_cognition(&checkpoint)?;
    let mut goal_actions = 0;
    while runtime.learner().current() != [1] && goal_actions < 3 {
        runtime.step(Some(&[1]),safe,|action| {
            let post = hidden; hidden = action as i64;
            Ok(ExternalPost { observation: vec![post],cost: 1 })
        })?;
        goal_actions += 1;
    }
    let reached = runtime.learner().current() == [1];
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
        "kind":"directed-construction-development","independent_qualification":false,
        "after_exploration":acquired,"learned_program":runtime.learner().program(),
        "checkpoint_bytes":checkpoint.len(),"goal_actions":goal_actions,"goal_reached":reached,
        "promotions":runtime.learner().promotions()
    }))?);
    if let Some(path) = std::env::args().nth(1) {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(&checkpoint)?;
    }
    if !reached { return Err("model did not reach the observed goal".into()); }
    Ok(())
}
