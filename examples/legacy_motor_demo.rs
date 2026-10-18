use aeterna_v1::{Authority, EvoConfig, EvoPhase};

fn main() {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 8;
    cfg.motor_cells = 2;
    cfg.dormant_cells = 8;
    cfg.hdc_dim = 64;
    cfg.residual_recruit_threshold = 0.22;
    cfg.min_recruit_support = 2;

    let mut evo = EvoPhase::new(cfg);
    let mut pre = vec![0.0; 8];
    pre[0] = 1.0;
    pre[2] = 1.0;
    evo.observe_initial_real(&pre, false);

    let action = evo.choose_motor();
    let before = evo.predict(action, Authority::Model);
    let report = evo.learn_factual_transition(action, &pre, action == 0);

    println!(
        "action={action} predicted_need={:.3} confidence={:.3} residual={:.3} relays={}",
        before.need,
        before.confidence,
        report.residual,
        evo.recruited_relays()
    );
}
