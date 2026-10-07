use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::{
    PhaseCognitiveProposal,PhaseHypothesisEcologyConfig,PhaseHypothesisProposal,
    PhaseMetaControlConfig,PhaseNativeConfig,
};

const IDS:[u64;4]=[0xA17,0xF03,0x771,0xC55];

fn carrier()->EvoPhase{
    let mut evo=EvoPhase::new(EvoConfig{
        sensory_cells:8,
        motor_cells:6,
        dormant_cells:128,
        hdc_dim:128,
        phase_learning_rate:1.0,
        ..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig{
        learning_rate:0.35,
        learning_enabled:true,
        readout_enabled:true,
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
    evo.set_phase_native_meta_learning_enabled(false);

    assert!(evo.enable_phase_native_hypothesis_ecology(
        PhaseHypothesisEcologyConfig{
            learning_rate:0.30,
            dormancy_threshold:0.05,
            learning_enabled:true,
            phase_learning_enabled:true,
        }
    ));
    for id in IDS{
        assert!(evo.register_phase_native_hypothesis(id));
    }
    evo
}

fn train(evo:&mut EvoPhase,id:u64,value:f32,n:usize){
    for _ in 0..n{
        assert!(evo.observe_phase_native_hypothesis_utility(id,value));
    }
}

fn proposals(ids:[u64;4])->[PhaseHypothesisProposal;4]{
    ids.map(|candidate_id|PhaseHypothesisProposal{
        candidate_id,applicability:1.0
    })
}

fn assert_wins_eight(evo:&EvoPhase,expected:u64){
    for i in 0..8usize{
        let mut p=proposals(IDS);
        if i%2==1{p.reverse();}
        let got=evo.choose_phase_native_hypothesis(&p)
            .expect("unique active hypothesis");
        assert_eq!(got.candidate_id,expected);
    }
}

fn records(evo:&EvoPhase)
    ->Vec<aeterna_v1::carrier::PhaseHypothesisRecordInfo>
{
    let mut r=evo.phase_native_hypothesis_records();
    r.sort_by_key(|x|x.candidate_id);
    r
}

fn meta_pair(evo:&EvoPhase,a:u64,b:u64)->u64{
    let aa=evo.phase_native_hypothesis_authority(a,1.0).unwrap();
    let bb=evo.phase_native_hypothesis_authority(b,1.0).unwrap();
    let p=[
        PhaseCognitiveProposal{
            proposal_id:a,action:0,
            fields:[0.0,0.0,0.0,aa,0.10]
        },
        PhaseCognitiveProposal{
            proposal_id:b,action:1,
            fields:[0.0,0.0,0.0,bb,0.10]
        },
    ];
    evo.choose_phase_native_meta_proposal(&p)
        .expect("U1 unique ecology-modulated winner").proposal_id
}

#[test]
fn u2_hypothesis_ecology_preserves_dorms_and_reactivates(){
    let mut evo=carrier();
    let meta_weights=evo.phase_native_meta_weights().unwrap();
    let initial=records(&evo);
    assert_eq!(initial.len(),4);

    let original=IDS[0];
    let stale=IDS[1];
    let irrelevant=IDS[2];
    let later=IDS[3];

    // Regime A.
    train(&mut evo,original,0.75,16);
    train(&mut evo,stale,0.55,16);
    train(&mut evo,irrelevant,0.0,16);
    train(&mut evo,later,0.0,16);

    println!("U2_A {:?}",records(&evo));
    assert_wins_eight(&evo,original);
    assert_eq!(meta_pair(&evo,original,stale),original);
    assert_eq!(records(&evo).len(),4);

    // Regime B. No world/change flag enters cognition.
    train(&mut evo,stale,0.0,16);
    train(&mut evo,irrelevant,0.0,16);
    train(&mut evo,later,1.0,16);

    let b=records(&evo);
    println!("U2_B {:?}",b);
    assert_wins_eight(&evo,later);
    assert_eq!(meta_pair(&evo,original,later),later);
    assert_eq!(evo.phase_native_hypothesis_dormant(stale),Some(true));
    assert_eq!(evo.phase_native_hypothesis_dormant(irrelevant),Some(true));
    assert!(b.iter().any(|r|r.candidate_id==original));
    assert_eq!(b.len(),4);

    // Regime C. Old useful evidence returns; same candidate must reactivate.
    train(&mut evo,original,1.0,16);
    train(&mut evo,later,0.0,16);

    let c=records(&evo);
    println!("U2_C {:?}",c);
    assert_wins_eight(&evo,original);
    assert_eq!(meta_pair(&evo,original,later),original);
    assert_eq!(evo.phase_native_hypothesis_dormant(original),Some(false));
    assert_eq!(c.len(),4);

    for before in &initial{
        let after=c.iter().find(|r|r.candidate_id==before.candidate_id).unwrap();
        assert_eq!(after.candidate_cell,before.candidate_cell);
        assert_eq!(after.utility_synapse,before.utility_synapse);
    }
    assert_eq!(evo.phase_native_meta_weights().unwrap(),meta_weights);

    // Native checkpoint preserves candidate ecology without restoring REAL.
    let checkpoint=evo.phase_native_checkpoint().unwrap();
    let mut restored=EvoPhase::new(evo.config().clone());
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    assert!(restored.current_real().is_none());
    assert_eq!(records(&restored),c);
    assert_wins_eight(&restored,original);
    assert_eq!(restored.phase_native_meta_weights().unwrap(),meta_weights);
}

#[test]
fn u2_hypothesis_authority_is_physically_causal(){
    let mut evo=carrier();
    let winner=IDS[0];
    let other=IDS[1];

    train(&mut evo,winner,1.0,20);
    train(&mut evo,other,0.20,20);
    train(&mut evo,IDS[2],0.0,20);
    train(&mut evo,IDS[3],0.0,20);

    let r=records(&evo);
    let win=r.iter().find(|x|x.candidate_id==winner).unwrap();
    let alt=r.iter().find(|x|x.candidate_id==other).unwrap();

    let mut lesion=0usize;
    let mut phase=0usize;
    let mut restore=0usize;
    let mut unrelated=0usize;

    for applicability in [1.0f32,0.92,0.84,0.76]{
        let p=[
            PhaseHypothesisProposal{candidate_id:winner,applicability},
            PhaseHypothesisProposal{candidate_id:other,applicability:1.0},
        ];
        assert_eq!(
            evo.choose_phase_native_hypothesis(&p).unwrap().candidate_id,
            winner
        );

        let mut broken=evo.clone();
        let saved=broken.perturb_phase_native_synapse_for_control(
            win.utility_synapse,0.0,0.0
        ).unwrap();
        lesion+=usize::from(
            broken.choose_phase_native_hypothesis(&p)
                .map(|d|d.candidate_id)!=Some(winner)
        );
        broken.restore_phase_native_synapse_for_control(
            win.utility_synapse,saved
        );
        restore+=usize::from(
            broken.choose_phase_native_hypothesis(&p)
                .map(|d|d.candidate_id)==Some(winner)
        );

        let mut shifted=evo.clone();
        shifted.perturb_phase_native_synapse_for_control(
            win.utility_synapse,1.0,std::f32::consts::PI
        ).unwrap();
        phase+=usize::from(
            shifted.choose_phase_native_hypothesis(&p)
                .map(|d|d.candidate_id)!=Some(winner)
        );

        let mut other_broken=evo.clone();
        other_broken.perturb_phase_native_synapse_for_control(
            alt.utility_synapse,0.0,0.0
        ).unwrap();
        unrelated+=usize::from(
            other_broken.choose_phase_native_hypothesis(&p)
                .map(|d|d.candidate_id)==Some(winner)
        );
    }

    println!(
        "U2_CAUSAL lesion={}/4 phase={}/4 restore={}/4 unrelated={}/4",
        lesion,phase,restore,unrelated
    );
    assert!(lesion>=3);
    assert!(phase>=3);
    assert_eq!(restore,4);
    assert!(unrelated>=3);
}

#[test]
fn u2_source_has_no_hypothesis_class_priority(){
    let source=include_str!("../src/phase_hypothesis_ecology.rs");
    for forbidden in [
        "world_id","task_id","ContextualRefinement","PerceptualRefinement",
        "CompositionalRefinement","RivalDiscrimination","GeneralEpistemic",
        "stale_candidate","useful_candidate","correct_candidate",
    ]{
        assert!(!source.contains(forbidden),
            "U2 source contains forbidden token {forbidden}");
    }
    for required in [
        "candidate_id","utility_synapse","conductance",
        "observe_phase_native_hypothesis_utility",
        "choose_phase_native_hypothesis",
    ]{
        assert!(source.contains(required),
            "U2 source missing physical dependency {required}");
    }
}
