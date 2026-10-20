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
