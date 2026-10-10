use aeterna_v1::scientific_runtime::{
    ScientificRuntime, StepOutcome, RuntimeError,
};
use aeterna_v1::{EvoPhase, HumanProtectionEvidence, HumanProtectionReason};
use std::cell::Cell;

// Reuse the already audited G19 representation/source-drive fixture. The
// imported historical tests remain intact and its fresh test stays ignored.
// G20 outcomes below are NOT counted as new independent G19/fresh worlds.
#[allow(dead_code)]
mod fixture {
    include!("g19_rival_hypothesis_discrimination.rs");

    pub struct World {
        pub state: usize,
        pub changed: bool,
        pub calls: usize,
        pub motor_mask: u8,
        l1: [[usize; 2]; 8],
        r: Roles,
    }

    impl World {
        fn next(&self, state: usize, action: usize) -> usize {
            match state {
                0 if action == self.r.probe_a => if self.changed { 2 } else { 1 },
                0 if action == self.r.probe_b => 3,
                0 if action == self.r.fallback_a => 5,
                0 if action == self.r.fallback_b => 6,
                1 if action == self.r.step_a => 7,
                3 if action == self.r.step_b => 8,
                5 if action == self.r.step_a => 1,
                6 if action == self.r.step_b => 3,
                2 if action == self.r.step_a => 0,
                4 if action == self.r.step_b => 0,
                7 if action == self.r.fallback_a => 0,
                8 if action == self.r.fallback_b => 0,
                _ => state,
            }
        }
        pub fn sense(&self) -> Vec<f32> {
            state_scene(&self.l1, self.state, LAYOUTS[4 + self.calls % 2])
        }
        pub fn goal(&self, a: bool) -> Vec<f32> {
            state_scene(&self.l1, if a { 7 } else { 8 }, LAYOUTS[5])
        }
        pub fn at_goal(&self, a: bool) -> bool {
            self.state == if a { 7 } else { 8 }
        }
        pub fn execute(&mut self, action: usize) -> Result<Vec<f32>, String> {
            self.calls += 1;
            self.motor_mask |= 1 << action;
            self.state = self.next(self.state, action);
            Ok(self.sense())
        }
        pub fn contradicted_a_link(&self, evo: &EvoPhase) -> PhaseCircuitInfo {
            transition_circuit(
                evo, state_ref(evo, &self.l1, 0), self.r.probe_a,
                state_ref(evo, &self.l1, 1),
            )
        }
    }

    pub fn prepare(swap: bool) -> (EvoPhase, World) {
        let drive = train_drive();
        let mut evo = target(&drive);
        let l1 = train_abstraction(&mut evo);
        let world = World {
            state: 0, changed: false, calls: 0, motor_mask: 0,
            l1, r: roles(swap),
        };
        // One prior learning history only. Every route and return transition is
        // factual. Opposing prior observations supply the inherited two rivals.
        for pre in 0..9usize {
            for action in 0..6usize {
                learn(&mut evo, &l1, pre, action, world.next(pre, action));
            }
        }
        learn(&mut evo, &l1, 0, world.r.probe_a, 2);
        learn(&mut evo, &l1, 0, world.r.probe_b, 4);
        assert_eq!(evo.planning_transition_count(), 0);
        (evo, world)
    }
}

fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: true, physical_effect_possible: true,
        predicted_harm_probability: 0.0, hazard_confidence: 1.0,
        emergency_stop: false,
    }
}

fn structure(evo: &EvoPhase) -> Vec<(usize, usize, usize, usize, usize)> {
    evo.phase_native_circuits().iter().map(|c| (
        c.relay_cell, c.afferent_synapse, c.successor_synapse,
        c.outcome_synapse, c.motor_synapse,
    )).collect()
}

fn assert_blocked_without_facts(
    runtime: &mut ScientificRuntime,
    evidence: Option<HumanProtectionEvidence>,
    reason: HumanProtectionReason,
) {
    let fingerprint = runtime.organism().phase_native_learned_fingerprint();
    let before = runtime.organism().current_real().unwrap().clone();
    let calls = Cell::new(0);
    let outcome = runtime.step(
        |_| evidence,
        |_| { calls.set(calls.get() + 1); Err("must never run".into()) },
    ).unwrap();
    let StepOutcome::Blocked(record) = outcome else { panic!("expected block"); };
    assert_eq!(record.reason, reason);
    assert_eq!(calls.get(), 0);
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fingerprint);
    let after = runtime.organism().current_real().unwrap();
    assert_eq!(after.sensory, before.sensory);
    assert_eq!(after.need, before.need);
    assert_eq!(after.tick, before.tick);
}

fn safety_interruption(runtime: &mut ScientificRuntime, world: &fixture::World) {
    assert_blocked_without_facts(runtime, Some(HumanProtectionEvidence {
        predicted_harm_probability: 0.5, ..safe()
    }), HumanProtectionReason::ExcessHumanHarmRisk);
    assert_blocked_without_facts(runtime, None, HumanProtectionReason::InvalidEvidence);
    assert_blocked_without_facts(runtime, Some(HumanProtectionEvidence {
        emergency_stop: true, ..safe()
    }), HumanProtectionReason::EmergencyStopSignal);
    let fingerprint = runtime.organism().phase_native_learned_fingerprint();
    let sequence = runtime.sequence();
    runtime.restart_cognition().unwrap();
    assert!(runtime.emergency_latched());
    assert!(runtime.organism().current_real().is_none());
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fingerprint);
    assert!(runtime.sequence() > sequence);
    runtime.observe_external(&world.sense()).unwrap();
    assert_blocked_without_facts(runtime, Some(safe()),
        HumanProtectionReason::EmergencyStopLatched);
    runtime.external_operator_reset();
    assert!(!runtime.emergency_latched());
}

#[derive(Debug)]
struct LifetimeResult {
    completed: usize,
    costs: Vec<usize>,
    changed_revision: bool,
    motor_mask: u8,
}

fn run_lifetime(swap: bool, freeze_after_four: bool) -> LifetimeResult {
    let (evo, mut world) = fixture::prepare(swap);
    let original_structure = structure(&evo);
    let drive = evo.phase_native_drive_weights();
    let mut runtime = ScientificRuntime::new(evo).unwrap();
    runtime.observe_external(&world.sense()).unwrap();
    let mut completed = 0usize;
    let mut costs = Vec::new();
    let mut changed_revision = false;

    for task in 0..8usize {
        let goal_a = task % 2 == 0;
        if task == 4 {
            // Only the environment changes. There is no change flag, rehearsal
            // or model repair supplied to the runtime.
            world.changed = true;
            if freeze_after_four { runtime.set_model_learning_enabled(false); }
        }
        runtime.set_goal(&world.goal(goal_a)).unwrap();
        if task == 1 && !freeze_after_four { safety_interruption(&mut runtime, &world); }
        if task == 6 {
            let fingerprint = runtime.organism().phase_native_learned_fingerprint();
            runtime.restart_cognition().unwrap();
            assert!(matches!(runtime.propose(), Err(RuntimeError::FreshObservationRequired)));
            assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fingerprint);
            runtime.observe_external(&world.sense()).unwrap();
        }
        let before = world.calls;
        let mut success = false;
        for _ in 0..16usize {
            if runtime.goal_reached().unwrap() { success = true; break; }
            let result = runtime.step(|_| Some(safe()), |action| world.execute(action));
            match result {
                Ok(StepOutcome::Executed { suppressed, .. }) => {
                    if task == 4 && suppressed > 0 { changed_revision = true; }
                }
                Ok(StepOutcome::GoalReached) => { success = true; break; }
                other => panic!("unexpected lifetime interruption: {:?}", other),
            }
        }
        if runtime.goal_reached().unwrap() { success = true; }
        let cost = world.calls - before;
        costs.push(cost);
        println!("G20_TASK swap={} frozen={} task={} goal_a={} success={} cost={} state={}",
            swap, freeze_after_four, task, goal_a, success, cost, world.state);
        assert_eq!(success, world.at_goal(goal_a), "score must be factual goal arrival");
        assert_eq!(runtime.organism().phase_native_drive_weights(), drive);
        assert_eq!(runtime.organism().planning_transition_count(), 0);
        assert!(runtime.organism().composite_concepts().is_empty());
        assert_eq!(structure(runtime.organism()), original_structure);
        assert!(runtime.audit().iter().zip(runtime.audit().iter().skip(1))
            .all(|(a, b)| a.sequence < b.sequence));
        if success { completed += 1; } else { break; }
    }
    if !freeze_after_four && completed == 8 {
        let old = world.contradicted_a_link(runtime.organism());
        assert!(old.revision > 0 && !old.counterexamples.is_empty());
        assert_eq!(runtime.organism().phase_native_synapse(old.successor_synapse).unwrap().weight, 0.0);
    }
    LifetimeResult { completed, costs, changed_revision, motor_mask: world.motor_mask }
}

#[test]
fn g20_one_lifetime_reasons_revises_retains_and_restarts_with_protection() {
    let mut total = 0;
    let mut mask = 0u8;
    for swap in [false, true] {
        let full = run_lifetime(swap, false);
        let frozen = run_lifetime(swap, true);
        println!("G20_LIFETIME swap={} full={:?} frozen={:?}", swap, full, frozen);
        assert_eq!(full.completed, 8);
        assert!(full.changed_revision);
        assert!(full.costs.iter().all(|cost| *cost <= 16));
        assert_eq!(frozen.completed, 4, "frozen model must fail first changed-A task");
        total += full.completed;
        mask |= full.motor_mask;
    }
    println!("G20_RESULT actual_goals={}/16 motor_mask={:#08b} safety=PASS restart=PASS frozen_changed=0/2", total, mask);
    assert_eq!(total, 16);
    assert_eq!(mask, 0b11_1111);
}

#[test]
fn g20_fault_and_invalid_inputs_cannot_create_facts_or_blind_retry() {
    let (evo, world) = fixture::prepare(false);
    let mut runtime = ScientificRuntime::new(evo).unwrap();
    runtime.observe_external(&world.sense()).unwrap();
    let fingerprint = runtime.organism().phase_native_learned_fingerprint();
    assert_eq!(runtime.set_goal(&[0.0]), Err(RuntimeError::InvalidRaster));
    assert_eq!(runtime.observe_external(&vec![f32::NAN; 400]), Err(RuntimeError::InvalidRaster));
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fingerprint);
    runtime.set_goal(&world.goal(true)).unwrap();
    let calls = Cell::new(0);
    let result = runtime.step(|_| Some(safe()), |_| {
        calls.set(calls.get() + 1);
        Err("simulated actuator I/O failure".into())
    }).unwrap();
    assert!(matches!(result, StepOutcome::ExecutionFault(_)));
    assert_eq!(calls.get(), 1);
    assert!(runtime.emergency_latched());
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fingerprint);
    let blocked = runtime.step(|_| Some(safe()), |_| {
        calls.set(calls.get() + 1); Ok(world.sense())
    }).unwrap();
    assert!(matches!(blocked, StepOutcome::Blocked(_)));
    assert_eq!(calls.get(), 1);
    runtime.external_operator_reset();
    assert!(matches!(runtime.propose(), Err(RuntimeError::FreshObservationRequired)));
}

#[test]
fn g20_runtime_delegates_reasoning_and_seals_every_execution() {
    let source = include_str!("../src/scientific_runtime.rs");
    for forbidden in ["STATE_PAIRS", "correct_action", "world.next", "EvoImaginationPlanner",
        "BinaryHeap", "HashMap", "_for_control("] {
        assert!(!source.contains(forbidden), "runtime bypass/token: {}", forbidden);
    }
    for required in ["choose_phase_native_goal_rival_probe", "choose_phase_native_goal_active_action",
        "observe_phase_native_rival_probe_result", "self.protection.screen", "consume_permit(permit)"] {
        assert!(source.contains(required), "missing integration boundary: {}", required);
    }
    let consumption = source.find("self.protection.consume_permit(permit)").unwrap();
    let execution = source.find("match execute(action)").unwrap();
    assert!(consumption < execution);
    assert!(!source.contains("pub fn organism_mut"));
}

#[test]
fn native_checkpoint_restoration_preserves_external_safety_and_rejects_wrong_shape_atomically() {
    let (evo,world)=fixture::prepare(false);
    let mut runtime=ScientificRuntime::new(evo).unwrap();
    runtime.observe_external(&world.sense()).unwrap();
    runtime.set_goal(&world.goal(true)).unwrap();
    let mut emergency=safe();emergency.emergency_stop=true;
    let result=runtime.step(|_|Some(emergency), |_|panic!("blocked motor executed")).unwrap();
    assert!(matches!(result,StepOutcome::Blocked(_)));
    assert!(runtime.emergency_latched());
    let checkpoint=runtime.organism().phase_native_checkpoint().unwrap();
    let fingerprint=runtime.organism().phase_native_learned_fingerprint();
    let sequence=runtime.sequence();
    runtime.restore_native_checkpoint(checkpoint).unwrap();
    assert!(runtime.emergency_latched());
    assert!(runtime.sequence()>sequence);
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(),fingerprint);
    assert!(matches!(runtime.propose(),Err(RuntimeError::FreshObservationRequired)));
    let mut wrong=EvoPhase::new(aeterna_v1::EvoConfig {sensory_cells:8,motor_cells:2,
        dormant_cells:8,..aeterna_v1::EvoConfig::default()});
    wrong.enable_phase_native_planning(aeterna_v1::carrier::PhaseNativeConfig::default());
    let sequence=runtime.sequence();
    assert!(matches!(runtime.restore_native_checkpoint(wrong.phase_native_checkpoint().unwrap()),
        Err(RuntimeError::InvalidCheckpoint)));
    assert_eq!(runtime.sequence(),sequence);
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(),fingerprint);
    runtime.observe_external(&world.sense()).unwrap();
    let result=runtime.step(|_|Some(safe()), |_|panic!("checkpoint cleared safety latch")).unwrap();
    assert!(matches!(result,StepOutcome::Blocked(_)));
}
