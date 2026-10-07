use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase};
use aeterna_v1::carrier::{PhaseNativeConfig, PhaseRecursiveConceptInfo};

const OFFSETS: [(usize, usize); 8] = [
    (1, 0), (0, 1), (1, 1), (2, 0),
    (0, 2), (2, 1), (1, 2), (2, 2),
];

const LEVEL1_TARGETS: [[usize; 2]; 4] = [
    [0, 1],
    [2, 7],
    [3, 6],
    [4, 5],
];

const NEGATIVE_MATCHINGS: [[[usize; 2]; 4]; 4] = [
    [[0, 7], [1, 6], [2, 5], [3, 4]],
    [[0, 6], [7, 5], [1, 4], [2, 3]],
    [[0, 5], [6, 4], [7, 3], [1, 2]],
    [[0, 4], [5, 3], [6, 2], [7, 1]],
];

const TOP_PAIRS: [(usize, usize, bool); 4] = [
    (0, 1, false),
    (2, 3, false),
    (0, 2, true),
    (1, 3, true),
];

const FOUNDATION_BINDINGS: [((usize, usize), (usize, usize)); 4] = [
    ((1, 1), (8, 8)),
    ((2, 1), (7, 7)),
    ((1, 3), (8, 5)),
    ((3, 1), (6, 7)),
];

const TOP_TUITION_LAYOUTS: [[(usize, usize); 4]; 4] = [
    [(1, 1), (1, 7), (7, 1), (7, 7)],
    [(2, 1), (2, 7), (7, 2), (7, 7)],
    [(1, 2), (1, 7), (8, 1), (8, 7)],
    [(2, 2), (2, 7), (7, 1), (7, 7)],
];

const TOP_HELDOUT_LAYOUTS: [[(usize, usize); 4]; 4] = [
    [(1, 4), (1, 8), (7, 2), (7, 8)],
    [(3, 1), (3, 7), (8, 2), (8, 7)],
    [(1, 5), (4, 1), (8, 4), (5, 8)],
    [(2, 4), (5, 1), (8, 5), (4, 8)],
];

fn pair_scene(
    first: usize,
    second: usize,
    left: (usize, usize),
    right: (usize, usize),
) -> Vec<f32> {
    let mut raster = vec![0.0f32; 144];
    for (atom, (x, y)) in [(first, left), (second, right)] {
        let (dx, dy) = OFFSETS[atom];
        assert!(x + dx < 12 && y + dy < 12);
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn top_scene(
    left_concept: usize,
    right_concept: usize,
    layout: [(usize, usize); 4],
) -> Vec<f32> {
    let mut raster = vec![0.0f32; 144];
    let atoms = [
        LEVEL1_TARGETS[left_concept][0],
        LEVEL1_TARGETS[left_concept][1],
        LEVEL1_TARGETS[right_concept][0],
        LEVEL1_TARGETS[right_concept][1],
    ];

    for (atom, (x, y)) in atoms.into_iter().zip(layout) {
        let (dx, dy) = OFFSETS[atom];
        assert!(x + dx < 12 && y + dy < 12);
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn top_expected(class_one: bool, swap: bool) -> usize {
    if class_one ^ swap { 3 } else { 2 }
}

fn carrier() -> EvoPhase {
    let cfg = EvoConfig {
        sensory_cells: 144,
        motor_cells: 4,
        dormant_cells: 192,
        hdc_dim: 192,
        weight_learning_rate: 1.0,
        phase_learning_rate: 1.0,
        min_recruit_support: 1,
        ..EvoConfig::default()
    };
    let mut evo = EvoPhase::new(cfg);
    let mut concept = ConceptConfig::for_raster(12, 12, 4, 192);
    concept.local_radius = 2;
    concept.atom_match_threshold = 0.97;
    concept.min_action_support = 4;
    concept.min_composite_support = 8;
    concept.child_predictiveness_ceiling = 0.20;
    concept.composite_promotion_threshold = 0.60;
    concept.readout_enabled = false;
    concept.learning_enabled = true;
    evo.enable_concept_memory(concept);
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_concepts());
    evo
}

fn train_level1(evo: &mut EvoPhase) {
    // Four target pairs: motor 0 succeeds, motor 1 fails.
    for pair in LEVEL1_TARGETS {
        for (left, right) in FOUNDATION_BINDINGS {
            let raster = pair_scene(pair[0], pair[1], left, right);
            for action in [0usize, 1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,
                    action,
                    action == 0,
                ));
            }
        }
    }

    // Four disjoint one-shot matchings balance every primitive child's
    // foundation evidence without letting any distractor pair reach support 8.
    for (matching_index, matching) in NEGATIVE_MATCHINGS.into_iter().enumerate() {
        let binding = FOUNDATION_BINDINGS[matching_index];
        for pair in matching {
            let raster = pair_scene(pair[0], pair[1], binding.0, binding.1);
            for action in [0usize, 1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,
                    action,
                    action == 1,
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(), 8);
    assert_eq!(
        evo.composite_concepts().len(),
        0,
        "G11 foundation must stay on the physical concept path"
    );
    assert_eq!(evo.phase_native_concept_atom_count(), 8);
    assert_eq!(evo.phase_native_promoted_concept_count(), 4);
}

fn train_level2(evo: &mut EvoPhase, swap: bool, require_structural_accept: bool) {
    assert!(evo.enable_phase_native_recursive_concepts());
    evo.set_phase_native_recursive_readout_enabled(false);

    for layout in TOP_TUITION_LAYOUTS {
        for (left, right, class_one) in TOP_PAIRS {
            let raster = top_scene(left, right, layout);
            for action in [2usize, 3usize] {
                let accepted = evo.observe_phase_native_recursive_concept_factual(
                    &raster,
                    action,
                    action == top_expected(class_one, swap),
                );
                if require_structural_accept {
                    assert!(
                        accepted,
                        "FULL/learning control must accept recursive factual observation"
                    );
                }
            }
        }
    }

    evo.set_phase_native_recursive_readout_enabled(true);
}

fn freeze(evo: &mut EvoPhase) {
    evo.set_concept_learning_enabled(false);
    evo.set_planning_learning_enabled(false);
}

fn score_recursive(evo: &EvoPhase, swap: bool) -> usize {
    TOP_PAIRS
        .into_iter()
        .enumerate()
        .map(|(index, (left, right, class_one))| {
            let raster = top_scene(left, right, TOP_HELDOUT_LAYOUTS[index]);
            usize::from(
                evo.choose_phase_native_recursive_concept_action(&raster)
                    == Some(top_expected(class_one, swap)),
            )
        })
        .sum()
}

fn active_level1_ids(evo: &EvoPhase, sensory: &[f32]) -> Vec<u64> {
    let active_atoms = evo.active_concept_atom_ids(sensory);
    let mut ids = evo
        .phase_native_concept_circuits()
        .iter()
        .filter(|circuit| {
            circuit.promoted
                && circuit
                    .child_ids
                    .iter()
                    .all(|id| active_atoms.binary_search(id).is_ok())
        })
        .map(|circuit| circuit.concept_id)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn recursive_for_scene(
    evo: &EvoPhase,
    sensory: &[f32],
) -> PhaseRecursiveConceptInfo {
    let ids = active_level1_ids(evo, sensory);
    assert_eq!(ids.len(), 2, "held-out top scene must activate exactly two L1 concepts");
    let pair = [ids[0], ids[1]];
    evo.phase_native_recursive_circuits()
        .iter()
        .find(|circuit| circuit.promoted && circuit.child_concept_ids == pair)
        .expect("promoted recursive concept for held-out pair")
        .clone()
}

fn level1_only_action(evo: &EvoPhase, sensory: &[f32]) -> Option<usize> {
    let ids = active_level1_ids(evo, sensory);
    if ids.is_empty() {
        return None;
    }
    let mut scores = [0.0f32; 2];
    for concept_id in ids {
        let circuit = evo
            .phase_native_concept_circuits()
            .iter()
            .find(|circuit| circuit.promoted && circuit.concept_id == concept_id)
            .expect("active L1 concept");
        for (slot, action) in [2usize, 3usize].into_iter().enumerate() {
            let synapse = evo
                .phase_native_synapse(circuit.motor_synapses[action])
                .expect("L1 physical motor evidence synapse");
            scores[slot] += 2.0 * synapse.weight - 1.0;
        }
    }
    Some(if scores[1] > scores[0] + 1.0e-6 { 3 } else { 2 })
}

fn score_level1_only(evo: &EvoPhase, swap: bool) -> usize {
    TOP_PAIRS
        .into_iter()
        .enumerate()
        .map(|(index, (left, right, class_one))| {
            let raster = top_scene(left, right, TOP_HELDOUT_LAYOUTS[index]);
            usize::from(
                level1_only_action(evo, &raster)
                    == Some(top_expected(class_one, swap)),
            )
        })
        .sum()
}

fn max_level1_top_evidence(evo: &EvoPhase) -> f32 {
    evo.phase_native_concept_circuits()
        .iter()
        .filter(|circuit| circuit.promoted)
        .flat_map(|circuit| {
            [2usize, 3usize].into_iter().map(|action| {
                let synapse = evo
                    .phase_native_synapse(circuit.motor_synapses[action])
                    .expect("L1 top evidence");
                (2.0 * synapse.weight - 1.0).abs()
            })
        })
        .fold(0.0f32, f32::max)
}

fn min_level2_winning_evidence(evo: &EvoPhase) -> f32 {
    evo.phase_native_recursive_circuits()
        .iter()
        .filter(|circuit| circuit.promoted)
        .map(|circuit| {
            [2usize, 3usize]
                .into_iter()
                .map(|action| {
                    let synapse = evo
                        .phase_native_synapse(circuit.motor_synapses[action])
                        .expect("L2 motor evidence");
                    2.0 * synapse.weight - 1.0
                })
                .fold(f32::NEG_INFINITY, f32::max)
        })
        .fold(1.0f32, f32::min)
}

fn verify_recursive_children_are_level1(evo: &EvoPhase) {
    let promoted = evo
        .phase_native_concept_circuits()
        .iter()
        .filter(|circuit| circuit.promoted)
        .map(|circuit| (circuit.concept_id, circuit.concept_cell))
        .collect::<std::collections::BTreeMap<_, _>>();

    for recursive in evo
        .phase_native_recursive_circuits()
        .iter()
        .filter(|circuit| circuit.promoted)
    {
        for index in 0..2 {
            assert_eq!(
                promoted.get(&recursive.child_concept_ids[index]),
                Some(&recursive.child_cells[index]),
                "G11 L2 child must reference an acquired L1 concept cell"
            );
        }
    }
}

#[test]
fn g11_recursive_physical_concepts_reuse_acquired_concepts_as_children() {
    let mut full_total = 0usize;
    let mut no_recursion_total = 0usize;
    let mut level1_only_total = 0usize;
    let mut lesion_total = 0usize;
    let mut phase_total = 0usize;
    let mut restored_total = 0usize;
    let mut unrelated_ok = 0usize;
    let mut lower_lesion_ok = 0usize;
    let mut zero_phase_total = 0usize;
    let mut zero_weight_total = 0usize;
    let mut no_growth_total = 0usize;
    let mut max_child_evidence = 0.0f32;
    let mut min_joint_evidence = 1.0f32;

    for swap in [false, true] {
        let mut stage1 = carrier();
        train_level1(&mut stage1);

        let mut full = stage1.clone();
        train_level2(&mut full, swap, true);
        assert_eq!(full.phase_native_promoted_recursive_count(), 4);
        verify_recursive_children_are_level1(&full);
        freeze(&mut full);

        let full_score = score_recursive(&full, swap);
        assert_eq!(full_score, 4);
        full_total += full_score;
        max_child_evidence = max_child_evidence.max(max_level1_top_evidence(&full));
        min_joint_evidence = min_joint_evidence.min(min_level2_winning_evidence(&full));

        let mut no_recursion = stage1.clone();
        assert!(no_recursion.enable_phase_native_recursive_concepts());
        no_recursion.set_phase_native_recursive_formation_enabled(false);
        no_recursion.set_phase_native_recursive_readout_enabled(false);
        for layout in TOP_TUITION_LAYOUTS {
            for (left, right, class_one) in TOP_PAIRS {
                let raster = top_scene(left, right, layout);
                for action in [2usize, 3usize] {
                    assert!(no_recursion.observe_phase_native_recursive_concept_factual(
                        &raster,
                        action,
                        action == top_expected(class_one, swap),
                    ));
                }
            }
        }
        freeze(&mut no_recursion);
        no_recursion_total += score_recursive(&no_recursion, swap);
        level1_only_total += score_level1_only(&no_recursion, swap);

        let target_raster = top_scene(
            TOP_PAIRS[0].0,
            TOP_PAIRS[0].1,
            TOP_HELDOUT_LAYOUTS[0],
        );
        let recursive = recursive_for_scene(&full, &target_raster);

        let mut lesioned = full.clone();
        let saved = lesioned
            .perturb_phase_native_synapse_for_control(
                recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("necessary L1->L2 synapse");
        lesion_total += score_recursive(&lesioned, swap);
        lesioned.restore_phase_native_synapse_for_control(
            recursive.child_synapses[0],
            saved,
        );
        restored_total += score_recursive(&lesioned, swap);

        let mut phase_shifted = full.clone();
        phase_shifted
            .perturb_phase_native_synapse_for_control(
                recursive.child_synapses[0],
                1.0,
                std::f32::consts::PI,
            )
            .expect("necessary L1->L2 phase synapse");
        phase_total += score_recursive(&phase_shifted, swap);

        let unrelated_raster = top_scene(
            TOP_PAIRS[1].0,
            TOP_PAIRS[1].1,
            TOP_HELDOUT_LAYOUTS[1],
        );
        let unrelated_recursive = recursive_for_scene(&full, &unrelated_raster);
        let mut unrelated = full.clone();
        unrelated
            .perturb_phase_native_synapse_for_control(
                unrelated_recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("unrelated L2 synapse");
        unrelated_ok += usize::from(
            unrelated.choose_phase_native_recursive_concept_action(&target_raster)
                == Some(top_expected(TOP_PAIRS[0].2, swap)),
        );

        let first_child_id = recursive.child_concept_ids[0];
        let lower = full
            .phase_native_concept_circuits()
            .iter()
            .find(|circuit| circuit.promoted && circuit.concept_id == first_child_id)
            .expect("L1 child circuit")
            .clone();
        let mut lower_lesioned = full.clone();
        lower_lesioned
            .perturb_phase_native_synapse_for_control(
                lower.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("atom->L1 dependency");
        lower_lesion_ok += usize::from(
            lower_lesioned
                .choose_phase_native_recursive_concept_action(&target_raster)
                == Some(top_expected(TOP_PAIRS[0].2, swap)),
        );

        let mut zero_phase = stage1.clone();
        zero_phase.set_learning_rates_for_control(1.0, 0.0);
        train_level2(&mut zero_phase, swap, true);
        freeze(&mut zero_phase);
        zero_phase_total += score_recursive(&zero_phase, swap);

        let mut zero_weight = stage1.clone();
        zero_weight.set_learning_rates_for_control(0.0, 1.0);
        train_level2(&mut zero_weight, swap, true);
        freeze(&mut zero_weight);
        zero_weight_total += score_recursive(&zero_weight, swap);

        let mut no_growth = stage1.clone();
        no_growth.set_structural_growth_for_control(false);
        train_level2(&mut no_growth, swap, false);
        freeze(&mut no_growth);
        no_growth_total += score_recursive(&no_growth, swap);
    }

    println!(
        "G11_RECURSIVE full={}/8 no_recursion={}/8 level1_only={}/8 lesion={}/8 phase_shift={}/8 restored={}/8 unrelated={}/2 lower_lesion_success={}/2 zero_phase={}/8 zero_weight={}/8 no_growth={}/8 max_l1_evidence={:.6} min_l2_evidence={:.6}",
        full_total,
        no_recursion_total,
        level1_only_total,
        lesion_total,
        phase_total,
        restored_total,
        unrelated_ok,
        lower_lesion_ok,
        zero_phase_total,
        zero_weight_total,
        no_growth_total,
        max_child_evidence,
        min_joint_evidence,
    );

    assert_eq!(full_total, 8);
    assert!(no_recursion_total <= 4);
    assert!(level1_only_total <= 4);
    assert!(max_child_evidence <= 0.20 + 1.0e-6);
    assert!(min_joint_evidence >= 0.60 - 1.0e-6);
    assert!(lesion_total <= 6);
    assert!(phase_total <= 6);
    assert_eq!(restored_total, 8);
    assert_eq!(unrelated_ok, 2);
    assert_eq!(
        lower_lesion_ok, 0,
        "breaking an atom->L1 path must remove the dependent L2 decision"
    );
    assert!(zero_phase_total <= 4);
    assert!(zero_weight_total <= 4);
    assert!(no_growth_total <= 4);
}

#[test]
fn g11_recursive_selector_has_no_flat_table_or_search_fallback() {
    let source = include_str!("../src/phase_recursive.rs");
    let start = source
        .find("pub fn choose_phase_native_recursive_concept_action")
        .expect("G11 recursive selector");
    let selector = &source[start..];

    for forbidden in [
        "choose_composite_action",
        "choose_atom_only_action",
        "choose_phase_native_concept_action",
        "EvoImaginationPlanner",
        "VecDeque",
        "BinaryHeap",
        "TOP_PAIRS",
        "LEVEL1_TARGETS",
        "top_expected",
    ] {
        assert!(
            !selector.contains(forbidden),
            "G11 selector contains forbidden fallback/token {forbidden}"
        );
    }

    for required in [
        "active_atom_ids",
        "physical.circuits",
        "physical.recursive",
        "child_cells",
        "child_synapses",
        "conductance",
        "motor_synapses",
    ] {
        assert!(
            selector.contains(required),
            "G11 selector missing recursive physical dependency {required}"
        );
    }
}


const FACTOR8: [[[usize; 2]; 4]; 7] = [
    [[0, 7], [1, 6], [2, 5], [3, 4]],
    [[0, 6], [7, 5], [1, 4], [2, 3]],
    [[0, 5], [6, 4], [7, 3], [1, 2]],
    [[0, 4], [5, 3], [6, 2], [7, 1]],
    [[0, 3], [4, 2], [5, 1], [6, 7]],
    [[0, 2], [3, 1], [4, 7], [5, 6]],
    [[0, 1], [2, 7], [3, 6], [4, 5]],
];

const FACTOR4: [[[usize; 2]; 2]; 3] = [
    [[0, 1], [2, 3]],
    [[0, 2], [1, 3]],
    [[0, 3], [1, 2]],
];

const FRESH_PAIR_BINDING_BANK: [((usize, usize), (usize, usize)); 8] = [
    ((1, 1), (8, 8)),
    ((2, 1), (7, 7)),
    ((1, 2), (8, 7)),
    ((2, 2), (7, 8)),
    ((1, 3), (8, 6)),
    ((3, 1), (6, 8)),
    ((2, 3), (8, 5)),
    ((3, 2), (5, 8)),
];

const FRESH_TOP_LAYOUT_BANK: [[(usize, usize); 4]; 12] = [
    [(1, 1), (1, 7), (7, 1), (7, 7)],
    [(2, 1), (2, 7), (7, 1), (7, 7)],
    [(1, 2), (1, 7), (7, 2), (7, 7)],
    [(2, 2), (2, 7), (7, 2), (7, 7)],
    [(1, 1), (1, 8), (8, 1), (8, 8)],
    [(2, 1), (2, 8), (8, 1), (8, 8)],
    [(1, 2), (1, 8), (8, 2), (8, 8)],
    [(2, 2), (2, 8), (8, 2), (8, 8)],
    [(1, 1), (2, 7), (7, 2), (8, 8)],
    [(2, 1), (1, 8), (8, 2), (7, 7)],
    [(1, 2), (2, 8), (8, 1), (7, 7)],
    [(2, 2), (1, 7), (7, 1), (8, 8)],
];

#[derive(Clone, Debug)]
struct FreshG11Block {
    offsets: [(usize, usize); 8],
    level1_targets: [[usize; 2]; 4],
    negative_matchings: [[[usize; 2]; 4]; 4],
    foundation_actions: [usize; 2],
    top_actions: [usize; 2],
    top_pairs: [(usize, usize, bool); 4],
    target_bindings: [((usize, usize), (usize, usize)); 4],
    negative_bindings: [((usize, usize), (usize, usize)); 4],
    top_tuition: [[(usize, usize); 4]; 4],
    top_heldout: [[(usize, usize); 4]; 8],
}

struct FreshG11Rng(u64);

impl FreshG11Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x611A_B57A_C710_0011)
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

fn g11_shuffle<T>(rng: &mut FreshG11Rng, values: &mut [T]) {
    for i in (1..values.len()).rev() {
        let j = rng.range(i + 1);
        values.swap(i, j);
    }
}

fn g11_mix(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn fresh_g11_blocks(authority: u64) -> (Vec<FreshG11Block>, u64) {
    let mut blocks = Vec::new();
    let mut digest = 14_695_981_039_346_656_037_u64;

    for sub in 0..10u64 {
        let derived = authority
            ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ 0x611A_5EED_D2C0_000B;
        let mut rng = FreshG11Rng::new(derived);

        let mut offsets = OFFSETS.to_vec();
        g11_shuffle(&mut rng, &mut offsets);
        let offsets: [(usize, usize); 8] = offsets.try_into().unwrap();

        let target_round = rng.range(7);
        let mut level1_targets = FACTOR8[target_round];
        g11_shuffle(&mut rng, &mut level1_targets);

        let mut other_rounds = (0..7usize)
            .filter(|round| *round != target_round)
            .collect::<Vec<_>>();
        g11_shuffle(&mut rng, &mut other_rounds);
        let negative_matchings = [
            FACTOR8[other_rounds[0]],
            FACTOR8[other_rounds[1]],
            FACTOR8[other_rounds[2]],
            FACTOR8[other_rounds[3]],
        ];

        let mut motors = [0usize, 1, 2, 3];
        g11_shuffle(&mut rng, &mut motors);
        let foundation_actions = [motors[0], motors[1]];
        let top_actions = [motors[2], motors[3]];

        let class0_round = rng.range(3);
        let mut class1_choices = (0..3usize)
            .filter(|round| *round != class0_round)
            .collect::<Vec<_>>();
        g11_shuffle(&mut rng, &mut class1_choices);
        let class1_round = class1_choices[0];
        let top_pairs = [
            (
                FACTOR4[class0_round][0][0],
                FACTOR4[class0_round][0][1],
                false,
            ),
            (
                FACTOR4[class0_round][1][0],
                FACTOR4[class0_round][1][1],
                false,
            ),
            (
                FACTOR4[class1_round][0][0],
                FACTOR4[class1_round][0][1],
                true,
            ),
            (
                FACTOR4[class1_round][1][0],
                FACTOR4[class1_round][1][1],
                true,
            ),
        ];

        let mut pair_bindings = FRESH_PAIR_BINDING_BANK.to_vec();
        g11_shuffle(&mut rng, &mut pair_bindings);
        let target_bindings: [((usize, usize), (usize, usize)); 4] =
            pair_bindings[..4].try_into().unwrap();
        let negative_bindings: [((usize, usize), (usize, usize)); 4] =
            pair_bindings[4..8].try_into().unwrap();

        let mut top_layouts = FRESH_TOP_LAYOUT_BANK.to_vec();
        g11_shuffle(&mut rng, &mut top_layouts);
        let top_tuition: [[(usize, usize); 4]; 4] =
            top_layouts[..4].try_into().unwrap();
        let top_heldout: [[(usize, usize); 4]; 8] =
            top_layouts[4..12].try_into().unwrap();

        digest = g11_mix(digest, sub);
        for (dx, dy) in offsets {
            digest = g11_mix(digest, dx as u64);
            digest = g11_mix(digest, dy as u64);
        }
        for pair in level1_targets {
            digest = g11_mix(digest, pair[0] as u64);
            digest = g11_mix(digest, pair[1] as u64);
        }
        for matching in negative_matchings {
            for pair in matching {
                digest = g11_mix(digest, pair[0] as u64);
                digest = g11_mix(digest, pair[1] as u64);
            }
        }
        for motor in foundation_actions.into_iter().chain(top_actions) {
            digest = g11_mix(digest, motor as u64);
        }
        for (a, b, class_one) in top_pairs {
            digest = g11_mix(digest, a as u64);
            digest = g11_mix(digest, b as u64);
            digest = g11_mix(digest, class_one as u64);
        }
        for binding in target_bindings.into_iter().chain(negative_bindings) {
            digest = g11_mix(digest, binding.0.0 as u64);
            digest = g11_mix(digest, binding.0.1 as u64);
            digest = g11_mix(digest, binding.1.0 as u64);
            digest = g11_mix(digest, binding.1.1 as u64);
        }
        for layout in top_tuition.into_iter().chain(top_heldout) {
            for (x, y) in layout {
                digest = g11_mix(digest, x as u64);
                digest = g11_mix(digest, y as u64);
            }
        }

        blocks.push(FreshG11Block {
            offsets,
            level1_targets,
            negative_matchings,
            foundation_actions,
            top_actions,
            top_pairs,
            target_bindings,
            negative_bindings,
            top_tuition,
            top_heldout,
        });
    }

    (blocks, digest)
}

fn fresh_pair_scene_g11(
    block: &FreshG11Block,
    first: usize,
    second: usize,
    binding: ((usize, usize), (usize, usize)),
) -> Vec<f32> {
    let mut raster = vec![0.0f32; 144];
    for (atom, (x, y)) in [(first, binding.0), (second, binding.1)] {
        let (dx, dy) = block.offsets[atom];
        assert!(x + dx < 12 && y + dy < 12);
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn fresh_top_scene_g11(
    block: &FreshG11Block,
    left_concept: usize,
    right_concept: usize,
    layout: [(usize, usize); 4],
) -> Vec<f32> {
    let mut raster = vec![0.0f32; 144];
    let atoms = [
        block.level1_targets[left_concept][0],
        block.level1_targets[left_concept][1],
        block.level1_targets[right_concept][0],
        block.level1_targets[right_concept][1],
    ];
    for (atom, (x, y)) in atoms.into_iter().zip(layout) {
        let (dx, dy) = block.offsets[atom];
        assert!(x + dx < 12 && y + dy < 12);
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn fresh_top_expected(block: &FreshG11Block, class_one: bool) -> usize {
    if class_one {
        block.top_actions[1]
    } else {
        block.top_actions[0]
    }
}

fn train_fresh_level1(evo: &mut EvoPhase, block: &FreshG11Block) {
    for pair in block.level1_targets {
        for binding in block.target_bindings {
            let raster = fresh_pair_scene_g11(block, pair[0], pair[1], binding);
            for action in block.foundation_actions {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,
                    action,
                    action == block.foundation_actions[0],
                ));
            }
        }
    }

    for (matching, binding) in block
        .negative_matchings
        .into_iter()
        .zip(block.negative_bindings)
    {
        for pair in matching {
            let raster = fresh_pair_scene_g11(block, pair[0], pair[1], binding);
            for action in block.foundation_actions {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,
                    action,
                    action == block.foundation_actions[1],
                ));
            }
        }
    }
}

fn train_fresh_level2(
    evo: &mut EvoPhase,
    block: &FreshG11Block,
    require_structural_accept: bool,
) {
    assert!(evo.enable_phase_native_recursive_concepts());
    evo.set_phase_native_recursive_readout_enabled(false);

    for layout in block.top_tuition {
        for (left, right, class_one) in block.top_pairs {
            let raster = fresh_top_scene_g11(block, left, right, layout);
            for action in block.top_actions {
                let accepted = evo.observe_phase_native_recursive_concept_factual(
                    &raster,
                    action,
                    action == fresh_top_expected(block, class_one),
                );
                if require_structural_accept {
                    assert!(accepted);
                }
            }
        }
    }

    evo.set_phase_native_recursive_readout_enabled(true);
}

fn score_fresh_recursive(evo: &EvoPhase, block: &FreshG11Block) -> usize {
    let mut score = 0usize;
    for (pair_index, (left, right, class_one)) in block.top_pairs.into_iter().enumerate() {
        for layout_index in 0..2 {
            let raster = fresh_top_scene_g11(
                block,
                left,
                right,
                block.top_heldout[pair_index * 2 + layout_index],
            );
            score += usize::from(
                evo.choose_phase_native_recursive_concept_action(&raster)
                    == Some(fresh_top_expected(block, class_one)),
            );
        }
    }
    score
}

fn fresh_level1_only_action(
    evo: &EvoPhase,
    block: &FreshG11Block,
    sensory: &[f32],
) -> Option<usize> {
    let ids = active_level1_ids(evo, sensory);
    if ids.is_empty() {
        return None;
    }

    let mut scores = [0.0f32; 2];
    for concept_id in ids {
        let circuit = evo
            .phase_native_concept_circuits()
            .iter()
            .find(|circuit| circuit.promoted && circuit.concept_id == concept_id)?;
        for slot in 0..2 {
            let action = block.top_actions[slot];
            let synapse = evo.phase_native_synapse(circuit.motor_synapses[action])?;
            scores[slot] += 2.0 * synapse.weight - 1.0;
        }
    }
    Some(if scores[1] > scores[0] + 1.0e-6 {
        block.top_actions[1]
    } else {
        block.top_actions[0]
    })
}

fn score_fresh_level1_only(evo: &EvoPhase, block: &FreshG11Block) -> usize {
    let mut score = 0usize;
    for (pair_index, (left, right, class_one)) in block.top_pairs.into_iter().enumerate() {
        for layout_index in 0..2 {
            let raster = fresh_top_scene_g11(
                block,
                left,
                right,
                block.top_heldout[pair_index * 2 + layout_index],
            );
            score += usize::from(
                fresh_level1_only_action(evo, block, &raster)
                    == Some(fresh_top_expected(block, class_one)),
            );
        }
    }
    score
}

fn fresh_recursive_for_scene(
    evo: &EvoPhase,
    sensory: &[f32],
) -> PhaseRecursiveConceptInfo {
    let ids = active_level1_ids(evo, sensory);
    assert_eq!(ids.len(), 2);
    let pair = [ids[0], ids[1]];
    evo.phase_native_recursive_circuits()
        .iter()
        .find(|circuit| circuit.promoted && circuit.child_concept_ids == pair)
        .expect("fresh promoted recursive circuit")
        .clone()
}

fn fresh_max_l1_evidence(evo: &EvoPhase, block: &FreshG11Block) -> f32 {
    evo.phase_native_concept_circuits()
        .iter()
        .filter(|circuit| circuit.promoted)
        .flat_map(|circuit| {
            block.top_actions.into_iter().map(|action| {
                let synapse = evo
                    .phase_native_synapse(circuit.motor_synapses[action])
                    .expect("fresh L1 evidence");
                (2.0 * synapse.weight - 1.0).abs()
            })
        })
        .fold(0.0f32, f32::max)
}

fn fresh_min_l2_evidence(evo: &EvoPhase, block: &FreshG11Block) -> f32 {
    evo.phase_native_recursive_circuits()
        .iter()
        .filter(|circuit| circuit.promoted)
        .map(|circuit| {
            block
                .top_actions
                .into_iter()
                .map(|action| {
                    let synapse = evo
                        .phase_native_synapse(circuit.motor_synapses[action])
                        .expect("fresh L2 evidence");
                    2.0 * synapse.weight - 1.0
                })
                .fold(f32::NEG_INFINITY, f32::max)
        })
        .fold(1.0f32, f32::min)
}

fn g11_wilson95(success: usize, n: usize) -> (f64, f64) {
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

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from first-attempt CI"]
fn g11_fresh_recursive_abstraction_pack() {
    let authority: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse()
        .expect("fresh G11 seed must be u64");
    let source_sha = std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha = std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let (blocks, digest) = fresh_g11_blocks(authority);
    println!(
        "FRESH_G11_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha, spec_sha, authority, digest
    );
    for (sub, block) in blocks.iter().enumerate() {
        println!("FRESH_G11_BLOCK sub={} {:?}", sub, block);
    }

    let mut full = 0usize;
    let mut no_recursion = 0usize;
    let mut level1_only = 0usize;
    let mut zero_phase = 0usize;
    let mut zero_weight = 0usize;
    let mut no_growth = 0usize;
    let mut per_seed = Vec::new();
    let mut max_l1 = 0.0f32;
    let mut min_l2 = 1.0f32;
    let mut ref_violations = 0usize;
    let mut level1_count_violations = 0usize;
    let mut level2_count_violations = 0usize;
    let mut atom_count_violations = 0usize;
    let mut lesion_ok = 0usize;
    let mut phase_ok = 0usize;
    let mut restored_ok = 0usize;
    let mut unrelated_ok = 0usize;
    let mut lower_lesion_ok = 0usize;
    let mut motor_role_mask = 0u8;

    for (sub, block) in blocks.iter().enumerate() {
        for motor in block
            .foundation_actions
            .into_iter()
            .chain(block.top_actions)
        {
            motor_role_mask |= 1u8 << motor;
        }

        let mut stage1 = carrier();
        train_fresh_level1(&mut stage1, block);

        if stage1.concept_atoms().len() != 8 {
            atom_count_violations += 1;
        }
        if stage1.phase_native_promoted_concept_count() != 4 {
            level1_count_violations += 1;
        }

        let mut full_evo = stage1.clone();
        train_fresh_level2(&mut full_evo, block, true);
        if full_evo.phase_native_promoted_recursive_count() != 4 {
            level2_count_violations += 1;
        }

        let level1_map = full_evo
            .phase_native_concept_circuits()
            .iter()
            .filter(|circuit| circuit.promoted)
            .map(|circuit| (circuit.concept_id, circuit.concept_cell))
            .collect::<std::collections::BTreeMap<_, _>>();
        for recursive in full_evo
            .phase_native_recursive_circuits()
            .iter()
            .filter(|circuit| circuit.promoted)
        {
            for index in 0..2 {
                if level1_map.get(&recursive.child_concept_ids[index])
                    != Some(&recursive.child_cells[index])
                {
                    ref_violations += 1;
                }
            }
        }

        max_l1 = max_l1.max(fresh_max_l1_evidence(&full_evo, block));
        min_l2 = min_l2.min(fresh_min_l2_evidence(&full_evo, block));
        freeze(&mut full_evo);

        let sub_score = score_fresh_recursive(&full_evo, block);
        full += sub_score;
        per_seed.push(sub_score);

        let mut nr = stage1.clone();
        assert!(nr.enable_phase_native_recursive_concepts());
        nr.set_phase_native_recursive_formation_enabled(false);
        nr.set_phase_native_recursive_readout_enabled(false);
        for layout in block.top_tuition {
            for (left, right, class_one) in block.top_pairs {
                let raster = fresh_top_scene_g11(block, left, right, layout);
                for action in block.top_actions {
                    assert!(nr.observe_phase_native_recursive_concept_factual(
                        &raster,
                        action,
                        action == fresh_top_expected(block, class_one),
                    ));
                }
            }
        }
        freeze(&mut nr);
        no_recursion += score_fresh_recursive(&nr, block);
        level1_only += score_fresh_level1_only(&nr, block);

        let mut zp = stage1.clone();
        zp.set_learning_rates_for_control(1.0, 0.0);
        train_fresh_level2(&mut zp, block, true);
        freeze(&mut zp);
        zero_phase += score_fresh_recursive(&zp, block);

        let mut zw = stage1.clone();
        zw.set_learning_rates_for_control(0.0, 1.0);
        train_fresh_level2(&mut zw, block, true);
        freeze(&mut zw);
        zero_weight += score_fresh_recursive(&zw, block);

        let mut ng = stage1.clone();
        ng.set_structural_growth_for_control(false);
        train_fresh_level2(&mut ng, block, false);
        freeze(&mut ng);
        no_growth += score_fresh_recursive(&ng, block);

        let first_pair = block.top_pairs[0];
        let expected = fresh_top_expected(block, first_pair.2);
        let first_scene = fresh_top_scene_g11(
            block,
            first_pair.0,
            first_pair.1,
            block.top_heldout[0],
        );
        let recursive = fresh_recursive_for_scene(&full_evo, &first_scene);

        let mut lesioned = full_evo.clone();
        let saved = lesioned
            .perturb_phase_native_synapse_for_control(
                recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh G11 necessary L1->L2 synapse");
        for layout_index in 0..2 {
            let raster = fresh_top_scene_g11(
                block,
                first_pair.0,
                first_pair.1,
                block.top_heldout[layout_index],
            );
            lesion_ok += usize::from(
                lesioned.choose_phase_native_recursive_concept_action(&raster)
                    == Some(expected),
            );
        }

        lesioned.restore_phase_native_synapse_for_control(
            recursive.child_synapses[0],
            saved,
        );
        for layout_index in 0..2 {
            let raster = fresh_top_scene_g11(
                block,
                first_pair.0,
                first_pair.1,
                block.top_heldout[layout_index],
            );
            restored_ok += usize::from(
                lesioned.choose_phase_native_recursive_concept_action(&raster)
                    == Some(expected),
            );
        }

        let mut phase_shifted = full_evo.clone();
        phase_shifted
            .perturb_phase_native_synapse_for_control(
                recursive.child_synapses[0],
                1.0,
                std::f32::consts::PI,
            )
            .expect("fresh G11 phase intervention");
        for layout_index in 0..2 {
            let raster = fresh_top_scene_g11(
                block,
                first_pair.0,
                first_pair.1,
                block.top_heldout[layout_index],
            );
            phase_ok += usize::from(
                phase_shifted.choose_phase_native_recursive_concept_action(&raster)
                    == Some(expected),
            );
        }

        let other_pair = block.top_pairs[1];
        let other_scene = fresh_top_scene_g11(
            block,
            other_pair.0,
            other_pair.1,
            block.top_heldout[2],
        );
        let other_recursive = fresh_recursive_for_scene(&full_evo, &other_scene);
        let mut unrelated = full_evo.clone();
        unrelated
            .perturb_phase_native_synapse_for_control(
                other_recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh G11 unrelated L2 synapse");
        unrelated_ok += usize::from(
            unrelated.choose_phase_native_recursive_concept_action(&first_scene)
                == Some(expected),
        );

        let child_id = recursive.child_concept_ids[0];
        let lower = full_evo
            .phase_native_concept_circuits()
            .iter()
            .find(|circuit| circuit.promoted && circuit.concept_id == child_id)
            .expect("fresh G11 L1 child")
            .clone();
        let mut lower_lesioned = full_evo.clone();
        lower_lesioned
            .perturb_phase_native_synapse_for_control(
                lower.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh G11 lower dependency");
        for layout_index in 0..2 {
            let raster = fresh_top_scene_g11(
                block,
                first_pair.0,
                first_pair.1,
                block.top_heldout[layout_index],
            );
            lower_lesion_ok += usize::from(
                lower_lesioned
                    .choose_phase_native_recursive_concept_action(&raster)
                    == Some(expected),
            );
        }

        println!(
            "FRESH_G11_SUB sub={} full={}/8 atoms={} l1_promoted={} l2_promoted={} max_l1={:.4} min_l2={:.4}",
            sub,
            sub_score,
            full_evo.concept_atoms().len(),
            full_evo.phase_native_promoted_concept_count(),
            full_evo.phase_native_promoted_recursive_count(),
            fresh_max_l1_evidence(&full_evo, block),
            fresh_min_l2_evidence(&full_evo, block),
        );
    }

    let (lo, hi) = g11_wilson95(full, 80);
    println!(
        "FRESH_G11_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} no_recursion={}/80 level1_only={}/80 zero_phase={}/80 zero_weight={}/80 no_growth={}/80 lesion={}/20 phase_shift={}/20 restored={}/20 unrelated={}/10 lower_lesion={}/20 atom_violations={} l1_violations={} l2_violations={} ref_violations={} max_l1={:.6} min_l2={:.6} motor_mask={:#06b}",
        full,
        lo,
        hi,
        per_seed,
        no_recursion,
        level1_only,
        zero_phase,
        zero_weight,
        no_growth,
        lesion_ok,
        phase_ok,
        restored_ok,
        unrelated_ok,
        lower_lesion_ok,
        atom_count_violations,
        level1_count_violations,
        level2_count_violations,
        ref_violations,
        max_l1,
        min_l2,
        motor_role_mask,
    );

    assert!(full >= 76);
    assert!(lo >= 0.87);
    assert!(per_seed.iter().all(|score| *score >= 6));
    assert_eq!(atom_count_violations, 0);
    assert_eq!(level1_count_violations, 0);
    assert_eq!(level2_count_violations, 0);
    assert_eq!(ref_violations, 0);
    assert!(no_recursion <= 16);
    assert!(level1_only <= 48);
    assert!(zero_phase <= 48);
    assert!(zero_weight <= 48);
    assert!(no_growth <= 16);
    assert!(max_l1 <= 0.20 + 1.0e-6);
    assert!(min_l2 >= 0.60 - 1.0e-6);
    assert!(lesion_ok <= 4);
    assert!(phase_ok <= 4);
    assert!(restored_ok >= 19);
    assert!(unrelated_ok >= 9);
    assert!(lower_lesion_ok <= 4);
    assert_eq!(motor_role_mask, 0b1111);
}
