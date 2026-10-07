use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase};
use aeterna_v1::carrier::{AutoAbstractionMetrics, PhaseNativeConfig, PhaseRecursiveConceptInfo};

const OFFSETS: [(usize, usize); 8] = [
    (1, 0), (0, 1), (1, 1), (2, 0),
    (0, 2), (2, 1), (1, 2), (2, 2),
];

const LEVEL1: [[usize; 2]; 4] = [
    [0, 1], [2, 7], [3, 6], [4, 5],
];

const NEGATIVE: [[[usize; 2]; 4]; 4] = [
    [[0, 7], [1, 6], [2, 5], [3, 4]],
    [[0, 6], [7, 5], [1, 4], [2, 3]],
    [[0, 5], [6, 4], [7, 3], [1, 2]],
    [[0, 4], [5, 3], [6, 2], [7, 1]],
];

const FOUNDATION: [((usize, usize), (usize, usize)); 4] = [
    ((1, 1), (8, 8)),
    ((2, 1), (7, 7)),
    ((1, 3), (8, 5)),
    ((3, 1), (6, 7)),
];

const TOP_PAIRS: [(usize, usize, bool); 4] = [
    (0, 1, false),
    (2, 3, false),
    (0, 2, true),
    (1, 3, true),
];

const HELDOUT: [[(usize, usize); 4]; 4] = [
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
        LEVEL1[left_concept][0],
        LEVEL1[left_concept][1],
        LEVEL1[right_concept][0],
        LEVEL1[right_concept][1],
    ];
    for (atom, (x, y)) in atoms.into_iter().zip(layout) {
        let (dx, dy) = OFFSETS[atom];
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn single_level1_scene(concept: usize, binding: usize) -> Vec<f32> {
    let left = (1 + binding % 3, 2 + (binding / 2) % 2);
    let right = (7 + (binding / 3) % 2, 7 + binding % 2);
    pair_scene(LEVEL1[concept][0], LEVEL1[concept][1], left, right)
}

fn residual_layout(index: usize) -> [(usize, usize); 4] {
    [
        (1 + index % 2, 1 + (index / 2) % 2),
        (1 + (index / 4) % 2, 7 + (index / 8) % 2),
        (7 + (index / 8) % 2, 1 + (index / 4) % 2),
        (7 + (index / 2) % 2, 7 + index % 2),
    ]
}

fn top_expected(class_one: bool, swap: bool) -> usize {
    if class_one ^ swap { 3 } else { 2 }
}

fn simple_expected(concept: usize, swap: bool) -> usize {
    let class_one = concept >= 2;
    top_expected(class_one, swap)
}

fn carrier() -> EvoPhase {
    let cfg = EvoConfig {
        sensory_cells: 144,
        motor_cells: 4,
        dormant_cells: 256,
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
    assert!(evo.enable_phase_native_auto_abstraction());
    evo
}

fn train_level1_adaptively(evo: &mut EvoPhase) {
    for pair in LEVEL1 {
        for (left, right) in FOUNDATION {
            let raster = pair_scene(pair[0], pair[1], left, right);
            for action in [0usize, 1usize] {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == 0,
                ));
            }
        }
    }

    for (matching_index, matching) in NEGATIVE.into_iter().enumerate() {
        let binding = FOUNDATION[matching_index];
        for pair in matching {
            let raster = pair_scene(pair[0], pair[1], binding.0, binding.1);
            for action in [0usize, 1usize] {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == 1,
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(), 8);
    assert_eq!(evo.phase_native_promoted_concept_count(), 4);
    assert_eq!(evo.phase_native_recursive_circuits().len(), 0);
}

fn adequate_child_regime(evo: &mut EvoPhase, swap: bool) {
    // Each L1 individually learns a reliable top-action explanation.
    for concept in 0..4 {
        for binding in 0..4 {
            let raster = single_level1_scene(concept, binding);
            let correct = simple_expected(concept, swap);
            for action in [2usize, 3usize] {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == correct,
                ));
            }
        }
    }

    // Two same-class pair scenes: two L1 concepts are co-active, but their
    // individual explanations agree and remain sufficient.
    for layout_index in 0..2 {
        let layout = residual_layout(layout_index);
        for (left, right, class_one) in [
            (0usize, 1usize, false),
            (2usize, 3usize, true),
        ] {
            let raster = top_scene(left, right, layout);
            let correct = top_expected(class_one, swap);
            for action in [2usize, 3usize] {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == correct,
                ));
            }
        }
    }
}

fn residual_regime(evo: &mut EvoPhase, swap: bool) {
    // Enough balanced contradictory experience to make the individual L1
    // explanations weak, then still leave >=8 pair facts after the crossing.
    for layout_index in 0..16 {
        let layout = residual_layout(layout_index);
        for (left, right, class_one) in TOP_PAIRS {
            let raster = top_scene(left, right, layout);
            let correct = top_expected(class_one, swap);
            for action in [2usize, 3usize] {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == correct,
                ));
            }
        }
    }
}

fn freeze(evo: &mut EvoPhase) {
    evo.set_concept_learning_enabled(false);
    evo.set_planning_learning_enabled(false);
}

fn score(evo: &EvoPhase, swap: bool) -> usize {
    TOP_PAIRS
        .into_iter()
        .enumerate()
        .map(|(index, (left, right, class_one))| {
            let raster = top_scene(left, right, HELDOUT[index]);
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
    assert_eq!(ids.len(), 2);
    let pair = [ids[0], ids[1]];
    evo.phase_native_recursive_circuits()
        .iter()
        .find(|circuit| circuit.promoted && circuit.child_concept_ids == pair)
        .expect("auto-promoted recursive circuit")
        .clone()
}

fn level1_only_score(evo: &EvoPhase, swap: bool) -> usize {
    TOP_PAIRS
        .into_iter()
        .enumerate()
        .map(|(index, (left, right, class_one))| {
            let raster = top_scene(left, right, HELDOUT[index]);
            let ids = active_level1_ids(evo, &raster);
            let mut scores = [0.0f32; 2];
            for id in ids {
                let circuit = evo
                    .phase_native_concept_circuits()
                    .iter()
                    .find(|c| c.promoted && c.concept_id == id)
                    .unwrap();
                for (slot, action) in [2usize, 3usize].into_iter().enumerate() {
                    let syn = evo
                        .phase_native_synapse(circuit.motor_synapses[action])
                        .unwrap();
                    scores[slot] += 2.0 * syn.weight - 1.0;
                }
            }
            let chosen = if scores[1] > scores[0] + 1.0e-6 { 3 } else { 2 };
            usize::from(chosen == top_expected(class_one, swap))
        })
        .sum()
}

fn max_top_child_evidence(evo: &EvoPhase) -> f32 {
    evo.phase_native_concept_circuits()
        .iter()
        .filter(|c| c.promoted)
        .flat_map(|circuit| {
            [2usize, 3usize].into_iter().map(|action| {
                let syn = evo
                    .phase_native_synapse(circuit.motor_synapses[action])
                    .unwrap();
                (2.0 * syn.weight - 1.0).abs()
            })
        })
        .fold(0.0f32, f32::max)
}

#[test]
fn g12_self_triggers_higher_abstraction_only_after_child_explanation_fails() {
    let mut full_total = 0usize;
    let mut no_escalation_total = 0usize;
    let mut frozen_simple_total = 0usize;
    let mut zero_phase_total = 0usize;
    let mut zero_weight_total = 0usize;
    let mut lesion_total = 0usize;
    let mut phase_total = 0usize;
    let mut restored_total = 0usize;
    let mut unrelated_total = 0usize;
    let mut lower_lesion_total = 0usize;
    let mut always_pre_residual_candidates = 0usize;
    let mut full_pre_residual_candidates = 0usize;

    for swap in [false, true] {
        let mut stage1 = carrier();
        train_level1_adaptively(&mut stage1);

        let mut full = stage1.clone();
        adequate_child_regime(&mut full, swap);
        let adequate_metrics = full.phase_native_auto_abstraction_metrics().unwrap();
        assert_eq!(adequate_metrics.recursive_candidates, 0);
        assert_eq!(adequate_metrics.recursive_promoted, 0);
        full_pre_residual_candidates += adequate_metrics.recursive_candidates;

        let mut always = stage1.clone();
        always.set_phase_native_auto_always_escalate_for_control(true);
        adequate_child_regime(&mut always, swap);
        let always_metrics = always.phase_native_auto_abstraction_metrics().unwrap();
        assert!(
            always_metrics.recursive_candidates > 0,
            "always-escalate baseline must waste structure before residual pressure"
        );
        always_pre_residual_candidates += always_metrics.recursive_candidates;

        let mut no_escalation = full.clone();
        no_escalation.set_phase_native_auto_abstraction_enabled(false);

        let mut frozen_simple = full.clone();
        frozen_simple.set_phase_native_auto_child_revision_for_control(false);

        let mut zero_phase = full.clone();
        zero_phase.set_learning_rates_for_control(1.0, 0.0);

        let mut zero_weight = full.clone();
        zero_weight.set_learning_rates_for_control(0.0, 1.0);

        residual_regime(&mut full, swap);
        residual_regime(&mut no_escalation, swap);
        residual_regime(&mut frozen_simple, swap);
        residual_regime(&mut zero_phase, swap);
        residual_regime(&mut zero_weight, swap);

        let metrics: AutoAbstractionMetrics =
            full.phase_native_auto_abstraction_metrics().unwrap();
        let recursive_diag = full
            .phase_native_recursive_circuits()
            .iter()
            .map(|circuit| {
                let weights = [2usize, 3usize].map(|action| {
                    full.phase_native_synapse(circuit.motor_synapses[action])
                        .map(|syn| syn.weight)
                        .unwrap_or(-1.0)
                });
                (
                    circuit.concept_id,
                    circuit.child_concept_ids,
                    circuit.support,
                    circuit.promoted,
                    weights,
                )
            })
            .collect::<Vec<_>>();
        println!(
            "G12_DIAG swap={} metrics={:?} recursive={:?} max_child={:.6}",
            swap,
            metrics,
            recursive_diag,
            max_top_child_evidence(&full),
        );
        assert_eq!(metrics.recursive_promoted, 4);
        assert!(metrics.first_weak_observation.is_some());
        assert!(metrics.first_candidate_observation.is_some());
        assert!(metrics.first_promotion_observation.is_some());
        assert!(
            metrics.first_candidate_observation.unwrap()
                >= metrics.first_weak_observation.unwrap(),
            "higher structure must not appear before supported child evidence becomes weak"
        );

        freeze(&mut full);
        freeze(&mut no_escalation);
        freeze(&mut frozen_simple);
        freeze(&mut zero_phase);
        freeze(&mut zero_weight);

        full_total += score(&full, swap);
        no_escalation_total += score(&no_escalation, swap);
        frozen_simple_total += score(&frozen_simple, swap);
        zero_phase_total += score(&zero_phase, swap);
        zero_weight_total += score(&zero_weight, swap);

        assert!(max_top_child_evidence(&full) <= 0.20 + 1.0e-6);

        let target = top_scene(TOP_PAIRS[0].0, TOP_PAIRS[0].1, HELDOUT[0]);
        let recursive = recursive_for_scene(&full, &target);

        let mut lesioned = full.clone();
        let saved = lesioned
            .perturb_phase_native_synapse_for_control(
                recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .unwrap();
        lesion_total += score(&lesioned, swap);
        lesioned.restore_phase_native_synapse_for_control(
            recursive.child_synapses[0],
            saved,
        );
        restored_total += score(&lesioned, swap);

        let mut phase_shifted = full.clone();
        phase_shifted
            .perturb_phase_native_synapse_for_control(
                recursive.child_synapses[0],
                1.0,
                std::f32::consts::PI,
            )
            .unwrap();
        phase_total += score(&phase_shifted, swap);

        let unrelated_scene =
            top_scene(TOP_PAIRS[1].0, TOP_PAIRS[1].1, HELDOUT[1]);
        let unrelated_recursive = recursive_for_scene(&full, &unrelated_scene);
        let mut unrelated = full.clone();
        unrelated
            .perturb_phase_native_synapse_for_control(
                unrelated_recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .unwrap();
        unrelated_total += usize::from(
            unrelated.choose_phase_native_recursive_concept_action(&target)
                == Some(top_expected(TOP_PAIRS[0].2, swap)),
        );

        let lower_id = recursive.child_concept_ids[0];
        let lower = full
            .phase_native_concept_circuits()
            .iter()
            .find(|c| c.promoted && c.concept_id == lower_id)
            .unwrap()
            .clone();
        let mut lower_lesioned = full.clone();
        lower_lesioned
            .perturb_phase_native_synapse_for_control(
                lower.child_synapses[0],
                0.0,
                0.0,
            )
            .unwrap();
        lower_lesion_total += usize::from(
            lower_lesioned
                .choose_phase_native_recursive_concept_action(&target)
                == Some(top_expected(TOP_PAIRS[0].2, swap)),
        );

        // Diagnostic only. The frozen G12 protocol does not gate PASS on
        // a per-permutation LEVEL1_ONLY ceiling; preregistered causal controls
        // are NO_ESCALATION / FROZEN_SIMPLE / ZERO_* plus lesions.
        let level1_only_diag = level1_only_score(&full, swap);
        println!(
            "G12_LEVEL1_ONLY_DIAG swap={} score={}/4",
            swap,
            level1_only_diag
        );
    }

    println!(
        "G12_AUTO full={}/8 no_escalation={}/8 frozen_simple={}/8 zero_phase={}/8 zero_weight={}/8 lesion={}/8 phase_shift={}/8 restored={}/8 unrelated={}/2 lower_lesion_success={}/2 pre_residual_full={} pre_residual_always={}",
        full_total,
        no_escalation_total,
        frozen_simple_total,
        zero_phase_total,
        zero_weight_total,
        lesion_total,
        phase_total,
        restored_total,
        unrelated_total,
        lower_lesion_total,
        full_pre_residual_candidates,
        always_pre_residual_candidates,
    );

    assert_eq!(full_total, 8);
    assert!(no_escalation_total <= 4);
    assert!(frozen_simple_total <= 4);
    assert!(zero_phase_total <= 4);
    assert!(zero_weight_total <= 4);
    assert!(always_pre_residual_candidates > full_pre_residual_candidates);
    assert!(lesion_total <= 6);
    assert!(phase_total <= 6);
    assert_eq!(restored_total, 8);
    assert_eq!(unrelated_total, 2);
    assert_eq!(lower_lesion_total, 0);
}

#[test]
fn g12_full_learning_path_has_no_explicit_recursive_tuition_or_task_phase_fallback() {
    let source = include_str!("../src/phase_auto_abstraction.rs");
    let start = source
        .find("pub fn observe_phase_native_adaptive_concept_factual")
        .expect("G12 adaptive factual entry");
    let path = &source[start..];

    for forbidden in [
        "observe_phase_native_recursive_concept_factual",
        "TOP_PAIRS",
        "LEVEL1",
        "task_phase",
        "phase_label",
        "EvoImaginationPlanner",
        "VecDeque",
        "BinaryHeap",
    ] {
        assert!(
            !path.contains(forbidden),
            "G12 adaptive path contains forbidden token {forbidden}"
        );
    }

    for required in [
        "active_level1_concepts",
        "action_support",
        "child_ceiling",
        "recursive",
        "structural_growth_enabled",
        "learn_phase_concept_running_mean_synapse",
    ] {
        assert!(
            path.contains(required),
            "G12 adaptive path missing {required}"
        );
    }
}


const G12_FACTOR8: [[[usize; 2]; 4]; 7] = [
    [[0, 7], [1, 6], [2, 5], [3, 4]],
    [[0, 6], [7, 5], [1, 4], [2, 3]],
    [[0, 5], [6, 4], [7, 3], [1, 2]],
    [[0, 4], [5, 3], [6, 2], [7, 1]],
    [[0, 3], [4, 2], [5, 1], [6, 7]],
    [[0, 2], [3, 1], [4, 7], [5, 6]],
    [[0, 1], [2, 7], [3, 6], [4, 5]],
];

const G12_FACTOR4: [[[usize; 2]; 2]; 3] = [
    [[0, 1], [2, 3]],
    [[0, 2], [1, 3]],
    [[0, 3], [1, 2]],
];

const G12_PAIR_BINDING_BANK: [((usize, usize), (usize, usize)); 8] = [
    ((1, 1), (8, 8)),
    ((2, 1), (7, 7)),
    ((1, 2), (8, 7)),
    ((2, 2), (7, 8)),
    ((1, 3), (8, 6)),
    ((3, 1), (6, 8)),
    ((2, 3), (8, 5)),
    ((3, 2), (5, 8)),
];

const G12_TOP_LAYOUT_BANK: [[(usize, usize); 4]; 12] = [
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
struct FreshG12Block {
    offsets: [(usize, usize); 8],
    level1_targets: [[usize; 2]; 4],
    negative_matchings: [[[usize; 2]; 4]; 4],
    foundation_actions: [usize; 2],
    top_actions: [usize; 2],
    simple_pairs: [[usize; 2]; 2],
    residual_pairs: [(usize, usize, bool); 4],
    target_bindings: [((usize, usize), (usize, usize)); 4],
    negative_bindings: [((usize, usize), (usize, usize)); 4],
    single_bindings: [((usize, usize), (usize, usize)); 6],
    top_tuition: [[(usize, usize); 4]; 4],
    top_heldout: [[(usize, usize); 4]; 8],
    single_repetitions: usize,
    simple_pair_repetitions: usize,
    residual_cycles: usize,
    residual_orders: Vec<[usize; 4]>,
}

struct FreshG12Rng(u64);

impl FreshG12Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x612A_B57A_C710_0012)
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

fn g12_shuffle<T>(rng: &mut FreshG12Rng, values: &mut [T]) {
    for i in (1..values.len()).rev() {
        let j = rng.range(i + 1);
        values.swap(i, j);
    }
}

fn g12_mix(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn fresh_g12_blocks(authority: u64) -> (Vec<FreshG12Block>, u64) {
    let mut blocks = Vec::new();
    let mut digest = 14_695_981_039_346_656_037_u64;

    for sub in 0..10u64 {
        let derived = authority
            ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ 0x612A_5EED_D2C0_000C;
        let mut rng = FreshG12Rng::new(derived);

        let mut offsets = OFFSETS.to_vec();
        g12_shuffle(&mut rng, &mut offsets);
        let offsets: [(usize, usize); 8] = offsets.try_into().unwrap();

        let target_round = rng.range(7);
        let mut level1_targets = G12_FACTOR8[target_round];
        g12_shuffle(&mut rng, &mut level1_targets);

        let mut other_rounds = (0..7usize)
            .filter(|round| *round != target_round)
            .collect::<Vec<_>>();
        g12_shuffle(&mut rng, &mut other_rounds);
        let negative_matchings = [
            G12_FACTOR8[other_rounds[0]],
            G12_FACTOR8[other_rounds[1]],
            G12_FACTOR8[other_rounds[2]],
            G12_FACTOR8[other_rounds[3]],
        ];

        let mut motors = [0usize, 1, 2, 3];
        g12_shuffle(&mut rng, &mut motors);
        let foundation_actions = [motors[0], motors[1]];
        let top_actions = [motors[2], motors[3]];

        let simple_round = rng.range(3);
        let mut simple_pairs = G12_FACTOR4[simple_round];
        g12_shuffle(&mut rng, &mut simple_pairs);

        let residual_class0_round = rng.range(3);
        let mut class1_choices = (0..3usize)
            .filter(|round| *round != residual_class0_round)
            .collect::<Vec<_>>();
        g12_shuffle(&mut rng, &mut class1_choices);
        let residual_class1_round = class1_choices[0];
        let residual_pairs = [
            (
                G12_FACTOR4[residual_class0_round][0][0],
                G12_FACTOR4[residual_class0_round][0][1],
                false,
            ),
            (
                G12_FACTOR4[residual_class0_round][1][0],
                G12_FACTOR4[residual_class0_round][1][1],
                false,
            ),
            (
                G12_FACTOR4[residual_class1_round][0][0],
                G12_FACTOR4[residual_class1_round][0][1],
                true,
            ),
            (
                G12_FACTOR4[residual_class1_round][1][0],
                G12_FACTOR4[residual_class1_round][1][1],
                true,
            ),
        ];

        let mut pair_bindings = G12_PAIR_BINDING_BANK.to_vec();
        g12_shuffle(&mut rng, &mut pair_bindings);
        let target_bindings: [((usize, usize), (usize, usize)); 4] =
            pair_bindings[..4].try_into().unwrap();
        let negative_bindings: [((usize, usize), (usize, usize)); 4] =
            pair_bindings[4..8].try_into().unwrap();

        let mut single_bank = G12_PAIR_BINDING_BANK.to_vec();
        g12_shuffle(&mut rng, &mut single_bank);
        let single_bindings: [((usize, usize), (usize, usize)); 6] =
            single_bank[..6].try_into().unwrap();

        let mut top_layouts = G12_TOP_LAYOUT_BANK.to_vec();
        g12_shuffle(&mut rng, &mut top_layouts);
        let top_tuition: [[(usize, usize); 4]; 4] =
            top_layouts[..4].try_into().unwrap();
        let top_heldout: [[(usize, usize); 4]; 8] =
            top_layouts[4..12].try_into().unwrap();

        let single_repetitions = 4 + rng.range(3);
        let simple_pair_repetitions = 2 + rng.range(2);
        let residual_cycles = 24 + rng.range(5);
        let mut residual_orders = Vec::with_capacity(residual_cycles);
        for _ in 0..residual_cycles {
            let mut order = [0usize, 1, 2, 3];
            g12_shuffle(&mut rng, &mut order);
            residual_orders.push(order);
        }

        digest = g12_mix(digest, sub);
        for (dx, dy) in offsets {
            digest = g12_mix(digest, dx as u64);
            digest = g12_mix(digest, dy as u64);
        }
        for pair in level1_targets {
            digest = g12_mix(digest, pair[0] as u64);
            digest = g12_mix(digest, pair[1] as u64);
        }
        for matching in negative_matchings {
            for pair in matching {
                digest = g12_mix(digest, pair[0] as u64);
                digest = g12_mix(digest, pair[1] as u64);
            }
        }
        for motor in foundation_actions.into_iter().chain(top_actions) {
            digest = g12_mix(digest, motor as u64);
        }
        for pair in simple_pairs {
            digest = g12_mix(digest, pair[0] as u64);
            digest = g12_mix(digest, pair[1] as u64);
        }
        for (a, b, class_one) in residual_pairs {
            digest = g12_mix(digest, a as u64);
            digest = g12_mix(digest, b as u64);
            digest = g12_mix(digest, class_one as u64);
        }
        for binding in target_bindings.into_iter().chain(negative_bindings) {
            digest = g12_mix(digest, binding.0.0 as u64);
            digest = g12_mix(digest, binding.0.1 as u64);
            digest = g12_mix(digest, binding.1.0 as u64);
            digest = g12_mix(digest, binding.1.1 as u64);
        }
        for binding in single_bindings {
            digest = g12_mix(digest, binding.0.0 as u64);
            digest = g12_mix(digest, binding.0.1 as u64);
            digest = g12_mix(digest, binding.1.0 as u64);
            digest = g12_mix(digest, binding.1.1 as u64);
        }
        for layout in top_tuition.into_iter().chain(top_heldout) {
            for (x, y) in layout {
                digest = g12_mix(digest, x as u64);
                digest = g12_mix(digest, y as u64);
            }
        }
        digest = g12_mix(digest, single_repetitions as u64);
        digest = g12_mix(digest, simple_pair_repetitions as u64);
        digest = g12_mix(digest, residual_cycles as u64);
        for order in &residual_orders {
            for index in order {
                digest = g12_mix(digest, *index as u64);
            }
        }

        blocks.push(FreshG12Block {
            offsets,
            level1_targets,
            negative_matchings,
            foundation_actions,
            top_actions,
            simple_pairs,
            residual_pairs,
            target_bindings,
            negative_bindings,
            single_bindings,
            top_tuition,
            top_heldout,
            single_repetitions,
            simple_pair_repetitions,
            residual_cycles,
            residual_orders,
        });
    }

    (blocks, digest)
}

fn fresh_g12_pair_scene(
    block: &FreshG12Block,
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

fn fresh_g12_top_scene(
    block: &FreshG12Block,
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

fn fresh_g12_top_expected(block: &FreshG12Block, class_one: bool) -> usize {
    if class_one {
        block.top_actions[1]
    } else {
        block.top_actions[0]
    }
}

fn fresh_g12_simple_expected(block: &FreshG12Block, concept: usize) -> usize {
    if block.simple_pairs[0].contains(&concept) {
        block.top_actions[0]
    } else {
        block.top_actions[1]
    }
}

fn train_fresh_g12_level1(evo: &mut EvoPhase, block: &FreshG12Block) {
    for pair in block.level1_targets {
        for binding in block.target_bindings {
            let raster = fresh_g12_pair_scene(block, pair[0], pair[1], binding);
            for action in block.foundation_actions {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
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
            let raster = fresh_g12_pair_scene(block, pair[0], pair[1], binding);
            for action in block.foundation_actions {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == block.foundation_actions[1],
                ));
            }
        }
    }
}

fn fresh_g12_adequate_regime(evo: &mut EvoPhase, block: &FreshG12Block) {
    for concept in 0..4usize {
        for rep in 0..block.single_repetitions {
            let pair = block.level1_targets[concept];
            let raster = fresh_g12_pair_scene(
                block,
                pair[0],
                pair[1],
                block.single_bindings[rep],
            );
            let correct = fresh_g12_simple_expected(block, concept);
            for action in block.top_actions {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == correct,
                ));
            }
        }
    }

    for rep in 0..block.simple_pair_repetitions {
        let layout = block.top_tuition[rep];
        for group in 0..2usize {
            let pair = block.simple_pairs[group];
            let raster = fresh_g12_top_scene(block, pair[0], pair[1], layout);
            let correct = block.top_actions[group];
            for action in block.top_actions {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == correct,
                ));
            }
        }
    }
}

fn fresh_g12_residual_regime(evo: &mut EvoPhase, block: &FreshG12Block) {
    for cycle in 0..block.residual_cycles {
        let layout = block.top_tuition[cycle % block.top_tuition.len()];
        for pair_index in block.residual_orders[cycle] {
            let (left, right, class_one) = block.residual_pairs[pair_index];
            let raster = fresh_g12_top_scene(block, left, right, layout);
            let correct = fresh_g12_top_expected(block, class_one);
            for action in block.top_actions {
                assert!(evo.observe_phase_native_adaptive_concept_factual(
                    &raster,
                    action,
                    action == correct,
                ));
            }
        }
    }
}

fn score_fresh_g12(evo: &EvoPhase, block: &FreshG12Block) -> usize {
    let mut score = 0usize;
    for (pair_index, (left, right, class_one)) in
        block.residual_pairs.into_iter().enumerate()
    {
        for layout_index in 0..2usize {
            let raster = fresh_g12_top_scene(
                block,
                left,
                right,
                block.top_heldout[pair_index * 2 + layout_index],
            );
            score += usize::from(
                evo.choose_phase_native_recursive_concept_action(&raster)
                    == Some(fresh_g12_top_expected(block, class_one)),
            );
        }
    }
    score
}

fn fresh_g12_max_child_evidence(
    evo: &EvoPhase,
    block: &FreshG12Block,
) -> f32 {
    evo.phase_native_concept_circuits()
        .iter()
        .filter(|circuit| circuit.promoted)
        .flat_map(|circuit| {
            block.top_actions.into_iter().map(|action| {
                let synapse = evo
                    .phase_native_synapse(circuit.motor_synapses[action])
                    .expect("fresh G12 child evidence");
                (2.0 * synapse.weight - 1.0).abs()
            })
        })
        .fold(0.0f32, f32::max)
}

fn fresh_g12_recursive_for_scene(
    evo: &EvoPhase,
    sensory: &[f32],
) -> PhaseRecursiveConceptInfo {
    let ids = active_level1_ids(evo, sensory);
    assert_eq!(ids.len(), 2);
    let pair = [ids[0], ids[1]];
    evo.phase_native_recursive_circuits()
        .iter()
        .find(|circuit| circuit.promoted && circuit.child_concept_ids == pair)
        .expect("fresh G12 promoted recursive circuit")
        .clone()
}

fn g12_wilson95(success: usize, n: usize) -> (f64, f64) {
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
fn g12_fresh_self_triggered_abstraction_pack() {
    let authority: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse()
        .expect("fresh G12 seed must be u64");
    let source_sha =
        std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha =
        std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let (blocks, digest) = fresh_g12_blocks(authority);
    println!(
        "FRESH_G12_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha, spec_sha, authority, digest
    );
    for (sub, block) in blocks.iter().enumerate() {
        println!("FRESH_G12_BLOCK sub={} {:?}", sub, block);
    }

    let mut full = 0usize;
    let mut no_escalation = 0usize;
    let mut frozen_simple = 0usize;
    let mut zero_phase = 0usize;
    let mut zero_weight = 0usize;
    let mut per_seed = Vec::new();
    let mut atom_l1_violations = 0usize;
    let mut adequate_structure_violations = 0usize;
    let mut final_structure_violations = 0usize;
    let mut timing_violations = 0usize;
    let mut missing_timing = 0usize;
    let mut economy_violations = 0usize;
    let mut max_child = 0.0f32;
    let mut lesion_ok = 0usize;
    let mut phase_ok = 0usize;
    let mut restored_ok = 0usize;
    let mut unrelated_ok = 0usize;
    let mut lower_lesion_ok = 0usize;
    let mut motor_mask = 0u8;
    let mut weak_points = Vec::new();
    let mut candidate_points = Vec::new();
    let mut promotion_points = Vec::new();
    let mut always_pre_counts = Vec::new();

    for (sub, block) in blocks.iter().enumerate() {
        for motor in block
            .foundation_actions
            .into_iter()
            .chain(block.top_actions)
        {
            motor_mask |= 1u8 << motor;
        }

        let mut stage1 = carrier();
        train_fresh_g12_level1(&mut stage1, block);
        if stage1.concept_atoms().len() != 8
            || stage1.phase_native_promoted_concept_count() != 4
            || !stage1.phase_native_recursive_circuits().is_empty()
        {
            atom_l1_violations += 1;
        }

        let mut full_evo = stage1.clone();
        fresh_g12_adequate_regime(&mut full_evo, block);
        let adequate = full_evo
            .phase_native_auto_abstraction_metrics()
            .expect("fresh G12 adequate metrics");
        if adequate.recursive_candidates != 0 || adequate.recursive_promoted != 0 {
            adequate_structure_violations += 1;
        }

        let mut always = stage1.clone();
        always.set_phase_native_auto_always_escalate_for_control(true);
        fresh_g12_adequate_regime(&mut always, block);
        let always_metrics = always
            .phase_native_auto_abstraction_metrics()
            .expect("fresh G12 always metrics");
        always_pre_counts.push(always_metrics.recursive_candidates);
        if always_metrics.recursive_candidates <= adequate.recursive_candidates {
            economy_violations += 1;
        }

        let mut no_e = full_evo.clone();
        no_e.set_phase_native_auto_abstraction_enabled(false);

        let mut frozen = full_evo.clone();
        frozen.set_phase_native_auto_child_revision_for_control(false);

        let mut zp = full_evo.clone();
        zp.set_learning_rates_for_control(1.0, 0.0);

        let mut zw = full_evo.clone();
        zw.set_learning_rates_for_control(0.0, 1.0);

        fresh_g12_residual_regime(&mut full_evo, block);
        fresh_g12_residual_regime(&mut no_e, block);
        fresh_g12_residual_regime(&mut frozen, block);
        fresh_g12_residual_regime(&mut zp, block);
        fresh_g12_residual_regime(&mut zw, block);

        let metrics = full_evo
            .phase_native_auto_abstraction_metrics()
            .expect("fresh G12 final metrics");
        if metrics.recursive_candidates != 4 || metrics.recursive_promoted != 4 {
            final_structure_violations += 1;
        }

        match (
            metrics.first_weak_observation,
            metrics.first_candidate_observation,
            metrics.first_promotion_observation,
        ) {
            (Some(weak), Some(candidate), Some(promotion)) => {
                weak_points.push(weak);
                candidate_points.push(candidate);
                promotion_points.push(promotion);
                if candidate < weak || promotion < candidate {
                    timing_violations += 1;
                }
            }
            _ => missing_timing += 1,
        }

        max_child = max_child.max(fresh_g12_max_child_evidence(&full_evo, block));

        freeze(&mut full_evo);
        freeze(&mut no_e);
        freeze(&mut frozen);
        freeze(&mut zp);
        freeze(&mut zw);

        let sub_score = score_fresh_g12(&full_evo, block);
        full += sub_score;
        per_seed.push(sub_score);
        no_escalation += score_fresh_g12(&no_e, block);
        frozen_simple += score_fresh_g12(&frozen, block);
        zero_phase += score_fresh_g12(&zp, block);
        zero_weight += score_fresh_g12(&zw, block);

        let first_pair = block.residual_pairs[0];
        let expected = fresh_g12_top_expected(block, first_pair.2);
        let first_scene = fresh_g12_top_scene(
            block,
            first_pair.0,
            first_pair.1,
            block.top_heldout[0],
        );
        let recursive = fresh_g12_recursive_for_scene(&full_evo, &first_scene);

        let mut lesioned = full_evo.clone();
        let saved = lesioned
            .perturb_phase_native_synapse_for_control(
                recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh G12 necessary L1->L2 synapse");
        for layout_index in 0..2 {
            let raster = fresh_g12_top_scene(
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
            let raster = fresh_g12_top_scene(
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
            .expect("fresh G12 phase intervention");
        for layout_index in 0..2 {
            let raster = fresh_g12_top_scene(
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

        let other_pair = block.residual_pairs[1];
        let other_scene = fresh_g12_top_scene(
            block,
            other_pair.0,
            other_pair.1,
            block.top_heldout[2],
        );
        let other_recursive =
            fresh_g12_recursive_for_scene(&full_evo, &other_scene);
        let mut unrelated = full_evo.clone();
        unrelated
            .perturb_phase_native_synapse_for_control(
                other_recursive.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh G12 unrelated L2 synapse");
        unrelated_ok += usize::from(
            unrelated.choose_phase_native_recursive_concept_action(&first_scene)
                == Some(expected),
        );

        let lower_id = recursive.child_concept_ids[0];
        let lower = full_evo
            .phase_native_concept_circuits()
            .iter()
            .find(|circuit| circuit.promoted && circuit.concept_id == lower_id)
            .expect("fresh G12 lower L1 child")
            .clone();
        let mut lower_lesioned = full_evo.clone();
        lower_lesioned
            .perturb_phase_native_synapse_for_control(
                lower.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh G12 lower dependency");
        for layout_index in 0..2 {
            let raster = fresh_g12_top_scene(
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
            "FRESH_G12_SUB sub={} full={}/8 adequate_candidates={} always_candidates={} final_candidates={} final_promoted={} weak={:?} candidate={:?} promotion={:?} max_child={:.6} single_reps={} simple_pair_reps={} residual_cycles={}",
            sub,
            sub_score,
            adequate.recursive_candidates,
            always_metrics.recursive_candidates,
            metrics.recursive_candidates,
            metrics.recursive_promoted,
            metrics.first_weak_observation,
            metrics.first_candidate_observation,
            metrics.first_promotion_observation,
            fresh_g12_max_child_evidence(&full_evo, block),
            block.single_repetitions,
            block.simple_pair_repetitions,
            block.residual_cycles,
        );
    }

    let (lo, hi) = g12_wilson95(full, 80);
    println!(
        "FRESH_G12_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} no_escalation={}/80 frozen_simple={}/80 zero_phase={}/80 zero_weight={}/80 atom_l1_violations={} adequate_structure_violations={} final_structure_violations={} timing_violations={} missing_timing={} economy_violations={} max_child={:.6} lesion={}/20 phase_shift={}/20 restored={}/20 unrelated={}/10 lower_lesion={}/20 motor_mask={:#06b} weak_points={:?} candidate_points={:?} promotion_points={:?} always_pre_counts={:?}",
        full,
        lo,
        hi,
        per_seed,
        no_escalation,
        frozen_simple,
        zero_phase,
        zero_weight,
        atom_l1_violations,
        adequate_structure_violations,
        final_structure_violations,
        timing_violations,
        missing_timing,
        economy_violations,
        max_child,
        lesion_ok,
        phase_ok,
        restored_ok,
        unrelated_ok,
        lower_lesion_ok,
        motor_mask,
        weak_points,
        candidate_points,
        promotion_points,
        always_pre_counts,
    );

    assert!(full >= 76);
    assert!(lo >= 0.87);
    assert!(per_seed.iter().all(|score| *score >= 6));
    assert_eq!(atom_l1_violations, 0);
    assert_eq!(adequate_structure_violations, 0);
    assert_eq!(final_structure_violations, 0);
    assert_eq!(timing_violations, 0);
    assert_eq!(missing_timing, 0);
    assert_eq!(economy_violations, 0);
    assert!(no_escalation <= 40);
    assert!(frozen_simple <= 40);
    assert!(zero_phase <= 40);
    assert!(zero_weight <= 40);
    assert!(max_child <= 0.20 + 1.0e-6);
    assert!(lesion_ok <= 4);
    assert!(phase_ok <= 4);
    assert!(restored_ok >= 19);
    assert!(unrelated_ok >= 9);
    assert!(lower_lesion_ok <= 4);
    assert_eq!(motor_mask, 0b1111);
}
