use aeterna_v1::construction::synthesis::{SearchConfig,Transition,Hint,synthesize,Job};
use aeterna_v1::construction::vm::{Expr,Library,Program,Op,Ty,Value,Meter,Error};
use aeterna_v1::construction::constructors::{Registry,Constructor,ConstructorStatus};
use aeterna_v1::construction::recursive::{Refinement,instantiate};

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

// These are deliberately constructed UNIT-test syntax fixtures, not acquisition
// evidence. The end-to-end demonstration injects neither solutions nor schemas.
fn syntax_fixture() -> (Registry,Expr) {
    let h = |id| Expr::Hole { id,ty: Ty::Int };
    let code = Expr::bin(Op::Add,Expr::bin(Op::Mul,h(0),h(0)),Expr::integer(0));
    let root = Constructor { id: 11,generation: 1,refinement: None,code: code.clone(),cell_count: 0,
        guard: Expr::Lit(Value::Bool(true)),training_trace_ids: vec![1,2],construction_work: 0,status: ConstructorStatus::Proposed };
    let bindings = std::collections::BTreeMap::from([(0,Expr::bin(Op::Add,h(0),h(1)))]);
    let child_code = instantiate(&code,&bindings);
    let child = Constructor { id: 7,generation: 2,refinement: Some(Refinement { parent: 11,bindings }),
        code: child_code.clone(),training_trace_ids: vec![3,4],..root.clone() };
    let mut value = serde_json::to_value(Registry::default()).unwrap();
    value["constructors"] = serde_json::to_value(vec![root,child]).unwrap();
    let registry: Registry = serde_json::from_value(value).unwrap();
    registry.validate(&Library::default()).unwrap();
    (registry,child_code)
}
#[test]
fn generation_number_does_not_override_smallest_completion_cost() {
    let (r,_) = syntax_fixture();
    let job = Job { parent: 1,target: 0,witness_ids: vec![],deterministic_contract: true,
        stateless_impossible: false,exposed_sources: vec![0,1],features: vec![2,8,0,0] };
    let hints = r.suggestions(&job,2,&SearchConfig::default(),&Library::default(),&mut Meter::new(10000)).unwrap();
    assert_eq!(hints.iter().map(|h| h.constructor_id).collect::<Vec<_>>(),vec![11,7]);
}
#[test]
fn constructor_rewrite_executes_and_rejects_a_forged_cache() {
    let (r,child) = syntax_fixture();
    assert_eq!(r.materialize_constructor(7,&mut Meter::new(1000)).unwrap(),child);
    assert_eq!(r.materialize_constructor(7,&mut Meter::new(1)),Err(Error::Budget));
    let mut value = serde_json::to_value(r).unwrap();
    value["constructors"][1]["code"] = serde_json::to_value(Expr::integer(0)).unwrap();
    let tampered: Registry = serde_json::from_value(value).unwrap();
    assert_eq!(tampered.materialize_constructor(7,&mut Meter::new(1000)),Err(Error::InvalidProgram));
}
