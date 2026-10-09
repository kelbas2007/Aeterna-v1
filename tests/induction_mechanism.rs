#[path = "../examples/support/induction.rs"]
mod induction;
use aeterna_v1::carrier::PhaseInductionConfig;
use aeterna_v1::scientific_runtime::{ReasoningMode, RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{Authority, EvoPhase};
fn acquired() -> EvoPhase {
    EvoPhase::from_online_checkpoint(&induction::train(induction::Function::Two(6), 0).0).unwrap()
}
fn frame(a: f32, b: f32) -> Vec<Option<f32>> {
    vec![Some(a), Some(b), Some(0.28), Some(0.28)]
}
#[test]
fn cold_candidates_and_query_cannot_manufacture_confirmations() {
    let e = induction::fresh(4);
    assert!(e
        .phase_induction_programs()
        .iter()
        .all(|p| p.nodes.is_empty()));
    assert!(e.phase_induction_predict(&frame(0.2, 0.8)).is_none());
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    // A single factual success may seed a candidate, but cannot confirm it.
    induction::teach(
        &mut rt,
        &[0.2, 0.8, 0.2, 0.2],
        1,
        &induction::roles(0),
        None,
    );
    assert!(rt
        .organism()
        .phase_induction_predict(&frame(0.2, 0.8))
        .is_none());
    let e = acquired();
    let fingerprint = e.phase_native_learned_fingerprint();
    let facts = e.phase_induction_programs();
    for _ in 0..32 {
        assert_eq!(
            e.phase_induction_predict(&frame(0.28, 0.72))
                .unwrap()
                .authority,
            Authority::Imagined
        );
    }
    assert_eq!(e.phase_native_learned_fingerprint(), fingerprint);
    assert_eq!(e.phase_induction_programs(), facts);
    assert_eq!(e.phase_vector_info().unwrap().active_prototypes, 0);
}
#[test]
fn learned_thresholds_edges_and_motor_strengths_control_prediction() {
    let mut e = acquired();
    let expected = e.phase_induction_predict(&frame(0.28, 0.72)).unwrap();
    let nodes = e.phase_induction_programs();
    let mut links = nodes
        .iter()
        .flat_map(|p| &p.nodes)
        .flat_map(|n| n.synapses.iter().copied())
        .collect::<Vec<_>>();
    links.sort_unstable();
    links.dedup();
    let mut saved = Vec::new();
    for &i in &links {
        saved.push((
            i,
            e.perturb_phase_native_synapse_for_control(i, 0.0, 0.0)
                .unwrap(),
        ));
    }
    assert!(e.phase_induction_predict(&frame(0.28, 0.72)).is_none());
    assert!(e.phase_vector_predict(&frame(0.28, 0.72)).is_none());
    for (i, s) in saved {
        e.restore_phase_native_synapse_for_control(i, s);
    }
    assert_eq!(
        e.phase_induction_predict(&frame(0.28, 0.72)).unwrap(),
        expected
    );
    // Select a query that actually crosses the perturbed root test.
    let crossing = vec![Some(0.72); 4];
    let crossing_expected = e.phase_induction_predict(&crossing).unwrap();
    let root = nodes[crossing_expected.action].nodes.first().unwrap();
    assert!(root.input.is_some());
    let index = root.synapses[0];
    let old = e
        .perturb_phase_native_synapse_for_control(index, 1.0, std::f32::consts::PI)
        .unwrap();
    assert_ne!(
        e.phase_induction_predict(&crossing),
        Some(crossing_expected.clone())
    );
    e.restore_phase_native_synapse_for_control(index, old);
    assert_eq!(
        e.phase_induction_predict(&crossing).unwrap(),
        crossing_expected
    );
}
#[test]
fn programs_execute_through_the_single_use_runtime_after_measurement() {
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    rt.set_model_learning_enabled(false);
    let before = rt.organism().phase_native_learned_fingerprint();
    rt.observe_external_partial(&[None; 4]).unwrap();
    let mapping = induction::roles(0);
    rt.step_partial(
        |_| Some(induction::safe()),
        |a| {
            assert_eq!(mapping[a], 2);
            Ok((frame(0.28, 0.72), 0.0))
        },
    )
    .unwrap();
    let proposal = rt.propose_unified().unwrap().unwrap();
    assert_eq!(proposal.mode, ReasoningMode::InducedProgramPrediction);
    assert!(!rt.goal_reached().unwrap());
    rt.step_partial(
        |_| Some(induction::safe()),
        |a| {
            assert_eq!(mapping[a], 1);
            Ok((vec![None; 4], 1.0))
        },
    )
    .unwrap();
    assert!(rt.goal_reached().unwrap());
    assert_eq!(rt.organism().phase_native_learned_fingerprint(), before);
}
#[test]
fn an_unobserved_truth_row_is_unsupported_even_when_two_explanations_fit() {
    let mut rt = ScientificRuntime::new(induction::fresh(4)).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let mapping = induction::roles(0);
    // OR and XOR agree on these three rows, but disagree on (high,high).
    for cycle in 0..128 {
        for bits in [0, 1, 2] {
            let x = induction::features(induction::Function::Two(6), bits, cycle % 4, 0, false);
            induction::teach(&mut rt, &x, usize::from(bits != 0), &mapping, None);
        }
    }
    assert!(rt
        .organism()
        .phase_induction_predict(&frame(0.72, 0.72))
        .is_none());
    assert!(rt
        .organism()
        .phase_induction_predict(&[None, Some(0.2), Some(0.2), Some(0.2)])
        .is_none());
    for x in [f32::NAN, f32::INFINITY, -0.01, 1.01] {
        assert!(rt
            .organism()
            .phase_induction_predict(&frame(x, 0.2))
            .is_none());
    }
}
#[test]
fn changed_facts_revise_programs_and_memory_stays_bounded() {
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let old = rt.organism().online_checkpoint_bytes().unwrap();
    let before: serde_json::Value = serde_json::from_slice(&old).unwrap();
    let old_programs = rt.organism().phase_induction_programs();
    let mapping = induction::roles(0);
    for cycle in 0..192 {
        for bits in 0..4 {
            let x = induction::features(induction::Function::Two(9), bits, cycle % 4, 0, false);
            induction::teach(
                &mut rt,
                &x,
                induction::Function::Two(9).outcome(bits),
                &mapping,
                None,
            );
        }
    }
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    let after: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        before["cells"].as_array().unwrap().len(),
        after["cells"].as_array().unwrap().len()
    );
    assert_eq!(
        before["synapses"].as_array().unwrap().len(),
        after["synapses"].as_array().unwrap().len()
    );
    let programs = rt.organism().phase_induction_programs();
    assert!(programs
        .iter()
        .all(|p| p.retained_facts <= 128 && p.nodes.len() <= 15));
    assert!(programs
        .iter()
        .zip(old_programs)
        .any(|(new, old)| new.generation > old.generation));
    let r = induction::evaluate(&bytes, induction::Function::Two(9), 0, None, 0);
    assert_eq!(r.correct, r.tasks);
}
#[test]
fn malformed_graph_and_old_version_restore_atomically_without_actuator_authority() {
    let e = acquired();
    let bytes = e.online_checkpoint_bytes().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["version"], 8);
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external_partial(&frame(0.28, 0.72)).unwrap();
    let fp = rt.organism().phase_native_learned_fingerprint();
    for kind in ["cycle", "version", "capacity"] {
        let mut bad = value.clone();
        match kind {
            "version" => bad["version"] = 7.into(),
            "capacity" => bad["vector"]["induction"]["config"]["facts_per_motor"] = 999.into(),
            _ => {
                let p = bad["vector"]["induction"]["programs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(|p| p["nodes"][0]["kind"] == "Branch")
                    .unwrap();
                let edge = bad["vector"]["induction"]["programs"][p]["nodes"][0]["edges"][0]
                    .as_u64()
                    .unwrap() as usize;
                bad["synapses"][edge]["to"] =
                    bad["vector"]["induction"]["programs"][p]["nodes"][0]["cell"].clone();
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
    rt.observe_external_partial(&frame(0.28, 0.72)).unwrap();
    let mut stop = induction::safe();
    stop.emergency_stop = true;
    assert!(matches!(
        rt.step_partial(|_| Some(stop), |_| panic!("blocked callback")),
        Ok(StepOutcome::Blocked(_))
    ));
    rt.restore_online_checkpoint(&bytes).unwrap();
    rt.observe_external_partial(&frame(0.28, 0.72)).unwrap();
    assert!(matches!(
        rt.step_partial(|_| Some(induction::safe()), |_| panic!("latched callback")),
        Ok(StepOutcome::Blocked(_))
    ));
}
#[test]
fn resource_checks_reject_mutation_and_the_host_has_no_program_menu() {
    let mut e = induction::fresh(4);
    let fp = e.phase_native_learned_fingerprint();
    assert!(!e.enable_phase_induction(PhaseInductionConfig::default()));
    assert_eq!(e.phase_native_learned_fingerprint(), fp);
    let mut cold = EvoPhase::new(aeterna_v1::EvoConfig {
        sensory_cells: 4,
        motor_cells: 3,
        dormant_cells: 3,
        ..Default::default()
    });
    cold.enable_phase_native_planning(aeterna_v1::carrier::PhaseNativeConfig::default());
    assert!(
        cold.enable_phase_native_online_learning(aeterna_v1::carrier::PhaseOnlineConfig {
            max_states: 1
        })
    );
    assert!(
        cold.enable_phase_vector_learning(aeterna_v1::carrier::PhaseVectorConfig {
            slots_per_motor: 1,
            ..Default::default()
        })
    );
    let before = cold.online_checkpoint_bytes().unwrap();
    assert!(!cold.enable_phase_induction(PhaseInductionConfig::default()));
    let mut invalid = PhaseInductionConfig::default();
    invalid.search_expansions = 0;
    assert!(!cold.enable_phase_induction(invalid));
    assert_eq!(cold.online_checkpoint_bytes().unwrap(), before);
    let source = include_str!("../src/phase_induction.rs");
    assert!(
        !source.contains("PhasePerceptProgram")
            && !source.contains("candidate_programs")
            && !source.contains("rule_templates")
    );
}
#[test]
fn the_first_counterexample_withdraws_a_program_before_replacement_is_supported() {
    let mut rt = ScientificRuntime::new(induction::fresh(4)).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let x = vec![0.2; 4];
    let mapping = induction::roles(0);
    for _ in 0..128 {
        induction::teach(&mut rt, &x, 0, &mapping, None);
    }
    let frame = x.iter().copied().map(Some).collect::<Vec<_>>();
    let old = rt.organism().phase_induction_predict(&frame).unwrap();
    assert_eq!(mapping[old.action], 0);
    rt.observe_external(&x).unwrap();
    rt.step_partial(
        |_| Some(induction::safe()),
        |a| {
            assert_eq!(a, old.action);
            Ok((vec![None; 4], 0.0))
        },
    )
    .unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external(&x).unwrap();
    assert!(rt.organism().phase_induction_predict(&frame).is_none());
    let program = &rt.organism().phase_induction_programs()[old.action];
    assert_eq!(program.retained_facts, 1);
    assert_eq!(program.future_checks, 0);
    assert!(program.discarded_conflicts > 0);
}
#[test]
fn predicate_threshold_is_acquired_from_data_below_the_old_boolean_midpoint() {
    let mut rt = ScientificRuntime::new(induction::fresh(4)).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let mapping = induction::roles(0);
    for i in 0..256 {
        let label = i % 2;
        let x = vec![if label == 0 { 0.05 } else { 0.45 }, 0.2, 0.2, 0.2];
        induction::teach(&mut rt, &x, label, &mapping, None);
    }
    for (input, label) in [(0.1, 0), (0.4, 1)] {
        let p = rt
            .organism()
            .phase_induction_predict(&[Some(input), Some(0.2), Some(0.2), Some(0.2)])
            .unwrap();
        assert_eq!(mapping[p.action], label);
    }
    assert!(rt
        .organism()
        .phase_induction_programs()
        .iter()
        .flat_map(|p| &p.nodes)
        .any(|n| n.input == Some(0) && n.threshold.is_some_and(|x| (x - 0.25).abs() < 1e-6)));
}
