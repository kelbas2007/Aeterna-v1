mod g7_support;

use aeterna_v1::{
    BeliefConfig, EvoConfig, EvoPhase, PlanningConfig, RasterFieldConfig,
};
use g7_support::relation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Context {
    A,
    B,
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    FullBelief,
    ObservationOnly,
    ResetHistory,
}

const CUE_A: u64 = 210_001;
const CUE_B: u64 = 210_002;
const CORRIDOR: u64 = 210_003;
const GOAL: u64 = 210_004;
const DEAD: u64 = 210_005;

fn organism(with_belief: bool) -> EvoPhase {
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
    evo.set_robust_high_level_perception(false);

    let mut planning = PlanningConfig::new(4);
    planning.max_depth = 3;
    planning.node_budget = 128;
    planning.discount = 0.95;
    evo.enable_imagination_planner(planning);

    if with_belief {
        evo.enable_belief_state(BeliefConfig::new(4, 192));
    }

    evo
}

fn cue(context: Context, origin: (usize, usize)) -> Vec<f32> {
    relation(
        match context {
            Context::A => CUE_A,
            Context::B => CUE_B,
        },
        origin.0,
        origin.1,
    )
}

fn corridor(origin: (usize, usize)) -> Vec<f32> {
    relation(CORRIDOR, origin.0, origin.1)
}

fn goal(origin: (usize, usize)) -> Vec<f32> {
    relation(GOAL, origin.0, origin.1)
}

fn dead(origin: (usize, usize)) -> Vec<f32> {
    relation(DEAD, origin.0, origin.1)
}

fn correct_terminal(context: Context, perm: [usize; 4]) -> usize {
    match context {
        Context::A => perm[1],
        Context::B => perm[2],
    }
}

fn factual_post(
    context: Context,
    perm: [usize; 4],
    stage: usize,
    action: usize,
    origin: (usize, usize),
) -> (Vec<f32>, f32) {
    match stage {
        0 => {
            if action == perm[0] {
                (corridor(origin), 0.0)
            } else {
                (dead(origin), 0.0)
            }
        }
        1 => {
            if action == correct_terminal(context, perm) {
                (goal(origin), 1.0)
            } else {
                (dead(origin), 0.0)
            }
        }
        _ => unreachable!(),
    }
}

fn train(mode: Mode, perm: [usize; 4]) -> EvoPhase {
    let with_belief = !matches!(mode, Mode::ObservationOnly);
    let mut evo = organism(with_belief);
    let origins = [(0usize, 0usize), (4, 0), (0, 4), (4, 4)];

    // Fixed factual curriculum. Every opaque action is tried from both stages
    // under both contexts. The evaluator context is used only to produce factual
    // POST/Need and never enters the carrier.
    for context in [Context::A, Context::B] {
        for origin in origins {
            let pre = cue(context, origin);
            let amb = corridor(origin);

            for action in 0..4usize {
                let (post, reward) = factual_post(context, perm, 0, action, origin);
                match mode {
                    Mode::ObservationOnly => {
                        evo.observe_planning_transition(&pre, action, &post, reward);
                    }
                    Mode::FullBelief | Mode::ResetHistory => {
                        evo.reset_belief(&pre).expect("belief PRE");
                        evo.observe_belief_planning_transition(action, &post, reward)
                            .expect("belief transition");
                    }
                }
            }

            for action in 0..4usize {
                let (post, reward) = factual_post(context, perm, 1, action, origin);
                match mode {
                    Mode::ObservationOnly => {
                        evo.observe_planning_transition(&amb, action, &post, reward);
                    }
                    Mode::ResetHistory => {
                        // Deliberately erase the cue history before learning the
                        // ambiguous corridor decision.
                        evo.reset_belief(&amb).expect("reset-history corridor");
                        evo.observe_belief_planning_transition(action, &post, reward)
                            .expect("reset-history terminal transition");
                    }
                    Mode::FullBelief => {
                        // Reconstruct the factual prefix for every terminal probe,
                        // so all actions are learned from the same history-conditioned
                        // ambiguous corridor belief.
                        evo.reset_belief(&pre).expect("belief cue");
                        evo.advance_belief(perm[0], &amb)
                            .expect("history-conditioned corridor");
                        evo.observe_belief_planning_transition(action, &post, reward)
                            .expect("belief terminal transition");
                    }
                }
            }
        }
    }

    evo.set_planning_learning_enabled(false);
    evo
}

fn corridor_belief(
    mature: &EvoPhase,
    context: Context,
    perm: [usize; 4],
    origin: (usize, usize),
) -> aeterna_v1::CarrierTrace {
    let mut evo = mature.clone();
    let pre = cue(context, origin);
    let amb = corridor(origin);
    evo.reset_belief(&pre).expect("belief cue");
    evo.advance_belief(perm[0], &amb)
        .expect("belief corridor")
}

fn execute(
    mature: &EvoPhase,
    mode: Mode,
    context: Context,
    perm: [usize; 4],
    origin: (usize, usize),
) -> (bool, Vec<usize>) {
    let mut evo = mature.clone();
    let pre = cue(context, origin);
    let amb = corridor(origin);
    let mut actions = Vec::new();

    let first = match mode {
        Mode::ObservationOnly => evo
            .plan_imagined(&pre)
            .expect("observation planner at cue")
            .first_action,
        Mode::FullBelief | Mode::ResetHistory => {
            evo.reset_belief(&pre).expect("belief cue");
            evo.plan_from_belief()
                .expect("belief planner at cue")
                .first_action
        }
    };
    actions.push(first);

    if first != perm[0] {
        return (false, actions);
    }

    let second = match mode {
        Mode::ObservationOnly => evo
            .plan_imagined(&amb)
            .expect("observation planner at ambiguous corridor")
            .first_action,
        Mode::FullBelief => {
            evo.advance_belief(first, &amb)
                .expect("advance full belief");
            evo.plan_from_belief()
                .expect("history-conditioned corridor plan")
                .first_action
        }
        Mode::ResetHistory => {
            evo.reset_belief(&amb)
                .expect("erase history at corridor");
            evo.plan_from_belief()
                .expect("reset-history corridor plan")
                .first_action
        }
    };
    actions.push(second);

    (second == correct_terminal(context, perm), actions)
}

#[test]
fn g9_history_conditioned_belief_resolves_identical_current_observation() {
    let permutations = [
        [0usize, 1, 2, 3],
        [2usize, 0, 3, 1],
        [1usize, 3, 0, 2],
        [3usize, 2, 1, 0],
    ];
    let heldout_origins = [(6usize, 6usize), (5, 7), (7, 5), (6, 7)];

    let mut full_ok = 0usize;
    let mut observation_ok = 0usize;
    let mut reset_ok = 0usize;
    let mut successful_terminal_ids = std::collections::BTreeSet::new();

    for (perm_index, perm) in permutations.into_iter().enumerate() {
        let full = train(Mode::FullBelief, perm);
        let observation = train(Mode::ObservationOnly, perm);
        let reset = train(Mode::ResetHistory, perm);
        let origin = heldout_origins[perm_index];

        // At the aliased decision point the raw raster is literally identical.
        assert_eq!(
            corridor(origin),
            corridor(origin),
            "both hidden contexts must expose the same current observation"
        );

        let belief_a = corridor_belief(&full, Context::A, perm, origin);
        let belief_b = corridor_belief(&full, Context::B, perm, origin);
        assert!(
            belief_a.similarity(&belief_b) < 0.97,
            "carrier-owned history must separate the two latent situations despite identical current observation"
        );

        for context in [Context::A, Context::B] {
            let (g_success, g_actions) =
                execute(&full, Mode::FullBelief, context, perm, origin);
            let (o_success, _) =
                execute(&observation, Mode::ObservationOnly, context, perm, origin);
            let (r_success, _) =
                execute(&reset, Mode::ResetHistory, context, perm, origin);

            full_ok += usize::from(g_success);
            observation_ok += usize::from(o_success);
            reset_ok += usize::from(r_success);

            if g_success {
                successful_terminal_ids.insert(g_actions[1]);
            }
        }
    }

    println!(
        "G9_MECHANISM full={}/8 observation_only={}/8 reset_history={}/8 terminal_ids={:?}",
        full_ok, observation_ok, reset_ok, successful_terminal_ids
    );

    assert_eq!(full_ok, 8, "FULL_BELIEF must solve all aliased held-out episodes");
    assert!(
        observation_ok <= 4,
        "memoryless observation-only planning must lose the hidden-history distinction"
    );
    assert!(
        reset_ok <= 4,
        "resetting recurrent belief at every observation must remove the history advantage"
    );
    assert!(
        successful_terminal_ids.len() >= 3,
        "success must follow opaque motor permutations rather than a fixed terminal ID"
    );
}
