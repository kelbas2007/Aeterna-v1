use aeterna_v1::carrier::{
    PhaseNativeConfig, PhaseOnlineConfig, PhaseRuleConfig, PhaseRuleDecisionKind,
};
use aeterna_v1::scientific_runtime::{ReasoningMode, RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{Authority, EvoConfig, EvoPhase, HumanProtectionEvidence};

const PERIOD: u32 = 257;

fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}

fn organism(width: usize, motors: usize, rules: PhaseRuleConfig) -> EvoPhase {
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
    assert!(evo.enable_phase_rule_learning(rules));
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
}

// Only this external evaluator knows the transformation, channel identities,
// signs or offsets. None are passed into cognition. Integer arithmetic here
// deliberately differs from the learner's floating point phase readout.
struct World {
    sources: Vec<Vec<usize>>,
    reflected: Vec<Vec<bool>>,
    offsets: Vec<Vec<u32>>,
    state: Vec<u32>,
    factual_history: Vec<Vec<u32>>,
}
impl World {
    fn new(seed: u64, width: usize, motors: usize) -> Self {
        let mut rng = Rng(seed);
        let mut sources = Vec::new();
        let mut reflected = Vec::new();
        let mut offsets = Vec::new();
        for _ in 0..motors {
            let mut source = (0..width).collect::<Vec<_>>();
            for i in (1..width).rev() {
                source.swap(i, rng.next() as usize % (i + 1));
            }
            sources.push(source);
            reflected.push((0..width).map(|_| rng.next() % 2 == 0).collect());
            offsets.push((0..width).map(|_| 1 + rng.next() % (PERIOD - 1)).collect());
        }
        Self {
            sources,
            reflected,
            offsets,
            state: (0..width).map(|_| rng.next() % PERIOD).collect(),
            factual_history: Vec::new(),
        }
    }
    fn transformed(&self, input: &[u32], action: usize) -> Vec<u32> {
        (0..input.len())
            .map(|j| {
                let value = input[self.sources[action][j]];
                let value = if self.reflected[action][j] {
                    PERIOD - value
                } else {
                    value
                };
                (value + self.offsets[action][j]) % PERIOD
            })
            .collect()
    }
    fn raster(input: &[u32]) -> Vec<f32> {
        input.iter().map(|&x| x as f32 / PERIOD as f32).collect()
    }
    fn observe(&self) -> Vec<f32> {
        Self::raster(&self.state)
    }
    fn act(&mut self, action: usize) -> Vec<f32> {
        self.factual_history.push(self.state.clone());
        self.state = self.transformed(&self.state, action);
        self.factual_history.push(self.state.clone());
        self.observe()
    }
}

fn all_confirmed(rt: &ScientificRuntime) -> bool {
    (0..rt.organism().config().motor_cells)
        .all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed)
}

fn learn(rt: &mut ScientificRuntime, world: &mut World, budget: usize) -> usize {
    rt.observe_external(&world.observe()).unwrap();
    rt.set_goal(&vec![0.1234567; world.state.len()]).unwrap(); // off the evaluator grid
    for n in 0..budget {
        if all_confirmed(rt) {
            return n;
        }
        let outcome = rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
        assert!(
            matches!(outcome, StepOutcome::Executed { .. }),
            "{outcome:?}"
        );
    }
    panic!(
        "acquisition exhausted {budget} actions: {:?}",
        (0..world.sources.len())
            .map(|a| rt.organism().phase_rule_action_info(a))
            .collect::<Vec<_>>()
    );
}

fn reach(rt: &mut ScientificRuntime, world: &mut World, goal: &[f32], budget: usize) -> usize {
    rt.observe_external(&world.observe()).unwrap();
    rt.set_goal(goal).unwrap();
    for n in 0..=budget {
        if rt.goal_reached().unwrap() {
            return n;
        }
        assert!(n < budget, "goal budget exhausted");
        let outcome = rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
        assert!(
            matches!(outcome, StepOutcome::Executed { .. }),
            "{outcome:?}"
        );
    }
    unreachable!()
}

#[test]
fn cold_rules_transfer_to_unseen_states_and_goals_without_learning_after_disk_restore() {
    let mut total_actions = 0;
    let mut transfers = 0;
    for seed in 1..=32 {
        let width = 3 + seed as usize % 3;
        let motors = 2 + seed as usize % 3;
        let mut world = World::new(seed * 0x9E3779B9, width, motors);
        let mut rt = ScientificRuntime::new(organism(width, motors, Default::default())).unwrap();
        total_actions += learn(&mut rt, &mut world, 80);
        let training_history = world.factual_history.clone();
        assert_eq!(rt.organism().phase_native_receptor_count(), 0);
        assert_eq!(rt.organism().planning_transition_count(), 0);
        let bytes = rt.organism().online_checkpoint_bytes().unwrap();
        let restored = EvoPhase::from_online_checkpoint(&bytes).unwrap();
        assert_eq!(
            restored.phase_native_learned_fingerprint(),
            rt.organism().phase_native_learned_fingerprint()
        );
        let mut resumed = ScientificRuntime::new(restored).unwrap();
        resumed.set_model_learning_enabled(false);
        assert_eq!(
            resumed.goal_reached(),
            Err(RuntimeError::FreshObservationRequired)
        );
        let fingerprint = resumed.organism().phase_native_learned_fingerprint();
        let mut rng = Rng(0xD00D1234 ^ seed);
        for _ in 0..8 {
            world.state = (0..width).map(|_| rng.next() % PERIOD).collect();
            assert!(
                !training_history.contains(&world.state),
                "evaluation PRE appeared during acquisition"
            );
            // Held-out predictions for every opaque action are compared to
            // independently computed consequences before any physical ACT.
            for action in 0..motors {
                let prediction = resumed
                    .organism()
                    .phase_rule_predict(action, &world.observe())
                    .unwrap();
                assert_eq!(prediction.authority, Authority::Imagined);
                assert!(prediction.evidence_sources.len() >= 3);
                let expected = World::raster(&world.transformed(&world.state, action));
                assert!(
                    prediction
                        .sensory
                        .iter()
                        .zip(&expected)
                        .all(|(a, b)| (a - b).abs() < 0.0001),
                    "seed={seed} action={action} expected={expected:?} actual={:?}",
                    prediction.sensory
                );
            }
            let mut target = world.state.clone();
            for _ in 0..4 {
                target = world.transformed(&target, rng.next() as usize % motors);
            }
            // A cancelling composition must not inflate the score with a
            // goal already satisfied by the initial observation.
            if target == world.state {
                target = (0..motors)
                    .map(|a| world.transformed(&world.state, a))
                    .find(|s| s != &world.state)
                    .expect("nontrivial held-out goal");
            }
            assert!(
                !training_history.contains(&target),
                "evaluation goal appeared during acquisition"
            );
            assert_ne!(target, world.state);
            reach(&mut resumed, &mut world, &World::raster(&target), 4);
            assert_eq!(
                resumed.organism().phase_native_learned_fingerprint(),
                fingerprint
            );
            transfers += 1;
        }
        assert_eq!(resumed.organism().phase_native_receptor_count(), 0);
    }
    println!("cold_worlds=32 heldout_goals={transfers} acquisition_actions={total_actions}");
    assert_eq!(transfers, 256);
}

#[test]
fn an_experiment_discriminates_hypotheses_instead_of_repeating_known_input() {
    let mut rt = ScientificRuntime::new(organism(1, 2, Default::default())).unwrap();
    rt.observe_external(&[0.10]).unwrap();
    rt.set_goal(&[0.1234567]).unwrap();
    let mut current = 0.10;
    for expected in [0, 1] {
        rt.step(
            |p| {
                assert_eq!(p.action, expected);
                Some(safe())
            },
            |a| {
                current = (current + [0.07, 0.11][a]) % 1.0;
                Ok(vec![current])
            },
        )
        .unwrap();
    }
    assert!(rt.organism().phase_rule_predict(0, &[0.10]).is_none());
    rt.observe_external(&[0.10]).unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let decision = rt.organism().phase_rule_decision(&[0.1234567]).unwrap();
    assert_eq!(decision.action, 1); // action 0 would replay its sole known PRE
    assert_eq!(decision.kind, PhaseRuleDecisionKind::Experiment);
    assert!(decision.expected_disagreement > 0.2);
    assert!(rt.organism().phase_rule_disagreement(0, &[0.10]).unwrap() < 0.00001);
    for _ in 0..10 {
        let proposal = rt.propose().unwrap().unwrap();
        assert_eq!(proposal.mode, ReasoningMode::RuleExperiment);
        assert_eq!(proposal.action, 1);
    }
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    let alternatives = rt
        .organism()
        .phase_rule_action_info(1)
        .unwrap()
        .candidate_counts[0];
    rt.step(|_| Some(safe()), |_| Ok(vec![0.21])).unwrap();
    assert!(
        rt.organism()
            .phase_rule_action_info(1)
            .unwrap()
            .candidate_counts[0]
            < alternatives
    );
    assert!(rt.organism().phase_rule_predict(1, &[0.81]).is_none()); // only two distinct sources
}

#[test]
fn repeated_pre_goals_blocked_actions_and_invalid_post_cannot_confirm_a_rule() {
    let mut rt = ScientificRuntime::new(organism(1, 1, Default::default())).unwrap();
    rt.observe_external(&[0.2]).unwrap();
    rt.set_goal(&[0.9]).unwrap();
    for _ in 0..6 {
        rt.observe_external(&[0.2]).unwrap();
        rt.step(|_| Some(safe()), |_| Ok(vec![0.4])).unwrap();
    }
    let info = rt.organism().phase_rule_action_info(0).unwrap();
    assert_eq!(info.distinct_support, 1);
    assert_eq!(info.observations, 6);
    assert!(!info.confirmed);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    rt.set_goal(&[0.7]).unwrap();
    assert_eq!(
        rt.observe_external(&[1.0]),
        Err(RuntimeError::InvalidRaster)
    );
    assert_eq!(rt.set_goal(&[f32::NAN]), Err(RuntimeError::InvalidRaster));
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    let outcome = rt
        .step(|_| None, |_| panic!("blocked action executed"))
        .unwrap();
    assert!(matches!(outcome, StepOutcome::Blocked(_)));
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    let outcome = rt.step(|_| Some(safe()), |_| Ok(vec![f32::NAN])).unwrap();
    assert!(matches!(outcome, StepOutcome::ExecutionFault(_)));
    assert!(rt.emergency_latched());
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn contradiction_revokes_only_the_changed_action_then_relearns_from_fresh_facts() {
    let mut world = World::new(0xABCDE, 3, 2);
    let mut rt = ScientificRuntime::new(organism(3, 2, Default::default())).unwrap();
    learn(&mut rt, &mut world, 80);
    let heldout = vec![0.21, 0.43, 0.87];
    let unaffected = rt.organism().phase_rule_predict(1, &heldout).unwrap();
    let old_revision = rt.organism().phase_rule_action_info(0).unwrap().revision;
    world.state = vec![17, 95, 210];
    let goal = World::raster(&world.transformed(&world.state, 0));
    rt.observe_external(&world.observe()).unwrap();
    rt.set_goal(&goal).unwrap();
    assert_eq!(rt.propose().unwrap().unwrap().action, 0);
    world.offsets[0][0] = (world.offsets[0][0] + 31) % PERIOD;
    rt.step(|_| Some(safe()), |a| Ok(world.act(a))).unwrap();
    let revised = rt.organism().phase_rule_action_info(0).unwrap();
    assert_eq!(revised.revision, old_revision + 1);
    assert_eq!(revised.distinct_support, 1);
    assert!(!revised.confirmed);
    assert!(rt.organism().phase_rule_predict(0, &heldout).is_none());
    assert_eq!(
        rt.organism().phase_rule_predict(1, &heldout).unwrap(),
        unaffected
    );
    learn(&mut rt, &mut world, 80);
    let expected = World::raster(&world.transformed(&[60, 120, 220], 0));
    let actual = rt
        .organism()
        .phase_rule_predict(0, &World::raster(&[60, 120, 220]))
        .unwrap();
    assert!(actual
        .sensory
        .iter()
        .zip(expected)
        .all(|(a, b)| (a - b).abs() < 0.0001));
    assert_eq!(
        rt.organism().phase_rule_action_info(1).unwrap().revision,
        unaffected.action_revision
    );
}

#[test]
fn acquired_physical_parameters_are_necessary_and_exact_restore_recovers_transfer() {
    let mut world = World::new(0x12345, 2, 1);
    let mut rt = ScientificRuntime::new(organism(2, 1, Default::default())).unwrap();
    learn(&mut rt, &mut world, 40);
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    let mut evo = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    let input = World::raster(&[34, 179]);
    let goal = World::raster(&world.transformed(&[34, 179], 0));
    let forecast = evo.phase_rule_predict(0, &input).unwrap();
    let synapse = evo.phase_rule_action_info(0).unwrap().synapses[0];
    let saved = evo
        .perturb_phase_native_synapse_for_control(synapse, 0.0, 0.0)
        .unwrap();
    assert!(evo.phase_rule_predict(0, &input).is_none());
    evo.set_planning_learning_enabled(false);
    evo.observe_initial_real(&input, false);
    assert!(evo.phase_rule_decision(&goal).is_none());
    evo.restore_phase_native_synapse_for_control(synapse, saved.clone());
    assert_eq!(evo.phase_rule_predict(0, &input).unwrap(), forecast);
    assert_eq!(
        evo.phase_rule_decision(&goal).unwrap().kind,
        PhaseRuleDecisionKind::GoalPlan
    );
    evo.perturb_phase_native_synapse_for_control(synapse, 1.0, std::f32::consts::PI)
        .unwrap();
    assert!((evo.phase_rule_predict(0, &input).unwrap().sensory[0] - goal[0]).abs() > 0.1);
    assert!(evo.phase_rule_decision(&goal).is_none());
    evo.restore_phase_native_synapse_for_control(synapse, saved);
    assert_eq!(evo.phase_rule_predict(0, &input).unwrap(), forecast);
}

#[test]
fn checkpoint_bounds_provenance_and_live_authority_survive_restore() {
    let mut world = World::new(0xFFEEDD, 3, 2);
    let mut rt = ScientificRuntime::new(organism(3, 2, Default::default())).unwrap();
    learn(&mut rt, &mut world, 80);
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    let valid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mutations: Vec<Box<dyn Fn(&mut serde_json::Value)>> = vec![
        Box::new(|v| {
            v["rules"]["actions"][0]["outputs"][0]["candidates"][0]["synapse"] = u64::MAX.into()
        }),
        Box::new(|v| {
            v["rules"]["actions"][0]["outputs"][0]["candidates"][0]["orientation"] = 0.into()
        }),
        Box::new(|v| v["rules"]["actions"][0]["evidence"][0]["sequence"] = 0.into()),
        Box::new(|v| v["rules"]["config"]["beam_width"] = u64::MAX.into()),
        Box::new(|v| v["rules"]["actions"][0]["outputs"][0]["cell"] = 0.into()),
        Box::new(|v| v["rules"]["factual_sequence"] = u64::MAX.into()),
    ];
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    for mutate in mutations {
        let mut corrupt = valid.clone();
        mutate(&mut corrupt);
        let corrupt = serde_json::to_vec(&corrupt).unwrap();
        assert!(EvoPhase::from_online_checkpoint(&corrupt).is_err());
        assert_eq!(
            rt.restore_online_checkpoint(&corrupt),
            Err(RuntimeError::InvalidCheckpoint)
        );
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
    }
    rt.set_goal(&vec![0.1234567; 3]).unwrap();
    rt.set_model_learning_enabled(false);
    // A frozen unsupported goal abstains before screening; use a real supported
    // goal to exercise an emergency stop at the live actuator boundary.
    let goal = World::raster(&world.transformed(&world.state, 0));
    rt.set_goal(&goal).unwrap();
    let mut stop = safe();
    stop.emergency_stop = true;
    rt.step(|_| Some(stop), |_| panic!("emergency ACT"))
        .unwrap();
    assert!(rt.emergency_latched());
    rt.restore_online_checkpoint(&bytes).unwrap();
    assert!(rt.emergency_latched());
    assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
    rt.observe_external(&world.observe()).unwrap();
    assert!(matches!(
        rt.step(|_| Some(safe()), |_| panic!("restore cleared stop"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn old_online_checkpoint_remains_readable_and_frozen_unknown_rules_abstain() {
    let mut evo = EvoPhase::new(EvoConfig::default());
    evo.enable_phase_native_planning(Default::default());
    assert!(evo.enable_phase_native_online_learning(Default::default()));
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&evo.online_checkpoint_bytes().unwrap()).unwrap();
    legacy["version"] = 1.into();
    legacy.as_object_mut().unwrap().remove("rules");
    let restored = EvoPhase::from_online_checkpoint(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert!(!restored.phase_rules_enabled());
    let mut rt = ScientificRuntime::new(organism(2, 2, Default::default())).unwrap();
    rt.observe_external(&[0.23, 0.45]).unwrap();
    rt.set_goal(&[0.67, 0.89]).unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn frozen_planning_abstains_at_its_budget_then_reuses_the_same_rule_with_more_budget() {
    let mut world = World {
        sources: vec![vec![0]],
        reflected: vec![vec![false]],
        offsets: vec![vec![17]],
        state: vec![13],
        factual_history: Vec::new(),
    };
    let config = PhaseRuleConfig {
        planning_depth: 1,
        ..Default::default()
    };
    let mut rt = ScientificRuntime::new(organism(1, 1, config)).unwrap();
    learn(&mut rt, &mut world, 10);
    rt.set_model_learning_enabled(false);
    world.state = vec![149];
    let input = world.observe();
    let goal = World::raster(&world.transformed(&world.transformed(&world.state, 0), 0));
    rt.observe_external(&input).unwrap();
    rt.set_goal(&goal).unwrap();
    assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
    let prior = rt.organism().phase_rule_predict(0, &input).unwrap();
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    checkpoint["rules"]["config"]["planning_depth"] = 2.into();
    checkpoint["rules"]["config"]["node_budget"] = 1.into();
    rt.restore_online_checkpoint(&serde_json::to_vec(&checkpoint).unwrap())
        .unwrap();
    rt.observe_external(&input).unwrap();
    assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
    checkpoint["rules"]["config"]["node_budget"] = 2.into();
    rt.restore_online_checkpoint(&serde_json::to_vec(&checkpoint).unwrap())
        .unwrap();
    assert_eq!(rt.organism().phase_rule_predict(0, &input).unwrap(), prior);
    assert_eq!(reach(&mut rt, &mut world, &goal, 2), 2);
}
