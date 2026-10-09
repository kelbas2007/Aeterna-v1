use aeterna_v1::carrier::{
    PhaseAdaptiveConfig, PhaseNativeConfig, PhaseOnlineConfig, PhaseRuleConfig, PhaseRuleLanguage,
};
use aeterna_v1::scientific_runtime::{RuntimeError, ScientificRuntime, StepOutcome};
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
}
#[derive(Clone)]
struct World {
    state: [i32; 2],
    context: usize,
    roles: [usize; 2],
    threshold: i32,
    regime: usize,
    events: u64,
}
impl World {
    fn new(seed: u64) -> Self {
        Self {
            state: [29, 181],
            context: (seed % 2) as usize,
            roles: if seed % 3 == 0 { [1, 0] } else { [0, 1] },
            threshold: 105 + (seed % 48) as i32,
            regime: 0,
            events: 0,
        }
    }
    fn transformed(&self, action: usize) -> [i32; 2] {
        let c = self.context;
        let x = 1 - c;
        let high = self.state[c] >= self.threshold;
        let (mut gain, mut bias) = match (self.roles[action], high) {
            (0, false) => (1, 17),
            (0, true) => (2, 71),
            (1, false) => (-1, 31),
            (1, true) => (1, 53),
            _ => unreachable!(),
        };
        if self.regime == 1 {
            gain = -gain;
            bias += 101;
        }
        let mut post = self.state;
        post[c] = (self.state[c] + 31).rem_euclid(PERIOD);
        post[x] = (gain * self.state[x] + bias).rem_euclid(PERIOD);
        post
    }
    // Deterministic bounded sensor error, independently evaluated. The learner
    // receives measured frames, never true values, conditions or regime labels.
    fn measured(&self) -> Vec<f32> {
        self.state
            .iter()
            .enumerate()
            .map(|(j, &x)| {
                let sign = if (self.events * 3 + j as u64) % 4 < 2 {
                    1.0
                } else {
                    -1.0
                };
                (x as f32 / 257.0 + sign * 0.00035).rem_euclid(1.0)
            })
            .collect()
    }
    fn act(&mut self, a: usize) -> Vec<f32> {
        self.state = self.transformed(a);
        self.events += 1;
        self.measured()
    }
}
fn fresh(config: PhaseAdaptiveConfig, motors: usize) -> EvoPhase {
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: 2,
        motor_cells: motors,
        dormant_cells: 2 * motors * (config.model_slots + 1) + 4,
        ..Default::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning_with_language(
        PhaseRuleConfig {
            planning_depth: 2,
            ..Default::default()
        },
        PhaseRuleLanguage::CircularAffine
    ));
    assert!(evo.enable_phase_adaptive_rules(config));
    evo
}
fn train(rt: &mut ScientificRuntime, world: &mut World, actions: usize, initial: bool) {
    if initial {
        rt.observe_external(&world.measured()).unwrap();
        rt.set_goal(&[0.1234567, 0.2345678]).unwrap();
    }
    for _ in 0..actions {
        if rt.goal_reached().unwrap() {
            let next = rt
                .organism()
                .current_real()
                .unwrap()
                .sensory
                .iter()
                .map(|x| (x + 0.5).rem_euclid(1.0))
                .collect::<Vec<_>>();
            rt.set_goal(&next).unwrap();
        }
        assert!(matches!(
            rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap(),
            StepOutcome::Executed { learned: true, .. }
        ));
    }
}
fn evaluate(bytes: &[u8], world: &World, seed: u64, tasks: usize) -> usize {
    let mut rt = ScientificRuntime::new(EvoPhase::from_online_checkpoint(bytes).unwrap()).unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mut rng = Rng(seed);
    let mut successes = 0;
    for task in 0..tasks {
        let mut w = world.clone();
        w.state[world.context] = if task % 2 == 0 { 19 } else { 229 };
        w.state[1 - world.context] = (rng.next() % 257) as i32;
        let target = w.transformed(task % 2).map(|x| x as f32 / 257.0);
        rt.observe_external(&w.measured()).unwrap();
        rt.set_goal(&target).unwrap();
        match rt.step(|_| Some(safe()), |a| Ok(w.act(a))) {
            Ok(StepOutcome::Executed { learned: false, .. }) => {
                successes += usize::from(rt.goal_reached().unwrap())
            }
            Err(RuntimeError::NoSupportedAction) => {}
            other => panic!("unexpected frozen result: {other:?}"),
        }
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
    }
    successes
}

#[test]
fn conditions_are_discovered_under_noise_and_beat_global_rules_with_the_same_budget() {
    let mut conditional_success = 0;
    let mut global_success = 0;
    for seed in 1..=16 {
        let mut world = World::new(0xCE0000 + seed);
        let mut rt = ScientificRuntime::new(fresh(
            PhaseAdaptiveConfig {
                min_inlier_fraction: 1.0,
                ..Default::default()
            },
            2,
        ))
        .unwrap();
        train(&mut rt, &mut world, 256, true);
        for a in 0..2 {
            let info = rt.organism().phase_adaptive_action_info(a).unwrap();
            let model = info
                .active
                .as_ref()
                .unwrap_or_else(|| panic!("seed={seed} a={a} missing: {info:?}"));
            let c = model
                .condition
                .as_ref()
                .unwrap_or_else(|| panic!("seed={seed} a={a} not conditional: {info:?}"));
            assert_eq!(c.source, world.context);
            assert!(c.lower < world.threshold as f32 / 257.0 + 0.001);
            assert!(c.upper > world.threshold as f32 / 257.0 - 0.001);
            assert!(model.supports.iter().all(|&n| n >= 6));
            let prediction = rt
                .organism()
                .phase_adaptive_predict(a, &[0.07, 0.87])
                .unwrap();
            assert_eq!(prediction.authority, Authority::Imagined);
            assert!(prediction.error_radius.iter().all(|&r| r == 0.002));
            let mut gap = vec![0.17; 2];
            gap[c.source] = (c.lower + c.upper) / 2.0;
            assert!(rt.organism().phase_adaptive_predict(a, &gap).is_none());
        }
        let bytes = rt.organism().online_checkpoint_bytes().unwrap();
        conditional_success += evaluate(&bytes, &world, 0xFA170000 + seed, 16);
        let mut baseline_world = World::new(0xCE0000 + seed);
        let mut baseline = ScientificRuntime::new(fresh(
            PhaseAdaptiveConfig {
                allow_conditions: false,
                min_inlier_fraction: 1.0,
                ..Default::default()
            },
            2,
        ))
        .unwrap();
        train(&mut baseline, &mut baseline_world, 256, true);
        global_success += evaluate(
            &baseline.organism().online_checkpoint_bytes().unwrap(),
            &baseline_world,
            0xFA170000 + seed,
            16,
        );
    }
    assert_eq!(conditional_success, 256);
    assert!(global_success < conditional_success);
    println!("noisy_worlds=16 tuition_budget=256 heldout_tasks=256 action_budget=1 conditional={conditional_success} global={global_success}");
}

#[test]
fn continuous_lifetime_revises_laws_preserves_old_physical_models_and_reactivates_them() {
    let mut world = World::new(0xABCD);
    let mut rt = ScientificRuntime::new(fresh(PhaseAdaptiveConfig::default(), 2)).unwrap();
    train(&mut rt, &mut world, 256, true);
    let original = (0..2)
        .map(|a| {
            rt.organism()
                .phase_adaptive_action_info(a)
                .unwrap()
                .active
                .unwrap()
        })
        .collect::<Vec<_>>();
    let original_parameters = original
        .iter()
        .map(|m| {
            m.synapses
                .iter()
                .map(|&s| rt.organism().phase_native_synapse(s).unwrap())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    world.regime = 1;
    train(&mut rt, &mut world, 256, false);
    for a in 0..2 {
        let info = rt.organism().phase_adaptive_action_info(a).unwrap();
        assert!(info.active.as_ref().unwrap().condition.is_some());
        assert!(
            info.archives
                .iter()
                .any(|m| m.synapses == original[a].synapses),
            "old model lost: {info:?}"
        );
        for (&s, old) in original[a].synapses.iter().zip(&original_parameters[a]) {
            let p = rt.organism().phase_native_synapse(s).unwrap();
            assert_eq!(p.phase_offset, old.phase_offset);
            assert_eq!(p.weight, old.weight);
        }
    }
    let changed = rt.organism().online_checkpoint_bytes().unwrap();
    assert_eq!(evaluate(&changed, &world, 0xA0170, 32), 32);
    // Independently validate restart without interrupting the original life.
    let mut resumed =
        ScientificRuntime::new(EvoPhase::from_online_checkpoint(&changed).unwrap()).unwrap();
    resumed.set_goal(&[0.1234567, 0.2345678]).unwrap();
    assert_eq!(
        resumed.propose(),
        Err(RuntimeError::FreshObservationRequired)
    );
    world.regime = 0;
    train(&mut rt, &mut world, 256, false);
    for a in 0..2 {
        let info = rt.organism().phase_adaptive_action_info(a).unwrap();
        assert!(info.reactivations > 0);
        assert_eq!(info.active.as_ref().unwrap().synapses, original[a].synapses);
        assert!(info.archives.iter().any(|m| m.condition.is_some()));
    }
    assert_eq!(
        evaluate(
            &rt.organism().online_checkpoint_bytes().unwrap(),
            &world,
            0xA0170,
            32
        ),
        32
    );
    println!("continuous_actions=768 changed_goals=32 recalled_goals=32 original_phase_parameters_preserved=true");
}

#[test]
fn noise_support_is_accumulated_without_publishing_ambiguous_or_duplicate_examples() {
    let mut rt = ScientificRuntime::new(fresh(PhaseAdaptiveConfig::default(), 1)).unwrap();
    let mut exact = EvoPhase::new(EvoConfig {
        sensory_cells: 2,
        motor_cells: 1,
        dormant_cells: 6,
        ..Default::default()
    });
    exact.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(exact.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(exact.enable_phase_rule_learning_with_language(
        PhaseRuleConfig::default(),
        PhaseRuleLanguage::CircularAffine
    ));
    let mut strict = ScientificRuntime::new(exact).unwrap();
    let mut rng = Rng(0xABCDE);
    for n in 0..32 {
        let pre = vec![
            (rng.next() % 257) as f32 / 257.0,
            (rng.next() % 257) as f32 / 257.0,
        ];
        rt.observe_external(&pre).unwrap();
        rt.set_goal(&[0.1234567, 0.2345678]).unwrap();
        let error = if n % 2 == 0 { 0.0007 } else { -0.0007 };
        let post = vec![
            (pre[0] + 17.0 / 257.0 + error).rem_euclid(1.0),
            (pre[1] - 31.0 / 257.0 + error).rem_euclid(1.0),
        ];
        strict.observe_external(&pre).unwrap();
        strict.set_goal(&[0.1234567, 0.2345678]).unwrap();
        strict.step(|_| Some(safe()), |_| Ok(post.clone())).unwrap();
        rt.step(|_| Some(safe()), |_| Ok(post)).unwrap();
        if n < 5 {
            assert!(rt
                .organism()
                .phase_adaptive_action_info(0)
                .unwrap()
                .active
                .is_none());
        }
    }
    assert!(rt
        .organism()
        .phase_adaptive_action_info(0)
        .unwrap()
        .active
        .is_some());
    assert!(
        !strict
            .organism()
            .phase_rule_action_info(0)
            .unwrap()
            .confirmed
    );
    println!("noisy_translation_tuition=32 robust_confirmed=true strict_confirmed=false");
    let f = rt
        .organism()
        .phase_adaptive_predict(0, &[0.37, 0.81])
        .unwrap();
    assert!((f.sensory[0] - (0.37 + 17.0 / 257.0)).abs() <= f.error_radius[0]);
    // A single large outlier does not revoke a supported model.
    let model = rt
        .organism()
        .phase_adaptive_action_info(0)
        .unwrap()
        .active
        .unwrap();
    rt.observe_external(&[0.19, 0.83]).unwrap();
    rt.set_goal(&[0.1234567, 0.2345678]).unwrap();
    rt.step(|_| Some(safe()), |_| Ok(vec![0.99, 0.22])).unwrap();
    let after = rt
        .organism()
        .phase_adaptive_action_info(0)
        .unwrap()
        .active
        .unwrap();
    assert_eq!(after.synapses, model.synapses);
    assert_eq!(after.formulas, model.formulas);
    let mut duplicate = ScientificRuntime::new(fresh(PhaseAdaptiveConfig::default(), 1)).unwrap();
    for _ in 0..40 {
        duplicate.observe_external(&[0.2, 0.4]).unwrap();
        duplicate.set_goal(&[0.1234567, 0.2345678]).unwrap();
        duplicate
            .step(|_| Some(safe()), |_| Ok(vec![0.3, 0.5]))
            .unwrap();
    }
    let info = duplicate.organism().phase_adaptive_action_info(0).unwrap();
    assert_eq!(info.retained_examples, 1);
    assert!(info.active.is_none());
    let mut ambiguous = ScientificRuntime::new(fresh(PhaseAdaptiveConfig::default(), 1)).unwrap();
    for n in 0..32 {
        let x = (n as f32 * 0.027).rem_euclid(1.0);
        ambiguous.observe_external(&[x, x]).unwrap();
        ambiguous.set_goal(&[0.1234567, 0.2345678]).unwrap();
        ambiguous
            .step(
                |_| Some(safe()),
                |_| {
                    Ok(vec![
                        (2.0 * x + 0.17).rem_euclid(1.0),
                        (x + 0.31).rem_euclid(1.0),
                    ])
                },
            )
            .unwrap();
    }
    assert!(ambiguous
        .organism()
        .phase_adaptive_action_info(0)
        .unwrap()
        .active
        .is_none());
}

#[test]
fn allocation_and_fit_budgets_are_atomic_and_nonlinear_laws_remain_unsupported() {
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: 8,
        motor_cells: 4,
        dormant_cells: 200,
        ..Default::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(evo.enable_phase_rule_learning_with_language(
        PhaseRuleConfig::default(),
        PhaseRuleLanguage::CircularAffine
    ));
    let before = evo.online_checkpoint_bytes().unwrap();
    assert!(!evo.enable_phase_adaptive_rules(PhaseAdaptiveConfig::default()));
    assert_eq!(before, evo.online_checkpoint_bytes().unwrap());
    let mut cold = fresh(PhaseAdaptiveConfig::default(), 1);
    assert!(!cold.enable_phase_adaptive_rules(PhaseAdaptiveConfig {
        fit_budget: 0,
        ..Default::default()
    }));
    assert!(cold.phase_adaptive_rules_enabled());
    let mut nonlinear = ScientificRuntime::new(fresh(
        PhaseAdaptiveConfig {
            min_inlier_fraction: 1.0,
            fit_budget: 1024,
            ..Default::default()
        },
        1,
    ))
    .unwrap();
    let mut rng = Rng(0xA01ABCDE);
    for _ in 0..96 {
        let x = (rng.next() % 257) as i32;
        let y = (rng.next() % 257) as i32;
        nonlinear
            .observe_external(&[x as f32 / 257.0, y as f32 / 257.0])
            .unwrap();
        nonlinear.set_goal(&[0.1234567, 0.2345678]).unwrap();
        nonlinear
            .step(
                |_| Some(safe()),
                |_| {
                    Ok(vec![
                        (x * x + 17).rem_euclid(PERIOD) as f32 / 257.0,
                        y as f32 / 257.0,
                    ])
                },
            )
            .unwrap();
    }
    nonlinear.set_model_learning_enabled(false);
    assert!(nonlinear
        .organism()
        .phase_adaptive_action_info(0)
        .unwrap()
        .active
        .is_none());
    assert_eq!(nonlinear.propose(), Err(RuntimeError::NoSupportedAction));
}

#[test]
fn memory_is_bounded_and_physical_lesions_cannot_read_a_backup_formula() {
    let mut world = World::new(0xABCD);
    let mut rt = ScientificRuntime::new(fresh(PhaseAdaptiveConfig::default(), 2)).unwrap();
    let before: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    train(&mut rt, &mut world, 2048, true);
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    let after: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        before["synapses"].as_array().unwrap().len(),
        after["synapses"].as_array().unwrap().len()
    );
    assert_eq!(
        before["cells"].as_array().unwrap().len(),
        after["cells"].as_array().unwrap().len()
    );
    assert!(bytes.len() < 200_000);
    for a in 0..2 {
        let info = rt.organism().phase_adaptive_action_info(a).unwrap();
        assert!(info.retained_examples <= 64);
        assert!(info.archives.len() <= 4);
    }
    let info = rt.organism().phase_adaptive_action_info(0).unwrap();
    let model = info.active.unwrap();
    let context = model.condition.as_ref().unwrap();
    let mut input = vec![0.37; 2];
    input[context.source] = 0.07;
    let original = rt.organism().phase_adaptive_predict(0, &input).unwrap();
    let mut evo = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    assert!(!evo.enable_phase_partial_observation(Default::default()));
    for &s in &model.synapses[..2] {
        let saved = evo
            .perturb_phase_native_synapse_for_control(s, 0.0, 0.0)
            .unwrap();
        assert!(evo.phase_adaptive_predict(0, &input).is_none());
        evo.restore_phase_native_synapse_for_control(s, saved.clone());
        assert_eq!(evo.phase_adaptive_predict(0, &input).unwrap(), original);
        evo.perturb_phase_native_synapse_for_control(s, 1.0, std::f32::consts::PI)
            .unwrap();
        let altered = evo.phase_adaptive_predict(0, &input).unwrap();
        assert!(altered
            .sensory
            .iter()
            .zip(&original.sensory)
            .any(|(x, y)| (x - y).abs() > 0.49));
        evo.restore_phase_native_synapse_for_control(s, saved);
    }
    println!(
        "continuous_actions=2048 checkpoint_bytes={} bounded_physical_topology=true",
        bytes.len()
    );
}

#[test]
fn checkpoint_and_execution_boundaries_preserve_freeze_stop_and_atomic_rejection() {
    let mut world = World::new(0xABCD);
    let mut rt = ScientificRuntime::new(fresh(PhaseAdaptiveConfig::default(), 2)).unwrap();
    train(&mut rt, &mut world, 256, true);
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mutations: Vec<Box<dyn Fn(&mut serde_json::Value)>> = vec![
        Box::new(|v| v["rules"]["adaptive"]["config"]["allow_conditions"] = false.into()),
        Box::new(|v| {
            v["rules"]["adaptive"]["actions"][0]["active"]["supports"] = serde_json::json!([64, 64])
        }),
        Box::new(|v| v["version"] = 4.into()),
        Box::new(|v| v["rules"]["adaptive"]["config"]["model_slots"] = u64::MAX.into()),
        Box::new(|v| {
            v["rules"]["adaptive"]["actions"][0]["active"]["slots"] = serde_json::json!([0, 0])
        }),
        Box::new(|v| {
            v["rules"]["adaptive"]["actions"][0]["active"]["condition"]["source"] = 99.into()
        }),
        Box::new(|v| {
            v["rules"]["adaptive"]["actions"][0]["slots"][0]["outputs"][0]["candidates"][0]
                ["synapse"] = 0.into()
        }),
    ];
    for change in mutations {
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        change(&mut value);
        assert_eq!(
            rt.restore_online_checkpoint(&serde_json::to_vec(&value).unwrap()),
            Err(RuntimeError::InvalidCheckpoint)
        );
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
    }
    world.state[world.context] = 19;
    rt.observe_external(&world.measured()).unwrap();
    rt.set_goal(&world.transformed(0).map(|x| x as f32 / 257.0))
        .unwrap();
    assert!(matches!(
        rt.step(|_| None, |_| panic!("blocked executor called"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    assert!(matches!(
        rt.step(|_| Some(safe()), |_| Ok(vec![f32::NAN, 0.3]))
            .unwrap(),
        StepOutcome::ExecutionFault(_)
    ));
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    rt.restore_online_checkpoint(&bytes).unwrap();
    assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
    rt.observe_external(&world.measured()).unwrap();
    let mut stop = safe();
    stop.emergency_stop = true;
    assert!(matches!(
        rt.step(|_| Some(stop), |_| panic!("stop executor called"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
    rt.restore_online_checkpoint(&bytes).unwrap();
    rt.observe_external(&world.measured()).unwrap();
    assert!(rt.emergency_latched());
    assert!(matches!(
        rt.step(|_| Some(safe()), |_| panic!("restore bypassed stop"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
}

#[test]
fn circular_uncertainty_crossing_zero_cannot_select_one_conditional_branch() {
    let mut rt = ScientificRuntime::new(fresh(
        PhaseAdaptiveConfig {
            min_inlier_fraction: 1.0,
            ..Default::default()
        },
        1,
    ))
    .unwrap();
    let mut rng = Rng(0xB01ABCDE);
    for _ in 0..80 {
        let x = (rng.next() % 257) as f32 / 257.0;
        let c = (rng.next() % 257) as f32 / 257.0;
        rt.observe_external(&[x, c]).unwrap();
        rt.set_goal(&[0.1234567, 0.2345678]).unwrap();
        rt.step(
            |_| Some(safe()),
            |_| {
                Ok(vec![
                    (x + if c < 0.5 { 0.17 } else { 0.31 }).rem_euclid(1.0),
                    0.9995,
                ])
            },
        )
        .unwrap();
    }
    assert_eq!(
        rt.organism()
            .phase_adaptive_action_info(0)
            .unwrap()
            .active
            .unwrap()
            .condition
            .unwrap()
            .source,
        1
    );
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    evo.set_planning_learning_enabled(false);
    evo.observe_initial_real(&[0.29, 0.1], false);
    let first = evo.phase_adaptive_predict(0, &[0.29, 0.1]).unwrap();
    assert!(evo.phase_adaptive_predict(0, &[0.29, 0.0001]).is_none());
    assert!(evo.phase_adaptive_predict(0, &[0.29, 0.9999]).is_none());
    assert!((first.sensory[0] - 0.46).abs() < 0.0001);
    assert!(first.sensory[1] + first.error_radius[1] > 1.0);
    // A naive point-only second step would predict 0.77. Its context interval
    // also includes zero, which uses the other acquired law. Abstain.
    assert!(evo.phase_adaptive_decision(&[0.77, 0.9995]).is_none());
    assert!(evo.phase_adaptive_decision(&[0.46, 0.9995]).is_some());
}
