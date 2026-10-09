#[path = "../examples/support/induction.rs"]
mod world;
use aeterna_v1::carrier::{
    PhaseInductionConfig, PhaseNativeConfig, PhaseOnlineConfig, PhasePrimitiveConfig,
    PhaseVectorConfig,
};
use aeterna_v1::scientific_runtime::ScientificRuntime;
use aeterna_v1::{EvoConfig, EvoPhase};
fn base() -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: 3,
        motor_cells: 3,
        dormant_cells: 128,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_vector_learning(PhaseVectorConfig {
        slots_per_motor: 1,
        exploration_observations: 64,
        ..Default::default()
    }));
    assert!(e.enable_phase_induction(PhaseInductionConfig::default()));
    e
}
fn fresh() -> EvoPhase {
    let mut e = base();
    assert!(e.enable_phase_primitives(PhasePrimitiveConfig {
        capacity: 4,
        ..Default::default()
    }));
    e
}
fn input(bits: usize, heldout: bool) -> Vec<f32> {
    (0..3)
        .map(|j| {
            if bits >> j & 1 == 0 {
                if heldout {
                    0.28
                } else {
                    0.2
                }
            } else if heldout {
                0.72
            } else {
                0.8
            }
        })
        .collect()
}
fn acquired() -> EvoPhase {
    let mut rt = ScientificRuntime::new(fresh()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    for _ in 0..32 {
        for bits in 0..8 {
            world::teach(
                &mut rt,
                &input(bits, false),
                (bits & 3).count_ones() as usize % 2,
                &world::roles(0),
                None,
            );
        }
    }
    assert!(!rt.organism().phase_primitives().is_empty());
    EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap()
}
#[test]
fn experience_creates_callable_operations_and_later_programs_reuse_them() {
    let mut rt = ScientificRuntime::new(acquired()).unwrap();
    let allocation: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    for _ in 0..128 {
        for bits in 0..8 {
            world::teach(
                &mut rt,
                &input(bits, false),
                bits.count_ones() as usize % 2,
                &world::roles(0),
                None,
            );
        }
    }
    let mut e = EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
        .unwrap();
    let library = e.phase_primitives();
    let after: serde_json::Value =
        serde_json::from_slice(&e.online_checkpoint_bytes().unwrap()).unwrap();
    for field in ["cells", "synapses"] {
        assert_eq!(
            allocation[field].as_array().unwrap().len(),
            after[field].as_array().unwrap().len()
        );
    }
    assert!(e
        .phase_induction_programs()
        .iter()
        .all(|p| p.retained_facts <= 128));
    let programs = e.phase_induction_programs();
    println!(
        "acquired_operations={} construction_uses={} inputs={:?}",
        library.len(),
        library.iter().map(|p| p.construction_uses).sum::<u64>(),
        programs
            .iter()
            .map(|p| &p.primitive_inputs)
            .collect::<Vec<_>>()
    );
    assert!(library.iter().any(|p| p.construction_uses > 0));
    assert!(programs.iter().any(|p| !p.primitive_inputs.is_empty()));
    let fingerprint = e.phase_native_learned_fingerprint();
    let queries = (0usize..8)
        .map(|bits| input(bits, true).into_iter().map(Some).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let expected = queries
        .iter()
        .map(|q| e.phase_induction_predict(q).unwrap())
        .collect::<Vec<_>>();
    for (bits, p) in expected.iter().enumerate() {
        assert_eq!(world::roles(0)[p.action], bits.count_ones() as usize % 2);
    }
    let before = e.phase_constructor_info();
    for op in &library {
        for q in &queries {
            e.phase_primitive_value(op.output_cell, q);
        }
    }
    assert_eq!(e.phase_native_learned_fingerprint(), fingerprint);
    assert_eq!(e.phase_constructor_info(), before);
    let mut saved = Vec::new();
    for i in library.iter().flat_map(|op| op.synapses.iter().copied()) {
        saved.push((
            i,
            e.perturb_phase_native_synapse_for_control(i, 0.0, 0.0)
                .unwrap(),
        ));
    }
    assert!(queries
        .iter()
        .zip(&expected)
        .any(|(q, p)| e.phase_induction_predict(q).as_ref() != Some(p)));
    assert!(library.iter().all(|op| e
        .phase_primitive_value(op.output_cell, &queries[0])
        .is_none()));
    for (i, s) in saved {
        e.restore_phase_native_synapse_for_control(i, s);
    }
    for (q, p) in queries.iter().zip(&expected) {
        assert_eq!(e.phase_induction_predict(q).as_ref(), Some(p));
    }
    let bytes = e.online_checkpoint_bytes().unwrap();
    let restored = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    assert_eq!(restored.phase_native_learned_fingerprint(), fingerprint);
    assert_eq!(restored.phase_primitives(), library);
}

#[test]
fn primitive_pool_admission_is_atomic_before_any_tuition() {
    let mut e = base();
    let before = e.online_checkpoint_bytes().unwrap();
    for config in [
        PhasePrimitiveConfig {
            capacity: 9,
            ..Default::default()
        },
        PhasePrimitiveConfig::default(),
        PhasePrimitiveConfig {
            capacity: 1,
            policy_learning_rate: f32::NAN,
        },
    ] {
        assert!(!e.enable_phase_primitives(config));
        assert_eq!(before, e.online_checkpoint_bytes().unwrap());
    }
}
#[test]
fn cold_library_bounds_invalid_calls_and_checkpoint_versions() {
    let mut e = fresh();
    assert!(e.phase_primitives().is_empty());
    let original = e.online_checkpoint_bytes().unwrap();
    assert!(!e.enable_phase_primitives(PhasePrimitiveConfig::default()));
    assert_eq!(e.online_checkpoint_bytes().unwrap(), original);
    let e = acquired();
    let bytes = e.online_checkpoint_bytes().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["version"], 9);
    let mut old = value.clone();
    old["version"] = 8.into();
    assert!(EvoPhase::from_online_checkpoint(&serde_json::to_vec(&old).unwrap()).is_err());
    // Turn a primitive test into a self-call; cyclic callable definitions cannot load.
    let mut cyclic = value.clone();
    let node = &cyclic["vector"]["induction"]["primitives"]["operations"][0]["program"]["nodes"][0];
    let link = node["test"].as_u64().unwrap() as usize;
    let cell = node["cell"].clone();
    cyclic["synapses"][link]["from"] = cell;
    assert!(EvoPhase::from_online_checkpoint(&serde_json::to_vec(&cyclic).unwrap()).is_err());
    let mut live = ScientificRuntime::new(e).unwrap();
    live.set_outcome_goal(1.0).unwrap();
    live.set_model_learning_enabled(false);
    let before = live.organism().online_checkpoint_bytes().unwrap();
    assert!(live
        .restore_online_checkpoint(&serde_json::to_vec(&cyclic).unwrap())
        .is_err());
    assert_eq!(live.organism().online_checkpoint_bytes().unwrap(), before);
    live.observe_external(&input(1, false)).unwrap();
    let result = live
        .step_partial(
            |_| Some(world::safe()),
            |a| {
                Ok((
                    input(1, false).into_iter().map(Some).collect(),
                    f32::from(world::roles(0)[a] == 1),
                ))
            },
        )
        .unwrap();
    assert!(matches!(
        result,
        aeterna_v1::scientific_runtime::StepOutcome::Executed { learned: false, .. }
    ));
    assert_eq!(live.organism().online_checkpoint_bytes().unwrap(), before);
    let mut bad = value;
    bad["vector"]["induction"]["primitives"]["config"]["capacity"] = 9.into();
    assert!(EvoPhase::from_online_checkpoint(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut impossible: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    impossible["vector"]["induction"]["primitives"]["operations"][0]["published_sequence"] =
        1.into();
    assert!(EvoPhase::from_online_checkpoint(&serde_json::to_vec(&impossible).unwrap()).is_err());
}
#[test]
fn learned_constructor_preference_is_physical_and_breaks_equal_loss_ties() {
    let mut rt = ScientificRuntime::new(fresh()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    // Two identical observed channels are equally predictive. There is no
    // supplied feature choice; subsequent actual results acquire a preference.
    for _ in 0..32 {
        for bit in 0..2 {
            let x = if bit == 0 { 0.2 } else { 0.8 };
            world::teach(&mut rt, &[x, x, 0.2], bit, &world::roles(0), None);
        }
    }
    let e = EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
        .unwrap();
    let info = e.phase_constructor_info().unwrap();
    assert!(info.factual_updates > 0);
    assert!(info.input_priorities[0].2 > info.input_priorities[1].2);
    // Persist a real physical intervention, then provide contradictory facts
    // that force reconstruction. The learned priority of channel zero is cut.
    let mut cut = EvoPhase::from_online_checkpoint(&e.online_checkpoint_bytes().unwrap()).unwrap();
    cut.perturb_phase_native_synapse_for_control(info.input_priorities[0].1, 0.0, 0.0)
        .unwrap();
    let mut left = ScientificRuntime::new(e).unwrap();
    let mut right = ScientificRuntime::new(cut).unwrap();
    left.set_outcome_goal(1.0).unwrap();
    right.set_outcome_goal(1.0).unwrap();
    for _ in 0..32 {
        for bit in 0..2 {
            let x = if bit == 0 { 0.2 } else { 0.8 };
            world::teach(&mut left, &[x, x, 0.2], 1 - bit, &world::roles(0), None);
            world::teach(&mut right, &[x, x, 0.2], 1 - bit, &world::roles(0), None);
        }
    }
    let roots = |rt: &ScientificRuntime| {
        rt.organism()
            .phase_induction_programs()
            .into_iter()
            .filter_map(|p| p.nodes.first().and_then(|n| n.input))
            .collect::<Vec<_>>()
    };
    assert_ne!(roots(&left), roots(&right));
}
