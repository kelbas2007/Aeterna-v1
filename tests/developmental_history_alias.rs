//! A minimal *history aliasing* control. The CURRENT visual frame is
//! identical in both conditions, but an earlier externally observed cue
//! and the executed transition differ. No class or object labels are passed
//! into the carrier. Teacher-like two-step native episodes establish whether
//! the recurrent experience-state is *representationally necessary*.
//! Real generalization is separately tested by Farama MemoryS7.
use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::PhaseNativeConfig;

fn organism(memory:bool)->EvoPhase{
    let mut e=EvoPhase::new(EvoConfig {
        sensory_cells:32,motor_cells:3,dormant_cells:32,
        hdc_dim:32,..EvoConfig::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_general_policy());
    if memory {
        assert!(e.enable_phase_native_developmental_memory());
    }
    e
}
fn cue(which:usize)->Vec<f32>{
    let mut x=vec![0.0;32];
    // Fixed sensor serialization, not a game ontology.
    for i in 0..32{
        if i%4==which {x[i]=1.0;}
    }
    x
}
fn trained(memory:bool)->EvoPhase{
    let mut e=organism(memory);
    let blank=vec![0.0;32];
    for iteration in 0..120 {
        for condition in [0usize,1usize] {
            let a=cue(condition);
            e.begin_phase_native_general_episode();
            assert!(e.observe_phase_native_general_initial(&a));
            // World generated factual reset/cue/blank transitions:
            // the final action must be learned from terminal feedback.
            assert!(e.observe_phase_native_general_transition(
                2,&a,&blank,0.0
            ));
            let correct=condition;
            // A real final feedback training tuple, not a query to the
            // carryer's target motor or a hidden semantic label.
            assert!(e.observe_phase_native_general_transition(
                correct,&blank,&blank,1.0
            ));
        }
        assert_eq!(e.phase_native_general_rewards(),(iteration+1)*2);
    }
    e
}
fn final_motor(e:&mut EvoPhase,condition:usize)->usize{
    let b=vec![0.0;32];
    let a=cue(condition);
    e.begin_phase_native_general_episode();
    assert!(e.observe_phase_native_general_initial(&a));
    assert!(e.observe_phase_native_general_transition(
        2,&a,&b,0.0
    ));
    e.choose_phase_native_general_action(&b).unwrap().action
}

#[test]
fn same_visible_frame_can_have_history_dependent_action_but_reactive_policy_cannot(){
    let mut recurrent=trained(true);
    recurrent.set_planning_learning_enabled(false);
    let a=final_motor(&mut recurrent,0);
    let b=final_motor(&mut recurrent,1);
    assert_ne!(a,b,"recurrent controller must distinguish event histories");
    let snapshot=recurrent.phase_native_checkpoint().unwrap();
    let mut restored=EvoPhase::new(recurrent.config().clone());
    assert!(restored.restore_phase_native_checkpoint(snapshot));
    restored.set_planning_learning_enabled(false);
    assert_eq!(final_motor(&mut restored,0),a);
    assert_eq!(final_motor(&mut restored,1),b);
    let mut reactive=trained(false);
    reactive.set_planning_learning_enabled(false);
    let a0=final_motor(&mut reactive,0);
    let b0=final_motor(&mut reactive,1);
    assert_eq!(a0,b0,"frame-only actor cannot disambiguate identical frame");
    println!("HISTORY_ALIAS_NATIVE_PASS same_current_image=true cueA_action={} cueB_action={} no_memory_action={} checkpoint=true",a,b,a0);
}

#[test]
fn zero_experience_has_no_preinstalled_history_specific_correct_action(){
    let mut e=organism(true);
    assert_eq!(e.phase_native_general_rewards(),0);
    assert_eq!(e.phase_native_general_updates(),0);
    let blank=vec![0.0;32];
    let initial=e.choose_phase_native_general_action(&blank).unwrap();
    assert!(initial.action<3);
}
