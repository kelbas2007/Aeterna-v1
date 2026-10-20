//! Small repeatable interface regressions, not external qualification evidence.
use aeterna_v1::construction::{
    autonomy::{ActionOffer, AutonomyPolicy},
    causal_memory::CausalConfig,
    sensors::SensorFrame,
    Config, Error,
};
use aeterna_v1::{
    scientific_runtime::{ScientificRuntime, SensorPost},
    HumanProtectionEvidence,
};

fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn image(namespace: u16, value: u8) -> SensorFrame {
    SensorFrame::Image {
        namespace,
        pixels: Some(vec![vec![value; 12]; 10]),
    }
}
fn runtime() -> ScientificRuntime {
    let config = Config {
        width: 32,
        actions: 16,
        planning_depth: 8,
        planning_work: 1_000_000,
        search_every: 1_000_000,
        journal_capacity: 128,
        ..Default::default()
    };
    let mut rt = ScientificRuntime::new_constructing(
        config,
        AutonomyPolicy::default(),
        vec![0; 32],
        "external-test",
    )
    .unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_causal_memory(CausalConfig {
        history_depth: 2,
        model_entries: 64,
        model_bytes: 65536,
        outcomes_per_entry: 4,
        planning_states: 32,
    })
    .unwrap();
    rt.enable_constructing_external_reward_memory(true, true)
        .unwrap();
    rt.observe_external_sensor_restart(&image(1, 0)).unwrap();
    rt
}
fn offers(excluded: Option<usize>) -> Vec<ActionOffer> {
    (0..16)
        .map(|a| ActionOffer {
            available: [3, 11].contains(&a) && Some(a) != excluded,
            max_cost: 1,
        })
        .collect()
}
#[test]
fn one_lifetime_learns_two_sensory_contracts_and_restores_frozen_knowledge() {
    let mut rt = runtime();
    for namespace in [1, 2] {
        for i in 0..32 {
            let value = if i % 2 == 0 { 0 } else { 255 };
            let target = if value == 0 { 3 } else { 11 };
            let frame = image(namespace, value);
            rt.observe_external_sensor_restart(&frame).unwrap();
            let mut excluded = None;
            for left in (1..=2).rev() {
                let mut correct = false;
                let mut chosen = 0;
                rt.step_external_sensors(&offers(excluded), Some(left), safe, |a| {
                    chosen = a;
                    correct = a == target;
                    Ok(SensorPost {
                        frame: frame.clone(),
                        cost: 1,
                        reward: if correct { 1.0 } else { 0.0 },
                    })
                })
                .unwrap();
                if correct {
                    break;
                }
                excluded = Some(chosen);
            }
            rt.finish_external_episode().unwrap();
        }
    }
    rt.freeze_constructing().unwrap();
    let before = rt
        .construction_learner()
        .unwrap()
        .external_reward_report()
        .unwrap();
    let bytes = rt.checkpoint_constructing().unwrap();
    let mut restored = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    for namespace in [1, 2] {
        for value in [0, 255] {
            let frame = image(namespace, value);
            let target = if value == 0 { 3 } else { 11 };
            restored.observe_external_sensor_restart(&frame).unwrap();
            let step = restored
                .step_external_sensors(&offers(None), Some(1), safe, |a| {
                    assert_eq!(a, target);
                    Ok(SensorPost {
                        frame: frame.clone(),
                        cost: 1,
                        reward: 1.0,
                    })
                })
                .unwrap();
            assert_eq!(step.action, target);
        }
    }
    let after = restored
        .construction_learner()
        .unwrap()
        .external_reward_report()
        .unwrap();
    assert_eq!(before.learned_fingerprint, after.learned_fingerprint);
    assert_eq!(before.learned_events, after.learned_events);
    assert_eq!(after.namespaces, 2);
    ScientificRuntime::from_constructing_checkpoint(&restored.checkpoint_constructing().unwrap())
        .unwrap();
}
#[test]
fn unknown_sensor_post_stops_actuation_and_does_not_invent_a_training_failure() {
    let mut rt = runtime();
    let before = rt
        .construction_learner()
        .unwrap()
        .external_reward_report()
        .unwrap();
    assert!(rt
        .step_external_sensors(&offers(None), Some(2), safe, |_| Err(Error::Execution(
            "sensor disconnected".into()
        )))
        .is_err());
    let after = rt
        .construction_learner()
        .unwrap()
        .external_reward_report()
        .unwrap();
    assert_eq!(before.learned_fingerprint, after.learned_fingerprint);
    assert_eq!(after.factual_events, 0);
    assert!(rt
        .step_external_sensors(&offers(None), Some(2), safe, |_| panic!(
            "must remain stopped"
        ))
        .is_err());
}

#[test]
fn invalid_sensor_posts_keep_the_known_actual_cost_without_learning() {
    for (cost, frame) in [
        (7, image(1, 0)),
        (0, image(2, 0)),
        (
            0,
            SensorFrame::Image {
                namespace: 1,
                pixels: Some(Vec::new()),
            },
        ),
    ] {
        let mut rt = runtime();
        let hash = rt
            .construction_learner()
            .unwrap()
            .external_reward_report()
            .unwrap()
            .learned_fingerprint;
        assert!(rt
            .step_external_sensors(&offers(None), Some(2), safe, |_| Ok(SensorPost {
                frame,
                cost,
                reward: 0.0
            }))
            .is_err());
        assert_eq!(rt.unified_report().unwrap().total_external_cost, cost);
        assert_eq!(
            rt.construction_learner()
                .unwrap()
                .external_reward_report()
                .unwrap()
                .learned_fingerprint,
            hash
        );
        assert_eq!(rt.construction_learner().unwrap().factual_event_count(), 0);
        assert!(rt
            .step_external_sensors(&offers(None), Some(1), safe, |_| panic!(
                "must remain stopped"
            ))
            .is_err());
    }
}

#[test]
fn jsonl_malformed_sensor_content_preserves_known_cost_and_stops_without_learning() {
    use serde_json::{json, Value};
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    for (cost, frame, reward) in [
        (
            7,
            json!({"kind":"image","namespace":1,"pixels":[[256]]}),
            json!(0.0),
        ),
        (
            3,
            json!({"kind":"image","namespace":65536,"pixels":null}),
            json!(0.0),
        ),
        (2, Value::Null, json!(0.0)),
        (
            1,
            json!({"kind":"image","namespace":1,"pixels":null}),
            json!("invalid"),
        ),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_aeterna-v1"))
            .arg("--jsonl")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut exchange = |message: Value| -> Value {
            writeln!(input, "{message}").unwrap();
            input.flush().unwrap();
            let mut line = String::new();
            output.read_line(&mut line).unwrap();
            serde_json::from_str(&line).unwrap()
        };
        assert_eq!(
            exchange(json!({"op":"external_init","adaptive":false,
            "frame":{"kind":"image","namespace":1,"pixels":null}}))["type"],
            "initialized"
        );
        let offers = vec![json!({"available":true,"max_cost":10}); 16];
        assert_eq!(
            exchange(json!({"op":"external_step","offers":offers,
            "remaining_actions":1}))["type"],
            "action"
        );
        let error =
            exchange(json!({"op":"post","result":{"frame":frame,"cost":cost,"reward":reward}}));
        assert_eq!(error["type"], "error", "{error}");
        assert_eq!(error["report"]["total_external_cost"], cost, "{error}");
        assert_eq!(error["external_reward_report"]["factual_events"], 0);
        assert_eq!(error["external_reward_report"]["learned_events"], 0);
        drop(exchange);
        drop(input);
        assert!(!child.wait().unwrap().success());
    }
}
