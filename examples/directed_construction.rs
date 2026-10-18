//! Authored development fixture, NOT an independent intelligence benchmark.
//! A frozen non-alternating probe rejects the action-negation confound found in
//! the initial b6221198 demo. Probe outcomes cannot train the frozen predictor.
use aeterna_v1::construction::{Config, ExternalPost, ProtectedConstructionRuntime};
use aeterna_v1::construction::synthesis::{SearchConfig,replay};
use aeterna_v1::construction::vm::Meter;
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
    for _ in 0..64 {
        runtime.step(None,safe,|action| {
            let post = hidden; hidden = action as i64;
            Ok(ExternalPost { observation: vec![post],cost: 1 })
        })?;
    }
    let acquired = runtime.learner().report();
    if acquired.state_cells == 0 || acquired.promotions == 0 { return Err(format!("no validated state acquisition: {acquired:?}").into()); }
    let checkpoint = runtime.checkpoint()?;
    runtime.restore_cognition(&checkpoint)?;
    let frozen = runtime.learner().program().clone();
    let library = runtime.learner().library().clone();
    let history = runtime.learner().records().iter().map(|r| r.transition.clone()).collect::<Vec<_>>();
    let mut meter = Meter::new(1000000);
    let mut state = replay(&frozen,&history,&library,&mut meter)?.1;
    let mut frozen_errors = 0; let mut confound_errors = 0; let mut cold_errors = 0;
    for action in [0,0,0,1,0,1,1,1].into_iter().cycle().take(24) {
        let pre = runtime.learner().current().to_vec();
        let (prediction,next) = frozen.step(&pre,action,&state,&library,&mut meter)?;
        runtime.step_action(action,safe,|a| {
            let post = hidden; hidden = a as i64;
            Ok(ExternalPost { observation: vec![post],cost: 1 })
        })?;
        let actual = runtime.learner().current();
        frozen_errors += usize::from(prediction != actual);
        confound_errors += usize::from([i64::from(action == 0)] != actual);
        cold_errors += usize::from(pre != actual);
        state = next;
    }
    let mut goal_actions = 0;
    while runtime.learner().current() != [0] && goal_actions < 3 {
        runtime.step(Some(&[0]),safe,|action| {
            let post = hidden; hidden = action as i64;
            Ok(ExternalPost { observation: vec![post],cost: 1 })
        })?;
        goal_actions += 1;
    }
    let reached = runtime.learner().current() == [0];
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
        "kind":"directed-construction-development","independent_qualification":false,
        "autonomous_acquisition_steps":64,"after_exploration":acquired,
        "frozen_program":frozen,"frozen_probe_steps":24,"frozen_probe_errors":frozen_errors,
        "action_negation_confound_errors":confound_errors,"cold_identity_errors":cold_errors,
        "checkpoint_bytes":checkpoint.len(),"goal_actions":goal_actions,"goal_reached":reached,
        "promotions":runtime.learner().promotions()
    }))?);
    if frozen_errors != 0 || confound_errors == 0 || cold_errors == 0 { return Err("frozen transfer/negative-control failure".into()); }
    if !reached { return Err("model did not reach the observed goal".into()); }
    if let Some(path) = std::env::args().nth(1) {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(&checkpoint)?;
    }
    Ok(())
}
