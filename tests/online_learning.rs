use aeterna_v1::carrier::{PhaseNativeConfig, PhaseOnlineConfig};
use aeterna_v1::scientific_runtime::{RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};

fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}

fn runtime(channels: usize, motors: usize, capacity: usize) -> ScientificRuntime {
    let mut evo = EvoPhase::new(EvoConfig {
        sensory_cells: channels,
        motor_cells: motors,
        dormant_cells: capacity * (motors + 2),
        weight_learning_rate: 1.0,
        phase_learning_rate: 1.0,
        min_recruit_support: 1,
        ..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig {
        horizon: 32,
        ..Default::default()
    });
    assert!(evo.enable_phase_native_online_learning(PhaseOnlineConfig {
        max_states: capacity
    }));
    ScientificRuntime::new(evo).unwrap()
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn shuffle<T>(&mut self, xs: &mut [T]) {
        for i in (1..xs.len()).rev() {
            let j = self.next() as usize % (i + 1);
            xs.swap(i, j);
        }
    }
}

// Only the external evaluator sees node indices, transitions, or motor roles.
// Connected random directed graphs have changing size, opaque observations,
// independently shuffled actions at every node, and cycles/self loops.
struct World {
    observations: Vec<Vec<f32>>,
    transitions: Vec<Vec<usize>>,
    state: usize,
}
impl World {
    fn new(seed: u64) -> Self {
        let mut rng = Rng(seed);
        let n = 5 + (rng.next() % 12) as usize;
        let mut observations = (0..n)
            .map(|i| {
                // Four generic bounded channels encode a shuffled binary pattern.
                (0..5)
                    .map(|bit| if (i >> bit) & 1 == 0 { 0.1 } else { 0.9 })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        rng.shuffle(&mut observations);
        let transitions = (0..n)
            .map(|i| {
                let mut edges = vec![(i + 1) % n, rng.next() as usize % n, i];
                rng.shuffle(&mut edges);
                edges
            })
            .collect();
        Self {
            observations,
            transitions,
            state: 0,
        }
    }

    fn reach(&mut self, rt: &mut ScientificRuntime, goal: usize, budget: usize) -> Option<usize> {
        rt.observe_external(&self.observations[self.state]).unwrap();
        rt.set_goal(&self.observations[goal]).unwrap();
        for steps in 0..=budget {
            if rt.goal_reached().unwrap() {
                return Some(steps);
            }
            if steps == budget {
                break;
            }
            match rt.step_unified(
                |_| Some(safe()),
                |action| {
                    self.state = self.transitions[self.state][action];
                    Ok((
                        self.observations[self.state].clone(),
                        f32::from(self.state == goal),
                    ))
                },
            ) {
                Ok(StepOutcome::Executed { .. }) => {}
                _ => return None,
            }
        }
        None
    }
}

#[test]
fn cold_learning_goal_switch_and_restart_across_unprepared_graphs() {
    let mut learned = 0;
    let mut reused = 0;
    for seed in 1..=64u64 {
        let mut world = World::new(seed * 0x9E37_79B9);
        let n = world.observations.len();
        let mut rt = runtime(5, 3, 64);
        assert_eq!(rt.organism().phase_native_receptor_count(), 0);
        assert_eq!(rt.organism().phase_native_circuits().len(), 0);
        assert!(
            world.reach(&mut rt, n - 1, n * n * 12).is_some(),
            "cold seed {seed}"
        );
        learned += 1;
        // New goals are requests, not motor instructions or factual tuition.
        for goal in [n / 2, 0] {
            assert!(
                world.reach(&mut rt, goal, n * n * 12).is_some(),
                "goal switch {seed}"
            );
        }
        rt.restart_cognition().unwrap();
        assert_eq!(
            rt.goal_reached(),
            Err(RuntimeError::FreshObservationRequired)
        );
        rt.set_model_learning_enabled(false);
        let fp = rt.organism().phase_native_learned_fingerprint();
        world.state = 0;
        assert!(
            world.reach(&mut rt, n - 1, n).is_some(),
            "frozen reuse seed {seed}"
        );
        assert_eq!(rt.organism().phase_native_learned_fingerprint(), fp);
        assert_eq!(rt.organism().planning_transition_count(), 0);
        reused += 1;
    }
    println!("ONLINE_DEVELOPMENT cold={learned}/64 frozen_restart={reused}/64");
}

#[test]
fn goals_invalid_inputs_freeze_and_blocked_actions_cannot_teach() {
    let mut rt = runtime(2, 2, 4);
    rt.set_goal(&[0.7, 0.9]).unwrap();
    assert_eq!(rt.organism().phase_native_receptor_count(), 0);
    assert_eq!(
        rt.observe_external(&[f32::NAN, 0.0]),
        Err(RuntimeError::InvalidRaster)
    );
    assert_eq!(rt.organism().phase_native_receptor_count(), 0);
    rt.observe_external(&[0.0, 0.0]).unwrap();
    let fp = rt.organism().phase_native_learned_fingerprint();
    let blocked = rt
        .step_unified(|_| None, |_| panic!("blocked callback executed"))
        .unwrap();
    assert!(matches!(blocked, StepOutcome::Blocked(_)));
    assert_eq!(rt.organism().phase_native_learned_fingerprint(), fp);
    rt.set_model_learning_enabled(false);
    let fp = rt.organism().phase_native_learned_fingerprint();
    assert_eq!(
        rt.observe_external(&[0.7, 0.9]),
        Err(RuntimeError::UnknownRepresentation)
    );
    assert_eq!(rt.organism().phase_native_learned_fingerprint(), fp);
}

#[test]
fn physical_successor_lesion_removes_frozen_online_plan_and_restore_recovers() {
    let mut world = World::new(149);
    let n = world.observations.len();
    let mut rt = runtime(5, 3, 64);
    world.reach(&mut rt, n - 1, n * n * 12).unwrap();
    let original = rt.organism().clone();
    let mut lesioned = original.clone();
    lesioned.set_planning_learning_enabled(false);
    let synapses = lesioned
        .phase_native_circuits()
        .iter()
        .map(|c| c.successor_synapse)
        .collect::<Vec<_>>();
    let saved = synapses
        .iter()
        .map(|&i| {
            (
                i,
                lesioned
                    .perturb_phase_native_synapse_for_control(i, 1.0, std::f32::consts::PI)
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let mut broken = ScientificRuntime::new(lesioned.clone()).unwrap();
    broken.set_model_learning_enabled(false);
    broken.observe_external(&world.observations[0]).unwrap();
    broken.set_goal(&world.observations[n - 1]).unwrap();
    assert_eq!(
        broken.propose_unified(),
        Err(RuntimeError::NoSupportedAction)
    );
    for (i, syn) in saved {
        lesioned.restore_phase_native_synapse_for_control(i, syn);
    }
    let mut restored = ScientificRuntime::new(lesioned).unwrap();
    restored.set_model_learning_enabled(false);
    world.state = 0;
    assert!(world.reach(&mut restored, n - 1, n).is_some());
}

#[test]
fn bounded_capacity_and_no_growth_are_reported_without_fabricated_state() {
    let mut rt = runtime(2, 2, 1);
    rt.observe_external(&[0.0, 0.0]).unwrap();
    assert_eq!(
        rt.observe_external(&[1.0, 1.0]),
        Err(RuntimeError::RepresentationCapacity)
    );
    assert_eq!(rt.organism().phase_native_receptor_count(), 1);
    let mut evo = runtime(2, 2, 4).organism().clone();
    evo.set_structural_growth_for_control(false);
    let mut no_growth = ScientificRuntime::new(evo).unwrap();
    assert_eq!(
        no_growth.observe_external(&[0.0, 0.0]),
        Err(RuntimeError::RepresentationCapacity)
    );
    assert_eq!(no_growth.organism().phase_native_receptor_count(), 0);
}

#[test]
fn persisted_knowledge_restores_and_malformed_snapshots_are_atomic_errors() {
    let mut world = World::new(719);
    let n = world.observations.len();
    let mut rt = runtime(5, 3, 64);
    world.reach(&mut rt, n - 1, n * n * 12).unwrap();
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    let fp = rt.organism().phase_native_learned_fingerprint();
    let loaded = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    assert_eq!(loaded.phase_native_learned_fingerprint(), fp);
    assert!(loaded.current_real().is_none());
    let mut recovered = ScientificRuntime::new(loaded).unwrap();
    recovered.set_model_learning_enabled(false);
    world.state = 0;
    assert!(world.reach(&mut recovered, n - 1, n).is_some());
    assert_eq!(recovered.organism().phase_native_learned_fingerprint(), fp);

    let frozen_bytes = recovered.organism().online_checkpoint_bytes().unwrap();
    let frozen = EvoPhase::from_online_checkpoint(&frozen_bytes).unwrap();
    let mut frozen_runtime = ScientificRuntime::new(frozen).unwrap();
    assert_eq!(
        frozen_runtime.observe_external(&[0.4; 5]),
        Err(RuntimeError::UnknownRepresentation),
        "constructing a runtime must honor the persisted freeze flag"
    );

    for bad in [
        bytes[..bytes.len() / 2].to_vec(),
        {
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            value["version"] = 999.into();
            serde_json::to_vec(&value).unwrap()
        },
        {
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            value["synapses"][0]["to"] = u64::MAX.into();
            serde_json::to_vec(&value).unwrap()
        },
        {
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            value["config"]["hdc_dim"] = u64::MAX.into();
            serde_json::to_vec(&value).unwrap()
        },
    ] {
        assert!(EvoPhase::from_online_checkpoint(&bad).is_err());
        assert_eq!(
            rt.restore_online_checkpoint(&bad),
            Err(RuntimeError::InvalidCheckpoint)
        );
        assert_eq!(rt.organism().phase_native_learned_fingerprint(), fp);
    }
}

#[test]
fn checkpoint_restore_preserves_live_emergency_latch_and_requires_fresh_sensing() {
    let mut world = World::new(331);
    let n = world.observations.len();
    let mut rt = runtime(5, 3, 64);
    world.reach(&mut rt, n - 1, n * n * 12).unwrap();
    let bytes = rt.organism().online_checkpoint_bytes().unwrap();
    rt.observe_external(&world.observations[0]).unwrap();
    rt.step_unified(
        |_| {
            Some(HumanProtectionEvidence {
                emergency_stop: true,
                ..safe()
            })
        },
        |_| panic!("emergency-stop callback executed"),
    )
    .unwrap();
    assert!(rt.emergency_latched());
    rt.restore_online_checkpoint(&bytes).unwrap();
    assert!(rt.emergency_latched());
    assert_eq!(
        rt.goal_reached(),
        Err(RuntimeError::FreshObservationRequired)
    );
    rt.observe_external(&world.observations[0]).unwrap();
    let outcome = rt
        .step_unified(|_| Some(safe()), |_| panic!("checkpoint cleared latch"))
        .unwrap();
    assert!(matches!(outcome, StepOutcome::Blocked(_)));
}

#[test]
fn factual_changed_transition_repairs_a_previously_acquired_route() {
    let mut rt = runtime(3, 2, 12);
    let observations = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let mut transitions = [[1, 2], [3, 0], [3, 0], [0, 0]];
    let mut state = 0;
    let run = |rt: &mut ScientificRuntime, state: &mut usize, transitions: &[[usize; 2]; 4]| {
        rt.observe_external(&observations[*state]).unwrap();
        rt.set_goal(&observations[3]).unwrap();
        for _ in 0..40 {
            if rt.goal_reached().unwrap() {
                return true;
            }
            rt.step(
                |_| Some(safe()),
                |action| {
                    *state = transitions[*state][action];
                    Ok(observations[*state].to_vec())
                },
            )
            .unwrap();
        }
        false
    };
    assert!(run(&mut rt, &mut state, &transitions));
    transitions[1][0] = 0;
    state = 0;
    assert!(run(&mut rt, &mut state, &transitions));
    assert!(rt
        .organism()
        .phase_native_circuits()
        .iter()
        .any(|c| c.revision > 0));
    state = 0;
    rt.set_model_learning_enabled(false);
    assert!(run(&mut rt, &mut state, &transitions));
}
