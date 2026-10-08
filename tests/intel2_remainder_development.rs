// Already exposed pack only. No new independent INTEL verdict.
// The included fixture acquires A/B/C in one organism; no target tuition.
include!("intel2_refined_state_controls.rs");

fn remainder_d_success(action: usize, x: usize, y: usize, truth: bool) -> bool {
    if truth { action == x } else { action == y }
}

#[test]
fn remainder_evaluator_checks_all_six_motors() {
    for x in 0..6 {
        for y in 0..6 {
            if x == y { continue; }
            for truth in [false, true] {
                let accepted = (0..6).filter(|&a| remainder_d_success(a, x, y, truth))
                    .collect::<Vec<_>>();
                assert_eq!(accepted, vec![if truth { x } else { y }]);
            }
        }
    }
    println!("REMAINDER_EVALUATOR six_motor_truth_table=PASS");
}

fn remainder_target_records(rt: &ScientificRuntime) -> String {
    format!("{:?}|{:?}|{:?}|{:?}", rt.organism().phase_native_circuits(),
        rt.organism().phase_native_context_witnesses(),
        rt.organism().phase_native_perceptual_witnesses(),
        rt.organism().phase_native_composition_witnesses())
}

#[test]
#[ignore = "explicit burned-pack development diagnostic, not fresh qualification"]
fn remainder_development_one_organism() {
    let seed = 37_687_243_350u64;
    let mut rng = Rng::new(seed);
    let mut roles = (0usize..24).collect::<Vec<_>>();
    shuffle(&mut rng, &mut roles);
    let a_states: [usize; 5] = roles[0..5].try_into().unwrap();
    let b_states: [usize; 5] = roles[5..10].try_into().unwrap();
    let d_states: [usize; 3] = roles[16..19].try_into().unwrap();
    let e5: [usize; 5] = roles[19..24].try_into().unwrap();
    let e6 = [e5[0], e5[1], e5[2], e5[3], e5[4], a_states[4]];
    let mut motors = [[0usize, 1, 2, 3, 4, 5]; 5];
    for row in &mut motors { shuffle(&mut rng, row); }
    let [ma, mb, mc, md, me] = motors;
    let op_xor = rng.next() & 1 == 1;
    let bins = if rng.next() & 1 == 0 { [0.20f32, 0.40] } else { [0.15f32, 0.35] };
    let (mut rt, l1, c_states, fixture_mc) = refined_context_fixture();
    assert_eq!(fixture_mc, mc);
    let original_meta = rt.organism().phase_native_meta_weights().unwrap();
    let mut monitor = Monitor::default();
    println!("REMAINDER_START burned_seed={seed} cognitive_baseline=924a566b4d3f1125fa67e73bc1f68c4ba53bdbd3 A_B_acquisition_reuse=PASS D={d_states:?}/{md:?} op_xor={op_xor} bins={bins:?} E={e6:?}/{me:?}");
    let mut c_score = [0usize; 2];
    let mut memoryless = 0usize;
    for i in 0..32 {
        let side = i % 2;
        let goal = arrive_at_refined_junction(&mut rt, &l1, c_states, side, (4+i)%6);
        let before = rt.organism().phase_native_learned_fingerprint();
        let p = rt.propose_unified().expect("C proposal").expect("C not terminal");
        c_score[side] += usize::from(p.action == mc[side]);
        assert_eq!(before, rt.organism().phase_native_learned_fingerprint());
        let current = foundation::scene(&l1, c_states[3], (4+i)%6);
        let mut probe = rt.organism().clone();
        memoryless += usize::from(probe.plan_phase_native_abstract_goal(&current, &goal, None)
            .map(|p| p.first_action) == Some(mc[side]));
    }
    let c_pass = c_score.iter().sum::<usize>() >= 28
        && c_score.iter().all(|&n| n >= 13) && memoryless <= 20;
    println!("REMAINDER_C sides={c_score:?}/[16,16] memoryless={memoryless}/32 pass={c_pass}");
    let before_restart = rt.organism().phase_native_learned_fingerprint();
    rt.restart_cognition().unwrap();
    assert!(rt.organism().current_real().is_none());
    assert_eq!(rt.organism().phase_native_learned_fingerprint(), before_restart);
    assert!(matches!(rt.propose_unified(), Err(RuntimeError::FreshObservationRequired)));
    for side in 0..2 {
        arrive_at_refined_junction(&mut rt, &l1, c_states, side, 4+side);
        assert_eq!(rt.propose_unified().unwrap().unwrap().action, mc[side]);
    }
    println!("REMAINDER_RESTART preserved=true fresh_sensing_required=true C_reuse=2/2");

    // Simulated software protection boundary, not a real physical device.
    let calls = Cell::new(0usize);
    let safety_fp = rt.organism().phase_native_learned_fingerprint();
    let harmless_post = foundation::scene(&l1, c_states[4], 0);
    let high = rt.step_unified(
        |_| Some(HumanProtectionEvidence { predicted_harm_probability: 0.5, ..safe() }),
        |_| { calls.set(calls.get()+1); Ok((harmless_post.clone(), 1.0)) }).unwrap();
    assert!(matches!(high, StepOutcome::Blocked(_)));
    let missing = rt.step_unified(|_| None,
        |_| { calls.set(calls.get()+1); Ok((harmless_post.clone(), 1.0)) }).unwrap();
    assert!(matches!(missing, StepOutcome::Blocked(_)));
    let emergency = rt.step_unified(
        |_| Some(HumanProtectionEvidence { emergency_stop: true, ..safe() }),
        |_| { calls.set(calls.get()+1); Ok((harmless_post.clone(), 1.0)) }).unwrap();
    assert!(matches!(emergency, StepOutcome::Blocked(_)));
    assert_eq!(calls.get(), 0);
    assert_eq!(rt.organism().phase_native_learned_fingerprint(), safety_fp);
    rt.restart_cognition().unwrap();
    assert!(rt.emergency_latched());
    rt.observe_external(&foundation::scene(&l1, c_states[1], 4)).unwrap();
    let latched = rt.step_unified(|_| Some(safe()),
        |_| { calls.set(calls.get()+1); Ok((harmless_post.clone(), 1.0)) }).unwrap();
    assert!(matches!(latched, StepOutcome::Blocked(_)));
    assert_eq!(calls.get(), 0);
    rt.external_operator_reset();
    assert!(!rt.emergency_latched());
    rt.set_model_learning_enabled(true);
    println!("REMAINDER_PROTECTION high=true missing=true restart_latch=true callbacks=0");

    // D: strict two-motor world, unchanged generic learner and promotion gates.
    let d_base = foundation::scene(&l1, d_states[0], 0);
    let d_goal = foundation::scene(&l1, d_states[1], 1);
    let d_base_cell = rt.organism().phase_native_abstract_state(&d_base).unwrap().cell;
    let safe_pool = (0..d_base.len()).filter(|&p|
        (0..6).all(|layout| foundation::scene(&l1, d_states[0], layout)[p] < 0.5))
        .collect::<Vec<_>>();
    assert!(safe_pool.len() >= 8);
    let selected = safe_pool.iter().rev().take(8).copied().collect::<Vec<_>>();
    let train_pos = [[selected[0], selected[1]], [selected[2], selected[3]]];
    let score_pos = [[selected[4], selected[5]], [selected[6], selected[7]]];
    for positions in train_pos.into_iter().chain(score_pos) {
        for layout in 0..6 {
            for combo in 0..4 {
                let s = weak_scene(&l1, d_states[0], layout, combo&2 != 0, combo&1 != 0, bins, positions);
                assert_eq!(rt.organism().phase_native_abstract_state(&s).unwrap().cell, d_base_cell);
            }
        }
    }
    println!("REMAINDER_D_GEOMETRY train={train_pos:?} score={score_pos:?} inherited_state_unchanged=true");
    let counts = if op_xor { [48usize,8,8,16] } else { [24usize,24,24,8] };
    let mut schedule = Vec::new();
    for (combo, count) in counts.into_iter().enumerate() {
        schedule.extend(std::iter::repeat(combo).take(count));
    }
    shuffle(&mut rng, &mut schedule);
    let [x, y] = [md[0], md[1]];
    let mut d_actions = [[0usize; 6]; 4];
    let mut d_stop = None;
    let mut d_steps = 0usize;
    let mut d_first_promotion = None;
    for i in 0..1200 {
        let promoted = rt.organism().phase_native_composition_witnesses().iter()
            .any(|w| w.promoted && w.base_cell == d_base_cell);
        if promoted && d_first_promotion.is_none() {
            d_first_promotion = Some(i);
        }
        // Development diagnostic: continue real self-selected action learning
        // after a program promotes, up to the original 1200-action ceiling.
        let combo = schedule[i % schedule.len()];
        let [a, b] = [combo&2 != 0, combo&1 != 0];
        let truth = if op_xor { a ^ b } else { a && b };
        let current = weak_scene(&l1, d_states[0], i%4, a, b, bins, train_pos[i%2]);
        rt.observe_external(&current).unwrap();
        rt.set_goal(&d_goal).unwrap();
        let result = rt.step_unified(|_| Some(safe()), |action| {
            d_steps += 1;
            d_actions[combo][action] += 1;
            let correct = remainder_d_success(action, x, y, truth);
            Ok((foundation::scene(&l1, if correct {d_states[1]} else {d_states[2]}, i%4),
                if correct {1.0} else {0.0}))
        });
        if !matches!(result, Ok(StepOutcome::Executed{..})) {
            d_stop = Some(format!("trial={i} combo={combo} result={result:?}"));
            break;
        }
    }
    let d_witnesses = rt.organism().phase_native_composition_witnesses().into_iter()
        .filter(|w| w.base_cell == d_base_cell).collect::<Vec<_>>();
    let d_promoted = d_witnesses.iter().any(|w| w.promoted);
    println!("REMAINDER_D_FIRST_PROMOTED {:?}",d_first_promotion);
    println!("REMAINDER_D_TRAIN actions={d_steps} histogram={d_actions:?} stop={d_stop:?} witnesses={d_witnesses:?}");
    rt.set_model_learning_enabled(false);
    let mut d_correct = 0usize;
    let mut d_unavailable = 0usize;
    let mut d_combo_scores = [0usize; 4];
    let mut labels = Vec::new();
    for i in 0..32 {
        let combo = i%4;
        let [a, b] = [combo&2 != 0, combo&1 != 0];
        let truth = if op_xor {a^b} else {a&&b};
        let current = weak_scene(&l1, d_states[0], 4+i%2, a, b, bins, score_pos[i%2]);
        rt.observe_external(&current).unwrap();
        rt.set_goal(&d_goal).unwrap();
        let before = rt.organism().phase_native_learned_fingerprint();
        match rt.propose_unified() {
            Ok(Some(p)) => {
                let ok = remainder_d_success(p.action, x, y, truth);
                d_correct += usize::from(ok);
                d_combo_scores[combo] += usize::from(ok);
                if i < 4 { println!("REMAINDER_D_READ combo={combo} action={} expected={}", p.action, if truth{x}else{y}); }
            },
            other => {
                d_unavailable += 1;
                if i < 4 { println!("REMAINDER_D_READ combo={combo} unavailable={other:?}"); }
            },
        }
        assert_eq!(before, rt.organism().phase_native_learned_fingerprint());
        labels.push((a,b,truth));
    }
    let single_ceiling = (0..2).flat_map(|which| [false,true].map(move |polarity| (which,polarity)))
        .map(|(which,polarity)| labels.iter().filter(|(a,b,t)|
            ((if which==0 {*a} else {*b}) == polarity) == *t).count()).max().unwrap();
    let d_pass = d_promoted && d_correct >= 28 && d_stop.is_none()
        && single_ceiling <= if op_xor {20} else {24};
    println!("REMAINDER_D promoted={d_promoted} score={d_correct}/32 unavailable={d_unavailable} combo_scores={d_combo_scores:?} single_oracle_ceiling={single_ceiling}/32 pass={d_pass}");
    rt.set_model_learning_enabled(true);

    // E continues after D failure, on the SAME resulting organism.
    let mut we = WorldE::new(e6, me);
    set_world(&mut rt, &l1, we.state, e6[3], 2);
    let mut e_actions = 0usize;
    let mut e_resets = 0usize;
    let mut e_histogram = [0usize; 6];
    let mut first_changed_action = None;
    let mut recovery = None;
    let mut e_stop = None;
    let mut stable_acquired_before_change = false;
    let mut e_goals = 0usize;
    let mut pre_change_record = None;
    for _ in 0..2400 {
        if e_actions >= 1200 || recovery.is_some() { break; }
        if we.state == e6[3] || we.state == e6[4] {
            we.state = e6[0];
            rt.observe_external(&foundation::scene(&l1, we.state, (e_actions+1)%6)).unwrap();
            e_resets += 1;
            continue;
        }
        let before_state = we.state;
        let already_changed = we.changed;
        let layout = e_actions%6;
        let before_revisions = rt.organism().phase_native_circuits().iter()
            .map(|c| (c.revision,c.counterexamples.len() as u64)).fold((0u64,0u64), |a,b| (a.0+b.0,a.1+b.1));
        let result = rt.step_unified(|_| Some(safe()), |action| {
            e_actions += 1;
            e_histogram[action] += 1;
            let (next,value) = we.step(action);
            if already_changed && before_state == e6[0] && action == me[0]
                && first_changed_action.is_none() {
                first_changed_action = Some(e_actions);
                pre_change_record = Some(before_revisions);
            }
            if !already_changed && before_state == e6[5] && action == me[5] && next == e6[3] {
                stable_acquired_before_change = true;
            }
            if next == e6[3] {
                e_goals += 1;
                if let Some(start) = first_changed_action {
                    if recovery.is_none() { recovery = Some(e_actions-start); }
                }
            }
            Ok((foundation::scene(&l1, next, layout),value))
        });
        if !matches!(result, Ok(StepOutcome::Executed{..})) {
            e_stop = Some(format!("at_action={e_actions} state={} result={result:?}",we.state));
            break;
        }
    }
    let after_revisions = rt.organism().phase_native_circuits().iter()
        .map(|c| (c.revision,c.counterexamples.len() as u64)).fold((0u64,0u64), |a,b| (a.0+b.0,a.1+b.1));
    let revision_evidence = pre_change_record.map(|b| after_revisions.0>b.0 && after_revisions.1>b.1).unwrap_or(false);
    rt.set_model_learning_enabled(false);
    // Transport the environment, not only its rendered observation.
    we.state = e6[5];
    set_world(&mut rt, &l1, we.state, e6[3], 5);
    let mut stable_arrived = false;
    for i in 0..6 {
        if we.state == e6[3] { stable_arrived=true; break; }
        let result = rt.step_unified(|_| Some(safe()), |action| {
            let (next,value)=we.step(action);
            Ok((foundation::scene(&l1,next,(5+i)%6),value))
        });
        if !matches!(result, Ok(StepOutcome::Executed{..})) { break; }
    }
    stable_arrived |= we.state == e6[3];
    let e_pass = we.changed && first_changed_action.is_some()
        && recovery.map(|n| n<=16).unwrap_or(false) && revision_evidence
        && stable_acquired_before_change && stable_arrived && e_stop.is_none();
    let drift_status = if first_changed_action.is_none() {"NOT_EXERCISED"}
        else if e_pass {"PASS"} else {"FAIL"};
    println!("REMAINDER_E actions={e_actions} resets={e_resets} goals={e_goals} shortcut_successes={} changed={} first_changed={first_changed_action:?} recovery={recovery:?} revision_evidence={revision_evidence} stable_preacquired={stable_acquired_before_change} stable_reuse={stable_arrived} stop={e_stop:?} histogram={e_histogram:?} status={drift_status}",we.shortcut_successes,we.changed);
    let before_retention = remainder_target_records(&rt);
    let mut fa = WorldA::new(a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]);
    let final_a=run_a(&mut rt,&l1,&mut monitor,&mut fa,6,5,None);
    let mut fb=WorldB::new(b_states,[mb[0],mb[1],mb[2]]);
    let final_b=run_b_goal(&mut rt,&l1,&mut monitor,&mut fb,false,8,4);
    let targets_unchanged=before_retention==remainder_target_records(&rt);
    let meta_unchanged=rt.organism().phase_native_meta_weights().unwrap()==original_meta;
    let legacy=rt.organism().planning_transition_count();
    let table=rt.organism().composite_concepts().len();
    println!("REMAINDER_RETENTION A={final_a} A_cost={} B={final_b} B_cost={} target_records_unchanged={targets_unchanged} meta_unchanged={meta_unchanged} legacy={legacy} table={table}",fa.calls,fb.calls);
    let pass=c_pass&&d_pass&&e_pass&&final_a&&final_b&&targets_unchanged&&meta_unchanged&&legacy==0&&table==0;
    println!("REMAINDER_SUMMARY C={c_pass} D={d_pass} E={e_pass} retention={}/2 verdict={}",usize::from(final_a)+usize::from(final_b),if pass {"DEVELOPMENT_CHAIN_PASS"} else {"DEVELOPMENT_CHAIN_FAIL"});
    assert!(pass,"development chain incomplete; individual stage evidence is printed above");
}
