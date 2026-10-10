use aeterna_v1::{EvoPhase,EvoConfig,HumanProtectionEvidence};
use aeterna_v1::carrier::{PhaseNativeConfig,PhaseMetaControlConfig,PhaseHypothesisEcologyConfig};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};

const DIM:usize=588;
const FRONT:usize=26;
fn scene(kind:u8,color:u8,state:u8)->Vec<f32>{
    let mut output=vec![0.0;DIM];
    for (i,value) in [kind,color,state].iter().enumerate(){
        for bit in 0..4 {
            output[FRONT*12+i*4+bit]=((value>>bit)&1) as f32;
        }
    }
    output
}
fn safe()->HumanProtectionEvidence{
    HumanProtectionEvidence{
        human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false
    }
}
fn organism()->ScientificRuntime {
    let mut evo=EvoPhase::new(EvoConfig{
        sensory_cells:DIM,motor_cells:7,dormant_cells:128,
        hdc_dim:64,..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig{
        learning_enabled:false,..PhaseMetaControlConfig::default()
    }));
    assert!(evo.enable_phase_native_hypothesis_ecology(
        PhaseHypothesisEcologyConfig::default()));
    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_factor_causality());
    assert!(rt.enable_object_grounding(7,7,3,4,1));
    rt
}
#[test]
fn factual_demonstration_yields_transferable_word_to_motor_under_u1_and_safety(){
    // World A: arbitrary named visible category: motor 3 causes
    // an actual local object transformation at the deictic position.
    let before=scene(5,2,0);
    let after=scene(1,0,0);
    let mut rt=organism();
    rt.observe_external(&before).unwrap();
    assert!(rt.teach_pointed_word("tavi",FRONT));
    assert!(!rt.observe_demonstrated_object_action(
        "tavi",FRONT,4,&before,&before), "no-op cannot teach");
    assert!(rt.observe_demonstrated_object_action(
        "tavi",FRONT,3,&before,&after));
    assert_eq!(rt.organism().phase_native_learned_affordances(),1);
    // World B: same object type but a changed color/state. No demonstration.
    let fresh=scene(5,3,1);
    rt.restart_cognition().unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external(&fresh).unwrap();
    rt.set_goal(&vec![0.0;DIM]).unwrap();
    assert!(rt.set_grounded_word_intent("tavi"));
    let decision=rt.propose_unified().unwrap().unwrap();
    assert_eq!(decision.action,3);
    let plan=rt.organism()
        .choose_phase_native_grounded_word_action(&fresh).unwrap();
    assert_eq!(plan.action,3);
    // Both the lexical binding and factual motor relation must be physical.
    let mut damaged=rt.organism().clone();
    let saved=damaged.perturb_phase_native_synapse_for_control(
        plan.synapse,0.0,0.0).unwrap();
    assert!(damaged.choose_phase_native_grounded_word_action(&fresh).is_none());
    damaged.restore_phase_native_synapse_for_control(plan.synapse,saved);
    assert_eq!(damaged.choose_phase_native_grounded_word_action(&fresh)
        .unwrap().action,3);

    // Real external callback is invoked only via U1 and Human Protection.
    let mut received=None;
    let real=rt.step_unified(|_|Some(safe()),|motor|{
        received=Some(motor);
        Ok((after.clone(),0.0))
    }).unwrap();
    assert!(matches!(real,StepOutcome::Executed{..}));
    assert_eq!(received,Some(3));

    // Unknown category, no transfer of action merely due to shared position.
    let novel=scene(6,2,0);
    rt.observe_external(&novel).unwrap();
    assert!(rt.organism().choose_phase_native_grounded_word_action(&novel)
        .is_none());
    println!("FUNCTIONAL_GROUNDING_NATIVE_PASS category_memory=1 word=1 action=3 checkpoint=true synapse_lesion=true u1_protection=true");
}
