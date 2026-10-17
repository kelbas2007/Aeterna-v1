use aeterna_v1::construction::synthesis::{SearchConfig,Transition,Hint,synthesize};
use aeterna_v1::construction::vm::{Expr,Library,Program,Op,Ty};

#[test]
fn over_specialized_schema_cannot_starve_a_small_sibling() {
    let lib = Library::default(); let p = Program::identity(2);
    let data: Vec<_> = (0..8).map(|i| Transition { id: i as u64,restart: true,
        pre: vec![0,i],action: 0,post: vec![i*i,i] }).collect();
    let h = |id| Expr::Hole { id,ty: Ty::Int };
    let hints = [
        Hint { constructor_id: 1,output: Expr::bin(Op::Add,Expr::integer(999),h(0)),cells: vec![] },
        Hint { constructor_id: 2,output: Expr::bin(Op::Mul,h(0),h(0)),cells: vec![] }
    ];
    let trace = synthesize(&p,&data,&lib,&SearchConfig { work: 1_000_000,..Default::default() },&hints).unwrap();
    assert_eq!(trace.training_loss,0.0); assert_eq!(trace.constructor_id,Some(2));
    assert!(trace.work < 20_000,"a failed narrow schema monopolized search: {}",trace.work);
}
