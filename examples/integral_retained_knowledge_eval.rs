//! Frozen, test-only closed-loop causal revision evaluator.
//!
//! The world supplies observations, a constant goal, equal action offers and
//! actual POSTs. It never supplies a chosen motor, hidden bit, role, rule,
//! experiment schedule or learned transition to ScientificRuntime. Evaluation
//! metadata is written only AFTER each protected action has been executed.
//! Capability gates belong to the separate scorer; this executable fails only
//! on malformed input, runtime/environment errors or integrity violations.
use aeterna_v1::construction::autonomy::{ActionOffer, AutonomyPolicy};
use aeterna_v1::construction::causal_memory::{CausalConfig, CausalReport};
use aeterna_v1::construction::Config;
use aeterna_v1::scientific_runtime::{AgencyCreditReport, ScientificRuntime, UnifiedPost};
use aeterna_v1::HumanProtectionEvidence;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};

type EvalResult<T> = Result<T, Box<dyn std::error::Error>>;
const ACTIONS: usize = 4;
const EPISODE_LIMIT: usize = 4;
const ACQUISITION_EPISODES: usize = 256;
const REVISION_EPISODES: usize = 512;
const HELD_EPISODES: usize = 64;
const BELIEF_READ_WORK: u64 = 120_000;
const ROLE_NAMES: [&str; ACTIONS] = [
    "informative_sensor",
    "noise_sensor",
    "actuator0",
    "actuator1",
];

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix(self.0)
    }
    fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            values.swap(i, (self.next() % (i + 1) as u64) as usize);
        }
    }
}

struct WorldSpec {
    seed: u64,
    world: usize,
    base: u64,
    // Indexed by the EVALUATOR'S physical role, values are opaque motor IDs.
    motors: [usize; ACTIONS],
    // start, sense0, sense1, noise0, noise1, success, failure.
    codes: [i64; 7],
}
impl WorldSpec {
    fn new(seed: u64, world: usize) -> Self {
        let base = mix(seed ^ (world as u64).wrapping_mul(0xd134_2543_de82_ef95));
        let mut motors = [0, 1, 2, 3];
        let mut codes = [101, 211, 307, 419, 503, 617, 733];
        Rng(base ^ 0x6d6f_746f_725f_6d61).shuffle(&mut motors);
        Rng(base ^ 0x6f62_735f_636f_6465).shuffle(&mut codes);
        Self {
            seed,
            world,
            base,
            motors,
            codes,
        }
    }
    fn observation(&self, state: usize) -> Vec<i64> {
        vec![0, self.codes[state]]
    }
    fn goal(&self) -> [Option<i64>; 2] {
        [None, Some(self.codes[5])]
    }
    fn role(&self, action: usize) -> usize {
        self.motors
            .iter()
            .position(|&motor| motor == action)
            .unwrap()
    }
    fn schedule_key(&self, schedule: &str) -> u64 {
        let salt = match schedule {
            "acquisition" => 0x6163_7175_6973_6974,
            "original_validation" => 0x6f72_6967_696e_616c,
            "reversal" => 0x7265_7665_7273_616c,
            "recovered_validation" => 0x7265_636f_7665_7265,
            _ => panic!("unknown frozen schedule"),
        };
        mix(self.base ^ salt)
    }
    fn hidden(&self, schedule: &str, episode: usize) -> usize {
        (mix(self.schedule_key(schedule)
            ^ 0x6869_6464_656e_5f62
            ^ (episode as u64).wrapping_mul(0xa24b_aed4_963e_e407))
            & 1) as usize
    }
    fn noise(&self, schedule: &str, episode: usize, step: usize) -> usize {
        (mix(self.schedule_key(schedule)
            ^ 0x6e6f_6973_655f_6269
            ^ (episode as u64).wrapping_mul(0x9fb2_1c65_1e98_df25)
            ^ (step as u64).wrapping_mul(0xc13f_a9a9_02a6_328f))
            & 1) as usize
    }
    fn random_motor(&self, schedule: &str, episode: usize, step: usize) -> usize {
        (mix(self.schedule_key(schedule)
            ^ 0x7261_6e64_5f61_6374
            ^ (episode as u64).wrapping_mul(0x91e1_0da5_c79e_7b1d)
            ^ (step as u64).wrapping_mul(0xd1b5_4a32_d192_ed03))
            % ACTIONS as u64) as usize
    }
    fn cue_schedule(&self, phase: &Phase) -> Vec<usize> {
        let mut cues: Vec<_> = (0..phase.episodes)
            .map(|episode| self.hidden(phase.schedule, episode))
            .collect();
        if phase.cue_shuffled {
            // Same cue marginal, independently permuted episode association.
            // This affects only the physical sensor, never the actor policy.
            Rng(self.schedule_key(phase.schedule) ^ 0x7368_7566_666c_6564).shuffle(&mut cues);
        }
        cues
    }
    fn execute(
        &self,
        phase: &Phase,
        episode: usize,
        step: usize,
        hidden: usize,
        cue: usize,
        action: usize,
    ) -> PhysicalPost {
        assert!(action < ACTIONS);
        let role = self.role(action);
        let terminal = role >= 2;
        let success = terminal && (role - 2 == (hidden ^ phase.rule));
        let state = match role {
            0 => 1 + cue,
            1 => 3 + self.noise(phase.schedule, episode, step),
            _ if success => 5,
            _ => 6,
        };
        PhysicalPost {
            action,
            role,
            post: self.observation(state),
            terminal,
            success,
        }
    }
}

struct Phase {
    name: &'static str,
    episodes: usize,
    rule: usize,
    cue_shuffled: bool,
    schedule: &'static str,
}
impl Phase {
    fn new(name: &'static str, episodes: usize, rule: usize, schedule: &'static str) -> Self {
        Self {
            name,
            episodes,
            rule,
            cue_shuffled: false,
            schedule,
        }
    }
}

struct PhysicalPost {
    action: usize,
    role: usize,
    post: Vec<i64>,
    terminal: bool,
    success: bool,
}

#[derive(Clone, Serialize)]
struct HistoryStep {
    pre: Vec<i64>,
    action: usize,
    post: Vec<i64>,
}

#[derive(Serialize)]
struct RuntimeSnapshot {
    factual_events: usize,
    total_external_cost: u64,
    causal_report: CausalReport,
    agency_credit: AgencyCreditReport,
    native_weights: [f32; 5],
    native_meta_observations: u64,
    controller_frozen: bool,
}

fn snapshot(rt: &ScientificRuntime) -> RuntimeSnapshot {
    let learner = rt.construction_learner().unwrap();
    let report = rt.unified_report().unwrap();
    let causal = learner.causal_report().unwrap();
    let agency = report.agency_credit.unwrap();
    assert_eq!(agency.externally_forced_actions, 0);
    assert_eq!(agency.option_continuation_actions, 0);
    // The repository freezes this credit ledger together with learning.
    // All additional frozen choices are independently checked by the harness.
    assert_eq!(agency.self_selected_actions, causal.learned_transitions);
    assert_eq!(
        causal.factual_transitions,
        learner.factual_event_count() as u64
    );
    assert_eq!(
        report.total_external_cost,
        learner.factual_event_count() as u64
    );
    RuntimeSnapshot {
        factual_events: learner.factual_event_count(),
        total_external_cost: report.total_external_cost,
        causal_report: causal,
        agency_credit: agency,
        native_weights: report.native_weights,
        native_meta_observations: report.native_meta_observations,
        controller_frozen: report.controller_frozen,
    }
}

fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}

fn make_runtime(world: &WorldSpec) -> EvalResult<ScientificRuntime> {
    let cfg = Config {
        width: 2,
        actions: ACTIONS,
        minimum_training: 128,
        search_every: 1_000_000,
        planning_depth: 4,
        planning_work: 120_000,
        journal_capacity: 4096,
        ..Default::default()
    };
    let policy = AutonomyPolicy {
        max_external_cost: 100_000,
        ..Default::default()
    };
    let mut rt = ScientificRuntime::new_constructing(
        cfg,
        policy,
        world.observation(0),
        "integral-causal-revision",
    )?;
    rt.enable_constructing_agency_aware_credit()?;
    rt.enable_constructing_self_directed_control()?;
    rt.enable_constructing_causal_memory(CausalConfig {
        history_depth: 3,
        ..Default::default()
    })?;
    Ok(rt)
}

fn restore_exact(bytes: &[u8]) -> EvalResult<ScientificRuntime> {
    let restored = ScientificRuntime::from_constructing_checkpoint(bytes)?;
    assert_eq!(restored.checkpoint_constructing()?, bytes);
    Ok(restored)
}

fn write_row(out: &mut BufWriter<File>, row: &Value) -> EvalResult<()> {
    serde_json::to_writer(&mut *out, row)?;
    out.write_all(b"\n")?;
    Ok(())
}

fn run_phase(
    rt: &mut ScientificRuntime,
    world: &WorldSpec,
    phase: Phase,
    trace: &mut BufWriter<File>,
) -> EvalResult<Value> {
    let before = snapshot(rt);
    let frozen = before.controller_frozen;
    let offers = vec![
        ActionOffer {
            available: true,
            max_cost: 1
        };
        ACTIONS
    ];
    let goal = world.goal();
    let cues = world.cue_schedule(&phase);
    let mut actions = 0usize;
    let mut callbacks = 0usize;
    let mut successes = 0usize;
    let mut failures = 0usize;
    let mut sensor_queries = 0usize;
    let mut noise_queries = 0usize;
    let mut readonly_work = 0u64;
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut episodes = Vec::new();
    for (episode, &cue) in cues.iter().enumerate() {
        let hidden = world.hidden(phase.schedule, episode);
        let mut current = world.observation(0);
        rt.observe_constructing_external_restart(current.clone())?;
        let mut history = Vec::<HistoryStep>::new();
        let mut ep_actions = 0usize;
        let mut ep_sensor = 0usize;
        let mut ep_noise = 0usize;
        let mut ep_success = false;
        let mut ep_terminal = false;
        for step_number in 0..EPISODE_LIMIT {
            assert_eq!(rt.construction_learner().unwrap().current(), current);
            let pre = current.clone();
            let before_causal = rt.construction_learner().unwrap().causal_report().unwrap();
            let (pre_beliefs, observer_work) = rt
                .construction_learner()
                .unwrap()
                .causal_action_beliefs(&offers, Some(&goal), BELIEF_READ_WORK)?;
            readonly_work += observer_work;
            let mut actual = None;
            let result = rt.step_constructing_with_action_limit(
                Some(&goal),
                &offers,
                Some(EPISODE_LIMIT - step_number),
                safe,
                |action| {
                    assert!(actual.is_none(), "one callback per protected choice");
                    callbacks += 1;
                    let post = world.execute(&phase, episode, step_number, hidden, cue, action);
                    let returned = UnifiedPost {
                        observation: post.post.clone(),
                        cost: 1,
                        reward: f32::from(post.success),
                    };
                    actual = Some(post);
                    Ok(returned)
                },
            )?;
            let post = actual.expect("runtime must execute exactly its selected motor");
            assert_eq!(post.action, result.action);
            actions += 1;
            ep_actions += 1;
            assert_eq!(callbacks, actions);
            assert_eq!(result.event, before.factual_events + actions);
            let learner = rt.construction_learner().unwrap();
            assert_eq!(learner.current(), post.post);
            let record = learner.records().last().unwrap();
            assert_eq!(record.transition.pre, pre);
            assert_eq!(record.transition.action, result.action);
            assert_eq!(record.transition.post, post.post);
            assert_eq!(record.external_cost, 1);
            let after_causal = learner.causal_report().unwrap();
            assert_eq!(
                after_causal.factual_transitions,
                before.causal_report.factual_transitions + actions as u64
            );
            if frozen {
                assert_eq!(
                    after_causal.learned_fingerprint,
                    before.causal_report.learned_fingerprint
                );
                assert_eq!(
                    after_causal.learned_transitions,
                    before.causal_report.learned_transitions
                );
            } else {
                assert_eq!(
                    after_causal.learned_transitions,
                    before.causal_report.learned_transitions + actions as u64
                );
            }
            *source_counts.entry(result.source.clone()).or_default() += 1;
            if post.role == 0 {
                sensor_queries += 1;
                ep_sensor += 1;
            }
            if post.role == 1 {
                noise_queries += 1;
                ep_noise += 1;
            }
            write_row(
                trace,
                &json!({
                    "schema":"aeterna.integral-causal-revision.step.v1",
                    "seed":world.seed,"world":world.world,"phase":phase.name,
                    "episode":episode,"step":step_number,"rule":phase.rule,
                    "hidden":hidden,"sensor_cue":cue,"cue_shuffled":phase.cue_shuffled,
                    "pre":&pre,"history":&history,"pre_beliefs":pre_beliefs,
                    "readonly_belief_work":observer_work,
                    "action":result.action,"role":ROLE_NAMES[post.role],
                    "source":result.source,"selected_fields":result.selected_fields,
                    "native_score":result.native_score,"factual_utility":result.factual_utility,
                    "operations":result.operations,"computation_work":result.computation_work,
                    "post":&post.post,"reward":f32::from(post.success),"cost":1,
                    "terminal":post.terminal,"success":post.success,"event":result.event,
                    "permit_sequence":record.permit_sequence,"vm_prediction":&record.prediction,
                    "learned_fingerprint_before":before_causal.learned_fingerprint,
                    "learned_fingerprint_after":after_causal.learned_fingerprint,
                }),
            )?;
            history.push(HistoryStep {
                pre,
                action: post.action,
                post: post.post.clone(),
            });
            if history.len() > 3 {
                history.remove(0);
            }
            current = post.post;
            ep_success = post.success;
            ep_terminal = post.terminal;
            if post.terminal {
                break;
            }
        }
        successes += usize::from(ep_success);
        failures += usize::from(ep_terminal && !ep_success);
        episodes.push(json!({
            "episode":episode,"hidden":hidden,"sensor_cue":cue,
            "success":ep_success,"terminal":ep_terminal,"actions":ep_actions,
            "sensor_queries":ep_sensor,"noise_queries":ep_noise,
        }));
    }
    let after = snapshot(rt);
    assert_eq!(after.factual_events - before.factual_events, callbacks);
    assert_eq!(
        after.total_external_cost - before.total_external_cost,
        callbacks as u64
    );
    assert_eq!(after.controller_frozen, frozen);
    if frozen {
        assert_eq!(
            after.agency_credit.self_selected_actions,
            before.agency_credit.self_selected_actions
        );
        assert_eq!(after.native_weights, before.native_weights);
        assert_eq!(
            after.native_meta_observations,
            before.native_meta_observations
        );
    } else {
        assert_eq!(
            after.agency_credit.self_selected_actions - before.agency_credit.self_selected_actions,
            callbacks as u64
        );
    }
    trace.flush()?;
    eprintln!(
        "INTEGRAL world={} phase={} goals={}/{} actions={}",
        world.world, phase.name, successes, phase.episodes, actions
    );
    Ok(json!({
        "phase":phase.name,"episodes":phase.episodes,"rule":phase.rule,
        "cue_shuffled":phase.cue_shuffled,"frozen":frozen,"schedule":phase.schedule,
        "successes":successes,"terminal_failures":failures,
        "timeouts":phase.episodes-successes-failures,"actions":actions,
        "physical_callbacks":callbacks,"sensor_queries":sensor_queries,
        "noise_queries":noise_queries,"source_counts":source_counts,
        "readonly_belief_work":readonly_work,"episode_results":episodes,
        "before":before,"after":after,
    }))
}

fn run_random(world: &WorldSpec, trace: &mut BufWriter<File>) -> EvalResult<Value> {
    let phase = Phase::new("random_held", HELD_EPISODES, 1, "recovered_validation");
    let mut successes = 0usize;
    let mut failures = 0usize;
    let mut actions = 0usize;
    let mut sensor_queries = 0usize;
    let mut noise_queries = 0usize;
    let mut episodes = Vec::new();
    for episode in 0..phase.episodes {
        let hidden = world.hidden(phase.schedule, episode);
        let mut current = world.observation(0);
        let mut history = Vec::<HistoryStep>::new();
        let mut ep_success = false;
        let mut ep_terminal = false;
        let mut ep_actions = 0usize;
        let mut ep_sensor = 0usize;
        let mut ep_noise = 0usize;
        for step_number in 0..EPISODE_LIMIT {
            let action = world.random_motor(phase.schedule, episode, step_number);
            let post = world.execute(&phase, episode, step_number, hidden, hidden, action);
            actions += 1;
            ep_actions += 1;
            if post.role == 0 {
                sensor_queries += 1;
                ep_sensor += 1;
            }
            if post.role == 1 {
                noise_queries += 1;
                ep_noise += 1;
            }
            write_row(
                trace,
                &json!({
                    "schema":"aeterna.integral-causal-revision.step.v1",
                    "seed":world.seed,"world":world.world,"phase":phase.name,
                    "episode":episode,"step":step_number,"rule":phase.rule,
                    "hidden":hidden,"sensor_cue":hidden,"cue_shuffled":false,
                    "pre":&current,"history":&history,"pre_beliefs":[],
                    "readonly_belief_work":0,"action":action,"role":ROLE_NAMES[post.role],
                    "source":"uniform-random","selected_fields":null,"native_score":null,
                    "post":&post.post,"reward":f32::from(post.success),"cost":1,
                    "terminal":post.terminal,"success":post.success,"event":actions,
                    "learned_fingerprint_before":null,"learned_fingerprint_after":null,
                }),
            )?;
            history.push(HistoryStep {
                pre: current,
                action,
                post: post.post.clone(),
            });
            if history.len() > 3 {
                history.remove(0);
            }
            current = post.post;
            ep_success = post.success;
            ep_terminal = post.terminal;
            if post.terminal {
                break;
            }
        }
        successes += usize::from(ep_success);
        failures += usize::from(ep_terminal && !ep_success);
        episodes.push(json!({
            "episode":episode,"hidden":hidden,"sensor_cue":hidden,
            "success":ep_success,"terminal":ep_terminal,"actions":ep_actions,
            "sensor_queries":ep_sensor,"noise_queries":ep_noise,
        }));
    }
    trace.flush()?;
    eprintln!(
        "INTEGRAL world={} phase=random_held goals={}/{} actions={}",
        world.world, successes, phase.episodes, actions
    );
    Ok(json!({
        "phase":phase.name,"episodes":phase.episodes,"rule":phase.rule,
        "cue_shuffled":false,"frozen":false,"schedule":phase.schedule,
        "successes":successes,"terminal_failures":failures,
        "timeouts":phase.episodes-successes-failures,"actions":actions,
        "physical_callbacks":actions,"sensor_queries":sensor_queries,
        "noise_queries":noise_queries,"source_counts":{"uniform-random":actions},
        "readonly_belief_work":0,"episode_results":episodes,"before":null,"after":null,
    }))
}

// Retained-knowledge extension. The helper prefix above is copied verbatim
// from the original frozen evaluator. The old evaluator is not edited. The
// orchestrator must verify all eight reconstructed traces and summaries
// against their archived originals before launching any repeat command.
fn checkpoint_identity(path: &std::path::Path, bytes: &[u8]) -> EvalResult<Value> {
    Ok(json!({
        "path":std::fs::canonicalize(path)?,
        "bytes":bytes.len(),
        "fingerprint":aeterna_v1::construction::vm::fingerprint(bytes),
    }))
}

fn persist_checkpoint(
    directory: &std::path::Path,
    filename: &str,
    bytes: &[u8],
) -> EvalResult<Value> {
    // This exercises the public parser and exact serialization, never edits
    // checkpoint fields or synthesizes a learning-enabled version of a freeze.
    let _ = restore_exact(bytes)?;
    let path = directory.join(filename);
    std::fs::write(&path, bytes)?;
    assert_eq!(std::fs::read(&path)?, bytes);
    checkpoint_identity(&path, bytes)
}

fn verify_boundary_equivalence(bytes: &[u8], world: &WorldSpec) -> EvalResult<()> {
    let mut freeze_then_restart = restore_exact(bytes)?;
    assert!(!snapshot(&freeze_then_restart).controller_frozen);
    freeze_then_restart.freeze_constructing()?;
    freeze_then_restart.observe_constructing_external_restart(world.observation(0))?;

    let mut restart_then_freeze = restore_exact(bytes)?;
    restart_then_freeze.observe_constructing_external_restart(world.observation(0))?;
    restart_then_freeze.freeze_constructing()?;
    assert_eq!(
        freeze_then_restart.checkpoint_constructing()?,
        restart_then_freeze.checkpoint_constructing()?,
        "ordinary restart must settle pending credit exactly like the old freeze",
    );
    Ok(())
}

fn run_life(
    mut acquired: ScientificRuntime,
    world: &WorldSpec,
    mut trace: &mut BufWriter<File>,
    checkpoint_dir: &std::path::Path,
) -> EvalResult<(Value, Value)> {
    let mut phases = Vec::new();
    phases.push(run_phase(
        &mut acquired,
        &world,
        Phase::new("acquire", ACQUISITION_EPISODES, 0, "acquisition"),
        &mut trace,
    )?);
    let acquired_checkpoint = acquired.checkpoint_constructing()?;
    let acquired_file = persist_checkpoint(
        checkpoint_dir,
        "acquired.checkpoint.json",
        &acquired_checkpoint,
    )?;

    // Validation is a frozen clone and cannot contaminate the live lineage.
    let mut original = restore_exact(&acquired_checkpoint)?;
    original.freeze_constructing()?;
    phases.push(run_phase(
        &mut original,
        &world,
        Phase::new("original_held", HELD_EPISODES, 0, "original_validation"),
        &mut trace,
    )?);
    assert_eq!(acquired.checkpoint_constructing()?, acquired_checkpoint);
    drop(original);
    drop(acquired);

    let mut adaptive = restore_exact(&acquired_checkpoint)?;
    let mut frozen = restore_exact(&acquired_checkpoint)?;
    frozen.freeze_constructing()?;
    // Both arms get identical exogenous hidden/noise schedules. The hidden
    // law changes here, at a real episode boundary, with no change signal.
    phases.push(run_phase(
        &mut adaptive,
        &world,
        Phase::new("adaptive", REVISION_EPISODES, 1, "reversal"),
        &mut trace,
    )?);
    phases.push(run_phase(
        &mut frozen,
        &world,
        Phase::new("frozen", REVISION_EPISODES, 1, "reversal"),
        &mut trace,
    )?);

    // Capture the complete LIVE state before the original irreversible freeze.
    // Serializing and verifying a disposable clone do not advance this actor.
    let continuation_checkpoint = adaptive.checkpoint_constructing()?;
    let continuation_file = persist_checkpoint(
        checkpoint_dir,
        "continuation-unfrozen.checkpoint.json",
        &continuation_checkpoint,
    )?;
    adaptive.freeze_constructing()?;
    let revised_checkpoint = adaptive.checkpoint_constructing()?;
    let stale_checkpoint = frozen.checkpoint_constructing()?;
    let revised_file = persist_checkpoint(
        checkpoint_dir,
        "revised-frozen.checkpoint.json",
        &revised_checkpoint,
    )?;
    let stale_file = persist_checkpoint(
        checkpoint_dir,
        "stale-frozen.checkpoint.json",
        &stale_checkpoint,
    )?;
    let mut verification = restore_exact(&continuation_checkpoint)?;
    assert!(!snapshot(&verification).controller_frozen);
    verification.freeze_constructing()?;
    assert_eq!(verification.checkpoint_constructing()?, revised_checkpoint);
    verify_boundary_equivalence(&continuation_checkpoint, world)?;
    let mut recovered = restore_exact(&revised_checkpoint)?;
    phases.push(run_phase(
        &mut recovered,
        &world,
        Phase::new("recovered_held", HELD_EPISODES, 1, "recovered_validation"),
        &mut trace,
    )?);
    let mut shuffled = restore_exact(&revised_checkpoint)?;
    let mut shuffled_phase = Phase::new(
        "shuffled_cue_held",
        HELD_EPISODES,
        1,
        "recovered_validation",
    );
    shuffled_phase.cue_shuffled = true;
    phases.push(run_phase(
        &mut shuffled,
        &world,
        shuffled_phase,
        &mut trace,
    )?);
    let mut stale = restore_exact(&stale_checkpoint)?;
    phases.push(run_phase(
        &mut stale,
        &world,
        Phase::new("stale_held", HELD_EPISODES, 1, "recovered_validation"),
        &mut trace,
    )?);
    phases.push(run_random(&world, &mut trace)?);
    assert_eq!(adaptive.checkpoint_constructing()?, revised_checkpoint);
    assert_eq!(frozen.checkpoint_constructing()?, stale_checkpoint);

    let result = json!({
        "schema":"aeterna.integral-causal-revision.v1",
        "seed":world.seed,"world":world.world,
        "motor_roles":{
            "informative_sensor":world.motors[0],"noise_sensor":world.motors[1],
            "actuator0":world.motors[2],"actuator1":world.motors[3],
        },
        "observation_codes":{
            "start":world.codes[0],"sense0":world.codes[1],"sense1":world.codes[2],
            "noise0":world.codes[3],"noise1":world.codes[4],
            "success":world.codes[5],"failure":world.codes[6],
        },
        "config":{
            "width":2,"actions":ACTIONS,"minimum_training":128,
            "search_every":1000000,"planning_depth":4,"planning_work":120000,
            "history_depth":3,"journal_capacity":4096,"max_external_cost":100000,
            "episode_action_limit":EPISODE_LIMIT,"acquisition_episodes":ACQUISITION_EPISODES,
            "revision_episodes":REVISION_EPISODES,"held_episodes":HELD_EPISODES,
            "readonly_belief_work_per_call":BELIEF_READ_WORK,
            "readonly_diagnostics_outside_actor_budget":true,
            "agency_credit_counts_learning_actions_only":true,
        },
        "checkpoint_bytes":{
            "acquired":acquired_checkpoint.len(),"revised":revised_checkpoint.len(),
            "stale":stale_checkpoint.len(),
        },
        "phases":phases,
        "integrity":{
            "equal_offers_all_four_actions":true,
            "no_forced_runtime_actions":true,
            "runtime_callback_matches_selected_action":true,
            "runtime_journal_matches_physical_pre_post":true,
            "runtime_cost_and_transition_counts_match_callbacks":true,
            "frozen_knowledge_and_native_weights_unchanged":true,
            "agency_ledger_matches_learning_actions":true,
            "checkpoint_restores_exactly":true,
            "heldout_clones_do_not_contaminate_live_lineage":true,
            "paired_exogenous_schedules":true,
            "scoring_only_latent_metadata":true,
        },
    });
    Ok((
        result,
        json!({
            "acquired":acquired_file,
            "continuation_unfrozen":continuation_file,
            "revised_frozen":revised_file,
            "stale_frozen":stale_file,
        }),
    ))
}

fn main() -> EvalResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !matches!(args.len(), 10 | 12) {
        return Err(concat!(
            "usage: integral_retained_knowledge_eval --mode reconstruct|repeat ",
            "--seed <u64> --world <0..7> --trace <path> --checkpoints <directory> ",
            "[--input-checkpoint <path>]",
        )
        .into());
    }
    let (
        mut mode,
        mut seed,
        mut world_index,
        mut trace_path,
        mut checkpoint_dir,
        mut input_checkpoint,
    ) = (None, None, None, None, None, None);
    for pair in args.chunks_exact(2) {
        match pair[0].as_str() {
            "--mode" if mode.is_none() => mode = Some(pair[1].clone()),
            "--seed" if seed.is_none() => seed = Some(pair[1].parse::<u64>()?),
            "--world" if world_index.is_none() => world_index = Some(pair[1].parse::<usize>()?),
            "--trace" if trace_path.is_none() => trace_path = Some(pair[1].clone()),
            "--checkpoints" if checkpoint_dir.is_none() => checkpoint_dir = Some(pair[1].clone()),
            "--input-checkpoint" if input_checkpoint.is_none() => {
                input_checkpoint = Some(pair[1].clone())
            }
            _ => return Err("unknown or repeated argument".into()),
        }
    }
    let mode = mode.ok_or("missing --mode")?;
    match mode.as_str() {
        "reconstruct" if input_checkpoint.is_none() => {}
        "repeat" if input_checkpoint.is_some() => {}
        "reconstruct" => return Err("reconstruct cannot take an input checkpoint".into()),
        "repeat" => return Err("repeat requires --input-checkpoint".into()),
        _ => return Err("--mode must be reconstruct or repeat".into()),
    }
    let world_index = world_index.ok_or("missing --world")?;
    if world_index >= 8 {
        return Err("--world must be in 0..7".into());
    }
    let world = WorldSpec::new(seed.ok_or("missing --seed")?, world_index);
    let checkpoint_dir = std::path::PathBuf::from(checkpoint_dir.ok_or("missing --checkpoints")?);
    std::fs::create_dir_all(&checkpoint_dir)?;
    let (initial, input_identity, input_boundary_equivalent) = match input_checkpoint {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            let bytes = std::fs::read(&path)?;
            let initial = restore_exact(&bytes)?;
            if snapshot(&initial).controller_frozen {
                return Err("repeat requires the captured unfrozen continuation checkpoint".into());
            }
            verify_boundary_equivalence(&bytes, &world)?;
            (initial, checkpoint_identity(&path, &bytes)?, Some(true))
        }
        None => (make_runtime(&world)?, Value::Null, None),
    };
    let before_reacquisition = snapshot(&initial);
    assert!(!before_reacquisition.controller_frozen);
    let initial_checkpoint = initial.checkpoint_constructing()?;
    let initial_file = persist_checkpoint(
        &checkpoint_dir,
        "initial.checkpoint.json",
        &initial_checkpoint,
    )?;
    let mut trace = BufWriter::new(File::create(trace_path.ok_or("missing --trace")?)?);
    let (summary, mut checkpoints) = run_life(initial, &world, &mut trace, &checkpoint_dir)?;
    checkpoints
        .as_object_mut()
        .unwrap()
        .insert("initial".into(), initial_file);
    let metadata = json!({
        "schema":"aeterna.integral-retained-knowledge.metadata.v1",
        "mode":mode,"seed":world.seed,"world":world.world,
        "input_checkpoint":input_identity,
        "before_reacquisition":before_reacquisition,
        "checkpoints":checkpoints,
        "input_boundary_equivalent":input_boundary_equivalent,
        "integrity":{
            "checkpoint_roundtrips_exact":true,
            "continuation_freezes_to_revised_exactly":true,
            "continuation_boundary_equivalent":true,
            "initial_unfrozen":true,
        },
    });
    let mut metadata_file = BufWriter::new(File::create(checkpoint_dir.join("metadata.json"))?);
    serde_json::to_writer_pretty(&mut metadata_file, &metadata)?;
    metadata_file.write_all(b"\n")?;
    metadata_file.flush()?;
    serde_json::to_writer(std::io::stdout().lock(), &summary)?;
    println!();
    Ok(())
}
