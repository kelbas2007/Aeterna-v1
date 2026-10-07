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
