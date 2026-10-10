#![allow(dead_code)]
// FACTOR-WORLD-2 open research. No target action schedule, motor roles,
// supervised effects, factor labels or transition table given to organism.
// Raw channels encode independent visual bits at newly randomized positions.
// Training never visits a world with both key AND supply held together.
include!("intel2_unified_worlds.rs");
use std::collections::{BTreeSet as Seen, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ObjectFact {
    room: usize,
    key: bool,
    battery: bool,
    supply: bool,
    shelf_key: bool,
    shelf_supply: bool,
}
impl ObjectFact {
    fn idx(self) -> usize {
        (((((self.room * 2 + self.key as usize) * 2
            + self.battery as usize) * 2
            + self.supply as usize) * 2
            + self.shelf_key as usize) * 2) + self.shelf_supply as usize
    }
}
fn initial(key_available: bool, supply_available: bool) -> ObjectFact {
    ObjectFact {
        room: 0, key: false, battery: false, supply: false,
        shelf_key: key_available, shelf_supply: supply_available,
    }
}

struct ObjectWorld {
    sensory_indices: [usize; 8],
    motors: [usize; 6],
}
impl ObjectWorld {
    fn raw(&self, f: ObjectFact) -> Vec<f32> {
        let mut scene = vec![0.0_f32; 400];
        let bits = [
            f.room == 0, f.room == 1, f.room == 2,
            f.key, f.battery, f.supply, f.shelf_key, f.shelf_supply,
        ];
        for (i, value) in bits.into_iter().enumerate() {
            scene[self.sensory_indices[i]] = if value { 1.0 } else { 0.0 };
        }
        scene
    }
    // All private world mechanics stay only in evaluator callback.
    fn apply(&self, mut f: ObjectFact, action: usize) -> ObjectFact {
        let m = self.motors;
        if action == m[0] && f.room == 0 && f.shelf_key {
            f.key = true;
            f.shelf_key = false;
        } else if action == m[1] && f.room == 0 && f.shelf_supply {
            f.supply = true;
            f.shelf_supply = false;
        } else if action == m[2] && f.room == 0 {
            f.room = 1;
        } else if action == m[3] && f.room == 1 {
            f.room = 0;
        } else if action == m[4] && f.room == 1 {
            f.battery = true;
        } else if action == m[5] && f.room == 1 && f.key && f.battery {
            f.room = 2;
            f.battery = false;
        }
        f
    }
}
fn key_goal() -> ObjectFact {
    ObjectFact { room: 2, key: true, battery: false, supply: false,
        shelf_key: false, shelf_supply: false }
}
fn supply_goal() -> ObjectFact {
    ObjectFact { room: 1, key: false, battery: true, supply: true,
        shelf_key: false, shelf_supply: false }
}
fn composite_goal() -> ObjectFact {
    ObjectFact { room: 2, key: true, battery: false, supply: true,
        shelf_key: false, shelf_supply: false }
}
fn oracle_steps(world: &ObjectWorld, start: ObjectFact, target: ObjectFact) -> Option<usize> {
    let mut seen = Seen::new();
    let mut q = VecDeque::from([(start, 0usize)]);
    seen.insert(start);
    while let Some((f, n)) = q.pop_front() {
        if f == target { return Some(n); }
        for a in 0..6 {
            let next = world.apply(f, a);
            if seen.insert(next) { q.push_back((next, n + 1)); }
        }
    }
    None
}
#[derive(Default)]
struct ResultRecord {
    reached: bool,
    acts: usize,
    planned: usize,
    goal_plan_steps: Option<usize>,
    surprise: usize,
    blocked: usize,
    unsupported: usize,
}
fn episode(
    rt: &mut ScientificRuntime, world: &ObjectWorld,
    start: ObjectFact, target: ObjectFact,
    limit: usize, drain_once: bool, training_states: &mut Seen<ObjectFact>,
) -> ResultRecord {
    let mut fact = start;
    let goal = world.raw(target);
    rt.observe_external(&world.raw(fact)).unwrap();
    rt.set_goal(&goal).unwrap();
    let first = rt.organism().choose_phase_native_factor_goal_plan(&goal);
    let mut measure = ResultRecord {
        goal_plan_steps: first.map(|x| x.steps), ..Default::default()
    };
    let mut drained = false;
    training_states.insert(fact);
    for _ in 0..limit {
        if fact == target { break; }
        let planned = rt.organism().choose_phase_native_factor_goal_plan(&goal);
        let before = fact;
        let mut actual = None;
        let out = rt.step_unified(
            |_| Some(safe()),
            |motor| {
                actual = Some(motor);
                if drain_once && !drained && motor == world.motors[5]
                    && fact.room == 1 && fact.key && fact.supply && fact.battery
                {
                    // Hidden one-shot actuator/battery failure: gate does NOT
                    // open, battery unexpectedly drains; no change flag.
                    fact.battery = false;
                    drained = true;
                } else {
                    fact = world.apply(fact, motor);
                }
                Ok((world.raw(fact), if fact == target {1.0} else {0.0}))
            },
        );
        match out {
            Ok(StepOutcome::Executed { proposal, .. }) => {
                assert_eq!(actual, Some(proposal.action));
                measure.acts += 1;
                measure.planned += usize::from(
                    planned.is_some_and(|p| p.action == proposal.action));
                measure.surprise += usize::from(
                    before.room == 1 && before.battery && !fact.battery
                        && fact.room == 1 && before.key && before.supply);
                training_states.insert(fact);
            }
            Ok(StepOutcome::GoalReached) => break,
            Ok(StepOutcome::Blocked(_)) => { measure.blocked += 1; break; }
            _ => { measure.unsupported += 1; break; }
        }
    }
    measure.reached = fact == target;
    measure
}
#[test]
fn factor_world2_composition_battery_surprise_and_carrier_control() {
    const ARMS: usize = 4;
    const TRAIN_EACH: usize = 140;
    const LIMIT: usize = 22;
    const SEED: u64 = 0xFA67_2026_0022_0002;
    let mut rng = Rng::new(SEED);
    let mut passed = 0usize;
    for arm in 0..ARMS {
        let (evo, _) = foundation::build24();
        let mut index = (0..400usize).collect::<Vec<_>>();
        let mut motor = (0..6usize).collect::<Vec<_>>();
        shuffle(&mut rng, &mut index);
        shuffle(&mut rng, &mut motor);
        let world = ObjectWorld {
            sensory_indices: index[..8].try_into().unwrap(),
            motors: motor.try_into().unwrap(),
        };
        let mut rt = ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_factor_causality());
        assert!(rt.enable_unified_cognition(meta_checkpoint(),
            PhaseHypothesisEcologyConfig {
                learning_rate: 0.35, dormancy_threshold: 0.05,
                learning_enabled: true, phase_learning_enabled: true,
            }));
        assert_eq!(oracle_steps(&world, initial(true, true), composite_goal()), Some(5));
        let mut trained = Seen::new();
        let mut key_success = 0usize;
        let mut supply_success = 0usize;
        for i in 0..(TRAIN_EACH * 2) {
            let (start, target) = if i % 2 == 0 {
                (initial(true, false), key_goal())
            } else {
                (initial(false, true), supply_goal())
            };
            let measured = episode(&mut rt, &world, start, target, LIMIT,
                false, &mut trained);
            if measured.reached {
                if i % 2 == 0 { key_success += 1; }
                else { supply_success += 1; }
            }
        }
        assert!(trained.iter().all(|f| !(f.key && f.supply)),
            "Joint inventory was present in training");
        let rules = rt.organism().phase_native_factor_rule_count();
        rt.restart_cognition().unwrap();
        rt.set_model_learning_enabled(false);
        let meta_before = rt.organism().phase_native_meta_weights();
        rt.observe_external(&world.raw(initial(true, true))).unwrap();
        rt.set_goal(&world.raw(composite_goal())).unwrap();
        let first_plan = rt.organism().choose_phase_native_factor_goal_plan(
            &world.raw(composite_goal()));
        let physical_link = if let Some(plan) = first_plan {
            let mut lesioned = rt.organism().clone();
            let synapse = plan.evidence_synapse.unwrap();
            let saved = lesioned.perturb_phase_native_synapse_for_control(
                synapse, 0.0, 0.0).unwrap();
            let disappears = lesioned.choose_phase_native_factor_goal_plan(
                &world.raw(composite_goal())).is_none();
            lesioned.restore_phase_native_synapse_for_control(synapse, saved);
            let restored = lesioned.choose_phase_native_factor_goal_plan(
                &world.raw(composite_goal())).is_some();
            disappears && restored
        } else { false };
        let held = episode(&mut rt, &world, initial(true, true),
            composite_goal(), 12, false, &mut Seen::new());
        let frozen = meta_before == rt.organism().phase_native_meta_weights();
        rt.set_model_learning_enabled(true);
        let drift = episode(&mut rt, &world, initial(true, true),
            composite_goal(), 16, true, &mut Seen::new());
        let pass = key_success >= 100 && supply_success >= 100
            && rules >= 5 && first_plan.is_some()
            && held.reached && held.acts == 5 && held.planned >= 4
            && drift.reached && drift.surprise == 1 && drift.acts <= 12
            && frozen && physical_link && held.blocked == 0
            && held.unsupported == 0 && drift.blocked == 0 && drift.unsupported == 0;
        passed += usize::from(pass);
        println!(
            "FACTOR_WORLD2_ARM arm={} training_key={}/{} training_supply={}/{} rules={} source_plan={:?} lesion={} frozen={} composed={} composed_actions={} planned={} discharge={} replanned={} replan_actions={} blocked={} unsupported={} pass={}",
            arm, key_success, TRAIN_EACH, supply_success, TRAIN_EACH,
            rules, first_plan, physical_link, frozen, held.reached,
            held.acts, held.planned, drift.surprise, drift.reached,
            drift.acts, held.blocked + drift.blocked,
            held.unsupported + drift.unsupported, pass
        );
    }
    println!(
        "FACTOR_WORLD2_SUMMARY passed={}/{} verdict={}",
        passed, ARMS,
        if passed == ARMS { "DEVELOPMENT_PASS" } else { "DEVELOPMENT_FAIL" }
    );
    // Preserve the honest first development result; green CI != cognitive PASS.
}
