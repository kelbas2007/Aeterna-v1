use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::{
    PhaseMetaControlConfig,PhaseNativeConfig,PhaseHypothesisEcologyConfig,
};
use aeterna_v1::scientific_runtime::{ScientificRuntime,StepOutcome};
use aeterna_v1::{HumanProtectionEvidence};
use std::cell::Cell;

#[allow(dead_code)]
mod fixture {
    include!("g22_perceptual_variable_invention.rs");

    pub fn target()->(EvoPhase,Vec<f32>,Vec<f32>){
        let drive=fixture::drive();
        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        train_useful(&mut evo,&l1,false);
        let current=fixture::scene(&l1,0,4,Some((MARKERS[2],A_VALUE)));
        let goal=fixture::scene(&l1,7,5,None);
        evo.observe_initial_real(&current,false);
        (evo,current,goal)
    }

    pub fn post_scene()->Vec<f32>{
        let drive=fixture::drive();
        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        fixture::scene(&l1,7,0,None)
    }
}

fn meta_checkpoint()->aeterna_v1::carrier::PhaseMetaControlCheckpoint{
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
    for _ in 0..96{
        for i in 0..5{
            let mut fields=[0.0f32;5];
            fields[i]=1.0;
            assert!(evo.observe_phase_native_meta_utility(fields,oracle[i]));
        }
    }
    evo.phase_native_meta_checkpoint().unwrap()
}

fn safe()->HumanProtectionEvidence{
    HumanProtectionEvidence{
        human_present:true,physical_effect_possible:true,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false,
    }
}

fn high_risk()->HumanProtectionEvidence{
    HumanProtectionEvidence{
        predicted_harm_probability:0.5,
        ..safe()
    }
}

fn transition_support(
    evo:&EvoPhase,
    pre:&[f32],
    action:usize,
    post:&[f32],
)->u64{
    let a=evo.phase_native_abstract_state(pre).unwrap().cell;
    let z=evo.phase_native_abstract_state(post).unwrap().cell;
    let motor=evo.config().sensory_cells+action;
    evo.phase_native_circuits().iter().filter(|c|{
        let aff=evo.phase_native_synapse(c.afferent_synapse).unwrap();
        let succ=evo.phase_native_synapse(c.successor_synapse).unwrap();
        let out=evo.phase_native_synapse(c.motor_synapse).unwrap();
        aff.from==a&&succ.to==z&&out.to==motor
    }).map(|c|c.support).sum()
}

#[test]
fn unified_runtime_uses_carrier_winner_then_protected_single_fact_and_generic_credit(){
    let (evo,current,goal)=fixture::target();
    let post=current.clone();
    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_unified_cognition(
        meta_checkpoint(),
        PhaseHypothesisEcologyConfig{
            learning_rate:0.35,dormancy_threshold:0.05,
            learning_enabled:true,phase_learning_enabled:true,
        }
    ));
    rt.observe_external(&current).unwrap();
    rt.set_goal(&goal).unwrap();

    let proposals=rt.organism().collect_phase_native_unified_proposals(&goal);
    println!("UNIFIED_ASSEMBLY_PROPOSALS {:?}",proposals);
    assert!(proposals.len()>=3,"need simultaneous heterogeneous proposals");
    let direct=rt.organism().choose_phase_native_unified_proposal(&proposals)
        .expect("carrier winner");
    assert!(direct.persistent_candidate_id.is_some(),
        "assembly witness requires a persistent explanatory winner");

    let mut reversed=proposals.clone();
    reversed.reverse();
    let rev=rt.organism().choose_phase_native_unified_proposal(&reversed).unwrap();
    assert_eq!(rev.action,direct.action);
    assert_eq!(rev.persistent_candidate_id,direct.persistent_candidate_id);

    let proposed=rt.propose_unified().unwrap().unwrap();
    assert_eq!(proposed.action,direct.action);

    // Blocked proposal must never become a fact or authority update.
    let before_fp=rt.organism().phase_native_learned_fingerprint();
    let before_tick=rt.organism().current_real().unwrap().tick;
    let calls=Cell::new(0usize);
    let blocked=rt.step_unified(
        |_|Some(high_risk()),
        |_|{
            calls.set(calls.get()+1);
            Ok((post.clone(),1.0))
        }
    ).unwrap();
    assert!(matches!(blocked,StepOutcome::Blocked(_)));
    assert_eq!(calls.get(),0);
    assert_eq!(rt.organism().phase_native_learned_fingerprint(),before_fp);
    assert_eq!(rt.organism().current_real().unwrap().tick,before_tick);

    let proposals=rt.organism().collect_phase_native_unified_proposals(&goal);
    let direct=rt.organism().choose_phase_native_unified_proposal(&proposals).unwrap();
    let candidate=direct.persistent_candidate_id.unwrap();
    let before_support=transition_support(rt.organism(),&current,direct.action,&post);
    let before_tick=rt.organism().current_real().unwrap().tick;

    let executed=rt.step_unified(
        |_|Some(safe()),
        |action|{
            assert_eq!(action,direct.action);
            Ok((post.clone(),1.0))
        }
    ).unwrap();
    assert!(matches!(executed,StepOutcome::Executed{..}));
    assert_eq!(rt.organism().current_real().unwrap().tick,before_tick+1);
    assert_eq!(
        transition_support(rt.organism(),&current,direct.action,&post),
        before_support+1,
        "one external POST must update shared parent transition exactly once"
    );
    assert!(rt.organism().phase_native_hypothesis_registered(candidate));
    let record=rt.organism().phase_native_hypothesis_records().into_iter()
        .find(|r|r.candidate_id==candidate).unwrap();
    assert!(record.weight>0.0&&record.observations==1);

    // Repeated no-gain facts are credited generically to all applicable stored
    // hypotheses. U2 regression separately verifies later reactivation.
    for _ in 0..8 {
        let _ = rt.step_unified(
            |_|Some(safe()),
            |_|Ok((current.clone(),0.0))
        );
    }
    assert_eq!(rt.organism().phase_native_hypothesis_dormant(candidate),Some(true));

    let records_before=rt.organism().phase_native_hypothesis_records();
    let meta_before=rt.organism().phase_native_meta_weights().unwrap();
    rt.restart_cognition().unwrap();
    assert_eq!(rt.organism().phase_native_meta_weights().unwrap(),meta_before);
    let records_after=rt.organism().phase_native_hypothesis_records();
    assert_eq!(records_after.len(),records_before.len());
    for before in records_before{
        let after=records_after.iter()
            .find(|r|r.candidate_id==before.candidate_id).unwrap();
        assert_eq!(after.candidate_cell,before.candidate_cell);
        assert_eq!(after.utility_synapse,before.utility_synapse);
        assert!((after.weight-before.weight).abs()<1e-6);
    }
}

#[test]
fn unified_runtime_path_contains_no_legacy_priority_call(){
    let source=include_str!("../src/scientific_runtime.rs");
    let start=source.find("pub fn step_unified").unwrap();
    let end=source[start..].find("/// In-memory cognitive restart")
        .map(|x|start+x).unwrap_or(source.len());
    let unified=&source[start..end];
    assert!(unified.contains("select_unified_internal"));
    assert!(unified.contains("consume_permit(permit)"));
    assert!(unified.find("consume_permit(permit)").unwrap()
        < unified.find("execute(action)").unwrap());
    assert!(!unified.contains("self.propose()"));
    assert!(!unified.contains("ContextualRefinement"));
    assert!(!unified.contains("PerceptualRefinement"));
    assert!(!unified.contains("RivalDiscrimination"));
    assert!(unified.contains("evidence_gain"));
    assert!(unified.contains("post_proposals"));
}
