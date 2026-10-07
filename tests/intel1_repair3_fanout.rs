#[allow(dead_code)]
mod intel {
    include!("intel1_unknown_worlds.rs");

    pub fn history_with_all_refiners(seed:u64, context_on:bool)->(bool,usize,usize,[usize;4]){
        let (evo,l1)=foundation::build();
        let mut rt=ScientificRuntime::new(evo).unwrap();
        if context_on { assert!(rt.enable_context_refinement()); }
        assert!(rt.enable_perceptual_refinement());
        assert!(rt.enable_compositional_refinement());

        let mut motors=[0usize,1,2,3,4,5];
        let mut rng=Rng::new(seed^0xA33);
        shuffle(&mut rng,&mut motors);
        let mut hw=HistoryWorld::new(seed^0x33,[motors[4],motors[5]]);
        rt.observe_external(&foundation::scene(&l1,hw.state,0)).unwrap();

        for k in 0..1200usize {
            let layout=k%4;
            let goal_state=if hw.state==7 {5} else {7};
            rt.set_goal(&foundation::scene(&l1,goal_state,1)).unwrap();
            let outcome=rt.step(|_|Some(safe()),|a|{
                let next=hw.step(a);
                Ok(foundation::scene(&l1,next,layout))
            });
            match outcome {
                Ok(StepOutcome::Executed{..}) => {}
                Ok(other) => {
                    println!(
                        "INTEL1_REPAIR3_STOP k={} state={} total={} outcome={:?} ctx={} percept={} comp={} latched={}",
                        k,hw.state,hw.total,other,
                        rt.organism().phase_native_context_witnesses().len(),
                        rt.organism().phase_native_perceptual_witnesses().len(),
                        rt.organism().phase_native_composition_witnesses().len(),
                        rt.emergency_latched()
                    );
                    break;
                }
                Err(error) => {
                    println!(
                        "INTEL1_REPAIR3_STOP k={} state={} total={} error={:?} ctx={} percept={} comp={}",
                        k,hw.state,hw.total,error,
                        rt.organism().phase_native_context_witnesses().len(),
                        rt.organism().phase_native_perceptual_witnesses().len(),
                        rt.organism().phase_native_composition_witnesses().len()
                    );
                    break;
                }
            }
            if hw.total>=128 { break; }
        }

        let promoted=rt.organism().phase_native_context_witnesses()
            .iter().any(|w|w.promoted);
        let scored=hw.ctx[1]+hw.ctx[3];
        (promoted,hw.score_correct,scored,hw.ctx)
    }

    pub fn one_fact_parent_support(seed:u64)->(u64,usize){
        let (evo,l1)=foundation::build();
        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_context_refinement());
        assert!(rt.enable_perceptual_refinement());
        assert!(rt.enable_compositional_refinement());

        let pre=foundation::scene(&l1,0,0);
        let post=foundation::scene(&l1,1,1);
        let goal=foundation::scene(&l1,7,2);
        rt.observe_external(&pre).unwrap();
        rt.set_goal(&goal).unwrap();

        let chosen=Cell::new(usize::MAX);
        let result=rt.step(|_|Some(safe()),|a|{
            chosen.set(a);
            Ok(post.clone())
        }).unwrap();
        assert!(matches!(result,StepOutcome::Executed{..}));
        let action=chosen.get();

        let pre_ref=rt.organism().phase_native_abstract_state(&pre).unwrap();
        let post_ref=rt.organism().phase_native_abstract_state(&post).unwrap();
        let motor=rt.organism().config().sensory_cells+action;
        let support=rt.organism().phase_native_circuits().iter()
            .filter(|c|{
                let a=rt.organism().phase_native_synapse(c.afferent_synapse).unwrap();
                let z=rt.organism().phase_native_synapse(c.successor_synapse).unwrap();
                let m=rt.organism().phase_native_synapse(c.motor_synapse).unwrap();
                a.from==pre_ref.cell && z.to==post_ref.cell && m.to==motor
            })
            .map(|c|c.support).sum::<u64>();
        (support,action)
    }
}

#[test]
fn intel1_repair3_all_enabled_learners_receive_one_history_stream(){
    let seed=0x1A7E_3300u64;
    let full=intel::history_with_all_refiners(seed,true);
    let control=intel::history_with_all_refiners(seed,false);
    println!("INTEL1_REPAIR3_HISTORY full={:?} no_context={:?}",full,control);
    assert!(full.0,"context witness must promote while all three refiners are enabled");
    assert!(full.2>=32);
    assert!(full.1*64 >= full.2*56,"FULL contextual accuracy must be >=87.5%");
    assert!(full.3[0]>=13 && full.3[2]>=13);
    assert!(control.1*64 <= control.2*40,
        "without context, identical junction should remain near memoryless ceiling");
}

#[test]
fn intel1_repair3_one_external_fact_updates_parent_support_once(){
    let (support,action)=intel::one_fact_parent_support(0x3311);
    println!("INTEL1_REPAIR3_SUPPORT action={} parent_support={}",action,support);
    assert_eq!(support,1,
        "one executed fact must not be multiplied by representation fanout");
}

#[test]
fn intel1_repair3_runtime_uses_generic_fanout_after_protected_execution(){
    let host=include_str!("../src/scientific_runtime.rs");
    let fanout=include_str!("../src/phase_refinement_fanout.rs");
    assert!(host.contains("observe_phase_native_refinement_fanout_result"));
    assert!(!host.contains("WORLD 3"));
    assert!(!fanout.contains("HistoryWorld"));
    assert!(!fanout.contains("motors["));
    assert!(host.find("consume_permit(permit)").unwrap()
        < host.find("match execute(action)").unwrap());
}
