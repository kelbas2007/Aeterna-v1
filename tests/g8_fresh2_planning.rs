mod g7_support;

use std::collections::BTreeSet;

use aeterna_v1::{
    Authority, EvoConfig, EvoPhase, EvoRasterField, PlanningConfig, RasterFieldConfig,
};
use g7_support::{perm4, render_relation, Rng, DISTRACTOR, DROPOUT, ROTATE, SCALE};

const MATCH: f32 = 0.97;
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
    codes: [u64; 7],
}

#[derive(Clone, Copy, Debug, Default)]
struct Outcome {
    success: bool,
    first_action: usize,
    physical_actions: usize,
    rollout_nodes: usize,
    first_selected_depth: usize,
    real_unchanged: bool,
    authority_ok: bool,
    no_imagination_nodes: bool,
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
        _ => panic!("G8-FRESH-2 uses only preregistered single nuisances"),
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

fn raw_unique_codes(rng: &mut Rng) -> [u64; 7] {
    let mut codes = [0u64; 7];
    let mut signatures: Vec<Vec<usize>> = Vec::new();

    for slot in 0..7 {
        loop {
            let code = rng.next_u64();
            let signature = active_signature(&render_relation(code, 0, 0, 0, 0));
            if !signatures.contains(&signature) {
                codes[slot] = code;
                signatures.push(signature);
                break;
            }
        }
    }

    codes
}

fn seal_field() -> EvoRasterField {
    EvoRasterField::new(RasterFieldConfig::for_raster(12, 12, 4), 192)
}

fn observable(
    field: &EvoRasterField,
    codes: &[u64; 7],
    nuisance: u8,
    sensor_seed: u64,
) -> bool {
    let mut clean = Vec::with_capacity(codes.len());
    let mut heldout = Vec::with_capacity(codes.len());

    for (state, code) in codes.iter().copied().enumerate() {
        let clean_raster = render_relation(code, 0, 0, 0, 0);
        let heldout_raster = render_relation(
            code,
            5,
            5,
            nuisance,
            sensor_seed ^ (state as u64).wrapping_mul(0xD1B5_4A32_D192_ED03),
        );

        let Some(clean_trace) = field.encode_robust_shape_trace(&clean_raster) else {
            return false;
        };
        let Some(heldout_trace) = field.encode_robust_shape_trace(&heldout_raster) else {
            return false;
        };
        clean.push(clean_trace);
        heldout.push(heldout_trace);
    }

    for i in 0..clean.len() {
        for j in (i + 1)..clean.len() {
            if clean[i].similarity(&clean[j]) >= MATCH
                || clean[j].similarity(&clean[i]) >= MATCH
            {
                return false;
            }
        }
    }

    for i in 0..heldout.len() {
        if clean[i].similarity(&heldout[i]) < MATCH
            || heldout[i].similarity(&clean[i]) < MATCH
        {
            return false;
        }
        for j in 0..clean.len() {
            if i == j {
                continue;
            }
            if heldout[i].similarity(&clean[j]) >= MATCH
                || clean[j].similarity(&heldout[i]) >= MATCH
            {
                return false;
            }
        }
    }

    true
}

fn sealed_codes(
    rng: &mut Rng,
    nuisance: u8,
    sensor_seed: u64,
    field: &EvoRasterField,
) -> ([u64; 7], usize) {
    for rejected in 0..20_000usize {
        let codes = raw_unique_codes(rng);
        if observable(field, &codes, nuisance, sensor_seed) {
            return (codes, rejected);
        }
    }
    panic!("observability seal could not generate a well-posed world within budget");
}

fn generate_pack(authority_seed: u64) -> (Vec<WorldSpec>, usize) {
    let field = seal_field();
    let mut worlds = Vec::with_capacity(N_WORLDS);
    let mut total_rejected = 0usize;

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
            let sensor_seed = rng.next_u64();
            let (codes, rejected) =
                sealed_codes(&mut rng, nuisance, sensor_seed, &field);
            total_rejected = total_rejected.saturating_add(rejected);

            worlds.push(WorldSpec {
                sub_seed,
                perm,
                depth,
                nuisance,
                sensor_seed,
                codes,
            });
        }
    }

    (worlds, total_rejected)
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

    let mut state = 0usize;
    let mut physical_actions = 0usize;
    let mut rollout_nodes = 0usize;
    let mut first_action = usize::MAX;
    let mut first_selected_depth = 0usize;
    let mut real_unchanged = true;
    let mut authority_ok = true;
    let mut no_imagination_nodes = true;

    for step in 0..world.depth {
        let sensory = state_raster(world, state, step);

        if step == 0 && matches!(arm, Arm::Full) {
            evo.observe_initial_real(&sensory, false);
        }
        let before_real = evo.current_real().cloned();

        let decision = match arm {
            Arm::Full | Arm::Shuffled => evo.plan_imagined(&sensory),
            Arm::Depth1 => evo.plan_imagined_depth(&sensory, 1),
            Arm::Immediate => evo.choose_immediate_model(&sensory),
        };

        let Some(decision) = decision else {
            return Outcome {
                success: false,
                first_action,
                physical_actions,
                rollout_nodes,
                first_selected_depth,
                real_unchanged,
                authority_ok: false,
                no_imagination_nodes,
            };
        };

        let after_real = evo.current_real().cloned();
        if before_real.as_ref().map(|r| (&r.sensory, r.need, r.tick))
            != after_real.as_ref().map(|r| (&r.sensory, r.need, r.tick))
        {
            real_unchanged = false;
        }

        match arm {
            Arm::Immediate => {
                authority_ok &= decision.authority == Authority::Model;
                no_imagination_nodes &= evo.imagined_rollout_nodes() == 0;
            }
            _ => {
                authority_ok &= decision.authority == Authority::Imagined;
            }
        }

        if step == 0 {
            first_action = decision.first_action;
            first_selected_depth = decision.selected_depth;
        }
        rollout_nodes = rollout_nodes.saturating_add(decision.expanded_nodes);
        physical_actions = physical_actions.saturating_add(1);

        let action = decision.first_action;
        if state == 0 {
            if action != world.perm[1] {
                return Outcome {
                    success: false,
                    first_action,
                    physical_actions,
                    rollout_nodes,
                    first_selected_depth,
                    real_unchanged,
                    authority_ok,
                    no_imagination_nodes,
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
                real_unchanged,
                authority_ok,
                no_imagination_nodes,
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
                real_unchanged,
                authority_ok,
                no_imagination_nodes,
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
        real_unchanged,
        authority_ok,
        no_imagination_nodes,
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

fn score_pack(worlds: &[WorldSpec], log_worlds: bool) -> (Stats, Stats, Stats, Stats, usize, usize, BTreeSet<usize>) {
    let mut full = Stats::default();
    let mut depth1 = Stats::default();
    let mut immediate = Stats::default();
    let mut shuffled = Stats::default();
    let mut integrity_violations = 0usize;
    let mut authority_violations = 0usize;
    let mut route_motor_ids = BTreeSet::new();

    for (world_index, world) in worlds.iter().enumerate() {
        let mut mature = organism();
        train_world_model(&mut mature, world);

        let f = execute(world, &mature, Arm::Full);
        let d = execute(world, &mature, Arm::Depth1);
        let i = execute(world, &mature, Arm::Immediate);
        let s = execute(world, &mature, Arm::Shuffled);

        if f.success {
            let integrity_ok =
                f.first_action == world.perm[1] && f.first_selected_depth >= world.depth;
            integrity_violations += usize::from(!integrity_ok);
            route_motor_ids.insert(f.first_action);
        }
        authority_violations += usize::from(!f.real_unchanged || !f.authority_ok);
        authority_violations += usize::from(!i.authority_ok || !i.no_imagination_nodes);

        accumulate(&mut full, world, f);
        accumulate(&mut depth1, world, d);
        accumulate(&mut immediate, world, i);
        accumulate(&mut shuffled, world, s);

        if log_worlds {
            println!(
                "G8_FRESH2_WORLD idx={} sub={} depth={} nuisance={} perm={:?} full={:?} depth1={:?} immediate={:?} shuffled={:?}",
                world_index,
                world.sub_seed,
                world.depth,
                world.nuisance,
                world.perm,
                f,
                d,
                i,
                s
            );
        }
    }

    (
        full,
        depth1,
        immediate,
        shuffled,
        integrity_violations,
        authority_violations,
        route_motor_ids,
    )
}

#[test]
fn g8_fresh2_observability_and_mechanism_preflight() {
    let (worlds, rejected) = generate_pack(0xA8E7_2026_0000_0002);
    assert_eq!(worlds.len(), N_WORLDS);
    let sample = &worlds[..12];
    let (full, depth1, immediate, shuffled, integrity, authority, _) =
        score_pack(sample, false);

    println!(
        "G8_FRESH2_PREFLIGHT full={}/12 depth1={}/12 immediate={}/12 shuffled={}/12 integrity={} authority={} seal_rejected={}",
        full.success, depth1.success, immediate.success, shuffled.success,
        integrity, authority, rejected
    );

    assert!(full.success >= 11);
    assert_eq!(integrity, 0);
    assert_eq!(authority, 0);
    assert!(full.success >= depth1.success + 6);
    assert!(full.success >= immediate.success + 6);
}

#[test]
fn g8_fresh2_one_use_planning_qualification() {
    if std::env::var("AETERNA_RUN_FRESH_G8_V2").ok().as_deref() != Some("1") {
        println!("G8_FRESH2_SKIPPED sealed qualification not requested");
        return;
    }

    let run_id: u64 = std::env::var("GITHUB_RUN_ID")
        .expect("fresh qualification requires external GITHUB_RUN_ID")
        .parse()
        .expect("GITHUB_RUN_ID must be numeric");
    let source_sha = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "unknown".into());
    let protocol = include_str!("../docs/G8_FRESH2_PLANNING_QUALIFICATION.md");
    let protocol_fingerprint = fnv64_bytes(protocol.as_bytes());

    let (worlds, rejected) = generate_pack(run_id);
    assert_eq!(worlds.len(), N_WORLDS);
    let digest = pack_digest(&worlds);

    println!(
        "G8_FRESH2_SEAL source_sha={} authority_seed={} protocol_fnv={:016x} pack_digest={:016x} n={} subseeds={} observability_rejected={}",
        source_sha,
        run_id,
        protocol_fingerprint,
        digest,
        worlds.len(),
        N_SUBSEEDS,
        rejected
    );

    let (full, depth1, immediate, shuffled, integrity, authority, route_motor_ids) =
        score_pack(&worlds, true);

    let (lo, hi) = wilson(full.success, N_WORLDS);
    println!(
        "G8_FRESH2_RESULT full={}/{} wilson95=[{:.4},{:.4}] depth1={}/{} immediate={}/{} shuffled={}/{} integrity_violations={} authority_violations={} full_subseeds={:?} depth_success={:?}/{:?} nuisance_success={:?}/{:?} mean_actions=[{:.3},{:.3},{:.3},{:.3}] mean_rollout_nodes=[{:.3},{:.3},{:.3}] route_motor_ids={:?} observability_rejected={}",
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
        integrity,
        authority,
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
        route_motor_ids,
        rejected
    );

    assert!(full.success >= 72, "FULL must reach >=90% fresh success");
    assert!(lo > 0.80, "FULL Wilson 95% lower bound must exceed 0.80");
    assert_eq!(integrity, 0, "no successful FULL world may use a shortened imagined horizon");
    assert_eq!(authority, 0, "authority/REAL firewall controls must remain intact");

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

    assert!(full.success >= depth1.success + 32);
    assert!(full.success >= immediate.success + 32);
    assert!(full.success >= shuffled.success + 24);
    assert!(route_motor_ids.len() >= 2);
}
