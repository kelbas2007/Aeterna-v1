#![allow(dead_code)]
// FACTOR-WORLD-1: OPEN negative/positive diagnostic, NOT a frozen authority test.
// Same organism acquires opaque actions in two disjoint physical contexts.
// Key and supply are never simultaneously available in training. Heldout
// requires their composition and unseen (room,key,power,supply) tuples.
// Evaluator never supplies a transition, motor role, feature or plan to EvoPhase.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;
use std::collections::{BTreeSet as FactorSeen, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fact {
    room: usize,
    key: bool,
    power: bool,
    supply: bool,
}
impl Fact {
    fn index(self) -> usize {
        self.room * 8 + usize::from(self.key) * 4
            + usize::from(self.power) * 2 + usize::from(self.supply)
    }
}
#[derive(Clone)]
struct FactorWorld {
    states: [usize; 24],
    motors: [usize; 6],
    key_available: bool,
    supply_available: bool,
    drain_on_entry: bool,
}
impl FactorWorld {
    fn sensory(&self, l1: &[[usize; 2]; 8], f: Fact, layout: usize) -> Vec<f32> {
        foundation::scene(l1, self.states[f.index()], layout)
    }
    fn apply(&self, mut f: Fact, motor: usize) -> Fact {
        let m = self.motors;
        if motor == m[0] && f.room == 0 && self.key_available {
            f.key = true;
        } else if motor == m[1] && f.room == 0 && self.supply_available {
            f.supply = true;
        } else if motor == m[2] && f.room == 0 {
            f.room = 1;
            if self.drain_on_entry { f.power = false; }
        } else if motor == m[3] && f.room == 1 {
            f.room = 0;
        } else if motor == m[4] && f.room == 1 {
            f.power = true;
        } else if motor == m[5] && f.room == 1 && f.key && f.power {
            f.room = 2;
            f.power = false;
        }
        f
    }
}
const START: Fact = Fact { room: 0, key: false, power: false, supply: false };
const KEY_GOAL: Fact = Fact { room: 2, key: true, power: false, supply: false };
const SUPPLY_GOAL: Fact = Fact { room: 1, key: false, power: true, supply: true };
const COMPOSED_GOAL: Fact = Fact { room: 2, key: true, power: false, supply: true };

#[derive(Default)]
struct Sample {
    reached: bool,
    steps: usize,
    blocked: usize,
    unsupported: usize,
    planned: usize,
    unexpected: usize,
    composite_visits: usize,
}
fn lifetime_episode(
    rt: &mut ScientificRuntime, l1: &[[usize; 2]; 8],
    world: &FactorWorld, episode: usize, start: Fact, target: Fact,
    budget: usize, seen: &mut FactorSeen<usize>,
) -> Sample {
    let mut f = start;
    let goal = world.sensory(l1, target, (episode + 1) % 6);
    rt.observe_external(&world.sensory(l1, f, episode % 6)).unwrap();
    rt.set_goal(&goal).unwrap();
    let mut result = Sample::default();
    seen.insert(f.index());
    for t in 0..budget {
        if f == target { break; }
        let before = f;
        let proposed = rt.organism().choose_phase_native_temporal_goal_plan(&goal);
        let mut executed = None;
        let outcome = rt.step_unified(
            |_| Some(safe()),
            |motor| {
                executed = Some(motor);
                f = world.apply(f, motor);
                Ok((world.sensory(l1, f, (episode + t + 1) % 6),
                    if f == target { 1.0 } else { 0.0 }))
            },
        );
        match outcome {
            Ok(StepOutcome::Executed { proposal, .. }) => {
                assert_eq!(executed, Some(proposal.action));
                result.steps += 1;
                result.planned += usize::from(
                    proposed.is_some_and(|p| p.action == proposal.action));
                result.unexpected += usize::from(
                    proposed.is_some_and(|p| p.action == proposal.action) && f == before);
                result.composite_visits += usize::from(f.key && f.supply);
                seen.insert(f.index());
            }
            Ok(StepOutcome::GoalReached) => break,
            Ok(StepOutcome::Blocked(_)) => { result.blocked += 1; break; }
            _ => { result.unsupported += 1; break; }
        }
    }
    result.reached = f == target;
    result
}

// Evaluator-only feasibility control. Never exported into the organism.
fn oracle_length(world: &FactorWorld, start: Fact, goal: Fact) -> Option<usize> {
    let mut explored = FactorSeen::new();
    let mut q = VecDeque::from([(start, 0usize)]);
    explored.insert(start.index());
    while let Some((f, depth)) = q.pop_front() {
        if f == goal { return Some(depth); }
        for motor in 0..6 {
            let next = world.apply(f, motor);
            if explored.insert(next.index()) {
                q.push_back((next, depth + 1));
            }
        }
    }
    None
}

#[test]
fn factor_world1_first_compositional_transfer_diagnostic() {
    const SEED: u64 = 0xFA6C_2026_0101_0001;
    const ARMS: usize = 4;
    const TRAIN_PER_CONTEXT: usize = 160;
    const TRAIN_BUDGET: usize = 18;
    let mut rng = Rng::new(SEED);
    let mut success = 0usize;
    let mut causal_plans = 0usize;
    let mut split_valid = 0usize;
    for arm in 0..ARMS {
        let (evo, l1) = foundation::build24();
        let mut state_labels: Vec<usize> = (0..24).collect();
        let mut motor_labels: Vec<usize> = (0..6).collect();
        shuffle(&mut rng, &mut state_labels);
        shuffle(&mut rng, &mut motor_labels);
        let prototype = FactorWorld {
            states: state_labels.try_into().unwrap(),
            motors: motor_labels.try_into().unwrap(),
            key_available: true,
            supply_available: false,
            drain_on_entry: false,
        };
        let key_context = prototype.clone();
        let supply_context = FactorWorld {
            key_available: false, supply_available: true, ..prototype.clone()
        };
        let composed = FactorWorld {
            key_available: true, supply_available: true, ..prototype.clone()
        };
        assert_eq!(oracle_length(&composed, START, COMPOSED_GOAL), Some(5));
        let mut rt = ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_temporal_evidence(PhaseTemporalEvidenceConfig {
            max_observations: 8, minimum_observations: 3, decisive_margin: 0.125,
        }));
        assert!(rt.set_temporal_multistep(true));
        assert!(rt.set_temporal_goal_replanning(true));
        assert!(rt.set_temporal_cold_goal_acquisition(true));
        assert!(rt.enable_unified_cognition(meta_checkpoint(),
            PhaseHypothesisEcologyConfig {
                learning_rate: 0.35, dormancy_threshold: 0.05,
                learning_enabled: true, phase_learning_enabled: true,
            }));
        let mut training_seen = FactorSeen::new();
        let mut key_goals = 0usize;
        let mut supply_goals = 0usize;
        // Interleaved task contexts, same continuing organism and weights.
        // Only goal + factual POST and reward are available to EvoPhase.
        for ep in 0..(TRAIN_PER_CONTEXT * 2) {
            let (world, goal) = if ep % 2 == 0 {
                (&key_context, KEY_GOAL)
            } else {
                (&supply_context, SUPPLY_GOAL)
            };
            let sample = lifetime_episode(
                &mut rt, &l1, world, ep, START, goal,
                TRAIN_BUDGET, &mut training_seen,
            );
            if sample.reached {
                if ep % 2 == 0 { key_goals += 1; }
                else { supply_goals += 1; }
            }
        }
        let proper_split = training_seen.iter().all(|&idx| !(idx % 8 >= 5 && idx % 8 % 2 == 1))
            && !training_seen.contains(&COMPOSED_GOAL.index());
        // More explicit independent check: no key && supply tuple appeared.
        let no_joint_tuple = training_seen.iter().all(|&idx| !(idx & 4 != 0 && idx & 1 != 0));
        assert!(no_joint_tuple, "heldout joint features leaked into training");
        assert!(proper_split);
        split_valid += 1;

        rt.restart_cognition().unwrap();
        rt.set_model_learning_enabled(false);
        let retained = rt.organism().phase_native_meta_weights();
        let goal = composed.sensory(&l1, COMPOSED_GOAL, 0);
        rt.observe_external(&composed.sensory(&l1, START, 0)).unwrap();
        rt.set_goal(&goal).unwrap();
        let start_plan = rt.organism().choose_phase_native_temporal_goal_plan(&goal);
        let mut eval_seen = FactorSeen::new();
        let heldout = lifetime_episode(&mut rt, &l1, &composed,
            9000 + arm, START, COMPOSED_GOAL, 14, &mut eval_seen);
        let frozen = retained == rt.organism().phase_native_meta_weights();
        let plan_supported = start_plan.is_some();
        causal_plans += usize::from(plan_supported);
        let pass = frozen && heldout.reached && plan_supported
            && heldout.planned >= 3
            && heldout.blocked == 0 && heldout.unsupported == 0;
        success += usize::from(pass);
        println!(
            "FACTOR_WORLD1_ARM arm={} key_train={}/{} supply_train={}/{} learned_edges={} train_joint=0 oracle_steps=5 start_plan={:?} frozen={} goal={} heldout_steps={} planned={} new_joint_states={} blocked={} unsupported={} pass={}",
            arm, key_goals, TRAIN_PER_CONTEXT, supply_goals, TRAIN_PER_CONTEXT,
            rt.organism().phase_native_temporal_transition_count(),
            start_plan, frozen, heldout.reached, heldout.steps,
            heldout.planned, heldout.composite_visits,
            heldout.blocked, heldout.unsupported, pass,
        );
    }
    let pass = success == ARMS && split_valid == ARMS;
    println!(
        "FACTOR_WORLD1_SUMMARY causal_start_plans={}/{} composed_goals={}/{} split_valid={}/{} verdict={}",
        causal_plans, ARMS, success, ARMS, split_valid, ARMS,
        if pass { "DEVELOPMENT_PASS" } else { "DEVELOPMENT_FAIL" },
    );
    // Explicitly preserve first diagnostic FAIL; do not turn it green by
    // asserting a requested cognitive outcome before the architecture exists.
}
