//! One continuous noisy lifetime acquires conditions, adapts to a changed law,
//! recalls its earlier physical models and resumes frozen in a new process.
use aeterna_v1::carrier::{
    PhaseAdaptiveConfig, PhaseNativeConfig, PhaseOnlineConfig, PhaseRuleConfig, PhaseRuleLanguage,
};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::{Read, Write};

struct World {
    state: [i32; 2],
    regime: usize,
    events: u64,
}
impl World {
    fn future(&self, action: usize) -> [i32; 2] {
        let (mut gain, mut bias) = match (action, self.state[1] >= 134) {
            (0, false) => (1, 17),
            (0, true) => (2, 71),
            (1, false) => (-1, 31),
            (1, true) => (1, 53),
            _ => unreachable!(),
        };
        if self.regime == 1 {
            gain = -gain;
            bias += 101;
        }
        [
            (gain * self.state[0] + bias).rem_euclid(257),
            (self.state[1] + 31).rem_euclid(257),
        ]
    }
    fn measured(&self) -> Vec<f32> {
        self.state
            .iter()
            .enumerate()
            .map(|(j, &x)| {
                let sign = if (self.events * 3 + j as u64) % 4 < 2 {
                    1.0
                } else {
                    -1.0
                };
                (x as f32 / 257.0 + sign * 0.00035).rem_euclid(1.0)
            })
            .collect()
    }
    fn act(&mut self, a: usize) -> Vec<f32> {
        self.state = self.future(a);
        self.events += 1;
        self.measured()
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
        dormant_cells: 24,
        ..Default::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning_with_language(
        PhaseRuleConfig {
            planning_depth: 2,
            ..Default::default()
        },
        PhaseRuleLanguage::CircularAffine
    ));
    assert!(evo.enable_phase_adaptive_rules(PhaseAdaptiveConfig::default()));
    evo
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: adaptive_learning <learn|run> <checkpoint.json>".into());
    }
    let learning = args[1] == "learn";
    let mut world = World {
        state: [29, 181],
        regime: 0,
        events: 0,
    };
    let evo = if learning {
        let mut rt = ScientificRuntime::new(fresh())?;
        rt.observe_external(&world.measured())?;
        rt.set_goal(&[0.1234567, 0.2345678])?;
        for stage in 0..3 {
            world.regime = usize::from(stage == 1);
            for _ in 0..256 {
                if rt.goal_reached()? {
                    let target = rt
                        .organism()
                        .current_real()
                        .unwrap()
                        .sensory
                        .iter()
                        .map(|x| (x + 0.5).rem_euclid(1.0))
                        .collect::<Vec<_>>();
                    rt.set_goal(&target)?;
                }
                if !matches!(
                    rt.step(|_| Some(safe()), |a| Ok(world.act(a)))?,
                    StepOutcome::Executed { learned: true, .. }
                ) {
                    return Err("continuous acquisition did not execute".into());
                }
            }
        }
        let bytes = rt.organism().online_checkpoint_bytes()?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
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
    if !evo.phase_adaptive_rules_enabled()
        || evo.config().sensory_cells != 2
        || evo.config().motor_cells != 2
    {
        return Err("this demo requires its acquired adaptive model".into());
    }
    let infos = (0..2)
        .map(|a| evo.phase_adaptive_action_info(a).unwrap())
        .collect::<Vec<_>>();
    if infos.iter().any(|i| {
        i.active.as_ref().is_none_or(|m| m.condition.is_none())
            || i.reactivations == 0
            || i.archives.is_empty()
    }) {
        return Err("conditional learning or old model recall failed".into());
    }
    let archives = infos.iter().map(|i| i.archives.len()).sum::<usize>();
    let reactivations = infos.iter().map(|i| i.reactivations).sum::<u64>();
    let mut rt = ScientificRuntime::new(evo)?;
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.state = [149, 19];
    world.regime = 0;
    let target = world.future(0).map(|x| x as f32 / 257.0);
    rt.observe_external(&world.measured())?;
    rt.set_goal(&target)?;
    if !matches!(
        rt.step(|_| Some(safe()), |a| Ok(world.act(a)))?,
        StepOutcome::Executed { learned: false, .. }
    ) {
        return Err("frozen goal did not execute".into());
    }
    let unchanged = rt.organism().phase_native_learned_fingerprint() == fingerprint;
    if !rt.goal_reached()? || !unchanged {
        return Err("frozen transfer check failed".into());
    }
    println!("mode={} continuous_actions={} conditional_actions=2 archived_models={} reactivations={} goal_actions=1 factual_goal=true frozen_unchanged={}", args[1], if learning { 768 } else { 0 }, archives, reactivations, unchanged);
    Ok(())
}
