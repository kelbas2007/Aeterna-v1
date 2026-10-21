//! Integration of the existing mechanisms through actual protected callbacks.
//! These small authored worlds verify wiring, not intelligence qualification.
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
fn frame(domain: usize, value: usize) -> SensorFrame {
    match domain {
        0 => SensorFrame::Image {
            namespace: 1,
            pixels: Some(vec![vec![(value % 256) as u8; 12]; 10]),
        },
        1 => SensorFrame::Signals {
            namespace: 2,
            channels: (0..9)
                .map(|j| {
                    Some(
                        (0..8)
                            .map(|t| (value as f64 + j as f64 * 0.2 + t as f64 * 0.03) / 128.0)
                            .collect(),
                    )
                })
                .collect(),
        },
        _ => SensorFrame::Scene {
            namespace: 3,
            pixels: vec![vec![1, 2, 3], vec![0, (value % 7) as u16, 0]],
        },
    }
}
fn runtime(recursive: bool) -> ScientificRuntime {
    let config = Config {
        width: 32,
        actions: 16,
        planning_work: 100_000,
        planning_depth: 4,
        search_every: 1_000_000,
        journal_capacity: 512,
        ..Default::default()
    };
    let mut rt = ScientificRuntime::new_constructing(
        config,
        AutonomyPolicy::default(),
        vec![0; 32],
        "one-organism",
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
    rt.enable_constructing_organism_development(recursive)
        .unwrap();
    rt.observe_external_sensor_restart(&frame(0, 0)).unwrap();
    rt
}

#[test]
fn acquired_rules_extend_the_same_restored_organism_and_freeze_with_its_memory() {
    let mut rt = runtime(true);
    for i in 0..8 { episode(&mut rt, 0, i); }
    let original = rt.checkpoint_constructing().unwrap();
    let mut rt = ScientificRuntime::from_constructing_checkpoint(&original).unwrap();
    let cost = rt.unified_report().unwrap().total_external_cost;
    rt.enable_constructing_contextual_control().unwrap();
    rt.enable_constructing_acquired_sensory_rules().unwrap();
    assert_eq!(rt.unified_report().unwrap().total_external_cost, cost);
    assert_eq!(rt.organism_development_report().unwrap().episodes, 8);
    assert_eq!(rt.enable_constructing_acquired_sensory_rules(), Err(Error::Busy));
    // Actual protected feedback through two interfaces, without model injection.
    for domain in 0..2 {
        for index in 0..200 {
            let input = frame(domain, index);
            rt.observe_external_sensor_restart(&input).unwrap();
            rt.step_external_sensors(&offers(), Some(1), safe, |_| Ok(SensorPost {
                frame: input.clone(), reward: 1.0, cost: 1,
            })).unwrap();
            rt.finish_external_episode().unwrap();
        }
        let saved = rt.checkpoint_constructing().unwrap();
        rt = ScientificRuntime::from_constructing_checkpoint(&saved).unwrap();
        assert_eq!(saved, rt.checkpoint_constructing().unwrap());
    }
    let report = rt.construction_learner().unwrap().external_reward_report().unwrap();
    let rules = report.acquired_rules.unwrap();
    assert!(rules.proposals > 0);
    assert!(rules.work > 0);
    assert_eq!(rt.organism_development_report().unwrap().episodes, 408);
    assert_eq!(rt.organism_development_report().unwrap().namespaces, [4, 9]);
    assert_eq!(rt.unified_report().unwrap().contextual_namespaces, Some(vec![4, 9]));
    rt.freeze_constructing().unwrap();
    let frozen = rt.construction_learner().unwrap().external_reward_report().unwrap().learned_fingerprint;
    episode(&mut rt, 0, 201);
    assert_eq!(frozen, rt.construction_learner().unwrap().external_reward_report().unwrap().learned_fingerprint);
    assert_eq!(rt.enable_constructing_acquired_sensory_rules(), Err(Error::Blocked));
    ScientificRuntime::from_constructing_checkpoint(&rt.checkpoint_constructing().unwrap()).unwrap();
}
fn offers() -> Vec<ActionOffer> {
    (0..16)
        .map(|a| ActionOffer {
            available: a == 3 || a == 11,
            max_cost: 1,
        })
        .collect()
}
fn episode(rt: &mut ScientificRuntime, domain: usize, index: usize) {
    let input = frame(domain, index);
    rt.observe_external_sensor_restart(&input).unwrap();
    let mut calls = 0;
    let step = rt
        .step_external_sensors(&offers(), Some(1), safe, |_| {
            calls += 1;
            // Deliberately unpredictable reward stresses self-revision rather
            // than installing a hand-written evaluator or a requested generation.
            Ok(SensorPost {
                frame: input.clone(),
                cost: 1,
                reward: if index % 2 == 0 { 1.0 } else { 0.0 },
            })
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert!(step.action == 3 || step.action == 11);
    rt.finish_external_episode().unwrap();
}

#[test]
fn one_factual_lifetime_drives_existing_rsp2_and_evaluator_across_modalities_and_resume() {
    let mut rt = runtime(true);
    let before_partial_freeze = rt.checkpoint_constructing().unwrap();
    assert_eq!(rt.freeze_recursive_search(), Err(Error::Blocked));
    assert!(!rt
        .construction_learner()
        .unwrap()
        .recursive_search()
        .unwrap()
        .is_frozen());
    assert_eq!(
        before_partial_freeze,
        rt.checkpoint_constructing().unwrap(),
        "a rejected partial stop must not mutate or strand the common lifetime"
    );
    assert_eq!(
        rt.improve_recursive_search(
            &[],
            &aeterna_v1::construction::search_policy::AdmissionSuite {
                fresh: vec![],
                transfer: vec![],
                producers: vec![]
            },
            0
        ),
        Err(Error::Blocked),
        "the host cannot inject labelled RSP2 lessons into the autonomous lifetime"
    );
    for index in 0..80 {
        episode(&mut rt, 0, index);
    }
    let first = rt.checkpoint_constructing().unwrap();
    let mut rt = ScientificRuntime::from_constructing_checkpoint(&first).unwrap();
    assert_eq!(first, rt.checkpoint_constructing().unwrap());
    for index in 0..80 {
        episode(&mut rt, 1, index);
    }
    let report = rt.organism_development_report().unwrap();
    assert_eq!(report.episodes, 160);
    assert_eq!(report.namespaces.len(), 2);
    assert!(report.recursive_attempts > 0, "{report:?}");
    assert_eq!(report.recursive_errors, 0, "{report:?}");
    assert!(report.recursive_work > 0);
    assert!(report.evaluator_searches > 0, "{report:?}");
    assert_eq!(
        report.evaluator_search_policy,
        Some(report.active_search_policy)
    );
    let rsp = rt
        .construction_learner()
        .unwrap()
        .recursive_search()
        .unwrap();
    assert_eq!(rsp.audits().len() as u64, report.recursive_attempts);
    rsp.validate().unwrap();
    // The EXTERNAL5-only parameter mutator is not a second developer here.
    assert_eq!(
        rt.construction_learner()
            .unwrap()
            .external_reward_report()
            .unwrap()
            .recursive_generations,
        0
    );
    for index in 0..8 {
        episode(&mut rt, 2, index);
    }
    for index in 0..8 {
        episode(&mut rt, 0, index);
    }
    assert_eq!(
        rt.organism_development_report().unwrap().namespaces.len(),
        3
    );
    assert_eq!(rt.unified_report().unwrap().total_external_cost, 176);
    let saved = rt.checkpoint_constructing().unwrap();
    let restored = ScientificRuntime::from_constructing_checkpoint(&saved).unwrap();
    assert_eq!(saved, restored.checkpoint_constructing().unwrap());
    assert_eq!(rt.organism_development_report().unwrap().episodes, 176);
    assert!(
        rt.trajectory_self_revision_report()
            .unwrap()
            .behavior_guard_enabled
    );
    rt.freeze_constructing().unwrap();
    let before = serde_json::to_value(rt.organism_development_report()).unwrap();
    episode(&mut rt, 0, 201);
    assert_eq!(
        before,
        serde_json::to_value(rt.organism_development_report()).unwrap()
    );
}

#[test]
fn recursion_control_preserves_the_same_lifetime_without_policy_admissions() {
    let mut rt = runtime(false);
    for domain in 0..2 {
        for index in 0..80 {
            episode(&mut rt, domain, index);
        }
    }
    let report = rt.organism_development_report().unwrap();
    assert_eq!(report.episodes, 160);
    assert_eq!(report.recursive_attempts, 0);
    assert_eq!(report.recursive_work, 0);
    assert!(report.evaluator_searches > 0);
    ScientificRuntime::from_constructing_checkpoint(&rt.checkpoint_constructing().unwrap())
        .unwrap();
}

#[test]
fn unknown_post_cannot_become_a_development_episode_or_a_training_failure() {
    let mut rt = runtime(true);
    episode(&mut rt, 0, 0);
    let before = serde_json::to_value(rt.organism_development_report()).unwrap();
    assert!(rt
        .step_external_sensors(&offers(), Some(1), safe, |_| Err(Error::Execution(
            "sensor disconnected".into()
        )))
        .is_err());
    assert_eq!(rt.finish_external_episode(), Err(Error::Blocked));
    assert_eq!(
        before,
        serde_json::to_value(rt.organism_development_report()).unwrap()
    );
}

#[test]
fn freeze_abandons_unfinished_development_but_preserves_the_committed_fact() {
    let mut rt = runtime(true);
    rt.step_external_sensors(&offers(), Some(1), safe, |_| {
        Ok(SensorPost {
            frame: frame(0, 0),
            cost: 1,
            reward: 0.0,
        })
    })
    .unwrap();
    rt.freeze_constructing().unwrap();
    let report = rt.organism_development_report().unwrap();
    assert_eq!(report.episodes, 0, "freeze is not an observed task failure");
    assert_eq!(report.abandoned_episodes, 1);
    assert_eq!(report.retained_factual_examples, 0);
    assert_eq!(
        rt.construction_learner()
            .unwrap()
            .external_reward_report()
            .unwrap()
            .factual_events,
        1
    );
    let bytes = rt.checkpoint_constructing().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let dev = &value["cognition"]["development"];
    assert_eq!(dev["pending_actions"], 0);
    assert_eq!(dev["pending"], serde_json::json!([]));
    assert!(dev["pending_arm"].is_null());
    let mut restored = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(bytes, restored.checkpoint_constructing().unwrap());
    restored.finish_external_episode().unwrap();
    assert_eq!(restored.organism_development_report().unwrap().episodes, 0);
}

#[test]
fn canonical_jsonl_entrypoint_owns_the_same_state_across_sensor_contracts() {
    use serde_json::{json, Value};
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_aeterna-v1"))
        .arg("--jsonl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut command = |request: Value| -> Value {
        serde_json::to_writer(&mut input, &request).unwrap();
        writeln!(input).unwrap();
        input.flush().unwrap();
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let value: Value = serde_json::from_str(&line).unwrap();
        assert_ne!(value["type"], "error", "{value}");
        value
    };
    command(json!({"op":"organism_init","frame":frame(0,7)}));
    for domain in 0..2 {
        command(json!({"op":"external_reset","frame":frame(domain,domain+7)}));
        let intent = command(json!({"op":"external_step","offers":offers(),"remaining_actions":1}));
        assert_eq!(intent["type"], "action");
        let committed = command(
            json!({"op":"post","result":{"frame":frame(domain,domain+7),"cost":1,"reward":1.0}}),
        );
        assert_eq!(committed["type"], "committed");
        command(json!({"op":"external_episode_end"}));
    }
    let report = command(json!({"op":"organism_report"}));
    assert_eq!(report["development"]["episodes"], 2);
    assert_eq!(report["development"]["namespaces"], json!([4, 9]));
    assert_eq!(report["organism"]["total_external_cost"], 2);
    let checkpoint = command(json!({"op":"checkpoint"}));
    let bytes = checkpoint["checkpoint"].as_str().unwrap().as_bytes();
    let restored = ScientificRuntime::from_constructing_checkpoint(bytes).unwrap();
    assert_eq!(restored.organism_development_report().unwrap().episodes, 2);
    command(json!({"op":"quit"}));
    assert!(child.wait().unwrap().success());
}
