use aeterna_v1::{EvoPhase};

#[allow(dead_code)]
mod fixture {
    include!("g19_rival_hypothesis_discrimination.rs");

    pub fn build()->(EvoPhase,[[usize;2];8]){
        let drive=train_drive();
        let mut evo=target(&drive);
        let l1=train_abstraction(&mut evo);
        assert!(evo.enable_phase_native_context_refinement());
        assert!(evo.enable_phase_native_perceptual_refinement());
        assert!(evo.enable_phase_native_compositional_refinement());
        (evo,l1)
    }
    pub fn scene(l1:&[[usize;2];8],state:usize,layout:usize)->Vec<f32>{
        state_scene(l1,state,LAYOUTS[layout%LAYOUTS.len()])
    }
}

fn feed(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    action:usize,
    post_state:usize,
    layout:usize,
){
    let post=fixture::scene(l1,post_state,layout);
    assert!(evo.observe_phase_native_refinement_fanout_result(action,&post).is_some());
}

fn create_unrelated_candidate(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
){
    // Same eventual junction base S0, but predecessors S3/S4 and anchor action 0.
    evo.clear_phase_native_context_history();
    evo.observe_initial_real(&fixture::scene(l1,3,0),false);
    feed(evo,l1,2,0,0); // predecessor history becomes S3 at S0
    feed(evo,l1,0,7,1); // discovery: S3 -- S0/a0 -> S7
    feed(evo,l1,2,4,2);
    feed(evo,l1,2,0,3); // predecessor history becomes S4 at S0
    feed(evo,l1,0,8,0); // collision: S4 -- S0/a0 -> S8

    let base=evo.phase_native_abstract_state(&fixture::scene(l1,0,0)).unwrap().cell;
    let witnesses=evo.phase_native_context_witnesses();
    let old=witnesses.iter().find(|w|
        w.base_cell==base && w.anchor_action==0
    ).expect("old same-base candidate");
    assert!(!old.promoted);
    assert_eq!(old.eligible_observations,0);
}

fn enter_predecessor(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    predecessor:usize,
    layout:usize,
){
    // Move factually from the current outcome to the desired predecessor, then
    // into the same ambiguous junction S0.
    feed(evo,l1,2,predecessor,layout);
    feed(evo,l1,2,0,(layout+1)%6);
}

fn train_target_candidate(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
){
    // Target history relation on the SAME base S0:
    // predecessor S1 -> action1 -> goal S7
    // predecessor S2 -> action1 -> dead S8.
    enter_predecessor(evo,l1,1,0);
    feed(evo,l1,1,7,1);
    enter_predecessor(evo,l1,2,2);
    feed(evo,l1,1,8,3);

    let base=evo.phase_native_abstract_state(&fixture::scene(l1,0,0)).unwrap().cell;
    let p1=evo.phase_native_abstract_state(&fixture::scene(l1,1,0)).unwrap().cell;
    let p2=evo.phase_native_abstract_state(&fixture::scene(l1,2,0)).unwrap().cell;
    let born=evo.phase_native_context_witnesses();
    assert!(born.iter().any(|w|
        w.base_cell==base && w.anchor_action==1
            && w.predecessor_cells.contains(&p1)
            && w.predecessor_cells.contains(&p2)
            && w.eligible_observations==0
    ),"second structurally distinct same-base context candidate must be born");

    // Future-only anchor evidence. Alternate contexts to satisfy switch gate.
    for i in 0..40usize {
        let pred=if i%2==0{1}else{2};
        let outcome=if pred==1{7}else{8};
        enter_predecessor(evo,l1,pred,i%6);
        feed(evo,l1,1,outcome,(i+2)%6);
    }

    // One factual useful alternative for context S2. This creates ordinary
    // outgoing knowledge on the already-acquired refined state but does not
    // need a second context collision.
    enter_predecessor(evo,l1,2,4);
    feed(evo,l1,5,7,5);
}

fn target_witness(
    evo:&EvoPhase,
    l1:&[[usize;2];8],
)->aeterna_v1::carrier::PhaseContextWitness{
    let base=evo.phase_native_abstract_state(&fixture::scene(l1,0,0)).unwrap().cell;
    let p1=evo.phase_native_abstract_state(&fixture::scene(l1,1,0)).unwrap().cell;
    let p2=evo.phase_native_abstract_state(&fixture::scene(l1,2,0)).unwrap().cell;
    evo.phase_native_context_witnesses().into_iter().find(|w|
        w.base_cell==base && w.anchor_action==1
            && w.predecessor_cells.contains(&p1)
            && w.predecessor_cells.contains(&p2)
    ).expect("target context witness")
}

fn score_target(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
)->(usize,[usize;4]){
    evo.set_planning_learning_enabled(false);
    let goal=fixture::scene(l1,7,5);
    let mut correct=0usize;
    let mut ctx=[0usize;4];

    for i in 0..64usize {
        let side=i%2;
        let pred=if side==0{1}else{2};
        let expected=if side==0{1}else{5};

        evo.clear_phase_native_context_history();
        evo.observe_initial_real(&fixture::scene(l1,pred,i%6),false);
        feed(evo,l1,2,0,(i+1)%6);
        let got=evo.phase_native_context_action(&goal);
        let ok=got==(true,Some(expected));
        correct+=usize::from(ok);
        ctx[side*2+1]+=1;
        ctx[side*2]+=usize::from(ok);
    }
    (correct,ctx)
}

#[test]
fn intel1_repair4_old_same_base_candidate_does_not_block_new_context(){
    let (mut evo,l1)=fixture::build();
    create_unrelated_candidate(&mut evo,&l1);
    let control=evo.clone();

    train_target_candidate(&mut evo,&l1);
    let w=target_witness(&evo,&l1);
    println!("INTEL1_REPAIR4_TARGET {:?}",w);
    assert!(w.promoted);
    assert!(w.eligible_observations>=32);
    assert!(w.context_switches>=4);

    let base=evo.phase_native_abstract_state(&fixture::scene(&l1,0,0)).unwrap().cell;
    let same_base=evo.phase_native_context_witnesses().into_iter()
        .filter(|x|x.base_cell==base).collect::<Vec<_>>();
    println!("INTEL1_REPAIR4_SAME_BASE {:?}",same_base);
    assert!(same_base.len()>=2,
        "old and new structurally distinct hypotheses must coexist");

    let (full,ctx)=score_target(&mut evo,&l1);
    println!("INTEL1_REPAIR4_SCORE full={}/64 ctx={:?}",full,ctx);
    assert!(full>=60);
    assert!(ctx[0]>=28 && ctx[2]>=28);

    let mut old=control;
    let (old_score,_)=score_target(&mut old,&l1);
    println!("INTEL1_REPAIR4_OLD_ONLY score={}/64",old_score);
    assert!(old_score<=40);
}

#[test]
fn intel1_repair4_context_source_is_structural_not_world_keyed(){
    let source=include_str!("../src/phase_contextual.rs");
    let fanout=include_str!("../src/phase_refinement_fanout.rs");
    for required in [
        "context_find_novel_collision",
        "context_witness_signature",
        "predecessor_cells.contains",
    ] {
        assert!(source.contains(required),"missing Repair-4 dependency {required}");
    }
    for forbidden in [
        "HistoryWorld","WORLD 3","world_id","task_id","correct_action",
        "clear_on_world","intel1",
    ] {
        assert!(!source.contains(forbidden),"task key in context source: {forbidden}");
        assert!(!fanout.contains(forbidden),"task key in fanout source: {forbidden}");
    }
}
