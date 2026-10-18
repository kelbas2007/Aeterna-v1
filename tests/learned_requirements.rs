use aeterna_v1::construction::{requirements::{CompletionLimits,ExecutedOperation,RequirementMemory},search_policy::{Example,SynthesisTask},vm::{Context,Error,Expr,Library,Meter,Op,Ty,Value},Config,Learner};
use aeterna_v1::construction::autonomy::AutonomyPolicy;
use aeterna_v1::scientific_runtime::ScientificRuntime;

fn executed(op:Op,n:usize)->Vec<ExecutedOperation>{
    (0..n).filter_map(|i|{
        let l=(i as i64*11%83)-37;
        let r=(i as i64*17%59)-24;
        op.apply(Value::Int(l),Value::Int(r)).ok().and_then(|z|z.int().ok()).map(|output|ExecutedOperation{op,left:l,right:r,output})
    }).collect()
}
fn task(n:usize,start:i64)->SynthesisTask {
    let examples=(0..n).map(|i|{
        let t=i as i64+start;
        let input=vec![(t*11%29)-8,(t*7%31)-12,(t*13%23)-6,(t*19%17)-5];
        let output=(input[0]+input[1])-(input[2]*input[3]);
        Example{input,output}
    }).collect();
    SynthesisTask{examples,library:Library::default()}
}
fn sketch()->Expr{Expr::bin(Op::Sub,Expr::bin(Op::Add,Expr::Input(0),Expr::Input(1)),Expr::Hole{id:0,ty:Ty::Int})}
fn verify(program:&Expr,rows:&SynthesisTask){
    let mut meter=Meter::new(50000);
    for row in &rows.examples {
        assert_eq!(program.eval(Context{input:&row.input,action:0,state:&[],args:&[]},&rows.library,&mut meter),Ok(Value::Int(row.output)));
    }
}
#[test]
fn inverse_is_acquired_from_executions_not_preloaded(){
    let mut b=RequirementMemory::default();assert_eq!(b.report().learned_rules,0);
    let dataset=executed(Op::Sub,35);assert_eq!(dataset.len(),35);
    let r=b.observe(&dataset).unwrap();assert_eq!(r.learned_rules,1);
    let rule=&b.rules()[0];assert_eq!(rule.op,Op::Sub);
    let mut meter=Meter::new(100);
    assert_eq!(b.requirement(Op::Sub,91,37,&mut meter).unwrap(),Some(54));
    assert_eq!(b.requirement(Op::Add,91,37,&mut meter).unwrap(),None);
    assert!(b.validate().is_ok());
}
#[test]
fn no_rule_cannot_be_claimed_as_solving_and_bad_examples_fail_atomically(){
    let mut b=RequirementMemory::default();
    assert_eq!(b.complete(&task(18,1),&sketch(),&CompletionLimits::default()).unwrap().program,None);
    let good=executed(Op::Sub,20);b.observe(&good).unwrap();let original=b.clone();
    let mut bad=executed(Op::Add,16);bad[0].output+=1;
    assert_eq!(b.observe(&bad),Err(Error::InvalidData));assert_eq!(b,original);
}
#[test]
fn missing_intermediate_expression_is_inferred_and_verified_on_unseen_inputs(){
    let mut b=RequirementMemory::default();b.observe(&executed(Op::Sub,35)).unwrap();
    let train=task(22,5);let probe=task(18,100);
    let r=b.complete(&train,&sketch(),&CompletionLimits{work:2_000_000,nodes:7,terms:4096}).unwrap();
    assert!(r.used_acquired_inverse);assert!(r.forward_verified);
    let found=r.program.expect("the acquired backward constraint must expose an independently learnable multiplicative fragment");
    verify(&found,&train);verify(&found,&probe);
}
#[test]
fn rule_memory_is_part_of_joint_checkpoint_and_freeze_blocks_learning(){
    let cfg=Config{width:4,actions:2,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0;4],"LC1-RSP3").unwrap();
    rt.enable_missing_computation_learning().unwrap();
    assert!(rt.requirements_report().is_some());
    rt.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let limits=CompletionLimits{work:2_000_000,nodes:7,terms:4096};
    let first=rt.complete_missing_computation(&task(22,5),&sketch(),&limits).unwrap();
    assert!(first.forward_verified);
    let before=rt.checkpoint_constructing().unwrap();
    let mut restored=ScientificRuntime::from_constructing_checkpoint(&before).unwrap();
    assert_eq!(restored.requirements_report(),rt.requirements_report());
    assert_eq!(restored.complete_missing_computation(&task(22,5),&sketch(),&limits).unwrap().program,first.program);
    rt.freeze_constructing().unwrap();let frozen=rt.checkpoint_constructing().unwrap();
    assert_eq!(rt.observe_executed_computations(&executed(Op::Sub,18)),Err(Error::Blocked));
    assert_eq!(rt.checkpoint_constructing().unwrap(),frozen);
    restored.freeze_constructing().unwrap();
}
#[test]
fn old_learner_checkpoint_remains_unchanged_with_lc1_opt_in_absent(){
    let learner=Learner::new(Config::default(),vec![0]).unwrap();
    let bytes=learner.checkpoint().unwrap();
    assert_eq!(Learner::from_checkpoint(&bytes).unwrap().checkpoint().unwrap(),bytes);
    assert!(learner.requirements_report().is_none());
}
#[test]
fn organism_can_discover_a_missing_expression_without_a_human_sketch() {
    let mut book=RequirementMemory::default();book.observe(&executed(Op::Sub,35)).unwrap();
    let train=task(22,5);let unseen=task(18,100);
    let result=book.discover(&train,&CompletionLimits{work:3_000_000,nodes:7,terms:4096}).unwrap();
    assert!(result.tried_rules>0);
    assert!(result.forward_verified,"no candidate survived forward VM verification");
    verify(&result.program.unwrap(),&unseen);
}

#[test]
fn acquired_requirements_coexist_with_rsp2_inside_one_restored_organism() {
    use aeterna_v1::construction::search_policy::{EvolutionConfig, SearchLimits};
    let mut rt=ScientificRuntime::new_constructing(Config{width:4,actions:2,..Default::default()},AutonomyPolicy::default(),vec![0;4],"LC1-RSP2-RSP3").unwrap();
    rt.enable_recursive_search(EvolutionConfig{object:SearchLimits::default(),meta:SearchLimits::default(),max_lifetime_work:5_000_000}).unwrap();
    rt.enable_missing_computation_learning().unwrap();
    rt.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let before=rt.checkpoint_constructing().unwrap();
    let mut restored=ScientificRuntime::from_constructing_checkpoint(&before).unwrap();
    assert!(restored.construction_learner().unwrap().recursive_search().is_some());
    assert_eq!(restored.requirements_report().unwrap().learned_rules,1);
    let completion=restored.discover_missing_computation(&task(22,5),&CompletionLimits{work:3_000_000,nodes:7,terms:4096}).unwrap();
    assert!(completion.forward_verified);
    verify(&completion.program.unwrap(),&task(16,100));
    assert_eq!(restored.construction_learner().unwrap().recursive_search().unwrap().generation(),0);
}
#[test]
fn contradictory_inverse_observations_refuse_an_unidentifiable_rule(){
    // 0 * x = 0 cannot identify a unique right operand. Contradictory
    // `(left,output)->right` labels must not acquire an inverse.
    let mut b=RequirementMemory::default();
    let mut rows=executed(Op::Mul,35);
    rows.push(ExecutedOperation{op:Op::Mul,left:0,right:7,output:0});
    rows.push(ExecutedOperation{op:Op::Mul,left:0,right:9,output:0});
    b.observe(&rows).unwrap();
    assert!(b.rules().iter().all(|r|r.op!=Op::Mul));
}

#[test]
fn observed_lc1_work_is_the_change_in_total_learner_work_not_total_vs_lc1() {
    let mut rt=ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-SHARED-WORK"
    ).unwrap();
    rt.enable_missing_computation_learning().unwrap();
    // An initial batch establishes a cumulative learned-requirement counter.
    // A second batch must charge ONLY the incremental work to the common
    // account, regardless of how much other construction has already spent.
    let first=executed(Op::Sub,35);
    let second=executed(Op::Add,16);
    for batch in [&first[..],&second[..]] {
        let before=rt.unified_report().unwrap();
        let result=rt.observe_executed_computations(batch).unwrap();
        let after=rt.unified_report().unwrap();
        let total_delta=after.learner.construction_work-before.learner.construction_work;
        let charged_delta=after.computation_work-before.computation_work;
        assert!(total_delta>0,"an actual study must consume work");
        assert_eq!(charged_delta,total_delta,"LC1 work charged using mismatched counters");
        assert_eq!(result.work,rt.requirements_report().unwrap().work);
        assert!(after.computation_work<1_000_000,"false saturation from underflow");
    }
    // Preserve both cumulative work counters through a real joint checkpoint.
    let before=rt.unified_report().unwrap();
    let cp=rt.checkpoint_constructing().unwrap();
    let restored=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    let after=restored.unified_report().unwrap();
    assert_eq!(after.learner.construction_work,before.learner.construction_work);
    assert_eq!(after.computation_work,before.computation_work);
}

#[test]
fn lc1_after_charged_rsp2_failure_uses_shared_learner_counter() {
    // A failed RSP2 producer attempt is still charged to construction_work.
    // LC1's own cumulative RequirementReport.work starts from zero; mixing
    // them caused `out.work - before` to underflow in debug (P1, PR #8).
    use aeterna_v1::construction::search_policy::{
        AdmissionSuite, EvolutionConfig, ProducerCase, SearchLimits, ValidationCase,
    };
    let mut rt=ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-AFTER-RSP2"
    ).unwrap();
    let object=SearchLimits{work:1000,evaluations:2,max_nodes:11,frontier:8,edits_per_expansion:1};
    let meta=SearchLimits{work:100,evaluations:1,max_nodes:31,frontier:8,edits_per_expansion:1};
    rt.enable_recursive_search(EvolutionConfig{
        object:object.clone(),meta,max_lifetime_work:500_000,
    }).unwrap();
    let mk_task=|id:i64|SynthesisTask{
        examples:(0..4).map(|j| {
            let x=id*100+j as i64;
            Example{input:vec![x,x+1],output:2*x+1}
        }).collect(),
        library:Library::default(),
    };
    let mk_case=|id:i64|ValidationCase{
        support:mk_task(id*2),probes:mk_task(id*2+1),
    };
    let gate=AdmissionSuite{
        fresh:(1..=8).map(mk_case).collect(),
        transfer:(21..=28).map(mk_case).collect(),
        producers:vec![
            ProducerCase{training:vec![mk_task(501)],examination:vec![mk_case(101),mk_case(102)]},
            ProducerCase{training:vec![mk_task(502)],examination:vec![mk_case(103),mk_case(104)]},
        ],
    };
    // The tiny meta budget is intentionally insufficient for producing a
    // candidate, but the valid gate causes real metered work to be reserved.
    let rsp2_result=rt.improve_recursive_search(&[mk_task(300)],&gate,99);
    assert!(rsp2_result.is_err());
    let before=rt.unified_report().unwrap();
    assert!(before.learner.construction_work>1000,
        "test must establish non-LC1 learner work before observing LC1");
    rt.enable_missing_computation_learning().unwrap();
    let report=rt.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let after=rt.unified_report().unwrap();
    assert_eq!(after.learner.construction_work-before.learner.construction_work, report.work);
    assert_eq!(after.computation_work-before.computation_work,report.work,
        "only LC1's new work may be charged, not the entire cumulative ledger");
    assert!(after.computation_work<500_000,"the counter must not saturate");
    let cp=rt.checkpoint_constructing().unwrap();
    let restored=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    assert_eq!(restored.unified_report().unwrap().computation_work,after.computation_work);
}

#[test]
fn lc1_induction_support_limit_rejects_before_work_reservation() {
    // General synthesis admits 256 rows; the current fragment-induction
    // backend supports only 128. LC1 must refuse the larger, otherwise valid
    // input before consuming or reserving the entire requested budget.
    use aeterna_v1::construction::requirements::{validate_completion_task, MAX_LC1_SUPPORT_ROWS};
    assert_eq!(MAX_LC1_SUPPORT_ROWS,128);
    let boundary=task(128,1);
    assert!(boundary.validate().is_ok());
    assert!(validate_completion_task(&boundary).is_ok());
    let mut memory=RequirementMemory::default();
    memory.observe(&executed(Op::Sub,35)).unwrap();
    let limits=CompletionLimits{work:2_000_000,nodes:7,terms:4096};
    let mut runtime=ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-ROW-BOUNDARY"
    ).unwrap();
    runtime.enable_missing_computation_learning().unwrap();
    runtime.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let checkpoint=runtime.checkpoint_constructing().unwrap();
    let baseline=runtime.unified_report().unwrap();
    for rows in [129,256] {
        let unsupported=task(rows,1);
        assert!(unsupported.validate().is_ok(),"SynthesisTask must still admit {rows}");
        assert_eq!(validate_completion_task(&unsupported),Err(Error::Capacity));
        assert_eq!(memory.complete(&unsupported,&sketch(),&limits),Err(Error::Capacity));
        assert_eq!(memory.discover(&unsupported,&limits),Err(Error::Capacity));
        assert_eq!(runtime.complete_missing_computation(&unsupported,&sketch(),&limits),Err(Error::Capacity));
        assert_eq!(runtime.discover_missing_computation(&unsupported,&limits),Err(Error::Capacity));
        assert_eq!(runtime.checkpoint_constructing().unwrap(),checkpoint);
        assert_eq!(runtime.unified_report().unwrap().computation_work,baseline.computation_work);
    }
    // A fully supported task still goes through the ordinary solver with
    // acquired inverse rules and a true forward-VM check.
    let result=runtime.complete_missing_computation(&task(22,5),&sketch(),&limits).unwrap();
    assert!(result.forward_verified);
}


#[test]
fn lc1_induction_input_width_validation_is_before_work_reservation() {
    use aeterna_v1::construction::requirements::{validate_completion_task, MAX_LC1_INPUTS};
    assert_eq!(MAX_LC1_INPUTS, 6);
    let mut memory=RequirementMemory::default();
    memory.observe(&executed(Op::Sub,35)).unwrap();
    let limits=CompletionLimits{work:2_000_000,nodes:7,terms:4096};
    let mut runtime=ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-INPUT-WIDTH-PREFLIGHT").unwrap();
    runtime.enable_missing_computation_learning().unwrap();
    runtime.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let checkpoint=runtime.checkpoint_constructing().unwrap();
    let before=runtime.unified_report().unwrap();
    let mut supported_six=task(22,5);
    for row in &mut supported_six.examples {row.input.extend([0,0]);}
    assert!(validate_completion_task(&supported_six).is_ok(),"six inputs remain supported");
    for width in [7,32] {
        let mut unsupported=task(22,5);
        for row in &mut unsupported.examples {
            row.input.extend(std::iter::repeat(0).take(width - row.input.len()));
        }
        assert!(unsupported.validate().is_ok(),"generic SynthesisTask accepts width {width}");
        assert_eq!(validate_completion_task(&unsupported),Err(Error::Capacity));
        assert_eq!(memory.complete(&unsupported,&sketch(),&limits),Err(Error::Capacity));
        assert_eq!(memory.discover(&unsupported,&limits),Err(Error::Capacity));
        assert_eq!(runtime.complete_missing_computation(&unsupported,&sketch(),&limits),Err(Error::Capacity));
        assert_eq!(runtime.discover_missing_computation(&unsupported,&limits),Err(Error::Capacity));
        assert_eq!(runtime.checkpoint_constructing().unwrap(),checkpoint);
        assert_eq!(runtime.unified_report().unwrap().computation_work,before.computation_work);
    }
    // The original 4-input task still succeeds after the rejected requests.
    let solved=runtime.complete_missing_computation(&task(22,5),&sketch(),&limits).unwrap();
    assert!(solved.forward_verified);
}

#[test]
fn lc1_invalid_sketches_are_rejected_before_work_reservation() {
    use aeterna_v1::construction::requirements::validate_completion_sketch;
    let mut memory=RequirementMemory::default();memory.observe(&executed(Op::Sub,35)).unwrap();
    let limits=CompletionLimits{work:2_000_000,nodes:7,terms:4096};
    let mut runtime=ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-SKETCH-PREFLIGHT").unwrap();
    runtime.enable_missing_computation_learning().unwrap();
    runtime.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let input=task(22,5);
    let checkpoint=runtime.checkpoint_constructing().unwrap();
    let before=runtime.unified_report().unwrap();
    let invalid=[
        Expr::integer(42),
        Expr::bin(Op::Sub,Expr::Input(0),Expr::Hole{id:1,ty:Ty::Int}),
        Expr::bin(Op::Sub,Expr::Hole{id:0,ty:Ty::Int},Expr::Hole{id:0,ty:Ty::Int}),
        Expr::bin(Op::Sub,Expr::Lit(Value::Bool(true)),Expr::Hole{id:0,ty:Ty::Int}),
        Expr::bin(Op::Lt,Expr::Input(0),Expr::Hole{id:0,ty:Ty::Int}),
    ];
    for bad in invalid {
        assert_eq!(validate_completion_sketch(&input,&bad,&limits),Err(Error::InvalidProgram));
        assert_eq!(memory.complete(&input,&bad,&limits),Err(Error::InvalidProgram));
        assert_eq!(runtime.complete_missing_computation(&input,&bad,&limits),Err(Error::InvalidProgram));
        assert_eq!(runtime.checkpoint_constructing().unwrap(),checkpoint,
            "preflight failure cannot alter acquired state");
        assert_eq!(runtime.unified_report().unwrap().computation_work,before.computation_work,
            "preflight must not charge limits.work");
    }
    assert!(runtime.complete_missing_computation(&input,&sketch(),&limits).unwrap().forward_verified);
}

#[test]
fn lc1_disabled_requests_cannot_consume_organism_budget() {
    // Neither the explicit nor the automatic endpoint may charge work if
    // the acquired-requirements book was never enabled. This previously
    // returned NoEvidence *after* reserving up to 20 million work units.
    let mut rt=ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-UNENABLED-PREFLIGHT"
    ).unwrap();
    let valid_task=task(22,5);
    let valid_sketch=sketch();
    let limits=CompletionLimits{work:20_000_000,nodes:7,terms:4096};
    assert!(rt.requirements_report().is_none());
    let checkpoint=rt.checkpoint_constructing().unwrap();
    let before=rt.unified_report().unwrap();
    for _ in 0..2 {
        assert_eq!(rt.complete_missing_computation(&valid_task,&valid_sketch,&limits),Err(Error::NoEvidence));
        assert_eq!(rt.discover_missing_computation(&valid_task,&limits),Err(Error::NoEvidence));
        assert_eq!(rt.checkpoint_constructing().unwrap(),checkpoint,
            "disabled LC1 request must not mutate the whole organism checkpoint");
        let now=rt.unified_report().unwrap();
        assert_eq!(now.computation_work,before.computation_work,
            "NoEvidence must not reserve any work when the study never ran");
        assert_eq!(now.learner.construction_work,before.learner.construction_work);
    }
    // After normal opt-in, the very same saved organism is still usable.
    let mut restored=ScientificRuntime::from_constructing_checkpoint(&checkpoint).unwrap();
    assert_eq!(restored.requirements_report(),None);
    assert_eq!(restored.discover_missing_computation(&valid_task,&limits),Err(Error::NoEvidence));
    restored.enable_missing_computation_learning().unwrap();
    restored.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let answer=restored.complete_missing_computation(
        &valid_task,&valid_sketch,
        &CompletionLimits{work:2_000_000,nodes:7,terms:4096}
    ).unwrap();
    assert!(answer.forward_verified);
}

#[test]
fn lc1_oversized_but_well_formed_sketch_rejected_before_budget_charge() {
    use aeterna_v1::construction::requirements::validate_completion_sketch;
    let mut rt=ScientificRuntime::new_constructing(
        Config{width:4,actions:2,..Default::default()},
        AutonomyPolicy::default(),vec![0;4],"LC1-OVERSIZE-SKETCH").unwrap();
    rt.enable_missing_computation_learning().unwrap();
    rt.observe_executed_computations(&executed(Op::Sub,35)).unwrap();
    let training=task(22,5);
    let limits=CompletionLimits{work:20_000_000,nodes:5,terms:4096};
    let known=Expr::bin(Op::Add,Expr::bin(Op::Add,Expr::Input(0),Expr::Input(1)),Expr::Input(2));
    let too_big=Expr::bin(Op::Sub,known,Expr::Hole{id:0,ty:Ty::Int});
    assert_eq!(too_big.nodes(),7);
    assert!(too_big.nodes()<=limits.nodes+4,"old code must accept this invalid bound");
    let before=rt.checkpoint_constructing().unwrap();
    let work_before=rt.unified_report().unwrap().computation_work;
    assert_eq!(validate_completion_sketch(&training,&too_big,&limits),Err(Error::InvalidProgram));
    assert_eq!(rt.complete_missing_computation(&training,&too_big,&limits),Err(Error::InvalidProgram));
    assert_eq!(rt.checkpoint_constructing().unwrap(),before);
    assert_eq!(rt.unified_report().unwrap().computation_work,work_before);
    // A sketch exactly as large as the limit still passes preflight.
    assert_eq!(sketch().nodes(),limits.nodes);
    assert!(validate_completion_sketch(&training,&sketch(),&limits).is_ok());
}

#[test]
fn lc1_restored_rule_cannot_claim_acquisition_from_undiverse_records() {
    let mut book=RequirementMemory::default();
    book.observe(&executed(Op::Sub,35)).unwrap();
    assert_eq!(book.report().learned_rules,1);
    let mut forged_json=serde_json::to_value(&book).unwrap();
    // Every row remains a *valid forward execution* and the stored inverse
    // continues to fit it; only the actual learning prerequisite of five
    // distinct left/right/output observations has been deliberately broken.
    let one=forged_json["observations"][0].clone();
    for r in forged_json["observations"].as_array_mut().unwrap(){*r=one.clone();}
    let forged:RequirementMemory=serde_json::from_value(forged_json.clone()).unwrap();
    assert_eq!(forged.report().learned_rules,1);
    assert_eq!(forged.validate(),Err(Error::InvalidData));
    // A recomputed checkpoint checksum must not turn forged learning
    // provenance into a valid acquired rule.
    let mut base=Learner::new(Config::default(),vec![0]).unwrap();
    base.enable_requirements().unwrap();
    let mut value=serde_json::to_value(&base).unwrap();
    value["requirements"]=forged_json;
    let forged_state:Learner=serde_json::from_value(value).unwrap();
    let re_enveloped=forged_state.checkpoint().unwrap();
    assert!(Learner::from_checkpoint(&re_enveloped).is_err());
    // Original real acquired state still restores with its rule.
    let mut sound=serde_json::to_value(&base).unwrap();
    sound["requirements"]=serde_json::to_value(&book).unwrap();
    let sound_state:Learner=serde_json::from_value(sound).unwrap();
    assert_eq!(Learner::from_checkpoint(&sound_state.checkpoint().unwrap()).unwrap().requirements_report().unwrap().learned_rules,1);
}

#[test]
fn lc1_checkpoint_rejects_fitting_but_noncanonical_inverse() {
    // Every executed record is coherent and suitably diverse, but two
    // expressions fit because the observed right operand equals left.
    // Learning must canonically select Input(0), not another coincidentally
    // fitting expression chosen by an edited checkpoint.
    let data=(1..=20).map(|i|ExecutedOperation{op:Op::Add,left:i,right:i,output:i*2}).collect::<Vec<_>>();
    let mut memory=RequirementMemory::default();
    memory.observe(&data).unwrap();
    assert_eq!(memory.rules().len(),1);
    assert_eq!(memory.rules()[0].right,Expr::Input(0));
    let mut encoded=serde_json::to_value(&memory).unwrap();
    let impostor=Expr::bin(Op::Sub,Expr::Input(1),Expr::Input(0));
    for row in &data {
        let mut meter=Meter::new(100);
        assert_eq!(impostor.eval(Context{input:&[row.left,row.output],action:0,state:&[],args:&[]},&Library::default(),&mut meter),Ok(Value::Int(row.right)));
    }
    encoded["rules"][0]["right"]=serde_json::to_value(&impostor).unwrap();
    let edited:RequirementMemory=serde_json::from_value(encoded).unwrap();
    assert_eq!(edited.validate(),Err(Error::InvalidData),"only the actual first acquired rule is admissible");
    assert_eq!(memory.validate(),Ok(()));
}
