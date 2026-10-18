use aeterna_v1::construction::constructors::Registry;
use aeterna_v1::construction::recursive::{instantiate,match_schema,SearchTrial};
use aeterna_v1::construction::synthesis::{SearchConfig,Transition,Hint,synthesize};
use aeterna_v1::construction::vm::{Error,Expr,Library,Program,Op,Ty,Meter};

fn affine(offset: i64,start: u64) -> Vec<Transition> {
    (0..6).map(|i| Transition { id: start+i,restart: true,pre: vec![i as i64],action: 0,post: vec![i as i64+offset] }).collect()
}
#[test]
fn prospective_commitment_cannot_be_changed_by_future_labels() {
    let lib = Library::default(); let cfg = SearchConfig::default(); let parent = Program::identity(1);
    let mut a = Registry::default(); let mut b = a.clone();
    let ta = a.begin_trial(&parent,&affine(17,0),&lib,&cfg).unwrap();
    let tb = b.begin_trial(&parent,&affine(17,0),&lib,&cfg).unwrap();
    assert_eq!(ta.program(),tb.program()); assert_eq!(ta.frozen_search_work(),tb.frozen_search_work());
    let yes = a.complete_trial(ta,&affine(17,6),true).unwrap();
    let no = b.complete_trial(tb,&affine(-99,6),true).unwrap();
    assert!(yes.model_accepted); assert!(!no.model_accepted);
    assert_eq!(no.constructors_acquired,0); assert!(!no.additional_search_value);
    assert_eq!(yes.candidate.training_loss,no.candidate.training_loss);
}
#[test]
fn single_use_ticket_survives_serialization_and_row_relabelling() {
    let lib = Library::default(); let cfg = SearchConfig::default(); let p = Program::identity(1);
    let mut r = Registry::default();
    let trial = r.begin_trial(&p,&affine(17,0),&lib,&cfg).unwrap();
    let duplicate: SearchTrial = serde_json::from_slice(&serde_json::to_vec(&trial).unwrap()).unwrap();
    let mut restored: Registry = serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();
    restored.complete_trial(trial,&affine(17,6),true).unwrap();
    assert_eq!(restored.complete_trial(duplicate,&affine(17,6),true).unwrap_err(),Error::Stale);
    assert_eq!(restored.begin_trial(&p,&affine(17,900),&lib,&cfg).unwrap_err(),Error::Stale);
}
#[test]
fn invalid_future_consumes_trial_without_teaching_a_constructor() {
    let lib = Library::default(); let cfg = SearchConfig::default(); let p = Program::identity(1);
    let mut r = Registry::default(); let trial = r.begin_trial(&p,&affine(17,0),&lib,&cfg).unwrap();
    assert_eq!(r.complete_trial(trial,&affine(17,0),true).unwrap_err(),Error::InvalidData);
    assert!(r.constructors().is_empty()); assert_eq!(r.meta_ledger().abandoned,1);
    assert!(r.begin_trial(&p,&affine(19,100),&lib,&cfg).is_ok());
}
#[test]
fn model_installation_veto_does_not_train() {
    let lib = Library::default(); let cfg = SearchConfig::default(); let mut r = Registry::default();
    let trial = r.begin_trial(&Program::identity(1),&affine(17,0),&lib,&cfg).unwrap();
    let report = r.complete_trial(trial,&affine(17,6),false).unwrap();
    assert!(!report.model_accepted); assert!(r.traces().is_empty());
}
#[test]
fn constructor_revision_replays_with_capture_free_linked_holes() {
    let h = |id| Expr::Hole { id,ty: Ty::Int };
    let parent = Expr::bin(Op::Mul,h(0),h(0));
    let next = Expr::bin(Op::Add,h(0),h(1));
    let child = Expr::bin(Op::Mul,next.clone(),next);
    let bindings = match_schema(&parent,&child,&Library::default()).unwrap().unwrap();
    assert_eq!(instantiate(&parent,&bindings),child);
    assert!(match_schema(&parent,&Expr::bin(Op::Mul,Expr::Input(0),Expr::Input(1)),&Library::default()).unwrap().is_none());
    assert!(match_schema(&h(0),&Expr::Lit(aeterna_v1::construction::vm::Value::Bool(true)),&Library::default()).unwrap().is_none());
}
#[test]
fn failed_hint_cannot_remove_the_broad_fallback() {
    let lib = Library::default(); let cfg = SearchConfig::default(); let p = Program::identity(1);
    let hint = Hint { constructor_id: 99,output: Expr::integer(-999),cells: vec![] };
    let trace = synthesize(&p,&affine(17,0),&lib,&cfg,&[hint]).unwrap();
    assert_eq!(trace.training_loss,0.0); assert_eq!(trace.constructor_id,None);
    assert!(trace.baseline_work > 0); assert!(trace.constructor_work > 0); assert!(trace.work <= cfg.work);
}
#[test]
fn misleading_constructor_id_does_not_create_a_generation() {
    let lib = Library::default(); let cfg = SearchConfig::default(); let mut r = Registry::default();
    let mut trace = synthesize(&Program::identity(1),&affine(17,0),&lib,&cfg,&[]).unwrap();
    trace.prospectively_accepted = true; trace.constructor_id = Some(123);
    assert_eq!(r.learn(trace,&lib,&mut Meter::new(10000)),Err(Error::InvalidProgram));
    assert!(r.constructors().is_empty()); assert!(r.traces().is_empty());
}
#[test]
fn cancelling_preserves_consumption_and_allows_next_problem() {
    let lib = Library::default(); let cfg = SearchConfig::default(); let p = Program::identity(1);
    let mut r = Registry::default(); let trial = r.begin_trial(&p,&affine(17,0),&lib,&cfg).unwrap();
    assert_eq!(r.begin_trial(&p,&affine(19,0),&lib,&cfg).unwrap_err(),Error::Busy);
    r.cancel_trial(trial).unwrap();
    assert_eq!(r.begin_trial(&p,&affine(17,0),&lib,&cfg).unwrap_err(),Error::Stale);
    assert!(r.begin_trial(&p,&affine(19,0),&lib,&cfg).is_ok());
}
