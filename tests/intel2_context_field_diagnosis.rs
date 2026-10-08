// Burned-pack diagnostic only. No new intelligence verdict or source repair.
include!("intel2_unified_worlds.rs");

#[test]
fn context_field_diagnosis_after_edge_cost_correction() {
    let seed: u64 = 37_687_243_350;
    let (evo, l1) = foundation::build24();
    let meta = meta_checkpoint();
    let mut rng = Rng::new(seed);
    let mut states = (0usize..24).collect::<Vec<_>>();
    shuffle(&mut rng, &mut states);
    let a_states: [usize; 5] = states[0..5].try_into().unwrap();
    let b_states: [usize; 5] = states[5..10].try_into().unwrap();
    let c_states: [usize; 6] = states[10..16].try_into().unwrap();
    let mut ma = [0usize, 1, 2, 3, 4, 5]; shuffle(&mut rng, &mut ma);
    let mut mb = [0usize, 1, 2, 3, 4, 5]; shuffle(&mut rng, &mut mb);
    let mut mc = [0usize, 1, 2, 3, 4, 5]; shuffle(&mut rng, &mut mc);
    let mut rt = ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_context_refinement());
    assert!(rt.enable_perceptual_refinement());
    assert!(rt.enable_compositional_refinement());
    assert!(rt.enable_unified_cognition(meta, PhaseHypothesisEcologyConfig {
        learning_rate: 0.35, dormancy_threshold: 0.05,
        learning_enabled: true, phase_learning_enabled: true,
    }));
    let mut monitor = Monitor::default();
    let mut wa = WorldA::new(a_states, [ma[0], ma[1], ma[2], ma[3], ma[4]]);
    assert!(run_a(&mut rt, &l1, &mut monitor, &mut wa, 48, 0, None));
    rt.set_model_learning_enabled(false);
    let mut wa_reuse = WorldA::new(a_states, [ma[0], ma[1], ma[2], ma[3], ma[4]]);
    assert!(run_a(&mut rt, &l1, &mut monitor, &mut wa_reuse, 6, 4, Some(a_states[1])));
    rt.set_model_learning_enabled(true);
    let mut wb = WorldB::new(b_states, [mb[0], mb[1], mb[2]]);
    assert!(run_b_goal(&mut rt, &l1, &mut monitor, &mut wb, false, 64, 1));
    assert!(run_b_goal(&mut rt, &l1, &mut monitor, &mut wb, true, 12, 2));
    let mut wc = WorldC::new(c_states, [mc[0], mc[1]], seed ^ 0xC0FFEE);
    set_world(&mut rt, &l1, wc.state, c_states[4], 0);
    let goal = foundation::scene(&l1, c_states[4], 1);
    let base = rt.organism().phase_native_abstract_state(
        &foundation::scene(&l1, c_states[3], 0)).unwrap().cell;
    for k in 0..1800usize {
        if wc.trials >= 72 && rt.organism().phase_native_context_witnesses()
            .iter().any(|w| w.promoted && w.base_cell == base) { break; }
        if wc.state == c_states[4] || wc.state == c_states[5] {
            wc.state = c_states[0];
            rt.observe_external(&foundation::scene(&l1, wc.state, (k+1)%6)).unwrap();
            rt.set_goal(&goal).unwrap();
            continue;
        }
        let outcome = step_u(&mut rt, &mut monitor, |a| {
            let (next, value) = wc.step(a);
            (foundation::scene(&l1, next, k%4), value)
        });
        assert!(outcome.is_ok(), "diagnostic training lost support: {outcome:?}");
    }
    let witnesses = rt.organism().phase_native_context_witnesses();
    let target = witnesses.iter().find(|w| w.promoted && w.base_cell == base)
        .expect("edge-cost diagnostic must acquire the same context");
    println!("CONTEXT_FIELD_TARGET {target:?}");
    for &cell in &target.state_cells {
        for circuit in rt.organism().phase_native_circuits() {
            let aff = rt.organism().phase_native_synapse(circuit.afferent_synapse).unwrap();
            if aff.from != cell { continue; }
            let motor = rt.organism().phase_native_synapse(circuit.motor_synapse).unwrap();
            let successor = rt.organism().phase_native_synapse(circuit.successor_synapse).unwrap();
            println!("CONTEXT_FIELD_CIRCUIT cell={} action={} support={} successor={} weight={} aff_weight={}",
                cell, motor.to - rt.organism().config().sensory_cells,
                circuit.support, successor.to, successor.weight, aff.weight);
        }
    }
    rt.set_model_learning_enabled(false);
    for side in 0..2usize {
        let pred = c_states[1+side];
        rt.observe_external(&foundation::scene(&l1, pred, 4+side)).unwrap();
        rt.set_goal(&goal).unwrap();
        let outcome = rt.step_unified(|_| Some(safe()),
            |_| Ok((foundation::scene(&l1, c_states[3], 4+side), 0.0))).unwrap();
        assert!(matches!(outcome, StepOutcome::Executed{..}));
        let before = rt.organism().phase_native_learned_fingerprint();
        let mut frozen_context = rt.organism().clone();
        let context_frozen = frozen_context.phase_native_context_action(&goal);
        let proposals = rt.organism().collect_phase_native_unified_proposals(&goal);
        let selected = rt.organism().choose_phase_native_unified_proposal(&proposals);
        let mut learning_context = rt.organism().clone();
        learning_context.set_planning_learning_enabled(true);
        let context_learning = learning_context.phase_native_context_action(&goal);
        let learning_proposals = learning_context.collect_phase_native_unified_proposals(&goal);
        let learning_selected = learning_context.choose_phase_native_unified_proposal(&learning_proposals);
        println!("CONTEXT_FIELD_SIDE side={} expected={} context_frozen={:?} frozen_selected={:?} context_learning={:?} learning_selected={:?}",
            side, mc[side], context_frozen, selected, context_learning, learning_selected);
        println!("CONTEXT_FIELD_FROZEN_PROPOSALS side={side} {proposals:?}");
        println!("CONTEXT_FIELD_LEARNING_PROPOSALS side={side} {learning_proposals:?}");
        assert_eq!(before, rt.organism().phase_native_learned_fingerprint(), "diagnostic probes mutated original cognition");
    }
}
