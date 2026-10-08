#![allow(dead_code)]
// TE5: generic phase-native belief-conditioned outcome credit.
// Physical cue sources and opaque motor labels are randomized independently;
// cognition receives only accumulated factual cues, action and POST utility.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

fn te5_train(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    cues:[usize;2],
    goal:usize,
    failure:usize,
    rewarded:[usize;2],
    n:usize,
){
    for _ in 0..n {
        for side in 0..2usize {
            for action in 0..6usize {
                assert!(evo.begin_phase_native_temporal_episode());
                for sample in 0..5usize {
                    assert!(evo.observe_phase_native_temporal_signal(
                        &foundation::scene(l1,cues[side],sample%6)
                    ));
                }
                let (post,reward)=if action==rewarded[side]{
                    (goal,1.0)
                }else{
                    (failure,0.0)
                };
                assert!(evo.observe_phase_native_temporal_outcome(
                    action,&foundation::scene(l1,post,2),reward
                ));
            }
        }
    }
}

#[test]
fn te5_six_independent_cue_motor_mappings_are_learned_and_physically_necessary(){
    const SEED:u64=0x7E55_2026_A77E_2026;
    let mut rng=Rng::new(SEED);
    for arm in 0..6usize {
        let (mut evo,l1)=foundation::build24();
        let mut classes=(0..24usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut classes);
        let cues=[classes[0],classes[1]];
        let goal=classes[2];let failure=classes[3];
        let mut motors=[0usize,1,2,3,4,5];
        shuffle(&mut rng,&mut motors);
        let useful=[motors[0],motors[1]];
        assert_ne!(useful[0],useful[1]);
        assert!(evo.enable_phase_native_temporal_evidence(
            PhaseTemporalEvidenceConfig {
                max_observations:8,
                minimum_observations:3,
                decisive_margin:0.125,
            }
        ));
        // Only cue-source identities are acquired from real raster input.
        // Neither class nor correct terminal motor is supplied to EvoPhase.
        for &cue in &cues {
            assert!(evo.observe_phase_native_temporal_signal(
                &foundation::scene(&l1,cue,0)
            ));
        }
        assert!(evo.begin_phase_native_temporal_episode());
        te5_train(&mut evo,&l1,cues,goal,failure,useful,12);
        evo.set_planning_learning_enabled(false);
        let mut credited=0usize;
        for side in 0..2 {
            assert!(evo.begin_phase_native_temporal_episode());
            for step in 0..5usize {
                assert!(evo.observe_phase_native_temporal_signal(
                    &foundation::scene(&l1,cues[side],step%6)
                ));
            }
            let physical=evo.phase_native_temporal_evidence().unwrap();
            assert!(physical.winner_cell.is_some());
            let winner=evo.choose_phase_native_temporal_outcome_action()
                .expect("factually learned phase-native outcome");
            assert_eq!(winner.action,useful[side]);
            assert!(winner.learned_value>0.75);
            credited+=1;
            let fingerprint=evo.phase_native_learned_fingerprint();
            assert_eq!(evo.choose_phase_native_temporal_outcome_action(),
                Some(winner));
            assert_eq!(evo.phase_native_learned_fingerprint(),fingerprint);

            let mut cut=evo.clone();
            let saved=cut.perturb_phase_native_synapse_for_control(
                winner.synapse,0.0,0.0
            ).unwrap();
            assert!(cut.choose_phase_native_temporal_outcome_action().is_none());
            cut.restore_phase_native_synapse_for_control(winner.synapse,saved);
            assert_eq!(cut.choose_phase_native_temporal_outcome_action(),
                Some(winner));
            assert_eq!(cut.phase_native_learned_fingerprint(),fingerprint);

            let mut phase_shift=evo.clone();
            let before=phase_shift.perturb_phase_native_synapse_for_control(
                winner.synapse,1.0,std::f32::consts::PI
            ).unwrap();
            assert!(phase_shift.choose_phase_native_temporal_outcome_action()
                .is_none());
            phase_shift.restore_phase_native_synapse_for_control(
                winner.synapse,before
            );
            assert_eq!(phase_shift.choose_phase_native_temporal_outcome_action(),
                Some(winner));

            let checkpoint=evo.phase_native_checkpoint().unwrap();
            let mut after=EvoPhase::new(evo.config().clone());
            assert!(after.restore_phase_native_checkpoint(checkpoint));
            assert_eq!(after.choose_phase_native_temporal_outcome_action(),
                Some(winner));
        }
        assert_eq!(credited,2);
        evo.set_planning_learning_enabled(true);

        // Genuine later contradictory factual outcomes revise old same
        // physical cue->motor links. No correct-motor hint is used by learning.
        let inverted=[useful[1],useful[0]];
        te5_train(&mut evo,&l1,cues,goal,failure,inverted,40);
        for side in 0..2usize {
            assert!(evo.begin_phase_native_temporal_episode());
            for i in 0..5usize {
                assert!(evo.observe_phase_native_temporal_signal(
                    &foundation::scene(&l1,cues[side],i%6)
                ));
            }
            let result=evo.choose_phase_native_temporal_outcome_action()
                .expect("revision of a physical reward association");
            assert_eq!(result.action,inverted[side]);
        }
        println!("TE5_PHYSICAL_ARM arm={} cue_classes={:?} first_actions={:?} revised_actions={:?} credited=2/2 lesion=true pi=true checkpoint=true reversal=true",
            arm,cues,useful,inverted);
    }
    println!("TE5_PHYSICAL_RESULT correct=12/12 causal=true reversed=12/12");
}
