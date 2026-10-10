//! JSONL stdio adapter for independent Gymnasium environments.
//! The action is selected by native EvoPhase via ScientificRuntime::step_unified.
//! The Python host receives an action ONLY after U1 selection and the protection
//! screen; it sends back the genuinely executed environment observation/reward.
//! No map, object IDs, action semantics, reward lookup or mission text enters it.
use aeterna_v1::carrier::{EvoConfig, EvoPhase, PhaseMetaControlConfig, PhaseNativeConfig, PhaseHypothesisEcologyConfig};
use aeterna_v1::human_protection::HumanProtectionEvidence;
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

const MAX_JSONL_FRAME_BYTES: usize = 64 * 1024;
const FRAME_TOO_LARGE: &str = "JSONL frame exceeds 64 KiB limit";

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
    let mut line = Vec::new();
    let n = input.take((MAX_JSONL_FRAME_BYTES + 1) as u64)
        .read_until(b'\n', &mut line).map_err(|e| e.to_string())?;
    if n == 0 { return Err("stdin EOF".into()); }
    if n > MAX_JSONL_FRAME_BYTES { return Err(FRAME_TOO_LARGE.into()); }
    serde_json::from_slice(&line).map_err(|e| e.to_string())
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
            // The oversized frame's unread tail must never become commands.
            Err(e) if e == FRAME_TOO_LARGE => return Err(e),
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
                let mut oversized_post = false;
                let r = rt.step_unified(
                    |_| Some(safe()),
                    |motor| {
                        selected = Some(motor);
                        send(out, &json!({"type":"action","action":motor}))
                            .map_err(|e| format!("send action: {e}"))?;
                        let post = match read(input) {
                            Ok(post) => post,
                            Err(e) => {
                                oversized_post = e == FRAME_TOO_LARGE;
                                return Err(e);
                            }
                        };
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
                if oversized_post { return Err(FRAME_TOO_LARGE.into()); }
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
            "innate_scaffold" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_innate_scaffold();
                send(out,&json!({"type":"innate_scaffold_ack",
                    "accepted":accepted}))?;
            }
            "relational_workspace" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_relational_workspace();
                send(out,&json!({"type":"relational_workspace_ack",
                    "accepted":accepted}))?;
            }
            "relation_status" => {
                let rt=runtime.as_ref().ok_or("init required")?;
                let readout=rt.organism().current_real()
                    .and_then(|real|rt.organism()
                        .phase_native_relational_readout(&real.sensory));
                send(out,&json!({"type":"relation_status",
                    "bound":rt.organism().phase_native_relational_held_subject(),
                    "features":readout,
                    "frames":rt.organism().phase_native_relational_subject_frames()}))?;
            }
            "sequence_replay" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_sequence_replay();
                send(out,&json!({"type":"sequence_replay_ack",
                    "accepted":accepted}))?;
            }
            "episodic_recall" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_episodic_recall();
                send(out,&json!({"type":"episodic_recall_ack",
                    "accepted":accepted}))?;
            }
            "developmental_memory" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_developmental_memory();
                send(out,&json!({"type":"developmental_memory_ack",
                    "accepted":accepted}))?;
            }
            "general_policy" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_general_policy();
                send(out,&json!({"type":"general_policy_ack",
                    "accepted":accepted}))?;
            }
            "context_value_learning" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                let accepted=rt.enable_context_value_learning();
                send(out,&json!({"type":"context_value_learning_ack",
                    "accepted":accepted}))?;
            }
            "value_abstraction" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                send(out,&json!({"type":"value_abstraction_ack",
                    "accepted":rt.enable_value_abstraction()}))?;
            }
            "value_abstraction_lesion" => {
                let rt=runtime.as_mut().ok_or("init required")?;
                send(out,&json!({"type":"value_abstraction_lesion_ack",
                    "accepted":rt.lesion_value_abstraction_for_control()}))?;
            }
            "value_predicates" => {
                let rt=runtime.as_ref().ok_or("init required")?;
                send(out,&json!({"type":"value_predicates",
                    "nodes":rt.organism().phase_native_value_predicate_programs()}))?;
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
                    "general_rewards":rt.organism().phase_native_general_rewards(),
                    "general_updates":rt.organism().phase_native_general_updates(),
                    "value_states":rt.organism().phase_native_value_state_count(),
                    "value_abstraction":rt.organism().phase_native_value_abstraction_status(),
                    "innate_bias_enabled":rt.organism().phase_native_innate_enabled(),
                    "innate_experience_events":rt.organism().phase_native_innate_readout()
                        .map_or(0,|r|r.observed_events),
                    "developmental_memory":rt.organism().phase_native_developmental_memory_enabled(),
                    "developmental_memory_events":rt.organism().phase_native_developmental_retained_events(),
                    "rewarded_episodic_memories":rt.organism().phase_native_episodic_count(),
                    "failed_episodic_experiences":rt.organism().phase_native_episodic_failures(),
                    "relational_bound":rt.organism().phase_native_relational_held_subject(),
                    "relational_frames":rt.organism().phase_native_relational_subject_frames(),
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
    fn contextual_value_learning_receives_only_the_executed_factual_post() {
        let mut observation=vec![0;588];
        observation[7]=1;
        let messages=[
            json!({"cmd":"init","dimension":588,"motors":7}),
            json!({"cmd":"general_policy"}),
            json!({"cmd":"developmental_memory"}),
            json!({"cmd":"relational_workspace"}),
            json!({"cmd":"context_value_learning"}),
            json!({"cmd":"reset","observation":observation,"learning":true}),
            json!({"cmd":"advance"}),
            json!({"cmd":"post","observation":observation,"reward":1.0}),
            json!({"cmd":"status"}),
            json!({"cmd":"quit"}),
        ];
        let bytes=messages.iter().map(|m|format!("{m}\n")).collect::<String>();
        let mut output=Vec::new();
        main_loop(&mut io::Cursor::new(bytes),&mut output).unwrap();
        let frames=String::from_utf8(output).unwrap().lines()
            .map(|line|serde_json::from_str::<Value>(line).unwrap()).collect::<Vec<_>>();
        assert_eq!(frames.iter().filter(|f|f["type"]=="action").count(),1);
        let status=frames.iter().find(|f|f["type"]=="status").unwrap();
        assert_eq!(status["general_updates"],1);
        assert_eq!(status["general_rewards"],1);
        assert!(status["value_states"].as_u64().unwrap()>0);
        assert_eq!(status["rewarded_examples"],0,"the unused factor solver must not absorb this reward");
    }

    #[test]
    fn oversized_jsonl_frame_terminates_without_interpreting_its_tail() {
        let mut bytes = vec![b' '; MAX_JSONL_FRAME_BYTES + 1024];
        bytes.extend_from_slice(b"\n{\"cmd\":\"init\",\"dimension\":8,\"motors\":2}\n");
        let mut input = io::Cursor::new(bytes);
        let mut output = Vec::new();
        assert_eq!(main_loop(&mut input, &mut output), Err(FRAME_TOO_LARGE.into()));
        assert!(output.is_empty(), "the trailing command must not initialize cognition");
        assert_eq!(input.position(), (MAX_JSONL_FRAME_BYTES + 1) as u64);
    }

    #[test]
    fn normal_jsonl_frames_stay_separate_and_eof_is_preserved() {
        let mut input = io::Cursor::new(b"{\"cmd\":\"status\"}\n{\"cmd\":\"quit\"}\n");
        assert_eq!(read(&mut input).unwrap()["cmd"], "status");
        assert_eq!(read(&mut input).unwrap()["cmd"], "quit");
        assert_eq!(read(&mut input), Err("stdin EOF".into()));
    }

    #[test]
    fn oversized_factual_post_stops_without_acknowledging_or_reading_commands() {
        let mut observation = vec![0; 588];
        observation[7] = 1;
        let messages = [
            json!({"cmd":"init","dimension":588,"motors":7}),
            json!({"cmd":"reset","observation":observation,"learning":true}),
            json!({"cmd":"advance"}),
        ];
        let mut bytes = Vec::new();
        for message in messages {
            bytes.extend_from_slice(message.to_string().as_bytes());
            bytes.push(b'\n');
        }
        let prefix_len = bytes.len();
        bytes.extend(std::iter::repeat_n(b' ', MAX_JSONL_FRAME_BYTES + 1024));
        bytes.extend_from_slice(b"\n{\"cmd\":\"status\"}\n{\"cmd\":\"quit\"}\n");
        let mut input = io::Cursor::new(bytes);
        let mut output = Vec::new();
        assert_eq!(main_loop(&mut input, &mut output), Err(FRAME_TOO_LARGE.into()));
        let frames: Vec<Value> = String::from_utf8(output).unwrap().lines()
            .map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(frames.iter().filter(|frame| frame["type"] == "action").count(), 1);
        assert!(!frames.iter().any(|frame| frame["type"] == "advance_ack" || frame["type"] == "status"));
        assert_eq!(input.position() as usize, prefix_len + MAX_JSONL_FRAME_BYTES + 1);
    }

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
