// Development controls on the burned diagnostic lifetime, not fresh scoring.
include!("intel2_unified_worlds.rs");

fn refined_context_fixture() -> (ScientificRuntime, [[usize; 2]; 8], [usize; 6], [usize; 6]) {
    let seed: u64 = 37_687_243_350;
    let (evo, l1) = foundation::build24();
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
    assert!(rt.enable_unified_cognition(meta_checkpoint(), PhaseHypothesisEcologyConfig {
        learning_rate: 0.35, dormancy_threshold: 0.05,
        learning_enabled: true, phase_learning_enabled: true,
    }));
    let original_meta = rt.organism().phase_native_meta_weights().unwrap();
    let mut monitor = Monitor::default();
    let mut wa = WorldA::new(a_states, [ma[0], ma[1], ma[2], ma[3], ma[4]]);
    assert!(run_a(&mut rt, &l1, &mut monitor, &mut wa, 48, 0, None));
    rt.set_model_learning_enabled(false);
    let mut reuse = WorldA::new(a_states, [ma[0], ma[1], ma[2], ma[3], ma[4]]);
    assert!(run_a(&mut rt, &l1, &mut monitor, &mut reuse, 6, 4, Some(a_states[1])));
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
        step_u(&mut rt, &mut monitor, |a| {
            let (next, value) = wc.step(a);
            (foundation::scene(&l1, next, k%4), value)
        }).expect("continued diagnostic lifetime must stay operational");
    }
    assert!(rt.organism().phase_native_context_witnesses().iter()
        .any(|w| w.promoted && w.base_cell == base));
    assert_eq!(original_meta, rt.organism().phase_native_meta_weights().unwrap());
    rt.set_model_learning_enabled(false);
    (rt, l1, c_states, mc)
}

fn arrive_at_refined_junction(rt: &mut ScientificRuntime, l1: &[[usize; 2]; 8],
    states: [usize; 6], side: usize, layout: usize) -> Vec<f32> {
    let goal = foundation::scene(l1, states[4], 1);
    rt.observe_external(&foundation::scene(l1, states[1+side], layout)).unwrap();
    rt.set_goal(&goal).unwrap();
    let outcome = rt.step_unified(|_| Some(safe()),
        |_| Ok((foundation::scene(l1, states[3], layout), 0.0))).unwrap();
    assert!(matches!(outcome, StepOutcome::Executed{..}));
    goal
}

#[test]
fn refined_state_controls_preserve_context_through_unified_competition() {
    let (mut rt, l1, states, motors) = refined_context_fixture();
    let mut scores = [0usize; 2];
    for i in 0..32usize {
        let side = i%2;
        let goal = arrive_at_refined_junction(&mut rt, &l1, states, side, (4+i)%6);
        let before = rt.organism().phase_native_learned_fingerprint();
        let mut proposals = rt.organism().collect_phase_native_unified_proposals(&goal);
        let chosen = rt.organism().choose_phase_native_unified_proposal(&proposals)
            .expect("unified refined-state readout").action;
        scores[side] += usize::from(chosen == motors[side]);
        proposals.reverse();
        assert_eq!(rt.organism().choose_phase_native_unified_proposal(&proposals).unwrap().action, chosen);
        assert_eq!(before, rt.organism().phase_native_learned_fingerprint());
    }
    println!("REFINED_STATE_SCORE sides={scores:?}/[16,16]");
    assert_eq!(scores, [16,16]);

    for side in 0..2usize {
        let goal = arrive_at_refined_junction(&mut rt, &l1, states, side, 4+side);
        let current_base = rt.organism().phase_native_abstract_state(
            &foundation::scene(&l1, states[3], 0)).unwrap().cell;
        let pred = rt.organism().phase_native_abstract_state(
            &foundation::scene(&l1, states[1+side], 0)).unwrap().cell;
        let target = rt.organism().phase_native_context_witnesses().into_iter()
            .find(|w| w.promoted && w.base_cell == current_base).unwrap();
        let internal_side = target.predecessor_cells.iter().position(|&p| p == pred).unwrap();
        let link = target.input_synapses[internal_side][1];
        let mut controlled = rt.organism().clone();
        let before = controlled.phase_native_learned_fingerprint();
        let saved = controlled.perturb_phase_native_synapse_for_control(link, 0.0, 0.0).unwrap();
        assert!(controlled.collect_phase_native_unified_proposals(&goal).is_empty(),
            "weight lesion must not expose contradictory parent fallback");
        controlled.restore_phase_native_synapse_for_control(link, saved.clone());
        assert_eq!(before, controlled.phase_native_learned_fingerprint());
        let proposals = controlled.collect_phase_native_unified_proposals(&goal);
        assert_eq!(controlled.choose_phase_native_unified_proposal(&proposals).unwrap().action, motors[side]);
        controlled.perturb_phase_native_synapse_for_control(link, 1.0, std::f32::consts::PI).unwrap();
        assert!(controlled.collect_phase_native_unified_proposals(&goal).is_empty());
        controlled.restore_phase_native_synapse_for_control(link, saved);
        assert_eq!(before, controlled.phase_native_learned_fingerprint());
        controlled.clear_phase_native_context_history();
        assert!(controlled.collect_phase_native_unified_proposals(&goal).is_empty(),
            "missing actual predecessor must not be guessed");
        println!("REFINED_STATE_CAUSAL side={side} lesion=true phase=true restore=true missing_history=true");
    }

    rt.restart_cognition().unwrap();
    rt.observe_external(&foundation::scene(&l1, states[3], 4)).unwrap();
    let goal = foundation::scene(&l1, states[4], 1);
    rt.set_goal(&goal).unwrap();
    assert_eq!(rt.propose_unified().unwrap_err(), RuntimeError::NoSupportedAction);
    for side in 0..2usize {
        arrive_at_refined_junction(&mut rt, &l1, states, side, 4+side);
        assert_eq!(rt.propose_unified().unwrap().unwrap().action, motors[side]);
    }
    println!("REFINED_STATE_RESTART preserved=true fresh_predecessor_required=true");
}
