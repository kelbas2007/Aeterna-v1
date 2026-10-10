use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::PhaseNativeConfig;
const W:usize=7;
const DIM:usize=W*W*12;
fn set(raw:&mut[f32],tile:usize,object:u8,color:u8){
    for (ch,v) in [object,color,0].iter().copied().enumerate(){
        for bit in 0..4{
            raw[tile*12+ch*4+bit]=((v>>bit)&1) as f32;
        }
    }
}
fn room(objects:&[(usize,u8)])->Vec<f32>{
    let mut raw=vec![0.0;DIM];
    for i in 0..49{set(&mut raw,i,2,1);}
    set(&mut raw,27,11,0); // body tile: not an object to bind
    for &(tile,code) in objects {set(&mut raw,tile,code,1);}
    raw
}
fn newborn()->EvoPhase{
    let mut brain=EvoPhase::new(EvoConfig{
        sensory_cells:DIM,motor_cells:7,
        dormant_cells:64,hdc_dim:64,..EvoConfig::default()
    });
    brain.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(brain.enable_phase_native_general_policy());
    assert!(brain.enable_phase_native_developmental_memory());
    assert!(brain.enable_phase_native_relational_workspace(7,7,3,4,2));
    brain
}
fn read_first(cue:u8,present:&[f32])->Vec<f32>{
    let mut o=newborn();
    let start=room(&[(8,cue)]);
    assert!(o.phase_native_relational_initial(&start));
    assert!(o.phase_native_relational_held_subject());
    // A previously seen subject is out of sight for many real frames.
    for _ in 0..10 {
        assert!(o.phase_native_relational_factual_post(&room(&[])));
    }
    o.phase_native_relational_readout(present).unwrap()
}
#[test]
fn distinct_past_cues_produce_different_present_relations_after_occlusion(){
    let currently_visible=room(&[(11,5),(16,6)]);
    let history_a=read_first(5,&currently_visible);
    let history_b=read_first(6,&currently_visible);
    assert_ne!(history_a,history_b);
    assert!(history_a[1]>0.0 && history_b[1]>0.0);
    assert!(history_a[2]>0.0 && history_b[2]>0.0);
    // The RELEVANT difference is which relative visible cell corresponds
    // to the past object. Both current frames are byte-for-byte identical.
    assert_ne!(&history_a[4..],&history_b[4..]);
    let unseen=read_first(5,&room(&[]));
    assert_eq!(unseen[1],0.0);
    assert!(unseen[3]>0.0,"absence of a matching visible subject is
        explicitly represented as unknown/occluded, not invented object");
    println!("RELATIONAL_STATE_PASS same_present=true two_history_relations=true ten_occluded_frames=true no_labels=true");
}
#[test]
fn physical_lesion_disables_relational_knowledge_not_motor_safety(){
    let mut o=newborn();
    let seen=room(&[(8,5)]);
    let compare=room(&[(11,5),(16,6)]);
    assert!(o.phase_native_relational_initial(&seen));
    let before=o.phase_native_relational_readout(&compare).unwrap();
    assert!(before[0]>0.0 && before[1]>0.0);
    let index=o.phase_native_relational_link().unwrap();
    let saved=o.perturb_phase_native_synapse_for_control(
        index,0.0,0.0).unwrap();
    let dead=o.phase_native_relational_readout(&compare).unwrap();
    assert!(dead.iter().all(|&v|v==0.0));
    o.restore_phase_native_synapse_for_control(index,saved);
    assert_eq!(o.phase_native_relational_readout(&compare).unwrap(),before);
    let checkpoint=o.phase_native_checkpoint().unwrap();
    let mut restored=EvoPhase::new(o.config().clone());
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    assert!(!restored.phase_native_relational_held_subject(),
        "new lifetime reset cannot invent an unobserved cue");
    assert_eq!(restored.phase_native_relational_readout(&compare).unwrap(),
        vec![0.0;96]);
    println!("RELATIONAL_PHYSICAL_PASS lesion=true restore=true no_fake_cue_after_restart=true");
}

#[test]
fn value_learning_can_acquire_an_object_first_seen_after_episode_start(){
    let current=room(&[(11,5),(16,6)]);
    let mut a=newborn();
    let mut b=newborn();
    assert!(a.enable_phase_native_context_value_learning());
    assert!(b.enable_phase_native_context_value_learning());
    for subject in [&mut a,&mut b] {
        assert!(subject.phase_native_relational_initial(&room(&[])));
        assert!(!subject.phase_native_relational_held_subject());
    }
    a.phase_native_relational_factual_post(&room(&[(8,5)]));
    b.phase_native_relational_factual_post(&room(&[(8,6)]));
    for _ in 0..20 {
        a.phase_native_relational_factual_post(&room(&[]));
        b.phase_native_relational_factual_post(&room(&[]));
    }
    assert!(a.phase_native_relational_held_subject());
    assert!(b.phase_native_relational_held_subject());
    assert_ne!(a.phase_native_relational_readout(&current),b.phase_native_relational_readout(&current));
    // The historical first-view-only behavior remains available unchanged.
    let mut legacy=newborn();
    legacy.phase_native_relational_initial(&room(&[]));
    legacy.phase_native_relational_factual_post(&room(&[(8,5)]));
    assert!(!legacy.phase_native_relational_held_subject());
}

#[test]
fn value_context_preserves_observed_multiplicity_and_obeys_relation_lesion(){
    let mut a=newborn();
    let mut b=newborn();
    assert!(a.enable_phase_native_context_value_learning());
    assert!(b.enable_phase_native_context_value_learning());
    let first_a=room(&[(8,5),(11,5),(16,6)]);
    let first_b=room(&[(8,5),(11,6),(16,6)]);
    a.phase_native_relational_initial(&first_a);
    b.phase_native_relational_initial(&first_b);
    let present=room(&[(11,5),(16,6)]);
    // The prior SET representation erases the different occurrence counts.
    assert_eq!(a.phase_native_relational_readout(&present),b.phase_native_relational_readout(&present));
    assert_ne!(a.phase_native_value_context_key(&present),b.phase_native_value_context_key(&present));
    for subject in [&mut a,&mut b] {
        let link=subject.phase_native_relational_link().unwrap();
        subject.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
    }
    assert_eq!(a.phase_native_value_context_key(&present),b.phase_native_value_context_key(&present));
}

#[test]
fn ambiguity_is_retained_as_hypotheses_not_discarded_or_given_a_correct_label(){
    let current=room(&[(16,5),(19,6)]);
    let first_a=room(&[(8,5),(11,7)]);
    let first_b=room(&[(8,6),(11,7)]);
    let mut a=newborn();
    let mut b=newborn();
    assert!(a.phase_native_relational_initial(&first_a));
    assert!(b.phase_native_relational_initial(&first_b));
    assert!(a.phase_native_relational_held_subject());
    assert!(b.phase_native_relational_held_subject());
    for _ in 0..8{
        assert!(a.phase_native_relational_factual_post(&room(&[])));
        assert!(b.phase_native_relational_factual_post(&room(&[])));
    }
    let av=a.phase_native_relational_readout(&current).unwrap();
    let bv=b.phase_native_relational_readout(&current).unwrap();
    assert_ne!(av,bv,"two distinct past observations MUST create
        distinct relational evidence at the SAME current input");
    assert!(av[1]>0.0 && bv[1]>0.0);
    println!("RELATIONAL_HYPOTHESES_PASS ambiguous_first_view=true distinct_cue_histories=true missing_frames=8 no_target_labels=true");
}

#[test]
fn relation_stays_invariant_under_unseen_symbol_code_permutation(){
    // Two independently coded environments: no learned MiniGrid ID table,
    // only equal/different relational structure and spatial location.
    let mut a=newborn();
    let mut b=newborn();
    assert!(a.phase_native_relational_initial(&room(&[(8,5)])));
    assert!(b.phase_native_relational_initial(&room(&[(8,9)])));
    for _ in 0..4{
        assert!(a.phase_native_relational_factual_post(&room(&[])));
        assert!(b.phase_native_relational_factual_post(&room(&[])));
    }
    let first=a.phase_native_relational_readout(
        &room(&[(16,5),(19,6)])).unwrap();
    let transformed=b.phase_native_relational_readout(
        &room(&[(16,9),(19,10)])).unwrap();
    assert_eq!(first,transformed,
        "same relation must not depend on absolute source category IDs");
    let changed=b.phase_native_relational_readout(
        &room(&[(16,10),(19,9)])).unwrap();
    assert_ne!(first,changed,
        "reversal of matching candidate must modify relation");
    println!("RELATIONAL_PERMUTATION_PASS unseen_codes=true same_structure=true different_candidate=true");
}
