use aeterna_v1::{EvoPhase};

#[allow(dead_code)]
mod stale {
    include!("intel1_repair4_context_competition.rs");

    pub fn prepared()->(EvoPhase,[[usize;2];8]){
        let (mut evo,l1)=fixture::build();
        create_unrelated_candidate(&mut evo,&l1);

        // Oversample only predecessor S3 on the old anchor. S4 remains the
        // missing side. This is exactly the one-sided stale condition Repair-5
        // must stop from monopolizing.
        for i in 0..4usize {
            enter_predecessor(&mut evo,&l1,3,i%6);
            feed(&mut evo,&l1,0,7,(i+1)%6);
        }
        (evo,l1)
    }

    pub fn set_predecessor(
        evo:&mut EvoPhase,l1:&[[usize;2];8],pred:usize,layout:usize
    ){
        enter_predecessor(evo,l1,pred,layout);
    }

    pub fn goal(l1:&[[usize;2];8])->Vec<f32>{
        fixture::scene(l1,7,5)
    }

    pub fn scene(l1:&[[usize;2];8],state:usize,layout:usize)->Vec<f32>{
        fixture::scene(l1,state,layout)
    }
}

#[allow(dead_code)]
mod lifetime {
    include!("intel1_unknown_worlds.rs");

    pub fn run_history(
        evo:EvoPhase,
        l1:[[usize;2];8],
        seed:u64,
    )->(bool,usize,usize,[usize;4]){
        let mut rt=ScientificRuntime::new(evo).unwrap();

        let mut motors=[0usize,1,2,3,4,5];
        let mut rng=Rng::new(seed^0xA55);
        shuffle(&mut rng,&mut motors);
        let mut hw=HistoryWorld::new(seed^0x55,[motors[4],motors[5]]);
        rt.observe_external(&foundation::scene(&l1,hw.state,0)).unwrap();

        for k in 0..1600usize {
            let goal_state=if hw.state==7 {5} else {7};
            rt.set_goal(&foundation::scene(&l1,goal_state,1)).unwrap();
            let layout=k%4;
            let result=rt.step(|_|Some(safe()),|a|{
                let next=hw.step(a);
                Ok(foundation::scene(&l1,next,layout))
            });
            match result {
                Ok(StepOutcome::Executed{..}) => {}
                Ok(StepOutcome::GoalReached) => continue,
                Ok(other) => {
                    println!("INTEL1_REPAIR5_STOP k={} state={} outcome={:?}",k,hw.state,other);
                    break;
                }
                Err(error) => {
                    println!(
                        "INTEL1_REPAIR5_STOP k={} state={} error={:?} candidates={:?}",
                        k,hw.state,error,rt.organism().phase_native_context_witnesses()
                    );
                    break;
                }
            }
            if hw.total>=128 {break;}
        }

        let promoted=rt.organism().phase_native_context_witnesses()
            .iter().any(|w|w.promoted);
        let scored=hw.ctx[1]+hw.ctx[3];
        println!(
            "INTEL1_REPAIR5_HISTORY promoted={} correct={}/{} ctx={:?} witnesses={:?}",
            promoted,hw.score_correct,scored,hw.ctx,
            rt.organism().phase_native_context_witnesses()
        );
        (promoted,hw.score_correct,scored,hw.ctx)
    }
}

#[allow(dead_code)]
mod r3 {
    include!("intel1_repair3_fanout.rs");
    pub fn no_context(seed:u64)->(bool,usize,usize,[usize;4]){
        intel::history_with_all_refiners(seed,false)
    }
}

#[test]
fn intel1_repair5_stale_candidate_yields_when_current_side_is_oversampled(){
    let (mut evo,l1)=stale::prepared();
    let goal=stale::goal(&l1);

    // S3 is deliberately oversampled relative to S4.
    stale::set_predecessor(&mut evo,&l1,3,0);
    let over=evo.phase_native_context_action(&goal);
    println!("INTEL1_REPAIR5_OVER {:?}",over);
    assert_ne!(over,(true,Some(0)),
        "over-sampled side must not keep monopolizing the old anchor");

    // S4 is the missing side of the same candidate.
    stale::set_predecessor(&mut evo,&l1,4,2);
    let under=evo.phase_native_context_action(&goal);
    println!("INTEL1_REPAIR5_UNDER {:?}",under);
    assert_eq!(under,(true,Some(0)),
        "under-sampled side should still request the candidate anchor");
}

#[test]
fn intel1_repair5_stale_history_does_not_block_new_context_learning(){
    let seed=0x1A7E_5500u64;
    let (evo,l1)=stale::prepared();
    let full=lifetime::run_history(evo,l1,seed);
    let control=r3::no_context(seed);

    println!("INTEL1_REPAIR5_COMPARE full={:?} no_context={:?}",full,control);
    assert!(full.0,"useful later context must promote despite stale candidate");
    assert!(full.2>=32);
    assert!(full.1*64>=full.2*56,"contextual accuracy must be >=87.5%");
    assert!(full.3[0]>=13&&full.3[2]>=13);
    assert!(control.1*64<=control.2*40);
}

#[test]
fn intel1_repair5_source_uses_information_balance_not_world_keys(){
    let source=include_str!("../src/phase_contextual.rs");
    for required in [
        "current <= opposite",
        "context_counts",
        "current + opposite",
    ] {
        assert!(source.contains(required),"missing Repair-5 dependency {required}");
    }
    for forbidden in [
        "HistoryWorld","WORLD 3","world_id","task_id","history_actions",
        "correct_action","AETERNA_INTEL1_SEED",
    ] {
        assert!(!source.contains(forbidden),"task key in contextual source: {forbidden}");
    }
}
