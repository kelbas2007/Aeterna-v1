mod g7_support;

use g7_support::{
    build_mature, fnv_mix, generate_worlds, perm4, prepare_worlds, score_world, wilson95, Arm,
    Rng, World, DISTRACTOR, DROPOUT, ROTATE, SCALE,
};

fn mean_std(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let d = value - mean;
            d * d
        })
        .sum::<f64>()
        / values.len() as f64;
    (mean, variance.sqrt())
}

#[test]
fn g7_preflight_composes_perception_exploration_revision_and_hierarchy() {
    let mut mature = build_mature(0xA7E7_0000_0000_0001);

    let worlds = (0..6usize)
        .map(|i| World {
            family: 400 + i,
            law: i % 3,
            informative: mature.perm[(i + 1) % 4],
            origin: if i % 2 == 0 { (5, 5) } else { (4, 5) },
            nuisance: 0,
            sensor_seed: 0xBEEF_0000 + i as u64,
        })
        .collect::<Vec<_>>();

    prepare_worlds(&mut mature, &worlds);

    let revised = mature
        .evo
        .macros()
        .iter()
        .find(|m| m.id == mature.revised_child_id)
        .expect("revised child must still exist");
    assert!(
        revised.revision > mature.pre_revision_counter,
        "G7 must preserve same child ID while increasing its revision"
    );

    let mut full_success = 0usize;
    let mut zero_success = 0usize;
    let mut no_hierarchy_success = 0usize;
    let mut unrevised_success = 0usize;
    let mut full_probes = 0usize;
    let mut zero_probes = 0usize;
    let mut full_candidates = 0usize;
    let mut no_hierarchy_candidates = 0usize;

    for world in worlds {
        let full = score_world(&mature, world, Arm::Full);
        let zero = score_world(&mature, world, Arm::ZeroExploration);
        let no_hierarchy = score_world(&mature, world, Arm::NoHierarchy);
        let unrevised = score_world(&mature, world, Arm::UnrevisedChild);

        println!(
            "G7_PREFLIGHT_WORLD family={} law={} informative={} full={:?} zero={:?} no_hierarchy={:?} unrevised={:?}",
            world.family, world.law, world.informative, full, zero, no_hierarchy, unrevised
        );

        full_success += usize::from(full.success);
        zero_success += usize::from(zero.success);
        no_hierarchy_success += usize::from(no_hierarchy.success);
        unrevised_success += usize::from(unrevised.success);
        full_probes += full.probes;
        zero_probes += zero.probes;
        full_candidates += full.candidate_evals;
        no_hierarchy_candidates += no_hierarchy.candidate_evals;

        println!(
            "G7_PREFLIGHT_WORLD world={:?} full={:?} zero={:?} no_hierarchy={:?} unrevised={:?}",
            world, full, zero, no_hierarchy, unrevised
        );

        assert!(
            full.success,
            "clean held-out preflight world must traverse the complete acquired chain"
        );
    }

    assert_eq!(full_success, 6);
    assert_eq!(zero_success, 6, "zero strategy may recover by extra physical probes");
    assert_eq!(
        no_hierarchy_success, 6,
        "generic sequence search may recover by extra candidate evaluations"
    );
    assert!(
        full_probes < zero_probes,
        "learned exploration must reduce clean held-out physical probes"
    );
    assert!(
        full_candidates < no_hierarchy_candidates,
        "acquired hierarchy must reduce child-sequence candidate evaluations"
    );
    assert!(
        unrevised_success < full_success,
        "the pre-revision child control must fail changed-law worlds"
    );

    println!(
        "G7_PREFLIGHT full={}/6 zero={}/6 no_hierarchy={}/6 unrevised={}/6 full_probes={} zero_probes={} full_candidates={} no_hierarchy_candidates={} tuition=[exploration:{} child:{} parent:{} revision:{}]",
        full_success,
        zero_success,
        no_hierarchy_success,
        unrevised_success,
        full_probes,
        zero_probes,
        full_candidates,
        no_hierarchy_candidates,
        mature.tuition_probes,
        mature.child_tuition_actions,
        mature.parent_tuition_actions,
        mature.revision_actions,
    );
}

#[test]
fn r1_robust_high_level_perception_survives_each_single_nuisance_end_to_end() {
    let mut mature = build_mature(0xA7E7_0000_0000_1001);
    let nuisances = [
        DISTRACTOR, DROPOUT, ROTATE, SCALE,
        DISTRACTOR, DROPOUT, ROTATE, SCALE,
    ];

    let worlds = nuisances
        .into_iter()
        .enumerate()
        .map(|(i, nuisance)| World {
            family: 600 + i,
            law: i % 3,
            informative: mature.perm[(i + 2) % 4],
            origin: if i % 2 == 0 { (4, 4) } else { (5, 4) },
            nuisance,
            sensor_seed: 0xC0DE_1000 + i as u64,
        })
        .collect::<Vec<_>>();

    prepare_worlds(&mut mature, &worlds);

    let mut success = 0usize;
    for world in worlds {
        let score = score_world(&mature, world, Arm::Full);
        println!("R1_NUISANCE_PREFLIGHT world={:?} score={:?}", world, score);
        success += usize::from(score.success);
        assert!(
            score.success,
            "robust high-level trace must preserve the complete learned chain under each single nuisance"
        );
    }

    assert_eq!(success, 8);
}

#[test]
fn r1_nonfresh_mixed_80_world_development_pack() {
    const DEV_SEED: u64 = 0xA7E7_5101_2026_1006;

    let mut total = 0usize;
    let mut full_ok = 0usize;
    let mut zero_ok = 0usize;
    let mut no_hierarchy_ok = 0usize;
    let mut unrevised_ok = 0usize;

    let mut mask_total = [0usize; 16];
    let mut mask_success = [0usize; 16];
    let mut matched_full_probes = 0usize;
    let mut matched_zero_probes = 0usize;
    let mut matched_probe_cases = 0usize;
    let mut matched_full_candidates = 0usize;
    let mut matched_no_hierarchy_candidates = 0usize;
    let mut matched_candidate_cases = 0usize;

    for sub in 0..10u64 {
        let maturity_seed = DEV_SEED
            ^ sub.wrapping_mul(0xD1B5_4A32_D192_ED03)
            ^ 0x771B_A37E_2C91_5F04;
        let mut perm_rng = Rng::new(maturity_seed);
        let perm = perm4(&mut perm_rng);
        let worlds = generate_worlds(DEV_SEED, sub, perm);

        let mut mature = build_mature(maturity_seed);
        assert_eq!(mature.perm, perm);
        prepare_worlds(&mut mature, &worlds);

        let mut sub_ok = 0usize;
        for world in worlds {
            let full = score_world(&mature, world, Arm::Full);
            let zero = score_world(&mature, world, Arm::ZeroExploration);
            let no_hierarchy = score_world(&mature, world, Arm::NoHierarchy);
            let unrevised = score_world(&mature, world, Arm::UnrevisedChild);

            total += 1;
            full_ok += usize::from(full.success);
            zero_ok += usize::from(zero.success);
            no_hierarchy_ok += usize::from(no_hierarchy.success);
            unrevised_ok += usize::from(unrevised.success);
            sub_ok += usize::from(full.success);

            let mask = world.nuisance as usize;
            mask_total[mask] += 1;
            mask_success[mask] += usize::from(full.success);

            if full.success && zero.success {
                matched_full_probes += full.probes;
                matched_zero_probes += zero.probes;
                matched_probe_cases += 1;
            }
            if full.success && no_hierarchy.success {
                matched_full_candidates += full.candidate_evals;
                matched_no_hierarchy_candidates += no_hierarchy.candidate_evals;
                matched_candidate_cases += 1;
            }
        }

        println!("R1_DEV_SUBSEED sub={} full_success={}/8", sub, sub_ok);
    }

    let full_rate = full_ok as f64 / total as f64;
    let matched_probe_full = matched_full_probes as f64 / matched_probe_cases.max(1) as f64;
    let matched_probe_zero = matched_zero_probes as f64 / matched_probe_cases.max(1) as f64;
    let matched_candidate_full =
        matched_full_candidates as f64 / matched_candidate_cases.max(1) as f64;
    let matched_candidate_no_hierarchy =
        matched_no_hierarchy_candidates as f64 / matched_candidate_cases.max(1) as f64;

    println!(
        "R1_DEV_RESULT seed={} N={} full={}/{} rate={:.4} zero={}/{} no_hierarchy={}/{} unrevised={}/{} masks_total={:?} masks_success={:?} matched_probe_full={:.4} matched_probe_zero={:.4} matched_candidates_full={:.4} matched_candidates_no_hierarchy={:.4}",
        DEV_SEED,
        total,
        full_ok,
        total,
        full_rate,
        zero_ok,
        total,
        no_hierarchy_ok,
        total,
        unrevised_ok,
        total,
        mask_total,
        mask_success,
        matched_probe_full,
        matched_probe_zero,
        matched_candidate_full,
        matched_candidate_no_hierarchy,
    );

    assert_eq!(total, 80);
    assert!(
        full_rate >= 0.80,
        "R1 development pack must clear the preregistered whole-chain 80% floor before any fresh retry"
    );

    for mask in [
        DISTRACTOR as usize,
        DROPOUT as usize,
        ROTATE as usize,
        SCALE as usize,
        (DISTRACTOR | ROTATE) as usize,
        (DROPOUT | SCALE) as usize,
    ] {
        assert!(mask_total[mask] > 0, "required development nuisance mask missing");
        assert!(
            mask_success[mask] > 0,
            "every single and combined nuisance mode must have at least one end-to-end success"
        );
    }

    assert!(
        matched_probe_cases > 0 && matched_probe_full < matched_probe_zero,
        "learned exploration advantage must remain visible on matched survivable development worlds"
    );
    assert!(
        matched_candidate_cases > 0
            && matched_candidate_full < matched_candidate_no_hierarchy,
        "hierarchy advantage must remain visible on matched survivable development worlds"
    );
}

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from CI"]
fn fresh_g7_whole_organism_pack() {
    let authority_seed: u64 = std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED is mandatory")
        .parse()
        .expect("fresh seed must be u64");
    let source_sha = std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_| "unknown".into());
    let spec_sha = std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_| "unknown".into());

    let mut digest = 14_695_981_039_346_656_037_u64;
    let mut packs = Vec::new();

    // Phase 1: generate and log the complete one-use pack before any scoring.
    for sub in 0..10u64 {
        let maturity_seed = authority_seed
            ^ sub.wrapping_mul(0xD1B5_4A32_D192_ED03)
            ^ 0x771B_A37E_2C91_5F04;
        let mut perm_rng = Rng::new(maturity_seed);
        let perm = perm4(&mut perm_rng);
        let worlds = generate_worlds(authority_seed, sub, perm);

        digest = fnv_mix(digest, sub);
        digest = fnv_mix(digest, maturity_seed);
        for action in perm {
            digest = fnv_mix(digest, action as u64);
        }
        for world in &worlds {
            digest = fnv_mix(digest, world.family as u64);
            digest = fnv_mix(digest, world.law as u64);
            digest = fnv_mix(digest, world.informative as u64);
            digest = fnv_mix(digest, world.origin.0 as u64);
            digest = fnv_mix(digest, world.origin.1 as u64);
            digest = fnv_mix(digest, world.nuisance as u64);
            digest = fnv_mix(digest, world.sensor_seed);
        }

        println!(
            "G7_MANIFEST sub={} maturity_seed={} motor_perm={:?} worlds={:?}",
            sub, maturity_seed, perm, worlds
        );
        packs.push((sub, maturity_seed, perm, worlds));
    }

    println!(
        "G7_PACK source_sha={} spec_sha={} authority_seed={} pack_digest={:016x} N=80",
        source_sha, spec_sha, authority_seed, digest
    );

    // Phase 2: score only after the manifest/digest has been emitted.
    let mut total = 0usize;
    let mut full_ok = 0usize;
    let mut zero_ok = 0usize;
    let mut no_hierarchy_ok = 0usize;
    let mut unrevised_ok = 0usize;

    let mut full_probe_values = Vec::new();
    let mut zero_probe_values = Vec::new();
    let mut full_primitive_values = Vec::new();
    let mut no_hierarchy_primitive_values = Vec::new();

    let mut matched_probe_full = 0usize;
    let mut matched_probe_zero = 0usize;
    let mut matched_probe_cases = 0usize;

    let mut matched_candidate_full = 0usize;
    let mut matched_candidate_no_hierarchy = 0usize;
    let mut matched_candidate_cases = 0usize;

    let mut revision_worlds = 0usize;
    let mut full_revision_ok = 0usize;
    let mut unrevised_revision_ok = 0usize;

    let mut nuisance_worlds = [0usize; 4];
    let mut nuisance_success = [0usize; 4];
    let mut per_seed = Vec::new();

    let mut total_exploration_tuition = 0u64;
    let mut total_child_tuition = 0usize;
    let mut total_parent_tuition = 0usize;
    let mut total_revision_tuition = 0usize;

    for (sub, maturity_seed, perm, worlds) in packs {
        let mut mature = build_mature(maturity_seed);
        assert_eq!(
            mature.perm, perm,
            "the logged motor permutation must equal the organism acquisition permutation"
        );

        prepare_worlds(&mut mature, &worlds);

        let revised = mature
            .evo
            .macros()
            .iter()
            .find(|m| m.id == mature.revised_child_id)
            .expect("revised child must remain present");
        assert!(
            revised.revision > mature.pre_revision_counter,
            "same acquired child ID must survive revision before fresh scoring"
        );

        total_exploration_tuition += mature.tuition_probes as u64;
        total_child_tuition += mature.child_tuition_actions;
        total_parent_tuition += mature.parent_tuition_actions;
        total_revision_tuition += mature.revision_actions;

        let mut sub_ok = 0usize;

        for world in worlds {
            let full = score_world(&mature, world, Arm::Full);
            let zero = score_world(&mature, world, Arm::ZeroExploration);
            let no_hierarchy = score_world(&mature, world, Arm::NoHierarchy);
            let unrevised = score_world(&mature, world, Arm::UnrevisedChild);

            total += 1;
            full_ok += usize::from(full.success);
            zero_ok += usize::from(zero.success);
            no_hierarchy_ok += usize::from(no_hierarchy.success);
            unrevised_ok += usize::from(unrevised.success);
            sub_ok += usize::from(full.success);

            full_probe_values.push(full.probes as f64);
            zero_probe_values.push(zero.probes as f64);
            full_primitive_values.push(full.primitive_actions as f64);
            no_hierarchy_primitive_values.push(no_hierarchy.primitive_actions as f64);

            if full.success && zero.success {
                matched_probe_full += full.probes;
                matched_probe_zero += zero.probes;
                matched_probe_cases += 1;
            }

            if full.success && no_hierarchy.success {
                matched_candidate_full += full.candidate_evals;
                matched_candidate_no_hierarchy += no_hierarchy.candidate_evals;
                matched_candidate_cases += 1;
            }

            if world.law == 0 || world.law == 2 {
                revision_worlds += 1;
                full_revision_ok += usize::from(full.success);
                unrevised_revision_ok += usize::from(unrevised.success);
            }

            for (idx, flag) in [DISTRACTOR, DROPOUT, ROTATE, SCALE].into_iter().enumerate() {
                if world.nuisance & flag != 0 {
                    nuisance_worlds[idx] += 1;
                    nuisance_success[idx] += usize::from(full.success);
                }
            }
        }

        println!(
            "G7_SUBSEED sub={} full_success={}/8 child_ids={:?} revised_child_id={} learned_weights={:?}",
            sub,
            sub_ok,
            mature.children,
            mature.revised_child_id,
            mature.evo.exploration_weights()
        );
        per_seed.push(sub_ok);
    }

    let full_rate = full_ok as f64 / total as f64;
    let (lo, hi) = wilson95(full_ok, total);
    let (full_probe_mean, full_probe_std) = mean_std(&full_probe_values);
    let (zero_probe_mean, zero_probe_std) = mean_std(&zero_probe_values);
    let (full_primitive_mean, full_primitive_std) = mean_std(&full_primitive_values);
    let (no_hierarchy_primitive_mean, no_hierarchy_primitive_std) =
        mean_std(&no_hierarchy_primitive_values);

    let matched_probe_full_mean = if matched_probe_cases == 0 {
        f64::INFINITY
    } else {
        matched_probe_full as f64 / matched_probe_cases as f64
    };
    let matched_probe_zero_mean = if matched_probe_cases == 0 {
        f64::INFINITY
    } else {
        matched_probe_zero as f64 / matched_probe_cases as f64
    };
    let matched_candidate_full_mean = if matched_candidate_cases == 0 {
        f64::INFINITY
    } else {
        matched_candidate_full as f64 / matched_candidate_cases as f64
    };
    let matched_candidate_no_hierarchy_mean = if matched_candidate_cases == 0 {
        f64::INFINITY
    } else {
        matched_candidate_no_hierarchy as f64 / matched_candidate_cases as f64
    };

    println!(
        "G7_RESULT N={} full={}/{} rate={:.4} wilson95=[{:.4},{:.4}] zero={}/{} no_hierarchy={}/{} unrevised={}/{} per_seed={:?}",
        total,
        full_ok,
        total,
        full_rate,
        lo,
        hi,
        zero_ok,
        total,
        no_hierarchy_ok,
        total,
        unrevised_ok,
        total,
        per_seed
    );
    println!(
        "G7_COST probes_full={:.4}±{:.4} probes_zero={:.4}±{:.4} primitive_full={:.4}±{:.4} primitive_no_hierarchy={:.4}±{:.4} matched_probe_full={:.4} matched_probe_zero={:.4} matched_parent_candidates_full={:.4} matched_parent_candidates_no_hierarchy={:.4} random_order_probe_mean=2.5000",
        full_probe_mean,
        full_probe_std,
        zero_probe_mean,
        zero_probe_std,
        full_primitive_mean,
        full_primitive_std,
        no_hierarchy_primitive_mean,
        no_hierarchy_primitive_std,
        matched_probe_full_mean,
        matched_probe_zero_mean,
        matched_candidate_full_mean,
        matched_candidate_no_hierarchy_mean,
    );
    println!(
        "G7_TUITION exploration_probes={} child_primitive_actions={} parent_primitive_actions={} revision_primitive_actions={}",
        total_exploration_tuition,
        total_child_tuition,
        total_parent_tuition,
        total_revision_tuition
    );
    println!(
        "G7_NUISANCE distractor={}/{} dropout={}/{} rotation={}/{} scale={}/{} revision_required_full={}/{} revision_required_unrevised={}/{}",
        nuisance_success[0],
        nuisance_worlds[0],
        nuisance_success[1],
        nuisance_worlds[1],
        nuisance_success[2],
        nuisance_worlds[2],
        nuisance_success[3],
        nuisance_worlds[3],
        full_revision_ok,
        revision_worlds,
        unrevised_revision_ok,
        revision_worlds
    );

    assert_eq!(total, 80, "G7 requires exactly the preregistered 80-world pack");
    assert!(
        full_rate >= 0.80,
        "FULL_ORGANISM success must be >= 0.80; completed failure is a cognitive FAIL"
    );
    assert!(
        lo >= 0.70,
        "FULL_ORGANISM Wilson 95% lower bound must be >= 0.70"
    );

    let beats_zero = full_ok > zero_ok
        || (matched_probe_cases > 0 && matched_probe_full_mean < matched_probe_zero_mean);
    assert!(
        beats_zero,
        "FULL_ORGANISM must beat ZERO_EXPLORATION in success or matched physical probe cost"
    );

    let beats_no_hierarchy = full_ok > no_hierarchy_ok
        || (matched_candidate_cases > 0
            && matched_candidate_full_mean < matched_candidate_no_hierarchy_mean);
    assert!(
        beats_no_hierarchy,
        "FULL_ORGANISM must beat NO_HIERARCHY in success or matched candidate cost"
    );

    assert!(
        revision_worlds > 0 && full_revision_ok > unrevised_revision_ok,
        "revised child must causally outperform its pre-revision control on changed-law worlds"
    );

    for i in 0..4 {
        assert!(nuisance_worlds[i] >= 2, "every nuisance category must appear >=2 times");
        assert!(
            nuisance_success[i] > 0,
            "no nuisance category may have zero FULL_ORGANISM successes"
        );
    }
}
