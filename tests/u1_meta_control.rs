use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::{
    PhaseCognitiveProposal,PhaseMetaControlConfig,PhaseNativeConfig,
};

const ORACLE:[f32;5]=[0.36,0.30,0.22,0.08,0.04];

fn carrier()->EvoPhase{
    let mut evo=EvoPhase::new(EvoConfig{
        sensory_cells:8,motor_cells:6,dormant_cells:96,hdc_dim:128,
        phase_learning_rate:1.0,
        ..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    evo
}

fn utility(fields:[f32;5])->f32{
    let mass=fields.into_iter().sum::<f32>().max(1.0e-8);
    fields.into_iter().zip(ORACLE).map(|(x,w)|x*w).sum::<f32>()/mass
}

fn train_meta()->EvoPhase{
    let mut evo=carrier();
    assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig{
        learning_rate:0.35,
        learning_enabled:true,
        readout_enabled:true,
        phase_learning_enabled:true,
    }));
    for _ in 0..96usize{
        for i in 0..5usize{
            let mut fields=[0.0f32;5];
            fields[i]=1.0;
            assert!(evo.observe_phase_native_meta_utility(fields,ORACLE[i]));
        }
    }
    let weights=evo.phase_native_meta_weights().unwrap();
    println!("U1_SOURCE weights={:?} obs={}",weights,evo.phase_native_meta_observations());
    assert!(weights.iter().all(|w|*w>0.01));
    evo
}

fn templates()->[[f32;5];3]{
    [
        [0.45,0.10,0.25,0.10,0.10], // strongest under learned utility
        [0.10,0.60,0.10,0.10,0.10],
        [0.20,0.15,0.45,0.10,0.10],
    ]
}

fn cases()->Vec<([PhaseCognitiveProposal;3],usize)>{
    let t=templates();
    let mut out=Vec::new();
    for rep in 0..4usize{
        for winner in 0..3usize{
            let mut p=[
                PhaseCognitiveProposal{proposal_id:10,action:0,fields:t[1]},
                PhaseCognitiveProposal{proposal_id:20,action:1,fields:t[1]},
                PhaseCognitiveProposal{proposal_id:30,action:2,fields:t[1]},
            ];
            p[winner].fields=t[0];
            p[(winner+1)%3].fields=t[2];
            p[(winner+2)%3].fields=t[1];
            // Change opaque IDs/actions across repeats without changing class role.
            for i in 0..3usize{
                p[i].proposal_id += (rep as u64)*100;
                p[i].action=(i+rep)%6;
            }
            out.push((p,winner));
        }
    }
    out
}

#[test]
fn u1_learned_physical_meta_control_selects_across_opaque_proposals(){
    let source=train_meta();
    let checkpoint=source.phase_native_meta_checkpoint().unwrap();

    let mut evo=carrier();
    assert!(evo.restore_phase_native_meta_checkpoint(checkpoint));
    evo.set_phase_native_meta_learning_enabled(false);

    let mut full=0usize;
    let mut fixed_priority=0usize;
    let mut id_only=0usize;
    let mut class_wins=[0usize;3];

    for (proposals,winner) in cases(){
        let expected=proposals[winner];
        let direct=evo.choose_phase_native_meta_proposal(&proposals)
            .expect("unique learned meta winner");
        full+=usize::from(direct.proposal_id==expected.proposal_id);
        if direct.proposal_id==expected.proposal_id{class_wins[winner]+=1;}

        let mut reversed=proposals;
        reversed.reverse();
        let rev=evo.choose_phase_native_meta_proposal(&reversed)
            .expect("order invariant winner");
        assert_eq!(rev.proposal_id,direct.proposal_id);
        assert_eq!(rev.action,direct.action);

        fixed_priority+=usize::from(proposals[0].proposal_id==expected.proposal_id);
        let min_id=proposals.iter().min_by_key(|p|p.proposal_id).unwrap();
        id_only+=usize::from(min_id.proposal_id==expected.proposal_id);

        // The winner also agrees with the factual utility oracle used only by
        // this evaluator, not by production competition.
        let oracle=proposals.iter()
            .max_by(|a,b|utility(a.fields).partial_cmp(&utility(b.fields)).unwrap())
            .unwrap();
        assert_eq!(oracle.proposal_id,expected.proposal_id);
    }

    let mut zero=carrier();
    assert!(zero.enable_phase_native_meta_control(PhaseMetaControlConfig::default()));
    let zero_score=cases().into_iter().filter(|(p,w)|
        zero.choose_phase_native_meta_proposal(p)
            .map(|d|d.proposal_id)==Some(p[*w].proposal_id)
    ).count();

    println!(
        "U1_RESULT full={}/12 fixed={}/12 id_only={}/12 zero={}/12 class_wins={:?}",
        full,fixed_priority,id_only,zero_score,class_wins
    );
    assert_eq!(full,12);
    assert!(fixed_priority<=6);
    assert!(id_only<=6);
    assert!(zero_score<=4);
    assert!(class_wins.iter().all(|x|*x>=2));
}

#[test]
fn u1_meta_winner_depends_causally_on_physical_synapses(){
    let source=train_meta();
    let checkpoint=source.phase_native_meta_checkpoint().unwrap();
    let mut evo=carrier();
    assert!(evo.restore_phase_native_meta_checkpoint(checkpoint));
    evo.set_phase_native_meta_learning_enabled(false);

    let syn=evo.phase_native_meta_synapses().unwrap();
    let goal_link=syn[0];
    let economy_link=syn[4];
    let mut lesion_lost=0usize;
    let mut phase_lost=0usize;
    let mut restored=0usize;
    let mut unrelated=0usize;

    for winner in 0..3usize{
        // Four competitions total are enough; repeat class0 twice.
        let repeats=if winner==0{2}else{1};
        for rep in 0..repeats{
            let t=templates();
            let mut p=[
                PhaseCognitiveProposal{proposal_id:100+winner as u64*10+rep as u64,action:0,fields:t[1]},
                PhaseCognitiveProposal{proposal_id:200+winner as u64*10+rep as u64,action:1,fields:t[1]},
                PhaseCognitiveProposal{proposal_id:300+winner as u64*10+rep as u64,action:2,fields:t[1]},
            ];
            p[winner].fields=t[0];
            p[(winner+1)%3].fields=t[2];
            let original=evo.choose_phase_native_meta_proposal(&p).unwrap();
            assert_eq!(original.proposal_id,p[winner].proposal_id);

            let mut broken=evo.clone();
            let saved=broken.perturb_phase_native_synapse_for_control(goal_link,0.0,0.0).unwrap();
            lesion_lost+=usize::from(
                broken.choose_phase_native_meta_proposal(&p)
                    .map(|d|d.proposal_id)!=Some(original.proposal_id)
            );
            broken.restore_phase_native_synapse_for_control(goal_link,saved.clone());
            restored+=usize::from(
                broken.choose_phase_native_meta_proposal(&p)
                    .map(|d|d.proposal_id)==Some(original.proposal_id)
            );

            let mut shifted=evo.clone();
            shifted.perturb_phase_native_synapse_for_control(
                goal_link,1.0,std::f32::consts::PI
            ).unwrap();
            phase_lost+=usize::from(
                shifted.choose_phase_native_meta_proposal(&p)
                    .map(|d|d.proposal_id)!=Some(original.proposal_id)
            );

            let mut other=evo.clone();
            other.perturb_phase_native_synapse_for_control(economy_link,0.0,0.0).unwrap();
            unrelated+=usize::from(
                other.choose_phase_native_meta_proposal(&p)
                    .map(|d|d.proposal_id)==Some(original.proposal_id)
            );
        }
    }

    println!(
        "U1_CAUSAL lesion_lost={}/4 phase_lost={}/4 restore={}/4 unrelated={}/4",
        lesion_lost,phase_lost,restored,unrelated
    );
    assert!(lesion_lost>=3);
    assert!(phase_lost>=3);
    assert_eq!(restored,4);
    assert!(unrelated>=3);
}

#[test]
fn u1_meta_competition_source_has_no_module_priority(){
    let source=include_str!("../src/phase_meta_control.rs");
    for forbidden in [
        "ContextualRefinement","PerceptualRefinement","CompositionalRefinement",
        "RivalDiscrimination","GeneralEpistemic","GoalDirectedAction",
        "world_id","task_id","correct_operation","HistoryWorld",
    ]{
        assert!(!source.contains(forbidden),"forbidden U1 source token {forbidden}");
    }
    for required in [
        "PhaseCognitiveProposal","conductance","weight_synapses",
        "observe_phase_native_meta_utility","choose_phase_native_meta_proposal",
    ]{
        assert!(source.contains(required),"missing U1 physical dependency {required}");
    }
}


#[test]
fn u1_positive_evidence_does_not_dilute_factual_utility_in_online_mode() {
    let mut evo=train_meta();
    let base=[1.0,0.0,0.0,0.0,0.0];
    let supported=[1.0,0.5,0.0,0.0,0.0];
    let legacy_base=evo.phase_native_meta_score(base).unwrap();
    let legacy_supported=evo.phase_native_meta_score(supported).unwrap();
    assert!(legacy_supported<legacy_base,
        "reproduce the established positive-evidence dilution");
    assert!(evo.set_phase_native_meta_monotone_evidence(true));
    let first=evo.phase_native_meta_score(base).unwrap();
    let second=evo.phase_native_meta_score(supported).unwrap();
    assert!(second>first+1.0e-4,
        "extra physically supported epistemic evidence must not lower value");
    let candidates=[
        PhaseCognitiveProposal{proposal_id:14,action:0,fields:base},
        PhaseCognitiveProposal{proposal_id:39,action:1,fields:supported},
    ];
    assert_eq!(evo.choose_phase_native_meta_proposal(&candidates).unwrap().action,1);
    let mut reversed=candidates;
    reversed.reverse();
    assert_eq!(evo.choose_phase_native_meta_proposal(&reversed).unwrap().action,1);
    let syn=evo.phase_native_meta_synapses().unwrap()[1];
    let original=evo.perturb_phase_native_synapse_for_control(syn,0.0,0.0).unwrap();
    assert_eq!(evo.phase_native_meta_score(base).unwrap(),
        evo.phase_native_meta_score(supported).unwrap(),
        "the added evidence is useless when its physical synapse is lesioned");
    assert!(evo.choose_phase_native_meta_proposal(&candidates).is_none(),
        "ties must fail closed, not invent a motor priority");
    evo.restore_phase_native_synapse_for_control(syn,original);
    assert_eq!(evo.choose_phase_native_meta_proposal(&candidates).unwrap().action,1);

    let checkpoint=evo.phase_native_meta_checkpoint().unwrap();
    let mut resumed=carrier();
    assert!(resumed.restore_phase_native_meta_checkpoint(checkpoint));
    assert!(resumed.phase_native_meta_monotone_evidence());
    assert_eq!(resumed.phase_native_meta_score(supported),
        evo.phase_native_meta_score(supported));
    assert!(resumed.set_phase_native_meta_monotone_evidence(false));
    assert_eq!(resumed.phase_native_meta_score(supported),Some(legacy_supported));
    println!("U1_MONOTONE positive_evidence=true physical_lesion=true restore=true legacy_unchanged=true");
}
