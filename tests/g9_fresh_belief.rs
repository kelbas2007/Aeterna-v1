mod g7_support;

use std::collections::BTreeSet;

use aeterna_v1::{
    BeliefConfig, EvoConfig, EvoPhase, EvoRasterField, PlanningConfig, RasterFieldConfig,
};
use g7_support::{perm4, relation, Rng};

const MATCH: f32 = 0.97;
const N_SUBSEEDS: usize = 10;
const WORLDS_PER_SUBSEED: usize = 8;
const N_WORLDS: usize = N_SUBSEEDS * WORLDS_PER_SUBSEED;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Context {
    A,
    B,
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    FullBelief,
    ObservationOnly,
    ResetHistory,
}

#[derive(Clone, Debug)]
struct WorldSpec {
    sub_seed: usize,
    context: Context,
    perm: [usize; 4],
    history_len: usize,
    origin: (usize, usize),
    codes: [u64; 5], // cue A, cue B, corridor, goal, dead
}

#[derive(Clone, Copy, Debug, Default)]
struct Outcome {
    success: bool,
    physical_actions: usize,
    terminal_action: usize,
    authority_ok: bool,
}

#[derive(Debug)]
struct Stats {
    success: usize,
    physical_actions: usize,
    per_subseed: [usize; N_SUBSEEDS],
    history_success: [usize; 4],
    history_total: [usize; 4],
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            success: 0,
            physical_actions: 0,
            per_subseed: [0; N_SUBSEEDS],
            history_success: [0; 4],
            history_total: [0; 4],
        }
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

fn seal_field() -> EvoRasterField {
    EvoRasterField::new(RasterFieldConfig::for_raster(12, 12, 4), 192)
}

fn trace_set_is_identifiable(field: &EvoRasterField, codes: &[u64; 5]) -> bool {
    let traces = codes
        .iter()
        .map(|code| {
            field
                .encode_relational_trace(&relation(*code, 0, 0))
                .expect("fresh G9 relation trace")
        })
        .collect::<Vec<_>>();

    for i in 0..traces.len() {
        for j in (i + 1)..traces.len() {
            if traces[i].similarity(&traces[j]) >= MATCH {
                return false;
            }
        }
    }
    true
}

fn sealed_codes(rng: &mut Rng, field: &EvoRasterField) -> ([u64; 5], usize) {
    for rejected in 0..20_000usize {
        let mut codes = [0u64; 5];
        for code in &mut codes {
            *code = rng.next_u64();
        }
        if trace_set_is_identifiable(field, &codes) {
            return (codes, rejected);
        }
    }
    panic!("G9 fresh generator exhausted representation-integrity budget");
}

fn generate_pack(authority_seed: u64) -> (Vec<WorldSpec>, usize) {
    let field = seal_field();
    let mut worlds = Vec::with_capacity(N_WORLDS);
    let mut rejected = 0usize;

    for sub_seed in 0..N_SUBSEEDS {
        let mut rng = Rng::new(
            authority_seed
                ^ (sub_seed as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        );

        for world_index in 0..WORLDS_PER_SUBSEED {
            let perm = perm4(&mut rng);
            let context = if world_index % 2 == 0 {
                Context::A
            } else {
                Context::B
            };
            let history_len = 1 + (world_index % 3);
            let origin = (5 + rng.range(3), 5 + rng.range(3));
            let (codes, local_rejected) = sealed_codes(&mut rng, &field);
            rejected = rejected.saturating_add(local_rejected);

            worlds.push(WorldSpec {
                sub_seed,
                context,
                perm,
                history_len,
                origin,
                codes,
            });
        }
    }

    (worlds, rejected)
}

fn pack_digest(worlds: &[WorldSpec]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for world in worlds {
        mix_u64(&mut hash, world.sub_seed as u64);
        mix_u64(
            &mut hash,
            match world.context {
                Context::A => 0,
                Context::B => 1,
            },
        );
        for motor in world.perm {
            mix_u64(&mut hash, motor as u64);
        }
        mix_u64(&mut hash, world.history_len as u64);
        mix_u64(&mut hash, world.origin.0 as u64);
        mix_u64(&mut hash, world.origin.1 as u64);
        for code in world.codes {
            mix_u64(&mut hash, code);
        }
    }
    hash
}

fn cue(world: &WorldSpec, context: Context, origin: (usize, usize)) -> Vec<f32> {
    relation(
        match context {
            Context::A => world.codes[0],
            Context::B => world.codes[1],
        },
        origin.0,
        origin.1,
    )
}

fn corridor(world: &WorldSpec, origin: (usize, usize)) -> Vec<f32> {
    relation(world.codes[2], origin.0, origin.1)
}

fn goal(world: &WorldSpec, origin: (usize, usize)) -> Vec<f32> {
    relation(world.codes[3], origin.0, origin.1)
}

fn dead(world: &WorldSpec, origin: (usize, usize)) -> Vec<f32> {
    relation(world.codes[4], origin.0, origin.1)
}

fn correct_terminal(context: Context, perm: [usize; 4]) -> usize {
    match context {
        Context::A => perm[1],
        Context::B => perm[2],
    }
}

fn organism(with_belief: bool) -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 4;
    cfg.dormant_cells = 96;
    cfg.hdc_dim = 192;

    let mut evo = EvoPhase::new(cfg);
    let mut raster = RasterFieldConfig::for_raster(12, 12, 4);
    raster.learning_enabled = false;
    raster.readout_enabled = false;
    evo.attach_raster_field(raster);

    let mut planning = PlanningConfig::new(4);
    planning.max_depth = 4;
    planning.node_budget = 256;
    planning.discount = 0.95;
    evo.enable_imagination_planner(planning);

    if with_belief {
        evo.enable_belief_state(BeliefConfig::new(4, 192));
    }

    evo
}

fn reconstruct_full_belief(
    evo: &mut EvoPhase,
    world: &WorldSpec,
    context: Context,
    origin: (usize, usize),
    stage: usize,
) {
    let pre = cue(world, context, origin);
    let amb = corridor(world, origin);
    evo.reset_belief(&pre).expect("fresh G9 cue belief");
    for _ in 0..stage {
        evo.advance_belief(world.perm[0], &amb)
            .expect("fresh G9 recurrent prefix");
    }
}

fn factual_transition(
    world: &WorldSpec,
    context: Context,
    stage: usize,
    action: usize,
    origin: (usize, usize),
) -> (Vec<f32>, f32) {
    if stage < world.history_len {
        if action == world.perm[0] {
            (corridor(world, origin), 0.0)
        } else {
            (dead(world, origin), 0.0)
        }
    } else if action == correct_terminal(context, world.perm) {
        (goal(world, origin), 1.0)
    } else {
        (dead(world, origin), 0.0)
    }
}

fn train(world: &WorldSpec, mode: Mode) -> EvoPhase {
    let with_belief = !matches!(mode, Mode::ObservationOnly);
    let mut evo = organism(with_belief);
    let origins = [(0usize, 0usize), (4, 0), (0, 4), (4, 4)];

    for context in [Context::A, Context::B] {
        for origin in origins {
            for stage in 0..=world.history_len {
                let current = if stage == 0 {
                    cue(world, context, origin)
                } else {
                    corridor(world, origin)
                };

                for action in 0..4usize {
                    let (post, reward) =
                        factual_transition(world, context, stage, action, origin);

                    match mode {
                        Mode::ObservationOnly => {
                            evo.observe_planning_transition(
                                &current,
                                action,
                                &post,
                                reward,
                            );
                        }
                        Mode::ResetHistory => {
                            evo.reset_belief(&current)
                                .expect("fresh G9 reset-history state");
                            evo.observe_belief_planning_transition(
                                action,
                                &post,
                                reward,
                            )
                            .expect("fresh G9 reset transition");
                        }
                        Mode::FullBelief => {
                            reconstruct_full_belief(
                                &mut evo,
                                world,
                                context,
                                origin,
                                stage,
                            );
                            evo.observe_belief_planning_transition(
                                action,
                                &post,
                                reward,
                            )
                            .expect("fresh G9 belief transition");
                        }
                    }
                }
            }
        }
    }

    evo.set_planning_learning_enabled(false);
    evo
}

fn belief_at_final_corridor(
    mature: &EvoPhase,
    world: &WorldSpec,
    context: Context,
) -> aeterna_v1::CarrierTrace {
    let mut evo = mature.clone();
    reconstruct_full_belief(
        &mut evo,
        world,
        context,
        world.origin,
        world.history_len,
    );
    evo.current_belief_trace()
        .expect("fresh G9 final corridor belief")
}

fn same_real(
    a: Option<&aeterna_v1::FactualFrame>,
    b: Option<&aeterna_v1::FactualFrame>,
) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.sensory == b.sensory && a.need == b.need && a.tick == b.tick,
        (None, None) => true,
        _ => false,
    }
}

fn execute(world: &WorldSpec, mature: &EvoPhase, mode: Mode) -> Outcome {
    let mut evo = mature.clone();
    let mut physical_actions = 0usize;
    let mut terminal_action = usize::MAX;
    let mut authority_ok = true;

    let mut current = cue(world, world.context, world.origin);
    evo.observe_initial_real(&current, false);

    if !matches!(mode, Mode::ObservationOnly) {
        evo.reset_belief(&current).expect("fresh G9 held-out cue belief");
    }

    for stage in 0..=world.history_len {
        if stage > 0 {
            current = corridor(world, world.origin);
            evo.observe_initial_real(&current, false);
            match mode {
                Mode::FullBelief => {
                    evo.advance_belief(world.perm[0], &current)
                        .expect("fresh G9 held-out recurrent advance");
                }
                Mode::ResetHistory => {
                    evo.reset_belief(&current)
                        .expect("fresh G9 held-out reset history");
                }
                Mode::ObservationOnly => {}
            }
        }

        let before = evo.current_real().cloned();
        let decision = match mode {
            Mode::ObservationOnly => evo.plan_imagined(&current),
            Mode::FullBelief | Mode::ResetHistory => evo.plan_from_belief(),
        };
        let after = evo.current_real().cloned();
        authority_ok &= same_real(before.as_ref(), after.as_ref());

        let Some(decision) = decision else {
            return Outcome {
                success: false,
                physical_actions,
                terminal_action,
                authority_ok: false,
            };
        };
        physical_actions = physical_actions.saturating_add(1);

        if stage < world.history_len {
            if decision.first_action != world.perm[0] {
                return Outcome {
                    success: false,
                    physical_actions,
                    terminal_action,
                    authority_ok,
                };
            }
        } else {
            terminal_action = decision.first_action;
            return Outcome {
                success: terminal_action == correct_terminal(world.context, world.perm),
                physical_actions,
                terminal_action,
                authority_ok,
            };
        }
    }

    Outcome {
        success: false,
        physical_actions,
        terminal_action,
        authority_ok,
    }
}

fn accumulate(stats: &mut Stats, world: &WorldSpec, outcome: Outcome) {
    stats.success += usize::from(outcome.success);
    stats.physical_actions += outcome.physical_actions;
    stats.per_subseed[world.sub_seed] += usize::from(outcome.success);
    stats.history_total[world.history_len] += 1;
    stats.history_success[world.history_len] += usize::from(outcome.success);
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

fn score(
    worlds: &[WorldSpec],
    log_worlds: bool,
) -> (
    Stats,
    Stats,
    Stats,
    usize,
    usize,
    BTreeSet<usize>,
) {
    let mut full = Stats::default();
    let mut observation = Stats::default();
    let mut reset = Stats::default();
    let mut belief_separation_violations = 0usize;
    let mut authority_violations = 0usize;
    let mut terminal_ids = BTreeSet::new();

    for (idx, world) in worlds.iter().enumerate() {
        let full_model = train(world, Mode::FullBelief);
        let observation_model = train(world, Mode::ObservationOnly);
        let reset_model = train(world, Mode::ResetHistory);

        // The current final corridor observation is literally identical across
        // evaluator contexts. Only the recurrent carrier history may differ.
        let corridor_a = corridor(world, world.origin);
        let corridor_b = corridor(world, world.origin);
        if corridor_a != corridor_b {
            belief_separation_violations += 1;
        }

        let belief_a =
            belief_at_final_corridor(&full_model, world, Context::A);
        let belief_b =
            belief_at_final_corridor(&full_model, world, Context::B);
        if belief_a.similarity(&belief_b) >= MATCH {
            belief_separation_violations += 1;
        }

        let f = execute(world, &full_model, Mode::FullBelief);
        let o = execute(world, &observation_model, Mode::ObservationOnly);
        let r = execute(world, &reset_model, Mode::ResetHistory);

        authority_violations += usize::from(!f.authority_ok);
        authority_violations += usize::from(!o.authority_ok);
        authority_violations += usize::from(!r.authority_ok);

        if f.success {
            terminal_ids.insert(f.terminal_action);
        }

        accumulate(&mut full, world, f);
        accumulate(&mut observation, world, o);
        accumulate(&mut reset, world, r);

        if log_worlds {
            println!(
                "G9_FRESH_WORLD idx={} sub={} context={:?} history={} origin={:?} perm={:?} full={:?} observation={:?} reset={:?}",
                idx,
                world.sub_seed,
                world.context,
                world.history_len,
                world.origin,
                world.perm,
                f,
                o,
                r
            );
        }
    }

    (
        full,
        observation,
        reset,
        belief_separation_violations,
        authority_violations,
        terminal_ids,
    )
}

#[test]
fn g9_fresh_preflight() {
    let (worlds, rejected) = generate_pack(0x9202_6000_0000_0001);
    let sample = &worlds[..12];
    let (full, observation, reset, separation, authority, _) =
        score(sample, false);

    println!(
        "G9_FRESH_PREFLIGHT full={}/12 observation={}/12 reset={}/12 separation={} authority={} rejected={}",
        full.success,
        observation.success,
        reset.success,
        separation,
        authority,
        rejected
    );

    assert!(full.success >= 11);
    assert!(observation.success <= 8);
    assert!(reset.success <= 8);
    assert_eq!(separation, 0);
    assert_eq!(authority, 0);
}

#[test]
fn g9_fresh_one_use_qualification() {
    if std::env::var("AETERNA_RUN_FRESH_G9").ok().as_deref() != Some("1") {
        println!("G9_FRESH_SKIPPED sealed qualification not requested");
        return;
    }

    let run_id: u64 = std::env::var("GITHUB_RUN_ID")
        .expect("fresh G9 qualification requires external GITHUB_RUN_ID")
        .parse()
        .expect("GITHUB_RUN_ID must be numeric");
    let source_sha = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "unknown".into());
    let protocol = include_str!("../docs/G9_FRESH_BELIEF_QUALIFICATION.md");
    let protocol_fingerprint = fnv64_bytes(protocol.as_bytes());

    let (worlds, rejected) = generate_pack(run_id);
    assert_eq!(worlds.len(), N_WORLDS);
    let digest = pack_digest(&worlds);

    println!(
        "G9_FRESH_SEAL source_sha={} authority_seed={} protocol_fnv={:016x} pack_digest={:016x} n={} subseeds={} representation_rejected={}",
        source_sha,
        run_id,
        protocol_fingerprint,
        digest,
        worlds.len(),
        N_SUBSEEDS,
        rejected
    );

    let (full, observation, reset, separation, authority, terminal_ids) =
        score(&worlds, true);
    let (lo, hi) = wilson(full.success, N_WORLDS);

    println!(
        "G9_FRESH_RESULT full={}/{} wilson95=[{:.4},{:.4}] observation={}/{} reset={}/{} separation_violations={} authority_violations={} per_subseed={:?} history_success={:?}/{:?} mean_actions=[{:.3},{:.3},{:.3}] terminal_ids={:?} representation_rejected={}",
        full.success,
        N_WORLDS,
        lo,
        hi,
        observation.success,
        N_WORLDS,
        reset.success,
        N_WORLDS,
        separation,
        authority,
        full.per_subseed,
        full.history_success,
        full.history_total,
        full.physical_actions as f64 / N_WORLDS as f64,
        observation.physical_actions as f64 / N_WORLDS as f64,
        reset.physical_actions as f64 / N_WORLDS as f64,
        terminal_ids,
        rejected
    );

    assert!(full.success >= 72);
    assert!(lo > 0.80);

    for history in 1..=3 {
        assert!(full.history_total[history] > 0);
        assert!(
            full.history_success[history] * 100
                >= 80 * full.history_total[history],
            "history length {} must reach >=80%",
            history
        );
    }

    assert!(observation.success <= 48);
    assert!(reset.success <= 48);
    assert!(full.success >= observation.success + 24);
    assert!(full.success >= reset.success + 24);
    assert_eq!(separation, 0);
    assert_eq!(authority, 0);
    assert!(terminal_ids.len() >= 3);
}
