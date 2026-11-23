//! Experimental, closed-loop comparison of inferred context dynamics.
//!
//! The actor receives only opaque observations, a fixed goal, equal-cost action
//! offers and the factual result of its own chosen action. Hidden rules, switch
//! times and physical motor roles remain inside this evaluator. All prospective
//! forecasts are read before execution; scoring metadata is emitted afterward.
#![recursion_limit = "256"]
use aeterna_v1::construction::autonomy::{ActionOffer, AutonomyPolicy};
use aeterna_v1::construction::causal_memory::{CausalConfig, CausalReport, RegimeMode};
use aeterna_v1::construction::Config;
use aeterna_v1::scientific_runtime::{AgencyCreditReport, ScientificRuntime, UnifiedPost};
use aeterna_v1::HumanProtectionEvidence;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

type EvalResult<T> = Result<T, Box<dyn std::error::Error>>;
const ACTIONS: usize = 4;
const EPISODE_LIMIT: usize = 4;
const ACQUISITION_EPISODES: usize = 384;
const SWITCHING_EPISODES: usize = 2048;
const STATIONARY_EPISODES: usize = 512;
const BELIEF_READ_WORK: u64 = 120_000;
const RETAINED_MAPPING_SEED: u64 = 45_231_338_992;
const ARMS: [&str; 5] = [
    "baseline",
    "ordinary",
    "phase",
    "phase_scrambled",
    "phase_uncoupled",
];
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
    mapping_seed: u64,
    world: usize,
    motors: [usize; ACTIONS],
    codes: [i64; 7],
}
impl WorldSpec {
    fn new(mapping_seed: u64, world: usize) -> Self {
        // Exact mapping generator from the original, frozen evaluator.
        let base = mix(mapping_seed ^ (world as u64).wrapping_mul(0xd134_2543_de82_ef95));
        let mut motors = [0, 1, 2, 3];
        let mut codes = [101, 211, 307, 419, 503, 617, 733];
        Rng(base ^ 0x6d6f_746f_725f_6d61).shuffle(&mut motors);
        Rng(base ^ 0x6f62_735f_636f_6465).shuffle(&mut codes);
        Self {
            mapping_seed,
            world,
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
    fn execute(&self, episode: &EpisodeSpec, step: usize, action: usize) -> PhysicalPost {
        assert!(action < ACTIONS);
        let role = self
            .motors
            .iter()
            .position(|&motor| motor == action)
            .unwrap();
        let terminal = role >= 2;
        let success = terminal && role - 2 == (episode.hidden ^ episode.rule);
        let state = match role {
            0 => 1 + episode.hidden,
            1 => 3 + episode.noise[step],
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

#[derive(Clone, Serialize)]
struct Segment {
    index: usize,
    start_episode: usize,
    end_episode: usize,
    length: usize,
    rule: usize,
}

#[derive(Clone, Serialize)]
struct EpisodeSpec {
    episode: usize,
    segment: usize,
    rule: usize,
    hidden: usize,
    noise: [usize; EPISODE_LIMIT],
}

fn private_seed(seed: u64, suite: &str, world: usize) -> u64 {
    let tag = match suite {
        "fresh" => 0x6672_6573_685f_7068,
        "retained" => 0x7265_7461_696e_7068,
        _ => unreachable!(),
    };
    mix(seed ^ tag ^ (world as u64).wrapping_mul(0xd134_2543_de82_ef95))
}

fn switching_segments(base: u64, initial_rule: usize) -> Vec<Segment> {
    let mut rng = Rng(base ^ 0x6477_656c_6c5f_746d);
    let weights: Vec<_> = (0..8).map(|_| 1 + rng.next() % 1024).collect();
    let total = weights.iter().sum::<u64>();
    let mut lengths: Vec<_> = weights
        .iter()
        .map(|w| 128 + (1024 * w / total) as usize)
        .collect();
    let remainder = SWITCHING_EPISODES - lengths.iter().sum::<usize>();
    let mut order: Vec<_> = (0..8).collect();
    rng.shuffle(&mut order);
    for &index in order.iter().take(remainder) {
        lengths[index] += 1;
    }
    assert_eq!(lengths.iter().sum::<usize>(), SWITCHING_EPISODES);
    let mut start = 0;
    lengths
        .into_iter()
        .enumerate()
        .map(|(index, length)| {
            let segment = Segment {
                index,
                start_episode: start,
                end_episode: start + length,
                length,
                rule: initial_rule ^ (index & 1),
            };
            start += length;
            segment
        })
        .collect()
}

fn make_schedule(base: u64, condition: &str, segments: &[Segment]) -> Vec<EpisodeSpec> {
    let salt = match condition {
        "acquisition" => 0x6163_7175_6973_7068,
        "switching" => 0x7377_6974_6368_7068,
        "stationary" => 0x7374_6174_696f_7068,
        _ => unreachable!(),
    };
    let base = mix(base ^ salt);
    let mut out = Vec::new();
    for segment in segments {
        for episode in segment.start_episode..segment.end_episode {
            let hidden = (mix(base
                ^ 0x6869_6464_656e_5f62
                ^ (episode as u64).wrapping_mul(0xa24b_aed4_963e_e407))
                & 1) as usize;
            let noise = std::array::from_fn(|step| {
                (mix(base
                    ^ 0x6e6f_6973_655f_6269
                    ^ (episode as u64).wrapping_mul(0x9fb2_1c65_1e98_df25)
                    ^ (step as u64).wrapping_mul(0xc13f_a9a9_02a6_328f))
                    & 1) as usize
            });
            out.push(EpisodeSpec {
                episode,
                segment: segment.index,
                rule: segment.rule,
                hidden,
                noise,
            });
        }
    }
    out
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
    computation_work: u64,
    causal_report: CausalReport,
    agency_credit: AgencyCreditReport,
    native_weights: [f32; 5],
    native_meta_observations: u64,
    controller_frozen: bool,
    regime_report: Value,
}

fn snapshot(rt: &ScientificRuntime) -> RuntimeSnapshot {
    let learner = rt.construction_learner().unwrap();
    let report = rt.unified_report().unwrap();
    let causal = learner.causal_report().unwrap();
    let agency = report.agency_credit.unwrap();
    assert_eq!(agency.externally_forced_actions, 0);
    assert_eq!(agency.option_continuation_actions, 0);
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
        computation_work: report.computation_work,
        causal_report: causal,
        agency_credit: agency,
        native_weights: report.native_weights,
        native_meta_observations: report.native_meta_observations,
        controller_frozen: report.controller_frozen,
        regime_report: serde_json::to_value(learner.causal_regime_report()).unwrap(),
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
    let rt = ScientificRuntime::from_constructing_checkpoint(bytes)?;
    assert_eq!(rt.checkpoint_constructing()?, bytes);
    Ok(rt)
}

fn write_json(path: &Path, value: &Value) -> EvalResult<()> {
    let mut file = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn checkpoint(out_dir: &Path, filename: &str, bytes: &[u8]) -> EvalResult<Value> {
    let _ = restore_exact(bytes)?;
    let path = out_dir.join(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, bytes)?;
    assert_eq!(std::fs::read(&path)?, bytes);
    Ok(
        json!({ "file":filename, "bytes":bytes.len(), "fingerprint":aeterna_v1::construction::vm::fingerprint(bytes), "roundtrip_exact":true }),
    )
}

fn regime_work(value: &Value) -> u64 {
    value
        .get("extra_inference_work")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

#[derive(Default, Serialize)]
struct Totals {
    episodes: usize,
    successes: usize,
    terminal_failures: usize,
    timeouts: usize,
    actions: usize,
    sensor_queries: usize,
    noise_queries: usize,
    terminal_actions: usize,
    prequential_nll_sum: f64,
    prequential_nll_count: usize,
    computation_work: u64,
    readonly_belief_work: u64,
    extra_inference_work: u64,
}
impl Totals {
    fn add_episode(&mut self, ep: &Totals) {
        self.episodes += ep.episodes;
        self.successes += ep.successes;
        self.terminal_failures += ep.terminal_failures;
        self.timeouts += ep.timeouts;
        self.actions += ep.actions;
        self.sensor_queries += ep.sensor_queries;
        self.noise_queries += ep.noise_queries;
        self.terminal_actions += ep.terminal_actions;
        self.prequential_nll_sum += ep.prequential_nll_sum;
        self.prequential_nll_count += ep.prequential_nll_count;
        self.computation_work += ep.computation_work;
        self.readonly_belief_work += ep.readonly_belief_work;
        self.extra_inference_work += ep.extra_inference_work;
    }
}

fn run_condition(
    rt: &mut ScientificRuntime,
    world: &WorldSpec,
    seed: u64,
    suite: &str,
    arm: &str,
    condition: &str,
    schedule: &[EpisodeSpec],
    segments: &[Segment],
    out_dir: &Path,
) -> EvalResult<Value> {
    let started = Instant::now();
    let before = snapshot(rt);
    assert!(!before.controller_frozen);
    let trace_file = format!("{condition}-{arm}.jsonl");
    let mut trace = BufWriter::new(File::create(out_dir.join(&trace_file))?);
    let offers = vec![
        ActionOffer {
            available: true,
            max_cost: 1
        };
        ACTIONS
    ];
    let goal = world.goal();
    let mut totals = Totals::default();
    let mut segment_totals: Vec<_> = segments.iter().map(|_| Totals::default()).collect();
    let mut episode_results = Vec::with_capacity(schedule.len());
    let mut callbacks = 0usize;
    let mut prior_permit = None;
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut operation_counts = BTreeMap::<String, usize>::new();
    let mut peak_entries = before.causal_report.retained_entries;
    let mut peak_model_bytes = before.causal_report.model_bytes;
    for scheduled in schedule {
        let mut current = world.observation(0);
        rt.observe_constructing_external_restart(current.clone())?;
        let mut history = Vec::<HistoryStep>::new();
        let mut ep = Totals {
            episodes: 1,
            ..Default::default()
        };
        let mut ep_success = false;
        let mut ep_terminal = false;
        let ep_work_before = rt.unified_report().unwrap().computation_work;
        let ep_regime_work_before = regime_work(&serde_json::to_value(
            rt.construction_learner().unwrap().causal_regime_report(),
        )?);
        for step_number in 0..EPISODE_LIMIT {
            assert_eq!(rt.construction_learner().unwrap().current(), current);
            let pre = current.clone();
            let before_causal = rt.construction_learner().unwrap().causal_report().unwrap();
            let regime_before =
                serde_json::to_value(rt.construction_learner().unwrap().causal_regime_report())?;
            let work_before = rt.unified_report().unwrap().computation_work;
            let (pre_beliefs, observer_work) = rt
                .construction_learner()
                .unwrap()
                .causal_action_beliefs(&offers, Some(&goal), BELIEF_READ_WORK)?;
            ep.readonly_belief_work += observer_work;
            let mut actual = None;
            let result = rt.step_constructing_with_action_limit(
                Some(&goal),
                &offers,
                Some(EPISODE_LIMIT - step_number),
                safe,
                |action| {
                    assert!(actual.is_none());
                    callbacks += 1;
                    let post = world.execute(scheduled, step_number, action);
                    let returned = UnifiedPost {
                        observation: post.post.clone(),
                        cost: 1,
                        reward: f32::from(post.success),
                    };
                    actual = Some(post);
                    Ok(returned)
                },
            )?;
            let post = actual.expect("one factual callback per chosen action");
            assert_eq!(post.action, result.action);
            ep.actions += 1;
            assert_eq!(callbacks, totals.actions + ep.actions);
            assert_eq!(result.event, before.factual_events + callbacks);
            let learner = rt.construction_learner().unwrap();
            assert_eq!(learner.current(), post.post);
            let record = learner.records().last().unwrap();
            assert_eq!(record.transition.pre, pre);
            assert_eq!(record.transition.action, result.action);
            assert_eq!(record.transition.post, post.post);
            assert_eq!(record.external_cost, 1);
            if let Some(previous) = prior_permit {
                assert!(record.permit_sequence > previous);
            }
            prior_permit = Some(record.permit_sequence);
            let after_causal = learner.causal_report().unwrap();
            let regime_after = serde_json::to_value(learner.causal_regime_report())?;
            assert_eq!(
                after_causal.factual_transitions,
                before.causal_report.factual_transitions + callbacks as u64
            );
            assert_eq!(
                after_causal.learned_transitions,
                before.causal_report.learned_transitions + callbacks as u64
            );
            peak_entries = peak_entries.max(after_causal.retained_entries);
            peak_model_bytes = peak_model_bytes.max(after_causal.model_bytes);
            let selected_belief = pre_beliefs
                .iter()
                .find(|belief| belief.action == result.action)
                .expect("selected action offered");
            let probability: f64 = selected_belief
                .outcomes
                .iter()
                .filter(|(outcome, _)| outcome == &post.post)
                .map(|(_, probability)| probability)
                .sum();
            assert!(probability.is_finite() && (0.0..=1.000000001).contains(&probability));
            let nll = -probability.max(1e-12).ln();
            ep.prequential_nll_sum += nll;
            ep.prequential_nll_count += 1;
            ep.sensor_queries += usize::from(post.role == 0);
            ep.noise_queries += usize::from(post.role == 1);
            ep.terminal_actions += usize::from(post.terminal);
            *source_counts.entry(result.source.clone()).or_default() += 1;
            for operation in &result.operations {
                *operation_counts.entry(operation.clone()).or_default() += 1;
            }
            // Only after execution are hidden evaluator annotations emitted.
            serde_json::to_writer(
                &mut trace,
                &json!({
                    "schema":"aeterna.integral-phase-context.step.v1",
                    "seed":seed,"suite":suite,"world":world.world,"mapping_seed":world.mapping_seed,
                    "arm":arm,"condition":condition,"episode":scheduled.episode,"step":step_number,
                    "segment":scheduled.segment,"rule":scheduled.rule,"hidden":scheduled.hidden,"sensor_cue":scheduled.hidden,
                    "pre":&pre,"history":&history,"pre_beliefs":pre_beliefs,
                    "readonly_belief_work":observer_work,"prequential_probability":probability,"prequential_nll":nll,
                    "action":result.action,"role":ROLE_NAMES[post.role],"source":result.source,
                    "selected_fields":result.selected_fields,"native_score":result.native_score,"factual_utility":result.factual_utility,
                    "operations":result.operations,"computation_work":result.computation_work,
                    "computation_work_before":work_before,"computation_work_delta":result.computation_work-work_before,
                    "post":&post.post,"reward":f32::from(post.success),"cost":1,
                    "terminal":post.terminal,"success":post.success,"event":result.event,"callbacks_total":callbacks,
                    "permit_sequence":record.permit_sequence,"vm_prediction":&record.prediction,
                    "learned_fingerprint_before":before_causal.learned_fingerprint,"learned_fingerprint_after":after_causal.learned_fingerprint,
                    "factual_transitions_before":before_causal.factual_transitions,"factual_transitions_after":after_causal.factual_transitions,
                    "learned_transitions_before":before_causal.learned_transitions,"learned_transitions_after":after_causal.learned_transitions,
                    "causal_retained_entries":after_causal.retained_entries,"causal_model_bytes":after_causal.model_bytes,
                    "regime_before":regime_before,"regime_after":regime_after,
                }),
            )?;
            trace.write_all(b"\n")?;
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
        ep.successes = usize::from(ep_success);
        ep.terminal_failures = usize::from(ep_terminal && !ep_success);
        ep.timeouts = usize::from(!ep_terminal);
        ep.computation_work = rt.unified_report().unwrap().computation_work - ep_work_before;
        ep.extra_inference_work = regime_work(&serde_json::to_value(
            rt.construction_learner().unwrap().causal_regime_report(),
        )?) - ep_regime_work_before;
        let mut row = serde_json::to_value(&ep)?;
        let row = row.as_object_mut().unwrap();
        row.insert("episode".into(), json!(scheduled.episode));
        row.insert("segment".into(), json!(scheduled.segment));
        row.insert("rule".into(), json!(scheduled.rule));
        row.insert("hidden".into(), json!(scheduled.hidden));
        row.insert("sensor_cue".into(), json!(scheduled.hidden));
        row.insert("success".into(), json!(ep_success));
        row.insert("terminal".into(), json!(ep_terminal));
        episode_results.push(Value::Object(row.clone()));
        segment_totals[scheduled.segment].add_episode(&ep);
        totals.add_episode(&ep);
    }
    trace.flush()?;
    let after = snapshot(rt);
    assert_eq!(after.factual_events - before.factual_events, callbacks);
    assert_eq!(
        after.total_external_cost - before.total_external_cost,
        callbacks as u64
    );
    assert_eq!(
        after.agency_credit.self_selected_actions - before.agency_credit.self_selected_actions,
        callbacks as u64
    );
    assert!(!after.controller_frozen);
    assert_eq!(
        totals.computation_work,
        after.computation_work - before.computation_work
    );
    assert_eq!(
        totals.extra_inference_work,
        regime_work(&after.regime_report) - regime_work(&before.regime_report)
    );
    assert!(peak_entries <= CausalConfig::default().model_entries);
    assert!(peak_model_bytes <= CausalConfig::default().model_bytes);
    let segment_results: Vec<_> = segments
        .iter()
        .zip(segment_totals.iter())
        .map(|(segment, totals)| json!({"segment":segment,"totals":totals}))
        .collect();
    let mut result = serde_json::to_value(&totals)?;
    let object = result.as_object_mut().unwrap();
    object.insert("arm".into(), json!(arm));
    object.insert("condition".into(), json!(condition));
    object.insert("physical_callbacks".into(), json!(callbacks));
    object.insert("source_counts".into(), json!(source_counts));
    object.insert("operation_counts".into(), json!(operation_counts));
    object.insert("episode_results".into(), json!(episode_results));
    object.insert("segments".into(), json!(segment_results));
    object.insert("before".into(), json!(before));
    object.insert("after".into(), json!(after));
    object.insert("peak_retained_entries".into(), json!(peak_entries));
    object.insert("peak_model_bytes".into(), json!(peak_model_bytes));
    object.insert("trace_file".into(), json!(trace_file));
    object.insert(
        "wall_seconds".into(),
        json!(started.elapsed().as_secs_f64()),
    );
    eprintln!("PHASE_CONTEXT suite={suite} world={} arm={arm} condition={condition} goals={}/{} actions={}",world.world,totals.successes,totals.episodes,totals.actions);
    Ok(result)
}

fn main() -> EvalResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !matches!(args.len(), 8 | 10) {
        return Err("usage: integral_phase_context_eval --seed <u64> --suite fresh|retained --world <index> --out-dir <directory> [--checkpoint <retained-live-checkpoint>]".into());
    }
    let (mut seed, mut suite, mut world_index, mut out_dir, mut input_checkpoint) =
        (None, None, None, None, None);
    for pair in args.chunks_exact(2) {
        match pair[0].as_str() {
            "--seed" if seed.is_none() => seed = Some(pair[1].parse::<u64>()?),
            "--suite" if suite.is_none() => suite = Some(pair[1].clone()),
            "--world" if world_index.is_none() => world_index = Some(pair[1].parse::<usize>()?),
            "--out-dir" if out_dir.is_none() => out_dir = Some(PathBuf::from(&pair[1])),
            "--checkpoint" if input_checkpoint.is_none() => {
                input_checkpoint = Some(PathBuf::from(&pair[1]))
            }
            _ => return Err("unknown or repeated argument".into()),
        }
    }
    let seed = seed.ok_or("missing --seed")?;
    let suite = suite.ok_or("missing --suite")?;
    let world_index = world_index.ok_or("missing --world")?;
    let initial_rule = match suite.as_str() {
        "fresh" if world_index < 16 && input_checkpoint.is_none() => 0,
        "retained" if world_index < 8 && input_checkpoint.is_some() => 1,
        _ => return Err("fresh requires world0..15 and no checkpoint; retained requires world0..7 and live checkpoint".into()),
    };
    let out_dir = out_dir.ok_or("missing --out-dir")?;
    std::fs::create_dir_all(&out_dir)?;
    let mapping_seed = if suite == "retained" {
        RETAINED_MAPPING_SEED
    } else {
        seed
    };
    let world = WorldSpec::new(mapping_seed, world_index);
    let base = private_seed(seed, &suite, world_index);
    let segments = switching_segments(base, initial_rule);
    let switching = make_schedule(base, "switching", &segments);
    let stationary_segments = vec![Segment {
        index: 0,
        start_episode: 0,
        end_episode: STATIONARY_EPISODES,
        length: STATIONARY_EPISODES,
        rule: initial_rule,
    }];
    let stationary = make_schedule(base, "stationary", &stationary_segments);
    let acquisition_segments = vec![Segment {
        index: 0,
        start_episode: 0,
        end_episode: ACQUISITION_EPISODES,
        length: ACQUISITION_EPISODES,
        rule: 0,
    }];
    let acquisition = if suite == "fresh" {
        make_schedule(base, "acquisition", &acquisition_segments)
    } else {
        Vec::new()
    };
    write_json(
        &out_dir.join("schedule.json"),
        &json!({
            "schema":"aeterna.integral-phase-context.schedule.v1","seed":seed,"suite":suite,"world":world_index,"mapping_seed":mapping_seed,
            "motors":world.motors,"codes":world.codes,"segments":segments,
            "conditions":{"switching":switching,"stationary":stationary,"acquisition":acquisition},
            "actor_receives_schedule":false,
        }),
    )?;
    let mut checkpoints = Vec::new();
    let mut input_identity = Value::Null;
    let (common_bytes, acquisition_result) = if let Some(path) = input_checkpoint {
        let bytes = std::fs::read(&path)?;
        let restored = restore_exact(&bytes)?;
        assert!(!snapshot(&restored).controller_frozen);
        assert!(restored
            .construction_learner()
            .unwrap()
            .causal_regime_report()
            .is_none());
        input_identity = json!({"file":path,"bytes":bytes.len(),"fingerprint":aeterna_v1::construction::vm::fingerprint(&bytes),"roundtrip_exact":true});
        (bytes, Value::Null)
    } else {
        let mut acquired = make_runtime(&world)?;
        let result = run_condition(
            &mut acquired,
            &world,
            seed,
            &suite,
            "baseline",
            "acquisition",
            &acquisition,
            &acquisition_segments,
            &out_dir,
        )?;
        (acquired.checkpoint_constructing()?, result)
    };
    let common = restore_exact(&common_bytes)?;
    let common_before = snapshot(&common);
    assert!(common_before.regime_report.is_null());
    assert!(!common_before.controller_frozen);
    let common_identity = checkpoint(&out_dir, "common.checkpoint.json", &common_bytes)?;
    checkpoints.push(common_identity.clone());
    let mut results = Vec::new();
    let mut arm_starts = Vec::new();
    for arm in ARMS {
        let mut initial = restore_exact(&common_bytes)?;
        assert_eq!(initial.checkpoint_constructing()?, common_bytes);
        let model_before = serde_json::to_value(snapshot(&initial).causal_report)?;
        let mode = match arm {
            "baseline" => None,
            "ordinary" => Some(RegimeMode::Ordinary),
            "phase" => Some(RegimeMode::Phase),
            "phase_scrambled" => Some(RegimeMode::PhaseScrambled),
            "phase_uncoupled" => Some(RegimeMode::PhaseUncoupled),
            _ => unreachable!(),
        };
        if let Some(mode) = mode {
            initial.enable_constructing_regime_inference(mode)?;
        }
        let after_enable = snapshot(&initial);
        let mut normalized_model = serde_json::to_value(&after_enable.causal_report)?;
        let reserved_state_bytes = after_enable
            .regime_report
            .get("state_bytes")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        normalized_model["model_bytes"] =
            json!(after_enable.causal_report.model_bytes as u64 - reserved_state_bytes);
        assert_eq!(normalized_model, model_before);
        assert_eq!(after_enable.factual_events, common_before.factual_events);
        assert_eq!(
            after_enable.total_external_cost,
            common_before.total_external_cost
        );
        assert_eq!(
            after_enable.computation_work,
            common_before.computation_work
        );
        assert_eq!(after_enable.native_weights, common_before.native_weights);
        let initial_bytes = initial.checkpoint_constructing()?;
        if arm == "baseline" {
            assert_eq!(initial_bytes, common_bytes);
        }
        let start_identity = checkpoint(
            &out_dir,
            &format!("starts/{arm}.checkpoint.json"),
            &initial_bytes,
        )?;
        checkpoints.push(start_identity.clone());
        arm_starts.push(json!({"arm":arm,"common_source_exact":true,"model_equivalent":true,"checkpoint":start_identity,"snapshot":after_enable}));
        for (condition, schedule, condition_segments) in [
            ("switching", &switching, &segments),
            ("stationary", &stationary, &stationary_segments),
        ] {
            let mut actor = restore_exact(&initial_bytes)?;
            assert_eq!(actor.checkpoint_constructing()?, initial_bytes);
            let mut result = run_condition(
                &mut actor,
                &world,
                seed,
                &suite,
                arm,
                condition,
                schedule,
                condition_segments,
                &out_dir,
            )?;
            let final_bytes = actor.checkpoint_constructing()?;
            let final_identity = checkpoint(
                &out_dir,
                &format!("finals/{condition}-{arm}.checkpoint.json"),
                &final_bytes,
            )?;
            checkpoints.push(final_identity.clone());
            let mut frozen = restore_exact(&final_bytes)?;
            frozen.freeze_constructing()?;
            assert!(snapshot(&frozen).controller_frozen);
            let frozen_identity = checkpoint(
                &out_dir,
                &format!("finals/{condition}-{arm}.frozen.checkpoint.json"),
                &frozen.checkpoint_constructing()?,
            )?;
            checkpoints.push(frozen_identity.clone());
            assert_eq!(actor.checkpoint_constructing()?, final_bytes);
            let result = result.as_object_mut().unwrap();
            result.insert("checkpoint_file".into(), final_identity["file"].clone());
            result.insert(
                "frozen_checkpoint_file".into(),
                frozen_identity["file"].clone(),
            );
            result.insert("checkpoint".into(), final_identity);
            result.insert("frozen_checkpoint".into(), frozen_identity);
            result.insert("start_checkpoint".into(), start_identity.clone());
            results.push(Value::Object(result.clone()));
        }
    }
    assert_eq!(common.checkpoint_constructing()?, common_bytes);
    let integrity = json!({
        "schema":"aeterna.integral-phase-context.integrity.v1","seed":seed,"suite":suite,"world":world_index,
        "common_source_exact":true,"baseline_clone_exact":true,"arm_start_model_equivalent":true,"final_roundtrips_exact":true,
        "common_checkpoint":common_identity,"input_checkpoint":input_identity,"checkpoints":checkpoints,
        "equal_offers_all_four_actions":true,"no_forced_runtime_actions":true,"runtime_callback_matches_selected_action":true,
        "runtime_journal_matches_physical_pre_post":true,"runtime_cost_and_transition_counts_match_callbacks":true,
        "agency_ledger_matches_learning_actions":true,"paired_exogenous_schedules":true,"scoring_only_latent_metadata":true,
        "branches_independent_from_same_common_state":true,"checkpoint_freeze_checks_execute_no_actions":true,
    });
    write_json(&out_dir.join("integrity.json"), &integrity)?;
    let summary = json!({
        "schema":"aeterna.integral-phase-context.world.v1","seed":seed,"suite":suite,"world":world_index,"mapping_seed":mapping_seed,
        "initial_rule":initial_rule,"motor_roles":{"informative_sensor":world.motors[0],"noise_sensor":world.motors[1],"actuator0":world.motors[2],"actuator1":world.motors[3]},
        "observation_codes":{"start":world.codes[0],"sense0":world.codes[1],"sense1":world.codes[2],"noise0":world.codes[3],"noise1":world.codes[4],"success":world.codes[5],"failure":world.codes[6]},
        "config":{"width":2,"actions":ACTIONS,"minimum_training":128,"search_every":1_000_000,"planning_depth":4,"planning_work":120_000,
            "history_depth":3,"model_entries":2048,"model_bytes":2*1024*1024,"outcomes_per_entry":16,"planning_states":192,
            "journal_capacity":4096,"max_external_cost":100_000,"episode_action_limit":EPISODE_LIMIT,
            "acquisition_episodes":ACQUISITION_EPISODES,"switching_episodes":SWITCHING_EPISODES,"stationary_episodes":STATIONARY_EPISODES,
            "readonly_belief_work_per_call":BELIEF_READ_WORK,"readonly_diagnostics_outside_actor_budget":true,
            "agency_credit_counts_learning_actions_only":true,"all_behavioral_episodes_learning_enabled":true,
            "nll_floor":1e-12,"phase_parameter_selection_on_test_results":false},
        "common_checkpoint":common_identity,"input_checkpoint":input_identity,"common_before":common_before,
        "acquisition":acquisition_result,"segments":segments,"arm_starts":arm_starts,"results":results,
        "integrity_file":"integrity.json","schedule_file":"schedule.json",
    });
    write_json(&out_dir.join("summary.json"), &summary)?;
    println!(
        "{}",
        json!({"suite":suite,"world":world_index,"summary":out_dir.join("summary.json"),"completed":true})
    );
    Ok(())
}
