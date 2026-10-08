// TE3: strict trusted-runtime boundary for a learned physical sensory motor.
// The test supplies balanced FACTUAL action consequences before the runtime,
// not the correct sample/commit motor to cognition; no historical FRONTIER
// target mapping or scored pack is reused.
#[allow(dead_code)]
mod fixture {
    include!("intel2_unified_worlds.rs");
    pub fn cold()->(EvoPhase,[[usize;2];8]){foundation::build24()}
    pub fn image(l1:&[[usize;2];8],state:usize,layout:usize)->Vec<f32>{
        foundation::scene(l1,state,layout)
    }
    pub fn transferred_meta()->PhaseMetaControlCheckpoint{meta_checkpoint()}
    pub fn safe_evidence()->HumanProtectionEvidence{safe()}
}
use aeterna_v1::{EvoPhase};
use aeterna_v1::carrier::{
    PhaseTemporalEvidenceConfig,PhaseHypothesisEcologyConfig,
};
use aeterna_v1::scientific_runtime::{ScientificRuntime,StepOutcome};

fn sensory_config()->PhaseTemporalEvidenceConfig{
    PhaseTemporalEvidenceConfig{
        max_observations:8,minimum_observations:3,decisive_margin:0.125
    }
}

fn trained_runtime(
    sensor:usize,classes:[usize;3]
)->(ScientificRuntime,[[usize;2];8],Vec<f32>){
    let (mut evo,l1)=fixture::cold();
    assert!(evo.enable_phase_native_temporal_evidence(sensory_config()));
    for &class in classes[..2].iter(){
        assert!(evo.observe_phase_native_temporal_signal(
            &fixture::image(&l1,class,0)
        ));
    }
    assert!(evo.begin_phase_native_temporal_episode());

    for cycle in 0..8usize{
        let pre=fixture::image(&l1,classes[cycle%2],cycle%6);
        for motor in 0..6usize {
            let post=fixture::image(&l1,
                if motor==sensor{classes[1-cycle%2]}else{classes[cycle%2]},
                (cycle+1)%6
            );
            assert!(evo.observe_phase_native_sensing_affordance(
                motor,&pre,&post
            ));
        }
    }
    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_unified_cognition(
        fixture::transferred_meta(),
        PhaseHypothesisEcologyConfig{
            learning_rate:0.35,
            dormancy_threshold:0.05,
            learning_enabled:true,
            phase_learning_enabled:true
        }
    ));
    rt.observe_external(&fixture::image(&l1,classes[0],0)).unwrap();
    let goal=fixture::image(&l1,classes[2],1);
    rt.set_goal(&goal).unwrap();
    (rt,l1,goal)
}

#[test]
fn te3_physical_sensing_project_competes_in_unified_without_host_priority(){
    let arms=[
        (0usize,[2usize,9usize,15usize]),
        (1usize,[10usize,3usize,21usize]),
        (2usize,[7usize,14usize,23usize]),
        (3usize,[13usize,4usize,19usize]),
        (4usize,[17usize,11usize,0usize]),
        (5usize,[6usize,20usize,16usize]),
    ];
    for (sensor,classes) in arms {
        let (mut rt,l1,goal)=trained_runtime(sensor,classes);
        let belief=rt.organism().phase_native_temporal_evidence().unwrap();
        assert_eq!(belief.observations,1);
        assert!(belief.needs_more);
        let native=rt.organism().choose_phase_native_temporal_sensing_action()
            .expect("must have real physical sensory motor");
        assert_eq!(native.action,sensor);
        let proposals=rt.organism().collect_phase_native_unified_proposals(&goal);
        let sensor_project=proposals.iter().filter(|p|
            p.proposal.action==sensor
            &&p.proposal.fields[1]>=native.missing_evidence
                 *native.learned_affordance-1.0e-6
        ).collect::<Vec<_>>();
        assert!(!sensor_project.is_empty(),"physical evidence debt must enter U1 competition");
        let winner=rt.organism().choose_phase_native_unified_proposal(&proposals);
        println!("TE3_ARBITRATION sensor={sensor} native={native:?} u1={:?} candidate={proposals:?} winner={winner:?}",
            rt.organism().phase_native_meta_weights());
        assert!(winner.is_some(),"generic U1 must avoid tied active-sensing deadlock");
        let before=rt.organism().phase_native_learned_fingerprint();
        assert_eq!(proposals,rt.organism().collect_phase_native_unified_proposals(&goal));
        assert_eq!(before,rt.organism().phase_native_learned_fingerprint());

        let mut damaged=rt.organism().clone();
        let original=damaged.perturb_phase_native_synapse_for_control(
            native.synapse,0.0,0.0
        ).unwrap();
        assert!(damaged.choose_phase_native_temporal_sensing_action().is_none());
        let damaged_proposals=damaged.collect_phase_native_unified_proposals(&goal);
        assert!(proposals.iter().any(|p|
            !damaged_proposals.iter().any(|q|q.proposal.proposal_id==p.proposal.proposal_id)
        ),"damaging the acquired sensor must eliminate its U1 proposal");
        damaged.restore_phase_native_synapse_for_control(native.synapse,original);
        assert_eq!(damaged.collect_phase_native_unified_proposals(&goal),proposals);
        assert_eq!(damaged.phase_native_learned_fingerprint(),before);

        // A protection decision must not invent a sample or change the model.
        let executed=std::cell::Cell::new(0usize);
        let blocked=rt.step_unified(
            |_|Some(aeterna_v1::HumanProtectionEvidence{
                predicted_harm_probability:0.5,
                ..fixture::safe_evidence()
            }),
            |_|{executed.set(executed.get()+1);
                Ok((fixture::image(&l1,classes[1],2),0.0))}
        ).unwrap();
        assert!(matches!(blocked,StepOutcome::Blocked(_)));
        assert_eq!(executed.get(),0);
        assert_eq!(rt.organism().phase_native_temporal_evidence().unwrap().observations,1);
        assert_eq!(rt.organism().phase_native_learned_fingerprint(),before);

        // The system is allowed to pick *any* legal opaque action under U1:
        // only a real qualified sensory motor's factual POST counts a sample.
        let mut used_motor=None;
        let answer=rt.step_unified(
            |_|Some(fixture::safe_evidence()),
            |motor|{
                used_motor=Some(motor);
                let post_class=if motor==sensor{classes[1]}else{classes[0]};
                Ok((fixture::image(&l1,post_class,3),0.0))
            }
        ).unwrap();
        assert!(matches!(answer,StepOutcome::Executed{..}));
        let motor=used_motor.unwrap();
        assert_eq!(motor,sensor,
            "the physical uncertain-evidence operation must win ordinary U1");
        let observed=rt.organism().phase_native_temporal_evidence().unwrap();
        assert_eq!(observed.observations,if motor==sensor{2}else{1},
            "no-op motor must not count as an independent fresh cue");
        println!("TE3_UNIFIED sensor={} chosen={} evidence={} physically_grounded=true protection=true",
            sensor,motor,observed.observations);
    }
}

#[test]
fn te3_opt_in_external_episode_cues_never_auto_recruit_terminal_post(){
    let (evo,l1)=fixture::cold();
    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_unified_cognition(
        fixture::transferred_meta(),
        PhaseHypothesisEcologyConfig{
            learning_rate:0.35,dormancy_threshold:0.05,
            learning_enabled:true,phase_learning_enabled:true
        }
    ));
    assert!(rt.enable_temporal_evidence(sensory_config()));
    // Only external episode-start inputs may recruit the first two raw cues.
    let a=fixture::image(&l1,0,0);
    let b=fixture::image(&l1,8,1);
    let goal=fixture::image(&l1,17,2);
    rt.observe_external(&a).unwrap();
    assert_eq!(rt.organism().phase_native_temporal_source_count(),1);
    rt.observe_external(&b).unwrap();
    assert_eq!(rt.organism().phase_native_temporal_source_count(),2);
    rt.set_goal(&goal).unwrap();
    let before=rt.organism().phase_native_temporal_evidence().unwrap();
    assert_eq!(before.observations,1);
    assert_eq!(rt.organism().choose_phase_native_temporal_sensing_action(),None);
    let executed=std::cell::Cell::new(0usize);
    let outcome=rt.step_unified(
        |_|Some(aeterna_v1::HumanProtectionEvidence{
            predicted_harm_probability:0.5,
            ..fixture::safe_evidence()
        }),
        |_|{executed.set(executed.get()+1);
            Ok((fixture::image(&l1,15,3),0.0))}
    ).unwrap();
    assert!(matches!(outcome,StepOutcome::Blocked(_)));
    assert_eq!(executed.get(),0);
    assert_eq!(rt.organism().phase_native_temporal_evidence().unwrap(),before);
    println!("TE3_EXTERNAL cold_sources=2 unknown_terminal_not_registered=true protection=true");
}
