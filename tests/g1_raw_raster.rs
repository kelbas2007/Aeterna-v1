use aeterna_v1::{EvoConfig, EvoPhase, RasterFieldConfig};

#[derive(Clone, Copy)]
enum PrivateClass {
    A,
    B,
}

fn raster(class: PrivateClass, x: usize, y: usize) -> Vec<f32> {
    let mut values = vec![0.0; 12 * 12];
    let points: [(usize, usize); 3] = match class {
        PrivateClass::A => [(x, y), (x + 1, y), (x + 2, y)],
        PrivateClass::B => [(x, y), (x, y + 1), (x, y + 2)],
    };
    for (px, py) in points {
        values[py * 12 + px] = 1.0;
    }
    values
}

fn world(class: PrivateClass, action: usize) -> bool {
    match class {
        PrivateClass::A => action == 0,
        PrivateClass::B => action == 1,
    }
}

fn carrier(formation: bool) -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 2;
    cfg.dormant_cells = 96;
    cfg.hdc_dim = 192;
    cfg.residual_recruit_threshold = 0.22;
    cfg.min_recruit_support = 2;

    let mut evo = EvoPhase::new(cfg);
    let mut raster_cfg = RasterFieldConfig::for_raster(12, 12, 2);
    raster_cfg.max_units = 64;
    raster_cfg.match_threshold = 0.97;
    raster_cfg.min_active = 2;
    raster_cfg.formation_enabled = formation;
    raster_cfg.readout_enabled = false;
    raster_cfg.learning_enabled = true;
    evo.attach_raster_field(raster_cfg);
    evo
}

#[test]
fn g1_raw_raster_acquires_translation_relative_carrier_motif() {
    let mut genuine = carrier(true);
    let mut no_formation = carrier(false);

    // Acquisition positions are fixed before qualification. Both arms receive
    // identical raw rasters and, because motif readout is disabled, must choose
    // the same opaque motor at every acquisition step.
    let training = [
        (PrivateClass::A, 1usize, 1usize),
        (PrivateClass::B, 8usize, 1usize),
        (PrivateClass::A, 4usize, 4usize),
        (PrivateClass::B, 2usize, 6usize),
        (PrivateClass::A, 7usize, 7usize),
        (PrivateClass::B, 9usize, 6usize),
    ];

    let mut factual_tuples = Vec::new();
    for _epoch in 0..28 {
        for (class, x, y) in training {
            let pre = raster(class, x, y);

            genuine.observe_initial_real(&pre, false);
            no_formation.observe_initial_real(&pre, false);

            let ga = genuine.choose_motor();
            let ca = no_formation.choose_motor();
            assert_eq!(ga, ca, "formation must not affect acquisition policy while readout is off");

            let need = world(class, ga);
            factual_tuples.push((ga, need));

            genuine.learn_factual_transition(ga, &pre, need);
            no_formation.learn_factual_transition(ca, &pre, need);
        }
    }

    assert!(!factual_tuples.is_empty());
    assert!(genuine.raster_units() > 0, "GENUINE must acquire raster carrier motifs");
    assert_eq!(
        no_formation.raster_units(),
        0,
        "NO_FORMATION must preserve the same experience without acquiring motifs"
    );

    // Pre-registered held-out translations: these absolute addresses never
    // appeared during acquisition.
    let heldout_a = raster(PrivateClass::A, 2, 9);
    let heldout_b = raster(PrivateClass::B, 6, 2);

    // A translation-relative motif must literally be the same acquired carrier
    // identity on a training surface and its unseen translation.
    let train_a = raster(PrivateClass::A, 1, 1);
    let train_ids = genuine
        .raster_field()
        .expect("attached field")
        .active_unit_ids(&train_a);
    let heldout_ids = genuine
        .raster_field()
        .expect("attached field")
        .active_unit_ids(&heldout_a);

    assert!(
        train_ids.iter().any(|id| heldout_ids.contains(id)),
        "at least one acquired carrier motif must survive translation as the same identity"
    );

    let mut g_readout = genuine.clone();
    g_readout.set_raster_learning_enabled(false);
    g_readout.set_raster_readout_enabled(true);

    let mut g_no_readout = genuine.clone();
    g_no_readout.set_raster_learning_enabled(false);
    g_no_readout.set_raster_readout_enabled(false);

    let mut control = no_formation.clone();
    control.set_raster_learning_enabled(false);
    control.set_raster_readout_enabled(true);

    let heldout = [
        (PrivateClass::A, heldout_a),
        (PrivateClass::B, heldout_b),
    ];

    let mut g_ok = 0usize;
    let mut no_readout_ok = 0usize;
    let mut control_ok = 0usize;

    for (class, pre) in heldout {
        let mut g = g_readout.clone();
        let mut n = g_no_readout.clone();
        let mut c = control.clone();

        g.observe_initial_real(&pre, false);
        n.observe_initial_real(&pre, false);
        c.observe_initial_real(&pre, false);

        let ga = g.choose_motor();
        let na = n.choose_motor();
        let ca = c.choose_motor();

        g_ok += usize::from(world(class, ga));
        no_readout_ok += usize::from(world(class, na));
        control_ok += usize::from(world(class, ca));
    }

    assert_eq!(
        g_ok, 2,
        "GENUINE readout must choose the evaluator-correct opaque motor on both unseen translations"
    );
    assert!(
        no_readout_ok < g_ok || control_ok < g_ok,
        "removing acquired motif readout or formation must remove the full held-out advantage"
    );
}
