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
        assert_eq!(
            intact.composite_concepts().len(),
            0,
            "physical G10 must not rely on dedicated pair/composite promotion state"
        );
        assert_eq!(intact.phase_native_concept_atom_count(), 4);
        assert_eq!(intact.phase_native_concept_circuits().len(), 4);
        assert_eq!(intact.phase_native_promoted_concept_count(), 4);
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
        ".composites()",
        "composite_action_evidence",
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


#[derive(Clone, Copy, Debug)]
struct PhysBinding {
    first: (usize, usize),
    second: (usize, usize),
}

#[derive(Clone, Debug)]
struct PhysFreshBlock {
    offsets: [(usize, usize); 4],
    swap: bool,
    heldout: [PhysBinding; 8],
}

struct PhysFreshRng(u64);

impl PhysFreshRng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x50A5_C011_CE70_F10D)
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

fn phys_pool() -> [(usize, usize); 8] {
    [(1,0), (0,1), (1,1), (2,0), (0,2), (2,1), (1,2), (2,2)]
}

fn phys_train_bindings() -> [PhysBinding; 4] {
    [
        PhysBinding { first: (1,1), second: (8,8) },
        PhysBinding { first: (2,1), second: (7,7) },
        PhysBinding { first: (1,3), second: (8,5) },
        PhysBinding { first: (3,1), second: (6,7) },
    ]
}

fn phys_heldout_bank() -> [PhysBinding; 8] {
    [
        PhysBinding { first: (1,5), second: (8,2) },
        PhysBinding { first: (2,5), second: (7,2) },
        PhysBinding { first: (3,4), second: (8,1) },
        PhysBinding { first: (1,4), second: (6,1) },
        PhysBinding { first: (2,2), second: (8,6) },
        PhysBinding { first: (2,4), second: (8,1) },
        PhysBinding { first: (1,5), second: (7,1) },
        PhysBinding { first: (2,3), second: (6,8) },
    ]
}

fn phys_shuffle<T>(rng: &mut PhysFreshRng, values: &mut [T]) {
    for i in (1..values.len()).rev() {
        let j = rng.range(i + 1);
        values.swap(i, j);
    }
}

fn phys_scene(
    first_delta: (usize, usize),
    second_delta: (usize, usize),
    binding: PhysBinding,
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

fn phys_pairs(offsets: [(usize, usize); 4]) -> [((usize,usize),(usize,usize),bool);4] {
    [
        (offsets[0], offsets[1], false),
        (offsets[2], offsets[3], false),
        (offsets[0], offsets[2], true),
        (offsets[1], offsets[3], true),
    ]
}

fn phys_expected(class_one: bool, swap: bool) -> usize {
    usize::from(class_one ^ swap)
}

fn phys_fnv(mut hash: u64, value: u64) -> u64 {
    const PRIME: u64 = 1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn phys_blocks(authority: u64) -> (Vec<PhysFreshBlock>, u64) {
    let mut blocks = Vec::new();
    let mut digest = 14_695_981_039_346_656_037_u64;
    let authority_flip = authority & 1 == 1;

    for sub in 0..10u64 {
        let derived = authority
            ^ sub.wrapping_mul(0xD1B5_4A32_D192_ED03)
            ^ 0xF10D_C011_A37E_0010;
        let mut rng = PhysFreshRng::new(derived);
        let mut pool = phys_pool().to_vec();
        phys_shuffle(&mut rng, &mut pool);
        let offsets = [pool[0], pool[1], pool[2], pool[3]];
        let mut heldout = phys_heldout_bank();
        phys_shuffle(&mut rng, &mut heldout);
        let swap = authority_flip ^ (sub % 2 == 1);

        digest = phys_fnv(digest, sub);
        digest = phys_fnv(digest, swap as u64);
        for (dx, dy) in offsets {
            digest = phys_fnv(digest, dx as u64);
            digest = phys_fnv(digest, dy as u64);
        }
        for binding in heldout {
            for value in [
                binding.first.0,
                binding.first.1,
                binding.second.0,
                binding.second.1,
            ] {
                digest = phys_fnv(digest, value as u64);
            }
        }
        blocks.push(PhysFreshBlock { offsets, swap, heldout });
    }
    (blocks, digest)
}

fn phys_carrier(mode: &str) -> EvoPhase {
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

fn train_phys_block(block: &PhysFreshBlock, mode: &str) -> EvoPhase {
    let mut evo = phys_carrier(mode);
    for binding in phys_train_bindings() {
        for (first, second, class_one) in phys_pairs(block.offsets) {
            let raster = phys_scene(first, second, binding);
            for action in 0..2 {
                let need = action == phys_expected(class_one, block.swap);
                let _ = evo.observe_phase_native_concept_factual(&raster, action, need);
            }
        }
    }
    evo.set_concept_learning_enabled(false);
    evo.set_planning_learning_enabled(false);
    evo.set_concept_readout_enabled(false);
    evo
}

fn score_phys_block(evo: &EvoPhase, block: &PhysFreshBlock) -> usize {
    let pairs = phys_pairs(block.offsets);
    let mut score = 0usize;
    for pair_index in 0..4 {
        let (first, second, class_one) = pairs[pair_index];
        for layout_index in 0..2 {
            let raster = phys_scene(
                first,
                second,
                block.heldout[pair_index * 2 + layout_index],
            );
            score += usize::from(
                evo.choose_phase_native_concept_action(&raster)
                    == Some(phys_expected(class_one, block.swap)),
            );
        }
    }
    score
}

fn phys_circuit_for(
    evo: &EvoPhase,
    first: (usize, usize),
    second: (usize, usize),
    binding: PhysBinding,
) -> aeterna_v1::carrier::PhaseConceptCircuitInfo {
    let raster = phys_scene(first, second, binding);
    let mut ids = evo.active_concept_atom_ids(&raster);
    ids.sort_unstable();
    assert_eq!(ids.len(), 2);
    let children = [ids[0], ids[1]];
    evo.phase_native_concept_circuits()
        .iter()
        .find(|circuit| circuit.child_ids == children && circuit.promoted)
        .expect("fresh physical promoted pair circuit")
        .clone()
}

fn phys_wilson95(success: usize, n: usize) -> (f64, f64) {
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
fn g10_fresh_phase_native_concept_pack() {
    let authority: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse()
        .expect("fresh physical G10 seed must be u64");
    let source_sha = std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha = std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let (blocks, digest) = phys_blocks(authority);
    println!(
        "FRESH_G10_PHYS_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha, spec_sha, authority, digest
    );
    for (sub, block) in blocks.iter().enumerate() {
        println!("FRESH_G10_PHYS_BLOCK sub={} {:?}", sub, block);
    }

    let mut full = 0usize;
    let mut zero_phase = 0usize;
    let mut zero_weight = 0usize;
    let mut no_capacity = 0usize;
    let mut no_growth = 0usize;
    let mut per_seed = Vec::new();
    let mut lesion_ok = 0usize;
    let mut phase_ok = 0usize;
    let mut restored_ok = 0usize;
    let mut unrelated_ok = 0usize;
    let mut table_composite_violations = 0usize;
    let mut physical_count_violations = 0usize;
    let mut swap_true = 0usize;
    let mut swap_false = 0usize;
    let mut synapse_counts = Vec::new();

    for (sub, block) in blocks.iter().enumerate() {
        swap_true += usize::from(block.swap);
        swap_false += usize::from(!block.swap);

        let intact = train_phys_block(block, "full");
        let sub_score = score_phys_block(&intact, block);
        full += sub_score;
        per_seed.push(sub_score);

        if !intact.composite_concepts().is_empty() {
            table_composite_violations += 1;
        }
        if intact.phase_native_concept_atom_count() != 4
            || intact.phase_native_promoted_concept_count() != 4
        {
            physical_count_violations += 1;
        }
        synapse_counts.push(intact.phase_native_concept_synapses().len());

        zero_phase += score_phys_block(&train_phys_block(block, "zero_phase"), block);
        zero_weight += score_phys_block(&train_phys_block(block, "zero_weight"), block);
        no_capacity += score_phys_block(&train_phys_block(block, "no_capacity"), block);
        no_growth += score_phys_block(&train_phys_block(block, "no_growth"), block);

        let pairs = phys_pairs(block.offsets);
        let (first, second, class_one) = pairs[0];
        let target_expected = phys_expected(class_one, block.swap);
        let circuit = phys_circuit_for(
            &intact,
            first,
            second,
            block.heldout[0],
        );

        let mut lesioned = intact.clone();
        let saved = lesioned
            .perturb_phase_native_synapse_for_control(
                circuit.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh physical lesion synapse");
        for layout_index in 0..2 {
            let raster = phys_scene(
                first,
                second,
                block.heldout[layout_index],
            );
            lesion_ok += usize::from(
                lesioned.choose_phase_native_concept_action(&raster)
                    == Some(target_expected),
            );
        }

        lesioned.restore_phase_native_synapse_for_control(
            circuit.child_synapses[0],
            saved.clone(),
        );
        for layout_index in 0..2 {
            let raster = phys_scene(
                first,
                second,
                block.heldout[layout_index],
            );
            restored_ok += usize::from(
                lesioned.choose_phase_native_concept_action(&raster)
                    == Some(target_expected),
            );
        }

        let mut phase_shifted = intact.clone();
        phase_shifted
            .perturb_phase_native_synapse_for_control(
                circuit.child_synapses[0],
                1.0,
                std::f32::consts::PI,
            )
            .expect("fresh physical phase intervention");
        for layout_index in 0..2 {
            let raster = phys_scene(
                first,
                second,
                block.heldout[layout_index],
            );
            phase_ok += usize::from(
                phase_shifted.choose_phase_native_concept_action(&raster)
                    == Some(target_expected),
            );
        }

        let (u_first, u_second, _) = pairs[1];
        let unrelated_circuit = phys_circuit_for(
            &intact,
            u_first,
            u_second,
            block.heldout[2],
        );
        let mut unrelated = intact.clone();
        unrelated
            .perturb_phase_native_synapse_for_control(
                unrelated_circuit.child_synapses[0],
                0.0,
                0.0,
            )
            .expect("fresh unrelated physical lesion");
        let target = phys_scene(first, second, block.heldout[0]);
        unrelated_ok += usize::from(
            unrelated.choose_phase_native_concept_action(&target)
                == Some(target_expected),
        );

        println!(
            "FRESH_G10_PHYS_SUB sub={} full={}/8 table_composites={} physical_atoms={} physical_promoted={} physical_synapses={} swap={}",
            sub,
            sub_score,
            intact.composite_concepts().len(),
            intact.phase_native_concept_atom_count(),
            intact.phase_native_promoted_concept_count(),
            intact.phase_native_concept_synapses().len(),
            block.swap
        );
    }

    let (lo, hi) = phys_wilson95(full, 80);
    println!(
        "FRESH_G10_PHYS_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} zero_phase={}/80 zero_weight={}/80 no_capacity={}/80 no_growth={}/80 lesion={}/20 phase_shift={}/20 restored={}/20 unrelated={}/10 table_violations={} physical_count_violations={} swap_true={} swap_false={} synapse_counts={:?}",
        full,
        lo,
        hi,
        per_seed,
        zero_phase,
        zero_weight,
        no_capacity,
        no_growth,
        lesion_ok,
        phase_ok,
        restored_ok,
        unrelated_ok,
        table_composite_violations,
        physical_count_violations,
        swap_true,
        swap_false,
        synapse_counts
    );

    assert!(full >= 76);
    assert!(lo >= 0.87);
    assert!(per_seed.iter().all(|score| *score >= 6));
    assert_eq!(table_composite_violations, 0);
    assert_eq!(physical_count_violations, 0);
    assert!(zero_phase <= 48);
    assert!(zero_weight <= 48);
    assert!(no_capacity <= 20);
    assert!(no_growth <= 20);
    assert!(lesion_ok <= 4);
    assert!(phase_ok <= 4);
    assert!(restored_ok >= 19);
    assert!(unrelated_ok >= 9);
    assert_eq!(swap_true, 5);
    assert_eq!(swap_false, 5);
}
