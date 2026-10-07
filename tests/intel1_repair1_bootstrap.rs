use aeterna_v1::scientific_runtime::{ReasoningMode, ScientificRuntime};

#[allow(dead_code)]
mod fixture {
    include!("g19_rival_hypothesis_discrimination.rs");
    pub fn base()->(EvoPhase,[[usize;2];8]){
        let drive=train_drive();
        let mut evo=target(&drive);
        let l1=train_abstraction(&mut evo);
        assert_eq!(evo.phase_native_circuits().len(),0);
        (evo,l1)
    }
    pub fn scene(l1:&[[usize;2];8],s:usize)->Vec<f32>{
        state_scene(l1,s,LAYOUTS[0])
    }
    pub fn teach(evo:&mut EvoPhase,l1:&[[usize;2];8],pre:usize,a:usize,post:usize){
        learn(evo,l1,pre,a,post);
    }
}

fn enable_all(rt:&mut ScientificRuntime){
    assert!(rt.enable_context_refinement());
    assert!(rt.enable_perceptual_refinement());
    assert!(rt.enable_compositional_refinement());
}

#[test]
fn unknown_goal_bootstraps_with_existing_general_epistemic_drive(){
    let (evo,l1)=fixture::base();
    let mut rt=ScientificRuntime::new(evo).unwrap();
    enable_all(&mut rt);
    rt.observe_external(&fixture::scene(&l1,0)).unwrap();
    rt.set_goal(&fixture::scene(&l1,7)).unwrap();

    let proposal=rt.propose().unwrap().expect("general epistemic bootstrap");
    assert_eq!(proposal.mode,ReasoningMode::GeneralEpistemic);
}

#[test]
fn known_goal_relevance_keeps_goal_conditioned_priority(){
    let (mut evo,l1)=fixture::base();
    fixture::teach(&mut evo,&l1,0,2,1);
    fixture::teach(&mut evo,&l1,1,3,7);
    let mut rt=ScientificRuntime::new(evo).unwrap();
    enable_all(&mut rt);
    rt.observe_external(&fixture::scene(&l1,0)).unwrap();
    rt.set_goal(&fixture::scene(&l1,7)).unwrap();

    let proposal=rt.propose().unwrap().expect("goal action");
    assert_ne!(proposal.mode,ReasoningMode::GeneralEpistemic);
}
