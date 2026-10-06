//! Diagnostic of the CURRENT implementation, not a positive ownership gate.
//! Passing means a phase-plasticity-independent planning path was reproduced.
//! See docs/PHASE_EXECUTION_AUDIT.md for the preregistered interpretation.

use aeterna_v1::{
    Authority, CarrierTrace, EvoConfig, EvoImaginationPlanner, EvoPhase, PlanDecision,
    PlanningConfig, RasterFieldConfig,
};
use aeterna_v1::hdc::PhaseVector;

fn planner_config() -> PlanningConfig {
    PlanningConfig {
        max_depth: 3,
        node_budget: 64,
        discount: 0.95,
        ..PlanningConfig::new(3)
    }
}

fn signature(plan: &PlanDecision) -> (usize, usize, u32, usize) {
    (
        plan.first_action,
        plan.selected_depth,
        plan.predicted_value.to_bits(),
        plan.expanded_nodes,
    )
}

#[test]
fn current_planning_component_operates_without_a_phase_cell_network_instance() {
    // A component-level dependency witness: no EvoPhase is instantiated here.
    // HDC traces and a learned transition table explicitly remain available.
    let traces: Vec<_> = (10..15)
        .map(|seed| CarrierTrace::exact(PhaseVector::from_seed(192, seed)))
        .collect();
    let mut planner = EvoImaginationPlanner::new(planner_config());
    planner.observe_factual_transition(traces[0].clone(), 0, traces[4].clone(), 0.25);
    planner.observe_factual_transition(traces[0].clone(), 1, traces[1].clone(), 0.0);
    planner.observe_factual_transition(traces[1].clone(), 2, traces[2].clone(), 0.0);
    planner.observe_factual_transition(traces[2].clone(), 0, traces[3].clone(), 1.0);
    planner.set_learning_enabled(false);

    let decision = planner.plan(&traces[0]).expect("learned table supplies a route");
    assert_eq!(decision.first_action, 1);
    assert_eq!(decision.selected_depth, 3);
    assert_eq!(decision.authority, Authority::Imagined);
    assert!((decision.predicted_value - 0.95_f32.powi(2)).abs() < 1.0e-6);
    println!(
        "PHASE_EXECUTION_AUDIT standalone_table_planner=SUCCESS phase_network_instances=0 depth={} expanded={}",
        decision.selected_depth, decision.expanded_nodes
    );
}

fn raw_pair(state: usize, x: usize, y: usize) -> Vec<f32> {
    // Evaluator-only geometry. Production receives no state index or offset.
    let delta = match state {
        0 => (1, 0),
        1 => (2, 0),
        2 => (3, 0),
        3 => (0, 1),
        4 => (0, 2),
        _ => panic!("unknown diagnostic state"),
    };
    assert!(x + delta.0 < 12 && y + delta.1 < 12);
    let mut raster = vec![0.0; 144];
    raster[y * 12 + x] = 1.0;
    raster[(y + delta.1) * 12 + x + delta.0] = 1.0;
    raster
}

fn carrier(disable_phase_plasticity: bool) -> EvoPhase {
    let mut config = EvoConfig {
        sensory_cells: 144,
        motor_cells: 3,
        dormant_cells: 32,
        hdc_dim: 192,
        ..EvoConfig::default()
    };
    if disable_phase_plasticity {
        config.dormant_cells = 0;
        config.structural_growth_enabled = false;
        config.weight_learning_rate = 0.0;
        config.phase_learning_rate = 0.0;
        config.eligibility_decay = 0.0;
    }
    let mut evo = EvoPhase::new(config);
    let mut raster = RasterFieldConfig::for_raster(12, 12, 3);
    raster.learning_enabled = false;
    raster.readout_enabled = false;
    evo.attach_raster_field(raster);
    evo.set_robust_high_level_perception(false);
    evo.enable_imagination_planner(planner_config());
    evo
}

fn acquire_model(evo: &mut EvoPhase, actions: [usize; 3]) {
    let states: Vec<_> = (0..5).map(|state| raw_pair(state, 0, 0)).collect();
    let field = evo.raster_field().unwrap();
    let traces: Vec<_> = states.iter()
        .map(|raster| field.encode_relational_trace(raster).unwrap())
        .collect();
    for i in 0..traces.len() {
        for j in (i + 1)..traces.len() {
            assert!(
                traces[i].similarity(&traces[j]) < 0.97,
                "diagnostic states must not alias in the exact relational channel"
            );
        }
    }

    // Same factual raw transitions and scalar rewards in both arms. No
    // phase-plasticity update is substituted for the production G8 tuition API.
    evo.observe_planning_transition(&states[0], actions[0], &states[4], 0.25);
    evo.observe_planning_transition(&states[0], actions[1], &states[1], 0.0);
    evo.observe_planning_transition(&states[1], actions[2], &states[2], 0.0);
    evo.observe_planning_transition(&states[2], actions[0], &states[3], 1.0);
    evo.set_planning_learning_enabled(false);
    assert_eq!(evo.planning_transition_count(), 4);
    assert!(evo.branches().is_empty());
    assert_eq!(evo.recruited_relays(), 0);
}

#[test]
fn current_g8_plans_survive_zero_phase_plasticity_and_zero_dormant_capacity() {
    let permutations = [
        [0, 1, 2], [0, 2, 1], [1, 0, 2],
        [1, 2, 0], [2, 0, 1], [2, 1, 0],
    ];
    let bindings = [(1, 1), (5, 2), (6, 8)];
    let mut identical = 0;

    for actions in permutations {
        let mut ordinary = carrier(false);
        let mut disabled = carrier(true);
        acquire_model(&mut ordinary, actions);
        acquire_model(&mut disabled, actions);
        assert_eq!(disabled.config().dormant_cells, 0);
        assert_eq!(disabled.config().phase_learning_rate, 0.0);
        assert_eq!(disabled.config().weight_learning_rate, 0.0);
        assert!(!disabled.config().structural_growth_enabled);

        for (x, y) in bindings {
            let pre = raw_pair(0, x, y);
            ordinary.observe_initial_real(&pre, false);
            disabled.observe_initial_real(&pre, false);
            let ordinary_real = ordinary.current_real().unwrap().clone();
            let disabled_real = disabled.current_real().unwrap().clone();

            let normal_plan = ordinary.plan_imagined(&pre).expect("normal table route");
            let disabled_plan = disabled.plan_imagined(&pre).expect("disabled-phase table route");
            assert_eq!(normal_plan.first_action, actions[1]);
            assert_eq!(normal_plan.selected_depth, 3);
            assert_eq!(normal_plan.authority, Authority::Imagined);
            assert_eq!(disabled_plan.authority, Authority::Imagined);
            assert_eq!(signature(&normal_plan), signature(&disabled_plan));
            assert!(ordinary.branches().is_empty() && disabled.branches().is_empty());
            assert_eq!(ordinary.recruited_relays(), 0);
            assert_eq!(disabled.recruited_relays(), 0);

            for (evo, before) in [(&ordinary, &ordinary_real), (&disabled, &disabled_real)] {
                let after = evo.current_real().unwrap();
                assert_eq!(after.tick, before.tick);
                assert_eq!(after.need, before.need);
                assert_eq!(after.sensory, before.sensory);
            }
            identical += 1;
        }
    }
    assert_eq!(identical, 18);
    println!(
        "PHASE_EXECUTION_AUDIT result=NEGATIVE_PHASE_DEPENDENCY_WITNESS matched_identical={}/18 permutations=6 bindings=3 recruited_relays=0 dendritic_branches=0 table_learning=PRESENT full_physical_ownership=NOT_ESTABLISHED",
        identical
    );
}

#[test]
fn removing_the_learned_table_removes_the_plan() {
    let mut empty = carrier(true);
    empty.set_planning_learning_enabled(false);
    let pre = raw_pair(0, 1, 1);
    empty.observe_initial_real(&pre, false);
    assert_eq!(empty.planning_transition_count(), 0);
    assert!(empty.plan_imagined(&pre).is_none());
    println!("PHASE_EXECUTION_AUDIT empty_table_control=NO_PLAN");
}
