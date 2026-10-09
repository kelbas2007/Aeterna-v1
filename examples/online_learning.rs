//! External simulator demonstrating cold learning and persistence across OS
//! processes. All world transitions stay in this adapter, outside cognition.
use aeterna_v1::carrier::{PhaseNativeConfig, PhaseOnlineConfig};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::Write;

fn observation(node: usize) -> Vec<f32> {
    let pattern = node * 7 % 32;
    (0..5)
        .map(|bit| if pattern & (1 << bit) == 0 { 0.1 } else { 0.9 })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: online_learning <learn|run> <checkpoint.json>".into());
    }
    let learning = args[1] == "learn";
    let evo = if learning {
        let mut evo = EvoPhase::new(EvoConfig {
            sensory_cells: 5,
            motor_cells: 3,
            dormant_cells: 256,
            weight_learning_rate: 1.0,
            phase_learning_rate: 1.0,
            min_recruit_support: 1,
            ..Default::default()
        });
        evo.enable_phase_native_planning(PhaseNativeConfig {
            horizon: 32,
            ..Default::default()
        });
        assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 32 }));
        evo
    } else {
        // Bound file reads as well as decoder allocations.
        let file = std::fs::File::open(&args[2])?;
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(
            &mut std::io::Read::take(file, 16 * 1024 * 1024 + 1),
            &mut bytes,
        )?;
        EvoPhase::from_online_checkpoint(&bytes)?
    };
    let mut rt = ScientificRuntime::new(evo)?;
    rt.set_model_learning_enabled(learning);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mut node = 0usize;
    rt.observe_external(&observation(node))?;
    rt.set_goal(&observation(10))?;
    let mut actions = 0;
    while !rt.goal_reached()? && actions < 1000 {
        let result = rt.step_unified(
            |_| {
                Some(HumanProtectionEvidence {
                    human_present: false,
                    physical_effect_possible: false,
                    predicted_harm_probability: 0.0,
                    hazard_confidence: 1.0,
                    emergency_stop: false,
                })
            },
            |action| {
                node = match (action + node) % 3 {
                    0 => (node + 1) % 11,
                    1 => (node * 3 + 2) % 11,
                    _ => node,
                };
                Ok((observation(node), f32::from(node == 10)))
            },
        )?;
        if !matches!(result, StepOutcome::Executed { .. }) {
            return Err(format!("execution stopped: {result:?}").into());
        }
        actions += 1;
    }
    if !rt.goal_reached()? {
        return Err("goal was not reached within the action budget".into());
    }
    if learning {
        let bytes = rt.organism().online_checkpoint_bytes()?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[2])?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    } else if fingerprint != rt.organism().phase_native_learned_fingerprint() {
        return Err("frozen evaluation changed learned knowledge".into());
    }
    println!(
        "mode={} goal_reached=true actions={} learned_states={} circuits={} frozen_unchanged={}",
        args[1],
        actions,
        rt.organism().phase_native_receptor_count(),
        rt.organism().phase_native_circuits().len(),
        !learning
    );
    Ok(())
}
