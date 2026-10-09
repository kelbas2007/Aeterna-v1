// Open development witness. These externally generated tasks deliberately
// control the required relation; this is NOT spontaneous curriculum discovery.
// The engine is never given the new input pair or any function name.
include!("acquired_primitives.rs");

fn xor_of(bits: usize, pair: [usize; 2]) -> usize {
    ((bits >> pair[0]) ^ (bits >> pair[1])) & 1
}
fn changed_values(bits: usize, variant: usize) -> Vec<f32> {
    let low = [0.24, 0.28, 0.31][variant];
    let high = [0.76, 0.72, 0.69][variant];
    (0..3)
        .map(|i| if bits >> i & 1 == 0 { low } else { high })
        .collect()
}
fn positive_source(e: &EvoPhase) -> usize {
    e.phase_primitives()
        .iter()
        .find(|op| {
            (0..8).all(|bits| {
                e.phase_primitive_value(
                    op.output_cell,
                    &input(bits, false).into_iter().map(Some).collect::<Vec<_>>(),
                )
                .is_some_and(|v| usize::from(v >= 0.5) == xor_of(bits, [0, 1]))
            })
        })
        .expect("an acquired nonconstant two-input definition")
        .output_cell
}
fn definitions(e: &EvoPhase, roots: &[usize]) -> Vec<String> {
    e.phase_primitives()
        .iter()
        .filter(|op| roots.contains(&op.output_cell))
        .map(|op| {
            format!(
                "{}:{:?}",
                op.output_cell,
                op.synapses
                    .iter()
                    .map(|&i| e.phase_native_synapse(i))
                    .collect::<Vec<_>>()
            )
        })
        .collect()
}

#[test]
fn same_acquired_definition_accepts_new_arguments_without_retraining() {
    let mut e = acquired();
    let roots = e
        .phase_primitives()
        .iter()
        .map(|op| op.output_cell)
        .collect::<Vec<_>>();
    let root = positive_source(&e);
    assert_eq!(e.phase_primitive_argument_sources(root), Some(vec![0, 1]));
    let original = definitions(&e, &roots);
    let before = e.online_checkpoint_bytes().unwrap();
    let mut passed = 0usize;
    let mut unbound = 0usize;
    for pair in [[0, 2], [1, 2], [2, 0], [2, 1]] {
        for variant in 0..3 {
            for bits in 0..8 {
                let frame = changed_values(bits, variant)
                    .into_iter()
                    .map(Some)
                    .collect::<Vec<_>>();
                let expected = xor_of(bits, pair);
                let bound = e
                    .phase_primitive_bound_value(root, &frame, &pair)
                    .expect("same physical definition at a new argument mapping");
                assert_eq!(usize::from(bound >= 0.5), expected);
                passed += 1;
                unbound += usize::from(
                    e.phase_primitive_value(root, &frame)
                        .is_some_and(|v| usize::from(v >= 0.5) == expected),
                );
            }
        }
    }
    assert_eq!(passed, 96);
    assert_eq!(unbound, 48);
    assert_eq!(
        e.online_checkpoint_bytes().unwrap(),
        before,
        "calls must not train a definition or save inferred answers"
    );
    assert_eq!(definitions(&e, &roots), original);
    assert!(e
        .phase_primitive_bound_value(root, &[Some(0.2); 3], &[0])
        .is_none());
    assert!(e
        .phase_primitive_bound_value(root, &[Some(0.2); 3], &[0, 3])
        .is_none());
    assert!(e
        .phase_primitive_bound_value(root, &[None; 3], &[1, 2])
        .is_none());
    let op = e
        .phase_primitives()
        .into_iter()
        .find(|o| o.output_cell == root)
        .unwrap();
    let link = op.synapses[0];
    let saved = e
        .perturb_phase_native_synapse_for_control(link, 0.0, 0.0)
        .unwrap();
    assert!(
        e.phase_primitive_bound_value(root, &[Some(0.2); 3], &[1, 2])
            .is_none(),
        "without the original physical definition the call must fail"
    );
    e.restore_phase_native_synapse_for_control(link, saved);
    let restored = EvoPhase::from_online_checkpoint(&e.online_checkpoint_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.phase_primitive_bound_value(root, &[Some(0.2); 3], &[1, 2]),
        e.phase_primitive_bound_value(root, &[Some(0.2); 3], &[1, 2])
    );
    println!("PRIMITIVE_ARGUMENT_MECHANISM correct=96/96 old_unbound=48/96 same_definition=true no_query_learning=true lesion=true restore=true");
}

#[test]
fn factual_experience_selects_rebinding_and_runtime_reuses_original_operations() {
    let mut e = acquired();
    let roots = e
        .phase_primitives()
        .iter()
        .map(|p| p.output_cell)
        .collect::<Vec<_>>();
    let original = definitions(&e, &roots);
    assert!(e.set_phase_primitive_argument_transfer(true));
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let mut exact = 0usize;
    let mut executed = 0usize;
    for (task, pair) in [[1usize, 2usize], [0usize, 2usize]].into_iter().enumerate() {
        let mut training_actions = 0;
        for _ in 0..16 {
            for bits in 0..8 {
                // Only the external environment sees the target pair. The
                // organism receives selected actions and their actual reward.
                training_actions += world::teach(
                    &mut rt,
                    &input(bits, false),
                    xor_of(bits, pair),
                    &world::roles(0),
                    None,
                );
            }
        }
        let bindings = rt.organism().phase_primitive_argument_bindings();
        println!(
            "PRIMITIVE_ARGUMENT_BINDING_DIAGNOSTIC task={} calls={:?}",
            task, bindings
        );
        assert_eq!(
            bindings.len(),
            2,
            "both outcome actions require factual bindings"
        );
        for (_, root, args, fit, checks) in &bindings {
            assert!(
                roots.contains(root),
                "must reuse a SOURCE operation, not a target copy"
            );
            let mut found = args.clone();
            found.sort_unstable();
            let mut expected = pair.to_vec();
            expected.sort_unstable();
            assert_eq!(
                found, expected,
                "the host never supplied this argument mapping"
            );
            assert!(*fit >= 4 && *checks >= 8);
        }
        assert_eq!(
            definitions(rt.organism(), &roots),
            original,
            "all source definitions must remain byte-for-byte physically unchanged"
        );
        let saved = rt.organism().online_checkpoint_bytes().unwrap();
        let recovered = EvoPhase::from_online_checkpoint(&saved).unwrap();
        assert_eq!(recovered.phase_primitive_argument_bindings(), bindings);
        let mut frozen = ScientificRuntime::new(recovered).unwrap();
        frozen.set_outcome_goal(1.0).unwrap();
        frozen.set_model_learning_enabled(false);
        for variant in 0..3 {
            for bits in 0..8 {
                let x = changed_values(bits, variant);
                let frame = x.iter().copied().map(Some).collect::<Vec<_>>();
                let before = frozen.organism().online_checkpoint_bytes().unwrap();
                let prediction = frozen
                    .organism()
                    .phase_primitive_argument_prediction(&frame)
                    .expect("factually validated parameterized call");
                assert!(roots.contains(&prediction.root_cell));
                assert_eq!(world::roles(0)[prediction.action], xor_of(bits, pair));
                assert_eq!(
                    frozen.organism().phase_induction_predict(&frame),
                    Some(prediction)
                );
                assert_eq!(
                    frozen.organism().online_checkpoint_bytes().unwrap(),
                    before,
                    "binding discovery and prediction are read-only"
                );
                exact += 1;
                executed +=
                    world::teach(&mut frozen, &x, xor_of(bits, pair), &world::roles(0), None);
            }
        }
        assert_eq!(definitions(rt.organism(), &roots), original);
        println!("PRIMITIVE_ARGUMENT_TASK task={} correct=24/24 factual_training_actions={} original_calls={:?}",
            task,training_actions,bindings);
    }
    assert_eq!(exact, 48);
    println!("PRIMITIVE_ARGUMENT_RUNTIME correct=48/48 protected_actions={} one_training_lifetime=true arguments_not_supplied=true original_definitions_unchanged=true checkpoint=true",executed);
}

#[test]
fn factual_argument_calls_have_synapse_addresses_and_lesion_controls() {
    let mut e = acquired();
    let roots = e
        .phase_primitives()
        .iter()
        .map(|p| p.output_cell)
        .collect::<Vec<_>>();
    let definitions_before = definitions(&e, &roots);
    assert!(e.set_phase_primitive_argument_transfer(true));
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    // The binder sees only ordinary executed action/outcome. The target is
    // held solely by the external task simulator, not passed to the model.
    for _ in 0..16 {
        for bits in 0..8 {
            world::teach(
                &mut rt,
                &input(bits, false),
                xor_of(bits, [1, 2]),
                &world::roles(0),
                None,
            );
        }
    }
    let e = rt.organism();
    let bindings = e.phase_primitive_argument_bindings();
    assert_eq!(
        bindings.len(),
        2,
        "two factual outcomes must have physical calls"
    );
    assert_eq!(e.phase_native_argument_binding_count(), 2);
    assert_eq!(definitions(e, &roots), definitions_before);
    let (action, root, args, _, _) = bindings[0].clone();
    assert_eq!(args, vec![1, 2]);
    assert!(roots.contains(&root));
    let addresses = e.phase_native_argument_binding_synapses(action).unwrap();
    assert_eq!(addresses.len(), 2);
    for (i, &link) in addresses.iter().enumerate() {
        let syn = e.phase_native_synapse(link).unwrap();
        assert_eq!(syn.to, args[i]);
        assert!(syn.weight > 0.5 && syn.confidence >= 0.5);
    }
    let bytes = e.online_checkpoint_bytes().unwrap();
    let mut resumed = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    assert_eq!(resumed.phase_primitive_argument_bindings(), bindings);
    let before_query = resumed.online_checkpoint_bytes().unwrap();
    let query = changed_values(0b110, 1)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    let normal = resumed.phase_primitive_argument_prediction(&query);
    assert!(normal.is_some());
    assert_eq!(resumed.online_checkpoint_bytes().unwrap(), before_query);
    let link = addresses[0];
    let saved = resumed
        .perturb_phase_native_synapse_for_control(link, 0.0, 0.0)
        .expect("physical argument synapse must be addressed");
    let remaining = resumed.phase_primitive_argument_bindings();
    assert_eq!(remaining.len(), 1, "lesion must disable THIS physical call");
    assert!(remaining.iter().all(|b| b.0 != action));
    resumed.restore_phase_native_synapse_for_control(link, saved);
    assert_eq!(resumed.phase_primitive_argument_bindings(), bindings);
    assert_eq!(resumed.phase_primitive_argument_prediction(&query), normal);
    println!("PRIMITIVE_PHYSICAL_BINDINGS calls=2 source_definitions_unchanged=true query_no_fit=true checkpoint=true lesion=true restore=true");
}
