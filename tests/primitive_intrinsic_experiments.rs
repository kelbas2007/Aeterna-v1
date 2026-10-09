#![allow(dead_code)]
// The external world presents a continuing hidden relation and factual
// rewards, not an ordered lesson list or a motor sequence. The organism
// generates its own experiment from live physical candidate disagreement.
// Its generic source operation was acquired earlier; this is NOT cold AGI.
include!("primitive_argument_transfer.rs");

#[test]
fn intrinsic_argument_disagreement_selects_its_own_protected_experiment(){
    use aeterna_v1::scientific_runtime::{ReasoningMode,StepOutcome};
    let mut e=acquired();
    assert!(e.set_phase_primitive_argument_transfer(true));
    assert!(e.enable_phase_primitive_synaptic_argument_competition());
    assert!(e.set_phase_native_intrinsic_argument_experiments(true));
    let mut rt=ScientificRuntime::new(e).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let probe_frame=input(0b001,false);
    let observed=probe_frame.iter().copied().map(Some).collect::<Vec<_>>();
    let wanted=rt.organism()
        .phase_native_intrinsic_argument_probe(&observed,&[false;3])
        .expect("live rival hypothesis disagreement is an intrinsic goal");
    rt.observe_external(&probe_frame).unwrap();
    let first=rt.propose_vector().unwrap().unwrap();
    assert_eq!(first.action,wanted.0);
    assert_eq!(first.mode,ReasoningMode::VectorExperiment);

    // One evolving world: no [task0,task1] training schedule.
    // A hidden fixed relation of its physical input channels determines
    // actual motor consequences. The agent is never given this pair.
    let mut state=0xB7E3_2026_6415_A17Cu64;
    let mut interest_episodes=0usize;
    let mut factual_actions=0usize;
    for _ in 0..256{
        state^=state<<13;state^=state>>7;state^=state<<17;
        let bits=(state%8) as usize;
        let x=input(bits,false);
        let raw=x.iter().copied().map(Some).collect::<Vec<_>>();
        interest_episodes+=usize::from(
            rt.organism().phase_native_intrinsic_argument_probe(
                &raw,&[false;3]
            ).is_some()
        );
        let before=rt.organism().phase_native_learned_fingerprint();
        let calls=world::teach(&mut rt,&x,xor_of(bits,[1,2]),
            &world::roles(0),None);
        factual_actions+=calls;
        assert_ne!(rt.organism().phase_native_learned_fingerprint(),before,
            "factual action must update physical hypothesis evidence");
    }
    assert!(interest_episodes>0);
    assert!(factual_actions>0);
    let bindings=rt.organism().phase_primitive_argument_bindings();
    assert_eq!(bindings.len(),2,
        "world-discovered physical argument winners, no assigned curriculum");
    let checkpoint=rt.organism().online_checkpoint_bytes().unwrap();
    let mut restored=EvoPhase::from_online_checkpoint(&checkpoint).unwrap();
    // Suppress plasticity for the unseen numeric variants.
    restored.set_planning_learning_enabled(false);
    let mut right=0usize;
    for variant in 0..3 {
        for bits in 0..8 {
            let frame=changed_values(bits,variant).into_iter()
                .map(Some).collect::<Vec<_>>();
            let predicted=restored.phase_primitive_argument_prediction(&frame)
                .expect("acquired physical candidate winner");
            right+=usize::from(world::roles(0)[predicted.action]
                ==xor_of(bits,[1,2]));
        }
    }
    assert_eq!(right,24);
    println!("PRIMITIVE_INTRINSIC_OPEN correct=24/24 self_selected_probe=true interest_episodes={} factual_actions={} checkpoint=true no_motor_tuition=true one_unscheduled_world=true",interest_episodes,factual_actions);
}
