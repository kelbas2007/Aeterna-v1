//! Deliberate corruption controls, not learning benchmarks. Recompute the
//! envelope checksum so rejection must come from semantic validation.
use aeterna_v1::construction::epistemic::EvidencePolicy;
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::vm::Expr;
use aeterna_v1::construction::{Config, ExternalPost, Learner, ProtectedConstructionRuntime};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};

fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence { human_present: false, physical_effect_possible: false,
        predicted_harm_probability: 0.0, hazard_confidence: 1.0, emergency_stop: false }
}
fn make(width: usize) -> ProtectedConstructionRuntime {
    let cfg = Config { width, minimum_training: 32, search_every: 32, shadow_steps: 4,
        journal_capacity: 2048, search: SearchConfig { work: 50_000, max_expr_nodes: 1,
        max_state: 0, max_updates: 1, ..Default::default() }, ..Default::default() };
    let mut r = ProtectedConstructionRuntime::new(EvoPhase::new(EvoConfig {
        sensory_cells: width, motor_cells: 2, ..Default::default()
    }), cfg, vec![0; width]).unwrap();
    r.enable_epistemic(EvidencePolicy { protected_outputs: if width == 2 { vec![0] } else { vec![] },
        ..Default::default() }).unwrap();
    r
}
fn feed(r: &mut ProtectedConstructionRuntime, begin: usize, end: usize) {
    let width = r.learner().current().len();
    for i in begin..end {
        let mut post = vec![0; width]; post[width - 1] = i64::from(i % 5 != 0);
        r.step_action(0, safe, |_| Ok(ExternalPost { observation: post, cost: 1 })).unwrap();
    }
}
fn pending() -> ProtectedConstructionRuntime {
    let mut r = make(1); feed(&mut r, 0, 48);
    assert_eq!(r.learner().epistemic_report().unwrap().pending.unwrap().samples, 16);
    r
}
fn rejects_semantic_corruption(mutate: impl FnOnce(&mut serde_json::Value)) {
    let r = pending();
    let mut value = serde_json::to_value(r.learner()).unwrap(); mutate(&mut value);
    let forged: Learner = serde_json::from_value(value).unwrap();
    let bytes = forged.checkpoint().unwrap(); // valid freshly computed checksum
    assert!(Learner::from_checkpoint(&bytes).is_err());
}
#[test]
fn inflated_evidence_is_rejected_even_with_a_recomputed_envelope_hash() {
    rejects_semantic_corruption(|s| s["shadow"]["evidence"]["wealth"] = serde_json::json!([100.0, 100.0, 100.0, 100.0]));
}
#[test]
fn refunded_pending_risk_is_rejected() {
    rejects_semantic_corruption(|s| s["epistemic"]["attempts"] = serde_json::json!([0]));
}
#[test]
fn relaxed_pending_threshold_is_rejected() {
    rejects_semantic_corruption(|s| s["shadow"]["evidence"]["alpha"] = serde_json::json!(0.5));
}
#[test]
fn mutually_consistent_fake_forecasts_and_evidence_fail_frozen_program_replay() {
    rejects_semantic_corruption(|s| {
        let split = s["shadow"]["training_len"].as_u64().unwrap() as usize;
        let mut wealth = [0.0_f64; 4];
        let bets = [0.0625_f64, 0.125, 0.25, 0.5];
        let mut total = 0.0;
        for row in &mut s["records"].as_array_mut().unwrap()[split..] {
            row["candidate_prediction"]["center"][0] = serde_json::json!(0);
            let actual = row["transition"]["post"][0].as_i64().unwrap();
            let old = row["prediction"]["center"][0].as_i64().unwrap();
            let gain = usize::from(old != actual) as f64 - usize::from(actual != 0) as f64;
            for (w, bet) in wealth.iter_mut().zip(bets) { *w += (bet * gain).ln_1p(); }
            total += gain;
        }
        let high = wealth.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let log_e = high + (wealth.iter().map(|w| (w - high).exp()).sum::<f64>() / 4.0).ln();
        let alpha = s["shadow"]["evidence"]["alpha"].as_f64().unwrap();
        assert!(log_e < -alpha.ln());
        s["shadow"]["evidence"]["wealth"] = serde_json::json!(wealth);
        s["shadow"]["evidence"]["total_gain"] = serde_json::json!(total);
        s["shadow"]["evidence"]["crossed"] = serde_json::json!(false);
    });
}
#[test]
fn imagined_actions_do_not_add_evidence() {
    let r = pending(); let before = r.checkpoint().unwrap();
    for _ in 0..20 { r.learner().choose_action(Some(&[1])).unwrap(); }
    assert_eq!(before, r.checkpoint().unwrap());
}
#[test]
fn rollback_does_not_refund_the_risk_budget() {
    let mut r = make(1); feed(&mut r, 0, 256);
    let before = r.learner().epistemic_report().unwrap();
    assert!(before.outcomes.iter().any(|o| o.installed));
    r.rollback_cognition().unwrap();
    assert_eq!(r.learner().epistemic_report().unwrap(), before);
    assert_eq!(r.learner().records().len(), 256);
    feed(&mut r, 256, 320);
    let after = r.learner().epistemic_report().unwrap();
    assert!(after.attempts[0] > before.attempts[0]);
    assert!(after.allocated_alpha > before.allocated_alpha);
}
#[test]
fn nonzero_target_can_learn_without_changing_a_protected_output() {
    let mut r = make(2); feed(&mut r, 0, 256);
    assert_eq!(r.learner().program().outputs, [Expr::Input(0), Expr::integer(1)]);
    let report = r.learner().epistemic_report().unwrap();
    assert_eq!(report.attempts[0], 0);
    let accepted = report.outcomes.iter().find(|o| o.installed).unwrap();
    assert_eq!(accepted.evidence.target, 1);
    assert_eq!(accepted.evidence.alpha, 0.0025);
}
