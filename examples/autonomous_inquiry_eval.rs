//! Prospective, closed-loop evaluation of persistent autonomous inquiry.
//! Private schedules and action meanings never enter the runtime. Every
//! action, including experiments, is chosen by the existing protected U1.
#![recursion_limit = "256"]
use aeterna_v1::construction::autonomy::{ActionOffer, AutonomyPolicy};
use aeterna_v1::construction::causal_memory::{CausalConfig, InquiryMode, RegimeMode};
use aeterna_v1::construction::Config;
use aeterna_v1::scientific_runtime::{ScientificRuntime, UnifiedPost};
use aeterna_v1::HumanProtectionEvidence;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

type EvalResult<T> = Result<T, Box<dyn std::error::Error>>;
const ACTIONS: usize = 5;
const LIMIT: usize = 5;
const ACQUISITION: usize = 768;
const SWITCHING: usize = 1024;
const STATIONARY: usize = 256;
const ARMS: [&str; 3] = ["ordinary", "active", "observe_only"];
const ROLES: [&str; ACTIONS] = [
    "cue_sensor",
    "diagnostic_sensor",
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
struct World {
    index: usize,
    motors: [usize; ACTIONS],
    codes: [i64; 9],
}
impl World {
    fn new(seed: u64, index: usize) -> Self {
        let base = mix(seed ^ (index as u64).wrapping_mul(0xd134_2543_de82_ef95));
        let mut motors = [0, 1, 2, 3, 4];
        let mut codes = [101, 211, 307, 419, 503, 617, 733, 829, 947];
        Rng(base ^ 0x696e_7175_6972_795f).shuffle(&mut motors);
        Rng(base ^ 0x6f70_6171_7565_5f63).shuffle(&mut codes);
        Self {
            index,
            motors,
            codes,
        }
    }
    fn obs(&self, state: usize) -> Vec<i64> {
        vec![0, self.codes[state]]
    }
    fn goal(&self) -> [Option<i64>; 2] {
        [None, Some(self.codes[7])]
    }
    fn execute(&self, ep: &Episode, step: usize, action: usize) -> PhysicalPost {
        let role = self.motors.iter().position(|&a| a == action).unwrap();
        let terminal = role >= 3;
        let success = terminal && role - 3 == (ep.hidden ^ ep.rule);
        let state = match role {
            0 => 1 + ep.hidden,
            1 => 3 + ep.rule,
            2 => 5 + ep.noise[step],
            _ if success => 7,
            _ => 8,
        };
        PhysicalPost {
            action,
            role,
            post: self.obs(state),
            terminal,
            success,
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
struct Segment {
    index: usize,
    start: usize,
    end: usize,
    rule: usize,
}
#[derive(Clone, Serialize)]
struct Episode {
    episode: usize,
    segment: usize,
    rule: usize,
    hidden: usize,
    noise: [usize; LIMIT],
}
fn segments(seed: u64, length: usize, count: usize, minimum: usize, first: usize) -> Vec<Segment> {
    assert!(length >= count * minimum);
    let mut rng = Rng(seed);
    let weights: Vec<_> = (0..count).map(|_| 1 + rng.next() % 1024).collect();
    let total = weights.iter().sum::<u64>();
    let remainder = length - count * minimum;
    let mut lengths: Vec<_> = weights
        .iter()
        .map(|w| minimum + (remainder as u64 * w / total) as usize)
        .collect();
    let mut order: Vec<_> = (0..count).collect();
    rng.shuffle(&mut order);
    for i in order
        .into_iter()
        .take(length - lengths.iter().sum::<usize>())
    {
        lengths[i] += 1;
    }
    let mut start = 0;
    lengths
        .into_iter()
        .enumerate()
        .map(|(index, length)| {
            let segment = Segment {
                index,
                start,
                end: start + length,
                rule: first ^ (index & 1),
            };
            start += length;
            segment
        })
        .collect()
}
fn schedule(seed: u64, segments: &[Segment]) -> Vec<Episode> {
    segments
        .iter()
        .flat_map(|s| {
            (s.start..s.end).map(move |episode| Episode {
                episode,
                segment: s.index,
                rule: s.rule,
                hidden: (mix(seed
                    ^ 0x6869_6464_656e_6269
                    ^ (episode as u64).wrapping_mul(0xa24b_aed4_963e_e407))
                    & 1) as usize,
                noise: std::array::from_fn(|step| {
                    (mix(seed
                        ^ 0x6e6f_6973_655f_6269
                        ^ (episode as u64).wrapping_mul(0x9fb2_1c65_1e98_df25)
                        ^ (step as u64).wrapping_mul(0xc13f_a9a9_02a6_328f))
                        & 1) as usize
                }),
            })
        })
        .collect()
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
fn make_runtime(world: &World) -> EvalResult<ScientificRuntime> {
    let mut rt = ScientificRuntime::new_constructing(
        Config {
            width: 2,
            actions: ACTIONS,
            minimum_training: 128,
            search_every: 1_000_000,
            planning_depth: LIMIT,
            planning_work: 120_000,
            journal_capacity: 4096,
            ..Default::default()
        },
        AutonomyPolicy {
            max_external_cost: 100_000,
            ..Default::default()
        },
        world.obs(0),
        "autonomous-inquiry",
    )?;
    rt.enable_constructing_agency_aware_credit()?;
    rt.enable_constructing_self_directed_control()?;
    rt.enable_constructing_causal_memory(CausalConfig {
        history_depth: 3,
        ..Default::default()
    })?;
    rt.enable_constructing_regime_inference(RegimeMode::Ordinary)?;
    Ok(rt)
}
fn restore(bytes: &[u8]) -> EvalResult<ScientificRuntime> {
    let rt = ScientificRuntime::from_constructing_checkpoint(bytes)?;
    assert_eq!(rt.checkpoint_constructing()?, bytes);
    Ok(rt)
}
fn snapshot(rt: &ScientificRuntime) -> Value {
    let learner = rt.construction_learner().unwrap();
    let runtime = rt.unified_report().unwrap();
    let causal = learner.causal_report().unwrap();
    let agency = runtime.agency_credit.as_ref().unwrap();
    assert_eq!(agency.externally_forced_actions, 0);
    assert_eq!(agency.option_continuation_actions, 0);
    assert_eq!(agency.self_selected_actions, causal.learned_transitions);
    assert_eq!(
        causal.factual_transitions,
        learner.factual_event_count() as u64
    );
    assert_eq!(runtime.total_external_cost, causal.factual_transitions);
    json!({"causal":causal, "regime":learner.causal_regime_report(), "inquiry":learner.causal_inquiry_report(),
        "events":learner.factual_event_count(), "cost":runtime.total_external_cost, "work":runtime.computation_work,
        "agency":agency, "native_weights":runtime.native_weights, "native_meta_observations":runtime.native_meta_observations,
        "frozen":runtime.controller_frozen})
}
fn write_json(path: &Path, value: &Value) -> EvalResult<()> {
    let mut stream = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut stream, value)?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}
fn checkpoint(out: &Path, name: &str, bytes: &[u8]) -> EvalResult<Value> {
    let _ = restore(bytes)?;
    let path = out.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(
        json!({"file":name,"bytes":bytes.len(),"fingerprint":aeterna_v1::construction::vm::fingerprint(bytes),"roundtrip_exact":true}),
    )
}
#[derive(Default, Serialize)]
struct Totals {
    episodes: usize,
    successes: usize,
    failures: usize,
    timeouts: usize,
    actions: usize,
    cue_queries: usize,
    diagnostic_queries: usize,
    noise_queries: usize,
    terminal_actions: usize,
    inquiry_source_actions: usize,
    predictive_nll: f64,
    computation_work: u64,
    readonly_work: u64,
}
impl Totals {
    fn add(&mut self, t: &Self) {
        self.episodes += t.episodes;
        self.successes += t.successes;
        self.failures += t.failures;
        self.timeouts += t.timeouts;
        self.actions += t.actions;
        self.cue_queries += t.cue_queries;
        self.diagnostic_queries += t.diagnostic_queries;
        self.noise_queries += t.noise_queries;
        self.terminal_actions += t.terminal_actions;
        self.inquiry_source_actions += t.inquiry_source_actions;
        self.predictive_nll += t.predictive_nll;
        self.computation_work += t.computation_work;
        self.readonly_work += t.readonly_work;
    }
}
fn run(
    rt: &mut ScientificRuntime,
    world: &World,
    arm: &str,
    condition: &str,
    schedule: &[Episode],
    out: &Path,
) -> EvalResult<Value> {
    let started = Instant::now();
    let before = snapshot(rt);
    let base_events = before["events"].as_u64().unwrap();
    let file = format!("{condition}-{arm}.jsonl");
    let mut trace = BufWriter::new(File::create(out.join(&file))?);
    let offers = vec![
        ActionOffer {
            available: true,
            max_cost: 1
        };
        ACTIONS
    ];
    let goal = world.goal();
    let mut totals = Totals::default();
    let mut episodes = Vec::new();
    let mut sources = BTreeMap::<String, usize>::new();
    let mut previous_permit = None;
    let mut peak_entries = 0;
    let mut peak_bytes = 0;
    let mut midpoint = Value::Null;
    for scheduled in schedule {
        if scheduled.episode == schedule.len() / 2 {
            let saved = rt.checkpoint_constructing()?;
            midpoint = checkpoint(
                out,
                &format!("checkpoints/{condition}-{arm}.midpoint.json"),
                &saved,
            )?;
            let before_restore = snapshot(rt);
            *rt = restore(&saved)?;
            assert_eq!(snapshot(rt), before_restore);
        }
        rt.observe_constructing_external_restart(world.obs(0))?;
        let mut current = world.obs(0);
        let mut history = Vec::<Value>::new();
        let mut ep = Totals {
            episodes: 1,
            ..Default::default()
        };
        let start_work = rt.unified_report().unwrap().computation_work;
        let mut terminal = false;
        let mut success = false;
        for step in 0..LIMIT {
            let learner = rt.construction_learner().unwrap();
            assert_eq!(learner.current(), current);
            let pre = current.clone();
            let causal_before = learner.causal_report().unwrap();
            let regime_before = serde_json::to_value(learner.causal_regime_report())?;
            let inquiry_before = serde_json::to_value(learner.causal_inquiry_report())?;
            let (beliefs, readonly_work) =
                learner.causal_action_beliefs(&offers, Some(&goal), 120_000)?;
            // Diagnostic reads must not advance inference or inquiry.
            assert_eq!(
                serde_json::to_value(learner.causal_regime_report())?,
                regime_before
            );
            assert_eq!(
                serde_json::to_value(learner.causal_inquiry_report())?,
                inquiry_before
            );
            let mut actual = None;
            let result = rt.step_constructing_with_action_limit(
                Some(&goal),
                &offers,
                Some(LIMIT - step),
                safe,
                |action| {
                    assert!(actual.is_none());
                    let post = world.execute(scheduled, step, action);
                    let returned = UnifiedPost {
                        observation: post.post.clone(),
                        cost: 1,
                        reward: f32::from(post.success),
                    };
                    actual = Some(post);
                    Ok(returned)
                },
            )?;
            let post = actual.ok_or("missing protected physical callback")?;
            assert_eq!(result.action, post.action);
            ep.actions += 1;
            let learner = rt.construction_learner().unwrap();
            let record = learner
                .records()
                .last()
                .ok_or("missing factual journal record")?;
            assert_eq!(record.transition.pre, pre);
            assert_eq!(record.transition.post, post.post);
            assert_eq!(record.transition.action, post.action);
            assert_eq!(record.external_cost, 1);
            assert!(previous_permit.is_none_or(|p| p < record.permit_sequence));
            previous_permit = Some(record.permit_sequence);
            let causal_after = learner.causal_report().unwrap();
            assert_eq!(
                causal_after.factual_transitions,
                base_events + (totals.actions + ep.actions) as u64
            );
            assert_eq!(
                causal_after.learned_transitions,
                causal_before.learned_transitions + 1
            );
            assert_eq!(result.event as u64, causal_after.factual_transitions);
            peak_entries = peak_entries.max(causal_after.retained_entries);
            peak_bytes = peak_bytes.max(causal_after.model_bytes);
            let belief = beliefs
                .iter()
                .find(|b| b.action == post.action)
                .ok_or("chosen action absent from offers")?;
            let probability = belief
                .outcomes
                .iter()
                .filter(|(p, _)| p == &post.post)
                .map(|(_, p)| *p)
                .sum::<f64>();
            assert!(probability.is_finite() && (0.0..=1.000000001).contains(&probability));
            let nll = -probability.max(1e-12).ln();
            ep.predictive_nll += nll;
            ep.readonly_work += readonly_work;
            ep.cue_queries += usize::from(post.role == 0);
            ep.diagnostic_queries += usize::from(post.role == 1);
            ep.noise_queries += usize::from(post.role == 2);
            ep.terminal_actions += usize::from(post.terminal);
            ep.inquiry_source_actions += usize::from(result.source == "causal-belief-inquiry");
            *sources.entry(result.source.clone()).or_default() += 1;
            // Hidden evaluator annotations are emitted only AFTER execution.
            serde_json::to_writer(
                &mut trace,
                &json!({
                    "schema":"aeterna.autonomous-inquiry.step.v1", "world":world.index,"arm":arm,"condition":condition,
                    "episode":scheduled.episode,"step":step,"segment":scheduled.segment,"rule":scheduled.rule,"hidden":scheduled.hidden,
                    "pre":pre,"history":history,"pre_beliefs":beliefs,"prequential_probability":probability,"nll":nll,
                    "action":result.action,"role":ROLES[post.role],"post":post.post,"terminal":post.terminal,"success":post.success,
                    "source":result.source,"selected_fields":result.selected_fields,"operations":result.operations,
                    "inquiry_decision":result.inquiry_decision,
                    "native_score":result.native_score,"reward":f32::from(post.success),"cost":1,"event":result.event,
                    "permit_sequence":record.permit_sequence,"learned_before":causal_before.learned_transitions,"learned_after":causal_after.learned_transitions,
                    "factual_before":causal_before.factual_transitions,"factual_after":causal_after.factual_transitions,
                    "learned_fingerprint_before":causal_before.learned_fingerprint,"learned_fingerprint_after":causal_after.learned_fingerprint,
                    "model_bytes":causal_after.model_bytes,"retained_entries":causal_after.retained_entries,
                    "regime_before":regime_before,"regime_after":learner.causal_regime_report(),
                    "inquiry_before":inquiry_before,"inquiry_after":learner.causal_inquiry_report(),
                    "readonly_work":readonly_work,"computation_work":result.computation_work
                }),
            )?;
            trace.write_all(b"\n")?;
            history.push(json!({"pre":current,"action":post.action,"post":post.post}));
            if history.len() > 3 {
                history.remove(0);
            }
            current = post.post;
            terminal = post.terminal;
            success = post.success;
            if terminal {
                break;
            }
        }
        ep.successes = usize::from(success);
        ep.failures = usize::from(terminal && !success);
        ep.timeouts = usize::from(!terminal);
        ep.computation_work = rt.unified_report().unwrap().computation_work - start_work;
        episodes.push(json!({"episode":scheduled.episode,"segment":scheduled.segment,"rule":scheduled.rule,"hidden":scheduled.hidden,
            "success":success,"terminal":terminal,"totals":ep}));
        totals.add(&ep);
    }
    trace.flush()?;
    let after = snapshot(rt);
    assert_eq!(
        after["events"].as_u64().unwrap() - base_events,
        totals.actions as u64
    );
    assert_eq!(
        after["cost"].as_u64().unwrap() - before["cost"].as_u64().unwrap(),
        totals.actions as u64
    );
    assert_eq!(
        after["work"].as_u64().unwrap() - before["work"].as_u64().unwrap(),
        totals.computation_work
    );
    assert!(peak_entries <= 2048 && peak_bytes <= 2 * 1024 * 1024);
    let final_bytes = rt.checkpoint_constructing()?;
    let final_identity = checkpoint(
        out,
        &format!("checkpoints/{condition}-{arm}.final.json"),
        &final_bytes,
    )?;
    // Freeze a clone, never the continuing organism. Checkpoints remain exact.
    let mut frozen = restore(&final_bytes)?;
    frozen.freeze_constructing()?;
    let frozen_identity = checkpoint(
        out,
        &format!("checkpoints/{condition}-{arm}.frozen.json"),
        &frozen.checkpoint_constructing()?,
    )?;
    assert_eq!(rt.checkpoint_constructing()?, final_bytes);
    eprintln!("INQUIRY world={} arm={arm} condition={condition} goals={}/{} actions={} diagnostic={} inquiry={}",
        world.index,totals.successes,totals.episodes,totals.actions,totals.diagnostic_queries,totals.inquiry_source_actions);
    Ok(
        json!({"arm":arm,"condition":condition,"totals":totals,"episodes":episodes,"sources":sources,
        "before":before,"after":after,"peak_entries":peak_entries,"peak_bytes":peak_bytes,"trace_file":file,
        "midpoint_checkpoint":midpoint,"final_checkpoint":final_identity,"frozen_checkpoint":frozen_identity,
        "wall_seconds":started.elapsed().as_secs_f64()}),
    )
}
fn main() -> EvalResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return Err(
            "usage: autonomous_inquiry_eval --seed <u64> --world <0..15> --out-dir <new-directory>"
                .into(),
        );
    }
    let (mut seed, mut world, mut out) = (None, None, None);
    for pair in args.chunks_exact(2) {
        match pair[0].as_str() {
            "--seed" if seed.is_none() => seed = Some(pair[1].parse::<u64>()?),
            "--world" if world.is_none() => world = Some(pair[1].parse::<usize>()?),
            "--out-dir" if out.is_none() => out = Some(PathBuf::from(&pair[1])),
            _ => return Err("unknown or duplicate argument".into()),
        }
    }
    let seed = seed.ok_or("missing seed")?;
    let index = world.ok_or("missing world")?;
    let out = out.ok_or("missing output directory")?;
    if index >= 16 {
        return Err("world must be 0..15".into());
    }
    std::fs::create_dir(&out)?;
    let world = World::new(seed, index);
    let base = mix(seed ^ (index as u64).wrapping_mul(0x94d0_49bb_1331_11eb));
    let acquisition_segments = segments(base ^ 0x6163_7175_6972_655f, ACQUISITION, 8, 64, 0);
    let switching_segments = segments(base ^ 0x7377_6974_6368_696e, SWITCHING, 32, 16, 1);
    let stationary_segments = vec![Segment {
        index: 0,
        start: 0,
        end: STATIONARY,
        rule: 1,
    }];
    let acquisition = schedule(base ^ 0x6163_715f_7363_6864, &acquisition_segments);
    let switching = schedule(base ^ 0x7377_695f_7363_6864, &switching_segments);
    let stationary = schedule(base ^ 0x7374_615f_7363_6864, &stationary_segments);
    write_json(
        &out.join("schedule.json"),
        &json!({"seed":seed,"world":index,"motors":world.motors,"codes":world.codes,
        "segments":{"acquisition":acquisition_segments,"switching":switching_segments,"stationary":stationary_segments},
        "episodes":{"acquisition":acquisition,"switching":switching,"stationary":stationary},"actor_receives_schedule":false}),
    )?;
    let mut common = make_runtime(&world)?;
    let acquisition_result = run(
        &mut common,
        &world,
        "ordinary",
        "acquisition",
        &acquisition,
        &out,
    )?;
    let common_bytes = common.checkpoint_constructing()?;
    let common_snapshot = snapshot(&common);
    let common_checkpoint = checkpoint(&out, "checkpoints/common.json", &common_bytes)?;
    let mut starts = Vec::new();
    let mut results = Vec::new();
    for arm in ARMS {
        let mut initial = restore(&common_bytes)?;
        let model_before = initial
            .construction_learner()
            .unwrap()
            .causal_report()
            .unwrap();
        match arm {
            "active" => initial.enable_constructing_autonomous_inquiry(InquiryMode::Active)?,
            "observe_only" => {
                initial.enable_constructing_autonomous_inquiry(InquiryMode::ObserveOnly)?
            }
            "ordinary" => (),
            _ => unreachable!(),
        }
        let model_after = initial
            .construction_learner()
            .unwrap()
            .causal_report()
            .unwrap();
        assert_eq!(
            model_before.learned_fingerprint,
            model_after.learned_fingerprint
        );
        assert_eq!(
            model_before.learned_transitions,
            model_after.learned_transitions
        );
        assert_eq!(model_before.retained_entries, model_after.retained_entries);
        let state = snapshot(&initial);
        assert_eq!(state["native_weights"], common_snapshot["native_weights"]);
        assert_eq!(
            state["native_meta_observations"],
            common_snapshot["native_meta_observations"]
        );
        assert_eq!(state["events"], common_snapshot["events"]);
        assert_eq!(state["cost"], common_snapshot["cost"]);
        let bytes = initial.checkpoint_constructing()?;
        if arm == "ordinary" {
            assert_eq!(bytes, common_bytes);
        }
        let identity = checkpoint(&out, &format!("checkpoints/start-{arm}.json"), &bytes)?;
        starts.push(json!({"arm":arm,"checkpoint":identity,"snapshot":state}));
        for (condition, schedule) in [("switching", &switching), ("stationary", &stationary)] {
            let mut actor = restore(&bytes)?;
            results.push(run(&mut actor, &world, arm, condition, schedule, &out)?);
        }
    }
    assert_eq!(common.checkpoint_constructing()?, common_bytes);
    write_json(
        &out.join("summary.json"),
        &json!({"schema":"aeterna.autonomous-inquiry.world.v1","seed":seed,"world":index,
        "config":{"width":2,"actions":ACTIONS,"action_limit":LIMIT,"acquisition_episodes":ACQUISITION,
            "switching_episodes":SWITCHING,"stationary_episodes":STATIONARY,"model_entries":2048,"model_bytes":2*1024*1024,
            "planning_states":192,"planning_work":120000,"planning_depth":LIMIT,"history_depth":3},
        "common_checkpoint":common_checkpoint,"common_snapshot":common_snapshot,"starts":starts,
        "acquisition":acquisition_result,"results":results,
        "integrity":{"no_forced_actions":true,"same_exact_common_checkpoint":true,"readonly_queries_pure":true,
            "midlife_restores_exact":true,"protected_callback_matches_journal":true,"all_episodes_learning_enabled":true,
            "hidden_annotations_after_execution":true,"paired_schedules":true,"checkpoint_clones_independent":true}}),
    )?;
    println!(
        "{}",
        json!({"world":index,"completed":true,"summary":out.join("summary.json")})
    );
    Ok(())
}
