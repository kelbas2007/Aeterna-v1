//! Runtime/transport integrity controls, not an autonomous-learning benchmark.
//! Restricted-offer fixtures isolate factual evidence and the protected motor
//! boundary; the separate evaluator measures freely selected investigations.
use aeterna_v1::construction::autonomy::{ActionOffer, AutonomyPolicy};
use aeterna_v1::construction::causal_memory::{CausalConfig, InquiryMode, RegimeMode};
use aeterna_v1::construction::{Config, Error, Learner};
use aeterna_v1::scientific_runtime::{ScientificRuntime, UnifiedPost};
use aeterna_v1::HumanProtectionEvidence;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}

fn fixture() -> ScientificRuntime {
    fixture_with_memory(CausalConfig {
        history_depth: 3,
        ..Default::default()
    })
}

fn fixture_with_memory(memory: CausalConfig) -> ScientificRuntime {
    let mut config = Config {
        minimum_training: 128,
        search_every: 1_000_000,
        journal_capacity: 256,
        planning_depth: 3,
        planning_work: 120_000,
        ..Default::default()
    };
    config.search.deterministic = true;
    let mut policy = AutonomyPolicy::default();
    policy.evidence = None;
    policy.ceiling.deterministic = true;
    policy.max_external_cost = 10_000;
    let mut rt =
        ScientificRuntime::new_constructing(config, policy, vec![0], "inquiry-contract").unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_causal_memory(memory).unwrap();
    rt
}

fn offers(first: bool, second: bool) -> [ActionOffer; 2] {
    [
        ActionOffer {
            available: first,
            max_cost: 1,
        },
        ActionOffer {
            available: second,
            max_cost: 1,
        },
    ]
}

fn fact(rt: &mut ScientificRuntime, post: i64) {
    rt.observe_constructing_external_restart(vec![0]).unwrap();
    let step = rt
        .step_constructing(Some(&[Some(11)]), &offers(true, false), safe, |action| {
            assert_eq!(action, 0);
            Ok(UnifiedPost {
                observation: vec![post],
                reward: f32::from(post == 11),
                cost: 1,
            })
        })
        .unwrap();
    assert_eq!(step.action, 0);
}

fn warm() -> ScientificRuntime {
    let mut rt = fixture();
    for _ in 0..24 {
        fact(&mut rt, 11);
    }
    rt
}

fn report(rt: &ScientificRuntime) -> Value {
    serde_json::to_value(
        rt.construction_learner()
            .unwrap()
            .causal_inquiry_report()
            .unwrap(),
    )
    .unwrap()
}

fn questioned() -> ScientificRuntime {
    let mut rt = warm();
    rt.enable_constructing_autonomous_inquiry(InquiryMode::Active)
        .unwrap();
    fact(&mut rt, 19);
    let inquiry = report(&rt);
    assert_eq!(
        inquiry["questions_opened"], 1,
        "the supported contradiction must open a real persistent question: {inquiry}"
    );
    assert_eq!(inquiry["active"]["triggering_action"], 0);
    assert_eq!(inquiry["active"]["predicted_post"], json!([11]));
    assert_eq!(inquiry["active"]["actual_post"], json!([19]));
    assert!(inquiry["active"]["support"].as_u64().unwrap() >= 8);
    rt
}

#[test]
fn inquiry_warm_opt_in_preserves_acquired_evidence_and_legacy_checkpoint_bytes() {
    let original = warm();
    let legacy = original.checkpoint_constructing().unwrap();
    let legacy_value: Value = serde_json::from_slice(&legacy).unwrap();
    assert!(
        legacy_value["cognition"]["current"]["learner"]["causal_memory"]
            .get("inquiry")
            .is_none()
    );
    let restored = ScientificRuntime::from_constructing_checkpoint(&legacy).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(), legacy);
    assert!(restored
        .construction_learner()
        .unwrap()
        .causal_inquiry_report()
        .is_none());

    for mode in [InquiryMode::Active, InquiryMode::ObserveOnly] {
        let mut rt = ScientificRuntime::from_constructing_checkpoint(&legacy).unwrap();
        let before = rt.construction_learner().unwrap().causal_report().unwrap();
        rt.enable_constructing_autonomous_inquiry(mode).unwrap();
        let after = rt.construction_learner().unwrap().causal_report().unwrap();
        assert_eq!(after.learned_fingerprint, before.learned_fingerprint);
        assert_eq!(after.learned_transitions, before.learned_transitions);
        assert_eq!(after.factual_transitions, before.factual_transitions);
        assert_eq!(
            rt.construction_learner()
                .unwrap()
                .causal_regime_report()
                .unwrap()
                .mode,
            RegimeMode::Ordinary
        );
        let configured = rt.checkpoint_constructing().unwrap();
        let loaded = ScientificRuntime::from_constructing_checkpoint(&configured).unwrap();
        assert_eq!(loaded.checkpoint_constructing().unwrap(), configured);
        assert!(rt.enable_constructing_autonomous_inquiry(mode).is_err());
        assert_eq!(rt.checkpoint_constructing().unwrap(), configured);
    }
}

#[test]
fn inquiry_question_and_continuation_survive_the_complete_runtime_checkpoint() {
    let mut rt = questioned();
    let before = report(&rt);
    assert_eq!(before["observations"], 1);
    let snapshot = rt.checkpoint_constructing().unwrap();
    for _ in 0..6 {
        assert_eq!(report(&rt), before);
        rt.construction_learner()
            .unwrap()
            .causal_action_beliefs(&offers(true, true), Some(&[Some(11)]), 20_000)
            .unwrap();
    }
    assert_eq!(
        rt.checkpoint_constructing().unwrap(),
        snapshot,
        "diagnostic reads and imagined forecasts must not create evidence"
    );
    let mut loaded = ScientificRuntime::from_constructing_checkpoint(&snapshot).unwrap();
    assert_eq!(loaded.checkpoint_constructing().unwrap(), snapshot);
    rt.observe_constructing_external_restart(vec![0]).unwrap();
    loaded
        .observe_constructing_external_restart(vec![0])
        .unwrap();
    assert_eq!(
        report(&rt),
        before,
        "an episode boundary must not erase an unanswered question"
    );
    for post in [19, 19, 19, 11, 11, 19] {
        fact(&mut rt, post);
        fact(&mut loaded, post);
        assert_eq!(
            rt.checkpoint_constructing().unwrap(),
            loaded.checkpoint_constructing().unwrap()
        );
    }
}

#[test]
fn inquiry_frozen_factual_inference_does_not_rewrite_learned_knowledge() {
    let mut rt = questioned();
    rt.freeze_constructing().unwrap();
    let model = rt.construction_learner().unwrap().causal_report().unwrap();
    let inquiry = report(&rt);
    for _ in 0..3 {
        fact(&mut rt, 19);
    }
    let after = rt.construction_learner().unwrap().causal_report().unwrap();
    assert_eq!(model.learned_transitions, after.learned_transitions);
    assert_eq!(model.learned_fingerprint, after.learned_fingerprint);
    assert_eq!(after.factual_transitions, model.factual_transitions + 3);
    assert_ne!(
        report(&rt),
        inquiry,
        "factual inference must remain live after freezing learning"
    );
    assert_eq!(
        report(&rt)["observations"].as_u64().unwrap(),
        inquiry["observations"].as_u64().unwrap() + 3
    );
    assert!(rt
        .enable_constructing_autonomous_inquiry(InquiryMode::ObserveOnly)
        .is_err());
    let bytes = rt.checkpoint_constructing().unwrap();
    assert_eq!(
        ScientificRuntime::from_constructing_checkpoint(&bytes)
            .unwrap()
            .checkpoint_constructing()
            .unwrap(),
        bytes
    );
}

#[test]
fn inquiry_cannot_turn_an_unavailable_or_unsafe_experiment_into_a_motor_permit() {
    let original = questioned().checkpoint_constructing().unwrap();
    for unsafe_action in [false, true] {
        let mut rt = ScientificRuntime::from_constructing_checkpoint(&original).unwrap();
        rt.observe_constructing_external_restart(vec![0]).unwrap();
        let before = report(&rt);
        let facts = rt.construction_learner().unwrap().factual_event_count();
        let cost = rt.unified_report().unwrap().total_external_cost;
        let result = rt.step_constructing(
            Some(&[Some(11)]),
            &offers(unsafe_action, unsafe_action),
            |action| {
                let mut evidence = safe(action);
                evidence.emergency_stop = unsafe_action;
                evidence
            },
            |_| panic!("an unanswered question is not actuator authority"),
        );
        assert!(matches!(result, Err(Error::Blocked)));
        assert_eq!(
            rt.construction_learner().unwrap().factual_event_count(),
            facts
        );
        assert_eq!(rt.unified_report().unwrap().total_external_cost, cost);
        assert_eq!(
            report(&rt),
            before,
            "a rejected proposal must not become a committed inquiry experiment"
        );
        if unsafe_action {
            assert!(rt.emergency_latched());
        }
    }
}

#[test]
fn inquiry_failed_actuation_cannot_resolve_a_question_or_invent_a_factual_post() {
    let original = questioned().checkpoint_constructing().unwrap();
    for invalid_post in [false, true] {
        let mut rt = ScientificRuntime::from_constructing_checkpoint(&original).unwrap();
        rt.observe_constructing_external_restart(vec![0]).unwrap();
        let before = report(&rt);
        let model = rt.construction_learner().unwrap().causal_report().unwrap();
        let cost = rt.unified_report().unwrap().total_external_cost;
        let result = rt.step_constructing(Some(&[Some(11)]), &offers(true, true), safe, |_| {
            if invalid_post {
                // Known external cost still belongs to the lifetime, even
                // when the adapter returns a malformed sensor vector.
                Ok(UnifiedPost {
                    observation: vec![19, 23],
                    cost: 1,
                    reward: 0.0,
                })
            } else {
                Err(Error::Execution(
                    "actuator returned no trustworthy POST".into(),
                ))
            }
        });
        assert!(result.is_err());
        assert!(rt.emergency_latched());
        assert!(rt.construction_learner().unwrap().report().poisoned);
        let after = report(&rt);
        assert_eq!(after["observations"], before["observations"]);
        assert_eq!(after["questions_resolved"], before["questions_resolved"]);
        assert_eq!(after["factual_experiments"], before["factual_experiments"]);
        assert_eq!(after["active"], before["active"]);
        let retained = rt.construction_learner().unwrap().causal_report().unwrap();
        assert_eq!(retained.factual_transitions, model.factual_transitions);
        assert_eq!(retained.learned_transitions, model.learned_transitions);
        assert_eq!(retained.learned_fingerprint, model.learned_fingerprint);
        assert_eq!(
            rt.unified_report().unwrap().total_external_cost,
            cost + u64::from(invalid_post)
        );
        let checkpoint = rt.checkpoint_constructing().unwrap();
        let mut loaded = ScientificRuntime::from_constructing_checkpoint(&checkpoint).unwrap();
        assert!(loaded.emergency_latched());
        assert_eq!(loaded.checkpoint_constructing().unwrap(), checkpoint);
        assert!(matches!(
            loaded.step_constructing(Some(&[Some(11)]), &offers(true, true), safe, |_| panic!(
                "a poisoned inquiry must not retry the physical action"
            )),
            Err(Error::Blocked)
        ));
    }
}

#[test]
fn inquiry_rejected_opt_in_does_not_partially_change_the_live_organism() {
    let mut rt = warm();
    rt.enable_constructing_regime_inference(RegimeMode::Phase)
        .unwrap();
    let before = rt.checkpoint_constructing().unwrap();
    assert!(rt
        .enable_constructing_autonomous_inquiry(InquiryMode::Active)
        .is_err());
    assert_eq!(rt.checkpoint_constructing().unwrap(), before);

    // A reserved inference state must fit the same memory ceiling. Failing
    // that check cannot leave an automatically enabled router behind.
    let mut small = fixture_with_memory(CausalConfig {
        history_depth: 3,
        model_bytes: 16_384,
        ..Default::default()
    });
    let before = small.checkpoint_constructing().unwrap();
    assert!(small
        .enable_constructing_autonomous_inquiry(InquiryMode::Active)
        .is_err());
    assert_eq!(small.checkpoint_constructing().unwrap(), before);
    assert!(small
        .construction_learner()
        .unwrap()
        .causal_regime_report()
        .is_none());
}

#[test]
fn inquiry_restore_checks_semantics_after_a_checksum_is_recomputed() {
    let rt = questioned();
    let original = serde_json::to_value(rt.construction_learner().unwrap()).unwrap();
    // A fresh envelope cannot legitimize an inference history longer than
    // the factual history. This does not rely on failing a stale checksum.
    let mut forged = original.clone();
    assert!(forged["causal_memory"]["inquiry"].is_object());
    forged["causal_memory"]["inquiry"]["observations"] = json!(u64::MAX);
    let forged: Learner = serde_json::from_value(forged).unwrap();
    let bytes = forged.checkpoint().unwrap();
    assert!(Learner::from_checkpoint(&bytes).is_err());
    let sound: Learner = serde_json::from_value(original).unwrap();
    assert!(Learner::from_checkpoint(&sound.checkpoint().unwrap()).is_ok());
}

struct Jsonl {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Jsonl {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_aeterna-v1"))
            .arg("--jsonl")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            input,
            output,
        }
    }
    fn exchange(&mut self, message: Value) -> Value {
        writeln!(self.input, "{message}").unwrap();
        self.input.flush().unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
    fn finish(mut self) {
        assert_eq!(self.exchange(json!({"op":"quit"}))["type"], "bye");
        drop(self.input);
        assert!(self.child.wait().unwrap().success());
    }
}

#[test]
fn inquiry_jsonl_initializes_one_organism_and_roundtrips_after_a_real_post() {
    let mut session = Jsonl::start();
    let initialized = session.exchange(json!({"op":"autonomous_init", "width":1,
        "actions":2, "planning_depth":3, "planning_work":120000,
        "journal_capacity":256}));
    assert_eq!(initialized["type"], "initialized", "{initialized}");
    let before = session.exchange(json!({"op":"inquiry_report"}));
    assert_eq!(before["type"], "inquiry_report");
    assert_eq!(before["report"]["mode"], "active");
    assert_eq!(before["report"]["observations"], 0);
    let action = session.exchange(json!({"op":"step", "available":[true,true],
        "goal":[11], "remaining_actions":1}));
    assert_eq!(action["type"], "action", "{action}");
    let committed = session.exchange(json!({"op":"post", "result":{
        "observation":[19], "cost":1, "reward":0.0}}));
    assert_eq!(committed["type"], "committed", "{committed}");
    let after = session.exchange(json!({"op":"inquiry_report"}));
    assert_ne!(before["report"], after["report"]);
    assert_eq!(after["report"]["observations"], 1);
    let restarted = session.exchange(json!({"op":"observe_restart","observation":[0]}));
    assert_eq!(restarted["type"], "restarted", "{restarted}");
    assert_eq!(
        session.exchange(json!({"op":"inquiry_report"})),
        after,
        "external restart cannot invent a POST or reset the inquiry"
    );
    assert_eq!(
        session.exchange(json!({"op":"causal_report"}))["report"]["factual_transitions"],
        1
    );
    let checkpoint = session.exchange(json!({"op":"checkpoint"}));
    let checkpoint = checkpoint["checkpoint"].as_str().unwrap().to_owned();
    session.finish();

    let mut restored = Jsonl::start();
    assert_eq!(
        restored.exchange(json!({"op":"load", "checkpoint":checkpoint}))["type"],
        "loaded"
    );
    assert_eq!(restored.exchange(json!({"op":"inquiry_report"})), after);
    assert_eq!(
        restored.exchange(json!({"op":"causal_report"}))["report"]["factual_transitions"],
        1
    );
    assert_eq!(
        restored.exchange(json!({"op":"report"}))["report"]["total_external_cost"],
        1
    );
    restored.finish();
}

#[test]
fn inquiry_jsonl_invalid_mode_is_rejected_without_an_action_handshake() {
    let mut session = Jsonl::start();
    assert_eq!(
        session.exchange(json!({"op":"init"}))["type"],
        "initialized"
    );
    let error = session.exchange(json!({"op":"enable_autonomous_inquiry", "mode":"omniscient"}));
    assert_eq!(error["type"], "error", "{error}");
    drop(session.input);
    assert!(!session.child.wait().unwrap().success());
}
