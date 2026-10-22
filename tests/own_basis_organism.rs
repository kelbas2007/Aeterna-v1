//! Whole-runtime engineering contracts. Synthetic fixtures verify wiring,
//! not intelligence or performance on independent real data.
use aeterna_v1::{
    construction::{
        autonomy::{ActionOffer, AutonomyPolicy},
        sensors::SensorFrame,
        Config, Error,
    },
    own_basis::{BasisAnswer, BasisConfig, BasisGoal},
    scientific_runtime::{ScientificRuntime, SensorPost},
    HumanProtectionEvidence,
};
fn image(i: usize) -> SensorFrame {
    let mut x = vec![
        vec![
            if i % 2 == 0 { 15 } else { 240 },
            if i % 2 == 0 { 240 } else { 15 }
        ]
        .repeat(6);
        5
    ];
    x[0][0] = (i % 256) as u8;
    x[0][1] = 255 - x[0][0];
    x[0][2] = (i / 256) as u8;
    x[0][3] = 255 - x[0][2];
    SensorFrame::Image {
        namespace: 7,
        pixels: Some(x),
    }
}
fn runtime(grow: bool) -> ScientificRuntime {
    let cfg = Config {
        width: 32,
        actions: 16,
        search_every: 1_000_000,
        journal_capacity: 64,
        planning_depth: 2,
        planning_work: 100_000,
        ..Default::default()
    };
    let mut r = ScientificRuntime::new_constructing(
        cfg,
        AutonomyPolicy::default(),
        vec![0; 32],
        "one-basis-life",
    )
    .unwrap();
    r.enable_constructing_self_directed_control().unwrap();
    r.enable_constructing_causal_memory(
        aeterna_v1::construction::causal_memory::CausalConfig::default(),
    )
    .unwrap();
    r.enable_constructing_external_reward_memory(true, false)
        .unwrap();
    r.enable_constructing_own_basis(
        BasisConfig {
            grow,
            max_coordinates: 64,
            max_features: 8,
            growth_units: 4,
            max_parameters: 8192,
            work_per_example: 100_000,
            ..Default::default()
        },
        &image(0),
    )
    .unwrap();
    r
}
fn teach(r: &mut ScientificRuntime, skill: &str, i: usize) {
    r.teach_own_basis(
        skill,
        &image(i),
        if i % 2 == 0 { "dark" } else { "bright" },
        &format!("fixture:{i}"),
    )
    .unwrap();
}
fn signature(r: &ScientificRuntime, name: &str) -> u64 {
    r.own_basis_report()
        .unwrap()
        .skills
        .iter()
        .find(|s| s.name == name)
        .unwrap()
        .model_fingerprint
}
fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.,
        hazard_confidence: 1.,
        emergency_stop: false,
    }
}
#[test]
fn explicit_teaching_learns_without_inventing_physical_events_and_retains_other_goal() {
    let mut r = runtime(false);
    let encode = |f| {
        aeterna_v1::construction::sensors::SensorEncoder::default()
            .observe(&f)
            .unwrap()
    };
    assert_eq!(
        encode(image(200)),
        encode(image(201)),
        "old summaries alias these raw orders"
    );
    for i in 0..96 {
        teach(&mut r, "light", i);
    }
    assert_eq!(r.construction_learner().unwrap().factual_event_count(), 0);
    for i in [200, 201] {
        let p = r.predict_own_basis("light", &image(i)).unwrap();
        assert!(p.probabilities[if i % 2 == 0 { "dark" } else { "bright" }] > 0.8);
    }
    let before = r.predict_own_basis("light", &image(201)).unwrap();
    let model = signature(&r, "light");
    for i in 100..196 {
        teach(&mut r, "other-instruction", i);
    }
    let after = r.predict_own_basis("light", &image(201)).unwrap();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(after).unwrap()
    );
    assert_eq!(model, signature(&r, "light"));
    assert_eq!(r.own_basis_report().unwrap().teaching_events, 192);
}
#[test]
fn pending_growth_optimizer_and_rng_resume_exactly() {
    let mut r = runtime(true);
    for i in 0..280 {
        teach(&mut r, "light", i);
    }
    assert!(r.own_basis_report().unwrap().skills[0].candidate);
    let bytes = r.checkpoint_constructing().unwrap();
    let mut resumed = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(bytes, resumed.checkpoint_constructing().unwrap());
    for i in 280..304 {
        teach(&mut r, "light", i);
        teach(&mut resumed, "light", i);
    }
    assert_eq!(
        r.checkpoint_constructing().unwrap(),
        resumed.checkpoint_constructing().unwrap()
    );
    assert_eq!(
        r.own_basis_report().unwrap().work,
        resumed.own_basis_report().unwrap().work
    );
}
#[test]
fn namespace_cannot_supply_the_answer_and_invalid_teaching_is_atomic() {
    let mut r = runtime(false);
    for i in 0..64 {
        teach(&mut r, "light", i);
    }
    let mut changed = image(201);
    if let SensorFrame::Image { namespace, .. } = &mut changed {
        *namespace = 999;
    }
    let p = r.predict_own_basis("light", &image(201)).unwrap();
    let q = r.predict_own_basis("light", &changed).unwrap();
    assert_eq!(p.probabilities, q.probabilities);
    let before = r.checkpoint_constructing().unwrap();
    let incompatible = SensorFrame::Signals {
        namespace: 7,
        channels: vec![Some(vec![0.; 4]); 9],
    };
    assert!(matches!(
        r.teach_own_basis("light", &incompatible, "dark", "bad-layout"),
        Err(Error::InvalidData)
    ));
    assert_eq!(before, r.checkpoint_constructing().unwrap());
}
#[test]
fn frozen_same_u1_uses_the_acquired_answer_goal_and_commits_one_actual_post() {
    let mut r = runtime(false);
    for i in 0..96 {
        teach(&mut r, "light", i);
    }
    r.set_own_basis_goal(Some(BasisGoal {
        skill: "light".into(),
        answers: vec![
            BasisAnswer {
                motor: 3,
                concept: "dark".into(),
            },
            BasisAnswer {
                motor: 9,
                concept: "bright".into(),
            },
        ],
    }))
    .unwrap();
    r.freeze_constructing().unwrap();
    let learned = r.own_basis_report().unwrap().learned_fingerprint;
    let before = r.checkpoint_constructing().unwrap();
    assert!(matches!(
        r.teach_own_basis("light", &image(8), "dark", "frozen"),
        Err(Error::Blocked)
    ));
    assert_eq!(before, r.checkpoint_constructing().unwrap());
    let offers: Vec<_> = (0..16)
        .map(|a| ActionOffer {
            available: a == 3 || a == 9,
            max_cost: 1,
        })
        .collect();
    for (i, wanted) in [(201, 9), (200, 3)] {
        r.observe_external_sensor_restart(&image(i)).unwrap();
        let mut calls = 0;
        let step = r
            .step_external_sensors(&offers, Some(1), safe, |action| {
                calls += 1;
                assert_eq!(action, wanted);
                Ok(SensorPost {
                    frame: image(i),
                    reward: 1.,
                    cost: 1,
                })
            })
            .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(step.action, wanted);
        assert!(step.operations.iter().any(|s| s == "evidence-dominance"));
        assert!(step.selected_fields[0] > 0.9);
    }
    assert_eq!(r.construction_learner().unwrap().factual_event_count(), 2);
    assert_eq!(r.own_basis_report().unwrap().learned_fingerprint, learned);
    let state = r.checkpoint_constructing().unwrap();
    assert_eq!(
        state,
        ScientificRuntime::from_constructing_checkpoint(&state)
            .unwrap()
            .checkpoint_constructing()
            .unwrap()
    );
}
#[test]
fn unavailable_alternative_does_not_invent_certainty() {
    let mut r = runtime(false);
    for i in 0..96 {
        teach(&mut r, "light", i);
    }
    r.set_own_basis_goal(Some(BasisGoal {
        skill: "light".into(),
        answers: vec![
            BasisAnswer {
                motor: 3,
                concept: "dark".into(),
            },
            BasisAnswer {
                motor: 9,
                concept: "bright".into(),
            },
        ],
    }))
    .unwrap();
    r.freeze_constructing().unwrap();
    r.observe_external_sensor_restart(&image(201)).unwrap();
    let offers: Vec<_> = (0..16)
        .map(|a| ActionOffer {
            available: a == 3,
            max_cost: 1,
        })
        .collect();
    let step = r
        .step_external_sensors(&offers, Some(1), safe, |a| {
            assert_eq!(a, 3);
            Ok(SensorPost {
                frame: image(201),
                reward: 0.,
                cost: 1,
            })
        })
        .unwrap();
    assert!(
        step.selected_fields[0] < 0.1,
        "an unavailable correct answer cannot increase the wrong forecast"
    );
}
#[test]
fn unaffordable_teaching_preserves_the_entire_life() {
    let cfg = Config {
        width: 32,
        actions: 16,
        search_every: 1_000_000,
        ..Default::default()
    };
    let mut r = ScientificRuntime::new_constructing(
        cfg,
        AutonomyPolicy::default(),
        vec![0; 32],
        "budget-basis",
    )
    .unwrap();
    let f = SensorFrame::Image {
        namespace: 7,
        pixels: Some(vec![vec![0; 32]; 32]),
    };
    r.enable_constructing_self_directed_control().unwrap();
    r.enable_constructing_causal_memory(
        aeterna_v1::construction::causal_memory::CausalConfig::default(),
    )
    .unwrap();
    r.enable_constructing_external_reward_memory(true, false)
        .unwrap();
    r.enable_constructing_own_basis(
        BasisConfig {
            grow: false,
            max_coordinates: 1024,
            work_per_example: 140_000,
            ..Default::default()
        },
        &f,
    )
    .unwrap();
    for _ in 0..7 {
        r.teach_own_basis("objects", &f, "one", "budget-fixture")
            .unwrap();
    }
    let bytes = r.checkpoint_constructing().unwrap();
    assert!(matches!(
        r.teach_own_basis("objects", &f, "two", "budget-fixture"),
        Err(Error::Budget)
    ));
    assert_eq!(bytes, r.checkpoint_constructing().unwrap());
}
#[test]
fn actual_measurement_post_supplies_the_next_protected_answer() {
    let mut r = runtime(false);
    for i in 0..96 {
        teach(&mut r, "light", i);
    }
    r.set_own_basis_goal(Some(BasisGoal {
        skill: "light".into(),
        answers: vec![
            BasisAnswer {
                motor: 3,
                concept: "dark".into(),
            },
            BasisAnswer {
                motor: 9,
                concept: "bright".into(),
            },
        ],
    }))
    .unwrap();
    r.freeze_constructing().unwrap();
    let learned = r.own_basis_report().unwrap().learned_fingerprint;
    let blank = SensorFrame::Image {
        namespace: 7,
        pixels: None,
    };
    r.observe_external_sensor_restart(&blank).unwrap();
    assert!(matches!(
        r.predict_own_basis("light", &blank),
        Err(Error::NoEvidence)
    ));
    let measure: Vec<_> = (0..16)
        .map(|a| ActionOffer {
            available: a == 0,
            max_cost: 1,
        })
        .collect();
    r.step_external_sensors(&measure, Some(2), safe, |a| {
        assert_eq!(a, 0);
        Ok(SensorPost {
            frame: image(201),
            reward: 0.,
            cost: 1,
        })
    })
    .unwrap();
    let answers: Vec<_> = (0..16)
        .map(|a| ActionOffer {
            available: a == 3 || a == 9,
            max_cost: 1,
        })
        .collect();
    let step = r
        .step_external_sensors(&answers, Some(1), safe, |a| {
            assert_eq!(a, 9);
            Ok(SensorPost {
                frame: image(201),
                reward: 0.,
                cost: 1,
            })
        })
        .unwrap();
    assert_eq!(step.action, 9);
    assert_eq!(r.construction_learner().unwrap().factual_event_count(), 2);
    assert_eq!(r.own_basis_report().unwrap().learned_fingerprint, learned);
}
#[test]
fn basis_and_recurrent_world_share_the_same_action_and_restored_life() {
    let mut r = runtime(false);
    r.enable_constructing_learning_world(
        aeterna_v1::learning_world::LearningWorldConfig {
            latent: 4,
            context_frames: 2,
            max_coordinates: 128,
            max_frame_values: 128,
            memory_entries: 4,
            memory_bytes: 65536,
            work_per_step: 200_000,
            reconstruction_targets: 8,
            replay_updates: 1,
            ..Default::default()
        },
        &image(0),
    )
    .unwrap();
    for i in 0..96 {
        teach(&mut r, "light", i);
    }
    r.set_own_basis_goal(Some(BasisGoal {
        skill: "light".into(),
        answers: vec![
            BasisAnswer {
                motor: 3,
                concept: "dark".into(),
            },
            BasisAnswer {
                motor: 9,
                concept: "bright".into(),
            },
        ],
    }))
    .unwrap();
    r.observe_external_sensor_restart(&image(201)).unwrap();
    let offers: Vec<_> = (0..16)
        .map(|a| ActionOffer {
            available: a == 3 || a == 9,
            max_cost: 1,
        })
        .collect();
    let mut calls = 0;
    r.step_external_sensors(&offers, Some(1), safe, |a| {
        calls += 1;
        assert_eq!(a, 9);
        Ok(SensorPost {
            frame: image(201),
            reward: 0.,
            cost: 1,
        })
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(r.construction_learner().unwrap().factual_event_count(), 1);
    assert!(r.learning_world_report().unwrap().online_updates > 0);
    let before = r.own_basis_report().unwrap().learned_fingerprint;
    let bytes = r.checkpoint_constructing().unwrap();
    let resumed = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(bytes, resumed.checkpoint_constructing().unwrap());
    assert_eq!(
        before,
        resumed.own_basis_report().unwrap().learned_fingerprint
    );
    assert!(resumed.unified_report().unwrap().learning_world.is_some());
}
