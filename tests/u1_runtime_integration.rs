use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::{
    PhaseCognitiveProposal,PhaseMetaControlConfig,PhaseNativeConfig,
};
use aeterna_v1::scientific_runtime::ScientificRuntime;

#[allow(dead_code)]
mod g21fixture {
    include!("g21_contextual_state_refinement.rs");

    pub fn mixed_state()->(EvoPhase,Vec<f32>){
        let (evo,mut world)=fixture::prepare(false);
        let mut runtime=ScientificRuntime::new(evo).unwrap();
        assert!(runtime.enable_context_refinement());
        runtime.observe_external(&world.sense()).unwrap();

        for _ in 0..320usize{
            set_current_goal(&mut runtime,&world);
            let _=execute_step(&mut runtime,&mut world);

            let candidates=runtime.organism().phase_native_context_witnesses();
            if candidates.iter().any(|w|!w.promoted&&!w.retired){
                let goal=world.requested_goal();
                let evo=runtime.organism().clone();

                let mut context_probe=evo.clone();
                let context=context_probe.phase_native_context_action(&goal).1;

                let mut rival_probe=evo.clone();
                let rival=rival_probe.choose_phase_native_goal_rival_probe(&goal);

                let mut epistemic=evo.clone();
                let general=epistemic.choose_phase_native_abstract_learned_drive_action();

                if context.is_some()&&rival.is_some()&&general.is_some(){
                    return (evo,goal);
                }
            }
        }
        panic!("failed to obtain simultaneous real cognitive proposals");
    }
}

fn meta_source()->EvoPhase{
    let mut evo=EvoPhase::new(EvoConfig{
        sensory_cells:8,motor_cells:6,dormant_cells:96,hdc_dim:128,
        phase_learning_rate:1.0,
        ..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig{
        learning_rate:0.35,learning_enabled:true,readout_enabled:true,
        phase_learning_enabled:true,
    }));
    let oracle=[0.36f32,0.30,0.22,0.08,0.04];
    for _ in 0..96usize{
        for i in 0..5usize{
            let mut fields=[0.0f32;5];
            fields[i]=1.0;
            assert!(evo.observe_phase_native_meta_utility(fields,oracle[i]));
        }
    }
    evo
}

fn bounded(x:f32)->f32{x.clamp(0.0,1.0)}

#[test]
fn u1_runtime_selects_same_winner_from_real_mechanisms_independent_of_order(){
    let (evo,goal)=g21fixture::mixed_state();
    let current=evo.current_real().unwrap().sensory.clone();

    let mut c=evo.clone();
    let context_action=c.phase_native_context_action(&goal).1.unwrap();
    let witness=c.phase_native_context_witnesses().into_iter()
        .find(|w|!w.promoted&&!w.retired)
        .expect("active context hypothesis");
    let evidence=bounded(witness.eligible_observations as f32/32.0);
    let context_fields=[
        0.05,
        bounded(1.0-evidence*0.5),
        1.0,
        bounded(0.25+0.75*evidence),
        0.70,
    ];

    let mut r=evo.clone();
    let rival_action=r.choose_phase_native_goal_rival_probe(&goal).unwrap();
    let disagreement=r.phase_native_goal_rival_disagreement_for_control(
        &current,&goal,rival_action
    ).unwrap_or(0.0);
    let rival_fields=[
        0.10,
        bounded(0.30+0.50*disagreement),
        bounded(disagreement),
        0.80,
        0.65,
    ];

    let mut g=evo.clone();
    let general_action=g.choose_phase_native_abstract_learned_drive_action().unwrap();
    let drive=g.phase_native_drive_weights().unwrap_or([0.0,0.0]);
    let drive_conf=bounded((drive[0]+drive[1])*0.5);
    let general_fields=[
        0.02,
        1.0,
        0.05,
        drive_conf,
        0.75,
    ];

    let proposals=[
        PhaseCognitiveProposal{
            proposal_id:0xC011,action:context_action,fields:context_fields
        },
        PhaseCognitiveProposal{
            proposal_id:0xBEEF,action:rival_action,fields:rival_fields
        },
        PhaseCognitiveProposal{
            proposal_id:0xE915,action:general_action,fields:general_fields
        },
    ];

    let source=meta_source();
    let checkpoint=source.phase_native_meta_checkpoint().unwrap();

    let mut runtime=ScientificRuntime::new(evo.clone()).unwrap();
    assert!(runtime.restore_unified_meta_control(checkpoint.clone()));

    let direct=runtime.organism()
        .choose_phase_native_meta_proposal(&proposals)
        .expect("direct physical winner");
    let via_runtime=runtime.select_unified_proposal(&proposals)
        .expect("runtime physical winner");
    assert_eq!(via_runtime,direct);

    let mut reversed=proposals;
    reversed.reverse();
    let reversed_winner=runtime.select_unified_proposal(&reversed)
        .expect("order-invariant runtime winner");
    assert_eq!(reversed_winner.proposal_id,direct.proposal_id);
    assert_eq!(reversed_winner.action,direct.action);

    // Opaque IDs can be replaced without changing fields/actions or winner role.
    let mut rekeyed=proposals;
    rekeyed[0].proposal_id=0x9911;
    rekeyed[1].proposal_id=0x1109;
    rekeyed[2].proposal_id=0x7717;
    let rekeyed_winner=runtime.select_unified_proposal(&rekeyed)
        .expect("ID-independent winner");
    let original_index=proposals.iter()
        .position(|p|p.proposal_id==direct.proposal_id).unwrap();
    assert_eq!(rekeyed_winner.proposal_id,rekeyed[original_index].proposal_id);
    assert_eq!(rekeyed_winner.action,direct.action);

    println!(
        "U1_INTEGRATION context_action={} rival_action={} general_action={} winner_index={} fields={:?}",
        context_action,rival_action,general_action,original_index,
        [context_fields,rival_fields,general_fields]
    );
}

#[test]
fn u1_runtime_transport_contains_no_module_priority_chain(){
    let source=include_str!("../src/scientific_runtime.rs");
    let start=source.find("pub fn restore_unified_meta_control").unwrap();
    let end=source[start..].find("fn record").map(|x|start+x).unwrap_or(source.len());
    let unified=&source[start..end];
    for forbidden in [
        "ContextualRefinement","PerceptualRefinement","CompositionalRefinement",
        "RivalDiscrimination","GeneralEpistemic","GoalDirectedAction",
        "phase_native_context_action","choose_phase_native_goal_rival_probe",
        "choose_phase_native_abstract_learned_drive_action",
    ]{
        assert!(!unified.contains(forbidden),
            "unified runtime transport contains authored priority token {forbidden}");
    }
    assert!(unified.contains("choose_phase_native_meta_proposal"));
}
