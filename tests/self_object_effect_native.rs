#![allow(dead_code)]
include!("functional_grounding_native.rs");

#[test]
fn protected_self_experiment_acquires_object_motor_without_teacher_action() {
    let mut rt=organism();
    assert!(rt.enable_self_object_experiment(FRONT));
    let before=scene(5,2,0);
    let after=scene(1,0,0);
    let dummy=vec![0.0;DIM];
    let mut executed=Vec::new();
    for attempt in 0..7 {
        rt.observe_external(&before).unwrap();
        rt.set_goal(&dummy).unwrap();
        let pre=rt.organism()
            .choose_phase_native_self_object_action(&before).unwrap();
        let chosen=pre.action;
        assert!(!pre.learned);
        let result=rt.step_unified(|_|Some(safe()),|motor|{
            executed.push(motor);
            Ok((if motor==3 {after.clone()} else {before.clone()},0.0))
        }).unwrap();
        assert!(matches!(result,StepOutcome::Executed{..}));
        if chosen==3 {break;}
        assert_eq!(attempt+1,chosen+1);
    }
    assert_eq!(executed,vec![0,1,2,3]);
    assert_eq!(rt.organism().phase_native_self_affordance_count(),1);
    assert_eq!(rt.organism().phase_native_self_trial_count(),4);
    rt.restart_cognition().unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external(&scene(5,4,2)).unwrap();
    rt.set_goal(&dummy).unwrap();
    let raw=scene(5,4,2);
    let remembered=rt.organism()
        .choose_phase_native_self_object_action(&raw).unwrap();
    assert_eq!(remembered.action,3);
    assert!(remembered.learned);
    let mut lesioned=rt.organism().clone();
    let link=remembered.synapse.unwrap();
    let saved=lesioned.perturb_phase_native_synapse_for_control(
        link,0.0,0.0).unwrap();
    assert!(lesioned.choose_phase_native_self_object_action(&raw).is_none(),
        "frozen organism cannot pretend an unpowered physical skill exists");
    lesioned.restore_phase_native_synapse_for_control(link,saved);
    assert_eq!(lesioned.choose_phase_native_self_object_action(&raw)
        .unwrap().action,3);
    let mut native_motor=None;
    let reply=rt.step_unified(|_|Some(safe()),|motor|{
        native_motor=Some(motor);
        Ok((after.clone(),0.0))
    }).unwrap();
    assert!(matches!(reply,StepOutcome::Executed{..}));
    assert_eq!(native_motor,Some(3));
    println!("SELF_OBJECT_NATIVE_PASS trials=4 self_discovered_motor=3 checkpoint=true lesion=true U1=true HP=true");
}

#[test]
fn self_object_episode_does_not_repeat_same_motor_at_identical_observation() {
    let mut rt=organism();
    assert!(rt.enable_self_object_experiment(FRONT));
    let initial=scene(5,2,0);
    rt.observe_external(&initial).unwrap();
    rt.set_goal(&vec![0.0;DIM]).unwrap();
    let mut tried=Vec::new();
    // Exactly the SAME external sensory state and unchanged factual POST
    // after every action: an episodic action budget must not retry a motor.
    for _ in 0..7 {
        let proposed=rt.organism()
            .choose_phase_native_self_object_action(&initial)
            .expect("seven untried opaque actions exist");
        assert!(!tried.contains(&proposed.action));
        let outcome=rt.step_unified(|_|Some(safe()),|motor|{
            assert_eq!(motor,proposed.action);
            Ok((initial.clone(),0.0))
        }).unwrap();
        assert!(matches!(outcome,StepOutcome::Executed{..}));
        tried.push(proposed.action);
    }
    assert_eq!(tried,vec![0,1,2,3,4,5,6]);
    assert!(rt.organism().choose_phase_native_self_object_action(&initial).is_none());
    assert_eq!(rt.organism().phase_native_self_trial_count(),7);
    // Reset clears the EPISODIC cooldown but MUST preserve factual
    // lifetime trial counts. The selected motor after reset is therefore
    // deliberately not constrained to equal the first action.
    rt.observe_external(&initial).unwrap();
    assert!(rt.organism().choose_phase_native_self_object_action(&initial)
        .is_some());
    assert_eq!(rt.organism().phase_native_self_trial_count(),7);
    println!("SELF_OBJECT_EPISODIC_COOLDOWN_PASS attempts=7 unique=7 exhausted=true reset=true");
}
