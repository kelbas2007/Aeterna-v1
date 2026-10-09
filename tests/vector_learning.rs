use aeterna_v1::carrier::{PhaseNativeConfig, PhaseOnlineConfig, PhaseVectorConfig};
use aeterna_v1::scientific_runtime::{ReasoningMode, RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{Authority, EvoConfig, EvoPhase, HumanProtectionEvidence};
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
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: 2,
        motor_cells: 3,
        dormant_cells: 12,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_vector_learning(PhaseVectorConfig {
        slots_per_motor: 4,
        neighbors: 1,
        minimum_margin: 0.0,
        exploration_observations: 8,
        sensing_support: 2,
        ..Default::default()
    }));
    e
}
fn frame(x: f32) -> Vec<Option<f32>> {
    vec![Some(x), Some(x)]
}
fn teach(rt: &mut ScientificRuntime, x: f32, correct: usize) {
    rt.observe_external_partial(&frame(x)).unwrap();
    for _ in 0..3 {
        if rt.goal_reached().unwrap() {
            break;
        }
        assert!(matches!(
            rt.step_partial(
                |_| Some(safe()),
                |a| Ok((frame(x), f32::from(a == correct)))
            )
            .unwrap(),
            StepOutcome::Executed { .. }
        ));
    }
    assert!(rt.goal_reached().unwrap());
}
fn acquired() -> EvoPhase {
    let mut rt = ScientificRuntime::new(fresh()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    for i in 0..80 {
        let x = f32::from(i % 2 == 1);
        teach(&mut rt, x, 1 + i % 2);
    }
    EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap()
}
#[test]
fn grayscale_endpoints_are_distinct_and_predictions_do_not_become_facts() {
    let e = acquired();
    let a = e.phase_vector_predict(&frame(0.0)).unwrap();
    let b = e.phase_vector_predict(&frame(1.0)).unwrap();
    assert_eq!((a.action, b.action), (1, 2));
    assert_eq!(a.authority, Authority::Imagined);
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external(&[0.0, 0.0]).unwrap();
    assert!(!rt.goal_reached().unwrap());
    assert_eq!(
        rt.set_goal(&[1.0, 1.0]),
        Err(RuntimeError::VectorModeRequired)
    );
}
#[test]
fn acquired_sensor_and_response_use_the_shared_protected_runtime() {
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    rt.set_model_learning_enabled(false);
    rt.set_outcome_goal(1.0).unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    rt.observe_external_partial(&[None, None]).unwrap();
    let p = rt.propose_vector().unwrap().unwrap();
    assert_eq!((p.action, p.mode), (0, ReasoningMode::VectorMeasurement));
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            assert_eq!(a, 0);
            Ok((frame(1.0), 0.0))
        },
    )
    .unwrap();
    assert!(!rt.goal_reached().unwrap());
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            assert_eq!(a, 2);
            Ok((frame(1.0), 1.0))
        },
    )
    .unwrap();
    assert!(rt.goal_reached().unwrap());
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}
#[test]
fn factual_correction_weakens_the_wrong_physical_response_and_learns_the_new_one() {
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let before = rt.organism().online_checkpoint_bytes().unwrap();
    let old: serde_json::Value = serde_json::from_slice(&before).unwrap();
    let before_fp = rt.organism().phase_native_learned_fingerprint();
    teach(&mut rt, 0.0, 2);
    let new: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    let response = old["vector"]["motors"][1]["prototypes"][0]["response"]
        .as_u64()
        .unwrap() as usize;
    assert!(
        new["synapses"][response]["weight"].as_f64() < old["synapses"][response]["weight"].as_f64()
    );
    assert_ne!(rt.organism().phase_native_learned_fingerprint(), before_fp);
    assert_eq!(
        rt.organism()
            .phase_vector_predict(&frame(0.0))
            .unwrap()
            .action,
        2
    );
}
#[test]
fn actual_feature_and_motor_links_are_necessary_and_exact_restore_recovers_them() {
    let mut e = acquired();
    let expected = e.phase_vector_predict(&frame(0.0)).unwrap();
    let indices = e.phase_vector_info().unwrap().prototype_synapses;
    let mut saved = Vec::new();
    for &i in &indices {
        saved.push((
            i,
            e.perturb_phase_native_synapse_for_control(i, 0.0, 0.0)
                .unwrap(),
        ));
    }
    assert!(e.phase_vector_predict(&frame(0.0)).is_none());
    for (i, s) in saved {
        e.restore_phase_native_synapse_for_control(i, s);
    }
    assert_eq!(e.phase_vector_predict(&frame(0.0)).unwrap(), expected);
    let mut shifted = Vec::new();
    for i in indices {
        let s = e.phase_native_synapse(i).unwrap();
        if s.from < 2 && s.weight > 0.0 {
            shifted.push((
                i,
                e.perturb_phase_native_synapse_for_control(i, 1.0, std::f32::consts::PI)
                    .unwrap(),
            ));
        }
    }
    assert_ne!(e.phase_vector_predict(&frame(0.0)), Some(expected.clone()));
    for (i, s) in shifted {
        e.restore_phase_native_synapse_for_control(i, s);
    }
    assert_eq!(e.phase_vector_predict(&frame(0.0)).unwrap(), expected);
}
#[test]
fn missing_out_of_support_and_invalid_inputs_do_not_produce_a_response() {
    let e = acquired();
    assert!(e.phase_vector_predict(&[None, Some(0.0)]).is_none());
    assert!(e.phase_vector_predict(&[Some(0.5), Some(0.5)]).is_none());
    for v in [f32::NAN, f32::INFINITY, -0.01, 1.01] {
        assert!(e.phase_vector_predict(&[Some(v), Some(0.0)]).is_none());
    }
    let mut cfg = PhaseVectorConfig::default();
    cfg.slots_per_motor = 65;
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    rt.set_model_learning_enabled(false);
    rt.set_outcome_goal(1.0).unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    assert_eq!(
        rt.observe_external(&[f32::NAN, 0.0]),
        Err(RuntimeError::InvalidRaster)
    );
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    let mut cold = EvoPhase::new(EvoConfig::default());
    cold.enable_phase_native_planning(PhaseNativeConfig::default());
    cold.enable_phase_native_online_learning(PhaseOnlineConfig::default());
    assert!(!cold.enable_phase_vector_learning(cfg));
}
#[test]
fn bounded_lifelong_feedback_keeps_fixed_topology_and_provenance() {
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let before: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    for i in 0..512 {
        let x = (i * 37 % 101) as f32 / 100.0;
        teach(&mut rt, x, if x < 0.5 { 1 } else { 2 });
    }
    let after: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    assert_eq!(
        before["cells"].as_array().unwrap().len(),
        after["cells"].as_array().unwrap().len()
    );
    assert_eq!(
        before["synapses"].as_array().unwrap().len(),
        after["synapses"].as_array().unwrap().len()
    );
    assert!(rt.organism().phase_vector_info().unwrap().active_prototypes <= 12);
    for m in after["vector"]["motors"].as_array().unwrap() {
        for p in m["prototypes"].as_array().unwrap() {
            assert!(p["sources"].as_array().unwrap().len() <= 2);
        }
    }
}
#[test]
fn restore_is_atomic_preserves_freeze_goal_stop_and_requires_fresh_frame() {
    let e = acquired();
    let bytes = e.online_checkpoint_bytes().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["version"], 9);
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external(&[0.0, 0.0]).unwrap();
    let fp = rt.organism().phase_native_learned_fingerprint();
    for field in ["version", "slots", "links"] {
        let mut bad = value.clone();
        match field {
            "version" => bad["version"] = 6.into(),
            "slots" => bad["vector"]["config"]["slots_per_motor"] = 65.into(),
            _ => {
                bad["vector"]["motors"][0]["prototypes"][0]["inputs"][1] =
                    bad["vector"]["motors"][0]["prototypes"][0]["inputs"][0].clone()
            }
        }
        assert_eq!(
            rt.restore_online_checkpoint(&serde_json::to_vec(&bad).unwrap()),
            Err(RuntimeError::InvalidCheckpoint)
        );
        assert_eq!(rt.organism().phase_native_learned_fingerprint(), fp);
    }
    rt.restore_online_checkpoint(&bytes).unwrap();
    assert_eq!(
        rt.propose_vector(),
        Err(RuntimeError::FreshObservationRequired)
    );
    rt.observe_external(&[0.0, 0.0]).unwrap();
    let mut stop = safe();
    stop.emergency_stop = true;
    assert!(matches!(
        rt.step_partial(|_| Some(stop), |_| panic!("blocked callback")),
        Ok(StepOutcome::Blocked(_))
    ));
    rt.restore_online_checkpoint(&bytes).unwrap();
    rt.observe_external(&[0.0, 0.0]).unwrap();
    assert!(rt.emergency_latched());
    assert!(matches!(
        rt.step_partial(|_| Some(safe()), |_| panic!("restore bypassed latch")),
        Ok(StepOutcome::Blocked(_))
    ));
    assert_eq!(rt.organism().phase_native_learned_fingerprint(), fp);
}
#[test]
fn unusable_post_latches_fault_and_does_not_teach_any_parameters() {
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external(&[0.0, 0.0]).unwrap();
    let fp = rt.organism().phase_native_learned_fingerprint();
    assert!(matches!(
        rt.step_partial(
            |_| Some(safe()),
            |_| Ok((vec![Some(f32::NAN), Some(0.0)], 1.0))
        )
        .unwrap(),
        StepOutcome::ExecutionFault(_)
    ));
    assert!(rt.emergency_latched());
    assert_eq!(rt.organism().phase_native_learned_fingerprint(), fp);
}
