use aeterna_v1::{EvoPhase};

#[allow(dead_code)]
mod r5 {
    include!("intel1_repair5_context_balance.rs");

    pub fn prepared()->(EvoPhase,[[usize;2];8]){
        stale::prepared()
    }

    pub fn set_pred(
        evo:&mut EvoPhase,l1:&[[usize;2];8],pred:usize,layout:usize
    ){
        stale::set_predecessor(evo,l1,pred,layout);
    }

    pub fn goal(l1:&[[usize;2];8])->Vec<f32>{
        stale::goal(l1)
    }

    pub fn run_stale(seed:u64)->(bool,usize,usize,[usize;4]){
        let (evo,l1)=stale::prepared();
        lifetime::run_history(evo,l1,seed)
    }

    pub fn no_context(seed:u64)->(bool,usize,usize,[usize;4]){
        r3::no_context(seed)
    }
}

#[test]
fn intel1_repair6_rival_probe_defers_when_context_side_is_oversampled(){
    let (mut evo,l1)=r5::prepared();
    let goal=r5::goal(&l1);

    // S3 is over-sampled for the old context candidate.
    r5::set_pred(&mut evo,&l1,3,0);
    let context=evo.phase_native_context_action(&goal);
    let rival=evo.choose_phase_native_goal_rival_probe(&goal);
    println!("INTEL1_REPAIR6_OVER context={:?} rival={:?}",context,rival);
    assert_ne!(context,(true,Some(0)));
    assert_ne!(rival,Some(0),
        "G19 must not reissue a rival probe that G21 already considers over-sampled");

    // S4 is the missing side of the same old candidate. Its anchor remains
    // information-bearing and must still be available.
    r5::set_pred(&mut evo,&l1,4,2);
    let context_under=evo.phase_native_context_action(&goal);
    let rival_under=evo.choose_phase_native_goal_rival_probe(&goal);
    println!(
        "INTEL1_REPAIR6_UNDER context={:?} rival={:?}",
        context_under,rival_under
    );
    assert_eq!(context_under,(true,Some(0)));
    assert_eq!(rival_under,Some(0));
}

#[test]
fn intel1_repair6_stale_rival_does_not_block_later_history_learning(){
    let seed=0x1A7E_6600u64;
    let full=r5::run_stale(seed);
    let control=r5::no_context(seed);

    println!("INTEL1_REPAIR6_HISTORY full={:?} no_context={:?}",full,control);
    assert!(full.0,"useful history hypothesis must promote");
    assert!(full.2>=32);
    assert!(full.1*64>=full.2*56,
        "contextual score must be >=87.5%");
    assert!(full.3[0]>=13&&full.3[2]>=13);
    assert!(control.1*64<=control.2*40);
}

#[test]
fn intel1_repair6_source_is_context_owned_not_world_keyed(){
    let context=include_str!("../src/phase_contextual.rs");
    let rival=include_str!("../src/phase_abstract_planning.rs");

    for required in [
        "phase_context_rival_probe_allowed",
        "current <= opposite",
        "context_probe_allowed",
    ] {
        assert!(
            context.contains(required)||rival.contains(required),
            "missing Repair-6 dependency {required}"
        );
    }

    for forbidden in [
        "HistoryWorld","WORLD 3","world_id","task_id","history_actions",
        "correct_action","AETERNA_INTEL1_SEED",
    ] {
        assert!(!context.contains(forbidden),
            "task key in contextual source: {forbidden}");
        assert!(!rival.contains(forbidden),
            "task key in rival source: {forbidden}");
    }
}
