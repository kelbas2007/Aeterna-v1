#![allow(dead_code)]
// STRUCTURE-1 mechanism repair, development only. The cognitive network
// receives just actual raw PRE/action/POST and opaque motor IDs. A physically
// necessary two-action sensor chain is acquired without naming the roles.
// This deliberately prepared causal acquisition is NOT a cold qualification.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[test]
fn chain_motor_affordance_is_physical_contextual_and_checkpointed(){
    let (mut evo,l1)=foundation::build24();
    let source=[4usize,13usize];
    let marker=22usize;
    let first=3usize;
    let second=0usize;
    assert!(evo.enable_phase_native_temporal_evidence(
        PhaseTemporalEvidenceConfig{
            max_observations:8,minimum_observations:3,
            decisive_margin:0.125,
        }
    ));
    for cue in source {
        assert!(evo.observe_phase_native_temporal_signal(
            &foundation::scene(&l1,cue,0)
        ));
    }
    assert!(evo.begin_phase_native_temporal_episode());
    assert!(evo.set_phase_native_temporal_chain_learning(true));
    assert!(evo.set_phase_native_temporal_autonomous_probe(true));
    let cue0=foundation::scene(&l1,source[0],0);
    let cue1=foundation::scene(&l1,source[1],0);
    let ready=foundation::scene(&l1,marker,0);
    assert!(evo.observe_phase_native_sensing_affordance(first,&cue0,&ready));
    evo.observe_initial_real(&ready,false);
    let (probe,_)=evo.choose_phase_native_temporal_unknown_probe()
        .expect("physical intermediate should create information debt");
    assert_ne!(probe,first,"a distinct motor must now be investigated");
    // The outside world responds to the executed second action. No API
    // receives the name 'read', hidden-cause bit or correct motor.
    assert!(evo.observe_phase_native_sensing_affordance(probe,&ready,&cue1));
    assert_eq!(evo.phase_native_temporal_chain_count(),1);
    assert!(evo.phase_native_temporal_factual_sample_from(probe,&ready,&cue1));
    assert!(!evo.phase_native_temporal_factual_sample_from(probe,&cue0,&cue1),
        "reading without the acquired physical marker must not count");
    assert!(evo.observe_phase_native_temporal_signal(&cue1));
    assert!(evo.begin_phase_native_temporal_episode());
    assert!(evo.observe_phase_native_temporal_signal(&cue0));

    evo.observe_initial_real(&cue0,false);
    let arm=evo.choose_phase_native_temporal_sensing_action()
        .expect("learned first motor");
    assert_eq!(arm.action,first);
    evo.observe_initial_real(&ready,false);
    let read=evo.choose_phase_native_temporal_sensing_action()
        .expect("learned second motor");
    assert_eq!(read.action,probe);
    assert!(evo.phase_native_temporal_action_affordance(probe).unwrap()>0.0);

    let checkpoint=evo.phase_native_checkpoint().unwrap();
    let mut restored=EvoPhase::new(evo.config().clone());
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    assert!(restored.phase_native_temporal_chain_learning());
    restored.observe_initial_real(&cue0,false);
    assert_eq!(restored.choose_phase_native_temporal_sensing_action(),Some(arm));
    let entry=restored.perturb_phase_native_synapse_for_control(
        arm.synapse,0.0,0.0).unwrap();
    assert!(restored.choose_phase_native_temporal_sensing_action().is_none());
    restored.restore_phase_native_synapse_for_control(arm.synapse,entry);
    restored.observe_initial_real(&ready,false);
    assert_eq!(restored.choose_phase_native_temporal_sensing_action(),Some(read));
    let link=restored.perturb_phase_native_synapse_for_control(
        read.synapse,0.0,0.0).unwrap();
    assert!(restored.choose_phase_native_temporal_sensing_action().is_none());
    assert!(!restored.phase_native_temporal_factual_sample_from(
        probe,&ready,&cue1
    ));
    restored.restore_phase_native_synapse_for_control(read.synapse,link);
    assert_eq!(restored.choose_phase_native_temporal_sensing_action(),Some(read));
    for _ in 0..2 {
        assert!(restored.observe_phase_native_temporal_signal(&cue0));
    }
    let terminal=restored.choose_phase_native_temporal_outcome_probe()
        .expect("unknown terminal action");
    assert_ne!(terminal.0,first);
    assert_ne!(terminal.0,probe,
        "an information-carrying motor is not an untried terminal candidate");
    println!("STRUCTURE1_CHAIN_MECHANISM physical_two_step=true prepost_context=true lesion_both=true checkpoint=true");
}
