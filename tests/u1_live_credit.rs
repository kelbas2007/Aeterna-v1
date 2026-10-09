#![allow(dead_code)]
// Development-only: the previously acquired opaque sensing motor competes in
// the ordinary protected U1 path, and factual POST (not prepared utility rows)
// now updates the EXISTING physical meta controller when explicitly enabled.
// This test is NOT a cold end-to-end intelligence qualification.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;
use aeterna_v1::scientific_runtime::LifetimeEventKind;

#[test]
fn u1_online_credit_depends_on_protected_factual_sensing_and_freezes() {
    let (mut evo,l1)=foundation::build24();
    let cues=[2usize,9usize];
    let sensor=0usize;
    assert!(evo.enable_phase_native_temporal_evidence(
        PhaseTemporalEvidenceConfig {
            max_observations:8,minimum_observations:3,decisive_margin:0.125
        }
    ));
    for &cue in &cues {
        assert!(evo.observe_phase_native_temporal_signal(
            &foundation::scene(&l1,cue,0)
        ));
    }
    assert!(evo.begin_phase_native_temporal_episode());
    // Prepared SENSOR AFFORDANCE only. No prepared U1 task utility or reward
    // is supplied after this point; U1 receives factual outcome online.
    for cycle in 0..8usize {
        let source=cues[cycle%2];
        let other=cues[1-cycle%2];
        for motor in 0..6usize {
            let post=if motor==sensor{other}else{source};
            assert!(evo.observe_phase_native_sensing_affordance(
                motor,
                &foundation::scene(&l1,source,cycle%6),
                &foundation::scene(&l1,post,(cycle+1)%6)
            ));
        }
    }
    let mut runtime=ScientificRuntime::new(evo).unwrap();
    assert!(runtime.enable_unified_cognition(
        meta_checkpoint(),
        PhaseHypothesisEcologyConfig {
            learning_rate:0.35,dormancy_threshold:0.05,
            learning_enabled:true,phase_learning_enabled:true,
        }
    ));
    runtime.observe_external(&foundation::scene(&l1,cues[0],0)).unwrap();
    runtime.set_goal(&foundation::scene(&l1,17,1)).unwrap();
    runtime.set_unified_decision_tracing(true);

    let before=runtime.organism().phase_native_meta_observations();
    let readout=runtime.inspect_unified_competition().expect("physical U1");
    let winner=readout.iter().max_by(|a,b|
        a.score.total_cmp(&b.score)
    ).expect("competition winner");
    assert_eq!(winner.action,sensor,"in this prepared physical affordance control");

    // No actuator permit -> no fact, no U1 credit and no audit of an execution.
    assert!(runtime.set_unified_online_learning(true));
    let blocked_calls=std::cell::Cell::new(0);
    let blocked=runtime.step_unified(
        |_|Some(HumanProtectionEvidence{
            predicted_harm_probability:0.5,..safe()
        }),
        |_|{
            blocked_calls.set(blocked_calls.get()+1);
            Ok((foundation::scene(&l1,cues[1],1),0.0))
        }
    ).unwrap();
    assert!(matches!(blocked,StepOutcome::Blocked(_)));
    assert_eq!(blocked_calls.get(),0);
    assert_eq!(runtime.organism().phase_native_meta_observations(),before);

    let mut actual=None;
    let executed=runtime.step_unified(
        |_|Some(safe()),
        |action|{
            actual=Some(action);
            let post=if action==sensor{cues[1]}else{cues[0]};
            Ok((foundation::scene(&l1,post,2),0.0))
        }
    ).unwrap();
    assert!(matches!(executed,StepOutcome::Executed{..}));
    assert_eq!(actual,Some(sensor));
    assert_eq!(runtime.organism().phase_native_meta_observations(),before+1);
    let event=runtime.audit().iter().rev().find_map(|record|
        match &record.kind {
            LifetimeEventKind::UnifiedDecisionTrace(trace)=>Some(trace),
            _=>None,
        }
    ).expect("bounded factual trace");
    assert_eq!(event.selected_action,sensor);
    assert_eq!(event.factual_outcome,Some(0.0));
    assert!(event.acquired_sample);
    assert!(event.online_utility.unwrap_or(0.0)>0.0);
    assert!(event.actions.iter().any(|a|a.action==sensor));
    assert_eq!(
        runtime.organism().phase_native_temporal_evidence().unwrap().observations,
        2
    );

    // Cognitive persistence must retain the learned U1 physical links.
    let weights=runtime.organism().phase_native_meta_weights().unwrap();
    runtime.restart_cognition().unwrap();
    assert_eq!(runtime.organism().phase_native_meta_weights().unwrap(),weights);
    runtime.set_model_learning_enabled(false);
    runtime.observe_external(&foundation::scene(&l1,cues[0],3)).unwrap();
    let frozen_before=runtime.organism().phase_native_meta_observations();
    let _=runtime.step_unified(
        |_|Some(safe()),
        |action|{
            let post=if action==sensor{cues[1]}else{cues[0]};
            Ok((foundation::scene(&l1,post,4),0.0))
        },
    );
    assert_eq!(runtime.organism().phase_native_meta_observations(),frozen_before);
    println!("U1_LIVE_CREDIT protected=true factual=true meta_updates=1 checkpoint=true frozen=true");
}
