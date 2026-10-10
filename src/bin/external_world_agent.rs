//! JSONL stdio adapter for independent Gymnasium environments.
//! The action is selected by native EvoPhase via ScientificRuntime::step_unified.
//! The Python host receives an action ONLY after U1 selection and the protection
//! screen; it sends back the genuinely executed environment observation/reward.
//! No map, object IDs, action semantics, reward lookup or mission text enters it.
use aeterna_v1::carrier::{EvoConfig, EvoPhase, PhaseMetaControlConfig, PhaseNativeConfig, PhaseHypothesisEcologyConfig};
use aeterna_v1::human_protection::HumanProtectionEvidence;
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn valid_bits(msg: &Value, key: &str, len: usize) -> Result<Vec<f32>, String> {
    let data = msg.get(key).and_then(Value::as_array)
        .ok_or(format!("missing {key} array"))?;
    if data.len() != len { return Err(format!("{key}: expected {len}, got {}", data.len())); }
    data.iter().map(|item| match item.as_u64() {
        Some(0) => Ok(0.0),
        Some(1) => Ok(1.0),
        _ => Err("raw observation must contain only 0/1 bits".into()),
    }).collect()
}
fn send<W: Write>(out: &mut W, value: &Value) -> Result<(), String> {
    serde_json::to_writer(&mut *out, value).map_err(|e| e.to_string())?;
    out.write_all(b"\n").map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())
}
fn read<R: BufRead>(input: &mut R) -> Result<Value, String> {
    let mut line = String::new();
    let n = input.read_line(&mut line).map_err(|e| e.to_string())?;
    if n == 0 { return Err("stdin EOF".into()); }
    serde_json::from_str(&line).map_err(|e| e.to_string())
}
fn safe() -> HumanProtectionEvidence {
    // These virtual motor actions have no physical human-world effect.
    // A real robot requires a trusted independent hazard source instead.
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn init(len: usize, motors: usize) -> Result<ScientificRuntime, String> {
    if !(8..=1024).contains(&len) || !(2..=32).contains(&motors) {
        return Err("invalid raw dimension or motor count".into());
    }
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: len, motor_cells: motors, dormant_cells: 128,
        hdc_dim: 64, ..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    if !evo.enable_phase_native_meta_control(PhaseMetaControlConfig {
        learning_enabled: false, ..PhaseMetaControlConfig::default()
    }) { return Err("native U1 creation failed".into()); }
    // The shared U1 action coalescer requires an attached U2 ecology;
    // without it all proposals are rejected as NoSupportedAction.
    if !evo.enable_phase_native_hypothesis_ecology(
        PhaseHypothesisEcologyConfig::default()
    ) { return Err("U2 ecology required by native U1 selector".into()); }
    let mut rt = ScientificRuntime::new(evo).map_err(|e| e.to_string())?;
    if !rt.enable_factor_causality() || !rt.enable_factor_external_reward_goal() {
        return Err("carrier-owned external reward mode refused".into());
    }
    // Public MiniGrid 7x7x3 categorical view: the first field is
    // a candidate appearance signature; other fields may vary with color
    // and door state. This is a sensor LAYOUT,
    // not a mapping from simulator category indexes to word meanings.
    if len == 588 && !rt.enable_object_grounding(7, 7, 3, 4, 1) {
        return Err("object grounding memory creation failed".into());
    }
    Ok(rt)
}
fn main_loop<R: BufRead, W: Write>(input: &mut R, out: &mut W) -> Result<(), String> {
    let mut runtime: Option<ScientificRuntime> = None;
    let mut dim = 0usize;
    loop {
        let msg = match read(input) {
            Ok(v) => v,
            Err(e) if e == "stdin EOF" => return Ok(()),
            Err(e) => { send(out, &json!({"type":"error","message":e}))?; continue; }
        };
        let operation = msg.get("cmd").and_then(Value::as_str).unwrap_or("");
        match operation {
            "init" => {
                if runtime.is_some() { return Err("cannot reinitialize cognition".into()); }
                dim = msg.get("dimension").and_then(Value::as_u64).unwrap_or(0) as usize;
                let motors = msg.get("motors").and_then(Value::as_u64).unwrap_or(0) as usize;
                runtime = Some(init(dim, motors)?);
                send(out, &json!({"type":"ready","dimension":dim,"motors":motors}))?;
            }
            "reset" => {
                let rt = runtime.as_mut().ok_or("init required")?;
                let bits = valid_bits(&msg, "observation", dim)?;
                let learning = msg.get("learning").and_then(Value::as_bool)
                    .ok_or("learning true/false required")?;
                rt.set_model_learning_enabled(learning);
                rt.observe_external(&bits).map_err(|e| e.to_string())?;
                // Unknown success observation: reward is the ONLY source of
                // a terminal goal, so this raw impossible target is just a
                // neutral placeholder until the first factual success.
                rt.set_goal(&vec![0.0; dim]).map_err(|e| e.to_string())?;
                send(out, &json!({"type":"reset_ack","learning":learning}))?;
            }
            "advance" => {
                let rt = runtime.as_mut().ok_or("init required")?;
                let mut selected: Option<usize> = None;
                let r = rt.step_unified(
                    |_| Some(safe()),
                    |motor| {
                        selected = Some(motor);
                        send(out, &json!({"type":"action","action":motor}))
                            .map_err(|e| format!("send action: {e}"))?;
                        let post = read(input)?;
                        if post.get("cmd").and_then(Value::as_str) != Some("post") {
                            return Err("expected factual post".into());
                        }
                        let sensory = valid_bits(&post, "observation", dim)?;
                        let reward = post.get("reward").and_then(Value::as_f64)
                            .ok_or("reward missing")? as f32;
                        if !reward.is_finite() || !(0.0..=1.0).contains(&reward) {
                            return Err("reward outside [0,1]".into());
                        }
                        Ok((sensory, reward))
                    },
                );
                match r {
                    Ok(StepOutcome::Executed { proposal, .. }) =>
                        send(out, &json!({
                            "type":"advance_ack","executed":proposal.action,
                            "rules":rt.organism().phase_native_factor_rule_count(),
                            "rewarded_examples":rt.organism()
                                .phase_native_factor_rewarded_examples(),
                        }))?,
                    Ok(StepOutcome::Blocked(_)) =>
                        send(out, &json!({"type":"stopped","reason":"protected_block"}))?,
                    Ok(StepOutcome::GoalReached) =>
                        send(out, &json!({"type":"stopped","reason":"goal_reached"}))?,
                    Ok(other) =>
                        send(out, &json!({"type":"stopped","reason":format!("{other:?}")}))?,
                    Err(e) =>
                        send(out, &json!({"type":"stopped","reason":e.to_string(),
                            "previous_action":selected}))?,
                }
            }
            "restart" => {
                let rt = runtime.as_mut().ok_or("init required")?;
                rt.restart_cognition().map_err(|e| e.to_string())?;
                send(out, &json!({"type":"restart_ack",
                    "word_count":rt.organism().phase_native_grounded_words()}))?;
            }
            "demonstrate" => {
                let rt = runtime.as_mut().ok_or("init required")?;
                let word = msg.get("word").and_then(Value::as_str)
                    .ok_or("demonstration word required")?;
                let tile = msg.get("tile").and_then(Value::as_u64)
                    .ok_or("demonstration tile required")? as usize;
                let action = msg.get("action").and_then(Value::as_u64)
                    .ok_or("demonstration executed action required")? as usize;
                let before = valid_bits(&msg, "before", dim)?;
                let after = valid_bits(&msg, "after", dim)?;
                let acquired = rt.observe_demonstrated_object_action(
                    word, tile, action, &before, &after);
                // A demonstrator's outside action is not counted as a U1-
                // selected motor. It is explicit, supervised motor tuition.
                // The resulting POST is still a REAL external observation.
                rt.observe_external(&after).map_err(|e| e.to_string())?;
                send(out, &json!({"type":"demonstration_ack",
                    "acquired":acquired,
                    "learned_affordances":rt.organism().phase_native_learned_affordances()}))?;
            }
            "embodied_navigation" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_embodied_navigation();
                send(out,&json!({"type":"embodied_navigation_ack",
                    "accepted":accepted}))?;
            }
            "self_experiment" => {
                let rt = runtime.as_mut().ok_or("init required")?;
                let tile = msg.get("front_tile").and_then(Value::as_u64)
                    .ok_or("relative front tile required")? as usize;
                let accepted = rt.enable_self_object_experiment(tile);
                send(out,&json!({"type":"self_experiment_ack",
                    "accepted":accepted}))?;
            }
            "word_intent" => {
                let rt = runtime.as_mut().ok_or("init required")?;
                let word = msg.get("word").and_then(Value::as_str)
                    .ok_or("word required")?;
                let known = rt.set_grounded_word_intent(word);
                send(out, &json!({"type":"word_intent_ack", "accepted":known}))?;
            }
            "teach_word" => {
                let rt = runtime.as_mut().ok_or("init required")?;
                let word = msg.get("word").and_then(Value::as_str)
                    .ok_or("word required")?;
                let tile = msg.get("tile").and_then(Value::as_u64)
                    .ok_or("pointed tile required")? as usize;
                let learned = rt.teach_pointed_word(word, tile);
                send(out, &json!({"type":"lesson_ack", "learned":learned,
                    "word_count":rt.organism().phase_native_grounded_words()}))?;
            }
            "locate_word" => {
                let rt = runtime.as_ref().ok_or("init required")?;
                let word = msg.get("word").and_then(Value::as_str)
                    .ok_or("word required")?;
                let found = rt.locate_grounded_word(word);
                send(out, &json!({"type":"referents", "word":word,
                    "tiles":found.iter().map(|r|r.tile_index).collect::<Vec<_>>(),
                    "strengths":found.iter().map(|r|r.strength).collect::<Vec<_>>() }))?;
            }
            "status" => {
                let rt = runtime.as_ref().ok_or("init required")?;
                send(out, &json!({
                    "type":"status",
                    "rules":rt.organism().phase_native_factor_rule_count(),
                    "rewarded_examples":rt.organism().phase_native_factor_rewarded_examples(),
                    "contradictions":rt.organism().phase_native_factor_contradictions(),
                    "safety_latched":rt.emergency_latched(),
                    "visual_categories":rt.organism().phase_native_grounded_categories(),
                    "word_count":rt.organism().phase_native_grounded_words(),
                    "learned_affordances":rt.organism().phase_native_learned_affordances(),
                    "visual_frames":rt.organism().phase_native_grounded_frames(),
                    "self_experiments":rt.organism().phase_native_self_trial_count(),
                    "self_affordances":rt.organism().phase_native_self_affordance_count(),
                    "inferred_motion":rt.organism().phase_native_embodied_move_evidence(),
                    "learned_affordances":rt.organism().phase_native_learned_affordances()
                }))?;
            }
            "quit" => return Ok(()),
            _ => send(out, &json!({"type":"error","message":"unknown command"}))?,
        }
    }
}
fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    if let Err(err) = main_loop(&mut stdin.lock(), &mut stdout.lock()) {
        eprintln!("external-world protocol error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_external_bridge_emits_a_native_action_before_any_reward() {
        let mut rt = init(588, 7).unwrap();
        let mut observed = vec![0.0_f32; 588];
        observed[7] = 1.0;
        rt.observe_external(&observed).unwrap();
        rt.set_goal(&vec![0.0; 588]).unwrap();
        let proposed = rt.propose_unified().unwrap()
            .expect("carrier must propose a motor before first reward");
        assert!(proposed.action < 7);
        let mut executed = 0usize;
        let outcome = rt.step_unified(
            |_| Some(safe()),
            |action| {
                executed = action + 1;
                Ok((observed.clone(), 0.0))
            },
        ).unwrap();
        assert!(matches!(outcome, StepOutcome::Executed { .. }));
        assert!(executed > 0, "a protected external callback must run");
    }
}
