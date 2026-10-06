use aeterna_v1::{EvoConfig, EvoPhase, MacroConfig, RasterFieldConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HiddenContext {
    Alpha,
    Beta,
}

fn shape(points: &[(usize, usize)], ox: usize, oy: usize) -> Vec<f32> {
    let mut raster = vec![0.0; 12 * 12];
    for (x, y) in points {
        raster[(oy + y) * 12 + (ox + x)] = 1.0;
    }
    raster
}

fn start_scene(ox: usize, oy: usize) -> Vec<f32> {
    shape(&[(0, 0), (1, 0), (0, 1)], ox, oy)
}

fn intermediate_scene(
    context: HiddenContext,
    first_action: usize,
    ox: usize,
    oy: usize,
) -> Vec<f32> {
    if first_action != 1 {
        return shape(&[(0, 0), (1, 1), (2, 2)], ox, oy);
    }

    match context {
        HiddenContext::Alpha => shape(&[(0, 0), (1, 0), (2, 0)], ox, oy),
        HiddenContext::Beta => shape(&[(0, 0), (0, 1), (0, 2)], ox, oy),
    }
}

fn final_need(context: HiddenContext, first_action: usize, second_action: usize) -> bool {
    if first_action != 1 {
        return false;
    }

    match context {
        HiddenContext::Alpha => second_action == 2,
        HiddenContext::Beta => second_action == 0,
    }
}

fn carrier(formation: bool) -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 3;
    cfg.dormant_cells = 96;
    cfg.hdc_dim = 192;

    let mut evo = EvoPhase::new(cfg);

    let mut raster_cfg = RasterFieldConfig::for_raster(12, 12, 3);
    raster_cfg.match_threshold = 0.97;
    raster_cfg.learning_enabled = false;
    raster_cfg.readout_enabled = false;
    evo.attach_raster_field(raster_cfg);

    let mut macro_cfg = MacroConfig::new(3);
    macro_cfg.match_threshold = 0.97;
    macro_cfg.min_promotion_support = 4;
    macro_cfg.formation_enabled = formation;
    macro_cfg.readout_enabled = false;
    macro_cfg.learning_enabled = true;
    evo.enable_macro_memory(macro_cfg);

    evo
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EpisodeRecord {
    context: u8,
    translation: (usize, usize),
    first_action: usize,
    second_action: usize,
    need: bool,
}

fn tuition(evo: &mut EvoPhase) -> (Vec<EpisodeRecord>, u32) {
    let worlds = [
        (HiddenContext::Alpha, 1usize, 1usize),
        (HiddenContext::Beta, 7usize, 1usize),
        (HiddenContext::Alpha, 3usize, 5usize),
        (HiddenContext::Beta, 8usize, 6usize),
        (HiddenContext::Alpha, 6usize, 8usize),
        (HiddenContext::Beta, 1usize, 7usize),
    ];

    let mut ledger = Vec::new();
    let mut physical_actions = 0u32;

    // Fixed generic exploration curriculum. Every two-action motor pair is
    // tried. The macro subsystem receives only factual trajectories + Need.
    for (context, ox, oy) in worlds {
        for first_action in 0..3 {
            for second_action in 0..3 {
                let pre = start_scene(ox, oy);
                let mid = intermediate_scene(context, first_action, ox, oy);
                let need = final_need(context, first_action, second_action);

                evo.observe_successful_macro_episode(
                    &pre,
                    first_action,
                    &mid,
                    second_action,
                    need,
                );

                physical_actions = physical_actions.saturating_add(2);
                ledger.push(EpisodeRecord {
                    context: match context {
                        HiddenContext::Alpha => 0,
                        HiddenContext::Beta => 1,
                    },
                    translation: (ox, oy),
                    first_action,
                    second_action,
                    need,
                });
            }
        }
    }

    (ledger, physical_actions)
}

fn heldout_episode(
    mature: &EvoPhase,
    context: HiddenContext,
    ox: usize,
    oy: usize,
    readout: bool,
) -> (usize, usize, bool) {
    let mut evo = mature.clone();
    evo.set_macro_learning_enabled(false);
    evo.set_macro_readout_enabled(readout);

    let pre = start_scene(ox, oy);
    evo.observe_initial_real(&pre, false);

    let first_action = evo
        .begin_macro_invocation(&pre)
        .unwrap_or_else(|| evo.choose_motor());

    let mid = intermediate_scene(context, first_action, ox, oy);
    evo.observe_initial_real(&mid, false);

    let second_action = evo
        .continue_macro_invocation(&mid)
        .unwrap_or_else(|| evo.choose_motor());

    let need = final_need(context, first_action, second_action);
    (first_action, second_action, need)
}

#[test]
fn g3_successful_carrier_dynamics_consolidate_into_reusable_macro() {
    let mut genuine = carrier(true);
    let mut no_consolidation = carrier(false);

    assert_eq!(genuine.macros().len(), 0, "macro must be absent before tuition");

    let (g_ledger, g_cost) = tuition(&mut genuine);
    let (c_ledger, c_cost) = tuition(&mut no_consolidation);

    assert_eq!(g_ledger, c_ledger, "matched arms must receive identical factual episodes");
    assert_eq!(g_cost, c_cost);
    assert_eq!(g_cost, 108, "tuition physical action cost must be explicit");

    assert_eq!(genuine.macros().len(), 1, "GENUINE must promote one reusable macro");
    assert_eq!(
        no_consolidation.macros().len(),
        0,
        "NO_CONSOLIDATION must retain the same facts without a macro"
    );

    let acquired = &genuine.macros()[0];
    assert_eq!(acquired.first_action, 1, "opaque first motor must be acquired from successful trajectories");
    assert!(acquired.support >= 6);
    assert!(acquired.branches.len() >= 2);

    let terminal_actions = acquired
        .branches
        .iter()
        .map(|branch| branch.next_action)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(terminal_actions.contains(&0));
    assert!(terminal_actions.contains(&2));

    // Clone is the current checkpoint-equivalent persistence boundary for G3:
    // acquired carrier state survives ordinary state duplication.
    let persisted = genuine.clone();
    assert_eq!(persisted.macros().len(), 1);
    assert_eq!(persisted.macros()[0].id, acquired.id);

    // Absolute translations below were absent from tuition.
    let heldout = [
        (HiddenContext::Alpha, 8usize, 3usize),
        (HiddenContext::Beta, 4usize, 8usize),
    ];

    let mut readout_success = 0usize;
    let mut no_readout_success = 0usize;
    let mut control_success = 0usize;
    let mut changed_first_action = false;

    for (context, ox, oy) in heldout {
        let (gf, gs, g_need) = heldout_episode(&persisted, context, ox, oy, true);
        let (nf, _ns, n_need) = heldout_episode(&persisted, context, ox, oy, false);
        let (cf, _cs, c_need) = heldout_episode(&no_consolidation, context, ox, oy, true);

        assert_eq!(gf, 1, "macro must invoke the learned first motor before held-out outcome");
        match context {
            HiddenContext::Alpha => assert_eq!(gs, 2),
            HiddenContext::Beta => assert_eq!(gs, 0),
        }

        readout_success += usize::from(g_need);
        no_readout_success += usize::from(n_need);
        control_success += usize::from(c_need);
        changed_first_action |= gf != nf || gf != cf;
    }

    assert_eq!(
        readout_success, 2,
        "the acquired macro must complete both unseen spatial bindings within two actions"
    );
    assert!(
        no_readout_success < readout_success || control_success < readout_success,
        "removing macro readout or formation must remove the complete held-out advantage"
    );
    assert!(
        changed_first_action,
        "the acquired macro must causally change at least one held-out physical action"
    );
}
