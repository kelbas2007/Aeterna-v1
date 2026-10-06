use aeterna_v1::{EvoConfig, EvoPhase};

const SIDE: usize = 12;
const PIXELS: usize = SIDE * SIDE;

fn cfg(growth: bool) -> EvoConfig {
    EvoConfig {
        sensory_cells: PIXELS,
        motor_cells: 4,
        dormant_cells: 96,
        hdc_dim: 256,
        residual_recruit_threshold: 0.22,
        min_recruit_support: 2,
        structural_growth_enabled: growth,
        acquired_readout_enabled: true,
        ..EvoConfig::default()
    }
}

fn pixel(frame: &mut [f32], row: usize, col: usize) {
    frame[row * SIDE + col] = 1.0;
}

fn raster(context: usize, nuisance_mask: u8) -> Vec<f32> {
    let mut frame = vec![0.0; PIXELS];

    // Surface structure shared by both latent contexts.
    for (row, col) in [(1, 1), (10, 10), (5, 5)] {
        pixel(&mut frame, row, col);
    }

    // These are just raw pixels to the carrier. Their interpretation exists
    // only in this evaluator.
    match context {
        0 => {
            for (row, col) in [(2, 3), (2, 4), (3, 3), (3, 4)] {
                pixel(&mut frame, row, col);
            }
        }
        1 => {
            for (row, col) in [(7, 8), (7, 9), (8, 8), (8, 9)] {
                pixel(&mut frame, row, col);
            }
        }
        _ => panic!("unknown evaluator context"),
    }

    let nuisance = [
        (1, 9),
        (9, 1),
        (4, 10),
        (10, 4),
        (6, 1),
        (1, 6),
        (9, 9),
        (4, 1),
    ];
    for (bit, (row, col)) in nuisance.into_iter().enumerate() {
        if nuisance_mask & (1 << bit) != 0 {
            pixel(&mut frame, row, col);
        }
    }
    frame
}

fn factual_need(context: usize, opaque_motor: usize) -> bool {
    (context == 0 && opaque_motor == 2) || (context == 1 && opaque_motor == 3)
}

fn common_experience(evo: &mut EvoPhase) {
    // Every arm receives exactly the same factual PRE/action/Need experience.
    // The curriculum is deliberately balanced over all opaque motors, so the
    // evaluator is not demonstrating only the correct answer.
    for epoch in 0..24usize {
        for context in [0usize, 1usize] {
            let nuisance = ((epoch * 37 + context * 11) & 0xff) as u8;
            let pre = raster(context, nuisance);
            for action in 0..4usize {
                evo.observe_initial_real(&pre, false);
                let need = factual_need(context, action);
                let _ = evo.learn_factual_transition(action, &pre, need);
            }
        }
    }
}

fn held_out_score(evo: &mut EvoPhase) -> (usize, Vec<usize>) {
    // Nuisance masks are absent from the acquisition set.
    let cases = [
        (0usize, 255u8),
        (1usize, 170u8),
        (0usize, 200u8),
        (1usize, 240u8),
    ];
    let mut score = 0usize;
    let mut actions = Vec::new();
    for (context, nuisance) in cases {
        let pre = raster(context, nuisance);
        evo.observe_initial_real(&pre, false);
        let action = evo.choose_motor();
        score += usize::from(factual_need(context, action));
        actions.push(action);
    }
    (score, actions)
}

#[test]
fn g1_raw_raster_distinction_requires_formation_and_readout() {
    let mut genuine = EvoPhase::new(cfg(true));
    let mut no_formation = EvoPhase::new(cfg(false));

    common_experience(&mut genuine);
    common_experience(&mut no_formation);

    assert!(
        genuine.recruited_relays() > 0,
        "GENUINE must form carrier structure from raw raster experience"
    );
    assert_eq!(
        no_formation.recruited_relays(),
        0,
        "NO_FORMATION must preserve the primitive substrate without recruitment"
    );

    // Clean downstream ablation: same acquired carrier state, only readout is disabled.
    let mut no_readout = genuine.clone();
    no_readout.set_acquired_readout_enabled(false);
    assert_eq!(
        no_readout.recruited_relays(),
        genuine.recruited_relays(),
        "NO_READOUT must retain the acquired structure"
    );

    let (g_score, g_actions) = held_out_score(&mut genuine);
    let (f_score, f_actions) = held_out_score(&mut no_formation);
    let (r_score, r_actions) = held_out_score(&mut no_readout);

    assert_eq!(
        g_score, 4,
        "GENUINE must transfer the acquired distinction across unseen surface nuisance: {g_actions:?}"
    );
    assert!(
        f_score < g_score,
        "formation ablation must lose the held-out advantage: genuine={g_actions:?}, no_formation={f_actions:?}"
    );
    assert!(
        r_score < g_score,
        "readout ablation must lose the held-out advantage while keeping formed state: genuine={g_actions:?}, no_readout={r_actions:?}"
    );
}
