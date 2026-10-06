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

fn synapse_signature(evo: &EvoPhase, indices: &[usize]) -> Vec<(usize, usize, u32, u32, u32, bool)> {
    indices
        .iter()
        .filter_map(|index| evo.phase_native_synapse(*index))
        .map(|syn| (
            syn.from,
            syn.to,
            syn.weight.to_bits(),
            syn.phase_offset.to_bits(),
            syn.confidence.to_bits(),
            syn.plastic,
        ))
        .collect()
}

fn current_native_indices(evo: &EvoPhase) -> Vec<usize> {
    let mut indices = std::collections::BTreeSet::new();
    for circuit in evo.phase_native_circuits() {
        indices.insert(circuit.afferent_synapse);
        indices.insert(circuit.successor_synapse);
        indices.insert(circuit.outcome_synapse);
        indices.insert(circuit.motor_synapse);
    }
    for index in evo.phase_native_decoder_synapses() {
        indices.insert(index);
    }
    if let Some(drive) = evo.phase_native_drive_synapses() {
        indices.extend(drive);
    }
    indices.into_iter().collect()
}

fn prediction_probe(evo: &EvoPhase, domain: usize, world: &World) -> Option<usize> {
    let mut clone = evo.clone();
    clone.set_planning_learning_enabled(false);
    clone.observe_initial_real(&domain_raster(domain, 0, 3, 3), false);
    clone
        .choose_phase_native_action_with_prediction()
        .map(|(decision, _)| decision.first_action)
}

fn zero_native_indices(evo: &mut EvoPhase, indices: &[usize]) {
    for index in indices {
        let _ = evo.perturb_phase_native_synapse_for_control(*index, 0.0, 0.0);
    }
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
    let mut a_native_indices: Option<Vec<usize>> = None;
    let mut a_native_signature: Option<Vec<(usize, usize, u32, u32, u32, bool)>> = None;
    let mut a_decoder_indices: Option<Vec<usize>> = None;
    let mut a_circuit_count: Option<usize> = None;

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

        if domain == 0 {
            let indices = current_native_indices(&full);
            a_native_signature = Some(synapse_signature(&full, &indices));
            a_decoder_indices = Some(full.phase_native_decoder_synapses());
            a_circuit_count = Some(full.phase_native_circuits().len());
            a_native_indices = Some(indices);
        } else if domain == 1 {
            let indices = a_native_indices.as_ref().expect("A native indices");
            let before = a_native_signature.as_ref().expect("A native signature");
            let after = synapse_signature(&full, indices);
            let changed = before
                .iter()
                .zip(&after)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, _)| indices[i])
                .collect::<Vec<_>>();
            let all_after = current_native_indices(&full);
            let old_set = indices.iter().copied().collect::<std::collections::BTreeSet<_>>();
            let new_all = all_after
                .iter()
                .copied()
                .filter(|index| !old_set.contains(index))
                .collect::<Vec<_>>();

            let old_decoders = a_decoder_indices
                .as_ref()
                .expect("A decoder indices")
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            let new_decoders = full
                .phase_native_decoder_synapses()
                .into_iter()
                .filter(|index| !old_decoders.contains(index))
                .collect::<Vec<_>>();

            let old_circuits = a_circuit_count.expect("A circuit count");
            let new_circuit_indices = full.phase_native_circuits()[old_circuits..]
                .iter()
                .flat_map(|circuit| [
                    circuit.afferent_synapse,
                    circuit.successor_synapse,
                    circuit.outcome_synapse,
                    circuit.motor_synapse,
                ])
                .collect::<Vec<_>>();

            let baseline_forward = prediction_probe(&full, 0, &worlds[0]);

            let mut no_new_all = full.clone();
            zero_native_indices(&mut no_new_all, &new_all);
            let no_new_all_forward = prediction_probe(&no_new_all, 0, &worlds[0]);

            let mut no_new_decoders = full.clone();
            zero_native_indices(&mut no_new_decoders, &new_decoders);
            let no_new_decoder_forward = prediction_probe(&no_new_decoders, 0, &worlds[0]);

            let mut no_new_circuits = full.clone();
            zero_native_indices(&mut no_new_circuits, &new_circuit_indices);
            let no_new_circuit_forward = prediction_probe(&no_new_circuits, 0, &worlds[0]);

            println!(
                "P5_A_SYNAPSE_MUTATION after_B changed_count={} changed_indices={:?} tracked_A_indices={} new_all={} new_decoders={} new_circuit_synapses={} forward_baseline={:?} forward_without_new_all={:?} forward_without_new_decoders={:?} forward_without_new_circuits={:?}",
                changed.len(),
                changed,
                indices.len(),
                new_all.len(),
                new_decoders.len(),
                new_circuit_indices.len(),
                baseline_forward,
                no_new_all_forward,
                no_new_decoder_forward,
                no_new_circuit_forward,
            );
        }

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


struct FreshP5Rng(u64);

impl FreshP5Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0xD6E8_FEB8_6659_FD93)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn range(&mut self, upper: usize) -> usize {
        (self.next() % upper as u64) as usize
    }
}

#[derive(Debug, Clone)]
struct FreshP5Block {
    source: Vec<World>,
    target: Vec<World>,
    order: [usize; 4],
    changed_domain: usize,
    intermediate_xy: [[(usize, usize); 4]; 4],
    pre_xy: [(usize, usize); 4],
    post_xy: [(usize, usize); 4],
    restart_xy: [(usize, usize); 4],
}

fn fresh_p5_world(rng: &mut FreshP5Rng, min_len: usize, max_len: usize) -> World {
    let length = min_len + rng.range(max_len - min_len + 1);
    let advance = (0..length).map(|_| rng.range(3)).collect::<Vec<_>>();
    let detour_at = rng.range(length);
    let detour_action = rng.range(3);
    World {
        length,
        advance,
        detour_at,
        detour_action,
    }
}

fn fresh_p5_xy(rng: &mut FreshP5Rng) -> (usize, usize) {
    (1 + rng.range(3), 1 + rng.range(3))
}

fn fresh_p5_order(rng: &mut FreshP5Rng) -> [usize; 4] {
    let mut order = [0usize, 1, 2, 3];
    for i in (1..4).rev() {
        let j = rng.range(i + 1);
        order.swap(i, j);
    }
    order
}

fn fnv64_mix(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn fresh_p5_blocks(authority: u64) -> (Vec<FreshP5Block>, u64) {
    let mut blocks = Vec::new();
    let mut digest = 14_695_981_039_346_656_037_u64;

    for sub in 0..10u64 {
        let derived = authority
            ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ 0x50A5_C011_71A1_5EED;
        let mut rng = FreshP5Rng::new(derived);
        let source = (0..8)
            .map(|_| fresh_p5_world(&mut rng, 2, 3))
            .collect::<Vec<_>>();
        let target = (0..4)
            .map(|_| fresh_p5_world(&mut rng, 3, 4))
            .collect::<Vec<_>>();
        let order = fresh_p5_order(&mut rng);
        let changed_domain = rng.range(4);

        let mut intermediate_xy = [[(1usize, 1usize); 4]; 4];
        for stage in 0..4 {
            for domain in 0..4 {
                intermediate_xy[stage][domain] = fresh_p5_xy(&mut rng);
            }
        }
        let mut pre_xy = [(1usize, 1usize); 4];
        let mut post_xy = [(1usize, 1usize); 4];
        let mut restart_xy = [(1usize, 1usize); 4];
        for domain in 0..4 {
            pre_xy[domain] = fresh_p5_xy(&mut rng);
            post_xy[domain] = fresh_p5_xy(&mut rng);
            restart_xy[domain] = fresh_p5_xy(&mut rng);
        }

        digest = fnv64_mix(digest, sub);
        for (family, worlds) in [(0u64, &source), (1u64, &target)] {
            digest = fnv64_mix(digest, family);
            for world in worlds {
                digest = fnv64_mix(digest, world.length as u64);
                digest = fnv64_mix(digest, world.detour_at as u64);
                digest = fnv64_mix(digest, world.detour_action as u64);
                for action in &world.advance {
                    digest = fnv64_mix(digest, *action as u64);
                }
            }
        }
        for domain in order {
            digest = fnv64_mix(digest, domain as u64);
        }
        digest = fnv64_mix(digest, changed_domain as u64);
        for stage in 0..4 {
            for domain in 0..4 {
                let (x, y) = intermediate_xy[stage][domain];
                digest = fnv64_mix(digest, x as u64);
                digest = fnv64_mix(digest, y as u64);
            }
        }
        for bank in [&pre_xy, &post_xy, &restart_xy] {
            for &(x, y) in bank.iter() {
                digest = fnv64_mix(digest, x as u64);
                digest = fnv64_mix(digest, y as u64);
            }
        }

        blocks.push(FreshP5Block {
            source,
            target,
            order,
            changed_domain,
            intermediate_xy,
            pre_xy,
            post_xy,
            restart_xy,
        });
    }

    (blocks, digest)
}

fn train_drive_from(source: &[World]) -> (PhaseDriveCheckpoint, usize) {
    let mut checkpoint = None;
    let mut cost = 0usize;
    for world in source {
        let mut evo = carrier(true);
        if let Some(previous) = checkpoint.take() {
            assert!(evo.restore_phase_native_drive_checkpoint(previous));
        } else {
            assert!(evo.enable_phase_native_learned_drive(PhaseDriveConfig::default()));
            assert_eq!(evo.phase_native_drive_weights(), Some([0.0, 0.0]));
        }
        assert_eq!(evo.phase_native_receptor_count(), 0);
        assert_eq!(evo.phase_native_circuits().len(), 0);
        let outcome = source_teacher_acquire(&mut evo, world);
        assert!(outcome.success, "fresh P5 source meta-world must be solvable");
        cost += outcome.interactions;
        checkpoint = evo.phase_native_drive_checkpoint();
    }
    (
        checkpoint.expect("fresh P5 learned drive checkpoint"),
        cost,
    )
}

fn p5_wilson95(success: usize, n: usize) -> (f64, f64) {
    let z = 1.959_963_984_540_054_f64;
    let n = n as f64;
    let p = success as f64 / n;
    let denominator = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / denominator;
    let half = z
        * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt()
        / denominator;
    (center - half, center + half)
}

fn p5_mean_sd(values: &[usize]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<usize>() as f64 / values.len() as f64;
    if values.len() == 1 {
        return (mean, 0.0);
    }
    let variance = values
        .iter()
        .map(|value| (*value as f64 - mean).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64;
    (mean, variance.sqrt())
}

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from first-attempt CI"]
fn p5_fresh_continual_retention_pack() {
    let authority: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse()
        .expect("fresh P5 seed must be u64");
    let source_sha = std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha = std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let (blocks, digest) = fresh_p5_blocks(authority);
    println!(
        "FRESH_P5_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha, spec_sha, authority, digest
    );
    for (sub, block) in blocks.iter().enumerate() {
        println!("FRESH_P5_BLOCK sub={} {:?}", sub, block);
    }

    let mut target_acquisition_success = 0usize;
    let mut intermediate_success = 0usize;
    let mut intermediate_total = 0usize;
    let mut pre_success = 0usize;
    let mut post_success = 0usize;
    let mut checkpoint_success = 0usize;
    let mut repair_success = 0usize;
    let mut frozen_changed_success = 0usize;
    let mut reset_earlier_success = 0usize;
    let mut zero_drive_acquisition_success = 0usize;
    let mut no_growth_acquisition_success = 0usize;
    let mut primary_per_seed = Vec::new();
    let mut acquisition_costs = Vec::new();
    let mut repair_costs = Vec::new();
    let mut meta_cost = 0usize;
    let mut drive_weight_violations = 0usize;
    let mut legacy_graph_violations = 0usize;
    let mut structural_counts = Vec::new();

    for (sub, block) in blocks.iter().enumerate() {
        let (drive_checkpoint, sub_meta_cost) = train_drive_from(&block.source);
        meta_cost += sub_meta_cost;

        let mut full = persistent_with_drive(drive_checkpoint.clone(), true);
        let frozen_drive = full.phase_native_drive_weights().expect("fresh P5 drive");
        assert!(frozen_drive[0] > 0.05 && frozen_drive[1] > 0.05);
        assert_eq!(full.phase_native_receptor_count(), 0);
        assert_eq!(full.phase_native_circuits().len(), 0);
        assert_eq!(full.planning_transition_count(), 0);

        let mut acquired = Vec::new();
        let mut counts = Vec::new();

        for (stage, &domain) in block.order.iter().enumerate() {
            full.set_planning_learning_enabled(true);
            let outcome = acquire_domain(
                &mut full,
                domain,
                &block.target[domain],
                false,
                ACQUIRE_BUDGET,
            );
            target_acquisition_success += usize::from(outcome.success);
            acquisition_costs.push(outcome.interactions);
            acquired.push(domain);

            if full.phase_native_drive_weights() != Some(frozen_drive) {
                drive_weight_violations += 1;
            }
            if full.planning_transition_count() != 0 {
                legacy_graph_violations += 1;
            }

            counts.push((
                full.phase_native_receptor_count(),
                full.phase_native_circuits().len(),
            ));

            full.set_planning_learning_enabled(false);
            for &prior in &acquired {
                intermediate_total += 1;
                let (x, y) = block.intermediate_xy[stage][prior];
                let retained = exploit_domain(
                    &mut full,
                    prior,
                    &block.target[prior],
                    false,
                    x,
                    y,
                );
                intermediate_success += usize::from(retained.success);
            }
        }
        structural_counts.push(counts);

        let mut sub_primary = 0usize;
        full.set_planning_learning_enabled(false);
        for domain in 0..4 {
            let (x, y) = block.pre_xy[domain];
            let retained = exploit_domain(
                &mut full,
                domain,
                &block.target[domain],
                false,
                x,
                y,
            );
            pre_success += usize::from(retained.success);
            sub_primary += usize::from(retained.success);
        }

        let mut frozen_changed = full.clone();
        frozen_changed.set_planning_learning_enabled(false);
        let changed = block.changed_domain;
        let (fx, fy) = block.post_xy[changed];
        let frozen_result = exploit_domain(
            &mut frozen_changed,
            changed,
            &block.target[changed],
            true,
            fx,
            fy,
        );
        frozen_changed_success += usize::from(frozen_result.success);

        full.set_planning_learning_enabled(true);
        let repaired = acquire_domain(
            &mut full,
            changed,
            &block.target[changed],
            true,
            REPAIR_BUDGET,
        );
        repair_success += usize::from(repaired.success);
        if repaired.success {
            repair_costs.push(repaired.interactions);
        }
        if full.phase_native_drive_weights() != Some(frozen_drive) {
            drive_weight_violations += 1;
        }
        if full.planning_transition_count() != 0 {
            legacy_graph_violations += 1;
        }

        full.set_planning_learning_enabled(false);
        for domain in 0..4 {
            let (x, y) = block.post_xy[domain];
            let retained = exploit_domain(
                &mut full,
                domain,
                &block.target[domain],
                domain == changed,
                x,
                y,
            );
            post_success += usize::from(retained.success);
            sub_primary += usize::from(retained.success);
        }

        let checkpoint = full.phase_native_checkpoint().expect("fresh P5 lifetime checkpoint");
        let mut restarted = restore_full(checkpoint);
        assert!(restarted.current_real().is_none());
        restarted.set_planning_learning_enabled(false);
        for domain in 0..4 {
            let (x, y) = block.restart_xy[domain];
            let retained = exploit_domain(
                &mut restarted,
                domain,
                &block.target[domain],
                domain == changed,
                x,
                y,
            );
            checkpoint_success += usize::from(retained.success);
        }

        primary_per_seed.push(sub_primary);

        // RESET_BETWEEN_WORLDS: only the final separately trained carrier survives.
        let mut final_cold = None;
        for &domain in &block.order {
            let mut cold = persistent_with_drive(drive_checkpoint.clone(), true);
            let _ = acquire_domain(
                &mut cold,
                domain,
                &block.target[domain],
                false,
                ACQUIRE_BUDGET,
            );
            final_cold = Some((domain, cold));
        }
        let (final_domain, mut final_cold) = final_cold.expect("final reset control");
        final_cold.set_planning_learning_enabled(false);
        for domain in 0..4 {
            if domain == final_domain {
                continue;
            }
            let (x, y) = block.pre_xy[domain];
            reset_earlier_success += usize::from(
                exploit_domain(
                    &mut final_cold,
                    domain,
                    &block.target[domain],
                    false,
                    x,
                    y,
                )
                .success,
            );
        }

        // ZERO_DRIVE persistent control.
        let mut zero = carrier(true);
        assert!(zero.enable_phase_native_learned_drive(PhaseDriveConfig {
            learning_enabled: false,
            ..PhaseDriveConfig::default()
        }));
        zero.set_phase_native_drive_learning_enabled(false);
        for &domain in &block.order {
            let outcome = acquire_domain(
                &mut zero,
                domain,
                &block.target[domain],
                false,
                ACQUIRE_BUDGET,
            );
            zero_drive_acquisition_success += usize::from(outcome.success);
        }

        // NO_GROWTH persistent control.
        let mut no_growth = persistent_with_drive(drive_checkpoint, false);
        for &domain in &block.order {
            let outcome = acquire_domain(
                &mut no_growth,
                domain,
                &block.target[domain],
                false,
                ACQUIRE_BUDGET,
            );
            no_growth_acquisition_success += usize::from(outcome.success);
        }

        println!(
            "FRESH_P5_SUB sub={} primary={}/8 acquired_so_far={} repair={} counts={:?} drive={:?}",
            sub,
            sub_primary,
            target_acquisition_success,
            repaired.success,
            structural_counts.last().unwrap(),
            frozen_drive,
        );
    }

    let primary_success = pre_success + post_success;
    let n = 80usize;
    let (lo, hi) = p5_wilson95(primary_success, n);
    let (acquisition_mean, acquisition_sd) = p5_mean_sd(&acquisition_costs);
    let (repair_mean, repair_sd) = p5_mean_sd(&repair_costs);

    println!(
        "FRESH_P5_RESULT N={} primary={}/{} wilson95=[{:.6},{:.6}] per_seed={:?} acquisition={}/40 intermediate={}/{} pre={}/40 repair={}/10 post={}/40 checkpoint={}/40 frozen_changed={}/10 reset_earlier={}/30 zero_drive_acq={}/40 no_growth_acq={}/40 drive_weight_violations={} legacy_graph_violations={}",
        n,
        primary_success,
        n,
        lo,
        hi,
        primary_per_seed,
        target_acquisition_success,
        intermediate_success,
        intermediate_total,
        pre_success,
        repair_success,
        post_success,
        checkpoint_success,
        frozen_changed_success,
        reset_earlier_success,
        zero_drive_acquisition_success,
        no_growth_acquisition_success,
        drive_weight_violations,
        legacy_graph_violations,
    );
    println!(
        "FRESH_P5_COST acquisition_mean={:.3} acquisition_sd={:.3} repair_mean={:.3} repair_sd={:.3} source_meta_cost={}",
        acquisition_mean,
        acquisition_sd,
        repair_mean,
        repair_sd,
        meta_cost,
    );
    println!("FRESH_P5_STRUCTURE {:?}", structural_counts);

    assert_eq!(target_acquisition_success, 40, "all 40 target worlds must be acquired");
    assert_eq!(
        intermediate_success, intermediate_total,
        "every intermediate retained revisit must succeed"
    );
    assert_eq!(pre_success, 40, "pre-change retention must be 40/40");
    assert!(repair_success >= 9, "changed-world repair must be >=9/10");
    assert!(post_success >= 39, "post-change retention must be >=39/40");
    assert!(primary_success >= 79, "primary FULL must be >=79/80");
    assert!(lo >= 0.93, "Wilson95 lower bound must be >=0.93");
    assert!(
        primary_per_seed.iter().all(|score| *score >= 7),
        "every sub-seed primary score must be >=7/8"
    );
    assert!(
        checkpoint_success >= 39,
        "checkpoint-restored retention must be >=39/40"
    );
    assert!(
        frozen_changed_success <= 2,
        "frozen changed-world control must be <=2/10"
    );
    assert!(
        reset_earlier_success <= 6,
        "reset-between-worlds earlier retention must be <=6/30"
    );
    assert!(
        zero_drive_acquisition_success <= 20,
        "zero-drive acquisition must be <=20/40"
    );
    assert!(
        no_growth_acquisition_success <= 4,
        "no-growth acquisition must be <=4/40"
    );
    assert!(
        acquisition_mean <= 35.0,
        "mean target acquisition cost must be <=35"
    );
    assert!(
        repair_mean <= 30.0,
        "mean successful repair cost must be <=30"
    );
    assert_eq!(drive_weight_violations, 0, "target P4 drive must remain frozen");
    assert_eq!(legacy_graph_violations, 0, "legacy graph backend must remain absent");
}
