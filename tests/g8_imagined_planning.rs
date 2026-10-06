use aeterna_v1::{Authority, EvoConfig, EvoPhase, PlanningConfig, RasterFieldConfig};

fn shape(kind: usize, ox: usize, oy: usize) -> Vec<f32> {
    let points: &[(usize, usize)] = match kind {
        0 => &[(0, 0), (1, 0), (0, 1), (2, 1)], // start
        1 => &[(0, 0), (1, 0), (2, 0), (0, 2)], // route state B
        2 => &[(0, 0), (0, 1), (0, 2), (2, 0)], // route state C
        3 => &[(0, 0), (1, 1), (2, 0), (1, 2)], // goal
        4 => &[(0, 0), (1, 1), (2, 2), (0, 2)], // immediate trap
        _ => &[(0, 0), (2, 0), (0, 2), (2, 2)], // dead end
    };

    let mut raster = vec![0.0; 12 * 12];
    for (x, y) in points {
        raster[(oy + y) * 12 + (ox + x)] = 1.0;
    }
    raster
}

fn organism() -> EvoPhase {
    let mut cfg = EvoConfig::default();
    cfg.sensory_cells = 12 * 12;
    cfg.motor_cells = 4;
    cfg.dormant_cells = 96;
    cfg.hdc_dim = 192;

    let mut evo = EvoPhase::new(cfg);
    let mut raster = RasterFieldConfig::for_raster(12, 12, 4);
    raster.learning_enabled = false;
    raster.readout_enabled = false;
    evo.attach_raster_field(raster);
    // G8 isolates planning from the robust-shape containment channel. The exact\n    // relational FHRR trace remains translation-invariant but preserves the\n    // distinct intermediate states needed to measure rollout depth.\n    evo.set_robust_high_level_perception(false);

    let mut planning = PlanningConfig::new(4);
    planning.max_depth = 4;
    planning.node_budget = 64;
    planning.discount = 0.95;
    evo.enable_imagination_planner(planning);
    evo
}

fn train_local_model(evo: &mut EvoPhase, perm: [usize; 4]) {
    // Three translations provide factual local transition experience. No route
    // label or correct first action enters the planner.
    for (ox, oy) in [(0usize, 0usize), (4, 0), (0, 4)] {
        let s = shape(0, ox, oy);
        let b = shape(1, ox, oy);
        let c = shape(2, ox, oy);
        let goal = shape(3, ox, oy);
        let trap = shape(4, ox, oy);
        let dead = shape(5, ox, oy);

        // Logical meanings are evaluator-only; cognition sees opaque motor IDs.
        evo.observe_planning_transition(&s, perm[0], &trap, 0.30);
        evo.observe_planning_transition(&s, perm[1], &b, 0.0);
        evo.observe_planning_transition(&s, perm[2], &dead, 0.0);
        evo.observe_planning_transition(&b, perm[3], &c, 0.0);
        evo.observe_planning_transition(&c, perm[3], &goal, 1.0);
    }
    evo.set_planning_learning_enabled(false);
}

fn factual_execute(
    evo: &mut EvoPhase,
    perm: [usize; 4],
    origin: (usize, usize),
    depth: usize,
) -> (bool, Vec<usize>, usize) {
    let mut current_kind = 0usize;
    let mut actions = Vec::new();
    let mut expanded = 0usize;

    for _step in 0..3 {
        let sensory = shape(current_kind, origin.0, origin.1);
        let plan = evo
            .plan_imagined_depth(&sensory, depth)
            .expect("carrier must produce an imagined plan");
        assert_eq!(plan.authority, Authority::Imagined);
        expanded += plan.expanded_nodes;
        let action = plan.first_action;
        actions.push(action);

        current_kind = match (current_kind, action) {
            (0, a) if a == perm[0] => 4, // immediate reward, terminal trap
            (0, a) if a == perm[1] => 1,
            (0, _) => 5,
            (1, a) if a == perm[3] => 2,
            (1, _) => 5,
            (2, a) if a == perm[3] => 3,
            (2, _) => 5,
            (x, _) => x,
        };

        if current_kind == 3 {
            return (true, actions, expanded);
        }
        if current_kind == 4 || current_kind == 5 {
            return (false, actions, expanded);
        }
    }

    (false, actions, expanded)
}

#[test]
fn g8_imagination_selects_delayed_reward_plan_before_physical_action() {
    let permutations = [
        [0usize, 1, 2, 3],
        [2usize, 0, 3, 1],
        [1usize, 3, 0, 2],
        [3usize, 2, 1, 0],
    ];
    let heldout_origins = [(6usize, 6usize), (5usize, 7usize)];

    let mut full_ok = 0usize;
    let mut depth1_ok = 0usize;
    let mut shuffled_ok = 0usize;
    let mut total_rollout_nodes = 0usize;

    for perm in permutations {
        let mut mature = organism();
        train_local_model(&mut mature, perm);
        assert_eq!(mature.planning_transition_count(), 5);

        for origin in heldout_origins {
            // Pure imagination must not alter REAL factual state.
            let start = shape(0, origin.0, origin.1);
            mature.observe_initial_real(&start, false);
            let before = mature.current_real().expect("REAL PRE").clone();

            let decision = mature.plan_imagined(&start).expect("full imagined plan");\n            println!("G8_DECISION perm={:?} origin={:?} decision={:?}", perm, origin, decision);
            assert_eq!(decision.first_action, perm[1]);
            assert!(decision.selected_depth >= 3);
            assert!(mature.imagined_rollout_nodes() >= 2);

            let after = mature.current_real().expect("REAL must remain present");
            assert_eq!(before.sensory, after.sensory);
            assert_eq!(before.need, after.need);
            assert_eq!(before.tick, after.tick);

            let mut full = mature.clone();
            let (success, actions, nodes) = factual_execute(&mut full, perm, origin, 4);
            full_ok += usize::from(success);
            total_rollout_nodes += nodes;
            assert_eq!(actions, vec![perm[1], perm[3], perm[3]]);

            let mut depth1 = mature.clone();
            let (success1, actions1, _) = factual_execute(&mut depth1, perm, origin, 1);
            depth1_ok += usize::from(success1);
            assert_eq!(
                actions1[0], perm[0],
                "depth-1 control should take the larger immediate factual value"
            );

            let mut shuffled = mature.clone();
            shuffled.permute_planning_successors_for_control();
            let (shuffled_success, _, _) = factual_execute(&mut shuffled, perm, origin, 4);
            shuffled_ok += usize::from(shuffled_success);
        }
    }

    assert_eq!(full_ok, 8, "FULL_IMAGINATION must solve all development witness worlds");
    assert_eq!(depth1_ok, 0, "DEPTH1 must fail delayed-reward worlds");
    assert!(
        shuffled_ok <= 2,
        "shuffling learned successor bindings must materially degrade planning"
    );
    assert!(total_rollout_nodes > 0);

    println!(
        "G8_MECHANISM full={}/8 depth1={}/8 shuffled={}/8 rollout_nodes={}",
        full_ok, depth1_ok, shuffled_ok, total_rollout_nodes
    );
}
