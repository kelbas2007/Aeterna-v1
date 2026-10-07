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
