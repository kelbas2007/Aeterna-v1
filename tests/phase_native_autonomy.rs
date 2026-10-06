//! P3 evaluator: the organism chooses acquisition actions itself.
//! Hidden transition laws exist only in this test-side world.
use aeterna_v1::{EvoConfig, EvoPhase, RasterFieldConfig};
use aeterna_v1::carrier::{PhaseNativeConfig, PhaseNativeCheckpoint};

const ACQUISITION_BUDGET: usize = 60;
const REVISION_BUDGET: usize = 40;

#[derive(Debug, Clone)]
struct World {
    length: usize,
    advance: Vec<usize>,
    detour_at: usize,
    detour_action: usize,
}

impl World {
    fn detour_state(&self) -> usize {
        self.length + 1
    }

    fn step(&self, state: usize, action: usize, revised: bool) -> (usize, f32) {
        assert!(action < 3);
        if state == self.length {
            return (self.length, 1.0);
        }

        let next = if revised && state == self.detour_state() {
            if action == self.detour_action {
                self.detour_at + 1
            } else {
                0
            }
        } else if state < self.length {
            let expected = self.advance[state];
            if revised && state == self.detour_at && action == expected {
                self.detour_state()
            } else if action == expected {
                state + 1
            } else {
                0
            }
        } else {
            0
        };

        let value = if next == self.length { 1.0 } else { 0.0 };
        (next, value)
    }
}

fn worlds() -> Vec<World> {
    (0..12usize)
        .map(|index| {
            let length = 3 + index % 3;
            let advance = (0..length)
                .map(|stage| (index + stage * 2 + 1) % 3)
                .collect::<Vec<_>>();
            let detour_at = 1 + (index % (length - 1));
            let detour_action = (advance[detour_at] + 1 + (index % 2)) % 3;
            World {
                length,
                advance,
                detour_at,
                detour_action,
            }
        })
        .collect()
}

fn raster(state: usize, x: usize, y: usize) -> Vec<f32> {
    let deltas = [
        (1usize, 0usize),
        (2, 0),
        (3, 0),
        (4, 0),
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 1),
        (1, 2),
        (2, 1),
    ];
    let (dx, dy) = deltas[state];
    assert!(x + dx < 12 && y + dy < 12);
    let mut values = vec![0.0; 144];
    values[y * 12 + x] = 1.0;
    values[(y + dy) * 12 + x + dx] = 1.0;
    values
}

fn carrier(mode: &str) -> EvoPhase {
    let mut cfg = EvoConfig {
        sensory_cells: 144,
        motor_cells: 3,
        dormant_cells: 96,
        hdc_dim: 192,
        weight_learning_rate: 1.0,
        phase_learning_rate: 1.0,
        min_recruit_support: 1,
        ..EvoConfig::default()
    };
    match mode {
        "zero_phase" => cfg.phase_learning_rate = 0.0,
        "zero_weight" => cfg.weight_learning_rate = 0.0,
        "no_growth" => cfg.structural_growth_enabled = false,
        _ => {}
    }

    let mut evo = EvoPhase::new(cfg);
    let mut field = RasterFieldConfig::for_raster(12, 12, 3);
    field.learning_enabled = false;
    field.readout_enabled = false;
    evo.attach_raster_field(field);
    evo.enable_phase_native_planning(PhaseNativeConfig {
        horizon: 8,
        ..PhaseNativeConfig::default()
    });
    if mode == "no_learning" {
        evo.set_planning_learning_enabled(false);
    }
    evo
}

#[derive(Debug)]
struct AcquireResult {
    reward: bool,
    interactions: usize,
    final_state: usize,
}

fn autonomous_acquire(
    evo: &mut EvoPhase,
    world: &World,
    revised: bool,
    budget: usize,
    direct_only: bool,
) -> AcquireResult {
    let mut state = 0usize;
    evo.observe_initial_real(&raster(state, 0, 0), false);

    for interaction in 1..=budget {
        let action = if direct_only {
            evo.choose_phase_native_direct_exploration_action()
        } else {
            evo.choose_phase_native_autonomous_action()
        };
        let Some(action) = action else {
            return AcquireResult {
                reward: false,
                interactions: interaction - 1,
                final_state: state,
            };
        };

        let (next, value) = world.step(state, action, revised);
        let post = raster(next, 0, 0);
        assert!(
            evo.observe_phase_native_action_result(action, &post, value).is_some(),
            "every factual interaction must enter the P2 learning loop"
        );
        state = next;

        if value >= 1.0 {
            return AcquireResult {
                reward: true,
                interactions: interaction,
                final_state: state,
            };
        }
    }

    AcquireResult {
        reward: false,
        interactions: budget,
        final_state: state,
    }
}

fn exploit(
    evo: &mut EvoPhase,
    world: &World,
    revised: bool,
    x: usize,
    y: usize,
    budget: usize,
) -> (bool, usize) {
    let mut state = 0usize;
    evo.observe_initial_real(&raster(state, x, y), false);

    for interaction in 1..=budget {
        let Some((decision, prediction)) = evo.choose_phase_native_action_with_prediction() else {
            return (false, interaction - 1);
        };
        assert!(prediction.confidence > 0.0);
        let (next, value) = world.step(state, decision.first_action, revised);
        state = next;
        evo.observe_initial_real(&raster(state, x, y), value >= 1.0);
        if value >= 1.0 {
            return (true, interaction);
        }
    }
    (false, budget)
}

fn random_acquire(world: &World, seed: u64, budget: usize) -> (bool, usize) {
    let mut x = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut state = 0usize;
    for interaction in 1..=budget {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        let action = (x.wrapping_mul(0x2545_F491_4F6C_DD1D) % 3) as usize;
        let (next, value) = world.step(state, action, false);
        state = next;
        if value >= 1.0 {
            return (true, interaction);
        }
    }
    (false, budget)
}

fn restore_into_new_carrier(checkpoint: PhaseNativeCheckpoint) -> EvoPhase {
    let mut restarted = carrier("native");
    assert!(restarted.restore_phase_native_checkpoint(checkpoint));
    restarted
}

#[test]
fn p3_cold_organism_autonomously_acquires_goal_model_and_survives_restart() {
    let mut full_success = 0usize;
    let mut direct_success = 0usize;
    let mut random_success = 0usize;
    let mut total_interactions = 0usize;
    let mut revision_interactions = 0usize;

    for (index, world) in worlds().into_iter().enumerate() {
        let mut evo = carrier("native");
        assert_eq!(evo.phase_native_circuits().len(), 0);
        assert_eq!(evo.phase_native_receptor_count(), 0);
        assert_eq!(evo.planning_transition_count(), 0);

        let acquired = autonomous_acquire(
            &mut evo,
            &world,
            false,
            ACQUISITION_BUDGET,
            false,
        );
        assert!(acquired.reward, "world {index} failed cold autonomous acquisition: {world:?}");
        assert_eq!(acquired.final_state, world.length);
        assert!(acquired.interactions <= ACQUISITION_BUDGET);
        assert!(evo.phase_native_circuits().len() >= world.length);
        assert!(evo.phase_native_receptor_count() >= world.length + 1);
        assert_eq!(evo.planning_transition_count(), 0);
        total_interactions += acquired.interactions;

        let learned_fingerprint = evo.phase_native_learned_fingerprint();
        let checkpoint = evo.phase_native_checkpoint().expect("P3 checkpoint");
        let mut restarted = restore_into_new_carrier(checkpoint);
        assert_eq!(restarted.phase_native_learned_fingerprint(), learned_fingerprint);
        assert!(restarted.current_real().is_none(), "REAL must not be restored from checkpoint");

        evo.set_planning_learning_enabled(false);
        restarted.set_planning_learning_enabled(false);
        let heldout_x = 4 + (index % 2);
        let heldout_y = 4 + ((index / 2) % 2);
        let solved = exploit(&mut evo, &world, false, heldout_x, heldout_y, world.length + 1);
        let restarted_solved =
            exploit(&mut restarted, &world, false, heldout_x, heldout_y, world.length + 1);
        assert_eq!(solved.0, true, "frozen acquired model must solve held-out translation");
        assert_eq!(restarted_solved.0, true, "restored model must solve held-out translation");
        assert_eq!(solved.1, world.length);
        assert_eq!(restarted_solved.1, world.length);
        full_success += 1;

        // A local-only exploration control gets stuck after current-state
        // unknowns are exhausted and cannot deliberately return to the deeper frontier.
        let mut direct = carrier("native");
        let direct_result = autonomous_acquire(
            &mut direct,
            &world,
            false,
            ACQUISITION_BUDGET,
            true,
        );
        direct_success += usize::from(direct_result.reward);

        random_success += usize::from(random_acquire(
            &world,
            0xA37E_0000 + index as u64,
            ACQUISITION_BUDGET,
        ).0);

        // Plasticity/capacity controls receive the same world only through
        // their own selected physical actions; no transition tuples are injected.
        for mode in ["no_learning", "no_growth", "zero_phase", "zero_weight"] {
            let mut control = carrier(mode);
            let result = autonomous_acquire(
                &mut control,
                &world,
                false,
                ACQUISITION_BUDGET,
                false,
            );
            assert!(
                !result.reward,
                "{mode} unexpectedly matched full P3 on world {index}: {world:?}"
            );
        }

        // Revision starts from the actually acquired organism, not a new curriculum.
        let mut revision = evo.clone();
        revision.set_planning_learning_enabled(true);

        let unchanged_state = if world.detour_at == 0 { 1 } else { 0 };
        let unchanged_action = world.advance[unchanged_state];
        let unchanged_before = revision
            .imagine_phase_native_actions(
                &raster(unchanged_state, 0, 0),
                &[unchanged_action],
            )
            .into_iter()
            .next()
            .expect("unchanged transition prediction before revision")
            .sensory;

        // Frozen stale model follows the old route into the unseen detour and
        // cannot acquire a continuation there.
        let mut frozen = evo.clone();
        frozen.set_planning_learning_enabled(false);
        let frozen_revised = exploit(
            &mut frozen,
            &world,
            true,
            3,
            4,
            world.length + 3,
        );
        assert!(!frozen_revised.0, "frozen stale model must not solve changed-law detour");

        let repaired = autonomous_acquire(
            &mut revision,
            &world,
            true,
            REVISION_BUDGET,
            false,
        );
        assert!(
            repaired.reward,
            "world {index} failed autonomous changed-law repair: {world:?}"
        );
        revision_interactions += repaired.interactions;

        revision.set_planning_learning_enabled(false);
        let revised_goal = exploit(
            &mut revision,
            &world,
            true,
            5,
            3,
            world.length + 3,
        );
        assert!(revised_goal.0, "revised organism must regain factual goal");

        let unchanged_after = revision
            .imagine_phase_native_actions(
                &raster(unchanged_state, 0, 0),
                &[unchanged_action],
            )
            .into_iter()
            .next()
            .expect("unchanged transition prediction after revision")
            .sensory;
        let unchanged_drift = unchanged_before
            .iter()
            .zip(&unchanged_after)
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>() / unchanged_before.len() as f32;
        let (expected_next, _) = world.step(unchanged_state, unchanged_action, true);
        let expected = raster(expected_next, 0, 0);
        let retained_error = unchanged_after
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>() / expected.len() as f32;
        assert!(
            retained_error < 0.01,
            "unrelated transition no longer predicts its factual successor: error={retained_error} drift={unchanged_drift}"
        );

        let revised_fp = revision.phase_native_learned_fingerprint();
        let cp = revision.phase_native_checkpoint().expect("revised checkpoint");
        let mut revised_restart = restore_into_new_carrier(cp);
        assert_eq!(revised_restart.phase_native_learned_fingerprint(), revised_fp);
        revised_restart.set_planning_learning_enabled(false);
        assert!(
            exploit(&mut revised_restart, &world, true, 5, 3, world.length + 3).0,
            "revised capability must survive checkpoint restore"
        );
    }

    println!(
        "P3_PREFLIGHT full={}/12 direct_only={}/12 random={}/12 mean_acquisition_interactions={:.3} mean_revision_interactions={:.3}",
        full_success,
        direct_success,
        random_success,
        total_interactions as f64 / 12.0,
        revision_interactions as f64 / 12.0,
    );

    assert_eq!(full_success, 12);
    assert!(direct_success < full_success, "frontier propagation must beat local-only exploration");
    assert!(random_success < full_success, "autonomous acquisition must beat seeded random actions under the same budget");
}

#[test]
fn p3_source_guard_excludes_host_graph_search_from_autonomous_selector() {
    let source = include_str!("../src/phase_native.rs");
    let start = source
        .find("fn phase_native_exploration_action")
        .expect("P3 selector source");
    let end = source[start..]
        .find("fn native_receptor")
        .map(|offset| start + offset)
        .expect("selector end");
    let selector = &source[start..end];

    for forbidden in [
        "EvoImaginationPlanner",
        "LearnedTransition",
        "ImaginedNode",
        "FrontierNode",
        "VecDeque",
        "BinaryHeap",
        ".pop()",
        ".remove(0)",
    ] {
        assert!(
            !selector.contains(forbidden),
            "P3 selector contains forbidden host-search token {forbidden}"
        );
    }
    for required in [
        "self.synapses",
        "self.cells",
        "conductance",
        "successor_synapse",
        "intrinsic",
    ] {
        assert!(selector.contains(required), "P3 selector missing {required}");
    }
}
