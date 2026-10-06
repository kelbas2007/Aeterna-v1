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
        let mut full = cold_with_drive(learned_checkpoint.clone());        let before_drive = full.phase_native_drive_weights().unwrap();
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

struct FreshRng(u64);

impl FreshRng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
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
struct FreshBlock {
    source: Vec<World>,
    target: Vec<World>,
    random_seeds: Vec<u64>,
}

fn fresh_world(rng: &mut FreshRng, min_len: usize, max_len: usize) -> World {
    let length = min_len + rng.range(max_len - min_len + 1);
    let advance = (0..length).map(|_| rng.range(3)).collect::<Vec<_>>();
    World { length, advance }
}

fn fnv_mix(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn fresh_blocks(authority: u64) -> (Vec<FreshBlock>, u64) {
    let mut blocks = Vec::new();
    let mut digest = 14_695_981_039_346_656_037_u64;

    for sub in 0..10u64 {
        let derived = authority
            ^ sub.wrapping_mul(0xD1B5_4A32_D192_ED03)
            ^ 0xA37E_4F34_6C91_2B55;
        let mut rng = FreshRng::new(derived);
        let source = (0..8)
            .map(|_| fresh_world(&mut rng, 2, 3))
            .collect::<Vec<_>>();
        let target = (0..8)
            .map(|_| fresh_world(&mut rng, 4, 5))
            .collect::<Vec<_>>();
        let random_seeds = (0..8).map(|_| rng.next()).collect::<Vec<_>>();

        digest = fnv_mix(digest, sub);
        for (kind, worlds) in [(0u64, &source), (1u64, &target)] {
            digest = fnv_mix(digest, kind);
            for world in worlds {
                digest = fnv_mix(digest, world.length as u64);
                for action in &world.advance {
                    digest = fnv_mix(digest, *action as u64);
                }
            }
        }
        for seed in &random_seeds {
            digest = fnv_mix(digest, *seed);
        }

        blocks.push(FreshBlock {
            source,
            target,
            random_seeds,
        });
    }

    (blocks, digest)
}

fn train_fresh_drive(
    source: &[World],
    phase_learning: bool,
) -> (PhaseDriveCheckpoint, usize) {
    let mut checkpoint = None;
    let mut interactions = 0usize;

    for (index, world) in source.iter().enumerate() {
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
        assert!(
            outcome.success,
            "fresh source world {index} failed P3 bootstrap: {world:?}"
        );
        interactions += outcome.interactions;
        checkpoint = evo.phase_native_drive_checkpoint();
    }

    (
        checkpoint.expect("fresh drive checkpoint after eight source worlds"),
        interactions,
    )
}

fn wilson95(success: usize, n: usize) -> (f64, f64) {
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

fn mean_sd(outcomes: &[Outcome]) -> (f64, f64) {
    let mean = outcomes.iter().map(|o| o.interactions).sum::<usize>() as f64
        / outcomes.len() as f64;
    let variance = outcomes
        .iter()
        .map(|o| (o.interactions as f64 - mean).powi(2))
        .sum::<f64>()
        / outcomes.len().saturating_sub(1).max(1) as f64;
    (mean, variance.sqrt())
}

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from first-attempt CI"]
fn p4_fresh_learned_drive_transfer_pack() {
    let authority: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse()
        .expect("fresh seed must be u64");
    let source_sha = std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha = std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let (blocks, digest) = fresh_blocks(authority);

    println!(
        "FRESH_P4_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha, spec_sha, authority, digest
    );
    for (sub, block) in blocks.iter().enumerate() {
        println!(
            "FRESH_P4_BLOCK sub={} source={:?} target={:?} random_seeds={:?}",
            sub, block.source, block.target, block.random_seeds
        );
    }

    let mut full = Vec::new();
    let mut zero = Vec::new();
    let mut lesion = Vec::new();
    let mut zero_phase = Vec::new();
    let mut random = Vec::new();
    let mut teacher = Vec::new();
    let mut per_seed = Vec::new();
    let mut meta_interactions = 0usize;
    let mut learned_weights = Vec::new();
    let mut learned_observations = Vec::new();

    for (sub, block) in blocks.iter().enumerate() {
        let (checkpoint, source_cost) = train_fresh_drive(&block.source, true);
        let (zero_phase_checkpoint, _) = train_fresh_drive(&block.source, false);
        meta_interactions += source_cost;

        let probe = cold_with_drive(checkpoint.clone());
        let weights = probe.phase_native_drive_weights().expect("fresh learned drive weights");
        assert!(
            weights[0] > 0.05 && weights[1] > 0.05,
            "sub-seed {sub} failed positive learned-drive weight gate: {weights:?}"
        );
        learned_weights.push(weights);
        learned_observations.push(probe.phase_native_drive_observations());

        let mut sub_success = 0usize;
        for (case, world) in block.target.iter().enumerate() {
            let mut full_evo = cold_with_drive(checkpoint.clone());
            assert_eq!(full_evo.phase_native_receptor_count(), 0);
            assert_eq!(full_evo.phase_native_circuits().len(), 0);
            assert_eq!(full_evo.planning_transition_count(), 0);
            let before = full_evo.phase_native_drive_weights().unwrap();
            let full_outcome = learned_acquire(&mut full_evo, world);
            assert_eq!(
                full_evo.phase_native_drive_weights().unwrap(),
                before,
                "target drive learning changed on sub={sub} case={case}"
            );
            sub_success += usize::from(full_outcome.success);
            full.push(full_outcome);

            let mut zero_evo = carrier();
            assert!(zero_evo.enable_phase_native_learned_drive(PhaseDriveConfig {
                learning_enabled: false,
                ..PhaseDriveConfig::default()
            }));
            zero_evo.set_phase_native_drive_learning_enabled(false);
            zero.push(learned_acquire(&mut zero_evo, world));

            let mut lesioned = cold_with_drive(checkpoint.clone());
            let frontier_synapse = lesioned.phase_native_drive_synapses().unwrap()[1];
            assert!(lesioned
                .perturb_phase_native_synapse_for_control(frontier_synapse, 0.0, 0.0)
                .is_some());
            lesion.push(learned_acquire(&mut lesioned, world));

            let mut no_phase = cold_with_drive(zero_phase_checkpoint.clone());
            zero_phase.push(learned_acquire(&mut no_phase, world));

            random.push(random_acquire(world, block.random_seeds[case]));

            let mut ceiling = carrier();
            teacher.push(teacher_acquire(&mut ceiling, world));
        }
        per_seed.push(sub_success);
    }

    let n = full.len();
    assert_eq!(n, 80);

    let full_success = full.iter().filter(|o| o.success).count();
    let zero_success = zero.iter().filter(|o| o.success).count();
    let lesion_success = lesion.iter().filter(|o| o.success).count();
    let zero_phase_success = zero_phase.iter().filter(|o| o.success).count();
    let random_success = random.iter().filter(|o| o.success).count();
    let teacher_success = teacher.iter().filter(|o| o.success).count();

    let (lo, hi) = wilson95(full_success, n);
    let (full_mean, full_sd) = mean_sd(&full);
    let (lesion_mean, lesion_sd) = mean_sd(&lesion);
    let (zero_phase_mean, zero_phase_sd) = mean_sd(&zero_phase);
    let (random_mean, random_sd) = mean_sd(&random);

    println!(
        "FRESH_P4_RESULT N={} full={}/{} wilson95=[{:.6},{:.6}] per_seed={:?} zero={}/{} lesion={}/{} zero_phase={}/{} random={}/{} p3_teacher={}/{}",
        n,
        full_success,
        n,
        lo,
        hi,
        per_seed,
        zero_success,
        n,
        lesion_success,
        n,
        zero_phase_success,
        n,
        random_success,
        n,
        teacher_success,
        n,
    );
    println!(
        "FRESH_P4_COST full_mean={:.3} full_sd={:.3} lesion_mean={:.3} lesion_sd={:.3} zero_phase_mean={:.3} zero_phase_sd={:.3} random_mean={:.3} random_sd={:.3} source_meta_interactions={}",
        full_mean,
        full_sd,
        lesion_mean,
        lesion_sd,
        zero_phase_mean,
        zero_phase_sd,
        random_mean,
        random_sd,
        meta_interactions,
    );
    println!(
        "FRESH_P4_DRIVE weights={:?} observations={:?}",
        learned_weights, learned_observations
    );

    let full_rate = full_success as f64 / n as f64;
    let lesion_rate = lesion_success as f64 / n as f64;
    let zero_phase_rate = zero_phase_success as f64 / n as f64;
    let random_rate = random_success as f64 / n as f64;

    assert!(full_success >= 76, "FULL_LEARNED_DRIVE must achieve >=76/80");
    assert!(lo >= 0.87, "Wilson 95% lower bound must be >=0.87");
    assert!(
        per_seed.iter().all(|success| *success >= 6),
        "every sub-seed must achieve >=6/8: {per_seed:?}"
    );
    assert!(zero_success <= 40, "ZERO_DRIVE must be <=40/80");
    assert!(
        full_rate - lesion_rate >= 0.20 || lesion_mean >= full_mean + 10.0,
        "FRONTIER_LESION must have >=0.20 success loss or >=10 extra interactions"
    );
    assert!(
        full_rate - zero_phase_rate >= 0.20 || zero_phase_mean >= full_mean + 10.0,
        "ZERO_PHASE_DRIVE must have >=0.20 success loss or >=10 extra interactions"
    );
    assert!(
        full_rate - random_rate >= 0.20 || random_mean > full_mean,
        "RANDOM_ACTION must have >=0.20 success loss or higher mean cost"
    );
    assert!(teacher_success >= 76, "P3 teacher sanity ceiling must solve >=76/80");
    assert!(full_mean <= 35.0, "FULL mean acquisition cost must be <=35");
}
