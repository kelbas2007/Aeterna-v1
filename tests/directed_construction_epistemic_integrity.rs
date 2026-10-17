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
    rejects_semantic_corruption(|s| s["shadow"]["evidence"]["wealth"] = serde_json::json!([100.0; 4]));
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
fn_coherent_placeholder() {}
