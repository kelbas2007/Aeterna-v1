use aeterna_v1::{
    EvoConfig, EvoPhase, HierarchyConfig, MacroConfig, RasterFieldConfig,
};

#[derive(Clone, Copy, Debug)]
enum CueKind {
    First,
    Second,
}

fn shape(points: &[(usize, usize)], ox: usize, oy: usize) -> Vec<f32> {
    let mut raster = vec![0.0; 12 * 12];
    for (x, y) in points {
        raster[(oy + y) * 12 + (ox + x)] = 1.0;
    }
    raster
}

fn child_pre(ox: usize, oy: usize) -> Vec<f32> {
    shape(&[(0, 0), (1, 0), (0, 1)], ox, oy)
}

fn child_mid(first_action: usize, branch: usize, ox: usize, oy: usize) -> Vec<f32> {
    match (first_action, branch) {
        (0, 0) => shape(&[(0, 0), (1, 0), (2, 0)], ox, oy),
        (0, 1) => shape(&[(0, 0), (0, 1), (0, 2)], ox, oy),
        (2, 0) => shape(&[(0, 0), (1, 1), (2, 2)], ox, oy),
        (2, 1) => shape(&[(0, 1), (1, 0), (1, 1)], ox, oy),
        _ => shape(&[(0, 0), (2, 0), (1, 1)], ox, oy),
    }
}

fn terminal_for(first_action: usize, branch: usize) -> usize {
    match (first_action, branch) {
        (0, 0) => 1,
        (0, 1) => 2,
        (2, 0) => 0,
        (2, 1) => 1,
        _ => 0,
    }
}

fn outer_cue(kind: CueKind, ox: usize, oy: usize) -> Vec<f32> {
    match kind {
        CueKind::First => shape(&[(0, 0), (1, 0), (2, 0)], ox, oy),
        CueKind::Second => shape(&[(0, 0), (0, 1), (0, 2)], ox, oy),
    }
}

fn carrier() -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 3;
    cfg.dormant_cells = 128;
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
    macro_cfg.readout_enabled = true;
    macro_cfg.learning_enabled = true;
    evo.enable_macro_memory(macro_cfg);

    evo
}

fn train_child(evo: &mut EvoPhase, first_action: usize, translations: &[(usize, usize)]) {
    for (i, (ox, oy)) in translations.iter().copied().enumerate() {
        let branch = i % 2;
        let pre = child_pre(ox, oy);
        let mid = child_mid(first_action, branch, ox, oy);
        let second = terminal_for(first_action, branch);
        evo.observe_successful_macro_episode(&pre, first_action, &mid, second, true);
    }
}

fn acquire_two_children(reverse_order: bool) -> EvoPhase {
    let mut evo = carrier();
    let translations_a = [(1, 1), (3, 1), (5, 1), (7, 1)];
    let translations_b = [(1, 5), (3, 5), (5, 5), (7, 5)];

    if reverse_order {
        train_child(&mut evo, 2, &translations_b);
        train_child(&mut evo, 0, &translations_a);
    } else {
        train_child(&mut evo, 0, &translations_a);
        train_child(&mut evo, 2, &translations_b);
    }

    assert_eq!(evo.macros().len(), 2);
    evo
}

fn child_ids(evo: &EvoPhase) -> (u64, u64) {
    let a = evo
        .macros()
        .iter()
        .find(|m| m.first_action == 0)
        .expect("child A")
        .id;
    let b = evo
        .macros()
        .iter()
        .find(|m| m.first_action == 2)
        .expect("child B")
        .id;
    (a, b)
}

fn execute_child(
    evo: &mut EvoPhase,
    child_id: u64,
    branch: usize,
    ox: usize,
    oy: usize,
) -> Option<[usize; 2]> {
    let pre = child_pre(ox, oy);
    let first = evo.begin_macro_by_id(child_id, &pre)?;
    let mid = child_mid(first, branch, ox, oy);
    let second = evo.continue_macro_invocation(&mid)?;
    Some([first, second])
}

fn expected_signature(first_action: usize, branch: usize) -> [usize; 2] {
    [first_action, terminal_for(first_action, branch)]
}

fn required_skill_order(kind: CueKind) -> [usize; 2] {
    match kind {
        // Skill identity is defined only by its emitted primitive behavior.
        CueKind::First => [2, 0],
        CueKind::Second => [0, 2],
    }
}

fn evaluate_sequence(
    evo: &mut EvoPhase,
    kind: CueKind,
    child_sequence: &[u64],
    ox: usize,
    oy: usize,
) -> (bool, usize) {
    let required = required_skill_order(kind);
    let branches = [0usize, 1usize];

    if child_sequence.len() != 2 {
        return (false, 0);
    }

    let mut primitive_actions = 0usize;

    for stage in 0..2 {
        let Some(actual) = execute_child(
            evo,
            child_sequence[stage],
            branches[stage],
            ox + stage,
            oy + stage,
        ) else {
            return (false, primitive_actions);
        };
        primitive_actions += 2;

        let expected = expected_signature(required[stage], branches[stage]);
        if actual != expected {
            return (false, primitive_actions);
        }
    }

    (true, primitive_actions)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParentRecord {
    cue: u8,
    translation: (usize, usize),
    child_sequence: Vec<u64>,
    need: bool,
    primitive_actions: usize,
}

fn all_child_sequences(children: &[u64]) -> Vec<Vec<u64>> {
    let mut ids = children.to_vec();
    ids.sort_unstable();

    let mut out = Vec::new();
    for a in &ids {
        for b in &ids {
            out.push(vec![*a, *b]);
        }
    }
    out
}

fn parent_tuition(
    evo: &mut EvoPhase,
    hierarchy_enabled: bool,
) -> (Vec<ParentRecord>, usize) {
    let mut cfg = HierarchyConfig::default();
    cfg.min_promotion_support = 3;
    cfg.formation_enabled = hierarchy_enabled;
    cfg.readout_enabled = false;
    cfg.learning_enabled = true;
    evo.enable_hierarchy_memory(cfg);

    let children = evo.macros().iter().map(|m| m.id).collect::<Vec<_>>();
    let candidates = all_child_sequences(&children);
    let episodes = [
        (CueKind::First, 1usize, 8usize),
        (CueKind::Second, 5usize, 8usize),
        (CueKind::First, 8usize, 3usize),
        (CueKind::Second, 2usize, 4usize),
        (CueKind::First, 4usize, 2usize),
        (CueKind::Second, 7usize, 6usize),
    ];

    let mut ledger = Vec::new();
    let mut cost = 0usize;

    for (kind, ox, oy) in episodes {
        let cue = outer_cue(kind, ox, oy);

        for sequence in &candidates {
            let (need, primitive_actions) =
                evaluate_sequence(evo, kind, sequence, ox, oy);
            cost += primitive_actions;

            evo.observe_successful_parent_sequence(&cue, sequence.clone(), need);

            ledger.push(ParentRecord {
                cue: match kind {
                    CueKind::First => 0,
                    CueKind::Second => 1,
                },
                translation: (ox, oy),
                child_sequence: sequence.clone(),
                need,
                primitive_actions,
            });
        }
    }

    (ledger, cost)
}

fn heldout_with_parent(
    mature: &EvoPhase,
    kind: CueKind,
    ox: usize,
    oy: usize,
) -> (bool, usize, usize) {
    let mut evo = mature.clone();
    evo.set_hierarchy_learning_enabled(false);
    evo.set_hierarchy_readout_enabled(true);

    let cue = outer_cue(kind, ox, oy);
    let sequence = evo
        .select_parent_sequence(&cue)
        .expect("acquired parent must select child sequence");

    let (need, primitive_actions) = evaluate_sequence(&mut evo, kind, &sequence, ox, oy);
    (need, 1, primitive_actions)
}

fn heldout_generic_search(
    mature: &EvoPhase,
    kind: CueKind,
    ox: usize,
    oy: usize,
) -> (bool, usize, usize) {
    let children = mature.macros().iter().map(|m| m.id).collect::<Vec<_>>();
    let candidates = all_child_sequences(&children);

    let mut evaluations = 0usize;
    let mut primitive_actions = 0usize;

    for sequence in candidates {
        evaluations += 1;
        let mut evo = mature.clone();
        let (need, actions) = evaluate_sequence(&mut evo, kind, &sequence, ox, oy);
        primitive_actions += actions;
        if need {
            return (true, evaluations, primitive_actions);
        }
    }

    (false, evaluations, primitive_actions)
}

fn qualify(reverse_child_ids: bool) -> (usize, usize) {
    let base = acquire_two_children(reverse_child_ids);
    let (child_a, child_b) = child_ids(&base);

    let mut genuine = base.clone();
    let mut no_hierarchy = base.clone();

    let (g_ledger, g_tuition_cost) = parent_tuition(&mut genuine, true);
    let (c_ledger, c_tuition_cost) = parent_tuition(&mut no_hierarchy, false);

    assert_eq!(g_ledger, c_ledger, "parent tuition facts must match exactly");
    assert_eq!(g_tuition_cost, c_tuition_cost);
    assert!(g_tuition_cost > 0);

    assert_eq!(genuine.parent_macros().len(), 2);
    assert_eq!(no_hierarchy.parent_macros().len(), 0);

    for parent in genuine.parent_macros() {
        assert_eq!(parent.child_sequence.len(), 2);
        assert!(parent
            .child_sequence
            .iter()
            .all(|id| *id == child_a || *id == child_b));
    }

    let heldout = [
        // These absolute translations are absent from parent tuition.
        (CueKind::First, 8usize, 4usize),
        (CueKind::Second, 4usize, 7usize),
    ];

    let mut parent_evals = 0usize;
    let mut control_evals = 0usize;
    let mut parent_actions = 0usize;
    let mut control_actions = 0usize;

    for (kind, ox, oy) in heldout {
        let (g_need, g_eval, g_actions) = heldout_with_parent(&genuine, kind, ox, oy);
        let (c_need, c_eval, c_actions) =
            heldout_generic_search(&no_hierarchy, kind, ox, oy);

        assert!(g_need);
        assert!(c_need);

        parent_evals += g_eval;
        control_evals += c_eval;
        parent_actions += g_actions;
        control_actions += c_actions;
    }

    assert!(
        parent_evals < control_evals,
        "acquired hierarchy must reduce candidate child-sequence evaluations"
    );
    assert!(
        parent_actions < control_actions,
        "acquired hierarchy must reduce primitive physical actions spent searching"
    );

    // Stronger opacity check: swap the two already-acquired macro IDs *after*
    // parent acquisition and remap parent references atomically. No skill
    // semantics may depend on numeric IDs.
    let mut permuted = genuine.clone();
    permuted.apply_macro_id_permutation(&[(child_a, child_b), (child_b, child_a)]);

    for (kind, ox, oy) in [
        (CueKind::First, 7usize, 4usize),
        (CueKind::Second, 3usize, 7usize),
    ] {
        let (need, evaluations, _actions) =
            heldout_with_parent(&permuted, kind, ox, oy);
        assert!(need, "consistent opaque child-id permutation must preserve capability");
        assert_eq!(evaluations, 1);
    }

    (parent_evals, control_evals)
}

#[test]
fn g5_acquired_parent_reuses_acquired_child_macros_hierarchically() {
    let normal = acquire_two_children(false);
    let reversed = acquire_two_children(true);

    let (normal_a, normal_b) = child_ids(&normal);
    let (reversed_a, reversed_b) = child_ids(&reversed);

    assert_ne!(
        (normal_a, normal_b),
        (reversed_a, reversed_b),
        "reversing child acquisition order must permute opaque carrier ids"
    );

    let (g1, c1) = qualify(false);
    let (g2, c2) = qualify(true);

    assert!(g1 < c1);
    assert!(g2 < c2);
}
