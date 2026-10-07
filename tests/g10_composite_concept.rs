use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase};

#[derive(Clone, Copy, Debug)]
enum AtomKind {
    A,
    B,
    C,
    D,
}

fn delta(kind: AtomKind) -> (usize, usize) {
    match kind {
        AtomKind::A => (1, 0),
        AtomKind::B => (0, 1),
        AtomKind::C => (1, 1),
        AtomKind::D => (2, 0),
    }
}

fn scene(
    first: AtomKind,
    second: AtomKind,
    first_origin: (usize, usize),
    second_origin: (usize, usize),
) -> Vec<f32> {
    let mut raster = vec![0.0f32; 12 * 12];
    for (kind, (x, y)) in [(first, first_origin), (second, second_origin)] {
        let (dx, dy) = delta(kind);
        assert!(x + dx < 12 && y + dy < 12);
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn pair_cases() -> [(AtomKind, AtomKind, bool); 4] {
    [
        (AtomKind::A, AtomKind::B, false),
        (AtomKind::C, AtomKind::D, false),
        (AtomKind::A, AtomKind::C, true),
        (AtomKind::B, AtomKind::D, true),
    ]
}

fn correct_action(class_one: bool, swap: bool) -> usize {
    usize::from(class_one ^ swap)
}

fn carrier(composite_formation: bool) -> EvoPhase {
    let cfg = EvoConfig {
        sensory_cells: 144,
        motor_cells: 2,
        dormant_cells: 64,
        hdc_dim: 192,
        ..EvoConfig::default()
    };
    let mut evo = EvoPhase::new(cfg);
    let mut concept = ConceptConfig::for_raster(12, 12, 2, 192);
    concept.local_radius = 2;
    concept.atom_match_threshold = 0.97;
    concept.min_action_support = 4;
    concept.min_composite_support = 8;
    concept.child_predictiveness_ceiling = 0.20;
    concept.composite_promotion_threshold = 0.60;
    concept.atom_formation_enabled = true;
    concept.composite_formation_enabled = composite_formation;
    concept.learning_enabled = true;
    concept.readout_enabled = false;
    evo.enable_concept_memory(concept);
    evo
}

fn train(swap: bool) -> (EvoPhase, EvoPhase) {
    let mut genuine = carrier(true);
    let mut no_construction = carrier(false);

    assert_eq!(genuine.composite_concepts().len(), 0);
    assert_eq!(no_construction.composite_concepts().len(), 0);

    let bindings = [
        ((1usize, 1usize), (8usize, 8usize)),
        ((2, 1), (7, 7)),
        ((1, 2), (8, 6)),
        ((2, 2), (7, 8)),
    ];

    for (first_origin, second_origin) in bindings {
        for (first, second, class_one) in pair_cases() {
            let raster = scene(first, second, first_origin, second_origin);
            for action in 0..2 {
                let need = action == correct_action(class_one, swap);
                genuine.observe_concept_factual(&raster, action, need);
                no_construction.observe_concept_factual(&raster, action, need);
            }
        }
    }

    (genuine, no_construction)
}

fn assert_same_atom_inventory(a: &EvoPhase, b: &EvoPhase) {
    assert_eq!(a.concept_atoms().len(), b.concept_atoms().len());
    assert_eq!(a.concept_atoms().len(), 4, "four lower-level relation atoms expected");

    for (left, right) in a.concept_atoms().iter().zip(b.concept_atoms()) {
        assert_eq!(left.id, right.id);
        assert_eq!(left.support, right.support);
        assert!(
            left.prototype.similarity(&right.prototype) > 0.999_999,
            "matched arms must acquire the same atom prototypes"
        );
    }
}

fn qualify_swap(swap: bool) -> (usize, usize, usize) {
    let (mut genuine, mut no_construction) = train(swap);
    assert_same_atom_inventory(&genuine, &no_construction);

    assert_eq!(
        no_construction.composite_concepts().len(),
        0,
        "NO_CONSTRUCTION must retain atoms without promoted composites"
    );
    assert_eq!(
        genuine.composite_concepts().len(),
        4,
        "all four useful acquired atom pairs should promote"
    );

    for atom in genuine.concept_atoms() {
        for action in 0..2 {
            let evidence = genuine
                .concept_atom_action_evidence(atom.id, action)
                .expect("atom evidence");
            assert!(
                evidence.abs() <= 0.20 + 1.0e-6,
                "individual child atom became predictive: atom={} action={} evidence={}",
                atom.id,
                action,
                evidence
            );
        }
    }

    let atom_ids = genuine
        .concept_atoms()
        .iter()
        .map(|atom| atom.id)
        .collect::<std::collections::BTreeSet<_>>();
    for concept in genuine.composite_concepts() {
        assert!(concept.child_ids.iter().all(|id| atom_ids.contains(id)));
        let strongest = (0..2)
            .map(|action| {
                genuine
                    .composite_concept_action_evidence(concept.id, action)
                    .unwrap()
                    .abs()
            })
            .fold(0.0f32, f32::max);
        assert!(
            strongest >= 0.60,
            "promoted composite lacks frozen evidence threshold: {}",
            strongest
        );
    }

    genuine.set_concept_learning_enabled(false);
    no_construction.set_concept_learning_enabled(false);
    genuine.set_concept_readout_enabled(true);
    no_construction.set_concept_readout_enabled(false);

    let mut no_readout = genuine.clone();
    no_readout.set_concept_readout_enabled(false);

    let heldout = [
        ((AtomKind::A, AtomKind::B, false), ((3usize, 1usize), (6usize, 8usize))),
        ((AtomKind::C, AtomKind::D, false), ((1, 4), (8, 2))),
        ((AtomKind::A, AtomKind::C, true), ((3, 3), (7, 7))),
        ((AtomKind::B, AtomKind::D, true), ((2, 4), (7, 1))),
    ];

    let mut genuine_ok = 0usize;
    let mut no_construction_ok = 0usize;
    let mut no_readout_ok = 0usize;

    for ((first, second, class_one), (first_origin, second_origin)) in heldout {
        let raster = scene(first, second, first_origin, second_origin);
        let expected = correct_action(class_one, swap);

        let full = genuine
            .choose_composite_concept_action(&raster)
            .expect("acquired composite must read out on held-out binding");
        genuine_ok += usize::from(full == expected);

        let atom_only = no_construction
            .choose_atom_only_concept_action(&raster)
            .expect("atom inventory remains available");
        no_construction_ok += usize::from(atom_only == expected);

        assert_eq!(
            no_readout.choose_composite_concept_action(&raster),
            None,
            "NO_READOUT must suppress composite contribution"
        );
        let no_readout_atom = no_readout
            .choose_atom_only_concept_action(&raster)
            .expect("NO_READOUT still retains child atoms");
        no_readout_ok += usize::from(no_readout_atom == expected);
    }

    (genuine_ok, no_construction_ok, no_readout_ok)
}

#[test]
fn g10_constructs_new_composite_concepts_when_children_are_not_predictive() {
    let normal = qualify_swap(false);
    let swapped = qualify_swap(true);

    let full = normal.0 + swapped.0;
    let no_construction = normal.1 + swapped.1;
    let no_readout = normal.2 + swapped.2;

    println!(
        "G10_MECHANISM full={}/8 no_construction={}/8 no_readout={}/8 normal={:?} swapped={:?}",
        full, no_construction, no_readout, normal, swapped
    );

    assert_eq!(full, 8, "GENUINE must solve every held-out combination");
    assert!(
        no_construction <= 4,
        "atoms alone must not solve the XOR-like composite task"
    );
    assert!(
        no_readout <= 4,
        "removing acquired composite readout must lose the advantage"
    );
}
