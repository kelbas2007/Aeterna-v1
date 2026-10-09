//! Four opaque motors in an external integer world. All five circular rule
//! families are acquired from actual actions during one continuous episode.
//! Resume the frozen model from a disk file in a separate process.
use aeterna_v1::carrier::{
    PhaseNativeConfig, PhaseOnlineConfig, PhaseRuleConfig, PhaseRuleLanguage,
};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::{Read, Write};

const PERIOD: i32 = 257;
const MAX_BYTES: usize = 16 * 1024 * 1024;

struct ExternalWorld([i32; 2]);
impl ExternalWorld {
    fn apply(state: [i32; 2], action: usize) -> [i32; 2] {
        let [x, y] = state;
        let values = match action {
            0 => [x + 17, y + 31],
            1 => [x + y + 11, 3 * y + 7],
            2 => [71, x - y + 13],
            3 => [2 * y + 5, -2 * x + 9],
            _ => panic!("invalid motor"),
        };
        values.map(|x| x.rem_euclid(PERIOD))
    }
    fn raster(state: [i32; 2]) -> Vec<f32> {
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
fn ready(rt: &ScientificRuntime) -> bool {
    (0..4).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed)
}
fn fresh() -> EvoPhase {
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: 2,
        motor_cells: 4,
        dormant_cells: 12,
        ..Default::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig {
        horizon: 16,
        ..Default::default()
    });
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning_with_language(
        PhaseRuleConfig {
            planning_depth: 4,
            ..Default::default()
        },
        PhaseRuleLanguage::CircularAffine,
    ));
    evo
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: expanded_rules <learn|run> <checkpoint.json>".into());
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
        if evo.phase_rule_language() != Some(PhaseRuleLanguage::CircularAffine)
            || evo.phase_partial_enabled()
            || evo.config().sensory_cells != 2
            || evo.config().motor_cells != 4
        {
            return Err("this demo requires its two-channel circular affine model".into());
        }
        evo
    };
    let mut rt = ScientificRuntime::new(evo)?;
    let mut world = ExternalWorld([7, 31]);
    let mut acquisition = 0;
    if learning {
        rt.observe_external(&world.observe())?;
        rt.set_goal(&[0.1234567; 2])?;
        while !ready(&rt) {
            if acquisition == 80 {
                return Err("continuous acquisition budget exhausted".into());
            }
            if !matches!(
                rt.step(|_| Some(safe()), |a| Ok(world.act(a)))?,
                StepOutcome::Executed { .. }
            ) {
                return Err("continuous acquisition did not execute".into());
            }
            acquisition += 1;
        }
        destination
            .as_mut()
            .unwrap()
            .write_all(&rt.organism().online_checkpoint_bytes()?)?;
        destination.as_mut().unwrap().sync_all()?;
    }
    if !ready(&rt) {
        return Err("unconfirmed demo model".into());
    }
    let mut families = Vec::new();
    for action in 0..4 {
        for formula in rt.organism().phase_rule_formulas(action).unwrap() {
            if !families.contains(&formula.family) {
                families.push(formula.family);
            }
        }
    }
    if families.len() != 5 {
        return Err("not all five families were acquired".into());
    }
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.0 = [149, 203];
    let mut target = world.0;
    for action in [2, 1, 3] {
        target = ExternalWorld::apply(target, action);
    }
    rt.observe_external(&world.observe())?;
    rt.set_goal(&ExternalWorld::raster(target))?;
    let mut execution = 0;
    while !rt.goal_reached()? {
        if execution == 3 {
            return Err("frozen goal budget exhausted".into());
        }
        if !matches!(
            rt.step(|_| Some(safe()), |a| Ok(world.act(a)))?,
            StepOutcome::Executed { learned: false, .. }
        ) {
            return Err("frozen execution did not execute".into());
        }
        execution += 1;
    }
    let unchanged = rt.organism().phase_native_learned_fingerprint() == fingerprint;
    if !unchanged {
        return Err("frozen knowledge changed".into());
    }
    println!("mode={} acquired_rules=4 families={} acquisition_actions={} frozen_goal=true goal_actions={} stored_states={} frozen_unchanged={}",
        args[1], families.len(), acquisition, execution, rt.organism().phase_native_receptor_count(), unchanged);
    Ok(())
}
