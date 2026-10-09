use aeterna_v1::carrier::{
    PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig, PhaseRuleConfig,
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
fn fresh(width: usize, motors: usize) -> EvoPhase {
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
    assert!(evo.enable_phase_rule_learning(PhaseRuleConfig::default()));
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

// Integer external dynamics and sensor masking are confined to the evaluator.
// No kind, permutation, source map, mask or hidden value enters the carrier.
struct World {
    state: Vec<u32>,
    kinds: Vec<usize>,
    stride: u32,
    history: Vec<Vec<u32>>,
}
impl World {
    fn new(seed: u64, width: usize) -> Self {
        let mut rng = Rng(seed);
        let mut kinds = vec![0, 1, 2];
        for i in (1..3).rev() {
            kinds.swap(i, rng.next() as usize % (i + 1));
        }
        Self {
            state: (0..width).map(|_| rng.next() % PERIOD).collect(),
            kinds,
            stride: 17,
            history: Vec::new(),
        }
    }
    fn raster(state: &[u32]) -> Vec<f32> {
        state.iter().map(|&x| x as f32 / PERIOD as f32).collect()
    }
    fn full(&self) -> Vec<f32> {
        Self::raster(&self.state)
    }
    fn partial(&self) -> Vec<Option<f32>> {
        (0..self.state.len())
            .map(|j| (j == 0).then_some(self.state[j] as f32 / PERIOD as f32))
            .collect()
    }
    fn transform(&self, input: &[u32], action: usize) -> Vec<u32> {
        let mut state = input.to_vec();
        match self.kinds[action] {
            0 => {}
            1 => state[0] = (state[0] + self.stride) % PERIOD,
            2 => state.rotate_left(1),
            _ => unreachable!(),
        }
        state
    }
    fn act(&mut self, action: usize) {
        self.history.push(self.state.clone());
        self.state = self.transform(&self.state, action);
        self.history.push(self.state.clone());
    }
}
fn masks_ready(rt: &ScientificRuntime) -> bool {
    (0..rt.organism().config().motor_cells)
        .all(|a| rt.organism().phase_partial_mask_info(a).unwrap().confirmed)
}
fn trained(world: &mut World) -> ScientificRuntime {
    let mut rt = ScientificRuntime::new(fresh(world.state.len(), 3)).unwrap();
    rt.observe_external(&world.full()).unwrap();
    rt.set_goal(&vec![0.1234567; world.state.len()]).unwrap();
    for _ in 0..80 {
        if (0..3).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed) {
            break;
        }
        let outcome = rt
            .step(
                |_| Some(safe()),
                |a| {
                    world.act(a);
                    Ok(world.full())
                },
            )
            .unwrap();
        assert!(matches!(outcome, StepOutcome::Executed { .. }));
    }
    assert!((0..3).all(|a| rt.organism().phase_rule_action_info(a).unwrap().confirmed));
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    assert!(evo.enable_phase_partial_observation(PhasePartialConfig::default()));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    let rule_counts = (0..3)
        .map(|a| {
            rt.organism()
                .phase_rule_action_info(a)
                .unwrap()
                .observations
        })
        .collect::<Vec<_>>();
    rt.observe_external_partial(&world.partial()).unwrap();
    let mut goal = vec![None; world.state.len()];
    goal[0] = Some(0.1234567);
    rt.set_partial_goal(&goal).unwrap();
    for _ in 0..30 {
        if masks_ready(&rt) {
            break;
        }
        let outcome = rt
            .step_partial(
                |_| Some(safe()),
                |a| {
                    world.act(a);
                    Ok((world.partial(), 0.0))
                },
            )
            .unwrap();
        assert!(matches!(outcome, StepOutcome::Executed { .. }));
    }
    assert!(masks_ready(&rt));
    assert_eq!(
        (0..3)
            .map(|a| rt
                .organism()
                .phase_rule_action_info(a)
                .unwrap()
                .observations)
            .collect::<Vec<_>>(),
        rule_counts,
        "inferred hidden values were used as raw tuition"
    );
    rt
}
fn reach(
    rt: &mut ScientificRuntime,
    world: &mut World,
    goal: &[Option<f32>],
    budget: usize,
) -> usize {
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(goal).unwrap();
    let mut information = 0;
    for n in 0..=budget {
        if rt.goal_reached().unwrap() {
            return information;
        }
        assert!(n < budget, "partial goal budget exhausted");
        let outcome = rt
            .step_partial(
                |_| Some(safe()),
                |a| {
                    world.act(a);
                    Ok((world.partial(), 0.0))
                },
            )
            .unwrap();
        if let StepOutcome::Executed {
            proposal, learned, ..
        } = outcome
        {
            if proposal.mode == ReasoningMode::PartialInformationGathering {
                information += 1;
            }
            assert!(!learned, "frozen model learned from partial evaluation");
        } else {
            panic!("{outcome:?}");
        }
    }
    unreachable!()
}

#[test]
fn frozen_partial_goals_transfer_in_32_opaque_worlds_and_after_restart() {
    let mut goals = 0;
    let mut information = 0;
    for seed in 1..=32 {
        let width = 2 + seed as usize % 3;
        let mut world = World::new(seed * 0x9E3779B9, width);
        let rt = trained(&mut world);
        let acquisition = world.history.clone();
        let bytes = rt.organism().online_checkpoint_bytes().unwrap();
        let evo = EvoPhase::from_online_checkpoint(&bytes).unwrap();
        assert!(
            evo.phase_partial_belief().is_none(),
            "REAL/beliefs replayed from checkpoint"
        );
        let mut rt = ScientificRuntime::new(evo).unwrap();
        rt.set_model_learning_enabled(false);
        let fingerprint = rt.organism().phase_native_learned_fingerprint();
        let rotate = world.kinds.iter().position(|&k| k == 2).unwrap();
        let shift = world.kinds.iter().position(|&k| k == 1).unwrap();
        let mut rng = Rng(seed ^ 0xAABBCCDD);
        for _ in 0..6 {
            let target = loop {
                world.state = (0..width).map(|_| rng.next() % PERIOD).collect();
                if acquisition.contains(&world.state) {
                    continue;
                }
                let mut target = world.state.clone();
                for _ in 0..width - 1 {
                    target = world.transform(&target, rotate);
                }
                for _ in 0..2 {
                    target = world.transform(&target, shift);
                }
                if target[0] != world.state[0] {
                    break target;
                }
            };
            let mut goal = vec![None; width];
            goal[0] = Some(target[0] as f32 / PERIOD as f32);
            information += reach(&mut rt, &mut world, &goal, 20);
            assert_eq!(
                rt.organism().phase_native_learned_fingerprint(),
                fingerprint
            );
            assert!(
                rt.organism().current_real().is_none(),
                "partial data zero-imputed into REAL"
            );
            goals += 1;
        }
        rt.restart_cognition().unwrap();
        assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
        assert!(rt.organism().phase_partial_belief().is_none());
    }
    println!("partial_worlds=32 frozen_goals={goals} information_actions={information}");
    assert_eq!(goals, 192);
    assert!(information >= 32);
}

#[test]
fn acquired_sensing_connection_is_necessary_and_exact_restore_recovers_goal() {
    let mut world = World::new(0x12345, 2);
    world.kinds = vec![0, 1, 2];
    let rt = trained(&mut world);
    let mut evo =
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
            .unwrap();
    let connection = evo.phase_rule_action_info(2).unwrap().synapses[0];
    let saved = evo
        .perturb_phase_native_synapse_for_control(connection, 0.0, 0.0)
        .unwrap();
    let bytes = evo.online_checkpoint_bytes().unwrap();
    let mut lesioned = ScientificRuntime::new(evo).unwrap();
    lesioned.set_model_learning_enabled(false);
    world.state = vec![0, 193];
    let goal = vec![Some(227.0 / PERIOD as f32), None];
    lesioned.observe_external_partial(&world.partial()).unwrap();
    lesioned.set_partial_goal(&goal).unwrap();
    assert_eq!(lesioned.propose(), Err(RuntimeError::NoSupportedAction));
    let mut restored = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    restored.restore_phase_native_synapse_for_control(connection, saved);
    let mut restored = ScientificRuntime::new(restored).unwrap();
    restored.set_model_learning_enabled(false);
    assert_eq!(reach(&mut restored, &mut world, &goal, 3), 1);
    assert_eq!(world.state[0], 227);
}

#[test]
fn zero_is_observable_and_inferred_hidden_goal_cannot_claim_factual_success() {
    let mut world = World::new(0x12345, 2);
    world.kinds = vec![0, 1, 2];
    let mut rt = trained(&mut world);
    rt.set_model_learning_enabled(false);
    world.state = vec![149, 0];
    let goal = vec![Some(0.0), None];
    assert_eq!(reach(&mut rt, &mut world, &goal, 1), 1);
    assert_eq!(
        rt.organism().phase_partial_belief().unwrap().factual,
        vec![Some(0.0), None]
    );
    world.state = vec![149, 0];
    // Restore the full-observation v2 model, then acquire a fresh mask from
    // actual execution. The known hidden zero must retain IMAGINED authority.
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    checkpoint.as_object_mut().unwrap().remove("partial");
    checkpoint["version"] = 2.into();
    let mut evo =
        EvoPhase::from_online_checkpoint(&serde_json::to_vec(&checkpoint).unwrap()).unwrap();
    assert!(evo.enable_phase_partial_observation(Default::default()));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    rt.set_model_learning_enabled(true);
    rt.observe_external(&world.full()).unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    rt.step_partial(
        |p| {
            assert_eq!(p.action, 0);
            Some(safe())
        },
        |a| {
            world.act(a);
            Ok((world.partial(), 0.0))
        },
    )
    .unwrap();
    let belief = rt.organism().phase_partial_belief().unwrap();
    assert_eq!(belief.factual[1], None);
    assert_eq!(belief.hypotheses[0][1], Some(0.0));
    rt.set_partial_goal(&[None, Some(0.0)]).unwrap();
    assert!(
        !rt.goal_reached().unwrap(),
        "derived hidden zero became factual success"
    );
}

#[test]
fn finite_alternatives_and_capacity_widening_never_teach_hidden_values() {
    for maximum in [1, 32] {
        let mut rt = ScientificRuntime::new(fresh(1, 1)).unwrap();
        rt.observe_external(&[0.1]).unwrap();
        rt.set_goal(&[0.1234567]).unwrap();
        rt.step(|_| Some(safe()), |_| Ok(vec![0.2])).unwrap();
        let mut evo =
            EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
                .unwrap();
        assert!(evo.enable_phase_partial_observation(PhasePartialConfig {
            max_hypotheses: maximum,
            ..Default::default()
        }));
        let mut rt = ScientificRuntime::new(evo).unwrap();
        rt.observe_external_partial(&[Some(0.3)]).unwrap();
        rt.set_partial_goal(&[Some(0.1234567)]).unwrap();
        rt.step_partial(|_| Some(safe()), |_| Ok((vec![None], 0.0)))
            .unwrap();
        let belief = rt.organism().phase_partial_belief().unwrap();
        assert_eq!(belief.factual, vec![None]);
        assert_eq!(belief.hypotheses_authority, Authority::Imagined);
        if maximum == 1 {
            assert!(belief.widened);
            assert_eq!(belief.hypotheses, vec![vec![None]]);
        } else {
            assert!(!belief.widened);
            assert_eq!(belief.hypotheses.len(), 2);
            assert!(belief
                .hypotheses
                .iter()
                .any(|row| (row[0].unwrap() - 0.4).abs() < 0.0001));
        }
        assert_eq!(
            rt.organism()
                .phase_rule_action_info(0)
                .unwrap()
                .observations,
            1
        );
        rt.set_partial_goal(&[Some(0.4)]).unwrap();
        assert!(!rt.goal_reached().unwrap());
        rt.set_model_learning_enabled(false);
        assert_eq!(rt.propose(), Err(RuntimeError::NoSupportedAction));
    }
}

#[test]
fn visible_contradiction_suspends_the_rule_until_fully_factual_reconfirmation() {
    let mut world = World::new(0x13579, 2);
    world.kinds = vec![0, 1, 2];
    let mut rt = trained(&mut world);
    let revision = rt.organism().phase_rule_action_info(1).unwrap().revision;
    world.state = vec![0, 193];
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(17.0 / PERIOD as f32), None])
        .unwrap();
    assert_eq!(rt.propose().unwrap().unwrap().action, 1);
    world.stride = 31;
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            world.act(a);
            Ok((world.partial(), 0.0))
        },
    )
    .unwrap();
    assert!(!rt.organism().phase_rule_action_info(1).unwrap().confirmed);
    assert_eq!(
        rt.organism().phase_rule_action_info(1).unwrap().revision,
        revision + 1
    );
    assert!(rt.organism().phase_rule_predict(1, &[0.2, 0.8]).is_none());
    assert!(
        EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap()).is_ok()
    );
    let mut rng = Rng(0xABCDEF);
    for _ in 0..20 {
        if rt.organism().phase_rule_action_info(1).unwrap().confirmed {
            break;
        }
        world.state = (0..2).map(|_| rng.next() % PERIOD).collect();
        rt.observe_external(&world.full()).unwrap();
        rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
        rt.step(
            |_| Some(safe()),
            |a| {
                world.act(a);
                Ok(world.full())
            },
        )
        .unwrap();
    }
    assert!(rt.organism().phase_rule_action_info(1).unwrap().confirmed);
    assert_eq!(
        rt.organism().phase_rule_action_info(1).unwrap().revision,
        revision + 1
    );
    assert_eq!(rt.organism().phase_rule_action_info(2).unwrap().revision, 0);
    let prediction = rt.organism().phase_rule_predict(1, &[0.0, 0.8]).unwrap();
    assert!((prediction.sensory[0] - 31.0 / PERIOD as f32).abs() < 0.0001);
}

#[test]
fn frozen_mismatch_invalidates_live_predictions_without_rewriting_knowledge() {
    let mut world = World::new(0x13579, 2);
    world.kinds = vec![0, 1, 2];
    let mut rt = trained(&mut world);
    rt.set_model_learning_enabled(false);
    world.state = vec![0, 193];
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(17.0 / PERIOD as f32), None])
        .unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.stride = 31;
    rt.step_partial(
        |_| Some(safe()),
        |a| {
            world.act(a);
            Ok((world.partial(), 0.0))
        },
    )
    .unwrap();
    assert_eq!(
        rt.organism()
            .phase_partial_belief()
            .unwrap()
            .invalidated_rules,
        vec![1]
    );
    assert!(!rt.goal_reached().unwrap());
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn changed_visibility_revokes_the_live_affordance_without_inventing_observation() {
    let mut world = World::new(0x13579, 2);
    world.kinds = vec![0, 1, 2];
    let mut rt = trained(&mut world);
    rt.set_model_learning_enabled(false);
    world.state = vec![0, 193];
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(17.0 / PERIOD as f32), None])
        .unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let old_mask = rt.organism().phase_partial_mask_info(1).unwrap();
    rt.step_partial(
        |p| {
            assert_eq!(p.action, 1);
            Some(safe())
        },
        |a| {
            world.act(a);
            // The previously visible channel now produces no measurement.
            Ok((vec![None, None], 0.0))
        },
    )
    .unwrap();
    let belief = rt.organism().phase_partial_belief().unwrap();
    assert_eq!(belief.factual, vec![None, None]);
    assert!(belief.invalidated_rules.is_empty());
    assert_eq!(belief.invalidated_masks, vec![1]);
    assert!(!rt.goal_reached().unwrap());
    let proposal = rt.propose().unwrap().unwrap();
    assert_ne!(proposal.action, 1, "used the disproved visibility mask");
    assert_eq!(rt.organism().phase_partial_mask_info(1).unwrap(), old_mask);
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn different_motor_masks_are_acquired_and_choose_the_needed_measurement() {
    let mut world = World::new(0x13579, 2);
    world.kinds = vec![0, 1, 2];
    let rt = trained(&mut world);
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    checkpoint["partial"] = serde_json::Value::Null;
    let mut evo =
        EvoPhase::from_online_checkpoint(&serde_json::to_vec(&checkpoint).unwrap()).unwrap();
    assert!(evo.enable_phase_partial_observation(Default::default()));
    let mut rt = ScientificRuntime::new(evo).unwrap();
    let rule_counts = (0..3)
        .map(|a| {
            rt.organism()
                .phase_rule_action_info(a)
                .unwrap()
                .observations
        })
        .collect::<Vec<_>>();
    rt.observe_external_partial(&[None, None]).unwrap();
    rt.set_partial_goal(&[None, Some(0.1234567)]).unwrap();
    // Only the external evaluator knows which motor reveals which channel.
    let masked_post = |world: &World, action| {
        let values = world.full();
        match action {
            0 => vec![Some(values[0]), None],
            1 => vec![None, Some(values[1])],
            2 => vec![None, None],
            _ => unreachable!(),
        }
    };
    for _ in 0..12 {
        if masks_ready(&rt) {
            break;
        }
        rt.step_partial(
            |_| Some(safe()),
            |a| {
                world.act(a);
                Ok((masked_post(&world, a), 0.0))
            },
        )
        .unwrap();
    }
    assert!(masks_ready(&rt));
    for (action, mask) in [vec![true, false], vec![false, true], vec![false, false]]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            rt.organism()
                .phase_partial_mask_info(action)
                .unwrap()
                .visible,
            mask
        );
        assert_eq!(
            rt.organism()
                .phase_rule_action_info(action)
                .unwrap()
                .observations,
            rule_counts[action]
        );
    }
    let evo = EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap())
        .unwrap();
    let mut rt = ScientificRuntime::new(evo).unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    world.state = vec![193, 0];
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[None, Some(0.0)]).unwrap();
    assert!(!rt.goal_reached().unwrap());
    let outcome = rt
        .step_partial(
            |proposal| {
                assert_eq!(proposal.action, 1);
                assert_eq!(proposal.mode, ReasoningMode::PartialInformationGathering);
                Some(safe())
            },
            |a| {
                world.act(a);
                Ok((masked_post(&world, a), 0.0))
            },
        )
        .unwrap();
    assert!(matches!(
        outcome,
        StepOutcome::Executed { learned: false, .. }
    ));
    assert!(rt.goal_reached().unwrap());
    assert_eq!(
        rt.organism().phase_partial_belief().unwrap().factual,
        vec![None, Some(0.0)]
    );
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn legacy_v2_and_mode_restore_preserve_goals_without_imputing_sparse_constraints() {
    let mut world = World::new(0x13579, 2);
    let mut partial_rt = trained(&mut world);
    let partial_bytes = partial_rt.organism().online_checkpoint_bytes().unwrap();
    let mut legacy: serde_json::Value = serde_json::from_slice(&partial_bytes).unwrap();
    legacy.as_object_mut().unwrap().remove("partial");
    legacy["version"] = 2.into();
    for action in legacy["rules"]["actions"].as_array_mut().unwrap() {
        action.as_object_mut().unwrap().remove("suspended");
    }
    let legacy_bytes = serde_json::to_vec(&legacy).unwrap();
    let legacy_evo = EvoPhase::from_online_checkpoint(&legacy_bytes).unwrap();
    assert!(!legacy_evo.phase_partial_enabled());
    assert!((0..3).all(|a| legacy_evo.phase_rule_action_info(a).unwrap().confirmed));

    partial_rt.set_partial_goal(&[Some(0.2), None]).unwrap();
    let fingerprint = partial_rt.organism().phase_native_learned_fingerprint();
    let belief = partial_rt.organism().phase_partial_belief();
    assert_eq!(
        partial_rt.restore_online_checkpoint(&legacy_bytes),
        Err(RuntimeError::InvalidCheckpoint)
    );
    assert_eq!(partial_rt.organism().phase_partial_belief(), belief);
    assert_eq!(
        partial_rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );

    let mut rt = ScientificRuntime::new(legacy_evo).unwrap();
    rt.set_goal(&[0.2, 0.8]).unwrap();
    rt.restore_online_checkpoint(&partial_bytes).unwrap();
    assert!(rt.organism().phase_partial_enabled());
    assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
    rt.observe_external_partial(&[Some(0.2), None]).unwrap();
    assert!(
        !rt.goal_reached().unwrap(),
        "dropped the hidden goal constraint"
    );
    rt.observe_external(&[0.2, 0.8]).unwrap();
    assert!(rt.goal_reached().unwrap());
    rt.restore_online_checkpoint(&legacy_bytes).unwrap();
    assert!(!rt.organism().phase_partial_enabled());
    assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
    rt.observe_external(&[0.2, 0.8]).unwrap();
    assert!(rt.goal_reached().unwrap());
}

#[test]
fn checkpoint_validates_masks_excludes_beliefs_and_preserves_live_stop() {
    let mut world = World::new(0x2468, 2);
    world.kinds = vec![0, 1, 2];
    let mut rt = trained(&mut world);
    rt.set_model_learning_enabled(false);
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    let valid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(valid["partial"].get("episode").is_none());
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mutations: Vec<Box<dyn Fn(&mut serde_json::Value)>> = vec![
        Box::new(|v| v["partial"]["config"]["max_hypotheses"] = u64::MAX.into()),
        Box::new(|v| v["partial"]["masks"][0]["visible"] = serde_json::json!([true])),
        Box::new(|v| v["partial"]["masks"][0]["source_ids"][0] = 0.into()),
        Box::new(|v| v["partial"]["episode"] = serde_json::json!({"factual":[0,0]})),
    ];
    for mutate in mutations {
        let mut value = valid.clone();
        mutate(&mut value);
        assert_eq!(
            rt.restore_online_checkpoint(&serde_json::to_vec(&value).unwrap()),
            Err(RuntimeError::InvalidCheckpoint)
        );
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
    }
    world.state = vec![0, 193];
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(227.0 / PERIOD as f32), None])
        .unwrap();
    let mut stop = safe();
    stop.emergency_stop = true;
    rt.step_partial(|_| Some(stop), |_| panic!("stopped actuator executed"))
        .unwrap();
    assert!(rt.emergency_latched());
    rt.restore_online_checkpoint(&bytes).unwrap();
    assert_eq!(rt.propose(), Err(RuntimeError::FreshObservationRequired));
    assert!(rt.organism().phase_partial_belief().is_none());
    rt.observe_external_partial(&world.partial()).unwrap();
    assert!(matches!(
        rt.step_partial(|_| Some(safe()), |_| panic!("checkpoint reset stop"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn blocked_actions_and_bad_partial_post_cannot_teach_or_fill_unknown_channels() {
    let mut world = World::new(0x12345, 2);
    world.kinds = vec![0, 1, 2];
    let mut rt = trained(&mut world);
    world.state = vec![0, 193];
    rt.observe_external_partial(&world.partial()).unwrap();
    rt.set_partial_goal(&[Some(227.0 / PERIOD as f32), None])
        .unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let belief = rt.organism().phase_partial_belief().unwrap();
    assert_eq!(
        rt.set_partial_goal(&[None, None]),
        Err(RuntimeError::InvalidRaster)
    );
    assert_eq!(
        rt.observe_external_partial(&[Some(1.0), None]),
        Err(RuntimeError::InvalidRaster)
    );
    assert!(matches!(
        rt.step_partial(|_| None, |_| panic!("blocked callback executed"))
            .unwrap(),
        StepOutcome::Blocked(_)
    ));
    assert_eq!(rt.organism().phase_partial_belief().unwrap(), belief);
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    assert!(matches!(
        rt.step_partial(|_| Some(safe()), |_| Ok((vec![Some(f32::NAN), None], 0.0)))
            .unwrap(),
        StepOutcome::ExecutionFault(_)
    ));
    assert!(rt.emergency_latched());
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}
