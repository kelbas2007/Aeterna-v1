use aeterna_v1::{
    EvoConfig, EvoPhase, ExplorationConfig, HierarchyConfig, MacroConfig, RasterFieldConfig,
};

pub const DISTRACTOR: u8 = 1;
pub const DROPOUT: u8 = 2;
pub const ROTATE: u8 = 4;
pub const SCALE: u8 = 8;

const CUE_BASE: u64 = 50_000;
const CHILD_PRE_CODE: u64 = 70_000;
const CHILD_MID_BASE: u64 = 71_000;

#[derive(Clone, Copy, Debug)]
pub struct World {
    pub family: usize,
    pub law: usize,
    pub informative: usize,
    pub origin: (usize, usize),
    pub nuisance: u8,
    pub sensor_seed: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum Arm {
    Full,
    ZeroExploration,
    NoHierarchy,
    UnrevisedChild,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Score {
    pub success: bool,
    pub probes: usize,
    pub candidate_evals: usize,
    pub primitive_actions: usize,
    pub rivals_at_start: usize,
    /// 0=start/rival activation, 1=epistemic loop, 2=identified law,
    /// 3=parent/sequence selected, 4=child execution completed.
    pub stage: u8,
}

#[derive(Clone)]
pub struct Mature {
    pub evo: EvoPhase,
    pub unrevised: EvoPhase,
    pub perm: [usize; 4],
    pub children: [u64; 2],
    pub tuition_probes: u32,
    pub child_tuition_actions: usize,
    pub parent_tuition_actions: usize,
    pub revision_actions: usize,
    pub revised_child_id: u64,
    pub pre_revision_counter: u64,
}

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn range(&mut self, upper: usize) -> usize {
        (self.next_u64() as usize) % upper
    }

    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            let j = self.range(i + 1);
            values.swap(i, j);
        }
    }
}

pub fn perm4(rng: &mut Rng) -> [usize; 4] {
    let mut values = [0usize, 1, 2, 3];
    rng.shuffle(&mut values);
    values
}

fn relation_points(code: u64) -> Vec<(usize, usize)> {
    let mut x = code ^ 0xD1B5_4A32_D192_ED03;
    let mut slots = Vec::new();
    while slots.len() < 4 {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        let slot = (x.wrapping_mul(0x2545_F491_4F6C_DD1D) % 16) as usize;
        if !slots.contains(&slot) {
            slots.push(slot);
        }
    }
    slots
        .into_iter()
        .map(|slot| (slot % 4, slot / 4))
        .collect()
}

pub fn relation(code: u64, ox: usize, oy: usize) -> Vec<f32> {
    render_relation(code, ox, oy, 0, 0)
}

pub fn render_relation(
    code: u64,
    ox: usize,
    oy: usize,
    nuisance: u8,
    sensor_seed: u64,
) -> Vec<f32> {
    let mut points = relation_points(code);
    let mut rng = Rng::new(sensor_seed ^ code.rotate_left(17));

    if nuisance & ROTATE != 0 {
        for point in &mut points {
            let (x, y) = *point;
            *point = (3 - y, x);
        }
    }

    if nuisance & SCALE != 0 {
        for point in &mut points {
            point.0 *= 2;
            point.1 *= 2;
        }
    }

    if nuisance & DROPOUT != 0 && points.len() > 2 {
        let idx = rng.range(points.len());
        points.remove(idx);
    }

    let mut raster = vec![0.0; 12 * 12];
    for (x, y) in points {
        let px = ox + x;
        let py = oy + y;
        assert!(px < 12 && py < 12, "generated G7 relation must stay in raster bounds");
        raster[py * 12 + px] = 1.0;
    }

    if nuisance & DISTRACTOR != 0 {
        for _ in 0..256 {
            let idx = rng.range(144);
            if raster[idx] < 0.5 {
                raster[idx] = 1.0;
                break;
            }
        }
    }

    raster
}

fn carrier() -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 4;
    cfg.dormant_cells = 160;
    cfg.hdc_dim = 192;

    let mut evo = EvoPhase::new(cfg);

    let mut raster_cfg = RasterFieldConfig::for_raster(12, 12, 4);
    raster_cfg.match_threshold = 0.97;
    raster_cfg.learning_enabled = false;
    raster_cfg.readout_enabled = false;
    evo.attach_raster_field(raster_cfg);
    evo.set_robust_high_level_perception(true);

    evo.enable_epistemic_state(0.97);
    evo.enable_exploration_strategy(ExplorationConfig::default());

    let mut macro_cfg = MacroConfig::new(4);
    macro_cfg.match_threshold = 0.97;
    macro_cfg.min_promotion_support = 4;
    macro_cfg.formation_enabled = true;
    macro_cfg.revision_enabled = true;
    macro_cfg.readout_enabled = true;
    macro_cfg.learning_enabled = true;
    evo.enable_macro_memory(macro_cfg);

    let mut hierarchy_cfg = HierarchyConfig::default();
    hierarchy_cfg.match_threshold = 0.97;
    hierarchy_cfg.min_promotion_support = 3;
    hierarchy_cfg.formation_enabled = true;
    hierarchy_cfg.readout_enabled = false;
    hierarchy_cfg.learning_enabled = true;
    evo.enable_hierarchy_memory(hierarchy_cfg);

    evo
}

fn post_code(
    family: usize,
    action: usize,
    law: usize,
    informative: usize,
) -> u64 {
    if action == informative {
        CUE_BASE + law as u64
    } else {
        90_000 + family as u64 * 16 + action as u64
    }
}

pub fn install_world_models(
    evo: &mut EvoPhase,
    family: usize,
    informative: usize,
    rivals: usize,
) {
    let pre_code = 10_000 + family as u64;
    for law in 0..rivals {
        let pre = relation(pre_code, law, 0);
        evo.begin_epistemic_tuition(&pre);
        for action in 0..4 {
            let post = relation(post_code(family, action, law, informative), 0, 4);
            evo.record_epistemic_tuition_transition(action, &post);
        }
        evo.commit_epistemic_tuition();
    }
}

fn train_strategy(evo: &mut EvoPhase, perm: [usize; 4]) -> u32 {
    assert_eq!(evo.exploration_weights(), Some([0.0; 3]));
    let mut probes = 0u32;

    for family in 0..8usize {
        let informative = perm[family % 4];
        install_world_models(evo, family, informative, 2);
        let pre_code = 10_000 + family as u64;

        for action in 0..4 {
            let pre = relation(pre_code, 5, 0);
            let before = evo.begin_epistemic_episode(&pre);
            assert_eq!(before, 2, "G7 strategy tuition must activate exactly two rivals");

            let features = evo.epistemic_probe_features(action);
            evo.mark_epistemic_probe_selected(action);

            let post = relation(post_code(family, action, 0, informative), 5, 4);
            let after = evo.observe_epistemic_probe(action, &post);
            evo.train_exploration_from_factual_gain(features, before, after);
            probes = probes.saturating_add(1);
        }
    }

    probes
}

fn child_mid_code(perm: [usize; 4], first_action: usize, branch: usize) -> Option<u64> {
    let logical = perm.iter().position(|action| *action == first_action)?;
    match logical {
        0 | 2 => Some(CHILD_MID_BASE + logical as u64 * 10 + branch as u64),
        _ => None,
    }
}

fn old_terminal(perm: [usize; 4], child: usize, branch: usize) -> usize {
    match (child, branch) {
        (0, 0) => perm[2],
        (0, 1) => perm[3],
        (1, 0) => perm[0],
        (1, 1) => perm[3],
        _ => unreachable!(),
    }
}

fn changed_terminal(perm: [usize; 4], child: usize, branch: usize) -> usize {
    if child == 0 && branch == 0 {
        perm[1]
    } else {
        old_terminal(perm, child, branch)
    }
}

fn train_child(
    evo: &mut EvoPhase,
    perm: [usize; 4],
    child: usize,
    origins: &[(usize, usize)],
) -> usize {
    let first_action = if child == 0 { perm[0] } else { perm[2] };
    for (i, (ox, oy)) in origins.iter().copied().enumerate() {
        let branch = i % 2;
        let pre = relation(CHILD_PRE_CODE, ox, oy);
        let mid = relation(
            child_mid_code(perm, first_action, branch).expect("known child first action"),
            ox,
            oy,
        );
        let second = old_terminal(perm, child, branch);
        evo.observe_successful_macro_episode(&pre, first_action, &mid, second, true);
    }
    origins.len() * 2
}

fn acquire_children(
    evo: &mut EvoPhase,
    perm: [usize; 4],
    reverse: bool,
) -> ([u64; 2], usize) {
    let origins_a = [(0usize, 0usize), (4, 0), (0, 4), (4, 4)];
    let origins_b = [(1usize, 0usize), (5, 0), (1, 4), (5, 4)];

    let child_tuition_actions = if reverse {
        train_child(evo, perm, 1, &origins_b) + train_child(evo, perm, 0, &origins_a)
    } else {
        train_child(evo, perm, 0, &origins_a) + train_child(evo, perm, 1, &origins_b)
    };

    assert_eq!(evo.macros().len(), 2, "G7 requires two acquired child macros");

    let child0 = evo
        .macros()
        .iter()
        .find(|m| m.first_action == perm[0])
        .expect("child 0 macro")
        .id;
    let child1 = evo
        .macros()
        .iter()
        .find(|m| m.first_action == perm[2])
        .expect("child 1 macro")
        .id;

    ([child0, child1], child_tuition_actions)
}

fn required_children(law: usize, children: [u64; 2]) -> [u64; 2] {
    match law {
        0 => [children[0], children[1]],
        1 => [children[1], children[0]],
        2 => [children[0], children[0]],
        _ => unreachable!(),
    }
}

fn required_branches(law: usize) -> [usize; 2] {
    match law {
        0 => [0, 0],
        1 => [1, 1],
        2 => [1, 0],
        _ => unreachable!(),
    }
}

fn child_index(children: [u64; 2], id: u64) -> Option<usize> {
    if id == children[0] {
        Some(0)
    } else if id == children[1] {
        Some(1)
    } else {
        None
    }
}

fn execute_child(
    evo: &mut EvoPhase,
    perm: [usize; 4],
    child_id: u64,
    branch: usize,
    origin: (usize, usize),
) -> Option<[usize; 2]> {
    let pre = relation(CHILD_PRE_CODE, origin.0, origin.1);
    let first = evo.begin_macro_by_id(child_id, &pre)?;
    let mid_code = child_mid_code(perm, first, branch)?;
    let mid = relation(mid_code, origin.0, origin.1);
    let second = evo.continue_macro_invocation(&mid)?;
    Some([first, second])
}

fn evaluate_sequence(
    evo: &mut EvoPhase,
    law: usize,
    sequence: &[u64],
    children: [u64; 2],
    perm: [usize; 4],
    changed: bool,
) -> (bool, usize) {
    if sequence.len() != 2 {
        return (false, 0);
    }

    let expected = required_children(law, children);
    let branches = required_branches(law);
    let origins = [(7usize, 1usize), (7usize, 6usize)];
    let mut primitive_actions = 0usize;

    for stage in 0..2 {
        let Some(actual) = execute_child(
            evo,
            perm,
            sequence[stage],
            branches[stage],
            origins[stage],
        ) else {
            return (false, primitive_actions);
        };
        primitive_actions += 2;

        let expected_child = child_index(children, expected[stage]).expect("known expected child");
        let expected_first = if expected_child == 0 { perm[0] } else { perm[2] };
        let expected_second = if changed {
            changed_terminal(perm, expected_child, branches[stage])
        } else {
            old_terminal(perm, expected_child, branches[stage])
        };

        if actual != [expected_first, expected_second] {
            return (false, primitive_actions);
        }
    }

    (true, primitive_actions)
}

fn all_child_sequences(children: [u64; 2]) -> Vec<Vec<u64>> {
    let mut ids = children.to_vec();
    ids.sort_unstable();
    let mut out = Vec::new();
    for a in &ids {
        for b in &ids {
            out.push(vec![*a, *b]);
        }
    }
    out
}

fn train_parents(evo: &mut EvoPhase, children: [u64; 2], perm: [usize; 4]) -> usize {
    let candidates = all_child_sequences(children);
    let origins = [(0usize, 0usize), (4, 0), (0, 4)];

    let mut primitive_actions = 0usize;

    for law in 0..3usize {
        for (ox, oy) in origins {
            let cue = relation(CUE_BASE + law as u64, ox, oy);
            for sequence in &candidates {
                let mut trial = evo.clone();
                let (need, actions) = evaluate_sequence(
                    &mut trial,
                    law,
                    sequence,
                    children,
                    perm,
                    false,
                );
                primitive_actions += actions;
                evo.observe_successful_parent_sequence(&cue, sequence.clone(), need);
            }
        }
    }

    assert_eq!(
        evo.parent_macros().len(),
        3,
        "three law-specific acquired parents must exist before child revision"
    );
    primitive_actions
}

fn revise_child0(
    evo: &mut EvoPhase,
    perm: [usize; 4],
    child0_id: u64,
) -> (u64, u64, usize) {
    let before = evo
        .macros()
        .iter()
        .find(|m| m.id == child0_id)
        .expect("child 0 before revision")
        .revision;

    let origins = [(2usize, 1usize), (5, 1), (2, 5), (5, 5)];
    for (ox, oy) in origins {
        let pre = relation(CHILD_PRE_CODE, ox, oy);
        let mid = relation(
            child_mid_code(perm, perm[0], 0).expect("child 0 branch"),
            ox,
            oy,
        );

        for second in 0..4usize {
            let need = second == perm[1];
            evo.observe_factual_macro_episode(&pre, perm[0], &mid, second, need);
        }
    }

    let revised = evo
        .macros()
        .iter()
        .find(|m| m.id == child0_id)
        .expect("same child 0 after revision");

    assert!(
        revised.revision > before,
        "same acquired child identity must accumulate factual revision"
    );
    (before, revised.revision, 4 * 4 * 2)
}

pub fn build_mature(seed: u64) -> Mature {
    let mut rng = Rng::new(seed);
    let perm = perm4(&mut rng);
    let reverse = rng.range(2) == 1;

    let mut evo = carrier();
    let tuition_probes = train_strategy(&mut evo, perm);
    assert_eq!(tuition_probes, 32);

    let (children, child_tuition_actions) = acquire_children(&mut evo, perm, reverse);
    let parent_tuition_actions = train_parents(&mut evo, children, perm);

    let unrevised = evo.clone();
    let (pre_revision_counter, _, revision_actions) =
        revise_child0(&mut evo, perm, children[0]);

    assert_eq!(evo.macros().len(), 2);
    assert_eq!(evo.parent_macros().len(), 3);

    Mature {
        evo,
        unrevised,
        perm,
        children,
        tuition_probes,
        child_tuition_actions,
        parent_tuition_actions,
        revision_actions,
        revised_child_id: children[0],
        pre_revision_counter,
    }
}

pub fn prepare_worlds(mature: &mut Mature, worlds: &[World]) {
    for world in worlds {
        install_world_models(
            &mut mature.evo,
            world.family,
            world.informative,
            3,
        );
        install_world_models(
            &mut mature.unrevised,
            world.family,
            world.informative,
            3,
        );
    }
}

pub fn score_world(mature: &Mature, world: World, arm: Arm) -> Score {
    let mut evo = match arm {
        Arm::UnrevisedChild => mature.unrevised.clone(),
        _ => mature.evo.clone(),
    };

    evo.set_exploration_learning_enabled(false);
    evo.set_macro_learning_enabled(false);
    evo.set_hierarchy_learning_enabled(false);
    evo.set_macro_readout_enabled(true);

    match arm {
        Arm::ZeroExploration => {
            evo.enable_exploration_strategy(ExplorationConfig {
                learning_rate: 0.18,
                learning_enabled: false,
                readout_enabled: true,
            });
        }
        _ => evo.set_exploration_readout_enabled(true),
    }

    evo.set_hierarchy_readout_enabled(!matches!(arm, Arm::NoHierarchy));

    let pre_code = 10_000 + world.family as u64;
    let pre = render_relation(
        pre_code,
        world.origin.0,
        world.origin.1,
        world.nuisance,
        world.sensor_seed ^ 0x1111,
    );

    let mut rivals = evo.begin_epistemic_episode(&pre);
    let rivals_at_start = rivals;
    // Open-world memory may legitimately activate extra compatible hypotheses
    // from earlier families. G7 tests whether active experimentation can reduce
    // that carrier-owned ambiguity, so >3 rivals is not itself a failure.
    if rivals < 3 {
        return Score::default();
    }

    let mut stage = 1u8;
    let mut probes = 0usize;
    let mut last_post = None;

    while rivals > 1 && probes < 4 {
        let Some(action) = evo.choose_learned_exploration_probe() else {
            return Score {
                probes,
                rivals_at_start,
                stage,
                ..Score::default()
            };
        };
        probes += 1;

        let code = post_code(world.family, action, world.law, world.informative);
        let post = render_relation(
            code,
            world.origin.0,
            world.origin.1,
            world.nuisance,
            world.sensor_seed ^ ((action as u64 + 1) << 32) ^ code,
        );
        rivals = evo.observe_epistemic_probe(action, &post);
        last_post = Some(post);

        if rivals == 0 {
            return Score {
                probes,
                rivals_at_start,
                stage,
                ..Score::default()
            };
        }
    }

    if rivals != 1 {
        return Score {
            probes,
            rivals_at_start,
            stage,
            ..Score::default()
        };
    }

    stage = 2;
    let cue = last_post.expect("at least one factual probe POST");

    if matches!(arm, Arm::NoHierarchy) {
        let candidates = all_child_sequences(mature.children);
        let mut candidate_evals = 0usize;
        let mut primitive_actions = 0usize;

        for sequence in candidates {
            candidate_evals += 1;
            let mut trial = evo.clone();
            let (success, actions) = evaluate_sequence(
                &mut trial,
                world.law,
                &sequence,
                mature.children,
                mature.perm,
                true,
            );
            primitive_actions += actions;
            if success {
                return Score {
                    success: true,
                    probes,
                    candidate_evals,
                    primitive_actions,
                    rivals_at_start,
                    stage: 4,
                };
            }
        }

        return Score {
            probes,
            candidate_evals,
            primitive_actions,
            rivals_at_start,
            stage: 3,
            ..Score::default()
        };
    }

    let Some(sequence) = evo.select_parent_sequence(&cue) else {
        return Score {
            probes,
            rivals_at_start,
            stage,
            ..Score::default()
        };
    };

    stage = 3;
    let (success, primitive_actions) = evaluate_sequence(
        &mut evo,
        world.law,
        &sequence,
        mature.children,
        mature.perm,
        true,
    );

    Score {
        success,
        probes,
        candidate_evals: 1,
        primitive_actions,
        rivals_at_start,
        stage: 4,
    }
}

pub fn generate_worlds(authority_seed: u64, sub: u64, perm: [usize; 4]) -> Vec<World> {
    let derived = authority_seed
        ^ sub.wrapping_mul(0xA24B_AED4_963E_E407)
        ^ 0xC6A4_A793_5BD1_E995;
    let mut rng = Rng::new(derived);

    let mut nuisance = [
        0u8,
        0u8,
        DISTRACTOR,
        DROPOUT,
        ROTATE,
        SCALE,
        DISTRACTOR | ROTATE,
        DROPOUT | SCALE,
    ];
    rng.shuffle(&mut nuisance);

    let score_origins = [(5usize, 5usize), (5, 4), (4, 5)];

    (0..8usize)
        .map(|i| {
            let family = 1_000 + sub as usize * 16 + i;
            let law = rng.range(3);
            let informative = perm[rng.range(4)];
            let origin = score_origins[rng.range(score_origins.len())];
            World {
                family,
                law,
                informative,
                origin,
                nuisance: nuisance[i],
                sensor_seed: rng.next_u64(),
            }
        })
        .collect()
}

pub fn wilson95(success: usize, n: usize) -> (f64, f64) {
    let z = 1.959_963_984_540_054_f64;
    let n = n as f64;
    let p = success as f64 / n;
    let denom = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / denom;
    let half =
        z * ((p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt()) / denom;
    (center - half, center + half)
}

pub fn fnv_mix(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}
