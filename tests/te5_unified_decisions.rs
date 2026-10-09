#![allow(dead_code)]
// TE5 U1 integration: factual acquisition is separate from readout and
// unambiguously recorded as prepared balanced motor experience.
// During evaluation ALL choices go through protected step_unified.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[test]
fn te5_unified_selects_acquired_sensing_then_belief_grounded_goal_motor(){
    let arms=[
        ([2usize,13usize,19usize,20usize],0usize,[4usize,3usize]),
        ([5usize,17usize,21usize,14usize],5usize,[1usize,2usize]),
        ([8usize,15usize,22usize,1usize],3usize,[0usize,4usize]),
        ([4usize,12usize,16usize,23usize],2usize,[5usize,1usize]),
        ([9usize,0usize,18usize,6usize],4usize,[2usize,3usize]),
        ([10usize,7usize,11usize,3usize],1usize,[4usize,0usize]),
    ];
    for (arm,(classes,sensor,terminal)) in arms.into_iter().enumerate(){
        let (mut evo,l1)=foundation::build24();
        assert!(evo.enable_phase_native_temporal_evidence(
            PhaseTemporalEvidenceConfig {
                max_observations:8,minimum_observations:3,
                decisive_margin:0.125
            }
        ));
        // Two physical cue identities acquired from raw factual observations.
        for &cue in &classes[..2] {
            assert!(evo.observe_phase_native_temporal_signal(
                &foundation::scene(&l1,cue,0)
            ));
        }
        assert!(evo.begin_phase_native_temporal_episode());

        // Matched factual exposure, no motor names enter the organism.
        // For each motor, the evaluator returns only its actual raw POST.
        for trial in 0..16 {
            let pre_cue=classes[trial%2];
            let other_cue=classes[1-trial%2];
            for motor in 0..6 {
                let post=if motor==sensor{other_cue}else{pre_cue};
                assert!(evo.observe_phase_native_sensing_affordance(
                    motor,
                    &foundation::scene(&l1,pre_cue,trial%6),
                    &foundation::scene(&l1,post,(trial+1)%6),
                ));
            }
        }
        // Balanced factual task consequences condition reward associations
        // on the physically observed cue, without hidden class labels.
        for _ in 0..16 {
            for side in 0..2 {
                for motor in 0..6 {
                    assert!(evo.begin_phase_native_temporal_episode());
                    for i in 0..5 {
                        assert!(evo.observe_phase_native_temporal_signal(
                            &foundation::scene(&l1,classes[side],i%6)
                        ));
                    }
                    let good=motor==terminal[side];
                    assert!(evo.observe_phase_native_temporal_outcome(
                        motor,
                        &foundation::scene(
                            &l1,if good{classes[2]}else{classes[3]},1
                        ),
                        f32::from(good),
                    ));
                }
            }
        }

        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_unified_cognition(
            meta_checkpoint(),
            PhaseHypothesisEcologyConfig{
                learning_rate:0.35,dormancy_threshold:0.05,
                learning_enabled:true,phase_learning_enabled:true
            }
        ));
        rt.set_model_learning_enabled(false);
        let raw_goal=foundation::scene(&l1,classes[2],3);
        rt.observe_external(&foundation::scene(&l1,classes[0],0)).unwrap();
        rt.set_goal(&raw_goal).unwrap();
        assert_eq!(rt.organism().phase_native_temporal_evidence().unwrap()
            .observations,1);
        let mut actual=Vec::<usize>::new();
        let blocked_calls=std::cell::Cell::new(0usize);
        let blocked=rt.step_unified(
            |_|Some(HumanProtectionEvidence{
                predicted_harm_probability:0.5,
                ..safe()
            }),
            |_|{
                blocked_calls.set(blocked_calls.get()+1);
                Ok((foundation::scene(&l1,classes[1],1),0.0))
            }
        ).unwrap();
        assert!(matches!(blocked,StepOutcome::Blocked(_)));
        assert_eq!(blocked_calls.get(),0);
        assert_eq!(rt.organism().phase_native_temporal_evidence().unwrap()
            .observations,1);

        for k in 0..5 {
            if rt.goal_reached().unwrap(){break;}
            let outcome=rt.step_unified(
                |_|Some(safe()),
                |motor|{
                    actual.push(motor);
                    // No evaluator-side action selection. The environment's
                    // factual POST is determined by the motor actually chosen.
                    let next=if motor==sensor{classes[0]}
                        else if motor==terminal[0]{classes[2]}
                        else{classes[3]};
                    Ok((foundation::scene(&l1,next,(k+1)%6),
                        f32::from(next==classes[2])))
                }
            ).unwrap();
            assert!(matches!(outcome,StepOutcome::Executed{..}));
        }
        assert_eq!(actual,[sensor,sensor,terminal[0]],
            "U1 must autonomously change from observing to learned commitment");
        assert!(rt.goal_reached().unwrap());
        println!("TE5_UNIFIED_ARM arm={} chosen={:?} native_sensor={} learned_commit={} safety=PASS",
            arm,actual,sensor,terminal[0]);
    }
    println!("TE5_UNIFIED_RESULT protected_sequence=6/6");
}
