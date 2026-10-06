use aeterna_v1::{EvoConfig, EvoPhase, ExplorationConfig, RasterFieldConfig};

pub fn relation(code: u64, ox: usize, oy: usize) -> Vec<f32> {
    let mut raster = vec![0.0; 12 * 12];
    let mut x = code ^ 0x9E37_79B9_7F4A_7C15;
    let mut slots = Vec::new();
    while slots.len() < 4 {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        let slot = (x.wrapping_mul(0x2545_F491_4F6C_DD1D) % 16) as usize;
        if !slots.contains(&slot) {
            slots.push(slot);
        }
    }
    for slot in slots {
        let lx = slot % 4;
        let ly = slot / 4;
        raster[(oy + ly) * 12 + (ox + lx)] = 1.0;
    }
    raster
}

pub fn carrier() -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 4;
    cfg.dormant_cells = 96;
    cfg.hdc_dim = 192;
    let mut evo = EvoPhase::new(cfg);

    let mut raster_cfg = RasterFieldConfig::for_raster(12, 12, 4);
    raster_cfg.match_threshold = 0.97;
    raster_cfg.learning_enabled = false;
    raster_cfg.readout_enabled = false;
    evo.attach_raster_field(raster_cfg);

    evo.enable_epistemic_state(0.97);
    evo.enable_exploration_strategy(ExplorationConfig::default());
    evo
}

pub fn post_code(
    family: usize,
    action: usize,
    law: usize,
    informative: usize,
    partial: Option<usize>,
    rivals: usize,
) -> u64 {
    let base = 10_000 + family as u64 * 1_000 + action as u64 * 100;
    if action == informative {
        return base + law as u64 + 1;
    }
    if partial == Some(action) && rivals >= 3 {
        return base + if law + 1 == rivals { 2 } else { 1 };
    }
    base + 50
}

pub fn install_world_models(
    evo: &mut EvoPhase,
    family: usize,
    pre_code: u64,
    informative: usize,
    partial: Option<usize>,
    rivals: usize,
) {
    for law in 0..rivals {
        let pre = relation(pre_code, 1 + law, 1);
        evo.begin_epistemic_tuition(&pre);
        for action in 0..4 {
            let code = post_code(family, action, law, informative, partial, rivals);
            let post = relation(code, 4, 5);
            evo.record_epistemic_tuition_transition(action, &post);
        }
        evo.commit_epistemic_tuition();
    }
}

pub fn actual_post(
    family: usize,
    action: usize,
    law: usize,
    informative: usize,
    partial: Option<usize>,
    rivals: usize,
) -> Vec<f32> {
    relation(
        post_code(family, action, law, informative, partial, rivals),
        6,
        6,
    )
}

pub fn train_strategy(evo: &mut EvoPhase) -> u32 {
    assert_eq!(evo.exploration_weights(), Some([0.0; 3]));
    let mut probes = 0u32;

    for family in 0..8usize {
        let informative = family % 4;
        let pre_code = 1_000 + family as u64;
        install_world_models(evo, family, pre_code, informative, None, 2);

        for action in 0..4 {
            let pre = relation(pre_code, 6, 2);
            let before = evo.begin_epistemic_episode(&pre);
            assert_eq!(before, 2);

            let features = evo.epistemic_probe_features(action);
            evo.mark_epistemic_probe_selected(action);

            let post = actual_post(family, action, 0, informative, None, 2);
            let after = evo.observe_epistemic_probe(action, &post);
            evo.train_exploration_from_factual_gain(features, before, after);
            probes = probes.saturating_add(1);
        }
    }
    probes
}

pub fn all_orders() -> Vec<[usize; 4]> {
    let mut out = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            if b == a { continue; }
            for c in 0..4 {
                if c == a || c == b { continue; }
                for d in 0..4 {
                    if d == a || d == b || d == c { continue; }
                    out.push([a, b, c, d]);
                }
            }
        }
    }
    assert_eq!(out.len(), 24);
    out
}

pub fn probes_for_order(
    mature: &EvoPhase,
    pre: &[f32],
    family: usize,
    informative: usize,
    partial: usize,
    law: usize,
    order: [usize; 4],
) -> usize {
    let mut evo = mature.clone();
    assert_eq!(evo.begin_epistemic_episode(pre), 3);
    for (i, action) in order.into_iter().enumerate() {
        evo.mark_epistemic_probe_selected(action);
        let post = actual_post(family, action, law, informative, Some(partial), 3);
        let remaining = evo.observe_epistemic_probe(action, &post);
        if remaining == 1 {
            return i + 1;
        }
    }
    4
}
