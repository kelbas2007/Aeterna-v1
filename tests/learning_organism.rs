//! Wiring/learning regressions for ONE actual runtime. These are engineering
//! fixtures, not a general-intelligence or independent external qualification.
use aeterna_v1::{
    construction::{
        autonomy::{ActionOffer, AutonomyPolicy},
        causal_memory::CausalConfig,
        sensors::{SensorEncoder, SensorFrame},
        Config, Error,
    },
    learning_world::LearningWorldConfig,
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
fn image(reverse: bool) -> SensorFrame {
    SensorFrame::Image {
        namespace: 7,
        pixels: Some(vec![
            if reverse {
                vec![255, 0].repeat(6)
            } else {
                vec![0, 255].repeat(6)
            };
            5
        ]),
    }
}
fn signal(reverse: bool) -> SensorFrame {
    let mut values = vec![0.0, 1.0, 0.0, 2.0];
    if reverse {
        values.reverse();
    }
    SensorFrame::Signals {
        namespace: 7,
        channels: std::iter::once(Some(values))
            .chain((0..8).map(|_| None))
            .collect(),
    }
}
fn packet(reverse: bool) -> SensorFrame {
    SensorFrame::Multimodal {
        namespace: 7,
        frames: vec![image(reverse), signal(reverse)],
    }
}
fn blank() -> SensorFrame {
    SensorFrame::Multimodal {
        namespace: 7,
        frames: vec![
            SensorFrame::Image {
                namespace: 7,
                pixels: Some(vec![vec![127; 12]; 5]),
            },
            signal(false),
        ],
    }
}
fn new_cue(reverse: bool) -> SensorFrame {
    let mut p = packet(reverse);
    if let SensorFrame::Multimodal { frames, .. } = &mut p {
        if let SensorFrame::Image {
            pixels: Some(rows), ..
        } = &mut frames[0]
        {
            for value in rows.iter_mut().flatten() {
                *value = if *value == 0 { 17 } else { 238 };
            }
        }
        if let SensorFrame::Signals { channels, .. } = &mut frames[1] {
            for value in channels.iter_mut().flatten().flatten() {
                *value = 0.1 + *value * 0.85;
            }
        }
    }
    p
}
fn profile() -> LearningWorldConfig {
    LearningWorldConfig {
        context_frames: 4,
        max_coordinates: 256,
        max_frame_values: 128,
        memory_entries: 16,
        memory_bytes: 256 * 1024,
        work_per_step: 1_000_000,
        reconstruction_targets: 32,
        ..Default::default()
    }
}
fn legacy(frame: &SensorFrame) -> ScientificRuntime {
    let cfg = Config {
        width: 32,
        actions: 16,
        planning_work: 100_000,
        planning_depth: 3,
        search_every: 1_000_000,
        journal_capacity: 512,
        ..Default::default()
    };
    let mut rt = ScientificRuntime::new_constructing(
        cfg,
        AutonomyPolicy::default(),
        vec![0; 32],
        "shared-learning-life",
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
    rt.enable_constructing_external_reward_memory(true, false)
        .unwrap();
    rt.enable_constructing_organism_development(false).unwrap();
    rt.observe_external_sensor_restart(frame).unwrap();
    rt
}
fn organism(frame: &SensorFrame) -> ScientificRuntime {
    let mut rt = legacy(frame);
    rt.enable_constructing_learning_world(profile(), frame)
        .unwrap();
    rt
}
fn offers(actions: &[usize]) -> Vec<ActionOffer> {
    (0..16)
        .map(|a| ActionOffer {
            available: actions.contains(&a),
            max_cost: 1,
        })
        .collect()
}
fn step(rt: &mut ScientificRuntime, allowed: &[usize], post: SensorFrame, reward: f32) -> usize {
    let mut calls = 0;
    let result = rt
        .step_external_sensors(&offers(allowed), Some(2), safe, |_| {
            calls += 1;
            Ok(SensorPost {
                frame: post,
                reward,
                cost: 1,
            })
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert!(allowed.contains(&result.action));
    assert_eq!(
        result.computation_work,
        rt.unified_report().unwrap().computation_work
    );
    result.action
}

#[test]
fn raw_order_and_simultaneous_modalities_reach_the_same_trainable_belief() {
    let a = packet(false);
    let b = packet(true);
    assert_eq!(
        SensorEncoder::default().observe(&a).unwrap(),
        SensorEncoder::default().observe(&b).unwrap()
    );
    let ra = organism(&a);
    let rb = organism(&b);
    let a = ra.learning_world_report().unwrap();
    let b = rb.learning_world_report().unwrap();
    assert_eq!(a.coordinates, 66);
    assert_eq!(a.coordinates, b.coordinates);
    assert_eq!(a.model_fingerprint, b.model_fingerprint);
    assert_ne!(
        a.belief, b.belief,
        "raw spatial/temporal order survives the legacy projection collision"
    );
    let before = serde_json::to_vec(&SensorEncoder::default()).unwrap();
    let mut e = SensorEncoder::default();
    let malformed = SensorFrame::Multimodal {
        namespace: 7,
        frames: vec![image(false), image(true)],
    };
    assert_eq!(e.observe(&malformed), Err(Error::InvalidData));
    assert_eq!(before, serde_json::to_vec(&e).unwrap());
}

#[test]
fn one_life_learns_raw_temporal_context_and_preserves_it_across_restore_and_freeze() {
    let mut rt = organism(&packet(false));
    let original = rt.learning_world_report().unwrap();
    // A cue disappears before the rewarded decision. Both cue variants have
    // EXACTLY the same old 32-vector. Only the new learned raw history differs.
    for episode in 0..640 {
        let reverse = episode % 2 != 0;
        rt.observe_external_sensor_restart(&packet(reverse))
            .unwrap();
        step(&mut rt, &[0], blank(), 0.0);
        let action = if episode % 4 < 2 { 3 } else { 11 };
        let reward = if (action == 3) != reverse { 1.0 } else { -1.0 };
        step(&mut rt, &[action], blank(), reward);
        rt.finish_external_episode().unwrap();
        if episode == 319 {
            let saved = rt.checkpoint_constructing().unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&saved).unwrap()["version"],
                2
            );
            rt = ScientificRuntime::from_constructing_checkpoint(&saved).unwrap();
            assert_eq!(saved, rt.checkpoint_constructing().unwrap());
        }
    }
    let learned = rt.learning_world_report().unwrap();
    assert_eq!(learned.factual_updates, 1280);
    assert_eq!(learned.coordinates, original.coordinates);
    assert!(learned.online_updates > 1000 && learned.consolidation_updates > 1000);
    assert_ne!(learned.model_fingerprint, original.model_fingerprint);
    assert!(learned.retained_experiences <= profile().memory_entries);
    assert!(learned.reserved_memory_bytes <= profile().memory_bytes);
    assert!(learned.work <= rt.unified_report().unwrap().computation_work);
    let mut correct = 0;
    for episode in 0..80 {
        let reverse = episode % 2 != 0;
        rt.observe_external_sensor_restart(&packet(reverse))
            .unwrap();
        step(&mut rt, &[0], blank(), 0.0);
        let action = rt
            .step_external_sensors(&offers(&[3, 11]), Some(1), safe, |action| {
                let success = (action == 3) != reverse;
                if success {
                    correct += 1;
                }
                Ok(SensorPost {
                    frame: blank(),
                    reward: if success { 1.0 } else { -1.0 },
                    cost: 1,
                })
            })
            .unwrap()
            .action;
        assert!(action == 3 || action == 11);
        rt.finish_external_episode().unwrap();
    }
    assert!(rt.learning_world_report().unwrap().selected_proposals > 0);
    assert!(
        correct >= 60,
        "U1 must use the acquired raw/context model: {correct}/80"
    );
    rt.freeze_constructing().unwrap();
    let fixed = rt.learning_world_report().unwrap().learned_fingerprint;
    let mut predictions = Vec::new();
    for reverse in [false, true] {
        rt.observe_external_sensor_restart(&new_cue(reverse))
            .unwrap();
        step(&mut rt, &[0], blank(), 0.0);
        let report = rt.learning_world_report().unwrap();
        predictions.push(
            report
                .action_forecasts
                .iter()
                .find(|p| p.action == 3)
                .unwrap()
                .reward,
        );
        assert_eq!(fixed, report.learned_fingerprint);
        let state = rt.checkpoint_constructing().unwrap();
        let mut resumed = ScientificRuntime::from_constructing_checkpoint(&state).unwrap();
        assert_eq!(state, resumed.checkpoint_constructing().unwrap());
        let a = step(&mut rt, &[3, 11], blank(), 0.0);
        let b = step(&mut resumed, &[3, 11], blank(), 0.0);
        assert_eq!(a, b);
        assert_eq!(
            rt.checkpoint_constructing().unwrap(),
            resumed.checkpoint_constructing().unwrap()
        );
    }
    assert!(
        predictions[0] > 0.3 && predictions[1] < -0.3,
        "acquired context-dependent reward predictions: {predictions:?}"
    );
}

#[test]
fn a_version_one_life_migrates_without_inventing_old_raw_experience() {
    let mut rt = legacy(&image(false));
    for _ in 0..4 {
        step(&mut rt, &[3], image(false), 1.0);
        rt.finish_external_episode().unwrap();
    }
    let old = rt.checkpoint_constructing().unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&old).unwrap()["version"],
        1
    );
    let original = rt.unified_report().unwrap();
    let mut rt = ScientificRuntime::from_constructing_checkpoint(&old).unwrap();
    rt.enable_constructing_learning_world(profile(), &packet(false))
        .unwrap();
    let report = rt.learning_world_report().unwrap();
    assert_eq!(report.attached_after_event, 4);
    assert_eq!(report.factual_updates, 0);
    assert_eq!(report.retained_experiences, 0);
    assert_eq!(
        rt.unified_report().unwrap().total_external_cost,
        original.total_external_cost
    );
    assert_eq!(
        rt.unified_report().unwrap().learner.events,
        original.learner.events
    );
    let saved = rt.checkpoint_constructing().unwrap();
    assert_eq!(
        saved,
        ScientificRuntime::from_constructing_checkpoint(&saved)
            .unwrap()
            .checkpoint_constructing()
            .unwrap()
    );
    let mut calls = 0;
    let before = rt.checkpoint_constructing().unwrap();
    assert!(matches!(
        rt.step_constructing(None, &offers(&[3]), safe, |_| {
            calls += 1;
            unreachable!()
        }),
        Err(Error::Blocked)
    ));
    assert_eq!(calls, 0);
    assert_eq!(before, rt.checkpoint_constructing().unwrap());
}

#[test]
fn goal_updates_working_state_without_adding_facts_or_learning_weights() {
    let mut rt = organism(&packet(false));
    let before = rt.learning_world_report().unwrap();
    rt.set_learning_world_goal(Some(packet(true)), 0.01)
        .unwrap();
    let goal = rt.learning_world_report().unwrap();
    assert!(goal.goal_active);
    assert_eq!(goal.goal_satisfied, Some(false));
    assert_eq!(goal.factual_updates, before.factual_updates);
    assert_eq!(goal.learned_fingerprint, before.learned_fingerprint);
    step(&mut rt, &[3, 11], packet(true), 0.0);
    assert_eq!(
        rt.learning_world_report().unwrap().goal_satisfied,
        Some(true)
    );
    let saved = rt.checkpoint_constructing().unwrap();
    let mut rt = ScientificRuntime::from_constructing_checkpoint(&saved).unwrap();
    rt.set_learning_world_goal(None, 0.01).unwrap();
    assert!(!rt.learning_world_report().unwrap().goal_active);
}

#[test]
fn blocked_and_invalid_actions_do_not_teach_fictional_post_observations() {
    let mut rt = organism(&packet(false));
    let before = rt.learning_world_report().unwrap();
    let mut calls = 0;
    assert!(rt
        .step_external_sensors(
            &offers(&[3, 11]),
            Some(1),
            |a| {
                let mut h = safe(a);
                h.emergency_stop = true;
                h
            },
            |_| {
                calls += 1;
                unreachable!()
            }
        )
        .is_err());
    assert_eq!(calls, 0);
    assert_eq!(
        rt.learning_world_report().unwrap().learned_fingerprint,
        before.learned_fingerprint
    );
    let mut rt = organism(&packet(false));
    let before = rt.learning_world_report().unwrap();
    assert!(rt
        .step_external_sensors(&offers(&[3, 11]), Some(1), safe, |_| {
            calls += 1;
            Ok(SensorPost {
                frame: SensorFrame::Image {
                    namespace: 7,
                    pixels: Some(Vec::new()),
                },
                reward: 1.0,
                cost: 1,
            })
        })
        .is_err());
    assert_eq!(calls, 1);
    assert_eq!(rt.learning_world_report().unwrap().factual_updates, 0);
    assert_eq!(
        rt.learning_world_report().unwrap().learned_fingerprint,
        before.learned_fingerprint
    );
    assert_eq!(rt.unified_report().unwrap().total_external_cost, 1);
    assert!(rt.emergency_latched());
}
