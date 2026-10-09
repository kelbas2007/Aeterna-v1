//! Acquire opaque actions and visibility from factual experience; reconstruct
//! all hidden modular roots and measure which root is real after disk restart.
use aeterna_v1::carrier::{
    PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig, PhaseRuleConfig, PhaseRuleLanguage,
};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::{Read, Write};

struct World([u32; 2]);
impl World {
    fn full(&self) -> Vec<f32> {
        self.0.iter().map(|&x| x as f32 / 257.0).collect()
    }
    fn partial(&self) -> Vec<Option<f32>> {
        vec![Some(self.full()[0]), None]
    }
    fn act(&mut self, a: usize) {
        self.0[0] = match a {
            0 => (2 * self.0[1] + 17) % 257,
            1 => (self.0[1] + 31) % 257,
            _ => unreachable!(),
        };
    }
}
fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn fresh() -> EvoPhase {
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: 2,
        motor_cells: 2,
        dormant_cells: 8,
        ..Default::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning_with_language(
        PhaseRuleConfig::default(),
        PhaseRuleLanguage::CircularAffine
    ));
    evo
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: inverse_inference <learn|run> <checkpoint.json>".into());
    }
    let learning = args[1] == "learn";
    let mut world = World([7, 31]);
    let mut rule_actions = 0;
    let mut mask_actions = 0;
    let evo = if learning {
        let mut rt = ScientificRuntime::new(fresh())?;
        while !(0..2).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed) {
            if rule_actions == 80 {
                return Err("rule acquisition budget exhausted".into());
            }
            world.0 = [
                ((rule_actions * 73 + 7) % 257) as u32,
                ((rule_actions * rule_actions * 31 + 17 * rule_actions + 31) % 257) as u32,
            ];
            rt.observe_external(&world.full())?;
            rt.set_goal(&[0.1234567, 0.2345678])?;
            if !matches!(
                rt.step(
                    |_| Some(safe()),
                    |a| {
                        world.act(a);
                        Ok(world.full())
                    }
                )?,
                StepOutcome::Executed { .. }
            ) {
                return Err("rule acquisition did not execute".into());
            }
            rule_actions += 1;
        }
        let mut evo = EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes()?)?;
        assert!(evo.enable_phase_partial_observation(PhasePartialConfig::default()));
        let mut rt = ScientificRuntime::new(evo)?;
        while !(0..2).all(|a| rt.organism().phase_partial_mask_info(a).unwrap().confirmed) {
            if mask_actions == 30 {
                return Err("visibility acquisition budget exhausted".into());
            }
            rt.observe_external_partial(&world.full().into_iter().map(Some).collect::<Vec<_>>())?;
            rt.set_partial_goal(&[Some(0.1234567), None])?;
            if !matches!(
                rt.step_partial(
                    |_| Some(safe()),
                    |a| {
                        world.act(a);
                        Ok((world.partial(), 0.0))
                    }
                )?,
                StepOutcome::Executed { .. }
            ) {
                return Err("visibility acquisition did not execute".into());
            }
            mask_actions += 1;
        }
        let mut evo = EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes()?)?;
        assert!(evo.set_phase_inverse_inference(true));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[2])?;
        file.write_all(&evo.online_checkpoint_bytes()?)?;
        file.sync_all()?;
        evo
    } else {
        let mut bytes = Vec::new();
        std::fs::File::open(&args[2])?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        EvoPhase::from_online_checkpoint(&bytes)?
    };
    if evo.config().sensory_cells != 2
        || evo.config().motor_cells != 2
        || !evo.phase_inverse_inference_enabled()
        || !(0..2).all(|a| {
            evo.phase_rule_action_info(a).is_some_and(|i| i.confirmed)
                && evo.phase_partial_mask_info(a).is_some_and(|i| i.confirmed)
        })
    {
        return Err("this demo requires its acquired inverse model".into());
    }
    let mut rt = ScientificRuntime::new(evo)?;
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.0 = [29, 181];
    rt.observe_external_partial(&world.partial())?;
    rt.set_partial_goal(&[Some(212.0 / 257.0), None])?;
    let mut roots = 0;
    for n in 0..2 {
        if !matches!(
            rt.step_partial(
                |_| Some(safe()),
                |a| {
                    world.act(a);
                    Ok((world.partial(), 0.0))
                }
            )?,
            StepOutcome::Executed { learned: false, .. }
        ) {
            return Err("frozen inference did not execute".into());
        }
        if n == 0 {
            roots = rt
                .organism()
                .phase_partial_belief()
                .unwrap()
                .inverse
                .unwrap()
                .pre_hypotheses
                .len();
        }
    }
    let unchanged = rt.organism().phase_native_learned_fingerprint() == fingerprint;
    if roots != 2 || !rt.goal_reached()? || !unchanged {
        return Err("inverse inference check failed".into());
    }
    println!("mode={} rule_actions={} mask_actions={} modular_roots={} goal_actions=2 factual_goal=true frozen_unchanged={}", args[1], rule_actions, mask_actions, roots, unchanged);
    Ok(())
}
