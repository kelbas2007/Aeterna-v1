//! Acquire noisy conditional laws, save them, then sense hidden context and
//! reach a factual goal with learning frozen in a separate process.
use aeterna_v1::carrier::{
    PhaseAdaptiveConfig, PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig, PhaseRuleConfig,
    PhaseRuleLanguage, PhaseUncertainConfig,
};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::{Read, Write};
fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn measured(s: [i32; 2], tick: usize) -> Vec<Option<f32>> {
    s.iter()
        .enumerate()
        .map(|(j, &x)| {
            Some(
                (x as f32 / 257.0 + if (tick + j) % 2 == 0 { 0.0003 } else { -0.0003 })
                    .rem_euclid(1.0),
            )
        })
        .collect()
}
fn act(a: usize, s: [i32; 2]) -> [i32; 2] {
    if a == 2 {
        s
    } else {
        [if (s[1] >= 130) == (a == 0) { 193 } else { 64 }, s[1]]
    }
}
fn fresh() -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: 2,
        motor_cells: 3,
        dormant_cells: 34,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_rule_learning_with_language(
        PhaseRuleConfig {
            planning_depth: 2,
            ..Default::default()
        },
        PhaseRuleLanguage::CircularAffine
    ));
    assert!(e.enable_phase_adaptive_rules(PhaseAdaptiveConfig {
        min_inlier_fraction: 1.0,
        ..Default::default()
    }));
    assert!(e.enable_phase_uncertain_observation(
        PhasePartialConfig::default(),
        PhaseUncertainConfig {
            measurement_radius: 0.0004,
            ..Default::default()
        }
    ));
    e
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: uncertain_learning <learn|run> <checkpoint.json>".into());
    }
    let e = if args[1] == "learn" {
        let mut rt = ScientificRuntime::new(fresh())?;
        let mut rng = 0xABCD1234u64;
        for tick in 0..160 {
            let mut state = [0; 2];
            for x in &mut state {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                *x = (rng % 257) as i32;
            }
            rt.observe_external_partial(&measured(state, tick))?;
            rt.set_partial_goal(&[Some(0.5), None])?;
            rt.step_partial(
                |_| Some(safe()),
                |a| Ok((measured(act(a, state), tick + 1), 0.0)),
            )?;
        }
        let bytes = rt.organism().online_checkpoint_bytes()?;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        EvoPhase::from_online_checkpoint(&bytes)?
    } else {
        let mut bytes = Vec::new();
        std::fs::File::open(&args[2])?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        EvoPhase::from_online_checkpoint(&bytes)?
    };
    if !e.phase_uncertain_observation_enabled()
        || e.config().sensory_cells != 2
        || e.config().motor_cells != 3
    {
        return Err("this example requires its uncertainty checkpoint".into());
    }
    let mut rt = ScientificRuntime::new(e)?;
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mut state = [21, 220];
    rt.observe_external_partial(&[Some(21.0 / 257.0), None])?;
    rt.set_partial_goal(&[Some(193.0 / 257.0), None])?;
    let mut actions = Vec::new();
    for tick in 0..2 {
        if !matches!(
            rt.step_partial(
                |_| Some(safe()),
                |a| {
                    actions.push(a);
                    state = act(a, state);
                    Ok((measured(state, tick), 0.0))
                }
            )?,
            StepOutcome::Executed { learned: false, .. }
        ) {
            return Err("frozen execution failed".into());
        }
    }
    if actions != [2, 0]
        || !rt.goal_reached()?
        || rt.organism().phase_native_learned_fingerprint() != fingerprint
    {
        return Err("acquired sensing/conditional goal or freeze failed".into());
    }
    println!("mode={} checkpoint_v=7 hidden_context=true actions={actions:?} factual_goal=true frozen_knowledge=true",args[1]);
    Ok(())
}
