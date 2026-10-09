use aeterna_v1::carrier::{
    PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig, PhaseRuleConfig, PhaseRuleLanguage,
};
use aeterna_v1::scientific_runtime::{ReasoningMode, RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{Authority, EvoConfig, EvoPhase, HumanProtectionEvidence};

const PERIOD: i32 = 257;
fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 as u32
    }
    fn state(&mut self, width: usize) -> Vec<i32> {
        (0..width)
            .map(|_| (self.next() % PERIOD as u32) as i32)
            .collect()
    }
}
#[derive(Clone)]
struct Law {
    terms: Vec<(usize, i32)>,
    bias: i32,
}
fn law(terms: &[(usize, i32)], bias: i32) -> Law {
    Law {
        terms: terms.to_vec(),
        bias,
    }
}
#[derive(Clone)]
struct World {
    state: Vec<i32>,
    laws: Vec<Vec<Law>>,
    masks: Vec<Vec<bool>>,
}
impl World {
    // Integer external evaluator: no equations, motor roles, masks or answers
    // are supplied to EvoPhase. Only permitted factual PRE/POST frames enter it.
    fn transform(&self, a: usize) -> Vec<i32> {
        self.laws[a]
            .iter()
            .map(|l| {
                (l.bias + l.terms.iter().map(|&(s, c)| c * self.state[s]).sum::<i32>())
                    .rem_euclid(PERIOD)
            })
            .collect()
    }
    fn full(&self) -> Vec<f32> {
        self.state
            .iter()
            .map(|&x| x as f32 / PERIOD as f32)
            .collect()
    }
    fn partial(&self, a: usize) -> Vec<Option<f32>> {
        self.full()
            .into_iter()
            .zip(&self.masks[a])
            .map(|(x, &v)| v.then_some(x))
            .collect()
    }
    fn act(&mut self, a: usize) {
        self.state = self.transform(a);
    }
}
fn acquired(world: &mut World, seed: u64, max: usize) -> Vec<u8> {
    let width = world.state.len();
    let motors = world.laws.len();
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: width,
        motor_cells: motors,
        dormant_cells: width * motors + 4,
        ..Default::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning_with_language(
        PhaseRuleConfig::default(),
        PhaseRuleLanguage::CircularAffine
    ));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    let mut rng = Rng(seed);
    for _ in 0..120 {
        if (0..motors).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed) {
            break;
        }
        world.state = rng.state(width);
        rt.observe_external(&world.full()).unwrap();
        rt.set_goal(&vec![0.1234567; width]).unwrap();
        assert!(matches!(
            rt.step(
                |_| Some(safe()),
                |a| {
                    world.act(a);
                    Ok(world.full())
                }
            )
            .unwrap(),
            StepOutcome::Executed { .. }
        ));
    }
    assert!((0..motors).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed));
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    assert!(evo.enable_phase_partial_observation(PhasePartialConfig {
        max_hypotheses: max,
        ..Default::default()
    }));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    for _ in 0..40 {
        if (0..motors).all(|a| rt.organism().phase_partial_mask_info(a).unwrap().confirmed) {
            break;
        }
        world.state = rng.state(width);
        rt.observe_external_partial(&world.full().into_iter().map(Some).collect::<Vec<_>>())
            .unwrap();
        let mut goal = vec![None; width];
        goal[0] = Some(0.1234567);
        rt.set_partial_goal(&goal).unwrap();
        assert!(matches!(
            rt.step_partial(
                |_| Some(safe()),
                |a| {
                    world.act(a);
                    Ok((world.partial(a), 0.0))
                }
            )
            .unwrap(),
            StepOutcome::Executed { .. }
        ));
    }
    assert!((0..motors).all(|a| rt.organism().phase_partial_mask_info(a).unwrap().confirmed));
    rt.organism().online_checkpoint_bytes().unwrap()
}
fn frozen(bytes: &[u8], inverse: bool) -> ScientificRuntime {
    let mut evo = EvoPhase::from_online_checkpoint(bytes).unwrap();
    assert!(evo.set_phase_inverse_inference(inverse));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    rt.set_model_learning_enabled(false);
    rt
}
fn benchmark_world(seed: u64) -> (World, [usize; 3], usize, usize) {
    let mut rng = Rng(seed);
    let mut p = [0, 1, 2];
    for i in (1..3).rev() {
        p.swap(i, rng.next() as usize % (i + 1));
    }
    let probe = (seed % 2) as usize;
    let finish = 1 - probe;
    let mut a = vec![law(&[], 0); 3];
    a[p[0]] = law(&[(p[0], 1), (p[1], 1)], (rng.next() % 257) as i32);
    a[p[1]] = law(&[(p[0], 1), (p[2], 1)], (rng.next() % 257) as i32);
    a[p[2]] = law(&[(p[1], 1)], (rng.next() % 257) as i32);
    let mut b = vec![law(&[], 0); 3];
    b[p[0]] = law(&[(p[2], 1)], (rng.next() % 257) as i32);
    b[p[1]] = law(&[(p[1], 1)], 0);
    b[p[2]] = law(&[(p[2], 1)], 0);
    let mut laws = vec![Vec::new(); 2];
    laws[probe] = a;
    laws[finish] = b;
    let mut masks = vec![vec![false; 3]; 2];
    masks[probe][p[0]] = true;
    masks[probe][p[1]] = true;
    masks[finish][p[0]] = true;
    (
        World {
            state: vec![0; 3],
            laws,
            masks,
        },
        p,
        probe,
        finish,
    )
}

#[test]
fn inverse_transfers_in_32_worlds_and_beats_forward_and_cyclic_with_two_actions() {
    let mut inverse_success = 0;
    let mut forward_success = 0;
    let mut cyclic_success = 0;
    for seed in 1..=32 {
        let (mut world, p, probe, finish) = benchmark_world(0xCAFE0000 + seed);
        let bytes = acquired(&mut world, 0xBADC0DE + seed, 32);
        let mut rng = Rng(0xA170000 + seed);
        for _ in 0..8 {
            // Exclude trivial goals and coincidences that let a blind action
            // reach the target before the discriminating probe.
            let (initial, goal) = loop {
                world.state = rng.state(3);
                let initial = world.state.clone();
                let blind = world.transform(finish)[p[0]];
                world.act(probe);
                let goal = world.transform(finish)[p[0]];
                let repeat = world.transform(probe)[p[0]];
                if initial[p[0]] != goal
                    && world.state[p[0]] != goal
                    && blind != goal
                    && repeat != goal
                {
                    break (initial, goal);
                }
            };
            for (enabled, counter) in [(true, &mut inverse_success), (false, &mut forward_success)]
            {
                let mut w = world.clone();
                w.state = initial.clone();
                let mut rt = frozen(&bytes, enabled);
                let fingerprint = rt.organism().phase_native_learned_fingerprint();
                let mut frame = vec![None; 3];
                frame[p[0]] = Some(w.full()[p[0]]);
                let mut target = vec![None; 3];
                target[p[0]] = Some(goal as f32 / PERIOD as f32);
                rt.observe_external_partial(&frame).unwrap();
                rt.set_partial_goal(&target).unwrap();
                for n in 0..2 {
                    if rt.goal_reached().unwrap() {
                        break;
                    }
                    let outcome = rt
                        .step_partial(
                            |proposal| {
                                if enabled {
                                    assert_eq!(
                                        proposal.action,
                                        if n == 0 { probe } else { finish }
                                    );
                                }
                                Some(safe())
                            },
                            |a| {
                                w.act(a);
                                Ok((w.partial(a), 0.0))
                            },
                        )
                        .unwrap();
                    assert!(matches!(
                        outcome,
                        StepOutcome::Executed { learned: false, .. }
                    ));
                    if enabled && n == 0 {
                        let belief = rt.organism().phase_partial_belief().unwrap();
                        let report = belief.inverse.unwrap();
                        assert_eq!(report.authority, Authority::Imagined);
                        assert_eq!(report.pre_hypotheses.len(), 1);
                        for (x, y) in report.pre_hypotheses[0].iter().zip(&initial) {
                            assert!((x.unwrap() - *y as f32 / PERIOD as f32).abs() < 0.0001);
                        }
                        assert_eq!(belief.factual[p[2]], None);
                        assert!(belief.hypotheses[0][p[2]].is_some());
                    }
                }
                *counter += usize::from(rt.goal_reached().unwrap());
                assert_eq!(
                    rt.organism().phase_native_learned_fingerprint(),
                    fingerprint
                );
                rt.restart_cognition().unwrap();
                assert!(rt.organism().phase_partial_belief().is_none());
                assert_eq!(
                    rt.organism().phase_native_learned_fingerprint(),
                    fingerprint
                );
            }
            let mut blind = world.clone();
            blind.state = initial;
            for a in 0..2 {
                blind.act(a);
                if blind.masks[a][p[0]] && blind.state[p[0]] == goal {
                    cyclic_success += 1;
                    break;
                }
            }
        }
    }
    assert_eq!(inverse_success, 256);
    assert!(forward_success < inverse_success);
    assert!(cyclic_success < inverse_success);
    println!("worlds=32 tasks=256 action_budget=2 inverse={inverse_success} forward={forward_success} cyclic={cyclic_success}");
}

fn gain_world(gain: i32) -> World {
    World {
        state: vec![0, 0],
        laws: vec![
            vec![law(&[(1, gain)], 17), law(&[(1, 1)], 0)],
            vec![law(&[(1, 1)], 31), law(&[(1, 1)], 0)],
        ],
        masks: vec![vec![true, false]; 2],
    }
}
#[test]
fn every_modular_root_survives_until_a_discriminating_measurement() {
    for gain in [2, -2, 3, -3] {
        let mut world = gain_world(gain);
        let bytes = acquired(&mut world, 0xC0DE + gain.unsigned_abs() as u64, 32);
        let mut rt = frozen(&bytes, true);
        let fingerprint = rt.organism().phase_native_learned_fingerprint();
        world.state = vec![29, 181];
        rt.observe_external_partial(&[Some(29.0 / 257.0), None])
            .unwrap();
        rt.set_partial_goal(&[Some(212.0 / 257.0), None]).unwrap();
        rt.step_partial(
            |p| {
                assert_eq!(p.action, 0);
                Some(safe())
            },
            |a| {
                world.act(a);
                Ok((world.partial(a), 0.0))
            },
        )
        .unwrap();
        let belief = rt.organism().phase_partial_belief().unwrap();
        let report = belief.inverse.unwrap();
        assert_eq!(report.pre_hypotheses.len(), gain.unsigned_abs() as usize);
        assert!(report
            .pre_hypotheses
            .iter()
            .any(|r| (r[1].unwrap() - 181.0 / 257.0).abs() < 0.0001));
        assert_eq!(belief.hypotheses.len(), gain.unsigned_abs() as usize);
        assert_eq!(belief.factual[1], None);
        rt.set_partial_goal(&[None, Some(181.0 / 257.0)]).unwrap();
        assert!(
            !rt.goal_reached().unwrap(),
            "inferred value became factual success"
        );
        rt.set_partial_goal(&[Some(212.0 / 257.0), None]).unwrap();
        rt.step_partial(
            |p| {
                assert_eq!(p.action, 1);
                assert_eq!(p.mode, ReasoningMode::PartialInformationGathering);
                Some(safe())
            },
            |a| {
                world.act(a);
                Ok((world.partial(a), 0.0))
            },
        )
        .unwrap();
        assert!(rt.goal_reached().unwrap());
        assert_eq!(
            rt.organism()
                .phase_partial_belief()
                .unwrap()
                .hypotheses
                .len(),
            1
        );
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
    }
}

#[test]
fn equations_propagate_backwards_across_outputs_but_continuous_relations_stay_unknown() {
    let mut chain = World {
        state: vec![0; 3],
        laws: vec![vec![
            law(&[(1, 1), (2, 1)], 17),
            law(&[(0, 1), (1, 1)], 31),
            law(&[(2, 1)], 0),
        ]],
        masks: vec![vec![true, true, false]],
    };
    let bytes = acquired(&mut chain, 0xD00D, 32);
    let mut rt = frozen(&bytes, true);
    chain.state = vec![29, 181, 87];
    rt.observe_external_partial(&[Some(29.0 / 257.0), None, None])
        .unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None, None]).unwrap();
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            chain.act(a);
            Ok((chain.partial(a), 0.0))
        },
    )
    .unwrap();
    let b = rt.organism().phase_partial_belief().unwrap();
    let pre = &b.inverse.unwrap().pre_hypotheses[0];
    assert!((pre[1].unwrap() - 181.0 / 257.0).abs() < 0.0001);
    assert!((pre[2].unwrap() - 87.0 / 257.0).abs() < 0.0001);
    assert!(b.hypotheses[0][2].is_some());
    let mut relation = World {
        state: vec![0; 2],
        laws: vec![vec![law(&[(0, 1), (1, 1)], 17), law(&[(0, 1)], 0)]],
        masks: vec![vec![true, false]],
    };
    let bytes = acquired(&mut relation, 0xD00D, 32);
    let mut rt = frozen(&bytes, true);
    relation.state = vec![29, 181];
    rt.observe_external_partial(&[None, None]).unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            relation.act(a);
            Ok((relation.partial(a), 0.0))
        },
    )
    .unwrap();
    let b = rt.organism().phase_partial_belief().unwrap();
    assert_eq!(b.inverse.unwrap().pre_hypotheses, vec![vec![None, None]]);
    assert_eq!(b.hypotheses[0][1], None);
}

#[test]
fn capacity_widens_all_roots_and_inferred_values_never_train_numeric_rules() {
    let mut world = gain_world(3);
    let bytes = acquired(&mut world, 0xD00D, 1);
    let mut evo = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    assert!(evo.set_phase_inverse_inference(true));
    let counts = (0..2)
        .map(|a| evo.phase_rule_action_info(a).unwrap())
        .collect::<Vec<_>>();
    let mut rt = ScientificRuntime::new(evo).unwrap();
    world.state = vec![29, 181];
    rt.observe_external_partial(&[Some(29.0 / 257.0), None])
        .unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            world.act(a);
            Ok((world.partial(a), 0.0))
        },
    )
    .unwrap();
    let b = rt.organism().phase_partial_belief().unwrap();
    assert!(b.widened);
    assert!(b.inverse.unwrap().widened);
    assert_eq!(b.hypotheses, vec![vec![Some(world.full()[0]), None]]);
    assert_eq!(
        (0..2)
            .map(|a| rt.organism().phase_rule_action_info(a).unwrap())
            .collect::<Vec<_>>(),
        counts
    );
}

#[test]
fn physical_pair_parameters_control_inverse_and_exact_restore_recovers_it() {
    let mut world = World {
        state: vec![0; 2],
        laws: vec![vec![law(&[(0, 1), (1, 1)], 17), law(&[(1, 1)], 0)]],
        masks: vec![vec![true, false]],
    };
    let bytes = acquired(&mut world, 0xBEEF, 32);
    let connections = EvoPhase::from_online_checkpoint(&bytes)
        .unwrap()
        .phase_rule_action_info(0)
        .unwrap()
        .synapses;
    for &connection in &connections[..2] {
        let mut evo = EvoPhase::from_online_checkpoint(&bytes).unwrap();
        assert!(evo.set_phase_inverse_inference(true));
        let saved = evo
            .perturb_phase_native_synapse_for_control(connection, 0.0, 0.0)
            .unwrap();
        let mut rt = ScientificRuntime::new(evo).unwrap();
        rt.set_model_learning_enabled(false);
        rt.observe_external_partial(&[Some(29.0 / 257.0), None])
            .unwrap();
        rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
        assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
        for delta in [std::f32::consts::PI, 0.0] {
            let mut evo = EvoPhase::from_online_checkpoint(&bytes).unwrap();
            evo.restore_phase_native_synapse_for_control(connection, saved.clone());
            if delta != 0.0 {
                evo.perturb_phase_native_synapse_for_control(connection, 1.0, delta)
                    .unwrap();
            }
            assert!(evo.set_phase_inverse_inference(true));
            let mut rt = ScientificRuntime::new(evo).unwrap();
            rt.set_model_learning_enabled(false);
            let fingerprint = rt.organism().phase_native_learned_fingerprint();
            world.state = vec![29, 181];
            rt.observe_external_partial(&[Some(29.0 / 257.0), None])
                .unwrap();
            rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
            rt.step_partial(
                |_| Some(safe()),
                |a| {
                    world.act(a);
                    Ok((world.partial(a), 0.0))
                },
            )
            .unwrap();
            let inferred = rt
                .organism()
                .phase_partial_belief()
                .unwrap()
                .inverse
                .unwrap()
                .pre_hypotheses[0][1]
                .unwrap();
            assert_eq!((inferred - 181.0 / 257.0).abs() < 0.0001, delta == 0.0);
            assert_eq!(
                rt.organism().phase_native_learned_fingerprint(),
                fingerprint
            );
        }
    }
}

#[test]
fn zero_is_a_real_root_and_contradiction_recovers_only_actual_fields() {
    let mut world = gain_world(2);
    let bytes = acquired(&mut world, 0xABCD, 32);
    let mut rt = frozen(&bytes, true);
    world.state = vec![29, 0];
    rt.observe_external_partial(&[Some(29.0 / 257.0), None])
        .unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            world.act(a);
            Ok((world.partial(a), 0.0))
        },
    )
    .unwrap();
    assert!(rt
        .organism()
        .phase_partial_belief()
        .unwrap()
        .inverse
        .unwrap()
        .pre_hypotheses
        .iter()
        .any(|r| r[1] == Some(0.0)));
    rt = frozen(&bytes, true);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.state = vec![29, 181];
    rt.observe_external_partial(&world.full().into_iter().map(Some).collect::<Vec<_>>())
        .unwrap();
    rt.set_partial_goal(&[Some(world.transform(0)[0] as f32 / 257.0), None])
        .unwrap();
    world.laws[0][0].bias += 71;
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            world.act(a);
            Ok((world.partial(a), 0.0))
        },
    )
    .unwrap();
    let b = rt.organism().phase_partial_belief().unwrap();
    assert!(b.inverse.unwrap().contradicted);
    assert_eq!(b.hypotheses, vec![world.partial(0)]);
    assert_eq!(b.invalidated_rules, vec![0]);
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn inverse_configuration_persists_but_episode_goals_and_bad_post_do_not_supply_evidence() {
    let mut world = gain_world(2);
    let bytes = acquired(&mut world, 0xABCD, 32);
    let mut rt = frozen(&bytes, true);
    let knowledge = rt.organism().online_checkpoint_bytes().unwrap();
    assert!(EvoPhase::from_online_checkpoint(&knowledge)
        .unwrap()
        .phase_inverse_inference_enabled());
    let mut old: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    old["version"] = 4.into();
    old["partial"]
        .as_object_mut()
        .unwrap()
        .remove("inverse_enabled");
    assert!(
        !EvoPhase::from_online_checkpoint(&serde_json::to_vec(&old).unwrap())
            .unwrap()
            .phase_inverse_inference_enabled()
    );
    let mut corrupt: serde_json::Value = serde_json::from_slice(&knowledge).unwrap();
    corrupt["version"] = 4.into();
    assert!(EvoPhase::from_online_checkpoint(&serde_json::to_vec(&corrupt).unwrap()).is_err());
    world.state = vec![29, 181];
    rt.observe_external_partial(&[Some(29.0 / 257.0), None])
        .unwrap();
    rt.set_partial_goal(&[None, Some(181.0 / 257.0)]).unwrap();
    assert_eq!(
        rt.organism().phase_partial_belief().unwrap().hypotheses,
        vec![vec![Some(29.0 / 257.0), None]]
    );
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    let before = rt.organism().phase_partial_belief();
    assert!(matches!(
        rt.step_partial(|_| None, |_| panic!("blocked executor called"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
    assert_eq!(rt.organism().phase_partial_belief(), before);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    assert!(matches!(
        rt.step_partial(|_| Some(safe()), |_| Ok((vec![Some(f32::NAN), None], 0.0)))
            .unwrap(),
        StepOutcome::ExecutionFault(_)
    ));
    assert_eq!(rt.organism().phase_partial_belief(), before);
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    rt.restore_online_checkpoint(&knowledge).unwrap();
    assert!(rt.organism().phase_partial_belief().is_none());
    assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
}
