use aeterna_v1::{Authority, EvoConfig, EvoPhase};

fn cfg(growth: bool) -> EvoConfig {
    EvoConfig {
        sensory_cells: 8,
        motor_cells: 2,
        dormant_cells: 32,
        hdc_dim: 128,
        residual_recruit_threshold: 0.22,
        min_recruit_support: 2,
        structural_growth_enabled: growth,
        ..EvoConfig::default()
    }
}

fn a(surface: bool) -> Vec<f32> {
    let mut x = vec![0.0; 8];
    x[0] = 1.0;
    x[2] = 1.0;
    if surface { x[6] = 1.0; }
    x
}

fn b(surface: bool) -> Vec<f32> {
    let mut x = vec![0.0; 8];
    x[0] = 1.0;
    x[5] = 1.0;
    if surface { x[7] = 1.0; }
    x
}

fn world(ctx: usize, action: usize) -> bool {
    (ctx == 0 && action == 0) || (ctx == 1 && action == 1)
}

fn train(evo: &mut EvoPhase) {
    // Context order deliberately prevents a global novelty alternation from
    // masquerading as contextual competence.
    let schedule = [0usize, 0, 1, 1, 1, 0];
    for _ in 0..24 {
        for ctx in schedule {
            let pre = if ctx == 0 { a(false) } else { b(false) };
            evo.observe_initial_real(&pre, false);
            let action = evo.choose_motor();
            let need = world(ctx, action);
            evo.learn_factual_transition(action, &pre, need);
        }
    }
}

#[test]
fn model_and_imagined_never_overwrite_real() {
    let mut evo = EvoPhase::new(cfg(true));
    let pre = a(false);
    evo.observe_initial_real(&pre, false);
    let before = evo.current_real().unwrap().clone();

    let p = evo.predict(0, Authority::Imagined);
    assert_eq!(p.authority, Authority::Imagined);
    let after = evo.current_real().unwrap();
    assert_eq!(before.sensory, after.sensory);
    assert_eq!(before.need, after.need);
    assert_eq!(before.tick, after.tick);
}

#[test]
fn residual_recruits_only_when_growth_is_enabled() {
    let mut genuine = EvoPhase::new(cfg(true));
    let mut control = EvoPhase::new(cfg(false));
    for _ in 0..8 {
        let pre = a(false);
        for evo in [&mut genuine, &mut control] {
            evo.observe_initial_real(&pre, false);
            evo.learn_factual_transition(0, &pre, true);
        }
    }
    assert!(genuine.recruited_relays() > 0);
    assert_eq!(control.recruited_relays(), 0);
}

#[test]
fn acquired_evo_structure_changes_held_out_surface_action() {
    let mut genuine = EvoPhase::new(cfg(true));
    let mut control = EvoPhase::new(cfg(false));
    train(&mut genuine);
    train(&mut control);

    let mut g_ok = 0;
    let mut c_ok = 0;
    for (ctx, pre) in [(0usize, a(true)), (1usize, b(true))] {
        genuine.observe_initial_real(&pre, false);
        control.observe_initial_real(&pre, false);
        let ga = genuine.choose_motor();
        let ca = control.choose_motor();
        g_ok += usize::from(world(ctx, ga));
        c_ok += usize::from(world(ctx, ca));
    }

    assert!(genuine.recruited_relays() > 0);
    assert_eq!(g_ok, 2, "GENUINE must use acquired context on both held-out surfaces");
    assert!(c_ok < g_ok, "matched no-growth control must lose the contextual advantage");
}

#[test]
fn factual_counterexample_revises_same_evo_owned_branch() {
    let mut evo = EvoPhase::new(cfg(true));
    train(&mut evo);

    let pre = a(false);
    evo.observe_initial_real(&pre, false);
    let branch_idx = evo.branches().iter()
        .position(|b| b.motor == 0 && b.inputs.iter().all(|i| pre[*i] >= 0.5))
        .expect("trained context branch");
    let relay = evo.branches()[branch_idx].relay_cell;
    let revision_before = evo.branches()[branch_idx].revision;

    for _ in 0..4 {
        evo.observe_initial_real(&pre, false);
        evo.learn_factual_transition(0, &pre, false);
    }

    let same = evo.branches().iter().find(|b| b.relay_cell == relay).unwrap();
    assert!(same.revision > revision_before);
}
