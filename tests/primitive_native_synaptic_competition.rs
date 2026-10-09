#![allow(dead_code)]
// Separate open development. In this mode a finite population of REAL phase
// synapses is reinforced by each permitted factual motor outcome; inference
// reads the physical winner. There is no buffered permutation search after
// learning, no teacher binding and no rewrite of acquired operation definitions.
include!("primitive_argument_transfer.rs");

#[test]
fn physical_synaptic_competition_learns_argument_calls_from_factual_outcomes(){
    let mut evo=acquired();
    let original_roots=evo.phase_primitives().iter()
        .map(|p|p.output_cell).collect::<Vec<_>>();
    let definition_before=definitions(&evo,&original_roots);
    assert!(evo.set_phase_primitive_argument_transfer(true));
    assert!(evo.enable_phase_primitive_synaptic_argument_competition());
    let hypotheses=evo.phase_primitive_synaptic_argument_hypothesis_count();
    assert!(hypotheses>=12,"must physically represent rival argument pairs");
    let mut rt=ScientificRuntime::new(evo).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    for _ in 0..24{
        for bits in 0..8{
            world::teach(&mut rt,&input(bits,false),
                xor_of(bits,[1,2]),&world::roles(0),None);
        }
    }
    let bindings=rt.organism().phase_primitive_argument_bindings();
    println!("PHYSICAL_NATIVE_CANDIDATES count={} winners={:?}",hypotheses,bindings);
    assert_eq!(bindings.len(),2);
    for (action,root,args,_,_) in &bindings {
        assert!(original_roots.contains(root));
        let winner=rt.organism()
            .phase_primitive_synaptic_argument_winner(*action).unwrap();
        assert_eq!(winner.0,*root);
        assert_eq!(winner.1,*args);
        let mut sorted=args.clone();
        sorted.sort_unstable();
        assert_eq!(sorted,vec![1,2]);
        let physical=rt.organism().phase_native_synapse(winner.2).unwrap();
        assert_eq!(vec![physical.from,physical.to],*args);
        assert!(physical.weight>=0.83);
    }
    assert_eq!(definitions(rt.organism(),&original_roots),definition_before);
    let saved=rt.organism().online_checkpoint_bytes().unwrap();
    let mut e=EvoPhase::from_online_checkpoint(&saved).unwrap();
    assert_eq!(e.phase_primitive_argument_bindings(),bindings);
    let action=bindings[0].0;
    let winner=e.phase_primitive_synaptic_argument_winner(action).unwrap();
    let save_link=e.perturb_phase_native_synapse_for_control(winner.2,0.0,0.0)
        .expect("candidate weight resides in physical substrate");
    assert_ne!(e.phase_primitive_synaptic_argument_winner(action).unwrap().2,
        winner.2,"lesioned candidate cannot win via saved metadata");
    e.restore_phase_native_synapse_for_control(winner.2,save_link);
    assert_eq!(e.phase_primitive_synaptic_argument_winner(action),Some(winner));
    let phase_link=e.perturb_phase_native_synapse_for_control(
        winner.2,1.0,std::f32::consts::PI
    ).expect("phase-sensitive candidate");
    assert_ne!(e.phase_primitive_synaptic_argument_winner(action).unwrap().2,
        winner.2,"phase lesion must change synaptically selected hypothesis");
    e.restore_phase_native_synapse_for_control(winner.2,phase_link);
    assert_eq!(e.phase_primitive_synaptic_argument_winner(action),Some(winner));
    let frozen=ScientificRuntime::new(e).unwrap();
    let mut correct=0usize;
    for variant in 0..3{
        for bits in 0..8 {
            let q=changed_values(bits,variant).into_iter()
                .map(Some).collect::<Vec<_>>();
            let p=frozen.organism().phase_primitive_argument_prediction(&q)
                .expect("physical winner supported by original definition");
            correct+=usize::from(world::roles(0)[p.action]
                ==xor_of(bits,[1,2]));
        }
    }
    assert_eq!(correct,24);
    println!("PRIMITIVE_SYNAPTIC_COMPETITION correct=24/24 candidates={} checkpoint=true lesion=true phase_lesion=true no_retrospective_search=true original_definitions_unchanged=true",hypotheses);
}
