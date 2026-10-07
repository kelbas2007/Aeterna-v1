use aeterna_v1::{EvoPhase};

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
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    action:usize,
    post_state:usize,
    layout:usize,
){
    let post=fixture::scene(l1,post_state,layout);
    assert!(evo.observe_phase_native_context_result(action,&post).is_some());
}

fn enter_base_from(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    predecessor:usize,
    layout:usize,
){
    // Move through the factual predecessor into shared base S0.
    feed(evo,l1,5,predecessor,layout%6);
    feed(evo,l1,5,0,(layout+1)%6);
}

#[allow(dead_code)]
mod r6 {
    include!("intel1_repair6_context_rival.rs");

    pub fn run_stale(seed:u64)->(bool,usize,usize,[usize;4]){
        r5::run_stale(seed)
    }

    pub fn no_context(seed:u64)->(bool,usize,usize,[usize;4]){
        r5::no_context(seed)
    }
}

#[test]
fn intel1_repair7_action_coverage_is_conditioned_on_factual_predecessor(){
    let (mut evo,l1)=fixture::build();
    let goal=fixture::scene(&l1,7,5);

    // Under predecessor S1, all six actions have already been factually seen
    // from shared base S0.
    for action in 0..6usize {
        enter_base_from(&mut evo,&l1,1,action%6);
        feed(&mut evo,&l1,action,8,(action+2)%6);
    }

    // Under predecessor S2, every action except action2 is covered.
    for action in [0usize,1,3,4,5] {
        enter_base_from(&mut evo,&l1,2,(action+1)%6);
        feed(&mut evo,&l1,action,8,(action+3)%6);
    }

    // Re-enter S0 from S2. Base-level action2 is globally known (from S1),
    // but must still be epistemically uncovered for this predecessor.
    enter_base_from(&mut evo,&l1,2,4);
    let before=evo.phase_native_context_action(&goal);
    println!("INTEL1_REPAIR7_BEFORE {:?}",before);
    assert_eq!(before,(true,Some(2)));

    // One factual action2 under S2 closes this predecessor-specific gap.
    feed(&mut evo,&l1,2,8,5);
    enter_base_from(&mut evo,&l1,2,0);
    let after=evo.phase_native_context_action(&goal);
    println!("INTEL1_REPAIR7_AFTER {:?}",after);
    assert_ne!(after,(true,Some(2)));
}

#[test]
fn intel1_repair7_stale_lifetime_discovers_useful_history_collision(){
    let seed=0x1A7E_7700u64;
    let full=r6::run_stale(seed);
    let control=r6::no_context(seed);
    println!("INTEL1_REPAIR7_HISTORY full={:?} no_context={:?}",full,control);

    assert!(full.0,"history-dependent candidate must promote");
    assert!(full.2>=32);
    assert!(full.1*64>=full.2*56,"contextual score must be >=87.5%");
    assert!(full.3[0]>=13&&full.3[2]>=13);
    assert!(control.1*64<=control.2*40);
}

#[test]
fn intel1_repair7_source_uses_predecessor_coverage_not_task_keys(){
    let source=include_str!("../src/phase_contextual.rs");
    for required in [
        "context_uncovered_action",
        "fact.predecessor == previous",
        "has_other_predecessor",
    ] {
        assert!(source.contains(required),"missing Repair-7 dependency {required}");
    }
    for forbidden in [
        "HistoryWorld","WORLD 3","world_id","task_id","history_actions",
        "correct_action","AETERNA_INTEL1_SEED",
    ] {
        assert!(!source.contains(forbidden),"task key in context source: {forbidden}");
    }
}
