use aeterna_v1::{Authority, EvoConfig, EvoPhase};

#[test]
fn basic_carrier_cycle_is_model_not_fact() {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 4;
    cfg.motor_cells = 2;
    cfg.dormant_cells = 8;
    let mut evo = EvoPhase::new(cfg);

    let pre = vec![1.0, 0.0, 1.0, 0.0];
    evo.observe_initial_real(&pre, false);
    let action = evo.choose_motor();
    let p = evo.predict(action, Authority::Model);
    assert_eq!(p.authority, Authority::Model);

    let report = evo.learn_factual_transition(action, &pre, action == 0);
    assert!(report.residual.is_finite());
    assert_eq!(evo.current_real().unwrap().sensory, pre);
}
