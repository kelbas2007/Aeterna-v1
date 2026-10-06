mod g7_support;

use aeterna_v1::{EvoConfig, EvoPhase, MacroConfig, RasterFieldConfig};
use g7_support::{render_relation, DISTRACTOR, DROPOUT, ROTATE, SCALE};

#[derive(Clone, Copy, Debug)]
enum Arm {
    Robust,
    DirectedOnly,
    NoRobustFormation,
    NoRobustReadout,
}

fn organism(robust: bool) -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 4;
    cfg.dormant_cells = 96;
    cfg.hdc_dim = 192;

    let mut evo = EvoPhase::new(cfg);
    let mut raster = RasterFieldConfig::for_raster(12, 12, 4);
    raster.learning_enabled = false;
    raster.readout_enabled = false;
    evo.attach_raster_field(raster);
    evo.set_robust_high_level_perception(robust);

    let mut macro_cfg = MacroConfig::new(4);
    macro_cfg.min_promotion_support = 4;
    macro_cfg.match_threshold = 0.97;
    macro_cfg.formation_enabled = true;
    macro_cfg.revision_enabled = true;
    macro_cfg.readout_enabled = true;
    macro_cfg.learning_enabled = true;
    evo.enable_macro_memory(macro_cfg);
    evo
}

fn train(arm: Arm) -> (EvoPhase, [usize; 4]) {
    // Opaque motor assignment: no geometric property maps to the numeric ID.
    let motor_for_family = [2usize, 0, 3, 1];
    let robust_during_formation = matches!(arm, Arm::Robust | Arm::NoRobustReadout);
    let mut evo = organism(robust_during_formation);

    let origins = [(0usize, 0usize), (4, 0), (0, 4), (4, 4)];

    for family in 0..4usize {
        let pre_code = 120_000 + family as u64;
        let correct_first = motor_for_family[family];

        for (trial, (ox, oy)) in origins.iter().copied().enumerate() {
            let branch = trial % 2;
            let pre = render_relation(pre_code, ox, oy, 0, 0xA100 + trial as u64);
            let mid = render_relation(
                130_000 + family as u64 * 8 + branch as u64,
                ox,
                oy,
                0,
                0xB100 + trial as u64,
            );
            let terminal = (correct_first + branch + 1) % 4;

            // Every opaque motor is physically tried for every family. Only the
            // factual successful first action contributes positive macro formation.
            for action in 0..4usize {
                evo.observe_factual_macro_episode(
                    &pre,
                    action,
                    &mid,
                    terminal,
                    action == correct_first,
                );
            }
        }
    }

    assert_eq!(
        evo.macros().len(),
        4,
        "clean factual tuition must acquire four distinct action-bearing macros"
    );
    evo.set_macro_learning_enabled(false);

    match arm {
        Arm::Robust => evo.set_robust_high_level_perception(true),
        Arm::DirectedOnly => evo.set_robust_high_level_perception(false),
        Arm::NoRobustFormation => evo.set_robust_high_level_perception(true),
        Arm::NoRobustReadout => evo.set_robust_high_level_perception(false),
    }

    (evo, motor_for_family)
}

fn evaluate(arm: Arm) -> ([usize; 5], usize) {
    let (evo, motor_for_family) = train(arm);
    let nuisance = [0u8, DISTRACTOR, DROPOUT, ROTATE, SCALE];
    let mut per_kind = [0usize; 5];
    let mut total = 0usize;

    for family in 0..4usize {
        for (kind, mask) in nuisance.iter().copied().enumerate() {
            // All evaluation origins are absent from tuition.
            let origin = if family % 2 == 0 { (5usize, 5usize) } else { (5, 4) };
            let sensory = render_relation(
                120_000 + family as u64,
                origin.0,
                origin.1,
                mask,
                0xC100_0000 + family as u64 * 32 + kind as u64,
            );
            let mut trial = evo.clone();
            let action = trial.begin_macro_invocation(&sensory);
            let success = action == Some(motor_for_family[family]);
            per_kind[kind] += usize::from(success);
            total += usize::from(success);
            println!(
                "R1_MECHANISM arm={:?} family={} nuisance={} action={:?} expected={} success={}",
                arm, family, mask, action, motor_for_family[family], success
            );
        }
    }

    (per_kind, total)
}

#[test]
fn r1_robust_carrier_preserves_acquired_motor_meaning_under_nuisance() {
    let (robust, robust_total) = evaluate(Arm::Robust);
    let (directed, directed_total) = evaluate(Arm::DirectedOnly);
    let (no_formation, no_formation_total) = evaluate(Arm::NoRobustFormation);
    let (no_readout, no_readout_total) = evaluate(Arm::NoRobustReadout);

    let robust_clean = robust[0];
    let robust_nuisance: usize = robust[1..].iter().sum();
    let directed_nuisance: usize = directed[1..].iter().sum();

    println!(
        "R1_RESULT robust={:?}/20 directed={:?}/20 no_formation={:?}/20 no_readout={:?}/20 totals=[{},{},{},{}]",
        robust, directed, no_formation, no_readout,
        robust_total, directed_total, no_formation_total, no_readout_total
    );

    assert!(
        robust_clean >= 4,
        "robust carrier must retain at least 90% clean held-out accuracy"
    );
    for (idx, success) in robust[1..].iter().copied().enumerate() {
        assert!(
            success > 0,
            "every nuisance category must have non-zero robust success; category={}",
            idx + 1
        );
    }
    assert!(
        robust_nuisance * 100 >= 80 * 16,
        "aggregate robust nuisance accuracy must be at least 80%"
    );
    assert!(
        robust_nuisance > directed_nuisance,
        "robust carrier must strictly exceed directed-only nuisance accuracy"
    );
    assert!(
        robust_total >= no_formation_total + 4,
        "removing robust formation must remove a material part of the advantage"
    );
    assert!(
        robust_total >= no_readout_total + 4,
        "removing robust readout must remove a material part of the advantage"
    );
}
