//! P1 evaluator. No evaluator labels or expected actions enter production.
use aeterna_v1::{Authority, EvoConfig, EvoPhase, PlanningConfig, RasterFieldConfig};
use aeterna_v1::carrier::PhaseNativeConfig;

const REPETITIONS: usize = 160;
const REVISION_OBSERVATIONS: usize = 32;

#[derive(Debug, Clone)]
struct World {
    length: usize,
    actions: [usize; 3],
    x: usize,
    y: usize,
    offset: usize,
}

fn raster(state: usize, x: usize, y: usize) -> Vec<f32> {
    let deltas = [(1,0), (2,0), (3,0), (4,0), (0,1),
                  (0,2), (0,3), (1,1), (1,2), (2,1)];
    let (dx, dy) = deltas[state];
    assert!(x + dx < 12 && y + dy < 12);
    let mut result = vec![0.0; 144];
    result[y * 12 + x] = 1.0;
    result[(y + dy) * 12 + x + dx] = 1.0;
    result
}

fn path_action(w: &World, stage: usize) -> usize {
    if stage == 0 { w.actions[1] } else { w.actions[(stage + 1) % 3] }
}

fn facts(w: &World) -> Vec<(usize, usize, usize, f32)> {
    let mut rows = vec![(0, w.actions[0], w.length + 1, 0.25)];
    for stage in 0..w.length {
        rows.push((stage, path_action(w, stage), stage + 1,
                   if stage + 1 == w.length { 1.0 } else { 0.0 }));
    }
    // Unreachable factual relation: a selective-lesion negative control.
    rows.push((w.length + 2, w.actions[2], w.length + 3, 0.4));
    let offset = w.offset % rows.len();
    rows.rotate_left(offset);
    rows
}

fn carrier(mode: &str) -> EvoPhase {
    let mut cfg = EvoConfig {
        sensory_cells: 144, motor_cells: 3, dormant_cells: 64, hdc_dim: 192,
        ..EvoConfig::default()
    };
    match mode {
        "zero_phase" => cfg.phase_learning_rate = 0.0,
        "zero_weight" => cfg.weight_learning_rate = 0.0,
        "no_capacity" => cfg.dormant_cells = 0,
        "no_growth" => cfg.structural_growth_enabled = false,
        _ => {}
    }
    let mut evo = EvoPhase::new(cfg);
    let mut field = RasterFieldConfig::for_raster(12, 12, 3);
    field.learning_enabled = false;
    field.readout_enabled = false;
    evo.attach_raster_field(field);
    if mode == "reference" {
        evo.enable_imagination_planner(PlanningConfig {
            max_depth: 6, node_budget: 128, ..PlanningConfig::new(3)
        });
    } else {
        evo.enable_phase_native_planning(PhaseNativeConfig::default());
    }
    evo
}

fn acquire(evo: &mut EvoPhase, w: &World) -> usize {
    let rows = facts(w);
    let mut observations = 0;
    for _ in 0..REPETITIONS {
        for &(from, action, to, value) in &rows {
            evo.observe_planning_transition(&raster(from, 0, 0), action, &raster(to, 0, 0), value);
            observations += 1;
        }
    }
    evo.set_planning_learning_enabled(false);
    observations
}

fn action(evo: &mut EvoPhase, w: &World) -> Option<usize> {
    evo.plan_imagined(&raster(0, w.x, w.y)).map(|p| p.first_action)
}

fn circuit_index(w: &World, logical_record: usize) -> usize {
    let n = w.length + 2;
    (logical_record + n - (w.offset % n)) % n
}

fn complete_episode(evo: &mut EvoPhase, w: &World) -> (bool, usize) {
    let mut interactions = 0;
    for stage in 0..w.length {
        let pre = raster(stage, w.x, w.y);
        evo.observe_initial_real(&pre, false);
        let before = evo.current_real().unwrap().clone();
        let Some(decision) = evo.plan_imagined(&pre) else { return (false, interactions); };
        assert_eq!(decision.authority, Authority::Imagined);
        let after = evo.current_real().unwrap();
        assert_eq!(before.sensory, after.sensory);
        assert_eq!(before.need, after.need);
        assert_eq!(before.tick, after.tick);
        interactions += 1;
        if decision.first_action != path_action(w, stage) { return (false, interactions); }
    }
    evo.observe_initial_real(&raster(w.length, w.x, w.y), true);
    (evo.current_real().unwrap().need, interactions)
}

fn check_interventions(w: &World) -> (usize, usize) {
    let mut evo = carrier("native");
    assert!(action(&mut evo, w).is_none());
    let tuition = acquire(&mut evo, w);
    assert_eq!(evo.planning_transition_count(), 0, "no legacy transition table is present");
    assert_eq!(evo.imagined_rollout_nodes(), 0, "no graph rollout is present");
    assert_eq!(evo.phase_native_circuits().len(), w.length + 2);
    assert!(evo.recruited_relays() >= 2 * w.length + 6);
    let pre = raster(0, w.x, w.y);
    evo.observe_initial_real(&pre, false);
    let learned = evo.phase_native_learned_fingerprint();
    let d = evo.plan_imagined(&pre).expect("trained physical path");
    assert_eq!(d.first_action, w.actions[1]);
    assert_eq!(d.selected_depth, w.length);
    assert!(d.predicted_value > 0.25);
    assert_eq!(learned, evo.phase_native_learned_fingerprint());
    assert_eq!(evo.choose_immediate_model(&pre).unwrap().first_action, w.actions[0]);
    assert_eq!(complete_episode(&mut evo.clone(), w), (true, w.length));
    let bridge = evo.phase_native_circuits()[circuit_index(w, 1)].successor_synapse;
    let saved = evo.perturb_phase_native_synapse_for_control(bridge, 0.0, 0.0).unwrap();
    assert_eq!(action(&mut evo, w), Some(w.actions[0]));
    evo.restore_phase_native_synapse_for_control(bridge, saved);
    assert_eq!(action(&mut evo, w), Some(w.actions[1]));
    assert_eq!(learned, evo.phase_native_learned_fingerprint());

    let saved = evo.perturb_phase_native_synapse_for_control(bridge, 1.0, std::f32::consts::PI).unwrap();
    assert_eq!(action(&mut evo, w), Some(w.actions[0]));
    evo.restore_phase_native_synapse_for_control(bridge, saved);
    assert_eq!(action(&mut evo, w), Some(w.actions[1]));
    assert_eq!(learned, evo.phase_native_learned_fingerprint());

    let unrelated = evo.phase_native_circuits()[circuit_index(w, w.length + 1)].successor_synapse;
    let saved = evo.perturb_phase_native_synapse_for_control(unrelated, 0.0, 0.0).unwrap();
    assert_eq!(action(&mut evo, w), Some(w.actions[1]));
    evo.restore_phase_native_synapse_for_control(unrelated, saved);

    let mut disconnected = evo.clone();
    let outputs: Vec<_> = disconnected.phase_native_circuits().iter().map(|c| c.motor_synapse).collect();
    for output in outputs {
        disconnected.perturb_phase_native_synapse_for_control(output, 0.0, 0.0).unwrap();
    }
    assert!(action(&mut disconnected, w).is_none(), "no motor readout means no fallback plan");

    let terminal = circuit_index(w, w.length);
    let old = evo.phase_native_circuits()[terminal].clone();
    let mut frozen = evo.clone();
    let frozen_fingerprint = frozen.phase_native_learned_fingerprint();
    evo.set_planning_learning_enabled(true);
    for _ in 0..REVISION_OBSERVATIONS {
        for organism in [&mut evo, &mut frozen] {
            organism.observe_planning_transition(
                &raster(w.length - 1, 0, 0), path_action(w, w.length - 1),
                &raster(w.length, 0, 0), 0.0);
        }
    }
    evo.set_planning_learning_enabled(false);
    let revised = &evo.phase_native_circuits()[terminal];
    assert_eq!(revised.relay_cell, old.relay_cell);
    assert_eq!(revised.outcome_synapse, old.outcome_synapse);
    assert!(revised.revision > old.revision);
    assert!(revised.counterexamples.len() > old.counterexamples.len());
    assert_eq!(evo.phase_native_circuits().len(), w.length + 2);
    assert_eq!(action(&mut evo, w), Some(w.actions[0]));
    assert_eq!(action(&mut frozen, w), Some(w.actions[1]));
    assert_eq!(frozen_fingerprint, frozen.phase_native_learned_fingerprint());
    (tuition, REVISION_OBSERVATIONS)
}

#[test]
fn p1_physical_connection_phase_and_same_identity_revision_are_causal() {
    let permutations = [[0,1,2], [0,2,1], [1,0,2], [1,2,0], [2,0,1], [2,1,0]];
    let mut cases = 0;
    for actions in permutations {
        for length in 2..=5 {
            let w = World { length, actions, x: 4, y: 5, offset: length % (length + 2) };
            let (tuition, revision) = check_interventions(&w);
            assert_eq!(tuition, REPETITIONS * (length + 2));
            assert_eq!(revision, REVISION_OBSERVATIONS);
            cases += 1;
        }
    }
    println!("P1_DEBUG cases={} lesions=causal phase=causal restoration=exact revision=same_identity graph_table=absent", cases);
}

#[test]
fn p1_plasticity_and_capacity_controls_cannot_use_the_reference_solver() {
    let w = World { length: 3, actions: [0,1,2], x: 2, y: 4, offset: 0 };
    for mode in ["zero_phase", "zero_weight", "no_capacity", "no_growth"] {
        let mut evo = carrier(mode);
        acquire(&mut evo, &w);
        assert_eq!(evo.planning_transition_count(), 0);
        assert!(action(&mut evo, &w).is_none(), "{mode} must not get a graph answer");
    }
    let mut frozen = carrier("native");
    frozen.set_planning_learning_enabled(false);
    acquire(&mut frozen, &w);
    assert!(frozen.phase_native_circuits().is_empty());
    assert!(action(&mut frozen, &w).is_none());
}

#[test]
fn p1_source_guard_excludes_route_search_from_native_kernel() {
    let source = include_str!("../src/phase_native.rs");
    for forbidden in ["LearnedTransition", "ImaginedNode", "FrontierNode",
                      ".plan(", ".plan_with_depth(", ".pop()", "imagination_planner.as_mut"] {
        assert!(!source.contains(forbidden), "native kernel contains forbidden execution path {forbidden}");
    }
    for required in ["self.synapses", "self.cells", "syn.phase_offset", "syn.eligibility"] {
        assert!(source.contains(required));
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn range(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize }
}

fn wilson(success: usize, n: usize) -> (f64, f64) {
    let z = 1.959963984540054_f64;
    let n = n as f64;
    let p = success as f64 / n;
    let den = 1.0 + z*z/n;
    let center = (p + z*z/(2.0*n)) / den;
    let half = z * (p*(1.0-p)/n + z*z/(4.0*n*n)).sqrt() / den;
    (center-half, center+half)
}

fn mean_sd(values: &[usize]) -> (f64, f64) {
    let mean = values.iter().sum::<usize>() as f64 / values.len() as f64;
    let variance = values.iter().map(|v| (*v as f64 - mean).powi(2)).sum::<f64>()
        / (values.len() - 1).max(1) as f64;
    (mean, variance.sqrt())
}

#[test]
#[ignore = "one-use post-freeze CI authority only"]
fn p1_fresh_bounded_shared_cell_pack() {
    let authority: u64 = std::env::var("AETERNA_FRESH_SEED").expect("CI authority required").parse().unwrap();
    let source = std::env::var("AETERNA_SOURCE_SHA").expect("source SHA required");
    let spec = std::env::var("AETERNA_SPEC_SHA").expect("spec SHA required");
    assert_eq!(std::env::var("GITHUB_RUN_ID").unwrap(), authority.to_string());
    assert_eq!(source.len(), 40);
    assert_eq!(spec.len(), 40);
    let mut pack = Vec::new();
    for sub in 0..10u64 {
        let mut rng = Rng(authority ^ sub.wrapping_mul(0xd1b54a32d192ed03));
        for _ in 0..8 {
            let mut actions = [0,1,2];
            for i in (1..3).rev() { let j = rng.range(i + 1); actions.swap(i, j); }
            let length = 2 + rng.range(4);
            pack.push((sub as usize, World {
                length, actions, x: 1 + rng.range(6), y: 1 + rng.range(6),
                offset: rng.range(length + 2),
            }));
        }
    }
    let mut digest = 14_695_981_039_346_656_037_u64;
    for (index, (sub, w)) in pack.iter().enumerate() {
        let row = format!("{}:{}:{:?}", index, sub, w);
        for byte in row.bytes() { digest ^= u64::from(byte); digest = digest.wrapping_mul(1_099_511_628_211); }
        println!("P1_WORLD {row}");
    }
    println!("P1_SEAL source={} spec={} authority={} pack_digest={:016x} N=80 seeds=10 BEFORE_ALL_TUITION_AND_SCORING", source, spec, authority, digest);

    let mut successes = [0usize; 10];
    let mut reference_success = 0;
    let mut controls = [0usize; 4];
    let mut lesion_success = 0;
    let mut phase_success = 0;
    let mut restored_success = 0;
    let mut revision_success = 0;
    let mut tuition_costs = Vec::new();
    let mut execution_costs = Vec::new();
    for (sub, w) in &pack {
        let mut native = carrier("native");
        tuition_costs.push(acquire(&mut native, w));
        let (solved, interactions) = complete_episode(&mut native.clone(), w);
        successes[*sub] += usize::from(solved);
        execution_costs.push(interactions);
        let mut reference = carrier("reference");
        acquire(&mut reference, w);
        reference_success += usize::from(complete_episode(&mut reference, w).0);
        for (i, mode) in ["zero_phase", "zero_weight", "no_capacity", "no_growth"].iter().enumerate() {
            let mut evo = carrier(mode);
            acquire(&mut evo, w);
            controls[i] += usize::from(complete_episode(&mut evo, w).0);
        }
        if solved {
            // Reuses the preregistered exact intervention suite on this fresh world.
            // No source/config adaptation is performed after seeing a world.
            check_interventions(w);
            let link = native.phase_native_circuits()[circuit_index(w, 1)].successor_synapse;
            let saved = native.perturb_phase_native_synapse_for_control(link, 0.0, 0.0).unwrap();
            lesion_success += usize::from(complete_episode(&mut native.clone(), w).0);
            native.restore_phase_native_synapse_for_control(link, saved);
            let saved = native.perturb_phase_native_synapse_for_control(link, 1.0, std::f32::consts::PI).unwrap();
            phase_success += usize::from(complete_episode(&mut native.clone(), w).0);
            native.restore_phase_native_synapse_for_control(link, saved);
            restored_success += usize::from(complete_episode(&mut native.clone(), w).0);
            revision_success += 1; // counted only after exact same-identity suite returned
        }
        println!("P1_WORLD_RESULT seed={} world={:?} full={} actual_actions={}", sub, w, solved, interactions);
    }
    let total: usize = successes.iter().sum();
    let ci = wilson(total, pack.len());
    let (tuition_mean, tuition_sd) = mean_sd(&tuition_costs);
    let (action_mean, action_sd) = mean_sd(&execution_costs);
    let block_mean = total as f64 / 80.0;
    let block_sd = (successes.iter().map(|v| (*v as f64 / 8.0 - block_mean).powi(2)).sum::<f64>() / 9.0).sqrt();
    let block_half = 2.262157 * block_sd / 10.0_f64.sqrt();
    println!("P1_FRESH full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} reference={}/80 controls_zero_phase_zero_weight_no_capacity_no_growth={:?} lesion={}/{} phase={}/{} restored={}/{} revised_same_identity={}/{}",
        total, ci.0, ci.1, successes, reference_success, controls,
        lesion_success, total, phase_success, total, restored_success, total, revision_success, total);
    println!("P1_COST matched_tuition_mean={:.3} sd={:.3} physical_actions_mean={:.3} sd={:.3} revision_observations={} block_t95_descriptive=[{:.6},{:.6}] block_interval_degenerates_if_all_blocks_equal=true",
        tuition_mean, tuition_sd, action_mean, action_sd, REVISION_OBSERVATIONS,
        (block_mean-block_half).max(0.0), (block_mean+block_half).min(1.0));
    assert!(total >= 76);
    assert!(successes.iter().all(|n| *n >= 6));
    assert_eq!(restored_success, total);
    assert_eq!(revision_success, total);
    assert!(lesion_success < total && phase_success < total);
    assert_eq!(reference_success, 80);
}
