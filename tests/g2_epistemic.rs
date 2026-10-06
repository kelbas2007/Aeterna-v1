use aeterna_v1::{EvoConfig, EvoPhase, RasterFieldConfig};

#[derive(Clone, Copy, Debug)]
enum HiddenLaw {
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

fn post_scene(law: HiddenLaw, action: usize, ox: usize, oy: usize) -> Vec<f32> {
    match (law, action) {
        // Action 0 is deliberately non-discriminating.
        (_, 0) => shape(&[(0, 0), (1, 1), (2, 2)], ox, oy),

        // Action 1 carries the strongest learned disagreement.
        (HiddenLaw::Alpha, 1) => shape(&[(0, 0), (1, 0), (2, 0)], ox, oy),
        (HiddenLaw::Beta, 1) => shape(&[(0, 0), (0, 1), (0, 2)], ox, oy),

        // Action 2 is a weaker law-specific consequence and is withheld during
        // held-out identification so it can test the surviving hypothesis.
        (HiddenLaw::Alpha, 2) => shape(&[(0, 0), (1, 0), (0, 1)], ox, oy),
        (HiddenLaw::Beta, 2) => shape(&[(0, 0), (1, 0), (1, 1)], ox, oy),

        _ => panic!("unexpected opaque action"),
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
    raster_cfg.formation_enabled = true;
    raster_cfg.readout_enabled = false;
    raster_cfg.learning_enabled = false;
    evo.attach_raster_field(raster_cfg);
    evo.enable_epistemic_state(0.97);
    evo
}

fn tuition(evo: &mut EvoPhase) -> u32 {
    // Fixed evaluator schedule. Hidden law labels are used only to generate
    // external factual rasters; they are never passed to EvoPhase.
    let episodes = [
        (HiddenLaw::Alpha, 1usize, 1usize),
        (HiddenLaw::Beta, 7usize, 1usize),
        (HiddenLaw::Alpha, 3usize, 5usize),
        (HiddenLaw::Beta, 8usize, 6usize),
        (HiddenLaw::Alpha, 6usize, 8usize),
        (HiddenLaw::Beta, 1usize, 7usize),
    ];

    let mut factual_probes = 0u32;

    for (law, ox, oy) in episodes {
        let pre = start_scene(ox, oy);
        evo.begin_epistemic_tuition(&pre);

        // The schedule exposes all opaque probes equally. No correct or
        // informative action is identified to the carrier.
        for action in 0..3 {
            let post = post_scene(law, action, ox, oy);
            evo.record_epistemic_tuition_transition(action, &post);
            factual_probes = factual_probes.saturating_add(1);
        }

        evo.commit_epistemic_tuition();
    }

    factual_probes
}

fn identify(
    mature: &EvoPhase,
    law: HiddenLaw,
    epistemic: bool,
    ox: usize,
    oy: usize,
) -> (u32, usize, f32, f32, f32) {
    let mut evo = mature.clone();
    let pre = start_scene(ox, oy);

    let rivals = evo.begin_epistemic_episode(&pre);
    assert_eq!(rivals, 2, "both acquired rival hypotheses must coexist initially");

    let d0 = evo.epistemic_disagreement(0);
    let d1 = evo.epistemic_disagreement(1);
    let d2 = evo.epistemic_disagreement(2);

    let mut first_action = usize::MAX;
    while evo.active_epistemic_rivals() > 1 && evo.epistemic_physical_probes() < 3 {
        let action = evo.choose_epistemic_probe(epistemic);
        if first_action == usize::MAX {
            first_action = action;
        }

        // Each probe is a factual experiment from the same start condition.
        let post = post_scene(law, action, ox, oy);
        evo.observe_epistemic_probe(action, &post);
    }

    assert_eq!(
        evo.active_epistemic_rivals(),
        1,
        "held-out factual probes must identify one surviving carrier hypothesis"
    );

    // Action 2 was not required for GENUINE identification. Its factual
    // consequence is withheld until after the rival set has collapsed.
    let withheld = post_scene(law, 2, ox, oy);
    let predicted_similarity = evo
        .epistemic_prediction_similarity(2, &withheld)
        .expect("one surviving hypothesis must predict the withheld consequence");

    (
        evo.epistemic_physical_probes(),
        first_action,
        d0,
        d1,
        d2.min(predicted_similarity),
    )
}

#[test]
fn g2_disagreement_selects_the_informative_probe() {
    let mut mature = carrier();
    let tuition_cost = tuition(&mut mature);

    assert_eq!(tuition_cost, 18);
    assert_eq!(
        mature.epistemic_hypotheses().len(),
        2,
        "factual tuition must acquire two incompatible world hypotheses"
    );
    assert!(
        mature
            .epistemic_hypotheses()
            .iter()
            .all(|hypothesis| hypothesis.support >= 3),
        "translated tuition episodes of the same law must consolidate into the same carrier hypothesis"
    );

    // Both held-out worlds use absolute translations absent from tuition.
    for (law, ox, oy) in [
        (HiddenLaw::Alpha, 8usize, 3usize),
        (HiddenLaw::Beta, 4usize, 8usize),
    ] {
        let (g_probes, g_first, d0, d1, g_quality) =
            identify(&mature, law, true, ox, oy);
        let (c_probes, c_first, _, _, _) =
            identify(&mature, law, false, ox, oy);

        assert!(
            d1 > d0,
            "opaque probe 1 must have greater acquired disagreement than the non-discriminating probe"
        );
        assert!(
            g_quality > 0.90,
            "the surviving hypothesis must predict the withheld relational consequence"
        );
        assert_eq!(
            g_first, 1,
            "GENUINE must select the maximally discriminating acquired probe first"
        );
        assert_eq!(
            c_first, 0,
            "matched generic exploration must begin with the ordinary novelty tie-break"
        );
        assert_eq!(g_probes, 1, "GENUINE should identify the hidden law in one factual probe");
        assert!(
            c_probes > g_probes,
            "generic exploration must require more factual probes than disagreement-driven selection"
        );
    }
}
