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

        // The new representation must be physically causal for an action:
        // remove its learned program pathway, then restore it exactly.
        let promoted=later.iter().find(|w|w.promoted&&!w.retired)
            .expect("promoted new candidate");
        let current=raw(&l1,5,false,true,false);
        let target=goal(&l1,5);
        evo.set_planning_learning_enabled(false);
        evo.observe_initial_real(&current,false);
        let intact=evo.phase_native_compositional_action(&target);
        assert_eq!(intact,(true,Some(anchor)),
            "promoted alternative must drive action readout");
        let fingerprint=evo.phase_native_learned_fingerprint();
        let mut causal_side=None;
        for side in 0..2usize {
            let link=promoted.input_synapses[side][1];
            let mut lesioned=evo.clone();
            let old=lesioned.perturb_phase_native_synapse_for_control(
                link,0.0,0.0
            ).expect("real native synapse");
            if lesioned.phase_native_compositional_action(&target)!=intact {
                lesioned.restore_phase_native_synapse_for_control(link,old);
                assert_eq!(lesioned.phase_native_learned_fingerprint(),fingerprint);
                assert_eq!(lesioned.phase_native_compositional_action(&target),intact);
                let mut shifted=evo.clone();
                shifted.perturb_phase_native_synapse_for_control(
                    link,1.0,std::f32::consts::PI
                ).unwrap();
                assert_ne!(shifted.phase_native_compositional_action(&target),intact);
                causal_side=Some(side);
                break;
            }
        }
        assert!(causal_side.is_some(),"no causal physical program pathway found");

        // A retired candidate stays stored but does not control the winner.
        let retired=rejected.first().unwrap();
        let mut unrelated=evo.clone();
        unrelated.perturb_phase_native_synapse_for_control(
            retired.input_synapses[0][1],0.0,0.0
        ).unwrap();
        assert_eq!(unrelated.phase_native_compositional_action(&target),intact);

        let checkpoint=evo.phase_native_checkpoint().unwrap();
        let mut restarted=EvoPhase::new(fixture::cfg(&evo));
        assert!(restarted.restore_phase_native_checkpoint(checkpoint));
        restarted.observe_initial_real(&current,false);
        assert_eq!(restarted.phase_native_compositional_action(&target),intact);

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
