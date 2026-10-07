use aeterna_v1::EvoPhase;
use aeterna_v1::carrier::{
    PhaseCompositionWitness, PhasePerceptFeature, PhasePerceptProgram,
};

#[allow(dead_code)]
mod fixture {
    include!("g19_rival_hypothesis_discrimination.rs");

    pub fn drive() -> PhaseDriveCheckpoint { train_drive() }
    pub fn organism(drive:&PhaseDriveCheckpoint)->EvoPhase { target(drive) }
    pub fn train_rep(evo:&mut EvoPhase)->[[usize;2];8] { train_abstraction(evo) }
    pub fn cfg(evo:&EvoPhase)->EvoConfig { evo.config().clone() }

    pub fn scene(
        l1:&[[usize;2];8],
        state:usize,
        layout:usize,
        weak:&[(usize,f32)],
    )->Vec<f32>{
        let mut s=state_scene(l1,state,LAYOUTS[layout%LAYOUTS.len()]);
        for &(index,value) in weak {
            assert!(index<s.len() && s[index]<0.5);
            s[index]=value;
        }
        s
    }

    pub fn action_pair(swap:bool)->(usize,usize){
        let r=roles(swap);
        (r.probe_a,r.probe_b)
    }
}

const A_VALUE:f32=0.20; // bin 3
const B_VALUE:f32=0.40; // bin 6
const NUISANCE_LOW:f32=0.10;
const NUISANCE_HIGH:f32=0.30;

const A_POS:[usize;4]=[399,379,359,339];
const B_POS:[usize;4]=[398,378,358,338];
const N_POS:[usize;4]=[397,377,357,337];

fn descriptor(value:f32)->PhasePerceptFeature{
    PhasePerceptFeature::WeakAmplitudeBin(
        ((value*16.0).floor() as u8).min(7)
    )
}

fn raw(
    l1:&[[usize;2];8],
    layout:usize,
    a:bool,
    b:bool,
    nuisance:bool,
)->Vec<f32>{
    let slot=layout%4;
    let mut weak=Vec::new();
    if a { weak.push((A_POS[slot],A_VALUE)); }
    if b { weak.push((B_POS[slot],B_VALUE)); }
    weak.push((N_POS[slot],if nuisance{NUISANCE_HIGH}else{NUISANCE_LOW}));
    fixture::scene(l1,0,layout,&weak)
}

fn goal(l1:&[[usize;2];8],layout:usize)->Vec<f32>{
    fixture::scene(l1,7,layout,&[])
}

fn dead(l1:&[[usize;2];8],layout:usize)->Vec<f32>{
    fixture::scene(l1,8,layout,&[])
}

fn fact(evo:&mut EvoPhase,pre:&[f32],action:usize,post:&[f32]){
    evo.observe_initial_real(pre,false);
    assert!(evo.observe_phase_native_compositional_result(action,post).is_some());
}

fn outcome_for_x(a:bool,b:bool)->bool{a^b}

#[derive(Clone,Copy,Default)]
struct AtomStats {
    counts: [[usize;2];2],
}

impl AtomStats {
    fn observe(&mut self,present:bool,goal:bool){
        self.counts[usize::from(present)][usize::from(goal)] += 1;
    }
    fn predict_goal(&self,present:bool)->bool{
        let c=self.counts[usize::from(present)];
        c[1] >= c[0]
    }
}

#[derive(Clone,Copy,Default)]
struct TrainingStats {
    a: AtomStats,
    b: AtomStats,
}

fn train(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
)->TrainingStats{
    let (x,y)=fixture::action_pair(swap);
    let mut stats=TrainingStats::default();
    assert!(evo.enable_phase_native_compositional_refinement());
    assert!(evo.phase_native_composition_witnesses().is_empty());

    let inherited=fixture::scene(l1,0,0,&[]);
    for &(a,b) in &[(false,false),(false,true),(true,false),(true,true)] {
        let s=raw(l1,0,a,b,false);
        assert_eq!(evo.phase_native_abstract_state(&s),
                   evo.phase_native_abstract_state(&inherited));
        assert_eq!(evo.active_concept_atom_ids(&s),
                   evo.active_concept_atom_ids(&inherited));
    }

    // Discovery: 10 -> goal, 11 -> dead. This creates competing Atom(B),
    // And(A,B), Xor(A,B) programs in frozen deterministic syntax order.
    let d1=raw(l1,0,true,false,false);
    let d2=raw(l1,1,true,true,false);
    fact(evo,&d1,x,&goal(l1,0));
    assert!(evo.phase_native_composition_witnesses().is_empty());
    fact(evo,&d2,x,&dead(l1,1));
    let born=evo.phase_native_composition_witnesses();
    assert!(born.len()>=3 && born.len()<=16);
    assert!(born.iter().all(|w|w.eligible_observations==0));
    let a_desc=descriptor(A_VALUE);
    let b_desc=descriptor(B_VALUE);
    assert!(born.iter().any(|w|w.program==PhasePerceptProgram::Atom(b_desc)));
    assert!(born.iter().any(|w|w.program==PhasePerceptProgram::And(a_desc,b_desc)));
    assert!(born.iter().any(|w|w.program==PhasePerceptProgram::Xor(a_desc,b_desc)));

    // Frozen future distribution per 10 anchor facts:
    // 00 x6, 01 x1, 10 x1, 11 x2.
    // A/B single-atom effects are ~0.19, AND effect=0.25, XOR effect=1.
    let block=[
        (false,false),(false,true),(false,false),(true,false),(false,false),
        (true,true),(false,false),(true,true),(false,false),(false,false),
    ];
    for cycle in 0..4usize {
        for (index,&(a,b)) in block.iter().enumerate() {
            let layout=(cycle+index)%4;
            let nuisance=(cycle+index)%2==0;
            let pre=raw(l1,layout,a,b,nuisance);
            let x_good=outcome_for_x(a,b);
            stats.a.observe(a,x_good);
            stats.b.observe(b,x_good);
            let post_x=if x_good{goal(l1,layout)}else{dead(l1,layout)};
            let post_y=if x_good{dead(l1,layout)}else{goal(l1,layout)};
            fact(evo,&pre,x,&post_x);
            fact(evo,&pre,y,&post_y);
        }
    }

    let witnesses=evo.phase_native_composition_witnesses();
    println!("G23_LEARNING swap={} witnesses={:?}",swap,witnesses);
    let a_desc=descriptor(A_VALUE);
    let b_desc=descriptor(B_VALUE);
    let xor=witnesses.iter().find(|w|
        w.program==PhasePerceptProgram::Xor(a_desc,b_desc))
        .expect("target XOR candidate");
    let atom=witnesses.iter().find(|w|
        w.program==PhasePerceptProgram::Atom(b_desc))
        .expect("target Atom candidate");
    let and=witnesses.iter().find(|w|
        w.program==PhasePerceptProgram::And(a_desc,b_desc))
        .expect("target AND candidate");

    assert!(xor.promoted && !xor.retired);
    assert_eq!(xor.eligible_observations,40);
    assert!(xor.log_evidence >= (16.0f64/0.01).ln());
    assert!(xor.atom_effects.iter().all(|e|*e<=0.25+1e-12));
    assert!(!atom.promoted);
    assert!(!and.promoted);
    assert!(and.atom_effects.iter().all(|e|*e<=0.25+1e-12));
    stats
}

fn train_irrelevant_composition(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
){
    let (x,_)=fixture::action_pair(swap);
    let c=0.10f32;
    let d=0.30f32;
    let positions=[396usize,376,356,336];

    let make=|layout:usize,c_on:bool,d_on:bool| {
        let mut weak=Vec::new();
        if c_on { weak.push((positions[layout%4],c)); }
        if d_on { weak.push((positions[(layout+1)%4],d)); }
        fixture::scene(l1,1,layout,&weak)
    };

    // Discovery collision at another base state.
    fact(evo,&make(0,true,false),x,&goal(l1,0));
    fact(evo,&make(1,true,true),x,&dead(l1,1));

    // Future evidence is deliberately nonpredictive for every simple/composed
    // program: outcome alternates independently of the raw pair.
    for i in 0..40usize {
        let c_on=i%2==0;
        let d_on=(i/2)%2==0;
        let good=(i/4)%2==0;
        let pre=make(i%4,c_on,d_on);
        let post=if good{goal(l1,i%4)}else{dead(l1,i%4)};
        fact(evo,&pre,x,&post);
    }
    let base=evo.phase_native_abstract_state(&make(0,false,false)).unwrap().cell;
    let ws=evo.phase_native_composition_witnesses().into_iter()
        .filter(|w|w.base_cell==base).collect::<Vec<_>>();
    assert!(!ws.is_empty());
    assert!(ws.iter().all(|w|!w.promoted));
}

fn xor_witness(evo:&EvoPhase)->PhaseCompositionWitness{
    let a=descriptor(A_VALUE);
    let b=descriptor(B_VALUE);
    evo.phase_native_composition_witnesses().into_iter()
        .find(|w|w.promoted && w.program==PhasePerceptProgram::Xor(a,b))
        .expect("promoted target XOR witness")
}

fn eval_program(program:PhasePerceptProgram,a:bool,b:bool)->bool{
    let fa=descriptor(A_VALUE);
    let fb=descriptor(B_VALUE);
    match program {
        PhasePerceptProgram::Atom(f)=> {
            (f==fa && a)||(f==fb && b)
        }
        PhasePerceptProgram::And(x,y)=> {
            let px=(x==fa&&a)||(x==fb&&b);
            let py=(y==fa&&a)||(y==fb&&b);
            px&&py
        }
        PhasePerceptProgram::Xor(x,y)=> {
            let px=(x==fa&&a)||(x==fb&&b);
            let py=(y==fa&&a)||(y==fb&&b);
            px^py
        }
    }
}

fn score(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
    stats:TrainingStats,
)->(usize,usize,usize,[usize;4]){
    evo.set_planning_learning_enabled(false);
    let (x,y)=fixture::action_pair(swap);
    let witnesses=evo.phase_native_composition_witnesses();
    let a_desc=descriptor(A_VALUE);
    let b_desc=descriptor(B_VALUE);
    let atom=witnesses.iter().find(|w|w.program==PhasePerceptProgram::Atom(b_desc)).unwrap();
    let and=witnesses.iter().find(|w|w.program==PhasePerceptProgram::And(a_desc,b_desc)).unwrap();
    assert!(!atom.promoted && !and.promoted);

    let mut full=0usize;
    let mut memoryless=0usize;
    let mut best_atom=0usize;
    let mut combos=[0usize;4];

    for combo in 0..4usize {
        let a=combo&2!=0;
        let b=combo&1!=0;
        for rep in 0..16usize {
            let layout=4+rep%2;
            let nuisance=(combo+rep)%2==0;
            let current=raw(l1,layout,a,b,nuisance);
            let target=goal(l1,layout);
            let expected=if outcome_for_x(a,b){x}else{y};

            evo.observe_initial_real(&current,false);
            let fp=evo.phase_native_learned_fingerprint();
            let got=evo.phase_native_compositional_action(&target);
            full+=usize::from(got==(true,Some(expected)));
            combos[combo]+=usize::from(got==(true,Some(expected)));
            assert_eq!(evo.phase_native_learned_fingerprint(),fp);

            let mut old=evo.clone();
            memoryless+=usize::from(
                old.plan_phase_native_abstract_goal(&current,&target,None)
                    .map(|d|d.first_action)==Some(expected)
            );

            let pa=stats.a.predict_goal(a);
            let pb=stats.b.predict_goal(b);
            let action_a=if pa{x}else{y};
            let action_b=if pb{x}else{y};
            // Report the stronger one-atom learner after scoring both on the
            // same held-out frames.
            best_atom += usize::from(action_a==expected) << 16;
            best_atom += usize::from(action_b==expected);
        }
    }

    let a_score=best_atom>>16;
    let b_score=best_atom & 0xFFFF;
    (full,memoryless,a_score.max(b_score),combos)
}

fn causal(
    evo:&EvoPhase,
    l1:&[[usize;2];8],
    swap:bool,
){
    let (x,_)=fixture::action_pair(swap);
    let current=raw(l1,5,true,false,true);
    let target=goal(l1,5);
    let w=xor_witness(evo);
    assert!(eval_program(w.program,true,false));
    let side=1usize;
    let link=w.input_synapses[side][1];

    let mut intact=evo.clone();
    intact.observe_initial_real(&current,false);
    assert_eq!(intact.phase_native_compositional_action(&target),(true,Some(x)));
    let fp=intact.phase_native_learned_fingerprint();

    let mut broken=intact.clone();
    let saved=broken.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
    assert_eq!(broken.phase_native_compositional_action(&target),(true,None));
    broken.restore_phase_native_synapse_for_control(link,saved);
    assert_eq!(broken.phase_native_learned_fingerprint(),fp);
    assert_eq!(broken.phase_native_compositional_action(&target),(true,Some(x)));

    let mut shifted=intact.clone();
    shifted.perturb_phase_native_synapse_for_control(
        link,1.0,std::f32::consts::PI
    ).unwrap();
    assert_eq!(shifted.phase_native_compositional_action(&target),(true,None));

    let base1=evo.phase_native_abstract_state(
        &fixture::scene(l1,1,0,&[])
    ).unwrap().cell;
    let unrelated_w=evo.phase_native_composition_witnesses().into_iter()
        .find(|w|w.base_cell==base1).expect("irrelevant composition witness");
    let mut unrelated=intact.clone();
    unrelated.perturb_phase_native_synapse_for_control(
        unrelated_w.input_synapses[0][1],0.0,0.0
    ).unwrap();
    assert_eq!(unrelated.phase_native_compositional_action(&target),(true,Some(x)));

    let checkpoint=intact.phase_native_checkpoint().unwrap();
    let mut restarted=EvoPhase::new(fixture::cfg(&intact));
    assert!(restarted.restore_phase_native_checkpoint(checkpoint));
    restarted.observe_initial_real(&current,false);
    assert_eq!(restarted.phase_native_compositional_action(&target),(true,Some(x)));
}

#[test]
fn g23_synthesizes_composed_current_sensory_predicate_when_atoms_fail(){
    for swap in [false,true] {
        let drive=fixture::drive();
        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        let stats=train(&mut evo,&l1,swap);
        train_irrelevant_composition(&mut evo,&l1,swap);

        let (full,memoryless,single,combos)=score(&mut evo,&l1,swap,stats);
        causal(&evo,&l1,swap);
        let witnesses=evo.phase_native_composition_witnesses();
        println!(
            "G23_RESULT swap={} full={}/64 memoryless={}/64 single_atom={}/64 combos={:?} witnesses={:?} legacy={} table_composites={}",
            swap,full,memoryless,single,combos,witnesses,
            evo.planning_transition_count(),evo.composite_concepts().len()
        );
        assert!(full>=60);
        assert!(memoryless<=40);
        assert!(single<=40);
        assert!(combos.iter().all(|x|*x>=15));
        assert!(witnesses.iter().any(|w|
            matches!(w.program,PhasePerceptProgram::And(_, _))&&!w.promoted));
        assert_eq!(evo.planning_transition_count(),0);
        assert!(evo.composite_concepts().is_empty());
    }
}

#[test]
fn g23_composition_is_native_not_an_xor_answer_table(){
    let source=include_str!("../src/phase_compositional.rs");
    for required in [
        "candidate_programs","PhasePerceptProgram::And",
        "PhasePerceptProgram::Xor","native_cell_observation",
        "composition_gate","context_gate","conductance",
        "phase_native_goal_decision_from_cells",
    ] {
        assert!(source.contains(required),"missing native dependency {}",required);
    }
    for forbidden in [
        "A_VALUE","B_VALUE","MARKERS","STATE_PAIRS","correct_action",
        "truth_table","world.","EvoImaginationPlanner","HashMap","BinaryHeap",
    ] {
        assert!(!source.contains(forbidden),"forbidden task shortcut {}",forbidden);
    }
    let host=include_str!("../src/scientific_runtime.rs");
    assert!(host.contains("phase_native_compositional_action"));
    assert!(host.contains("observe_phase_native_compositional_result"));
    assert!(host.find("consume_permit(permit)").unwrap()
        < host.find("match execute(action)").unwrap());
}
