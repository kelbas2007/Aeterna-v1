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


#[derive(Clone, Copy, Debug)]
struct FreshBinding {
    first: (usize, usize),
    second: (usize, usize),
}

#[derive(Clone, Debug)]
struct FreshG10Block {
    offsets: [(usize, usize); 4],
    swap: bool,
    heldout: [FreshBinding; 8],
}

struct FreshG10Rng(u64);

impl FreshG10Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0xC011_CE70_A37E_0010)
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

fn fresh_relation_pool() -> [(usize, usize); 8] {
    [(1,0), (0,1), (1,1), (2,0), (0,2), (2,1), (1,2), (2,2)]
}

fn fresh_training_bindings() -> [FreshBinding; 4] {
    [
        FreshBinding { first: (1,1), second: (8,8) },
        FreshBinding { first: (2,1), second: (7,7) },
        FreshBinding { first: (1,3), second: (8,5) },
        FreshBinding { first: (3,1), second: (6,7) },
    ]
}

fn fresh_heldout_bank() -> [FreshBinding; 8] {
    [
        FreshBinding { first: (1,5), second: (8,2) },
        FreshBinding { first: (2,5), second: (7,2) },
        FreshBinding { first: (3,4), second: (8,1) },
        FreshBinding { first: (1,4), second: (6,1) },
        FreshBinding { first: (2,2), second: (8,6) },
        FreshBinding { first: (2,4), second: (8,1) },
        FreshBinding { first: (1,5), second: (7,1) },
        FreshBinding { first: (2,3), second: (6,8) },
    ]
}

fn shuffle<T>(rng: &mut FreshG10Rng, values: &mut [T]) {
    for i in (1..values.len()).rev() {
        let j = rng.range(i + 1);
        values.swap(i, j);
    }
}

fn fresh_scene(
    first_delta: (usize, usize),
    second_delta: (usize, usize),
    binding: FreshBinding,
) -> Vec<f32> {
    let mut raster = vec![0.0f32; 144];
    for (delta, origin) in [(first_delta, binding.first), (second_delta, binding.second)] {
        let (x, y) = origin;
        let (dx, dy) = delta;
        assert!(x + dx < 12 && y + dy < 12);
        raster[y * 12 + x] = 1.0;
        raster[(y + dy) * 12 + x + dx] = 1.0;
    }
    raster
}

fn fresh_pairs(offsets: [(usize, usize); 4]) -> [((usize,usize),(usize,usize),bool);4] {
    [
        (offsets[0], offsets[1], false),
        (offsets[2], offsets[3], false),
        (offsets[0], offsets[2], true),
        (offsets[1], offsets[3], true),
    ]
}

fn fresh_correct_action(class_one: bool, swap: bool) -> usize {
    usize::from(class_one ^ swap)
}

fn fnv_mix_g10(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn fresh_g10_blocks(authority: u64) -> (Vec<FreshG10Block>, u64) {
    let mut blocks = Vec::new();
    let mut digest = 14_695_981_039_346_656_037_u64;
    let authority_flip = authority & 1 == 1;

    for sub in 0..10u64 {
        let derived = authority
            ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ 0xA37E_10C0_5EED_771B;
        let mut rng = FreshG10Rng::new(derived);

        let mut pool = fresh_relation_pool().to_vec();
        shuffle(&mut rng, &mut pool);
        let offsets = [pool[0], pool[1], pool[2], pool[3]];

        let mut heldout = fresh_heldout_bank();
        shuffle(&mut rng, &mut heldout);

        // Authority-derived global flip + sub-seed parity guarantees that both
        // opaque motor mappings are represented in every sealed 10-seed pack.
        let swap = authority_flip ^ (sub % 2 == 1);

        digest = fnv_mix_g10(digest, sub);
        digest = fnv_mix_g10(digest, swap as u64);
        for (dx, dy) in offsets {
            digest = fnv_mix_g10(digest, dx as u64);
            digest = fnv_mix_g10(digest, dy as u64);
        }
        for binding in heldout {
            for value in [
                binding.first.0,
                binding.first.1,
                binding.second.0,
                binding.second.1,
            ] {
                digest = fnv_mix_g10(digest, value as u64);
            }
        }

        blocks.push(FreshG10Block { offsets, swap, heldout });
    }

    (blocks, digest)
}

fn train_fresh_g10(block: &FreshG10Block, construction: bool) -> EvoPhase {
    let mut evo = carrier(construction);
    for binding in fresh_training_bindings() {
        for (first, second, class_one) in fresh_pairs(block.offsets) {
            let raster = fresh_scene(first, second, binding);
            for action in 0..2 {
                let need = action == fresh_correct_action(class_one, block.swap);
                evo.observe_concept_factual(&raster, action, need);
            }
        }
    }
    evo
}

fn g10_wilson95(success: usize, n: usize) -> (f64, f64) {
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
fn g10_fresh_composite_concept_pack() {
    let authority: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse()
        .expect("fresh G10 seed must be u64");
    let source_sha = std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha = std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let (blocks, digest) = fresh_g10_blocks(authority);
    println!(
        "FRESH_G10_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha, spec_sha, authority, digest
    );
    for (sub, block) in blocks.iter().enumerate() {
        println!("FRESH_G10_BLOCK sub={} {:?}", sub, block);
    }

    let mut full_success = 0usize;
    let mut no_construction_success = 0usize;
    let mut no_readout_success = 0usize;
    let mut per_seed = Vec::new();
    let mut inventory_mismatches = 0usize;
    let mut child_reference_violations = 0usize;
    let mut max_child_evidence = 0.0f32;
    let mut min_composite_evidence = 1.0f32;
    let mut atom_counts = Vec::new();
    let mut composite_counts = Vec::new();
    let mut swap_true = 0usize;
    let mut swap_false = 0usize;

    for (sub, block) in blocks.iter().enumerate() {
        swap_true += usize::from(block.swap);
        swap_false += usize::from(!block.swap);

        let mut genuine = train_fresh_g10(block, true);
        let mut no_construction = train_fresh_g10(block, false);

        atom_counts.push((genuine.concept_atoms().len(), no_construction.concept_atoms().len()));
        composite_counts.push((
            genuine.composite_concepts().len(),
            no_construction.composite_concepts().len(),
        ));

        if genuine.concept_atoms().len() != no_construction.concept_atoms().len() {
            inventory_mismatches += 1;
        }
        for (left, right) in genuine.concept_atoms().iter().zip(no_construction.concept_atoms()) {
            if left.id != right.id
                || left.support != right.support
                || left.prototype.similarity(&right.prototype) <= 0.999_999
            {
                inventory_mismatches += 1;
            }
        }

        let atom_ids = genuine
            .concept_atoms()
            .iter()
            .map(|atom| atom.id)
            .collect::<std::collections::BTreeSet<_>>();

        for atom in genuine.concept_atoms() {
            for action in 0..2 {
                max_child_evidence = max_child_evidence.max(
                    genuine
                        .concept_atom_action_evidence(atom.id, action)
                        .unwrap()
                        .abs(),
                );
            }
        }

        for concept in genuine.composite_concepts() {
            if !concept.child_ids.iter().all(|id| atom_ids.contains(id)) {
                child_reference_violations += 1;
            }
            let winning = (0..2)
                .map(|action| {
                    genuine
                        .composite_concept_action_evidence(concept.id, action)
                        .unwrap()
                })
                .fold(f32::NEG_INFINITY, f32::max);
            min_composite_evidence = min_composite_evidence.min(winning);
        }

        assert_eq!(
            genuine.concept_atoms().len(),
            4,
            "sub-seed {sub} must acquire exactly four selected atoms"
        );
        assert_eq!(
            no_construction.concept_atoms().len(),
            4,
            "matched sub-seed {sub} must acquire the same four atoms"
        );
        assert_eq!(
            genuine.composite_concepts().len(),
            4,
            "sub-seed {sub} must promote all four useful composites"
        );
        assert_eq!(
            no_construction.composite_concepts().len(),
            0,
            "NO_CONSTRUCTION must promote zero composites"
        );

        genuine.set_concept_learning_enabled(false);
        no_construction.set_concept_learning_enabled(false);
        genuine.set_concept_readout_enabled(true);
        no_construction.set_concept_readout_enabled(false);
        let mut no_readout = genuine.clone();
        no_readout.set_concept_readout_enabled(false);

        let pairs = fresh_pairs(block.offsets);
        let mut sub_ok = 0usize;
        for pair_index in 0..4 {
            let (first, second, class_one) = pairs[pair_index];
            for layout_index in 0..2 {
                let binding = block.heldout[pair_index * 2 + layout_index];
                let raster = fresh_scene(first, second, binding);
                let expected = fresh_correct_action(class_one, block.swap);

                let full = genuine
                    .choose_composite_concept_action(&raster)
                    .expect("fresh composite readout");
                full_success += usize::from(full == expected);
                sub_ok += usize::from(full == expected);

                let atom_only = no_construction
                    .choose_atom_only_concept_action(&raster)
                    .expect("fresh atom-only control");
                no_construction_success += usize::from(atom_only == expected);

                assert_eq!(no_readout.choose_composite_concept_action(&raster), None);
                let readout_off = no_readout
                    .choose_atom_only_concept_action(&raster)
                    .expect("fresh no-readout atom fallback");
                no_readout_success += usize::from(readout_off == expected);
            }
        }
        per_seed.push(sub_ok);

        println!(
            "FRESH_G10_SUB sub={} full={}/8 atoms={} composites={} swap={} max_child_so_far={:.4} min_composite_so_far={:.4}",
            sub,
            sub_ok,
            genuine.concept_atoms().len(),
            genuine.composite_concepts().len(),
            block.swap,
            max_child_evidence,
            min_composite_evidence
        );
    }

    let n = 80usize;
    let (lo, hi) = g10_wilson95(full_success, n);
    let full_rate = full_success as f64 / n as f64;
    let no_construction_rate = no_construction_success as f64 / n as f64;
    let no_readout_rate = no_readout_success as f64 / n as f64;

    println!(
        "FRESH_G10_RESULT N={} full={}/{} wilson95=[{:.6},{:.6}] per_seed={:?} no_construction={}/{} no_readout={}/{} inventory_mismatches={} child_ref_violations={} max_child_evidence={:.6} min_composite_evidence={:.6} swap_true={} swap_false={} atom_counts={:?} composite_counts={:?}",
        n,
        full_success,
        n,
        lo,
        hi,
        per_seed,
        no_construction_success,
        n,
        no_readout_success,
        n,
        inventory_mismatches,
        child_reference_violations,
        max_child_evidence,
        min_composite_evidence,
        swap_true,
        swap_false,
        atom_counts,
        composite_counts
    );

    assert!(full_success >= 76, "FULL_COMPOSITE must achieve >=76/80");
    assert!(lo >= 0.87, "Wilson 95% lower bound must be >=0.87");
    assert!(
        per_seed.iter().all(|score| *score >= 6),
        "every fresh sub-seed must achieve >=6/8"
    );
    assert!(no_construction_success <= 48);
    assert!(no_readout_success <= 48);
    assert!(
        full_rate - no_construction_rate >= 0.30,
        "FULL must beat NO_CONSTRUCTION by >=0.30"
    );
    assert!(
        full_rate - no_readout_rate >= 0.30,
        "FULL must beat NO_READOUT by >=0.30"
    );
    assert_eq!(inventory_mismatches, 0);
    assert_eq!(child_reference_violations, 0);
    assert!(max_child_evidence <= 0.20 + 1.0e-6);
    assert!(min_composite_evidence >= 0.60 - 1.0e-6);
    assert_eq!(swap_true, 5);
    assert_eq!(swap_false, 5);
}
