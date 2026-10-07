use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase};
use aeterna_v1::carrier::PhaseNativeConfig;

#[derive(Clone, Copy, Debug)]
enum AtomKind { A, B, C, D }

fn delta(kind: AtomKind) -> (usize, usize) {
    match kind {
        AtomKind::A => (1, 0),
        AtomKind::B => (0, 1),
        AtomKind::C => (1, 1),
        AtomKind::D => (2, 0),
    }
}

fn pair_cases() -> [(AtomKind, AtomKind, bool); 4] {
    [
        (AtomKind::A, AtomKind::B, false),
        (AtomKind::C, AtomKind::D, false),
        (AtomKind::A, AtomKind::C, true),
        (AtomKind::B, AtomKind::D, true),
    ]
}

fn scene(
    first: AtomKind,
    second: AtomKind,
    first_origin: (usize, usize),
    second_origin: (usize, usize),
) -> Vec<f32> {
    let mut raster = vec![0.0f32; 144];
    for (kind, (x, y)) in [(first, first_origin), (second, second_origin)] {
        let (dx, dy) = delta(kind);
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn correct_action(class_one: bool, swap: bool) -> usize {
    usize::from(class_one ^ swap)
}

fn carrier(mode: &str) -> EvoPhase {
    let mut cfg = EvoConfig {
        sensory_cells: 144,
        motor_cells: 2,
        dormant_cells: 64,
        hdc_dim: 192,
        weight_learning_rate: 1.0,
        phase_learning_rate: 1.0,
        min_recruit_support: 1,
        ..EvoConfig::default()
    };
    match mode {
        "zero_phase" => cfg.phase_learning_rate = 0.0,
        "zero_weight" => cfg.weight_learning_rate = 0.0,
        "no_capacity" => cfg.dormant_cells = 0,
        "no_growth" => cfg.structural_growth_enabled = false,
        _ => {}
    }

    let mut evo = EvoPhase::new(cfg);
    let mut concept = ConceptConfig::for_raster(12, 12, 2, 192);
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

fn train(mode: &str, swap: bool) -> EvoPhase {
    let mut evo = carrier(mode);
    let bindings = [
        ((1usize, 1usize), (8usize, 8usize)),
        ((2, 1), (7, 7)),
        ((1, 3), (8, 5)),
        ((3, 1), (6, 7)),
    ];

    for (left, right) in bindings {
        for (first, second, class_one) in pair_cases() {
            let raster = scene(first, second, left, right);
            for action in 0..2 {
                let need = action == correct_action(class_one, swap);
                let _ = evo.observe_phase_native_concept_factual(&raster, action, need);
            }
        }
    }

    evo.set_concept_learning_enabled(false);
    evo.set_planning_learning_enabled(false);
    // Dedicated concept readout stays OFF: physical G10 must stand alone.
    evo.set_concept_readout_enabled(false);
    evo
}

fn heldout() -> [((AtomKind, AtomKind, bool), ((usize, usize), (usize, usize))); 4] {
    [
        ((AtomKind::A, AtomKind::B, false), ((3, 1), (6, 8))),
        ((AtomKind::C, AtomKind::D, false), ((1, 4), (8, 2))),
        ((AtomKind::A, AtomKind::C, true), ((3, 3), (7, 7))),
        ((AtomKind::B, AtomKind::D, true), ((2, 4), (7, 1))),
    ]
}

fn score(evo: &EvoPhase, swap: bool) -> usize {
    heldout()
        .into_iter()
        .map(|((first, second, class_one), (left, right))| {
            let raster = scene(first, second, left, right);
            usize::from(
                evo.choose_phase_native_concept_action(&raster)
                    == Some(correct_action(class_one, swap)),
            )
        })
        .sum()
}

fn circuit_for_pair(
    evo: &EvoPhase,
    first: AtomKind,
    second: AtomKind,
) -> aeterna_v1::carrier::PhaseConceptCircuitInfo {
    let raster = scene(first, second, (3, 1), (7, 7));
    let mut ids = evo.active_concept_atom_ids(&raster);
    ids.sort_unstable();
    let children = [ids[0], ids[1]];
    evo.phase_native_concept_circuits()
        .iter()
        .find(|circuit| circuit.child_ids == children)
        .expect("physical composite circuit for acquired pair")
        .clone()
}

#[test]
fn g10_physical_composite_readout_depends_on_actual_phase_synapses() {
    let mut intact_total = 0usize;
    let mut lesion_total = 0usize;
    let mut phase_total = 0usize;
    let mut restored_total = 0usize;
    let mut unrelated_target_ok = 0usize;

    for swap in [false, true] {
        let intact = train("full", swap);
        assert_eq!(intact.composite_concepts().len(), 4);
        assert_eq!(intact.phase_native_concept_atom_count(), 4);
        assert_eq!(intact.phase_native_concept_circuits().len(), 4);
        assert_eq!(
            intact.choose_composite_concept_action(
                &scene(AtomKind::A, AtomKind::B, (3, 1), (6, 8))
            ),
            None,
            "dedicated table readout must remain disabled"
        );

        let baseline = score(&intact, swap);
        assert_eq!(baseline, 4);
        intact_total += baseline;

        let ab = circuit_for_pair(&intact, AtomKind::A, AtomKind::B);
        let cd = circuit_for_pair(&intact, AtomKind::C, AtomKind::D);

        let mut lesioned = intact.clone();
        let saved = lesioned
            .perturb_phase_native_synapse_for_control(ab.child_synapses[0], 0.0, 0.0)
            .expect("necessary physical child synapse");
        let lost = score(&lesioned, swap);
        lesion_total += lost;
        assert!(
            lost <= baseline - 1,
            "necessary physical lesion must lose at least the AB decision"
        );

        lesioned.restore_phase_native_synapse_for_control(ab.child_synapses[0], saved);
        let recovered = score(&lesioned, swap);
        assert_eq!(recovered, baseline);
        restored_total += recovered;

        let mut phase_shifted = intact.clone();
        phase_shifted
            .perturb_phase_native_synapse_for_control(
                ab.child_synapses[0],
                1.0,
                std::f32::consts::PI,
            )
            .expect("necessary physical child synapse for phase shift");
        let phase_lost = score(&phase_shifted, swap);
        phase_total += phase_lost;
        assert!(
            phase_lost <= baseline - 1,
            "pi phase shift must remove at least the AB physical composite decision"
        );

        let mut unrelated = intact.clone();
        unrelated
            .perturb_phase_native_synapse_for_control(cd.child_synapses[0], 0.0, 0.0)
            .expect("unrelated physical concept synapse");
        let ab_raster = scene(AtomKind::A, AtomKind::B, (3, 1), (6, 8));
        unrelated_target_ok += usize::from(
            unrelated.choose_phase_native_concept_action(&ab_raster)
                == Some(correct_action(false, swap)),
        );
    }

    let zero_phase = score(&train("zero_phase", false), false)
        + score(&train("zero_phase", true), true);
    let zero_weight = score(&train("zero_weight", false), false)
        + score(&train("zero_weight", true), true);
    let no_capacity = score(&train("no_capacity", false), false)
        + score(&train("no_capacity", true), true);
    let no_growth = score(&train("no_growth", false), false)
        + score(&train("no_growth", true), true);

    println!(
        "G10_PHYS intact={}/8 lesion={}/8 phase_shift={}/8 restored={}/8 unrelated_target={}/2 zero_phase={}/8 zero_weight={}/8 no_capacity={}/8 no_growth={}/8",
        intact_total,
        lesion_total,
        phase_total,
        restored_total,
        unrelated_target_ok,
        zero_phase,
        zero_weight,
        no_capacity,
        no_growth
    );

    assert_eq!(intact_total, 8);
    assert!(lesion_total <= 6, "necessary lesion must lose >=2/8 decisions");
    assert!(phase_total <= 6, "pi phase shift must lose >=2/8 decisions");
    assert_eq!(restored_total, 8);
    assert_eq!(unrelated_target_ok, 2);
    assert!(zero_phase <= 4);
    assert!(zero_weight <= 4);
    assert!(no_capacity <= 4);
    assert!(no_growth <= 4);
}

#[test]
fn g10_physical_selector_has_no_dedicated_readout_or_search_fallback() {
    let source = include_str!("../src/phase_concept.rs");
    let start = source
        .find("pub fn choose_phase_native_concept_action")
        .expect("physical G10 selector");
    let selector = &source[start..];

    for forbidden in [
        "choose_composite_action",
        "choose_atom_only_action",
        "EvoImaginationPlanner",
        "VecDeque",
        "BinaryHeap",
        "correct_action",
        "AtomKind",
    ] {
        assert!(
            !selector.contains(forbidden),
            "physical G10 selector contains forbidden fallback/token {forbidden}"
        );
    }

    for required in [
        "active_atom_ids",
        "conductance",
        "child_synapses",
        "motor_synapses",
        "self.synapses",
        "motor_potentials",
    ] {
        assert!(
            selector.contains(required),
            "physical G10 selector missing physical dependency {required}"
        );
    }
}
