use aeterna_v1::{EvoPhase};
use aeterna_v1::carrier::{
    PhaseCognitiveProposal,PhaseEcologicalCognitiveProposal,
    PhaseMetaControlConfig,PhaseHypothesisEcologyConfig,
};

#[allow(dead_code)]
mod mixed {
    include!("g22_perceptual_variable_invention.rs");

    pub fn build()->(EvoPhase,Vec<f32>,Vec<f32>){
        let drive=fixture::drive();
        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        train_useful(&mut evo,&l1,false);

        assert!(evo.enable_phase_native_context_refinement());
        let good=fixture::scene(&l1,7,0,None);
        let dead=fixture::scene(&l1,8,0,None);
        let base_plain=fixture::scene(&l1,0,0,None);

        // History collision on action4, independent of the current raw marker.
        evo.clear_phase_native_context_history();
        evo.observe_initial_real(&fixture::scene(&l1,1,0,None),false);
        assert!(evo.observe_phase_native_context_result(
            5,&base_plain
        ).is_some());
        assert!(evo.observe_phase_native_context_result(
            4,&good
        ).is_some());

        evo.clear_phase_native_context_history();
        evo.observe_initial_real(&fixture::scene(&l1,2,1,None),false);
        assert!(evo.observe_phase_native_context_result(
            5,&fixture::scene(&l1,0,1,None)
        ).is_some());
        assert!(evo.observe_phase_native_context_result(
            4,&fixture::scene(&l1,8,1,None)
        ).is_some());

        assert!(evo.phase_native_context_witnesses()
            .iter().any(|w|!w.promoted&&!w.retired));

        // Current factual state carries BOTH history and a promoted G22 raw cue.
        let current=fixture::scene(&l1,0,4,Some((MARKERS[2],A_VALUE)));
        let goal=fixture::scene(&l1,7,5,None);
        evo.clear_phase_native_context_history();
        evo.observe_initial_real(&fixture::scene(&l1,1,4,None),false);
        assert!(evo.observe_phase_native_context_result(
            5,&current
        ).is_some());

        let mut c=evo.clone();
        assert!(c.phase_native_context_action(&goal).1.is_some());
        let mut p=evo.clone();
        assert!(p.phase_native_perceptual_action(&goal).1.is_some());
        let mut r=evo.clone();
        assert!(r.choose_phase_native_goal_rival_probe(&goal).is_some());

        (evo,current,goal)
    }

    pub fn context_fields(evo:&mut EvoPhase,goal:&[f32])->(usize,[f32;5]){
        let action=evo.phase_native_context_action(goal).1.unwrap();
        let w=evo.phase_native_context_witnesses().into_iter()
            .find(|w|!w.promoted&&!w.retired)
            .expect("context residual");
        let evidence=(w.eligible_observations as f32/32.0).clamp(0.0,1.0);
        // Generic confidence/economy transport; ecology will modulate confidence.
        (action,[0.0,0.0,0.0,(0.10+0.90*evidence).clamp(0.0,1.0),0.20])
    }

    pub fn percept_fields(evo:&mut EvoPhase,goal:&[f32])->(usize,[f32;5]){
        let action=evo.phase_native_perceptual_action(goal).1.unwrap();
        let w=evo.phase_native_perceptual_witnesses().into_iter()
            .find(|w|w.promoted&&!w.retired)
            .expect("promoted current-sensory residual");
        let evidence=(w.eligible_observations as f32/32.0).clamp(0.0,1.0);
        (action,[0.0,0.0,0.0,evidence,0.20])
    }

    pub fn rival_fields(
        evo:&mut EvoPhase,current:&[f32],goal:&[f32]
    )->(usize,[f32;5]){
        let action=evo.choose_phase_native_goal_rival_probe(goal).unwrap();
        let disagreement=evo.phase_native_goal_rival_disagreement_for_control(
            current,goal,action
        ).unwrap_or(0.0).clamp(0.0,1.0);
        assert!(disagreement>0.0);
        (action,[0.0,0.0,0.0,disagreement,0.20])
    }
}

const C:u64=0xC011;
const P:u64=0xA22E;
const L:u64=0xBEEF;

fn train_meta(evo:&mut EvoPhase){
    assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig{
        learning_rate:0.35,
        learning_enabled:true,
        readout_enabled:true,
        phase_learning_enabled:true,
    }));
    // Generic learned currency: evidence/confidence is valuable; equal
    // computation economy has small positive weight.
    for _ in 0..32{
        assert!(evo.observe_phase_native_meta_utility(
            [0.0,0.0,0.0,1.0,0.0],1.0
        ));
        assert!(evo.observe_phase_native_meta_utility(
            [0.0,0.0,0.0,0.0,1.0],0.05
        ));
    }
    evo.set_phase_native_meta_learning_enabled(false);
    let w=evo.phase_native_meta_weights().unwrap();
    assert!(w[3]>0.95&&w[4]<0.10);
}

fn register_ecology(evo:&mut EvoPhase){
    assert!(evo.enable_phase_native_hypothesis_ecology(
        PhaseHypothesisEcologyConfig{
            learning_rate:0.35,
            dormancy_threshold:0.05,
            learning_enabled:true,
            phase_learning_enabled:true,
        }
    ));
    for id in [C,P,L]{assert!(evo.register_phase_native_hypothesis(id));}
}

fn proposals(
    evo:&EvoPhase,current:&[f32],goal:&[f32]
)->[PhaseEcologicalCognitiveProposal;3]{
    let mut c=evo.clone();
    let (ca,cf)=mixed::context_fields(&mut c,goal);
    let mut p=evo.clone();
    let (pa,pf)=mixed::percept_fields(&mut p,goal);
    let mut l=evo.clone();
    let (la,lf)=mixed::rival_fields(&mut l,current,goal);
    [
        PhaseEcologicalCognitiveProposal{
            candidate_id:C,applicability:1.0,
            proposal:PhaseCognitiveProposal{proposal_id:101,action:ca,fields:cf},
        },
        PhaseEcologicalCognitiveProposal{
            candidate_id:P,applicability:1.0,
            proposal:PhaseCognitiveProposal{proposal_id:202,action:pa,fields:pf},
        },
        PhaseEcologicalCognitiveProposal{
            candidate_id:L,applicability:1.0,
            proposal:PhaseCognitiveProposal{proposal_id:303,action:la,fields:lf},
        },
    ]
}

fn role_for(proposals:&[PhaseEcologicalCognitiveProposal;3],proposal_id:u64)->u64{
    proposals.iter().find(|p|p.proposal.proposal_id==proposal_id)
        .unwrap().candidate_id
}

fn choose(
    evo:&EvoPhase,current:&[f32],goal:&[f32],
)->Option<u64>{
    let ps=proposals(evo,current,goal);
    evo.choose_phase_native_ecological_meta_proposal(&ps)
        .map(|d|role_for(&ps,d.proposal_id))
}

fn update(evo:&mut EvoPhase,values:[f32;3],n:usize){
    for _ in 0..n{
        for (id,value) in [C,P,L].into_iter().zip(values){
            assert!(evo.observe_phase_native_hypothesis_utility(id,value));
        }
    }
}

#[test]
fn u3_real_context_perceptual_and_rival_structures_switch_cognitive_authority(){
    let (mut evo,current,goal)=mixed::build();
    train_meta(&mut evo);
    register_ecology(&mut evo);
    let meta_before=evo.phase_native_meta_weights().unwrap();
    let addresses=evo.phase_native_hypothesis_records();

    let mut sequence=Vec::new();
    let mut fixed_priority=0usize;

    // H: factual outcomes favor history/context explanation.
    update(&mut evo,[1.0,0.0,0.0],12);
    let ps_h=proposals(&evo,&current,&goal);
    assert_eq!(choose(&evo,&current,&goal),Some(C));
    sequence.push(C);
    fixed_priority+=usize::from(ps_h[0].candidate_id==C);

    // P: no reset or world-mode API; usefulness shifts to raw-current cue.
    update(&mut evo,[0.0,1.0,0.0],20);
    assert_eq!(choose(&evo,&current,&goal),Some(P));
    sequence.push(P);
    fixed_priority+=usize::from(ps_h[0].candidate_id==P);

    // L: transition-law rival is now the useful explanation.
    update(&mut evo,[0.0,0.0,1.0],20);
    assert_eq!(choose(&evo,&current,&goal),Some(L));
    sequence.push(L);
    fixed_priority+=usize::from(ps_h[0].candidate_id==L);

    // Return to H: original context candidate must reactivate in place.
    update(&mut evo,[1.0,0.0,0.0],20);
    assert_eq!(choose(&evo,&current,&goal),Some(C));
    sequence.push(C);
    fixed_priority+=usize::from(ps_h[0].candidate_id==C);

    assert_eq!(sequence,[C,P,L,C]);
    assert!(fixed_priority<=2);
    assert_eq!(evo.phase_native_meta_weights().unwrap(),meta_before);

    let after=evo.phase_native_hypothesis_records();
    for before in addresses{
        let now=after.iter().find(|r|r.candidate_id==before.candidate_id).unwrap();
        assert_eq!(now.candidate_cell,before.candidate_cell);
        assert_eq!(now.utility_synapse,before.utility_synapse);
    }

    // Enumeration order must not change role winner.
    let mut reversed=proposals(&evo,&current,&goal);
    reversed.reverse();
    let direct=evo.choose_phase_native_ecological_meta_proposal(&reversed).unwrap();
    assert_eq!(role_for(&reversed,direct.proposal_id),C);

    // Proposal IDs are opaque transport only.
    let mut rekeyed=proposals(&evo,&current,&goal);
    rekeyed[0].proposal.proposal_id=0x9911;
    rekeyed[1].proposal.proposal_id=0x2288;
    rekeyed[2].proposal.proposal_id=0x7717;
    let decision=evo.choose_phase_native_ecological_meta_proposal(&rekeyed).unwrap();
    assert_eq!(role_for(&rekeyed,decision.proposal_id),C);

    println!(
        "U3_SEQUENCE roles={:?} records={:?} meta={:?}",
        sequence,after,meta_before
    );
}

#[test]
fn u3_both_meta_and_ecology_physics_are_causally_required(){
    let (mut evo,current,goal)=mixed::build();
    train_meta(&mut evo);
    register_ecology(&mut evo);
    update(&mut evo,[1.0,0.0,0.0],16);
    assert_eq!(choose(&evo,&current,&goal),Some(C));

    let meta_conf=evo.phase_native_meta_synapses().unwrap()[3];
    let records=evo.phase_native_hypothesis_records();
    let c_link=records.iter().find(|r|r.candidate_id==C).unwrap().utility_synapse;
    let p_link=records.iter().find(|r|r.candidate_id==P).unwrap().utility_synapse;

    let mut meta_lesion=evo.clone();
    let meta_saved=meta_lesion.perturb_phase_native_synapse_for_control(
        meta_conf,0.0,0.0
    ).unwrap();
    assert_ne!(choose(&meta_lesion,&current,&goal),Some(C));
    meta_lesion.restore_phase_native_synapse_for_control(meta_conf,meta_saved);
    assert_eq!(choose(&meta_lesion,&current,&goal),Some(C));

    let mut meta_phase=evo.clone();
    meta_phase.perturb_phase_native_synapse_for_control(
        meta_conf,1.0,std::f32::consts::PI
    ).unwrap();
    assert_ne!(choose(&meta_phase,&current,&goal),Some(C));

    let mut eco_lesion=evo.clone();
    let eco_saved=eco_lesion.perturb_phase_native_synapse_for_control(
        c_link,0.0,0.0
    ).unwrap();
    assert_ne!(choose(&eco_lesion,&current,&goal),Some(C));
    eco_lesion.restore_phase_native_synapse_for_control(c_link,eco_saved);
    assert_eq!(choose(&eco_lesion,&current,&goal),Some(C));

    let mut eco_phase=evo.clone();
    eco_phase.perturb_phase_native_synapse_for_control(
        c_link,1.0,std::f32::consts::PI
    ).unwrap();
    assert_ne!(choose(&eco_phase,&current,&goal),Some(C));

    let mut unrelated=evo.clone();
    unrelated.perturb_phase_native_synapse_for_control(
        p_link,0.0,0.0
    ).unwrap();
    assert_eq!(choose(&unrelated,&current,&goal),Some(C));

    // Zero all U1 meta synapses -> no unique useful competition.
    let mut zero_meta=evo.clone();
    for link in zero_meta.phase_native_meta_synapses().unwrap(){
        zero_meta.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
    }
    assert!(zero_meta.choose_phase_native_ecological_meta_proposal(
        &proposals(&zero_meta,&current,&goal)
    ).is_none());

    // Zero all U2 ecology authority -> all proposals dormant.
    let mut zero_eco=evo.clone();
    for r in zero_eco.phase_native_hypothesis_records(){
        zero_eco.perturb_phase_native_synapse_for_control(
            r.utility_synapse,0.0,0.0
        ).unwrap();
    }
    assert!(zero_eco.choose_phase_native_ecological_meta_proposal(
        &proposals(&zero_eco,&current,&goal)
    ).is_none());
}

#[test]
fn u3_unified_scorer_contains_no_explanation_class_priority(){
    let source=include_str!("../src/phase_hypothesis_ecology.rs");
    let start=source.find("pub fn choose_phase_native_ecological_meta_proposal")
        .unwrap();
    let scorer=&source[start..];
    for forbidden in [
        "ContextualRefinement","PerceptualRefinement","CompositionalRefinement",
        "RivalDiscrimination","GeneralEpistemic",
        "regime_h","regime_p","regime_l",
        "world_id","task_id","correct_explanation",
    ]{
        assert!(!scorer.contains(forbidden),
            "U3 scorer contains authored class token {forbidden}");
    }
    for required in [
        "phase_hypothesis_authority_with_state",
        "proposal.fields[3]",
        "choose_phase_native_meta_proposal",
    ]{
        assert!(scorer.contains(required),"missing U3 dependency {required}");
    }
}
