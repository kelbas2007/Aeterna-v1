use aeterna_v1::EvoPhase;

#[allow(dead_code)]
mod fixture {
    include!("g19_rival_hypothesis_discrimination.rs");

    pub fn build()->(EvoPhase,[[usize;2];8]){
        let drive=train_drive();
        let mut evo=target(&drive);
        let l1=train_abstraction(&mut evo);
        assert!(evo.enable_phase_native_context_refinement());
        (evo,l1)
    }
    pub fn scene(l1:&[[usize;2];8],state:usize,layout:usize)->Vec<f32>{
        state_scene(l1,state,LAYOUTS[layout%LAYOUTS.len()])
    }
}

fn feed(
    evo:&mut EvoPhase,l1:&[[usize;2];8],
    action:usize,post:usize,layout:usize,
){
    let raw=fixture::scene(l1,post,layout);
    assert!(evo.observe_phase_native_context_result(action,&raw).is_some());
}

fn enter(
    evo:&mut EvoPhase,l1:&[[usize;2];8],
    predecessor:usize,layout:usize,
){
    evo.clear_phase_native_context_history();
    evo.observe_initial_real(&fixture::scene(l1,predecessor,layout),false);
    feed(evo,l1,5,0,(layout+1)%6);
}

fn train_two_promoted(
    evo:&mut EvoPhase,l1:&[[usize;2];8],
){
    // Candidate A, anchor0: pred1 -> S3, pred2 -> S4.
    enter(evo,l1,1,0); feed(evo,l1,0,3,1);
    enter(evo,l1,2,2); feed(evo,l1,0,4,3);

    // Candidate B, anchor1: reverse successors.
    enter(evo,l1,1,4); feed(evo,l1,1,4,5);
    enter(evo,l1,2,0); feed(evo,l1,1,3,1);

    // Future-only alternating evidence promotes both.
    for i in 0..16usize {
        enter(evo,l1,1,i%6); feed(evo,l1,0,3,(i+1)%6);
        enter(evo,l1,2,(i+2)%6); feed(evo,l1,0,4,(i+3)%6);
        enter(evo,l1,1,(i+4)%6); feed(evo,l1,1,4,(i+5)%6);
        enter(evo,l1,2,(i+1)%6); feed(evo,l1,1,3,(i+2)%6);
    }

    // A longer route from S3 to the goal gives a fallback if a direct refined
    // link is damaged.
    evo.clear_phase_native_context_history();
    evo.observe_initial_real(&fixture::scene(l1,3,0),false);
    feed(evo,l1,4,7,1);

    // Common direct goal action2 learned from both factual predecessors. Every
    // applicable promoted refined state receives this ordinary factual action.
    enter(evo,l1,1,2); feed(evo,l1,2,7,3);
    enter(evo,l1,2,4); feed(evo,l1,2,7,5);
}

fn current_from_pred(
    evo:&mut EvoPhase,l1:&[[usize;2];8],pred:usize
){
    enter(evo,l1,pred,0);
}

#[test]
fn intel1_repair8_promoted_contexts_act_only_on_physical_plan_consensus(){
    let (mut evo,l1)=fixture::build();
    train_two_promoted(&mut evo,&l1);

    let base=evo.phase_native_abstract_state(&fixture::scene(&l1,0,0)).unwrap().cell;
    let pred1=evo.phase_native_abstract_state(&fixture::scene(&l1,1,0)).unwrap().cell;
    let goal_raw=fixture::scene(&l1,7,5);
    let goal=evo.phase_native_abstract_state(&goal_raw).unwrap().cell;

    let promoted=evo.phase_native_context_witnesses().into_iter()
        .filter(|w|w.base_cell==base&&w.promoted&&w.predecessor_cells.contains(&pred1))
        .collect::<Vec<_>>();
    println!("INTEL1_REPAIR8_PROMOTED {:?}",promoted);
    assert!(promoted.len()>=2);

    evo.set_planning_learning_enabled(false);
    current_from_pred(&mut evo,&l1,1);
    let agreed=evo.phase_native_context_action(&goal_raw);
    println!("INTEL1_REPAIR8_AGREE {:?}",agreed);
    assert_eq!(agreed,(true,Some(2)));

    // Damage only one promoted refined state's direct action2 -> goal link.
    // That state still owns a learned anchor0 -> S3 -> goal route; the other
    // promoted state retains direct action2. The two physical plans now
    // disagree, so readout must remain fail-closed.
    let target=promoted.iter().find(|w|w.anchor_action==0)
        .expect("anchor0 promoted witness");
    let side=target.predecessor_cells.iter().position(|&p|p==pred1).unwrap();
    let state_cell=target.state_cells[side];
    let motor=evo.config().sensory_cells+2;
    let circuit=evo.phase_native_circuits().iter().find(|c|{
        let a=evo.phase_native_synapse(c.afferent_synapse).unwrap();
        let s=evo.phase_native_synapse(c.successor_synapse).unwrap();
        let m=evo.phase_native_synapse(c.motor_synapse).unwrap();
        a.from==state_cell&&s.to==goal&&m.to==motor
    }).expect("target refined direct-goal circuit").clone();

    evo.perturb_phase_native_synapse_for_control(
        circuit.successor_synapse,0.0,0.0
    ).unwrap();
    current_from_pred(&mut evo,&l1,1);
    let disagree=evo.phase_native_context_action(&goal_raw);
    println!("INTEL1_REPAIR8_DISAGREE {:?}",disagree);
    assert_eq!(disagree,(true,None));
}

#[allow(dead_code)]
mod r7 {
    include!("intel1_repair7_context_coverage.rs");
    pub fn system(seed:u64)->(bool,usize,usize,[usize;4]){
        r6::run_stale(seed)
    }
    pub fn control(seed:u64)->(bool,usize,usize,[usize;4]){
        r6::no_context(seed)
    }
}

#[test]
fn intel1_repair8_stale_lifetime_exploits_promoted_context_consensus(){
    let seed=0x1A7E_8800u64;
    let full=r7::system(seed);
    let control=r7::control(seed);
    println!("INTEL1_REPAIR8_HISTORY full={:?} no_context={:?}",full,control);

    assert!(full.0);
    assert!(full.2>=32);
    assert!(full.1*64>=full.2*56);
    assert!(full.3[0]>=13&&full.3[2]>=13);
    assert!(control.1*64<=control.2*40);
}

#[test]
fn intel1_repair8_source_requires_plan_consensus_not_candidate_identity(){
    let source=include_str!("../src/phase_contextual.rs");
    for required in [
        "consensus",
        "decision.first_action",
        "active_states.len() > 1",
    ] {
        assert!(source.contains(required),"missing Repair-8 dependency {required}");
    }
    for forbidden in [
        "HistoryWorld","WORLD 3","world_id","task_id","history_actions",
        "correct_action","AETERNA_INTEL1_SEED",
    ] {
        assert!(!source.contains(forbidden),"task key in context source: {forbidden}");
    }
}
