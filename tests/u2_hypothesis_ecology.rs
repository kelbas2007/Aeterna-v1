use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::{
    PhaseNativeConfig,PhaseMetaControlConfig,PhaseHypothesisEcologyConfig,
    PhaseHypothesisProposal,PhaseCognitiveProposal,
};

fn organism()->EvoPhase{
    let mut evo=EvoPhase::new(EvoConfig{
        sensory_cells:8,
        motor_cells:6,
        dormant_cells:128,
        hdc_dim:128,
        weight_learning_rate:1.0,
        phase_learning_rate:1.0,
        min_recruit_support:1,
        ..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig{
        learning_rate:0.30,
        learning_enabled:true,
        readout_enabled:true,
        phase_learning_enabled:true,
    }));
    assert!(evo.enable_phase_native_hypothesis_ecology(
        PhaseHypothesisEcologyConfig{
            learning_rate:0.30,
            dormancy_threshold:0.05,
            learning_enabled:true,
            phase_learning_enabled:true,
        }
    ));
    evo
}

fn repeat_utility(evo:&mut EvoPhase,id:u64,value:f32,n:usize){
    for _ in 0..n{
        assert!(evo.observe_phase_native_hypothesis_utility(id,value));
    }
}

fn all_applicable(ids:[u64;4])->Vec<PhaseHypothesisProposal>{
    ids.into_iter()
        .map(|candidate_id|PhaseHypothesisProposal{candidate_id,applicability:1.0})
        .collect()
}

fn winner(evo:&EvoPhase,ids:[u64;4])->Option<u64>{
    evo.choose_phase_native_hypothesis(&all_applicable(ids))
        .map(|d|d.candidate_id)
}

fn meta_proposals(
    evo:&EvoPhase,
    ids:[u64;2],
)->Vec<PhaseCognitiveProposal>{
    ids.into_iter().enumerate().map(|(index,id)|{
        let authority=evo.phase_native_hypothesis_authority(id,1.0).unwrap();
        PhaseCognitiveProposal{
            proposal_id:1000+id,
            action:index,
            // Two proposals are identical except that confidence is supplied
            // by U2's physical authority. Constant economy prevents the U1
            // normalized score from cancelling confidence magnitude.
            fields:[0.0,0.0,0.0,authority,1.0],
        }
    }).collect()
}

fn train_meta_confidence(evo:&mut EvoPhase){
    for _ in 0..24{
        assert!(evo.observe_phase_native_meta_utility(
            [0.0,0.0,0.0,1.0,0.0],1.0
        ));
        assert!(evo.observe_phase_native_meta_utility(
            [0.0,0.0,0.0,0.0,1.0],0.10
        ));
    }
    let w=evo.phase_native_meta_weights().unwrap();
    assert!(w[3]>0.90);
    assert!(w[4]<0.20);
}

#[test]
fn u2_persistent_hypothesis_ecology_dorms_switches_and_reactivates(){
    let mut evo=organism();
    train_meta_confidence(&mut evo);
    let meta_before=evo.phase_native_meta_weights().unwrap();

    // Opaque order deliberately carries no semantic meaning.
    // Roles for the evaluator only:
    // ids[0]=original useful A, ids[1]=eventually stale B,
    // ids[2]=irrelevant C, ids[3]=later-useful D.
    let ids=[91u64,7u64,42u64,13u64];
    for id in ids{
        assert!(evo.register_phase_native_hypothesis(id));
    }
    assert_eq!(evo.phase_native_hypothesis_records().len(),4);

    // Regime A: A and B useful, A strongest.
    repeat_utility(&mut evo,ids[0],0.80,18);
    repeat_utility(&mut evo,ids[1],0.60,18);
    repeat_utility(&mut evo,ids[2],0.00,8);
    repeat_utility(&mut evo,ids[3],0.00,4);
    for flip in 0..8{
        let mut p=all_applicable(ids);
        if flip%2==1{p.reverse();}
        assert_eq!(
            evo.choose_phase_native_hypothesis(&p).map(|d|d.candidate_id),
            Some(ids[0])
        );
    }
    let addresses_a=evo.phase_native_hypothesis_records();

    // U1 coupling in regime A.
    let m=meta_proposals(&evo,[ids[0],ids[3]]);
    assert_eq!(
        evo.choose_phase_native_meta_proposal(&m).map(|d|d.proposal_id),
        Some(1000+ids[0])
    );

    // Regime B: no reset/change signal. B decays, C stays useless, D becomes
    // useful. A gets no new observation but remains stored.
    repeat_utility(&mut evo,ids[1],0.00,24);
    repeat_utility(&mut evo,ids[2],0.00,12);
    repeat_utility(&mut evo,ids[3],1.00,10);

    for flip in 0..8{
        let mut p=all_applicable(ids);
        if flip%2==1{p.rotate_left(2);}
        assert_eq!(
            evo.choose_phase_native_hypothesis(&p).map(|d|d.candidate_id),
            Some(ids[3])
        );
    }
    assert_eq!(evo.phase_native_hypothesis_dormant(ids[1]),Some(true));
    assert_eq!(evo.phase_native_hypothesis_dormant(ids[2]),Some(true));
    assert_eq!(evo.phase_native_hypothesis_records().len(),4);

    let m=meta_proposals(&evo,[ids[0],ids[3]]);
    assert_eq!(
        evo.choose_phase_native_meta_proposal(&m).map(|d|d.proposal_id),
        Some(1000+ids[3])
    );

    // Regime C: evidence for A returns. No re-registration.
    repeat_utility(&mut evo,ids[0],1.00,18);
    for flip in 0..8{
        let mut p=all_applicable(ids);
        if flip%2==1{p.reverse();}
        assert_eq!(
            evo.choose_phase_native_hypothesis(&p).map(|d|d.candidate_id),
            Some(ids[0])
        );
    }
    let m=meta_proposals(&evo,[ids[0],ids[3]]);
    assert_eq!(
        evo.choose_phase_native_meta_proposal(&m).map(|d|d.proposal_id),
        Some(1000+ids[0])
    );

    let records=evo.phase_native_hypothesis_records();
    assert_eq!(records.len(),4);
    for before in &addresses_a{
        let after=records.iter()
            .find(|r|r.candidate_id==before.candidate_id).unwrap();
        assert_eq!(after.candidate_cell,before.candidate_cell);
        assert_eq!(after.utility_synapse,before.utility_synapse);
    }
    assert_eq!(evo.phase_native_meta_weights().unwrap(),meta_before);

    println!(
        "U2_LIFETIME winner=A->D->A records={:?} meta={:?}",
        records,meta_before
    );
}

#[test]
fn u2_winner_depends_on_physical_hypothesis_synapse_and_restores(){
    let mut evo=organism();
    let ids=[311u64,19u64,207u64,73u64];
    for id in ids{assert!(evo.register_phase_native_hypothesis(id));}
    repeat_utility(&mut evo,ids[0],0.90,18);
    repeat_utility(&mut evo,ids[1],0.45,18);
    repeat_utility(&mut evo,ids[2],0.10,18);
    repeat_utility(&mut evo,ids[3],0.30,18);
    assert_eq!(winner(&evo,ids),Some(ids[0]));

    let records=evo.phase_native_hypothesis_records();
    let winning=records.iter().find(|r|r.candidate_id==ids[0]).unwrap();
    let unrelated=records.iter().find(|r|r.candidate_id==ids[2]).unwrap();

    let mut lesion_lost=0;
    let mut phase_lost=0;
    let mut restored=0;
    let mut unrelated_kept=0;

    for turn in 0..4{
        let mut order=ids;
        if turn%2==1{order.reverse();}

        let mut broken=evo.clone();
        let saved=broken.perturb_phase_native_synapse_for_control(
            winning.utility_synapse,0.0,0.0
        ).unwrap();
        lesion_lost+=usize::from(winner(&broken,order)!=Some(ids[0]));
        broken.restore_phase_native_synapse_for_control(
            winning.utility_synapse,saved
        );
        restored+=usize::from(winner(&broken,order)==Some(ids[0]));

        let mut shifted=evo.clone();
        shifted.perturb_phase_native_synapse_for_control(
            winning.utility_synapse,1.0,std::f32::consts::PI
        ).unwrap();
        phase_lost+=usize::from(winner(&shifted,order)!=Some(ids[0]));

        let mut other=evo.clone();
        other.perturb_phase_native_synapse_for_control(
            unrelated.utility_synapse,0.0,0.0
        ).unwrap();
        unrelated_kept+=usize::from(winner(&other,order)==Some(ids[0]));
    }
    println!(
        "U2_CAUSAL lesion_lost={}/4 phase_lost={}/4 restore={}/4 unrelated={}/4",
        lesion_lost,phase_lost,restored,unrelated_kept
    );
    assert!(lesion_lost>=3);
    assert!(phase_lost>=3);
    assert_eq!(restored,4);
    assert!(unrelated_kept>=3);
}

#[test]
fn u2_checkpoint_preserves_provenance_utility_dormancy_and_u1_weights(){
    let mut evo=organism();
    train_meta_confidence(&mut evo);
    let ids=[501u64,111u64,902u64,66u64];
    for id in ids{assert!(evo.register_phase_native_hypothesis(id));}
    repeat_utility(&mut evo,ids[0],0.80,16);
    repeat_utility(&mut evo,ids[1],0.00,16);
    repeat_utility(&mut evo,ids[2],0.00,8);
    repeat_utility(&mut evo,ids[3],1.00,10);

    let before_records=evo.phase_native_hypothesis_records();
    let before_meta=evo.phase_native_meta_weights().unwrap();
    let before_winner=winner(&evo,ids);

    let checkpoint=evo.phase_native_checkpoint().unwrap();
    let mut restarted=EvoPhase::new(evo.config().clone());
    assert!(restarted.restore_phase_native_checkpoint(checkpoint));

    let after_records=restarted.phase_native_hypothesis_records();
    assert_eq!(after_records.len(),before_records.len());
    for before in &before_records{
        let after=after_records.iter()
            .find(|r|r.candidate_id==before.candidate_id).unwrap();
        assert_eq!(after.candidate_cell,before.candidate_cell);
        assert_eq!(after.utility_synapse,before.utility_synapse);
        assert_eq!(after.observations,before.observations);
        assert!((after.weight-before.weight).abs()<1e-6);
        assert_eq!(after.dormant,before.dormant);
    }
    assert_eq!(restarted.phase_native_meta_weights().unwrap(),before_meta);
    assert_eq!(winner(&restarted,ids),before_winner);
}

#[test]
fn u2_ecology_source_has_no_semantic_candidate_priority(){
    let source=include_str!("../src/phase_hypothesis_ecology.rs");
    for required in [
        "candidate_id",
        "applicability",
        "conductance",
        "dormancy_threshold",
        "factual_usefulness",
    ]{
        assert!(source.contains(required),"missing U2 dependency {required}");
    }
    for forbidden in [
        "world_id","task_id","ContextualRefinement","PerceptualRefinement",
        "CompositionalRefinement","RivalDiscrimination","GeneralEpistemic",
        "stale_candidate","useful_candidate","correct_candidate",
        "candidate_type","insertion_order",
    ]{
        assert!(!source.contains(forbidden),"semantic priority token {forbidden}");
    }
}
