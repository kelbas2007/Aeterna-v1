//! Learn circular motor rules from factual full observations, then acquire
//! output visibility from sparse POST. Resume frozen cognition in another
//! process. The external world's laws and hidden values never enter planning.
use aeterna_v1::carrier::{
    PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig, PhaseRuleConfig,
};
use aeterna_v1::scientific_runtime::{ReasoningMode, ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::{Read, Write};

const PERIOD: u32 = 257;
const MAX_BYTES: usize = 16 * 1024 * 1024;

struct ExternalWorld([u32; 2]);
impl ExternalWorld {
    fn full(&self) -> Vec<f32> {
        self.0.iter().map(|&x| x as f32 / PERIOD as f32).collect()
    }
    fn partial(&self) -> Vec<Option<f32>> {
        vec![Some(self.0[0] as f32 / PERIOD as f32), None]
    }
    fn act(&mut self, action: usize) {
        match action {
            0 => {}
            1 => self.0[0] = (self.0[0] + 17) % PERIOD,
            2 => self.0.swap(0, 1),
            _ => panic!("invalid motor"),
        }
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
        motor_cells: 3,
        dormant_cells: 10,
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

fn rules_ready(rt: &ScientificRuntime) -> bool {
    (0..3).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed)
}

fn masks_ready(rt: &ScientificRuntime) -> bool {
    (0..3).all(|a| rt.organism().phase_partial_mask_info(a).unwrap().confirmed)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: partial_observation <learn|run> <checkpoint.json>".into());
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
    let mut world = ExternalWorld([7, 31]);
    let mut rule_acquisition = 0;
    let evo = if learning {
        let mut rt = ScientificRuntime::new(fresh())?;
        rt.observe_external(&world.full())?;
        rt.set_goal(&[0.1234567, 0.1234567])?;
        while !rules_ready(&rt) {
            if rule_acquisition == 60 {
                return Err("full-observation rule acquisition budget exhausted".into());
            }
            if !matches!(
                rt.step(
                    |_| Some(safe()),
                    |a| {
                        world.act(a);
                        Ok(world.full())
                    },
                )?,
                StepOutcome::Executed { .. }
            ) {
                return Err("rule acquisition did not execute".into());
            }
            rule_acquisition += 1;
        }
        let mut evo = EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes()?)?;
        assert!(evo.enable_phase_partial_observation(PhasePartialConfig::default()));
        evo
    } else {
        let mut bytes = Vec::new();
        std::fs::File::open(&args[2])?
            .take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_BYTES {
            return Err("checkpoint exceeds byte limit".into());
        }
        let evo = EvoPhase::from_online_checkpoint(&bytes)?;
        if !evo.phase_partial_enabled()
            || !evo.phase_rules_enabled()
            || evo.config().sensory_cells != 2
            || evo.config().motor_cells != 3
        {
            return Err("this demo requires its acquired partial-observation model".into());
        }
        evo
    };
    let mut rt = ScientificRuntime::new(evo)?;
    let mut mask_acquisition = 0;
    if learning {
        rt.observe_external_partial(&world.partial())?;
        rt.set_partial_goal(&[Some(0.1234567), None])?;
        while !masks_ready(&rt) {
            if mask_acquisition == 30 {
                return Err("visibility acquisition budget exhausted".into());
            }
            if !matches!(
                rt.step_partial(
                    |_| Some(safe()),
                    |a| {
                        world.act(a);
                        Ok((world.partial(), 0.0))
                    },
                )?,
                StepOutcome::Executed { .. }
            ) {
                return Err("visibility acquisition did not execute".into());
            }
            mask_acquisition += 1;
        }
        destination
            .as_mut()
            .unwrap()
            .write_all(&rt.organism().online_checkpoint_bytes()?)?;
        destination.as_mut().unwrap().sync_all()?;
    }
    if !rules_ready(&rt) || !masks_ready(&rt) {
        return Err("the demo model has unconfirmed rules or visibility".into());
    }
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.0 = [0, 193];
    rt.observe_external_partial(&world.partial())?;
    rt.set_partial_goal(&[Some(227.0 / PERIOD as f32), None])?;
    let mut execution = 0;
    let mut information = 0;
    while !rt.goal_reached()? {
        if execution == 3 {
            return Err("partial goal budget exhausted".into());
        }
        match rt.step_partial(
            |_| Some(safe()),
            |a| {
                world.act(a);
                Ok((world.partial(), 0.0))
            },
        )? {
            StepOutcome::Executed {
                proposal, learned, ..
            } => {
                if learned {
                    return Err("frozen execution reported learning".into());
                }
                information +=
                    usize::from(proposal.mode == ReasoningMode::PartialInformationGathering);
            }
            _ => return Err("partial execution did not execute".into()),
        }
        execution += 1;
    }
    let unchanged = rt.organism().phase_native_learned_fingerprint() == fingerprint;
    if !unchanged || information != 1 || world.0[0] != 227 {
        return Err("partial sensing/frozen execution check failed".into());
    }
    println!(
        "mode={} acquired_rules=3 acquired_masks=3 rule_acquisition_actions={} mask_acquisition_actions={} information_actions={} goal_actions={} factual_goal=true stored_states={} frozen_unchanged={}",
        args[1], rule_acquisition, mask_acquisition, information, execution,
        rt.organism().phase_native_receptor_count(), unchanged
    );
    Ok(())
}
