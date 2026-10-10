use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::PhaseNativeConfig;

fn subject()->EvoPhase {
    let mut e=EvoPhase::new(EvoConfig{
        sensory_cells:32,motor_cells:3,dormant_cells:32,
        hdc_dim:32,..EvoConfig::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_general_policy());
    assert!(e.enable_phase_native_developmental_memory());
    assert!(e.enable_phase_native_episodic_recall());
    assert!(e.enable_phase_native_experience_replay());
    e
}
fn cue(which:usize)->Vec<f32>{
    (0..32).map(|i|if i%4==which {1.0}else{0.0}).collect()
}
#[test]
fn native_replays_real_successful_sequences_and_failed_episodes_without_oracle(){
    let mut e=subject();
    let blank=vec![0.0;32];
    assert_eq!(e.phase_native_episodic_count(),0);
    assert_eq!(e.phase_native_episodic_failures(),0);
    let a=cue(0);
    e.begin_phase_native_general_episode();
    assert!(e.observe_phase_native_general_initial(&a));
    assert!(e.observe_phase_native_general_transition(2,&a,&blank,0.0));
    assert!(e.observe_phase_native_general_transition(0,&blank,&blank,1.0));
    assert!(e.phase_native_episodic_count()>=2,
        "whole preceding trajectory must acquire positive associations");
    assert_eq!(e.phase_native_episodic_failures(),0);

    // Failed self-action sequence with a DIFFERENT prior cue, with NO
    // terminal reward. The only failure evidence is the whole real
    // episode ending without the objective being achieved.
    let b=cue(1);
    e.begin_phase_native_general_episode();
    assert!(e.observe_phase_native_general_initial(&b));
    assert!(e.observe_phase_native_general_transition(2,&b,&blank,0.0));
    assert!(e.observe_phase_native_general_transition(1,&blank,&blank,0.0));
    e.begin_phase_native_general_episode();
    assert!(e.phase_native_episodic_failures()>=2);
    let rewarded=e.phase_native_episodic_count();
    let failed=e.phase_native_episodic_failures();
    let checkpoint=e.phase_native_checkpoint().unwrap();
    let mut restored=EvoPhase::new(e.config().clone());
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    assert_eq!(restored.phase_native_episodic_count(),rewarded);
    assert_eq!(restored.phase_native_episodic_failures(),failed);

    restored.set_planning_learning_enabled(false);
    let fp=restored.phase_native_learned_fingerprint();
    let observe=cue(0);
    restored.begin_phase_native_general_episode();
    assert!(restored.observe_phase_native_general_initial(&observe));
    assert!(restored.observe_phase_native_general_transition(
        2,&observe,&blank,0.0
    ));
    assert_eq!(restored.phase_native_learned_fingerprint(),fp,
        "a frozen thought episode must not modify persistent knowledge");
    let mut dead=restored.clone();
    for motor in 0..3{
        let link=dead.phase_native_general_synapse(motor).unwrap();
        dead.perturb_phase_native_synapse_for_control(link,0.0,0.0)
            .unwrap();
    }
    assert!(dead.choose_phase_native_general_action(&blank).is_none(),
        "recalled intention may NOT override missing physical motor");
    println!("AUTOBIOGRAPHICAL_REPLAY_NATIVE_PASS whole_success_events={} failed_events={} checkpoint=true frozen=true motor_lesion=true",
        rewarded,failed);
}
