//! P5 mechanism evaluator: one persistent organism learns several worlds in sequence,
//! retains earlier models, repairs one changed law, and survives one full checkpoint.
use aeterna_v1::{EvoConfig, EvoPhase, RasterFieldConfig};
use aeterna_v1::carrier::{PhaseDriveCheckpoint, PhaseDriveConfig, PhaseNativeCheckpoint, PhaseNativeConfig};

const ACQUIRE_BUDGET: usize = 60;
const REPAIR_BUDGET: usize = 40;

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
        if state == self.length {
            return (state, 1.0);
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
        (next, f32::from(next == self.length))
    }
}

fn target_worlds() -> Vec<World> {
    vec![
        World {
            length: 3,
            advance: vec![1, 2, 0],
            detour_at: 1,
            detour_action: 1,
        },
        World {
            length: 4,
            advance: vec![2, 1, 0, 2],
            detour_at: 2,
            detour_action: 1,
        },
        World {
            length: 3,
            advance: vec![0, 2, 1],
            detour_at: 1,
            detour_action: 0,
        },
        World {
            length: 4,
            advance: vec![1, 0, 2, 1],
            detour_at: 1,
            detour_action: 2,
        },
    ]
}

fn source_worlds() -> Vec<World> {
    (0..8usize)
        .map(|index| {
            let length = 2 + index % 2;
            let advance = (0..length)
                .map(|stage| (index + 2 * stage + 1) % 3)
                .collect::<Vec<_>>();
            World {
                length,
                advance,
                detour_at: 0,
                detour_action: 0,
            }
        })
        .collect()
}

fn carrier(growth: bool) -> EvoPhase {
    let cfg = EvoConfig {
        sensory_cells: 144,
        motor_cells: 3,
        dormant_cells: 96,
        hdc_dim: 192,
        weight_learning_rate: 1.0,
        phase_learning_rate: 1.0,
        min_recruit_support: 1,
        structural_growth_enabled: growth,
        ..EvoConfig::default()
    };
    let mut evo = EvoPhase::new(cfg);
    let mut field = RasterFieldConfig::for_raster(12, 12, 3);
    field.learning_enabled = false;
    field.readout_enabled = false;
    evo.attach_raster_field(field);
    evo.enable_phase_native_planning(PhaseNativeConfig {
        horizon: 8,
        ..PhaseNativeConfig::default()
    });
    evo
}

fn relation_delta(index: usize) -> (usize, usize) {
    const DELTAS: [(usize, usize); 32] = [
        (1,0),(2,0),(3,0),(4,0),(5,0),(6,0),
        (0,1),(0,2),(0,3),(0,4),(0,5),(0,6),
        (1,1),(1,2),(1,3),(1,4),(1,5),
        (2,1),(2,2),(2,3),(2,4),(2,5),
        (3,1),(3,2),(3,3),(3,4),
        (4,1),(4,2),(4,3),
        (5,1),(5,2),(6,1),
    ];
    DELTAS[index]
}

fn source_raster(state: usize) -> Vec<f32> {
    let (dx, dy) = relation_delta(state);
    let mut raster = vec![0.0; 144];
    raster[0] = 1.0;
    raster[dy * 12 + dx] = 1.0;
    raster
}

fn domain_raster(domain: usize, state: usize, x: usize, y: usize) -> Vec<f32> {
    let index = 6 + domain * 6 + state;
    let (dx, dy) = relation_delta(index);
    assert!(x + dx < 12 && y + dy < 12);
    let mut raster = vec![0.0; 144];
    raster[y * 12 + x] = 1.0;
    raster[(y + dy) * 12 + x + dx] = 1.0;
    raster
}

fn max_cross_domain_trace_similarity(evo: &EvoPhase, worlds: &[World]) -> f32 {
    let field = evo.raster_field().expect("raster field");
    let mut best = -1.0_f32;
    for a_domain in 0..worlds.len() {
        for b_domain in (a_domain + 1)..worlds.len() {
            for a_state in 0..=worlds[a_domain].length {
                for b_state in 0..=worlds[b_domain].length {
                    let a = field
                        .encode_relational_trace(&domain_raster(a_domain, a_state, 0, 0))
                        .expect("A trace");
                    let b = field
                        .encode_relational_trace(&domain_raster(b_domain, b_state, 0, 0))
                        .expect("B trace");
                    best = best.max(a.similarity(&b));
                }
            }
        }
    }
    best
}

fn diagnostic_decision(evo: &mut EvoPhase, domain: usize, world: &World) -> Option<(usize, f32, Vec<f32>)> {
    let sensory = domain_raster(domain, 0, 3, 3);
    evo.observe_initial_real(&sensory, false);
    let decision = evo.plan_imagined(&sensory)?;
    Some((
        decision.first_action,
        decision.predicted_value,
        evo.phase_native_motor_potentials().to_vec(),
    ))
}

#[derive(Debug, Clone, Copy)]
struct Outcome {
    success: bool,
    interactions: usize,
}

fn source_teacher_acquire(evo: &mut EvoPhase, world: &World) -> Outcome {
    let mut state = 0usize;
    evo.observe_initial_real(&source_raster(state), false);
    for interaction in 1..=ACQUIRE_BUDGET {
        let Some(action) = evo.choose_phase_native_autonomous_action() else {
            return Outcome { success: false, interactions: interaction - 1 };
        };
        let (next, value) = world.step(state, action, false);
        assert!(evo
            .observe_phase_native_action_result(action, &source_raster(next), value)
            .is_some());
        state = next;
        if value >= 1.0 {
            return Outcome { success: true, interactions: interaction };
        }
    }
    Outcome { success: false, interactions: ACQUIRE_BUDGET }
}

fn train_drive() -> (PhaseDriveCheckpoint, usize) {
    let mut checkpoint = None;
    let mut cost = 0usize;
    for world in source_worlds() {
        let mut evo = carrier(true);
        if let Some(previous) = checkpoint.take() {
            assert!(evo.restore_phase_native_drive_checkpoint(previous));
        } else {
            assert!(evo.enable_phase_native_learned_drive(PhaseDriveConfig::default()));
            assert_eq!(evo.phase_native_drive_weights(), Some([0.0, 0.0]));
        }
        assert_eq!(evo.phase_native_receptor_count(), 0);
        assert_eq!(evo.phase_native_circuits().len(), 0);
        let outcome = source_teacher_acquire(&mut evo, &world);
        assert!(outcome.success);
        cost += outcome.interactions;
        checkpoint = evo.phase_native_drive_checkpoint();
    }
    (checkpoint.expect("P5 learned drive checkpoint"), cost)
}

fn persistent_with_drive(checkpoint: PhaseDriveCheckpoint, growth: bool) -> EvoPhase {
    let mut evo = carrier(growth);
    if growth {
        assert!(evo.restore_phase_native_drive_checkpoint(checkpoint));
    } else {
        // Allocate the same drive channels before testing the no-growth substrate.
        assert!(evo.restore_phase_native_drive_checkpoint(checkpoint));
    }
    evo.set_phase_native_drive_learning_enabled(false);
    assert_eq!(evo.phase_native_receptor_count(), 0);
    assert_eq!(evo.phase_native_circuits().len(), 0);
    assert_eq!(evo.planning_transition_count(), 0);
    evo
}

fn acquire_domain(
    evo: &mut EvoPhase,
    domain: usize,
    world: &World,
    revised: bool,
    budget: usize,
) -> Outcome {
    let mut state = 0usize;
    evo.observe_initial_real(&domain_raster(domain, state, 0, 0), false);
    for interaction in 1..=budget {
        let Some(action) = evo.choose_phase_native_learned_drive_action() else {
            return Outcome { success: false, interactions: interaction - 1 };
        };
        let (next, value) = world.step(state, action, revised);
        let post = domain_raster(domain, next, 0, 0);
        assert!(evo
            .observe_phase_native_action_result(action, &post, value)
            .is_some());
        state = next;
        if value >= 1.0 {
            return Outcome { success: true, interactions: interaction };
        }
    }
    Outcome { success: false, interactions: budget }
}

fn exploit_domain(
    evo: &mut EvoPhase,
    domain: usize,
    world: &World,
    revised: bool,
    x: usize,
    y: usize,
) -> Outcome {
    let mut state = 0usize;
    evo.observe_initial_real(&domain_raster(domain, state, x, y), false);
    for interaction in 1..=(world.length + 1) {
        let Some((decision, prediction)) = evo.choose_phase_native_action_with_prediction() else {
            return Outcome { success: false, interactions: interaction - 1 };
        };
        assert!(prediction.confidence > 0.0);
        let (next, value) = world.step(state, decision.first_action, revised);
        state = next;
        evo.observe_initial_real(&domain_raster(domain, state, x, y), value >= 1.0);
        if value >= 1.0 {
            return Outcome { success: true, interactions: interaction };
        }
    }
    Outcome { success: false, interactions: world.length + 1 }
}

fn trace_frozen_route(
    mature: &EvoPhase,
    domain: usize,
    world: &World,
    revised: bool,
    x: usize,
    y: usize,
) -> Vec<(usize, usize, usize, f32, Vec<f32>)> {
    let mut evo = mature.clone();
    evo.set_planning_learning_enabled(false);
    let mut state = 0usize;
    evo.observe_initial_real(&domain_raster(domain, state, x, y), false);
    let mut trace = Vec::new();
    for _ in 0..(world.length + 1) {
        let Some((decision, prediction)) = evo.choose_phase_native_action_with_prediction() else {
            trace.push((state, usize::MAX, world.advance.get(state).copied().unwrap_or(usize::MAX), 0.0, evo.phase_native_motor_potentials().to_vec()));
            break;
        };
        let expected = world.advance.get(state).copied().unwrap_or(usize::MAX);
        let potentials = evo.phase_native_motor_potentials().to_vec();
        trace.push((state, decision.first_action, expected, decision.predicted_value, potentials));
        let (next, value) = world.step(state, decision.first_action, revised);
        state = next;
        evo.observe_initial_real(&domain_raster(domain, state, x, y), value >= 1.0);
        if value >= 1.0 {
            break;
        }
        let _ = prediction;
    }
    trace
}

fn restore_full(checkpoint: PhaseNativeCheckpoint) -> EvoPhase {
    let mut evo = carrier(true);
    assert!(evo.restore_phase_native_checkpoint(checkpoint));
    evo
}

#[test]
fn p5_one_persistent_organism_retains_and_selectively_revises_multiple_worlds() {
    let worlds = target_worlds();
    let (drive_checkpoint, meta_cost) = train_drive();
    let mut full = persistent_with_drive(drive_checkpoint.clone(), true);
    let max_cross_similarity = max_cross_domain_trace_similarity(&full, &worlds);
    println!("P5_REPRESENTATION max_cross_domain_similarity={:.6}", max_cross_similarity);
    assert!(
        max_cross_similarity < 0.97,
        "P5 evaluator target families alias at the carrier trace threshold: {max_cross_similarity}"
    );
    let frozen_drive = full.phase_native_drive_weights().unwrap();
    assert!(frozen_drive[0] > 0.05 && frozen_drive[1] > 0.05);

    let mut acquisition_costs = Vec::new();
    let mut retention_actions = 0usize;
    let mut receptor_counts = Vec::new();
    let mut circuit_counts = Vec::new();

    for domain in 0..worlds.len() {
        full.set_planning_learning_enabled(true);
        let outcome = acquire_domain(
            &mut full,
            domain,
            &worlds[domain],
            false,
            ACQUIRE_BUDGET,
        );
        assert!(outcome.success, "persistent acquisition failed domain {domain}");
        assert_eq!(full.phase_native_drive_weights().unwrap(), frozen_drive);
        acquisition_costs.push(outcome.interactions);
        receptor_counts.push(full.phase_native_receptor_count());
        circuit_counts.push(full.phase_native_circuits().len());

        full.set_planning_learning_enabled(false);
        for prior in 0..=domain {
            let held_x = 3 + (prior % 2);
            let held_y = 3 + ((domain + prior) % 2);
            let before_retention = diagnostic_decision(&mut full, prior, &worlds[prior]);
            let retained = exploit_domain(
                &mut full,
                prior,
                &worlds[prior],
                false,
                held_x,
                held_y,
            );
            if !retained.success {
                let route = trace_frozen_route(
                    &full,
                    prior,
                    &worlds[prior],
                    false,
                    held_x,
                    held_y,
                );
                println!(
                    "P5_RETENTION_FAIL learned_through={} prior={} expected_first={} decision={:?} receptors={} circuits={} route={:?}",
                    domain,
                    prior,
                    worlds[prior].advance[0],
                    before_retention,
                    full.phase_native_receptor_count(),
                    full.phase_native_circuits().len(),
                    route,
                );
            }
            assert!(
                retained.success,
                "domain {prior} forgotten after learning domain {domain}"
            );
            assert!(retained.interactions <= worlds[prior].length + 1);
            retention_actions += retained.interactions;
        }
    }

    assert!(
        receptor_counts.windows(2).all(|w| w[1] > w[0]),
        "new domains must add retained receptor structure: {receptor_counts:?}"
    );
    assert!(
        circuit_counts.windows(2).all(|w| w[1] > w[0]),
        "new domains must add retained transition structure: {circuit_counts:?}"
    );
    assert_eq!(full.planning_transition_count(), 0);

    // Preserve a prediction from an untouched A transition.
    let unchanged_state = 1usize;
    let unchanged_action = worlds[0].advance[unchanged_state];
    let unchanged_before = full
        .imagine_phase_native_actions(
            &domain_raster(0, unchanged_state, 0, 0),
            &[unchanged_action],
        )
        .into_iter()
        .next()
        .expect("untouched A prediction before B revision")
        .sensory;

    let pre_revision = full.clone();
    let mut frozen_changed_b = pre_revision.clone();
    frozen_changed_b.set_planning_learning_enabled(false);
    let stale = exploit_domain(
        &mut frozen_changed_b,
        1,
        &worlds[1],
        true,
        4,
        4,
    );
    assert!(!stale.success, "frozen old B model must fail changed B");

    full.set_planning_learning_enabled(true);
    let repaired = acquire_domain(&mut full, 1, &worlds[1], true, REPAIR_BUDGET);
    assert!(repaired.success, "persistent organism failed autonomous B repair");
    assert_eq!(full.phase_native_drive_weights().unwrap(), frozen_drive);

    full.set_planning_learning_enabled(false);
    for domain in 0..worlds.len() {
        let revised = domain == 1;
        let retained = exploit_domain(
            &mut full,
            domain,
            &worlds[domain],
            revised,
            5,
            3,
        );
        assert!(
            retained.success,
            "domain {domain} unavailable after selective B repair"
        );
    }

    let unchanged_after = full
        .imagine_phase_native_actions(
            &domain_raster(0, unchanged_state, 0, 0),
            &[unchanged_action],
        )
        .into_iter()
        .next()
        .expect("untouched A prediction after B revision")
        .sensory;
    let expected_next = worlds[0].step(unchanged_state, unchanged_action, false).0;
    let expected = domain_raster(0, expected_next, 0, 0);
    let retained_error = unchanged_after
        .iter()
        .zip(&expected)
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>() / expected.len() as f32;
    let drift = unchanged_before
        .iter()
        .zip(&unchanged_after)
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>() / unchanged_before.len() as f32;
    assert!(
        retained_error < 0.01,
        "untouched A transition corrupted by B revision: error={retained_error} drift={drift}"
    );

    let fingerprint = full.phase_native_learned_fingerprint();
    let checkpoint = full.phase_native_checkpoint().expect("P5 full checkpoint");
    let mut restarted = restore_full(checkpoint);
    assert_eq!(restarted.phase_native_learned_fingerprint(), fingerprint);
    assert!(restarted.current_real().is_none());
    restarted.set_planning_learning_enabled(false);
    for domain in 0..worlds.len() {
        assert!(
            exploit_domain(
                &mut restarted,
                domain,
                &worlds[domain],
                domain == 1,
                4,
                5,
            )
            .success,
            "checkpoint-restored organism lost domain {domain}"
        );
    }

    // RESET_BETWEEN_WORLDS: only the final cold world's model is retained.
    let mut final_cold = None;
    for domain in 0..worlds.len() {
        let mut cold = persistent_with_drive(drive_checkpoint.clone(), true);
        let outcome = acquire_domain(&mut cold, domain, &worlds[domain], false, ACQUIRE_BUDGET);
        assert!(outcome.success);
        final_cold = Some(cold);
    }
    let mut final_cold = final_cold.unwrap();
    final_cold.set_planning_learning_enabled(false);
    assert!(
        !exploit_domain(&mut final_cold, 0, &worlds[0], false, 3, 3).success,
        "reset-between-worlds final carrier unexpectedly retained A"
    );

    // ZERO_DRIVE persistent control.
    let mut zero = carrier(true);
    assert!(zero.enable_phase_native_learned_drive(PhaseDriveConfig {
        learning_enabled: false,
        ..PhaseDriveConfig::default()
    }));
    zero.set_phase_native_drive_learning_enabled(false);
    let mut zero_success = 0usize;
    let mut zero_cost = 0usize;
    for domain in 0..worlds.len() {
        let outcome = acquire_domain(&mut zero, domain, &worlds[domain], false, ACQUIRE_BUDGET);
        zero_success += usize::from(outcome.success);
        zero_cost += outcome.interactions;
    }

    // NO_GROWTH cannot build the persistent physical world model.
    let mut no_growth = persistent_with_drive(drive_checkpoint, false);
    let mut no_growth_success = 0usize;
    for domain in 0..worlds.len() {
        let outcome = acquire_domain(
            &mut no_growth,
            domain,
            &worlds[domain],
            false,
            ACQUIRE_BUDGET,
        );
        no_growth_success += usize::from(outcome.success);
    }

    let full_cost: usize = acquisition_costs.iter().sum();
    println!(
        "P5_MECHANISM meta_cost={} acquisitions={:?} full_total={} retention_actions={} repair={} receptors={:?} circuits={:?} zero={}/4 zero_cost={} no_growth={}/4 fingerprint={:016x}",
        meta_cost,
        acquisition_costs,
        full_cost,
        retention_actions,
        repaired.interactions,
        receptor_counts,
        circuit_counts,
        zero_success,
        zero_cost,
        no_growth_success,
        fingerprint,
    );

    assert_eq!(acquisition_costs.len(), 4);
    assert!(
        zero_success < 4 || zero_cost > full_cost,
        "ZERO_DRIVE must be worse in acquisition success or total cost"
    );
    assert!(no_growth_success < 4, "NO_GROWTH must not match persistent FULL");
}

#[test]
fn p5_source_guard_has_no_world_identity_channel() {
    let production = [
        include_str!("../src/phase_native.rs"),
        include_str!("../src/phase_drive.rs"),
        include_str!("../src/phase_forward.rs"),
    ]
    .join("\n");
    for forbidden in [
        "set_world_id",
        "task_id",
        "domain_id",
        "correct_action",
        "hidden_action",
        "load_world_checkpoint",
    ] {
        assert!(
            !production.contains(forbidden),
            "P5 production path contains forbidden cognition channel token {forbidden}"
        );
    }
}
