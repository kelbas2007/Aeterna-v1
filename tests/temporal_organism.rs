//! Whole-runtime correctness checks; not intelligence qualification.
use aeterna_v1::{
    construction::{
        autonomy::{ActionOffer, AutonomyPolicy},
        sensors::SensorFrame,
        Config, Error,
    },
    learning_world::{LearningWorldConfig, TemporalConfig},
    scientific_runtime::{ScientificRuntime, SensorPost},
    HumanProtectionEvidence,
};

fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.,
        hazard_confidence: 1.,
        emergency_stop: false,
    }
}
fn frame(ns: u16, value: f64) -> SensorFrame {
    SensorFrame::Signals {
        namespace: ns,
        channels: std::iter::once(Some(vec![value]))
            .chain((0..8).map(|_| None))
            .collect(),
    }
}
fn offers() -> Vec<ActionOffer> {
    (0..16)
        .map(|a| ActionOffer {
            available: a == 3,
            max_cost: 1,
        })
        .collect()
}
fn runtime(ns: u16) -> ScientificRuntime {
    let cfg = Config {
        width: 32,
        actions: 16,
        search_every: 1_000_000,
        journal_capacity: 512,
        planning_work: 10_000,
        ..Default::default()
    };
    let mut rt = ScientificRuntime::new_constructing(
        cfg,
        AutonomyPolicy::default(),
        vec![0; 32],
        "one-temporal-life",
    )
    .unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_causal_memory(aeterna_v1::construction::causal_memory::CausalConfig {
        history_depth: 2,
        model_entries: 64,
        model_bytes: 65536,
        outcomes_per_entry: 4,
        planning_states: 32,
    })
    .unwrap();
    rt.enable_constructing_external_reward_memory(true, false)
        .unwrap();
    rt.enable_constructing_organism_development(false).unwrap();
    let first = frame(ns, 0.2);
    rt.observe_external_sensor_restart(&first).unwrap();
    rt.enable_constructing_learning_world(
        LearningWorldConfig {
            max_coordinates: 128,
            max_frame_values: 64,
            memory_entries: 8,
            memory_bytes: 65536,
            work_per_step: 16_000_000,
            ..Default::default()
        },
        &first,
    )
    .unwrap();
    rt.enable_constructing_temporal_learning(TemporalConfig {
        search: false,
        ..Default::default()
    })
    .unwrap();
    rt
}
fn step(rt: &mut ScientificRuntime, ns: u16, value: f64) {
    let mut calls = 0;
    rt.step_external_sensors(&offers(), Some(1), safe, |a| {
        assert_eq!(a, 3);
        calls += 1;
        Ok(SensorPost {
            frame: frame(ns, value),
            reward: 0.2,
            cost: 1,
        })
    })
    .unwrap();
    assert_eq!(calls, 1);
}

#[test]
fn temporal_one_life_learning_restore_and_freeze() {
    let values: Vec<f64> =
        serde_json::from_str(include_str!("data/temporal_cpu_development.json")).unwrap();
    let mut rt = runtime(90);
    for &value in values.iter().take(80) {
        step(&mut rt, 90, value);
    }
    let report = rt.learning_world_report().unwrap();
    let t = report.temporal.as_ref().unwrap();
    assert_eq!(t.committed_frames, 80);
    assert!(t.parameter_updates > 60);
    assert_eq!(t.forecast_values, 80);
    let saved = rt.checkpoint_constructing().unwrap();
    let mut resumed = ScientificRuntime::from_constructing_checkpoint(&saved).unwrap();
    assert_eq!(saved, resumed.checkpoint_constructing().unwrap());
    for &value in &values[80..112] {
        step(&mut rt, 90, value);
        step(&mut resumed, 90, value);
    }
    assert_eq!(
        rt.checkpoint_constructing().unwrap(),
        resumed.checkpoint_constructing().unwrap()
    );
    rt.freeze_constructing().unwrap();
    let learned = rt
        .learning_world_report()
        .unwrap()
        .temporal
        .unwrap()
        .learned_fingerprint;
    for &value in &values[112..] {
        step(&mut rt, 90, value);
    }
    let after = rt.learning_world_report().unwrap().temporal.unwrap();
    assert_eq!(after.learned_fingerprint, learned);
    assert_eq!(after.committed_frames, 144);
    ScientificRuntime::from_constructing_checkpoint(&rt.checkpoint_constructing().unwrap())
        .unwrap();
}

#[test]
fn temporal_namespace_is_not_a_predictive_feature() {
    let mut a = runtime(90);
    let mut b = runtime(91);
    for i in 0..40 {
        let value = (i as f64 * 0.7).sin() * 0.1 + 0.5;
        step(&mut a, 90, value);
        step(&mut b, 91, value);
    }
    assert_eq!(
        a.predict_constructing_temporal().unwrap().values,
        b.predict_constructing_temporal().unwrap().values
    );
}

#[test]
fn temporal_invalid_real_post_never_trains_or_advances_the_model() {
    let mut rt = runtime(90);
    let before = rt.learning_world_report().unwrap().temporal.unwrap();
    let error = rt
        .step_external_sensors(&offers(), Some(1), safe, |_| {
            Ok(SensorPost {
                frame: frame(90, f64::NAN),
                reward: 0.2,
                cost: 1,
            })
        })
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Execution(_) | Error::InvalidData | Error::Blocked
    ));
    let after = rt.learning_world_report().unwrap().temporal.unwrap();
    assert_eq!(before.learned_fingerprint, after.learned_fingerprint);
    assert_eq!(after.committed_frames, 0);
}

#[test]
fn temporal_computation_policy_keeps_learning_rewards_when_reconstruction_is_paused() {
    let mut rt = runtime(90);
    for _ in 0..180 {
        step(&mut rt, 90, 0.8);
    }
    let report = rt.learning_world_report().unwrap().temporal.unwrap();
    assert!(report.neural_reconstruction_paused);
    let before: serde_json::Value =
        serde_json::from_slice(&rt.checkpoint_constructing().unwrap()).unwrap();
    for _ in 0..24 {
        rt.step_external_sensors(&offers(), Some(1), safe, |_| {
            Ok(SensorPost {
                frame: frame(90, 0.8),
                reward: -0.1,
                cost: 1,
            })
        })
        .unwrap();
    }
    let after: serde_json::Value =
        serde_json::from_slice(&rt.checkpoint_constructing().unwrap()).unwrap();
    let a = &before["cognition"]["learning_world"]["network"];
    let b = &after["cognition"]["learning_world"]["network"];
    assert_eq!(a["coordinates"], b["coordinates"]);
    assert_eq!(a["recurrence"], b["recurrence"]);
    assert_ne!(a["heads"][0]["reward"], b["heads"][0]["reward"]);
}

#[test]
fn temporal_world_prediction_is_fixed_before_the_real_answer() {
    let mut rt = runtime(90);
    for i in 0..36 {
        step(&mut rt, 90, 0.3 + 0.1 * (i as f64).sin());
    }
    let previous = rt.learning_world_report().unwrap().temporal.unwrap();
    let forecast = rt.predict_constructing_world(3).unwrap();
    let p = forecast.values["s:0:1:0"];
    let value = 0.61;
    let expected = (p - value / (1. + value)).powi(2);
    step(&mut rt, 90, value);
    let after = rt.learning_world_report().unwrap().temporal.unwrap();
    let measured = after.world_forecast_mse * after.world_forecast_values as f64
        - previous.world_forecast_mse * previous.world_forecast_values as f64;
    assert!((measured - expected).abs() < 1e-12);
}
