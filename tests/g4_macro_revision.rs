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

fn old_need(context: HiddenContext, first_action: usize, second_action: usize) -> bool {
    if first_action != 1 {
        return false;
    }
    match context {
        HiddenContext::Alpha => second_action == 2,
        HiddenContext::Beta => second_action == 0,
    }
}

fn changed_need(context: HiddenContext, first_action: usize, second_action: usize) -> bool {
    if first_action != 1 {
        return false;
    }
    match context {
        HiddenContext::Alpha => second_action == 1,
        HiddenContext::Beta => second_action == 0,
    }
}

fn carrier() -> EvoPhase {
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
    macro_cfg.formation_enabled = true;
    macro_cfg.revision_enabled = true;
    macro_cfg.readout_enabled = false;
    macro_cfg.learning_enabled = true;
    evo.enable_macro_memory(macro_cfg);

    evo
}

fn g3_tuition(evo: &mut EvoPhase) {
    let worlds = [
        (HiddenContext::Alpha, 1usize, 1usize),
        (HiddenContext::Beta, 7usize, 1usize),
        (HiddenContext::Alpha, 3usize, 5usize),
        (HiddenContext::Beta, 8usize, 6usize),
        (HiddenContext::Alpha, 6usize, 8usize),
        (HiddenContext::Beta, 1usize, 7usize),
    ];

    for (context, ox, oy) in worlds {
        for first_action in 0..3 {
            for second_action in 0..3 {
                let pre = start_scene(ox, oy);
                let mid = intermediate_scene(context, first_action, ox, oy);
                let need = old_need(context, first_action, second_action);
                evo.observe_successful_macro_episode(
                    &pre,
                    first_action,
                    &mid,
                    second_action,
                    need,
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RevisionRecord {
    translation: (usize, usize),
    second_action: usize,
    need: bool,
}

fn revision_curriculum(evo: &mut EvoPhase) -> (Vec<RevisionRecord>, u32) {
    let translations = [(2usize, 8usize), (5usize, 5usize), (8usize, 2usize), (3usize, 7usize)];
    let mut ledger = Vec::new();
    let mut physical_actions = 0u32;

    for (ox, oy) in translations {
        let pre = start_scene(ox, oy);
        let mid = intermediate_scene(HiddenContext::Alpha, 1, ox, oy);

        // Generic terminal sweep: every opaque action is tried. The revision
        // mechanism is not told which action became correct.
        for second_action in 0..3 {
            let need = changed_need(HiddenContext::Alpha, 1, second_action);
            evo.observe_factual_macro_episode(
                &pre,
                1,
                &mid,
                second_action,
                need,
            );

            ledger.push(RevisionRecord {
                translation: (ox, oy),
                second_action,
                need,
            });
            physical_actions = physical_actions.saturating_add(2);
        }
    }

    (ledger, physical_actions)
}

fn run_macro(
    mature: &EvoPhase,
    context: HiddenContext,
    ox: usize,
    oy: usize,
) -> (usize, usize, bool) {
    let mut evo = mature.clone();
    evo.set_macro_learning_enabled(false);
    evo.set_macro_readout_enabled(true);

    let pre = start_scene(ox, oy);
    let first = evo
        .begin_macro_invocation(&pre)
        .expect("acquired macro must match held-out PRE");

    let mid = intermediate_scene(context, first, ox, oy);
    let second = evo
        .continue_macro_invocation(&mid)
        .expect("same acquired macro must select a terminal branch");

    let need = changed_need(context, first, second);
    (first, second, need)
}

#[test]
fn g4_factual_counterexamples_revise_same_acquired_macro() {
    let mut base = carrier();
    g3_tuition(&mut base);

    assert_eq!(base.macros().len(), 1);
    let frozen_id = base.macros()[0].id;
    let frozen_revision = base.macros()[0].revision;

    let alpha_old = base.macros()[0]
        .branches
        .iter()
        .find(|branch| branch.next_action == 2)
        .expect("G3 alpha branch must exist");
    assert_eq!(alpha_old.failures, 0);

    let mut revision = base.clone();
    let mut no_revision = base.clone();
    no_revision.set_macro_revision_enabled(false);

    let (r_ledger, r_cost) = revision_curriculum(&mut revision);
    let (c_ledger, c_cost) = revision_curriculum(&mut no_revision);

    assert_eq!(r_ledger, c_ledger, "revision ablation must receive identical factual evidence");
    assert_eq!(r_cost, c_cost);
    assert_eq!(r_cost, 24, "revision physical interaction cost must be explicit");

    assert_eq!(revision.macros().len(), 1, "revision must not spawn a replacement macro");
    assert_eq!(revision.macros()[0].id, frozen_id, "the same acquired macro identity must survive");
    assert!(
        revision.macros()[0].revision > frozen_revision,
        "factual contradictions must increment revision on the same macro"
    );
    assert!(
        revision.macros()[0].counterexamples.len() >= 8,
        "failed opaque terminal actions must remain as preserved factual counterexamples"
    );

    let old_branch = revision.macros()[0]
        .branches
        .iter()
        .find(|branch| branch.next_action == 2)
        .expect("obsolete action evidence must remain represented");
    assert!(
        old_branch.failures >= 4,
        "old alpha action must accumulate explicit contradiction evidence rather than being deleted"
    );

    let new_branch = revision.macros()[0]
        .branches
        .iter()
        .find(|branch| branch.next_action == 1)
        .expect("new factual success must create support for the revised alpha action");
    assert!(new_branch.support >= 4);

    assert_eq!(no_revision.macros().len(), 1);
    assert_eq!(no_revision.macros()[0].id, frozen_id);
    assert_eq!(
        no_revision.macros()[0].revision,
        frozen_revision,
        "NO_REVISION must keep the acquired program frozen"
    );

    // Held-out absolute translations are absent from G3 tuition and G4 revision.
    let (af, as_, a_need) = run_macro(&revision, HiddenContext::Alpha, 9, 4);
    let (bf, bs, b_need) = run_macro(&revision, HiddenContext::Beta, 5, 9);
    let (_cf, cs, c_need) = run_macro(&no_revision, HiddenContext::Alpha, 9, 4);

    assert_eq!(af, 1);
    assert_eq!(as_, 1, "revised alpha branch must select the newly supported opaque action");
    assert!(a_need);

    assert_eq!(bf, 1);
    assert_eq!(bs, 0, "unchanged beta branch must retain its old terminal action");
    assert!(b_need);

    assert_eq!(cs, 2, "frozen control must still invoke the obsolete alpha action");
    assert!(!c_need, "NO_REVISION must fail the changed held-out alpha context");
}
