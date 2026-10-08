// Fresh mechanism witness for bounded hypothesis succession. This test never
// supplies the correct explanation to cognition; only factual raw inputs and
// action consequences are observed.
#[allow(dead_code)]
mod old_g23_fixture {
    include!("g23_compositional_perceptual_function.rs");

    fn record(evo:&mut EvoPhase, pre:&[f32], action:usize,
              post:&[f32], shared_fanout:bool) {
        evo.observe_initial_real(pre,false);
        let result=if shared_fanout {
            evo.observe_phase_native_refinement_fanout_result(action,post)
        }else{
            evo.observe_phase_native_compositional_result(action,post)
        };
        assert!(result.is_some(),"factual observation rejected");
    }

    pub fn run_succession(shared_fanout:bool) {
        let drive=fixture::drive();
        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        let (anchor,_other)=fixture::action_pair(false);
        assert!(evo.enable_phase_native_compositional_refinement());
        let base=evo.phase_native_abstract_state(&raw(&l1,0,false,false,false))
            .unwrap().cell;

        // Initial collision: weak A appears to explain contradictory outcomes.
        // The following 128 *future* results make A nonpredictive.
        record(&mut evo,&raw(&l1,0,false,false,false),anchor,
               &goal(&l1,0),shared_fanout);
        record(&mut evo,&raw(&l1,1,true,false,false),anchor,
               &dead(&l1,1),shared_fanout);
        let first=evo.phase_native_composition_witnesses().into_iter()
            .filter(|w|w.base_cell==base).collect::<Vec<_>>();
        assert!(!first.is_empty(),"initial wrong candidates were not born");

        for i in 0..128usize {
            let a=i%2==0;
            let useful=(i/2)%2==0;
            let pre=raw(&l1,i%4,a,false,false);
            let post=if useful{goal(&l1,i%4)}else{dead(&l1,i%4)};
            record(&mut evo,&pre,anchor,&post,shared_fanout);
        }
        let rejected=evo.phase_native_composition_witnesses().into_iter()
            .filter(|w|w.base_cell==base).collect::<Vec<_>>();
        assert_eq!(rejected.len(),first.len());
        assert!(rejected.iter().all(|w|w.retired&&!w.promoted),
            "the first hypotheses must be disproved before succession");

        // New, previously unseen weak cue B now predicts factual successors.
        // Discovery and test data are not seeded with a program label.
        for i in 0..100usize {
            let b=i%2==0;
            let pre=raw(&l1,i%4,false,b,false);
            let post=if b{goal(&l1,i%4)}else{dead(&l1,i%4)};
            record(&mut evo,&pre,anchor,&post,shared_fanout);
        }
        let later=evo.phase_native_composition_witnesses().into_iter()
            .filter(|w|w.base_cell==base).collect::<Vec<_>>();
        let born_later=later.iter().filter(|w|
            !rejected.iter().any(|old|
                old.program==w.program &&
                old.anchor_action==w.anchor_action &&
                old.successor_cells==w.successor_cells)
        ).collect::<Vec<_>>();
        assert!(!born_later.is_empty(),"no novel replacement after retirement");
        assert!(later.iter().any(|w|w.promoted && !w.retired),
            "new evidence-backed hypothesis must promote");
        assert!(rejected.iter().all(|old|later.iter().any(|w|
            w.program==old.program && w.born_fact==old.born_fact && w.retired)),
            "rejected provenance must not be deleted");
        assert!(later.len()<=16,"bounded capacity must hold");
        for (i,a) in later.iter().enumerate(){
            for b in &later[i+1..] {
                if a.program==b.program && a.anchor_action==b.anchor_action {
                    let same=(a.successor_cells==b.successor_cells)
                        || (a.successor_cells[0]==b.successor_cells[1]
                            && a.successor_cells[1]==b.successor_cells[0]);
                    assert!(!same,"exact structural duplicate regenerated");
                }
            }
        }
        println!("SUCCESSION_RESULT shared={} old={} later={} born={} promoted={} retired={}",
            shared_fanout,first.len(),later.len(),born_later.len(),
            later.iter().filter(|w|w.promoted).count(),
            later.iter().filter(|w|w.retired).count());
    }
}

#[test]
fn composition_replaces_rejected_hypotheses_direct() {
    old_g23_fixture::run_succession(false);
}

#[test]
fn composition_replaces_rejected_hypotheses_shared_fanout() {
    old_g23_fixture::run_succession(true);
}
