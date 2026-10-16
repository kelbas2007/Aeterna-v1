//! Repeatable development controls; no private qualification data is used.
use aeterna_v1::construction::constructors::{Registry,TransferCase,acquire_function};
use aeterna_v1::construction::synthesis::{Transition,SearchConfig,synthesize,replay,absolute_loss};
use aeterna_v1::construction::vm::{Error,Expr,Op,Library,Meter,Program};

fn affine_case(future_offset: i64) -> TransferCase {
    let data = |start: i64,end: i64,offset: i64| (start..end).map(|i| Transition {
        id: i as u64,restart: true,pre: vec![i],action: 0,post: vec![i+offset]
    }).collect();
    TransferCase { acquisition: data(0,8,17),future: data(8,12,future_offset) }
}
#[test]
fn future_labels_do_not_change_synthesis_and_batches_are_single_use() {
    let lib = Library::default(); let cfg = SearchConfig::default();
    let mut a = Registry::default(); let mut b = a.clone();
    let first = a.evaluate(&[affine_case(17)],&lib,&cfg).unwrap();
    let changed = b.evaluate(&[affine_case(-99)],&lib,&cfg).unwrap();
    let x = &first.rows[0]; let y = &changed.rows[0];
    assert_eq!(x.same_library.training_loss,y.same_library.training_loss);
    assert_eq!(x.same_library.program_nodes,y.same_library.program_nodes);
    assert_eq!(x.same_library.work,y.same_library.work);
    assert_eq!(x.same_library.future_loss,0.0);
    assert!(y.same_library.future_loss > 0.0);
    assert!(!first.additional_search_value);
    assert!(!first.independent_qualification);
    assert_eq!(a.evaluate(&[affine_case(17)],&lib,&cfg).unwrap_err(),Error::Stale);
}
#[test]
fn comparison_charges_the_same_library_and_does_not_invent_a_speedup() {
    let mut lib = Library::default();
    acquire_function(&Expr::bin(Op::Mul,Expr::Input(0),Expr::Input(0)),&mut lib,1).unwrap();
    let report = Registry::default().evaluate(&[affine_case(17)],&lib,&SearchConfig::default()).unwrap();
    let row = &report.rows[0];
    assert_eq!(row.no_library.library_nodes,0);
    assert!(row.same_library.library_nodes > 0);
    assert_eq!(row.same_library.library_nodes,row.constructors.library_nodes);
    assert_eq!(row.same_library.future_loss,row.constructors.future_loss);
    assert!(!report.additional_search_value);
}
#[test]
fn program_from_observations_is_frozen_before_disjoint_probe() {
    let make = |start: i64,end: i64| (start..end).map(|i| Transition {
        id: i as u64,restart: true,pre: vec![i],action: 0,post: vec![i+31]
    }).collect::<Vec<_>>();
    let training = make(0,8); let future = make(8,16); let lib = Library::default();
    let parent = Program::identity(1);
    let trace = synthesize(&parent,&training,&lib,&SearchConfig::default(),&[]).unwrap();
    assert!(!trace.prospectively_accepted);
    let program = trace.winner.unwrap().apply(&parent,&lib).unwrap();
    let frozen_id = program.fingerprint();
    let predictions = replay(&program,&future,&lib,&mut Meter::new(10000)).unwrap().0;
    assert_eq!(absolute_loss(&predictions,&future).unwrap(),0.0);
    assert_eq!(program.fingerprint(),frozen_id);
}
