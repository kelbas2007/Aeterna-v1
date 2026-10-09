use aeterna_v1::carrier::{
    PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig, PhaseRuleConfig, PhaseRuleFamily,
    PhaseRuleLanguage,
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

fn fresh(width: usize, motors: usize, language: PhaseRuleLanguage, depth: usize) -> EvoPhase {
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: width,
        motor_cells: motors,
        dormant_cells: width * motors + 4,
        ..Default::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig {
        horizon: 16,
        ..Default::default()
    });
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning_with_language(
        PhaseRuleConfig {
            planning_depth: depth,
            ..Default::default()
        },
        language,
    ));
    evo
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

// Only the external evaluator owns coefficients and offsets. Dynamics use
// independent integer arithmetic, not the learner's templates/phase readout.
#[derive(Clone)]
struct Law {
    terms: Vec<(usize, i32)>,
    offset: i32,
}
impl Law {
    fn new(terms: &[(usize, i32)], offset: i32) -> Self {
        let mut terms = terms.to_vec();
        terms.sort();
        Self { terms, offset }
    }
    fn evaluate(&self, input: &[i32]) -> i32 {
        (self.offset + self.terms.iter().map(|&(i, a)| a * input[i]).sum::<i32>())
            .rem_euclid(PERIOD)
    }
}
struct World {
    laws: Vec<Vec<Law>>,
    state: Vec<i32>,
    history: Vec<Vec<i32>>,
}
impl World {
    fn generated(seed: u64, width: usize, motors: usize) -> Self {
        let mut rng = Rng(seed);
        let mut laws = Vec::new();
        for a in 0..motors {
            let mut outputs = Vec::new();
            for j in 0..width {
                let i = rng.next() as usize % width;
                let k = (i + 1 + rng.next() as usize % (width - 1)) % width;
                let sign = if rng.next() % 2 == 0 { 1 } else { -1 };
                let terms = match (a * width + j) % 7 {
                    0 => vec![],
                    1 => vec![(i, sign * 2)],
                    2 => vec![(i, sign * 3)],
                    3 => vec![(i, 1), (k, 1)],
                    4 => vec![(i, 1), (k, -1)],
                    5 => vec![(i, sign)],
                    _ => vec![(i, -1), (k, -1)],
                };
                outputs.push(Law::new(&terms, (rng.next() % PERIOD as u32) as i32));
            }
            laws.push(outputs);
        }
        for i in (1..motors).rev() {
            laws.swap(i, rng.next() as usize % (i + 1));
        }
        Self {
            laws,
            state: rng.state(width),
            history: Vec::new(),
        }
    }
    fn raster(state: &[i32]) -> Vec<f32> {
        state.iter().map(|&x| x as f32 / PERIOD as f32).collect()
    }
    fn observe(&self) -> Vec<f32> {
        Self::raster(&self.state)
    }
    fn partial(&self) -> Vec<Option<f32>> {
        vec![Some(self.observe()[0]), None]
    }
    fn transformed(&self, input: &[i32], action: usize) -> Vec<i32> {
        self.laws[action]
            .iter()
            .map(|law| law.evaluate(input))
            .collect()
    }
    fn act(&mut self, action: usize) -> Vec<f32> {
        self.history.push(self.state.clone());
        self.state = self.transformed(&self.state, action);
        self.history.push(self.state.clone());
        self.observe()
    }
}

fn mixed_world() -> World {
    World {
        laws: vec![
            vec![Law::new(&[(0, 1)], 17), Law::new(&[(1, 1)], 31)],
            vec![Law::new(&[(0, 1), (1, 1)], 11), Law::new(&[(1, 3)], 7)],
            vec![Law::new(&[], 71), Law::new(&[(0, 1), (1, -1)], 13)],
            vec![Law::new(&[(1, 2)], 5), Law::new(&[(0, -2)], 9)],
        ],
        state: vec![7, 31],
        history: Vec::new(),
    }
}
fn ready(rt: &ScientificRuntime) -> bool {
    (0..rt.organism().config().motor_cells)
        .all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed)
}
fn learn_episodes(rt: &mut ScientificRuntime, world: &mut World, seed: u64) -> usize {
    let mut rng = Rng(seed);
    rt.set_goal(&vec![0.1234567; world.state.len()]).unwrap();
    for n in 0..120 {
        if ready(rt) {
            return n;
        }
        // Fresh real starting states are supplied by the external environment;
        // the organism chooses the motor. This is not continuous cold discovery.
        world.state = rng.state(world.state.len());
        rt.observe_external(&world.observe()).unwrap();
        let outcome = rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
        assert!(matches!(outcome, StepOutcome::Executed { .. }));
    }
    panic!("episode acquisition budget exhausted");
}
fn assert_forecast(rt: &ScientificRuntime, world: &World, input: &[i32], action: usize) {
    let forecast = rt
        .organism()
        .phase_rule_predict(action, &World::raster(input))
        .unwrap();
    assert_eq!(forecast.authority, Authority::Imagined);
    let expected = World::raster(&world.transformed(input, action));
    for (&a, &b) in forecast.sensory.iter().zip(&expected) {
        let delta = (a - b).abs();
        assert!(
            delta.min(1.0 - delta) < 0.0001,
            "{forecast:?} vs {expected:?}"
        );
    }
}

#[test]
fn expanded_families_transfer_in_32_worlds_to_256_frozen_unseen_goals() {
    let mut acquisitions = 0;
    let mut goals = 0;
    let mut families = [0; 5];
    for seed in 1..=32 {
        let width = 3 + seed as usize % 3;
        let motors = 3 + seed as usize % 2;
        let mut world = World::generated(seed * 0x9E3779B9, width, motors);
        let mut rt =
            ScientificRuntime::new(fresh(width, motors, PhaseRuleLanguage::CircularAffine, 4))
                .unwrap();
        acquisitions += learn_episodes(&mut rt, &mut world, seed ^ 0xABCDEF);
        for a in 0..motors {
            for (formula, law) in rt
                .organism()
                .phase_rule_formulas(a)
                .unwrap()
                .iter()
                .zip(&world.laws[a])
            {
                assert_eq!(
                    formula
                        .terms
                        .iter()
                        .map(|t| (t.source, t.coefficient as i32))
                        .collect::<Vec<_>>(),
                    law.terms
                );
                families[match formula.family {
                    PhaseRuleFamily::Constant => 0,
                    PhaseRuleFamily::SingleChannel => 1,
                    PhaseRuleFamily::IntegerGain => 2,
                    PhaseRuleFamily::TwoChannelSum => 3,
                    PhaseRuleFamily::TwoChannelDifference => 4,
                }] += 1;
            }
        }
        let acquisition = world.history.clone();
        let evo =
            EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
                .unwrap();
        assert_eq!(
            evo.phase_rule_language(),
            Some(PhaseRuleLanguage::CircularAffine)
        );
        assert!(evo.current_real().is_none());
        let mut rt = ScientificRuntime::new(evo).unwrap();
        rt.set_model_learning_enabled(false);
        let fingerprint = rt.organism().phase_native_learned_fingerprint();
        let mut rng = Rng(seed ^ 0x12345678);
        for _ in 0..8 {
            let target = (0..1024)
                .find_map(|_| {
                    world.state = rng.state(width);
                    let mut target = world.state.clone();
                    for _ in 0..3 {
                        target = world.transformed(&target, rng.next() as usize % motors);
                    }
                    (!acquisition.contains(&world.state)
                        && !acquisition.contains(&target)
                        && target != world.state)
                        .then_some(target)
                })
                .expect("no unseen nontrivial goal");
            for action in 0..motors {
                assert_forecast(&rt, &world, &world.state, action);
            }
            rt.observe_external(&world.observe()).unwrap();
            rt.set_goal(&World::raster(&target)).unwrap();
            for n in 0..=3 {
                if rt.goal_reached().unwrap() {
                    break;
                }
                assert!(n < 3, "frozen affine goal exceeded budget");
                assert!(matches!(
                    rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap(),
                    StepOutcome::Executed { learned: false, .. }
                ));
            }
            assert!(rt.goal_reached().unwrap());
            assert_eq!(
                rt.organism().phase_native_learned_fingerprint(),
                fingerprint
            );
            goals += 1;
        }
        assert_eq!(rt.organism().phase_native_receptor_count(), 0);
    }
    assert_eq!(goals, 256);
    assert!(families.iter().all(|&n| n > 0));
    println!("affine_worlds=32 heldout_goals={goals} acquisition_actions={acquisitions} family_counts={families:?}");
}

#[test]
fn correlated_channels_remain_ambiguous_until_a_real_discriminating_experiment() {
    let mut world = World {
        laws: vec![vec![
            Law::new(&[(0, 1), (1, 1)], 0),
            Law::new(&[(0, 1), (1, -1)], 0),
        ]],
        state: vec![0, 0],
        history: vec![],
    };
    let mut rt = ScientificRuntime::new(fresh(2, 1, PhaseRuleLanguage::CircularAffine, 1)).unwrap();
    rt.set_goal(&[0.1234567; 2]).unwrap();
    for x in [23, 71, 149] {
        world.state = vec![x, x];
        rt.observe_external(&world.observe()).unwrap();
        rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
    }
    let info = rt.organism().phase_rule_action_info(0).unwrap();
    assert_eq!(info.distinct_support, 3);
    assert!(info.candidate_counts.iter().all(|&n| n > 1));
    assert!(!info.confirmed);
    rt.set_model_learning_enabled(false);
    world.state = vec![29, 181];
    rt.observe_external(&world.observe()).unwrap();
    assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    assert!(
        rt.organism()
            .phase_rule_disagreement(0, &world.observe())
            .unwrap()
            > 0.1
    );
    rt.set_goal(&[0.8, 0.7]).unwrap();
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    rt.set_model_learning_enabled(true);
    rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
    assert!(ready(&rt));
    assert_forecast(&rt, &world, &[17, 219], 0);
}

#[test]
fn old_single_channel_vocabulary_cannot_replace_a_two_source_rule() {
    for language in [
        PhaseRuleLanguage::SingleChannel,
        PhaseRuleLanguage::CircularAffine,
    ] {
        let mut world = World {
            laws: vec![vec![
                Law::new(&[(0, 1), (1, 1)], 17),
                Law::new(&[(0, 1), (1, -1)], 31),
            ]],
            state: vec![0, 0],
            history: vec![],
        };
        let mut rt = ScientificRuntime::new(fresh(2, 1, language, 1)).unwrap();
        rt.set_goal(&[0.1234567; 2]).unwrap();
        for state in [vec![11, 43], vec![79, 131], vec![153, 211]] {
            world.state = state;
            rt.observe_external(&world.observe()).unwrap();
            rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
        }
        rt.set_model_learning_enabled(false);
        world.state = vec![29, 181];
        rt.observe_external(&world.observe()).unwrap();
        rt.set_goal(&World::raster(&world.transformed(&world.state, 0)))
            .unwrap();
        if language == PhaseRuleLanguage::SingleChannel {
            assert!(!ready(&rt));
            assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
        } else {
            assert!(ready(&rt));
            rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
            assert!(rt.goal_reached().unwrap());
        }
    }
}

#[test]
fn both_pair_connections_and_the_constant_bias_are_physically_necessary() {
    let mut world = World {
        laws: vec![vec![
            Law::new(&[(0, 1), (2, 1)], 17),
            Law::new(&[(0, 1), (1, -1)], 31),
            Law::new(&[], 71),
        ]],
        state: vec![0; 3],
        history: vec![],
    };
    let mut rt = ScientificRuntime::new(fresh(3, 1, PhaseRuleLanguage::CircularAffine, 1)).unwrap();
    learn_episodes(&mut rt, &mut world, 0xABCD);
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    evo.set_planning_learning_enabled(false);
    let input = World::raster(&[29, 181, 87]);
    let forecast = evo.phase_rule_predict(0, &input).unwrap();
    evo.observe_initial_real(&input, false);
    let connections = evo.phase_rule_action_info(0).unwrap().synapses;
    assert_eq!(connections.len(), 5);
    for (i, &connection) in connections.iter().enumerate() {
        let saved = evo
            .perturb_phase_native_synapse_for_control(connection, 0.0, 0.0)
            .unwrap();
        assert!(evo.phase_rule_predict(0, &input).is_none());
        assert!(evo.phase_rule_decision(&forecast.sensory).is_none());
        evo.restore_phase_native_synapse_for_control(connection, saved.clone());
        assert_eq!(evo.phase_rule_predict(0, &input).unwrap(), forecast);
        evo.perturb_phase_native_synapse_for_control(connection, 1.0, std::f32::consts::PI)
            .unwrap();
        let changed = evo.phase_rule_predict(0, &input).unwrap();
        let output = if i < 2 {
            0
        } else if i < 4 {
            1
        } else {
            2
        };
        assert!((changed.sensory[output] - forecast.sensory[output]).abs() > 0.49);
        assert!(evo.phase_rule_decision(&forecast.sensory).is_none());
        evo.restore_phase_native_synapse_for_control(connection, saved);
        assert_eq!(evo.phase_rule_predict(0, &input).unwrap(), forecast);
    }
}

#[test]
fn a_changed_formula_family_is_revoked_and_reconfirmed_without_erasing_other_actions() {
    let mut world = mixed_world();
    let mut rt = ScientificRuntime::new(fresh(2, 4, PhaseRuleLanguage::CircularAffine, 1)).unwrap();
    learn_episodes(&mut rt, &mut world, 0xABCD);
    let other = rt.organism().phase_rule_predict(2, &[0.2, 0.8]).unwrap();
    world.state = vec![29, 181];
    rt.observe_external(&world.observe()).unwrap();
    rt.set_goal(&World::raster(&world.transformed(&world.state, 1)))
        .unwrap();
    assert_eq!(rt.propose().unwrap().unwrap().action, 1);
    world.laws[1][0] = Law::new(&[(0, -3)], 43);
    rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
    let info = rt.organism().phase_rule_action_info(1).unwrap();
    assert_eq!(info.revision, 1);
    assert_eq!(info.distinct_support, 1);
    assert!(!info.confirmed);
    assert_eq!(
        rt.organism().phase_rule_predict(2, &[0.2, 0.8]).unwrap(),
        other
    );
    learn_episodes(&mut rt, &mut world, 0x987654);
    assert_eq!(
        rt.organism().phase_rule_formulas(1).unwrap()[0].family,
        PhaseRuleFamily::IntegerGain
    );
    assert_eq!(rt.organism().phase_rule_action_info(1).unwrap().revision, 1);
    assert_forecast(&rt, &world, &[49, 211], 1);
    assert_eq!(
        rt.organism().phase_rule_predict(2, &[0.2, 0.8]).unwrap(),
        other
    );
}

#[test]
fn partial_inference_handles_a_constant_without_inputs_and_measures_an_unknown_sum() {
    let mut world = mixed_world();
    let mut rt = ScientificRuntime::new(fresh(2, 4, PhaseRuleLanguage::CircularAffine, 1)).unwrap();
    learn_episodes(&mut rt, &mut world, 0xABCD);
    let counts = (0..4)
        .map(|a| {
            rt.organism()
                .phase_rule_action_info(a)
                .unwrap()
                .observations
        })
        .collect::<Vec<_>>();
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    assert!(evo.enable_phase_partial_observation(PhasePartialConfig::default()));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    for _ in 0..20 {
        if (0..4).all(|a| rt.organism().phase_partial_mask_info(a).unwrap().confirmed) {
            break;
        }
        rt.step_partial(
            |_| Some(safe()),
            |a| {
                world.act(a);
                Ok((world.partial(), 0.0))
            },
        )
        .unwrap();
    }
    assert!((0..4).all(|a| rt.organism().phase_partial_mask_info(a).unwrap().confirmed));
    assert_eq!(
        (0..4)
            .map(|a| rt
                .organism()
                .phase_rule_action_info(a)
                .unwrap()
                .observations)
            .collect::<Vec<_>>(),
        counts
    );
    let evo = EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
        .unwrap();
    let mut rt = ScientificRuntime::new(evo).unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    rt.observe_external_partial(&[None, None]).unwrap();
    rt.set_partial_goal(&[Some(71.0 / PERIOD as f32), None])
        .unwrap();
    assert!(!rt.goal_reached().unwrap());
    rt.step_partial(
        |p| {
            assert_eq!(p.action, 2);
            Some(safe())
        },
        |a| {
            world.act(a);
            Ok((world.partial(), 0.0))
        },
    )
    .unwrap();
    assert!(rt.goal_reached().unwrap());
    world.state = vec![149, 203];
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(106.0 / PERIOD as f32), None])
        .unwrap();
    rt.step_partial(
        |p| {
            assert_eq!(p.action, 1);
            assert_eq!(p.mode, ReasoningMode::PartialInformationGathering);
            Some(safe())
        },
        |a| {
            world.act(a);
            Ok((world.partial(), 0.0))
        },
    )
    .unwrap();
    assert!(rt.goal_reached().unwrap());
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    assert_eq!(
        rt.organism().phase_partial_belief().unwrap().factual[1],
        None
    );
}

#[test]
fn v5_validates_formula_topology_and_legacy_v3_remains_readable() {
    let mut world = mixed_world();
    let mut rt = ScientificRuntime::new(fresh(2, 4, PhaseRuleLanguage::CircularAffine, 1)).unwrap();
    learn_episodes(&mut rt, &mut world, 0xABCD);
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    let valid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(valid["version"], 5);
    let pair = valid["rules"]["actions"][0]["outputs"][0]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c.get("secondary").is_some())
        .unwrap();
    let mutations: Vec<Box<dyn Fn(&mut serde_json::Value)>> = vec![
        Box::new(|v| v["version"] = 3.into()),
        Box::new(move |v| {
            v["rules"]["actions"][0]["outputs"][0]["candidates"][pair]["secondary"]["source"] =
                0.into()
        }),
        Box::new(move |v| {
            v["rules"]["actions"][0]["outputs"][0]["candidates"][pair]["secondary"]["synapse"] =
                v["rules"]["actions"][0]["outputs"][0]["candidates"][pair]["synapse"].clone()
        }),
        Box::new(|v| v["rules"]["language"] = "ArbitraryPrograms".into()),
        Box::new(|v| {
            v["rules"]["actions"][0]["outputs"][0]["candidates"][5]["orientation"] = 4.into()
        }),
    ];
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    for mutate in mutations {
        let mut corrupt = valid.clone();
        mutate(&mut corrupt);
        assert_eq!(
            rt.restore_online_checkpoint(&serde_json::to_vec(&corrupt).unwrap()),
            Err(RuntimeError::InvalidCheckpoint)
        );
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
    }
    let goal = World::raster(&world.transformed(&world.state, 1));
    rt.set_goal(&goal).unwrap();
    rt.set_model_learning_enabled(false);
    let mut stop = safe();
    stop.emergency_stop = true;
    rt.step(|_| Some(stop), |_| panic!("stopped actuator"))
        .unwrap();
    rt.restore_online_checkpoint(&bytes).unwrap();
    assert!(rt.emergency_latched());
    assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
    rt.observe_external(&world.observe()).unwrap();
    assert!(matches!(
        rt.step(|_| Some(safe()), |_| panic!("restore bypassed stop"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
    let mut old = ScientificRuntime::new(fresh(2, 4, PhaseRuleLanguage::SingleChannel, 1)).unwrap();
    world.laws = vec![world.laws[0].clone(); 4];
    learn_episodes(&mut old, &mut world, 0xABCD);
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&old.organism().online_checkpoint_bytes().unwrap()).unwrap();
    legacy["version"] = 3.into();
    legacy["rules"].as_object_mut().unwrap().remove("language");
    let evo = EvoPhase::from_online_checkpoint(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(
        evo.phase_rule_language(),
        Some(PhaseRuleLanguage::SingleChannel)
    );
    assert!((0..4).all(|a| evo.phase_rule_action_info(a).unwrap().confirmed));
}

#[test]
fn the_larger_candidate_budget_is_atomic_and_does_not_grow_with_tuition() {
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: 16,
        motor_cells: 32,
        dormant_cells: 516,
        ..Default::default()
    });
    evo.enable_phase_native_planning(Default::default());
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    let fingerprint = evo.phase_native_learned_fingerprint();
    assert!(!evo.enable_phase_rule_learning_with_language(
        Default::default(),
        PhaseRuleLanguage::CircularAffine
    ));
    assert_eq!(evo.phase_native_learned_fingerprint(), fingerprint);
    assert!(evo.enable_phase_rule_learning(Default::default()));
    let fingerprint = evo.phase_native_learned_fingerprint();
    assert!(!evo.expand_phase_rule_language());
    assert_eq!(evo.phase_native_learned_fingerprint(), fingerprint);
    let mut world = mixed_world();
    let mut rt = ScientificRuntime::new(fresh(2, 4, PhaseRuleLanguage::CircularAffine, 1)).unwrap();
    let before: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    learn_episodes(&mut rt, &mut world, 0xABCD);
    let after: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    assert_eq!(before["synapses"].as_array().unwrap().len(), 168);
    assert_eq!(
        before["synapses"].as_array().unwrap().len(),
        after["synapses"].as_array().unwrap().len()
    );
    assert_eq!(
        before["cells"].as_array().unwrap().len(),
        after["cells"].as_array().unwrap().len()
    );
}

#[test]
fn expanding_acquired_knowledge_preserves_parameters_and_requires_resolving_new_ambiguity() {
    let mut world = World {
        laws: vec![vec![Law::new(&[(0, 1)], 17), Law::new(&[(0, 1)], 31)]],
        state: vec![0, 0],
        history: Vec::new(),
    };
    let mut rt = ScientificRuntime::new(fresh(2, 1, PhaseRuleLanguage::SingleChannel, 1)).unwrap();
    rt.set_goal(&[0.1234567; 2]).unwrap();
    for x in [23, 71, 149] {
        world.state = vec![x, 0];
        rt.observe_external(&world.observe()).unwrap();
        rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
    }
    assert!(ready(&rt));
    // Acquire a real output mask without giving the numerical model inferred
    // hidden tuition. The expansion must retain this independent knowledge.
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    assert!(evo.enable_phase_partial_observation(Default::default()));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    for _ in 0..3 {
        rt.step_partial(
            |_| Some(safe()),
            |a| {
                world.act(a);
                Ok((world.partial(), 0.0))
            },
        )
        .unwrap();
    }
    assert!(rt.organism().phase_partial_mask_info(0).unwrap().confirmed);
    let before: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    let old_info = rt.organism().phase_rule_action_info(0).unwrap();
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    evo.set_planning_learning_enabled(false);
    let fingerprint = evo.phase_native_learned_fingerprint();
    assert!(!evo.expand_phase_rule_language());
    assert_eq!(evo.phase_native_learned_fingerprint(), fingerprint);
    evo.set_planning_learning_enabled(true);
    assert!(evo.expand_phase_rule_language());
    assert!(evo.current_real().is_none());
    assert_eq!(
        evo.phase_rule_language(),
        Some(PhaseRuleLanguage::CircularAffine)
    );
    let after: serde_json::Value =
        serde_json::from_slice(&evo.online_checkpoint_bytes().unwrap()).unwrap();
    assert_eq!(
        after["partial"], before["partial"],
        "lost acquired visibility evidence"
    );
    let old_synapses = before["synapses"].as_array().unwrap();
    assert_eq!(
        &after["synapses"].as_array().unwrap()[..old_synapses.len()],
        old_synapses.as_slice()
    );
    let info = evo.phase_rule_action_info(0).unwrap();
    assert_eq!(info.observations, old_info.observations);
    assert_eq!(info.revision, old_info.revision);
    assert_eq!(info.evidence_sources, old_info.evidence_sources);
    assert!(
        !info.confirmed,
        "expanded language hid a new plausible explanation"
    );
    assert!(info.candidate_counts.iter().all(|&n| n > 1));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    rt.set_model_learning_enabled(false);
    world.state = vec![29, 181];
    rt.observe_external(&world.observe()).unwrap();
    rt.set_goal(&[0.1234567; 2]).unwrap();
    assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
    rt.set_model_learning_enabled(true);
    rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
    assert!(ready(&rt));
    assert_eq!(
        rt.organism()
            .phase_rule_action_info(0)
            .unwrap()
            .observations,
        old_info.observations + 1
    );
    assert_eq!(
        rt.organism().phase_rule_action_info(0).unwrap().revision,
        old_info.revision
    );
    assert_forecast(&rt, &world, &[49, 219], 0);
}

#[test]
fn unsupported_nonlinear_dynamics_do_not_become_a_confirmed_affine_model() {
    let mut rt = ScientificRuntime::new(fresh(1, 1, PhaseRuleLanguage::CircularAffine, 1)).unwrap();
    rt.set_goal(&[0.1234567]).unwrap();
    for x in [11, 29, 71, 149, 203] {
        rt.observe_external(&[x as f32 / PERIOD as f32]).unwrap();
        rt.step(
            |_| Some(safe()),
            |_| Ok(vec![((x * x + 17) % PERIOD) as f32 / PERIOD as f32]),
        )
        .unwrap();
        assert!(!ready(&rt));
    }
    rt.set_model_learning_enabled(false);
    assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
}
