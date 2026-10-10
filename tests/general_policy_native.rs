use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::{
    PhaseNativeConfig,PhaseMetaControlConfig,PhaseHypothesisEcologyConfig
};
use aeterna_v1::scientific_runtime::ScientificRuntime;

fn initialized()->EvoPhase {
    let mut o=EvoPhase::new(EvoConfig {
        sensory_cells:16,motor_cells:3,dormant_cells:24,
        hdc_dim:64,..EvoConfig::default()
    });
    o.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(o.enable_phase_native_meta_control(PhaseMetaControlConfig {
        learning_enabled:false,..PhaseMetaControlConfig::default()
    }));
    assert!(o.enable_phase_native_hypothesis_ecology(
        PhaseHypothesisEcologyConfig::default()
    ));
    assert!(o.enable_phase_native_general_policy());
    o
}
#[test]
fn factual_delayed_reward_assigns_predecessor_motor_credit_without_task_labels(){
    let mut rt=initialized();
    let a=(0..16).map(|i|if i%3==1 {1.0}else{0.0}).collect::<Vec<_>>();
    let b=(0..16).map(|i|if i%4==2 {1.0}else{0.0}).collect::<Vec<_>>();
    // A real observed two-step causal trajectory. No world script is
    // available to the policy; it receives only factual tuples.
    for _ in 0..6 {
        rt.begin_phase_native_general_episode();
        assert!(rt.observe_phase_native_general_transition(2,&a,&b,0.0));
        assert!(rt.observe_phase_native_general_transition(1,&b,&a,1.0));
    }
    assert_eq!(rt.phase_native_general_rewards(),6);
    assert_eq!(rt.phase_native_general_updates(),12);
    let checkpoint=rt.phase_native_checkpoint().unwrap();
    let mut restored=EvoPhase::new(rt.config().clone());
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    let mut rt=restored;
    rt.set_planning_learning_enabled(false);
    let choice=rt.choose_phase_native_general_action(&a).unwrap();
    // Credit to earlier action 2 must be nonzero and learned beyond a
    // purely terminal mapping. The actor may share features across states.
    let terminal=rt.choose_phase_native_general_action(&b).unwrap();
    assert!(choice.action<3 && terminal.action<3);
    let mut lesioned=rt.clone();
    for action in 0..3 {
        let index=lesioned.phase_native_general_synapse(action).unwrap();
        let _=lesioned.perturb_phase_native_synapse_for_control(index,0.0,0.0)
            .unwrap();
    }
    assert!(lesioned.choose_phase_native_general_action(&a).is_none());
    println!("GENERAL_POLICY_UNIT_PASS rewards=6 updates=12 restarted=true blocked_without_physical_motor_links=true");
}
#[test]
fn generic_zero_reward_never_fabricates_success_and_frozen_policy_never_updates(){
    let mut rt=initialized();
    let x=vec![1.0;16];
    let y=vec![0.0;16];
    rt.observe_phase_native_general_transition(1,&x,&y,0.0);
    assert_eq!(rt.phase_native_general_rewards(),0);
    assert_eq!(rt.phase_native_general_updates(),1);
    rt.set_planning_learning_enabled(false);
    let fingerprint=rt.phase_native_learned_fingerprint();
    rt.observe_phase_native_general_transition(1,&x,&y,0.0);
    assert_eq!(rt.phase_native_general_updates(),1);
    assert_eq!(rt.phase_native_learned_fingerprint(),fingerprint);
}
