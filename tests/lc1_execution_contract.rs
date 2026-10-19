//! Adversarial LC1 contract checks: hostile syntax, tight search size,
//! failed computations and the whole-organism metered-work ledger.
use aeterna_v1::construction::{
    requirements::{CompletionLimits,ExecutedOperation,RequirementMemory,validate_completion_sketch},
    search_policy::{Example,SynthesisTask},
    vm::{Error,Expr,Library,Op,Ty,Value},Config,
};
use aeterna_v1::construction::autonomy::AutonomyPolicy;
use aeterna_v1::scientific_runtime::ScientificRuntime;

fn trained_book()->RequirementMemory {
    let mut b=RequirementMemory::default();
    let rows=(0..35).map(|i|{
        let x=(i as i64*11%83)-37;
        let y=(i as i64*17%59)-24;
        ExecutedOperation{op:Op::Sub,left:x,right:y,output:x-y}
    }).collect::<Vec<_>>();
    b.observe(&rows).unwrap();assert_eq!(b.rules().len(),1);b
}
fn task()->SynthesisTask {
    let examples=(0..22).map(|j| {
        let k=j as i64+5;
        let input=vec![(k*11%29)-8,(k*7%31)-12,(k*13%23)-6,(k*19%17)-5];
        let output=(input[0]+input[1])-(input[2]*input[3]);
        Example{input,output}
    }).collect();
    SynthesisTask{examples,library:Library::default()}
}
fn limits(nodes:usize)->CompletionLimits {CompletionLimits{work:2_000_000,nodes,terms:4096}}
fn rt()->ScientificRuntime {
    ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-ADVERSARIAL-CONTRACT"
    ).unwrap()
}

#[test]
fn malformed_very_deep_sketch_fails_bounded_preflight_without_recursive_walk() {
    let mut known=Expr::Input(0);
    for _ in 0..8_192 {
        known=Expr::bin(Op::Add,known,Expr::integer(1));
    }
    let sketch=Expr::bin(Op::Sub,known,Expr::Hole{id:0,ty:Ty::Int});
    assert!(matches!(validate_completion_sketch(&task(),&sketch,&limits(7)),
        Err(Error::InvalidProgram)|Err(Error::Capacity)));
    // Dropping a deliberately hostile Box chain recursively can overflow even
    // though the actual LC1 preflight rejected it correctly.
    std::mem::forget(sketch);
}

#[test]
fn fragment_search_never_uses_nodes_beyond_actual_remaining_output_capacity() {
    let b=trained_book();
    let sketch=Expr::bin(Op::Sub,Expr::Input(0),Expr::Hole{id:0,ty:Ty::Int});
    assert_eq!(sketch.nodes(),3);
    let constrained=b.complete(&task(),&sketch,&limits(3)).unwrap();
    assert!(!constrained.forward_verified);
    assert!(constrained.work>0);
    // The 1-node fragment search performs only leaf/constant attempts. The
    // former max(3) accidentally widened this to a 3-node grammar.
    assert!(constrained.work<10_000,"one-node search spent {}",constrained.work);
}

#[test]
fn runtime_settles_failed_lc1_evaluation_by_metered_work_not_reserved_limit() {
    let mut runtime=rt();
    runtime.enable_missing_computation_learning().unwrap();
    let inputs=(0..35).map(|i|{
        let x=(i as i64*11%83)-37;let y=(i as i64*17%59)-24;
        ExecutedOperation{op:Op::Sub,left:x,right:y,output:x-y}
    }).collect::<Vec<_>>();
    runtime.observe_executed_computations(&inputs).unwrap();
    let before=runtime.unified_report().unwrap().computation_work;
    // Each support row is otherwise valid (input <= 1,000,000). The VM
    // evaluation reaches overflow after seven metered instructions, whereas
    // the old runtime billed the entire 2,000,000-unit reservation on Err.
    let hostile=SynthesisTask{examples:(0..8).map(|_|Example{
        input:vec![1_000_000,1_000_000,1_000_000,0],output:0
    }).collect(),library:Library::default()};
    let skeleton=Expr::bin(Op::Sub,
        Expr::bin(Op::Mul,Expr::bin(Op::Mul,Expr::bin(Op::Mul,Expr::Input(0),Expr::Input(1)),Expr::Input(2)),Expr::integer(20)),
        Expr::Hole{id:0,ty:Ty::Int});
    assert_eq!(runtime.complete_missing_computation(&hostile,&skeleton,&limits(9)),Err(Error::Overflow));
    let after=runtime.unified_report().unwrap().computation_work;
    assert_eq!(after-before,7,"only the executed VM instructions are billable");
    assert!(after-before<limits(7).work);
    let before_retry=after;
    let result=runtime.discover_missing_computation(&task(),&limits(7)).unwrap();
    assert!(result.forward_verified);
    let after_retry=runtime.unified_report().unwrap().computation_work;
    assert_eq!(after_retry-before_retry,result.work);
}
