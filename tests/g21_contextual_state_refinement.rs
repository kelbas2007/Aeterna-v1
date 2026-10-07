use aeterna_v1::{EvoPhase, HumanProtectionEvidence};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome, RuntimeError};
use std::cell::Cell;

// Inherited perceptual/source-drive tuition only. The NEW contextual distinction
// is never taught by a tuple list or by a hidden context label.
#[allow(dead_code)]
mod fixture {
    include!("g19_rival_hypothesis_discrimination.rs");

    pub struct World {
        pub state: usize,
        pub context: bool,
        pub calls: usize,
        pub scoring: bool,
        pub decision_visits: usize,
        pub swap: bool,
        l1: [[usize; 2]; 8],
        rng: u64,
        pair_first: bool,
        cue_visits: usize,
    }
    impl World {
        pub fn raw(&self, state: usize) -> Vec<f32> {
            let index = if self.scoring { 4 + self.calls % 2 } else { self.calls % 2 };
            state_scene(&self.l1, state, LAYOUTS[index])
        }
        pub fn sense(&self) -> Vec<f32> { self.raw(self.state) }
        pub fn requested_goal(&self) -> Vec<f32> { self.raw(7) }
        pub fn home_goal(&self) -> Vec<f32> { self.raw(5) }
        pub fn expected(&self) -> usize {
            if self.context ^ self.swap { 5 } else { 4 }
        }
        fn next_context(&mut self) {
            if self.cue_visits % 2 == 0 {
                self.rng ^= self.rng >> 12;
                self.rng ^= self.rng << 25;
                self.rng ^= self.rng >> 27;
                self.pair_first = self.rng.wrapping_mul(0x2545F4914F6CDD1D) >> 63 == 1;
                self.context = self.pair_first;
            } else { self.context = !self.pair_first; }
            self.cue_visits += 1;
        }
        pub fn execute(&mut self, action: usize) -> Result<Vec<f32>, String> {
            assert!(action < 6);
            self.calls += 1;
            self.state = match self.state {
                0 => {
                    self.decision_visits += 1;
                    if action == self.expected() { 7 } else { 8 }
                }
                1 | 2 => 0,
                5 => { self.next_context(); if self.context { 2 } else { 1 } },
                7 | 8 => 5,
                other => other,
            };
            Ok(self.sense())
        }
    }

    pub fn prepare(swap: bool) -> (EvoPhase, World) {
        let drive = train_drive();
        let mut evo = target(&drive);
        let l1 = train_abstraction(&mut evo);
        let world = World {
            state: 1, context: false, calls: 0, scoring: false,
            decision_visits: 0, swap, l1,
            rng: if swap { 113 } else { 37 }, pair_first: false, cue_visits: 0,
        };
        // Pre-existing coarse model is correct for one history only. It has no
        // contextual cells or answer table and has never seen the new conflict.
        for pre in 0..9usize {
            for action in 0..6usize {
                let post = match pre {
                    0 => if action == world.expected() { 7 } else { 8 },
                    1 | 2 => 0,
                    5 => 1,
                    7 | 8 => 5,
                    other => other,
                };
                learn(&mut evo, &l1, pre, action, post);
            }
        }
        for action in 0..6usize { learn(&mut evo, &l1, 5, action, 2); }
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

fn set_current_goal(runtime: &mut ScientificRuntime, world: &fixture::World) {
    // Uniform operational requests: leave an already reached target for the
    // common staging point, otherwise seek the same good raw observation.
    // No context-specific goal is ever supplied at the ambiguous junction.
    runtime.set_goal(&if world.state == 7 { world.home_goal() }
        else { world.requested_goal() }).unwrap();
}

fn execute_step(runtime: &mut ScientificRuntime, world: &mut fixture::World) -> usize {
    set_current_goal(runtime, world);
    let result = runtime.step(|_| Some(safe()), |a| world.execute(a));
    match result {
        Ok(StepOutcome::Executed { proposal, .. }) => proposal.action,
        other => panic!("unexpected protected lifetime step at state {}: {:?}", world.state, other),
    }
}

fn protected_restart(runtime: &mut ScientificRuntime, world: &fixture::World) {
    let fp = runtime.organism().phase_native_learned_fingerprint();
    let facts = runtime.organism().current_real().unwrap().clone();
    let calls = Cell::new(0);
    let blocked = runtime.step(|_| Some(HumanProtectionEvidence {
        emergency_stop: true, ..safe()
    }), |_| { calls.set(calls.get() + 1); Ok(world.sense()) }).unwrap();
    assert!(matches!(blocked, StepOutcome::Blocked(_)));
    assert_eq!(calls.get(), 0);
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fp);
    assert_eq!(runtime.organism().current_real().unwrap().sensory, facts.sensory);
    runtime.restart_cognition().unwrap();
    assert!(runtime.emergency_latched());
    assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fp);
    assert!(matches!(runtime.propose(), Err(RuntimeError::FreshObservationRequired)));
    runtime.observe_external(&world.sense()).unwrap();
    let still_blocked = runtime.step(|_| Some(safe()), |_| {
        calls.set(calls.get() + 1); Ok(world.sense())
    }).unwrap();
    assert!(matches!(still_blocked, StepOutcome::Blocked(_)));
    assert_eq!(calls.get(), 0);
    runtime.external_operator_reset();
}

fn causal_checks(evo: &EvoPhase, world: &fixture::World) {
    let base = evo.phase_native_abstract_state(&world.sense()).unwrap();
    let predecessor = evo.phase_native_abstract_state(
        &world.raw(if world.context { 2 } else { 1 })
    ).unwrap().cell;
    let witness = evo.phase_native_context_witnesses().into_iter()
        .find(|w| w.base_cell == base.cell && w.promoted).unwrap();
    let side = witness.predecessor_cells.iter().position(|&p| p == predecessor).unwrap();
    let link = witness.input_synapses[side][1];
    let goal = world.requested_goal();
    let correct = world.expected();

    let mut intact = evo.clone();
    assert_eq!(intact.phase_native_context_action(&goal), (true, Some(correct)));
    let original = intact.phase_native_learned_fingerprint();
    let mut damaged = intact.clone();
    let saved = damaged.perturb_phase_native_synapse_for_control(link, 0.0, 0.0).unwrap();
    assert_eq!(damaged.phase_native_context_action(&goal), (true, None));
    damaged.restore_phase_native_synapse_for_control(link, saved);
    assert_eq!(damaged.phase_native_learned_fingerprint(), original);
    assert_eq!(damaged.phase_native_context_action(&goal), (true, Some(correct)));
    let mut shifted = intact.clone();
    shifted.perturb_phase_native_synapse_for_control(link, 1.0, std::f32::consts::PI).unwrap();
    assert_eq!(shifted.phase_native_context_action(&goal), (true, None));
    let mut unrelated = intact.clone();
    unrelated.perturb_phase_native_synapse_for_control(
        witness.input_synapses[1 - side][1], 0.0, 0.0
    ).unwrap();
    assert_eq!(unrelated.phase_native_context_action(&goal), (true, Some(correct)));

    let checkpoint = intact.phase_native_checkpoint().unwrap();
    let mut restarted = EvoPhase::new(intact.config().clone());
    assert!(restarted.restore_phase_native_checkpoint(checkpoint));
    restarted.observe_initial_real(&world.sense(), false);
    assert_eq!(restarted.phase_native_context_action(&goal), (true, None),
        "a fresh junction image cannot recreate missing pre-restart history");
}

#[test]
fn g21_same_image_becomes_two_evidence_gated_operational_states() {
    for swap in [false, true] {
        let (evo, mut world) = fixture::prepare(swap);
        assert!(evo.phase_native_context_witnesses().is_empty());
        let drive = evo.phase_native_drive_weights();
        let old_cells = evo.phase_native_abstract_state(&world.raw(0)).unwrap().cell;
        let mut runtime = ScientificRuntime::new(evo).unwrap();
        assert!(runtime.enable_context_refinement());
        runtime.observe_external(&world.sense()).unwrap();
        let mut promotion_at = None;
        let mut promotion_support = 0;
        let mut promotion_log_e = 0.0;
        for step in 0..1600usize {
            execute_step(&mut runtime, &mut world);
            if promotion_at.is_none() {
                if let Some(w) = runtime.organism().phase_native_context_witnesses().into_iter()
                    .find(|w| w.base_cell == old_cells && w.promoted)
                {
                    assert!(w.eligible_observations >= 32);
                    assert!(w.context_switches >= 4);
                    assert!(w.log_evidence >= (16.0f64 / 0.01).ln());
                    promotion_at = Some(step + 1);
                    promotion_support = w.eligible_observations;
                    promotion_log_e = w.log_evidence;
                }
            }
            if step >= 1023 && world.state == 5 { break; }
        }
        println!("G21_LEARNING swap={} actions={} junction_visits={} promotion_at={:?} support={} log_e={:.6} witnesses={:?}",
            swap, world.calls, world.decision_visits, promotion_at, promotion_support,
            promotion_log_e, runtime.organism().phase_native_context_witnesses());
        assert!(promotion_at.is_some(), "new contextual representation must be acquired");
        assert!(world.calls <= 1600);
        assert_eq!(world.state, 5);
        let evidence = runtime.organism().phase_native_context_witnesses().into_iter()
            .find(|w| w.base_cell == old_cells && w.promoted).unwrap();
        assert_ne!(evidence.state_cells[0], evidence.state_cells[1]);
        assert!(evidence.state_cells.iter().all(|&c| c != old_cells));
        assert_eq!(runtime.organism().phase_native_drive_weights(), drive);
        runtime.set_model_learning_enabled(false);
        let fp = runtime.organism().phase_native_learned_fingerprint();
        world.scoring = true;
        let mut scored = 0usize;
        let mut full = 0usize;
        let mut memoryless = 0usize;
        let mut per_context = [[0usize; 2]; 2];
        let mut causally_checked = [false; 2];
        let mut restarted = false;

        for _ in 0..400usize {
            set_current_goal(&mut runtime, &world);
            if scored >= 32 && !restarted && matches!(world.state, 1 | 2) {
                protected_restart(&mut runtime, &world);
                restarted = true;
            }
            let is_junction = world.state == 0;
            let correct = world.expected();
            let context = usize::from(world.context);
            if is_junction {
                // The comparison receives exactly the same raw current and goal
                // and acquired model, but uses the old unrefined state readout.
                let mut control = runtime.organism().clone();
                let scene = runtime.organism().current_real().unwrap().sensory.clone();
                memoryless += usize::from(control.plan_phase_native_abstract_goal(
                    &scene, &world.requested_goal(), None
                ).map(|d| d.first_action) == Some(correct));
                if !causally_checked[context] {
                    causal_checks(runtime.organism(), &world);
                    causally_checked[context] = true;
                }
            }
            let action = execute_step(&mut runtime, &mut world);
            if is_junction {
                full += usize::from(action == correct);
                per_context[context][0] += usize::from(action == correct);
                per_context[context][1] += 1;
                scored += 1;
            }
            assert_eq!(runtime.organism().phase_native_learned_fingerprint(), fp);
            assert_eq!(runtime.organism().planning_transition_count(), 0);
            assert!(runtime.organism().composite_concepts().is_empty());
            if scored == 64 { break; }
        }
        println!("G21_RESULT swap={} full={}/64 memoryless={}/64 contexts={:?} promotion_at={:?} evidence_samples={} log_e={:.6} causal={:?} restart={} legacy=0",
            swap, full, memoryless, per_context, promotion_at, promotion_support,
            promotion_log_e, causally_checked, restarted);
        assert_eq!(scored, 64);
        assert!(full >= 60);
        assert!(memoryless <= 40);
        assert!(causally_checked.into_iter().all(|x| x));
        assert!(restarted);
    }
}

#[test]
fn g21_state_refinement_is_native_not_a_host_context_answer_table() {
    let native = include_str!("../src/phase_contextual.rs");
    for required in ["native_cell_observation", "context_gate", "context_counts",
        "conductance", "phase_native_goal_decision_from_cells",
        "context_find_novel_collision", "context_witness_signature"] {
        assert!(native.contains(required), "missing native dependency {}", required);
    }
    for forbidden in ["world.context", "STATE_PAIRS", "expected()", "EvoImaginationPlanner",
        "HashMap", "BTreeMap", "BinaryHeap", "counter % 2"] {
        assert!(!native.contains(forbidden), "forbidden task shortcut {}", forbidden);
    }
    let host = include_str!("../src/scientific_runtime.rs");
    assert!(!host.contains("context_log_evidence"));
    assert!(!host.contains("predecessor_cells"));
    assert!(host.contains("phase_native_context_action"));
    assert!(host.contains("observe_phase_native_refinement_fanout_result"));
    let fanout = include_str!("../src/phase_refinement_fanout.rs");
    assert!(fanout.contains("phase_context_sidecar"));
    assert!(host.find("consume_permit(permit)").unwrap() < host.find("match execute(action)").unwrap());
}
