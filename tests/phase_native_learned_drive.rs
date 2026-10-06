//! P4 mechanism evaluator: learn exploration value, then transfer only the drive.
use aeterna_v1::{EvoConfig, EvoPhase, RasterFieldConfig};
use aeterna_v1::carrier::{PhaseDriveCheckpoint, PhaseDriveConfig, PhaseNativeConfig};

const BUDGET: usize = 60;

#[derive(Debug, Clone)]
struct World {
    length: usize,
    advance: Vec<usize>,
}

impl World {
    fn step(&self, state: usize, action: usize) -> (usize, f32) {
        if state == self.length {
            return (state, 1.0);
        }
        let next = if action == self.advance[state] { state + 1 } else { 0 };
        (next, f32::from(next == self.length))
    }
}

fn source_worlds() -> Vec<World> {
    (0..8usize)
        .map(|index| {
            let length = 2 + index % 2;
            let advance = (0..length)
                .map(|stage| (index + 2 * stage + 1) % 3)
                .collect();
            World { length, advance }
        })
        .collect()
}

fn target_worlds() -> Vec<World> {
    (0..12usize)
        .map(|index| {
            let length = 4 + index % 2;
            let advance = (0..length)
                .map(|stage| (2 * index + stage + 1) % 3)
                .collect();
            World { length, advance }
        })
        .collect()
}

fn raster(state: usize, target_bank: bool) -> Vec<f32> {
    let deltas = [
        (1usize, 0usize),
        (2, 0),
        (3, 0),
        (1, 1),
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 2),
        (2, 1),
        (2, 2),
    ];
    let index = if target_bank { 4 + state } else { state };
    let (dx, dy) = deltas[index];
    let mut values = vec![0.0; 144];
    values[0] = 1.0;
    values[dy * 12 + dx] = 1.0;
    values
}

fn carrier() -> EvoPhase {
    let cfg = EvoConfig {
        sensory_cells: 144,
        motor_cells: 3,
        dormant_cells: 96,
        hdc_dim: 192,
        weight_learning_rate: 1.0,
        phase_learning_rate: 1.0,
        min_recruit_support: 1,
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

#[derive(Debug, Clone, Copy)]
struct Outcome {
    success: bool,
    interactions: usize,
}

fn teacher_acquire(evo: &mut EvoPhase, world: &World) -> Outcome {
    let mut state = 0usize;
    evo.observe_initial_real(&raster(state, false), false);
    for interaction in 1..=BUDGET {
        let Some(action) = evo.choose_phase_native_autonomous_action() else {
            return Outcome { success: false, interactions: interaction - 1 };
        };
        let (next, value) = world.step(state, action);
        assert!(evo
            .observe_phase_native_action_result(action, &raster(next, false), value)
            .is_some());
        state = next;
        if value >= 1.0 {
            return Outcome { success: true, interactions: interaction };
        }
    }
    Outcome { success: false, interactions: BUDGET }
}

fn learned_acquire(evo: &mut EvoPhase, world: &World) -> Outcome {
    let mut state = 0usize;
    evo.observe_initial_real(&raster(state, true), false);
    for interaction in 1..=BUDGET {
        let Some(action) = evo.choose_phase_native_learned_drive_action() else {
            return Outcome { success: false, interactions: interaction - 1 };
        };
        let (next, value) = world.step(state, action);
        assert!(evo
            .observe_phase_native_action_result(action, &raster(next, true), value)
            .is_some());
        state = next;
        if value >= 1.0 {
            return Outcome { success: true, interactions: interaction };
        }
    }
    Outcome { success: false, interactions: BUDGET }
}

fn random_acquire(world: &World, seed: u64) -> Outcome {
    let mut x = seed ^ 0xD1B5_4A32_D192_ED03;
    let mut state = 0usize;
    for interaction in 1..=BUDGET {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        let action = (x.wrapping_mul(0x2545_F491_4F6C_DD1D) % 3) as usize;
        let (next, value) = world.step(state, action);
        state = next;
        if value >= 1.0 {
            return Outcome { success: true, interactions: interaction };
        }
    }
    Outcome { success: false, interactions: BUDGET }
}

fn train_drive(phase_learning: bool) -> PhaseDriveCheckpoint {
    let mut checkpoint = None;

    for (index, world) in source_worlds().iter().enumerate() {
        let mut evo = carrier();
        if let Some(previous) = checkpoint.take() {
            assert!(evo.restore_phase_native_drive_checkpoint(previous));
        } else {
            assert!(evo.enable_phase_native_learned_drive(PhaseDriveConfig {
                learning_rate: 0.35,
                discount: 0.90,
                learning_enabled: true,
                readout_enabled: true,
                phase_learning_enabled: phase_learning,
            }));
            assert_eq!(evo.phase_native_drive_weights(), Some([0.0, 0.0]));
        }

        assert_eq!(evo.phase_native_receptor_count(), 0);
        assert_eq!(evo.phase_native_circuits().len(), 0);
        assert_eq!(evo.planning_transition_count(), 0);

        let outcome = teacher_acquire(&mut evo, world);
        assert!(outcome.success, "source world {index} P3 bootstrap failed");
        checkpoint = evo.phase_native_drive_checkpoint();
    }

    checkpoint.expect("drive checkpoint after source tuition")
}

fn cold_with_drive(checkpoint: PhaseDriveCheckpoint) -> EvoPhase {
    let mut evo = carrier();
    assert!(evo.restore_phase_native_drive_checkpoint(checkpoint));
    assert_eq!(evo.phase_native_receptor_count(), 0);
    assert_eq!(evo.phase_native_circuits().len(), 0);
    assert_eq!(evo.planning_transition_count(), 0);
    evo.set_phase_native_drive_learning_enabled(false);
    evo
}

fn mean_cost(outcomes: &[Outcome]) -> f64 {
    outcomes.iter().map(|o| o.interactions).sum::<usize>() as f64 / outcomes.len() as f64
}

#[test]
fn p4_learned_physical_drive_transfers_to_cold_longer_worlds() {
    let learned_checkpoint = train_drive(true);
    let zero_phase_checkpoint = train_drive(false);

    let mut probe = cold_with_drive(learned_checkpoint.clone());
    let learned_weights = probe.phase_native_drive_weights().expect("learned weights");
    assert!(learned_weights[0] > 0.05, "DIRECT_UNMODELLED weight must be acquired");
    assert!(learned_weights[1] > 0.01, "REACHABLE_FRONTIER weight must be acquired");
    assert!(probe.phase_native_drive_observations() > 0);

    // The drive-only transfer itself must contain no source-world model.
    assert_eq!(probe.phase_native_receptor_count(), 0);
    assert_eq!(probe.phase_native_circuits().len(), 0);

    let worlds = target_worlds();
    let mut learned = Vec::new();
    let mut zero = Vec::new();
    let mut lesion = Vec::new();
    let mut zero_phase = Vec::new();
    let mut random = Vec::new();
    let mut teacher = Vec::new();

    for (index, world) in worlds.iter().enumerate() {
        let mut full = cold_with_drive(learned_checkpoint.clone());
        let before_drive = full.phase_native_drive_weights().unwrap();
        let outcome = learned_acquire(&mut full, world);
        assert_eq!(
            full.phase_native_drive_weights().unwrap(),
            before_drive,
            "target drive learning must stay frozen"
        );
        learned.push(outcome);

        let mut zero_evo = carrier();
        assert!(zero_evo.enable_phase_native_learned_drive(PhaseDriveConfig {
            learning_enabled: false,
            ..PhaseDriveConfig::default()
        }));
        zero_evo.set_phase_native_drive_learning_enabled(false);
        zero.push(learned_acquire(&mut zero_evo, world));

        let mut lesioned = cold_with_drive(learned_checkpoint.clone());
        let frontier_synapse = lesioned.phase_native_drive_synapses().unwrap()[1];
        assert!(lesioned
            .perturb_phase_native_synapse_for_control(frontier_synapse, 0.0, 0.0)
            .is_some());
        lesion.push(learned_acquire(&mut lesioned, world));

        let mut no_phase = cold_with_drive(zero_phase_checkpoint.clone());
        zero_phase.push(learned_acquire(&mut no_phase, world));

        random.push(random_acquire(world, 0xA37E_4000 + index as u64));

        let mut ceiling = carrier();
        teacher.push(teacher_acquire(&mut ceiling, world));
    }

    let learned_success = learned.iter().filter(|o| o.success).count();
    let zero_success = zero.iter().filter(|o| o.success).count();
    let lesion_success = lesion.iter().filter(|o| o.success).count();
    let zero_phase_success = zero_phase.iter().filter(|o| o.success).count();
    let random_success = random.iter().filter(|o| o.success).count();
    let teacher_success = teacher.iter().filter(|o| o.success).count();

    let learned_mean = mean_cost(&learned);
    let lesion_mean = mean_cost(&lesion);
    let zero_phase_mean = mean_cost(&zero_phase);
    let random_mean = mean_cost(&random);

    println!(
        "P4_MECHANISM weights={:?} observations={} learned={}/12 mean={:.3} zero={}/12 lesion={}/12 mean={:.3} zero_phase={}/12 mean={:.3} random={}/12 mean={:.3} p3_teacher={}/12",
        learned_weights,
        probe.phase_native_drive_observations(),
        learned_success,
        learned_mean,
        zero_success,
        lesion_success,
        lesion_mean,
        zero_phase_success,
        zero_phase_mean,
        random_success,
        random_mean,
        teacher_success,
    );

    assert!(learned_success >= 11, "LEARNED_DRIVE must solve >=11/12");
    assert!(zero_success <= 6, "ZERO_DRIVE must solve <=6/12");
    assert!(
        lesion_success < learned_success || lesion_mean > learned_mean + 1.0,
        "frontier weight lesion must cause a success or cost loss"
    );
    assert!(
        zero_phase_success < learned_success || zero_phase_mean > learned_mean + 1.0,
        "zero drive phase learning must be strictly worse"
    );
    assert!(
        random_success < learned_success || random_mean > learned_mean,
        "learned drive must beat random in success or mean physical cost"
    );
    assert_eq!(teacher_success, 12, "P3 ceiling should solve all deterministic targets");

    // Direct causal restore on one target: lesion the actual learned frontier
    // synapse, observe loss/extra cost, restore the exact synapse without
    // retraining the drive, and require recovery.
    let world = &worlds[0];
    let mut causal = cold_with_drive(learned_checkpoint);
    let frontier_synapse = causal.phase_native_drive_synapses().unwrap()[1];
    let saved = causal
        .perturb_phase_native_synapse_for_control(frontier_synapse, 0.0, 0.0)
        .expect("learned frontier synapse");
    let lesioned_once = learned_acquire(&mut causal, world);
    causal.restore_phase_native_synapse_for_control(frontier_synapse, saved);
    causal.set_phase_native_drive_learning_enabled(false);
    let restored = learned_acquire(&mut causal, world);
    assert!(
        restored.success
            && (!lesioned_once.success || restored.interactions < lesioned_once.interactions),
        "exact physical synapse restoration must recover capability/cost without drive retraining"
    );
}

#[test]
fn p4_target_selector_has_no_p3_or_host_search_fallback() {
    let source = include_str!("../src/phase_drive.rs");
    let start = source
        .find("pub fn choose_phase_native_learned_drive_action")
        .expect("P4 selector");
    let end = source[start..]
        .find("pub(super) fn phase_native_drive_after_fact")
        .map(|offset| start + offset)
        .expect("P4 selector end");
    let selector = &source[start..end];

    for forbidden in [
        "phase_native_exploration_action",
        "choose_phase_native_autonomous_action",
        "EvoImaginationPlanner",
        "LearnedTransition",
        "VecDeque",
        "BinaryHeap",
        "FrontierNode",
    ] {
        assert!(
            !selector.contains(forbidden),
            "P4 selector contains forbidden fallback/search token {forbidden}"
        );
    }
    for required in [
        "phase_drive_frontier_activity",
        "phase_drive_features",
        "phase_drive_score",
        "pending_features",
    ] {
        assert!(selector.contains(required), "P4 selector missing {required}");
    }
}
