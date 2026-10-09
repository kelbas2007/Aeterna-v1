//! An external circular sensor world. Its laws never enter the carrier.
//! Run `learn FILE`, then `run FILE` in a separate process.
use aeterna_v1::carrier::{PhaseNativeConfig, PhaseOnlineConfig, PhaseRuleConfig};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::{Read, Write};

const PERIOD: u32 = 257;
const MAX_BYTES: usize = 16 * 1024 * 1024;

struct ExternalWorld([u32; 2]);
impl ExternalWorld {
    fn apply(state: [u32; 2], action: usize) -> [u32; 2] {
        match action {
            0 => [(state[0] + 13) % PERIOD, (state[1] + 27) % PERIOD],
            1 => [(state[1] + 17) % PERIOD, (PERIOD - state[0] + 9) % PERIOD],
            _ => panic!("invalid motor"),
        }
    }
    fn raster(state: [u32; 2]) -> Vec<f32> {
        state.iter().map(|&x| x as f32 / PERIOD as f32).collect()
    }
    fn observe(&self) -> Vec<f32> {
        Self::raster(self.0)
    }
    fn act(&mut self, action: usize) -> Vec<f32> {
        self.0 = Self::apply(self.0, action);
        self.observe()
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
    evo.enable_phase_native_planning(PhaseNativeConfig {
        horizon: 16,
        ..Default::default()
    });
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning(PhaseRuleConfig::default()));
    evo
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: learned_rules <learn|run> <checkpoint.json>".into());
    }
    let learning = args[1] == "learn";
    let mut destination = if learning {
        Some(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[2])?,
        )
    } else {
        None
    };
    let evo = if learning {
        fresh()
    } else {
        let mut bytes = Vec::new();
        std::fs::File::open(&args[2])?
            .take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_BYTES {
            return Err("checkpoint exceeds byte limit".into());
        }
        let evo = EvoPhase::from_online_checkpoint(&bytes)?;
        if !evo.phase_rules_enabled()
            || evo.config().sensory_cells != 2
            || evo.config().motor_cells != 2
        {
            return Err("this demo requires its acquired two-channel rule model".into());
        }
        evo
    };
    let mut rt = ScientificRuntime::new(evo)?;
    let mut world = ExternalWorld([7, 31]);
    let mut acquisition = 0;
    if learning {
        rt.observe_external(&world.observe())?;
        rt.set_goal(&[0.1234567, 0.1234567])?;
        while !(0..2).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed) {
            if acquisition == 40 {
                return Err("cold acquisition budget exhausted".into());
            }
            if !matches!(
                rt.step(|_| Some(safe()), |a| Ok(world.act(a)))?,
                StepOutcome::Executed { .. }
            ) {
                return Err("cold acquisition did not execute".into());
            }
            acquisition += 1;
        }
        destination
            .as_mut()
            .unwrap()
            .write_all(&rt.organism().online_checkpoint_bytes()?)?;
        destination.as_mut().unwrap().sync_all()?;
    }
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.0 = [149, 203]; // an unseen observation, not a pretrained state identity
    let mut target = world.0;
    for action in [1, 0, 1, 0] {
        target = ExternalWorld::apply(target, action);
    }
    rt.observe_external(&world.observe())?;
    rt.set_goal(&ExternalWorld::raster(target))?;
    let mut execution = 0;
    while !rt.goal_reached()? {
        if execution == 4 {
            return Err("held-out goal budget exhausted".into());
        }
        if !matches!(
            rt.step(|_| Some(safe()), |a| Ok(world.act(a)))?,
            StepOutcome::Executed { .. }
        ) {
            return Err("held-out execution was blocked".into());
        }
        execution += 1;
    }
    let unchanged = rt.organism().phase_native_learned_fingerprint() == fingerprint;
    if !unchanged {
        return Err("frozen learned model changed".into());
    }
    println!("mode={} acquired_rules=2 acquisition_actions={} heldout_goal=true goal_actions={} stored_states={} frozen_unchanged={}",
        args[1],acquisition,execution,rt.organism().phase_native_receptor_count(),unchanged);
    Ok(())
}
