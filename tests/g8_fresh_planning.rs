mod g7_support;

use std::collections::BTreeSet;

use aeterna_v1::{Authority, EvoConfig, EvoPhase, PlanningConfig, RasterFieldConfig};
use g7_support::{perm4, render_relation, Rng, DISTRACTOR, DROPOUT, ROTATE, SCALE};

const N_SUBSEEDS: usize = 10;
const WORLDS_PER_SUBSEED: usize = 8;
const N_WORLDS: usize = N_SUBSEEDS * WORLDS_PER_SUBSEED;

#[derive(Clone, Copy, Debug)]
enum Arm {
    Full,
    Depth1,
    Immediate,
    Shuffled,
}

#[derive(Clone, Debug)]
struct WorldSpec {
    sub_seed: usize,
    perm: [usize; 4],
    depth: usize,
    nuisance: u8,
    sensor_seed: u64,
    codes: [u64; 7], // start, r1, r2, r3, goal, trap, dead
}

#[derive(Clone, Copy, Debug, Default)]
struct Outcome {
    success: bool,
    first_action: usize,
    physical_actions: usize,
    rollout_nodes: usize,
    first_selected_depth: usize,
}

#[derive(Debug)]
struct Stats {
    success: usize,
    physical_actions: usize,
    rollout_nodes: usize,
    per_subseed: [usize; N_SUBSEEDS],
    depth_success: [usize; 5],
    depth_total: [usize; 5],
    nuisance_success: [usize; 5],
    nuisance_total: [usize; 5],
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            success: 0,
            physical_actions: 0,
            rollout_nodes: 0,
            per_subseed: [0; N_SUBSEEDS],
            depth_success: [0; 5],
            depth_total: [0; 5],
            nuisance_success: [0; 5],
            nuisance_total: [0; 5],
        }
    }
}

fn nuisance_index(mask: u8) -> usize {
    match mask {
        0 => 0,
        DISTRACTOR => 1,
        DROPOUT => 2,
        ROTATE => 3,
        SCALE => 4,
        _ => panic!("G8-FRESH uses only preregistered single nuisances"),
    }
}

fn fnv64_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn mix_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= byte as u64;
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn active_signature(raster: &[f32]) -> Vec<usize> {
    raster
        .iter()
        .enumerate()
        .filter_map(|(idx, value)| (*value >= 0.5).then_some(idx))
        .collect()
}

fn unique_state_codes(rng: &mut Rng) -> [u64; 7] {
    let mut codes = [0u64; 7];
    let mut signatures: Vec<Vec<usize>> = Vec::new();

    for slot in 0..7 {
        loop {
            let code = rng.next_u64();
            let raster = render_relation(code, 0, 0, 0, 0);
            let signature = active_signature(&raster);
            if !signatures.contains(&signature) {
                codes[slot] = code;
                signatures.push(signature);
                break;
            }
        }
    }

    codes
}

fn generate_pack(authority_seed: u64) -> Vec<WorldSpec> {
    let mut worlds = Vec::with_capacity(N_WORLDS);

    for sub_seed in 0..N_SUBSEEDS {
        let mut rng = Rng::new(
            authority_seed
                ^ (sub_seed as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        );
        let perm = perm4(&mut rng);

        for world_index in 0..WORLDS_PER_SUBSEED {
            let depth = match world_index {
                0 => 2,
                1 => 3,
                2 => 4,
                _ => 2 + rng.range(3),
            };
            let nuisance = match world_index {
                4 => DISTRACTOR,
                5 => DROPOUT,
                6 => ROTATE,
                7 => SCALE,
                _ => 0,
            };

            worlds.push(WorldSpec {
                sub_seed,
                perm,
                depth,
                nuisance,
                sensor_seed: rng.next_u64(),
                codes: unique_state_codes(&mut rng),
            });
        }
    }

    worlds
}

fn pack_digest(worlds: &[WorldSpec]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for world in worlds {
        mix_u64(&mut hash, world.sub_seed as u64);
        for motor in world.perm {
            mix_u64(&mut hash, motor as u64);
        }
        mix_u64(&mut hash, world.depth as u64);
        mix_u64(&mut hash, world.nuisance as u64);
        mix_u64(&mut hash, world.sensor_seed);
        for code in world.codes {
            mix_u64(&mut hash, code);
        }
    }
    hash
}

fn organism() -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 4;
    cfg.dormant_cells = 160;
    cfg.hdc_dim = 192;

    let mut evo = EvoPhase::new(cfg);
    let mut raster = RasterFieldConfig::for_raster(12, 12, 4);
    raster.learning_enabled = false;
    raster.readout_enabled = false;
    evo.attach_raster_field(raster);
    evo.set_robust_high_level_perception(true);

    let mut planning = PlanningConfig::new(4);
    planning.max_depth = 4;
    planning.node_budget = 256;
    planning.discount = 0.95;
    evo.enable_imagination_planner(planning);
    evo
}

fn render_clean(code: u64, origin: (usize, usize)) -> Vec<f32> {
    render_relation(code, origin.0, origin.1, 0, 0)
}

fn train_world_model(evo: &mut EvoPhase, world: &WorldSpec) {
    for origin in [(0usize, 0usize), (4, 0), (0, 4)] {
        let start = render_clean(world.codes[0], origin);
        let trap = render_clean(world.codes[5], origin);
        let dead = render_clean(world.codes[6], origin);
        let route = [
            render_clean(world.codes[1], origin),
            render_clean(world.codes[2], origin),
            render_clean(world.codes[3], origin),
        ];
        let goal = render_clean(world.codes[4], origin);

        evo.observe_planning_transition(&start, world.perm[0], &trap, 0.30);
        evo.observe_planning_transition(&start, world.perm[1], &route[0], 0.0);
        evo.observe_planning_transition(&start, world.perm[2], &dead, 0.0);
        evo.observe_planning_transition(&start, world.perm[3], &dead, 0.0);

        for route_idx in 0..(world.depth - 1) {
            let from = &route[route_idx];
            let is_final = route_idx + 1 == world.depth - 1;
            let to = if is_final {
                &goal
            } else {
                &route[route_idx + 1]
            };

            for logical in 0..4 {
                let action = world.perm[logical];
                if action == world.perm[3] {
                    evo.observe_planning_transition(
                        from,
                        action,
                        to,
                        if is_final { 1.0 } else { 0.0 },
                    );
                } else {
                    evo.observe_planning_transition(from, action, &dead, 0.0);
                }
            }
        }
    }

    evo.set_planning_learning_enabled(false);
}

fn state_raster(world: &WorldSpec, state: usize, step: usize) -> Vec<f32> {
    render_relation(
        world.codes[state],
        5,
        5,
        world.nuisance,
        world.sensor_seed ^ (step as u64).wrapping_mul(0xD1B5_4A32_D192_ED03),
    )
}

fn execute(world: &WorldSpec, mature: &EvoPhase, arm: Arm) -> Outcome {
    let mut evo = mature.clone();
    if matches!(arm, Arm::Shuffled) {
        evo.permute_planning_successors_for_control();
    }

    let mut state = 0usize; // start
    let mut physical_actions = 0usize;
    let mut rollout_nodes = 0usize;
    let mut first_action = usize::MAX;
    let mut first_selected_depth = 0usize;

    for step in 0..world.depth {
        let sensory = state_raster(world, state, step);

        if step == 0 && matches!(arm, Arm::Full) {
            evo.observe_initial_real(&sensory, false);
            let before = evo.current_real().expect("REAL PRE before imagination").clone();
            let decision = evo.plan_imagined(&sensory).expect("FULL must produce a plan");
            let after = evo.current_real().expect("REAL preserved after imagination");
            assert_eq!(before.sensory, after.sensory);
            assert_eq!(before.need, after.need);
            assert_eq!(before.tick, after.tick);
            assert_eq!(decision.authority, Authority::Imagined);
        }

        let decision = match arm {
            Arm::Full | Arm::Shuffled => evo
                .plan_imagined(&sensory)
                .expect("imagined planner must return a decision"),
            Arm::Depth1 => evo
                .plan_imagined_depth(&sensory, 1)
                .expect("depth-1 planner must return a decision"),
            Arm::Immediate => evo
                .choose_immediate_model(&sensory)
                .expect("immediate MODEL control must return a decision"),
        };

        match arm {
            Arm::Immediate => assert_eq!(decision.authority, Authority::Model),
            _ => assert_eq!(decision.authority, Authority::Imagined),
        }

        if step == 0 {
            first_action = decision.first_action;
            first_selected_depth = decision.selected_depth;
        }
        rollout_nodes = rollout_nodes.saturating_add(decision.expanded_nodes);
        physical_actions = physical_actions.saturating_add(1);

        let action = decision.first_action;

        if state == 0 {
            if action == world.perm[0] {
                return Outcome {
                    success: false,
                    first_action,
                    physical_actions,
                    rollout_nodes,
                    first_selected_depth,
                };
            }
            if action != world.perm[1] {
                return Outcome {
                    success: false,
                    first_action,
                    physical_actions,
                    rollout_nodes,
                    first_selected_depth,
                };
            }
            state = 1;
            continue;
        }

        if !(1..=3).contains(&state) || action != world.perm[3] {
            return Outcome {
                success: false,
                first_action,
                physical_actions,
                rollout_nodes,
                first_selected_depth,
            };
        }

        let route_idx = state - 1;
        let is_final = route_idx + 1 == world.depth - 1;
        if is_final {
            return Outcome {
                success: true,
                first_action,
                physical_actions,
                rollout_nodes,
                first_selected_depth,
            };
        }

        state += 1;
    }

    Outcome {
        success: false,
        first_action,
        physical_actions,
        rollout_nodes,
        first_selected_depth,
    }
}

fn accumulate(stats: &mut Stats, world: &WorldSpec, outcome: Outcome) {
    stats.success += usize::from(outcome.success);
    stats.physical_actions += outcome.physical_actions;
    stats.rollout_nodes += outcome.rollout_nodes;
    stats.per_subseed[world.sub_seed] += usize::from(outcome.success);
    stats.depth_total[world.depth] += 1;
    stats.depth_success[world.depth] += usize::from(outcome.success);
    let ni = nuisance_index(world.nuisance);
    stats.nuisance_total[ni] += 1;
    stats.nuisance_success[ni] += usize::from(outcome.success);
}

fn wilson(success: usize, total: usize) -> (f64, f64) {
    let n = total as f64;
    let p = success as f64 / n;
    let z = 1.959_963_984_540_054f64;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let margin =
        z * ((p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt()) / denom;
    (center - margin, center + margin)
}

#[test]
fn g8_fresh_one_use_planning_qualification() {
    if std::env::var("AETERNA_RUN_FRESH_G8").ok().as_deref() != Some("1") {
        println!("G8_FRESH_SKIPPED set AETERNA_RUN_FRESH_G8=1 only for sealed qualification");
        return;
    }

    let run_id: u64 = std::env::var("GITHUB_RUN_ID")
        .expect("fresh qualification requires external GITHUB_RUN_ID")
        .parse()
        .expect("GITHUB_RUN_ID must be numeric");
    let source_sha = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "unknown".into());

    let protocol = include_str!("../docs/G8_FRESH_PLANNING_QUALIFICATION.md");
    let protocol_fingerprint = fnv64_bytes(protocol.as_bytes());

    let worlds = generate_pack(run_id);
    assert_eq!(worlds.len(), N_WORLDS);
    let digest = pack_digest(&worlds);

    // Seal output is emitted before any world is scored.
    println!(
        "G8_FRESH_SEAL source_sha={} authority_seed={} protocol_fnv={:016x} pack_digest={:016x} n={} subseeds={}",
        source_sha,
        run_id,
        protocol_fingerprint,
        digest,
        worlds.len(),
        N_SUBSEEDS
    );

    let mut full = Stats::default();
    let mut depth1 = Stats::default();
    let mut immediate = Stats::default();
    let mut shuffled = Stats::default();
    let mut route_motor_ids = BTreeSet::new();

    for (world_index, world) in worlds.iter().enumerate() {
        let mut mature = organism();
        train_world_model(&mut mature, world);

        let full_outcome = execute(world, &mature, Arm::Full);
        let depth1_outcome = execute(world, &mature, Arm::Depth1);
        let immediate_outcome = execute(world, &mature, Arm::Immediate);
        let shuffled_outcome = execute(world, &mature, Arm::Shuffled);

        if full_outcome.success {
            assert_eq!(
                full_outcome.first_action, world.perm[1],
                "successful FULL world must follow the opaque delayed-route motor"
            );
            assert!(
                full_outcome.first_selected_depth >= world.depth,
                "FULL must see through the delayed reward horizon before first action"
            );
            route_motor_ids.insert(full_outcome.first_action);
        }

        accumulate(&mut full, world, full_outcome);
        accumulate(&mut depth1, world, depth1_outcome);
        accumulate(&mut immediate, world, immediate_outcome);
        accumulate(&mut shuffled, world, shuffled_outcome);

        println!(
            "G8_FRESH_WORLD idx={} sub={} depth={} nuisance={} perm={:?} full={} depth1={} immediate={} shuffled={}",
            world_index,
            world.sub_seed,
            world.depth,
            world.nuisance,
            world.perm,
            full_outcome.success,
            depth1_outcome.success,
            immediate_outcome.success,
            shuffled_outcome.success
        );
    }

    let (lo, hi) = wilson(full.success, N_WORLDS);
    println!(
        "G8_FRESH_RESULT full={}/{} wilson95=[{:.4},{:.4}] depth1={}/{} immediate={}/{} shuffled={}/{} full_subseeds={:?} depth_success={:?}/{:?} nuisance_success={:?}/{:?} mean_actions=[{:.3},{:.3},{:.3},{:.3}] mean_rollout_nodes=[{:.3},{:.3},{:.3}] route_motor_ids={:?}",
        full.success,
        N_WORLDS,
        lo,
        hi,
        depth1.success,
        N_WORLDS,
        immediate.success,
        N_WORLDS,
        shuffled.success,
        N_WORLDS,
        full.per_subseed,
        full.depth_success,
        full.depth_total,
        full.nuisance_success,
        full.nuisance_total,
        full.physical_actions as f64 / N_WORLDS as f64,
        depth1.physical_actions as f64 / N_WORLDS as f64,
        immediate.physical_actions as f64 / N_WORLDS as f64,
        shuffled.physical_actions as f64 / N_WORLDS as f64,
        full.rollout_nodes as f64 / N_WORLDS as f64,
        depth1.rollout_nodes as f64 / N_WORLDS as f64,
        shuffled.rollout_nodes as f64 / N_WORLDS as f64,
        route_motor_ids
    );

    assert!(full.success >= 72, "FULL must reach >=90% fresh success");
    assert!(lo > 0.80, "FULL Wilson 95% lower bound must exceed 0.80");

    for depth in 2..=4 {
        assert!(full.depth_total[depth] > 0);
        assert!(
            full.depth_success[depth] * 100 >= 80 * full.depth_total[depth],
            "each route depth must reach >=80%; depth={}",
            depth
        );
    }

    for nuisance in 1..=4 {
        assert!(full.nuisance_total[nuisance] > 0);
        assert!(
            full.nuisance_success[nuisance] * 100 >= 70 * full.nuisance_total[nuisance],
            "each nuisance category must reach >=70%; nuisance={}",
            nuisance
        );
    }

    assert!(
        full.success >= depth1.success + 32,
        "FULL must beat DEPTH1 by >=40 percentage points"
    );
    assert!(
        full.success >= immediate.success + 32,
        "FULL must beat NO_IMAGINATION/immediate MODEL by >=40 percentage points"
    );
    assert!(
        full.success >= shuffled.success + 24,
        "FULL must beat SHUFFLED_MODEL by >=30 percentage points"
    );
    assert!(
        route_motor_ids.len() >= 2,
        "success must follow opaque motor permutations rather than one fixed numeric motor"
    );
}
