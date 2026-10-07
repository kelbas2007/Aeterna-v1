use aeterna_v1::{EvoPhase};
use aeterna_v1::carrier::{PhasePerceptFeature};

#[allow(dead_code)]
mod fixture {
    include!("g19_rival_hypothesis_discrimination.rs");

    pub fn prepare() -> (PhaseDriveCheckpoint, [[usize;2];8]) {
        let drive=train_drive();
        let mut evo=target(&drive);
        let l1=train_abstraction(&mut evo);
        (drive,l1)
    }

    pub fn organism(drive:&PhaseDriveCheckpoint)->EvoPhase {
        target(drive)
    }

    pub fn train_rep(evo:&mut EvoPhase)->[[usize;2];8] {
        train_abstraction(evo)
    }

    pub fn scene(
        l1:&[[usize;2];8],
        state:usize,
        layout_index:usize,
        weak:Option<(usize,f32)>,
    )->Vec<f32>{
        let mut s=state_scene(l1,state,LAYOUTS[layout_index%LAYOUTS.len()]);
        if let Some((index,value))=weak {
            assert!(index<s.len());
            assert!(s[index]<0.5,"weak marker position must not overwrite inherited motif");
            s[index]=value;
        }
        s
    }

    pub fn action_pair(swap:bool)->(usize,usize){
        let r=roles(swap);
        (r.probe_a,r.probe_b)
    }

    pub fn cfg(evo:&EvoPhase)->EvoConfig { evo.config().clone() }
}

const A_VALUE:f32=0.20; // WeakAmplitudeBin(3)
const B_VALUE:f32=0.40; // WeakAmplitudeBin(6)
const NULL_C_VALUE:f32=0.10;
const NULL_D_VALUE:f32=0.30;

const MARKERS:[usize;4]=[399,398,379,378];

fn feature_for(value:f32)->PhasePerceptFeature{
    PhasePerceptFeature::WeakAmplitudeBin(
        ((value*16.0).floor() as u8).min(7)
    )
}

fn fact(
    evo:&mut EvoPhase,
    pre:&[f32],
    action:usize,
    post:&[f32],
){
    evo.observe_initial_real(pre,false);
    assert!(evo.observe_phase_native_perceptual_result(action,post).is_some());
}

fn train_useful(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
){
    let (x,y)=fixture::action_pair(swap);
    assert!(evo.enable_phase_native_perceptual_refinement());
    assert!(evo.phase_native_perceptual_witnesses().is_empty());

    let base_plain=fixture::scene(l1,0,0,None);
    let a0=fixture::scene(l1,0,0,Some((MARKERS[0],A_VALUE)));
    let b0=fixture::scene(l1,0,0,Some((MARKERS[0],B_VALUE)));
    assert_eq!(evo.phase_native_abstract_state(&a0),evo.phase_native_abstract_state(&b0));
    assert_eq!(evo.active_concept_atom_ids(&base_plain),evo.active_concept_atom_ids(&a0));
    assert_eq!(evo.active_concept_atom_ids(&base_plain),evo.active_concept_atom_ids(&b0));

    let good=fixture::scene(l1,7,0,None);
    let dead=fixture::scene(l1,8,0,None);

    // Candidate selection only. These two facts MUST NOT count as validation.
    fact(evo,&a0,x,&good);
    assert!(evo.phase_native_perceptual_witnesses().is_empty());
    fact(evo,&b0,x,&dead);
    let born=evo.phase_native_perceptual_witnesses();
    assert_eq!(born.len(),1);
    assert_eq!(born[0].eligible_observations,0);
    assert!(!born[0].promoted);
    assert_eq!(born[0].features,[feature_for(A_VALUE),feature_for(B_VALUE)]);

    // Future-only evidence plus ordinary state-specific action experience.
    for cycle in 0..16usize {
        let marker=MARKERS[cycle%MARKERS.len()];
        let layout=cycle%4;
        let a=fixture::scene(l1,0,layout,Some((marker,A_VALUE)));
        let b=fixture::scene(l1,0,layout,Some((marker,B_VALUE)));
        let good=fixture::scene(l1,7,layout,None);
        let dead=fixture::scene(l1,8,layout,None);

        fact(evo,&a,x,&good);
        fact(evo,&a,y,&dead);
        fact(evo,&b,x,&dead);
        fact(evo,&b,y,&good);
    }

    let witnesses=evo.phase_native_perceptual_witnesses();
    let w=&witnesses[0];
    println!("G22_LEARNING swap={} witness={:?}",swap,w);
    assert!(w.promoted);
    assert!(!w.retired);
    assert_eq!(w.eligible_observations,32);
    assert!(w.side_switches>=4);
    assert!(w.log_evidence >= (16.0f64/0.01).ln());
}

fn train_irrelevant_candidate(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
){
    let (x,_)=fixture::action_pair(swap);
    let c0=fixture::scene(l1,1,0,Some((MARKERS[0],NULL_C_VALUE)));
    let d0=fixture::scene(l1,1,0,Some((MARKERS[0],NULL_D_VALUE)));
    let good=fixture::scene(l1,7,0,None);
    let dead=fixture::scene(l1,8,0,None);

    // Select a second candidate from a discovery collision.
    fact(evo,&c0,x,&good);
    fact(evo,&d0,x,&dead);

    for i in 0..128usize {
        let value=if i%2==0 {NULL_C_VALUE}else{NULL_D_VALUE};
        // Outcome alternates independently of feature side in four-step blocks:
        // each side receives equal good/dead counts.
        let outcome_good=(i/2)%2==0;
        let pre=fixture::scene(
            l1,1,i%4,Some((MARKERS[i%MARKERS.len()],value))
        );
        let post=fixture::scene(l1,if outcome_good{7}else{8},i%4,None);
        fact(evo,&pre,x,&post);
    }
    let base=evo.phase_native_abstract_state(
        &fixture::scene(l1,1,0,None)
    ).unwrap().cell;
    let w=evo.phase_native_perceptual_witnesses().into_iter()
        .find(|w|w.base_cell==base).expect("irrelevant candidate");
    println!("G22_NULL_CANDIDATE swap={} witness={:?}",swap,w);
    assert!(!w.promoted);
    assert!(w.retired);
    assert_eq!(w.eligible_observations,128);
}

fn score(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
)->(usize,usize,[usize;2]){
    evo.set_planning_learning_enabled(false);
    let (x,y)=fixture::action_pair(swap);
    let mut full=0usize;
    let mut memoryless=0usize;
    let mut sides=[0usize;2];

    for i in 0..64usize {
        let side=i%2;
        let value=if side==0{A_VALUE}else{B_VALUE};
        let expected=if side==0{x}else{y};
        let layout=4+i%2;
        let current=fixture::scene(
            l1,0,layout,Some((MARKERS[i%MARKERS.len()],value))
        );
        let goal=fixture::scene(l1,7,layout,None);
        evo.observe_initial_real(&current,false);
        let fp=evo.phase_native_learned_fingerprint();
        let got=evo.phase_native_perceptual_action(&goal);
        full+=usize::from(got==(true,Some(expected)));
        sides[side]+=usize::from(got==(true,Some(expected)));
        assert_eq!(evo.phase_native_learned_fingerprint(),fp);

        let mut old=evo.clone();
        memoryless+=usize::from(
            old.plan_phase_native_abstract_goal(&current,&goal,None)
                .map(|d|d.first_action)==Some(expected)
        );
    }
    (full,memoryless,sides)
}

fn causal_checks(
    evo:&EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
){
    let (x,_)=fixture::action_pair(swap);
    let scene=fixture::scene(l1,0,5,Some((MARKERS[2],A_VALUE)));
    let goal=fixture::scene(l1,7,5,None);
    let base=evo.phase_native_abstract_state(&scene).unwrap().cell;
    let w=evo.phase_native_perceptual_witnesses().into_iter()
        .find(|w|w.base_cell==base && w.promoted).unwrap();
    let side=w.features.iter().position(|f|*f==feature_for(A_VALUE)).unwrap();
    let link=w.input_synapses[side][1];

    let mut intact=evo.clone();
    intact.observe_initial_real(&scene,false);
    assert_eq!(intact.phase_native_perceptual_action(&goal),(true,Some(x)));
    let fp=intact.phase_native_learned_fingerprint();

    let mut broken=intact.clone();
    let saved=broken.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
    assert_eq!(broken.phase_native_perceptual_action(&goal),(true,None));
    broken.restore_phase_native_synapse_for_control(link,saved);
    assert_eq!(broken.phase_native_learned_fingerprint(),fp);
    assert_eq!(broken.phase_native_perceptual_action(&goal),(true,Some(x)));

    let mut shifted=intact.clone();
    shifted.perturb_phase_native_synapse_for_control(
        link,1.0,std::f32::consts::PI
    ).unwrap();
    assert_eq!(shifted.phase_native_perceptual_action(&goal),(true,None));

    let mut unrelated=intact.clone();
    unrelated.perturb_phase_native_synapse_for_control(
        w.input_synapses[1-side][1],0.0,0.0
    ).unwrap();
    assert_eq!(unrelated.phase_native_perceptual_action(&goal),(true,Some(x)));

    let checkpoint=intact.phase_native_checkpoint().unwrap();
    let mut restarted=EvoPhase::new(fixture::cfg(&intact));
    assert!(restarted.restore_phase_native_checkpoint(checkpoint));
    restarted.observe_initial_real(&scene,false);
    assert_eq!(restarted.phase_native_perceptual_action(&goal),(true,Some(x)),
        "current raw feature must survive restart without predecessor history");
}

#[test]
fn g22_current_raw_signal_becomes_new_operational_perceptual_variable(){
    for swap in [false,true] {
        let drive=fixture::train_drive();
        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        train_useful(&mut evo,&l1,swap);
        train_irrelevant_candidate(&mut evo,&l1,swap);

        let (full,memoryless,sides)=score(&mut evo,&l1,swap);
        causal_checks(&evo,&l1,swap);
        println!(
            "G22_RESULT swap={} full={}/64 memoryless={}/64 sides={:?} witnesses={:?} legacy={} table_composites={}",
            swap,full,memoryless,sides,evo.phase_native_perceptual_witnesses(),
            evo.planning_transition_count(),evo.composite_concepts().len()
        );
        assert!(full>=60);
        assert!(memoryless<=40);
        assert!(sides[0]>=30 && sides[1]>=30);
        assert_eq!(evo.planning_transition_count(),0);
        assert!(evo.composite_concepts().is_empty());
    }
}

#[test]
fn g22_perceptual_refinement_is_native_not_a_marker_answer_table(){
    let source=include_str!("../src/phase_perceptual.rs");
    for required in [
        "phase_raw_features",
        "percept_candidate_pair",
        "native_cell_observation",
        "context_gate",
        "conductance",
        "phase_native_goal_decision_from_cells",
        "feature_cells",
    ] {
        assert!(source.contains(required),"missing native dependency {}",required);
    }
    for forbidden in [
        "A_VALUE","B_VALUE","MARKERS","STATE_PAIRS","correct_action",
        "world.","HashMap","BTreeMap","BinaryHeap","EvoImaginationPlanner",
    ] {
        assert!(!source.contains(forbidden),"forbidden task shortcut {}",forbidden);
    }
    let host=include_str!("../src/scientific_runtime.rs");
    assert!(host.contains("phase_native_perceptual_action"));
    assert!(host.contains("observe_phase_native_perceptual_result"));
    assert!(host.find("consume_permit(permit)").unwrap()
        < host.find("match execute(action)").unwrap());
}
