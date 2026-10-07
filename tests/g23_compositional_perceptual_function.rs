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
    assert!(xor.eligible_observations>=32 && xor.eligible_observations<=40);
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


#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Fresh23Op{And,Xor}

#[derive(Clone,Copy,Debug)]
struct Fresh23Sample{
    combo:u8,
    nuisance:bool,
    layout:u8,
    pattern:u8,
}

#[derive(Clone,Copy,Debug)]
struct Fresh23Irrelevant{
    combo:u8,
    good:bool,
    layout:u8,
    pattern:u8,
}

#[derive(Clone,Debug)]
struct Fresh23Spec{
    op:Fresh23Op,
    bins:[u8;5],
    states:[usize;4],
    motors:[usize;6],
    positions:[[usize;3];4],
    layouts:[u8;6],
    train:Vec<Fresh23Sample>,
    score:Vec<Fresh23Sample>,
    irrelevant:Vec<Fresh23Irrelevant>,
}

struct Fresh23Rng(u64);
impl Fresh23Rng{
    fn new(seed:u64)->Self{Self(seed^0xA323_23C0_6D50_2026)}
    fn next(&mut self)->u64{
        let mut x=self.0;
        x^=x>>12;x^=x<<25;x^=x>>27;
        self.0=x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self,n:usize)->usize{(self.next()%n as u64)as usize}
}
fn fresh23_shuffle<T>(rng:&mut Fresh23Rng,x:&mut[T]){
    for i in(1..x.len()).rev(){let j=rng.range(i+1);x.swap(i,j);}
}
fn fresh23_fnv(mut h:u64,x:u64)->u64{
    for b in x.to_le_bytes(){h^=b as u64;h=h.wrapping_mul(1_099_511_628_211);}
    h
}
fn fresh23_value(bin:u8)->f32{(bin as f32+0.5)/16.0}
fn fresh23_desc(bin:u8)->PhasePerceptFeature{
    PhasePerceptFeature::WeakAmplitudeBin(bin)
}
fn fresh23_program(op:Fresh23Op,a:u8,b:u8)->PhasePerceptProgram{
    let mut x=fresh23_desc(a);let mut y=fresh23_desc(b);
    if y<x{std::mem::swap(&mut x,&mut y);}
    match op{
        Fresh23Op::And=>PhasePerceptProgram::And(x,y),
        Fresh23Op::Xor=>PhasePerceptProgram::Xor(x,y),
    }
}
fn fresh23_wrong_program(op:Fresh23Op,a:u8,b:u8)->PhasePerceptProgram{
    fresh23_program(
        match op{Fresh23Op::And=>Fresh23Op::Xor,Fresh23Op::Xor=>Fresh23Op::And},
        a,b
    )
}
fn fresh23_truth(op:Fresh23Op,a:bool,b:bool)->bool{
    match op{Fresh23Op::And=>a&&b,Fresh23Op::Xor=>a^b}
}
fn fresh23_specs(authority:u64)->(Vec<Fresh23Spec>,u64){
    let mut global=Fresh23Rng::new(authority^0x23_0000_0000);
    let mut motor_base=[0usize,1,2,3,4,5];
    fresh23_shuffle(&mut global,&mut motor_base);
    let parity=(authority&1)as usize;
    let mut specs=Vec::new();
    let mut digest=14_695_981_039_346_656_037u64;

    for sub in 0..10usize{
        let mut rng=Fresh23Rng::new(
            authority^(sub as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^0x23F0_5EED_0000_0023
        );
        let op=if ((sub&1)^parity)==0{Fresh23Op::And}else{Fresh23Op::Xor};

        let mut all_bins=[1u8,2,3,4,5,6,7];
        fresh23_shuffle(&mut rng,&mut all_bins);
        let bins=[all_bins[0],all_bins[1],all_bins[2],all_bins[3],all_bins[4]];

        let mut states=[0usize,1,2,3,4,5,6,7,8];
        fresh23_shuffle(&mut rng,&mut states);
        let state_roles=[states[0],states[1],states[2],states[3]];

        let x=motor_base[sub%6];
        let y=motor_base[(sub+1)%6];
        let mut rest=(0usize..6).filter(|m|*m!=x&&*m!=y).collect::<Vec<_>>();
        fresh23_shuffle(&mut rng,&mut rest);
        let motors=[x,y,rest[0],rest[1],rest[2],rest[3]];

        let mut pool=(16usize..20)
            .flat_map(|yy|(0usize..20).map(move|xx|yy*20+xx))
            .collect::<Vec<_>>();
        fresh23_shuffle(&mut rng,&mut pool);
        let mut positions=[[0usize;3];4];
        for p in 0..4{for q in 0..3{positions[p][q]=pool[p*3+q];}}

        let mut layouts=[0u8,1,2,3,4,5];
        fresh23_shuffle(&mut rng,&mut layouts);

        let base_block:Vec<u8>=match op{
            Fresh23Op::And=>vec![0,0,0,1,1,1,2,2,2,3],
            Fresh23Op::Xor=>vec![0,0,0,0,0,0,1,2,3,3],
        };
        let mut train=Vec::new();
        for block in 0..8usize{
            let mut b=base_block.clone();
            fresh23_shuffle(&mut rng,&mut b);
            for (i,combo) in b.into_iter().enumerate(){
                train.push(Fresh23Sample{
                    combo,
                    nuisance:rng.range(2)==1,
                    layout:layouts[(block+i)%4],
                    pattern:((block+i)%2)as u8,
                });
            }
        }
        assert_eq!(train.len(),80);

        let mut score_combos=vec![0u8,0,1,1,2,2,3,3];
        fresh23_shuffle(&mut rng,&mut score_combos);
        let score=score_combos.into_iter().enumerate().map(|(i,combo)|
            Fresh23Sample{
                combo,
                nuisance:rng.range(2)==1,
                layout:layouts[4+i%2],
                pattern:(2+i%2)as u8,
            }
        ).collect::<Vec<_>>();

        let irrelevant=(0..40usize).map(|i|Fresh23Irrelevant{
            combo:((i%2)<<1|((i/2)%2))as u8,
            good:(i/4)%2==0,
            layout:layouts[i%4],
            pattern:(i%2)as u8,
        }).collect::<Vec<_>>();

        let spec=Fresh23Spec{
            op,bins,states:state_roles,motors,positions,layouts,
            train,score,irrelevant,
        };

        digest=fresh23_fnv(digest,sub as u64);
        digest=fresh23_fnv(digest,match op{Fresh23Op::And=>1,Fresh23Op::Xor=>2});
        for x in spec.bins{digest=fresh23_fnv(digest,x as u64);}
        for x in spec.states{digest=fresh23_fnv(digest,x as u64);}
        for x in spec.motors{digest=fresh23_fnv(digest,x as u64);}
        for p in spec.positions{for x in p{digest=fresh23_fnv(digest,x as u64);}}
        for x in spec.layouts{digest=fresh23_fnv(digest,x as u64);}
        for s in &spec.train{
            digest=fresh23_fnv(digest,s.combo as u64);
            digest=fresh23_fnv(digest,s.nuisance as u64);
            digest=fresh23_fnv(digest,s.layout as u64);
            digest=fresh23_fnv(digest,s.pattern as u64);
        }
        for s in &spec.score{
            digest=fresh23_fnv(digest,s.combo as u64);
            digest=fresh23_fnv(digest,s.nuisance as u64);
            digest=fresh23_fnv(digest,s.layout as u64);
            digest=fresh23_fnv(digest,s.pattern as u64);
        }
        specs.push(spec);
    }
    (specs,digest)
}

fn fresh23_raw(
    l1:&[[usize;2];8],
    spec:&Fresh23Spec,
    state:usize,
    sample:Fresh23Sample,
)->Vec<f32>{
    let a=sample.combo&2!=0;
    let b=sample.combo&1!=0;
    let pos=spec.positions[sample.pattern as usize];
    let mut weak=Vec::new();
    if a{weak.push((pos[0],fresh23_value(spec.bins[0])));}
    if b{weak.push((pos[1],fresh23_value(spec.bins[1])));}
    if sample.nuisance{weak.push((pos[2],fresh23_value(spec.bins[2])));}
    fixture::scene(l1,state,sample.layout as usize,&weak)
}
fn fresh23_irrelevant_raw(
    l1:&[[usize;2];8],
    spec:&Fresh23Spec,
    sample:Fresh23Irrelevant,
)->Vec<f32>{
    let c=sample.combo&2!=0;
    let d=sample.combo&1!=0;
    let pos=spec.positions[sample.pattern as usize];
    let mut weak=Vec::new();
    if c{weak.push((pos[0],fresh23_value(spec.bins[3])));}
    if d{weak.push((pos[1],fresh23_value(spec.bins[4])));}
    fixture::scene(l1,spec.states[3],sample.layout as usize,&weak)
}


fn fresh23_safe_positions(
    l1:&[[usize;2];8],
    state:usize,
    layouts:&[u8],
)->Vec<usize>{
    (0..400usize).filter(|&index|{
        layouts.iter().all(|layout|
            fixture::scene(l1,state,*layout as usize,&[])[index] < 0.5
        )
    }).collect()
}

fn fresh23_specs_safe(
    authority:u64,
    l1:&[[usize;2];8],
)->(Vec<Fresh23Spec>,u64){
    let (mut specs,_)=fresh23_specs(authority);
    let mut digest=14_695_981_039_346_656_037u64;
    for (sub,spec) in specs.iter_mut().enumerate(){
        let mut rng=Fresh23Rng::new(
            authority^(sub as u64).wrapping_mul(0xD1B5_4A32_D192_ED03)
                ^0x23F3_5AFE_0000_0023
        );
        let base_layouts=spec.layouts.to_vec();
        let mut base_pool=fresh23_safe_positions(l1,spec.states[0],&base_layouts);
        let mut irrelevant_pool=fresh23_safe_positions(
            l1,spec.states[3],&spec.layouts[..4]
        );
        assert!(base_pool.len()>=12 && irrelevant_pool.len()>=12,
            "FRESH_G23_3_PRESEAL_SAFE_POOL sub={} base={} irrelevant={}",
            sub,base_pool.len(),irrelevant_pool.len());
        fresh23_shuffle(&mut rng,&mut base_pool);
        fresh23_shuffle(&mut rng,&mut irrelevant_pool);
        for pattern in 0..4usize {
            spec.positions[pattern]=[
                base_pool[pattern*3],
                base_pool[pattern*3+1],
                base_pool[pattern*3+2],
            ];
        }

        // The irrelevant helper reuses positions[pattern][0..2], so require
        // those same selected positions to be safe on its separate base too.
        // Replace with positions from the intersection for the layouts where
        // irrelevant observations occur.
        let mut joint=(0..400usize).filter(|index|
            base_layouts.iter().all(|layout|
                fixture::scene(l1,spec.states[0],*layout as usize,&[])[*index] < 0.5
            ) && spec.layouts[..4].iter().all(|layout|
                fixture::scene(l1,spec.states[3],*layout as usize,&[])[*index] < 0.5
            )
        ).collect::<Vec<_>>();
        assert!(joint.len()>=12,
            "FRESH_G23_3_PRESEAL_JOINT_POOL sub={} joint={}",sub,joint.len());
        fresh23_shuffle(&mut rng,&mut joint);
        for pattern in 0..4usize {
            spec.positions[pattern]=[
                joint[pattern*3],
                joint[pattern*3+1],
                joint[pattern*3+2],
            ];
        }

        digest=fresh23_fnv(digest,sub as u64);
        digest=fresh23_fnv(digest,match spec.op{Fresh23Op::And=>1,Fresh23Op::Xor=>2});
        for x in spec.bins{digest=fresh23_fnv(digest,x as u64);}
        for x in spec.states{digest=fresh23_fnv(digest,x as u64);}
        for x in spec.motors{digest=fresh23_fnv(digest,x as u64);}
        for p in spec.positions{for x in p{digest=fresh23_fnv(digest,x as u64);}}
        for x in spec.layouts{digest=fresh23_fnv(digest,x as u64);}
        for s in &spec.train{
            digest=fresh23_fnv(digest,s.combo as u64);
            digest=fresh23_fnv(digest,s.nuisance as u64);
            digest=fresh23_fnv(digest,s.layout as u64);
            digest=fresh23_fnv(digest,s.pattern as u64);
        }
        for s in &spec.score{
            digest=fresh23_fnv(digest,s.combo as u64);
            digest=fresh23_fnv(digest,s.nuisance as u64);
            digest=fresh23_fnv(digest,s.layout as u64);
            digest=fresh23_fnv(digest,s.pattern as u64);
        }
    }
    (specs,digest)
}

fn fresh23_wilson95(success:usize,n:usize)->(f64,f64){
    let z=1.959_963_984_540_054_f64;
    let n=n as f64;let p=success as f64/n;
    let den=1.0+z*z/n;
    let center=(p+z*z/(2.0*n))/den;
    let half=z*(p*(1.0-p)/n+z*z/(4.0*n*n)).sqrt()/den;
    (center-half,center+half)
}

#[test]
#[ignore="requires one-use AETERNA_FRESH_SEED from first-attempt CI"]
fn g23_fresh_compositional_perceptual_pack(){
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required").parse().unwrap();
    let source_sha=std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_|"unknown".into());
    let (specs,digest)=fresh23_specs(authority);

    println!(
        "FRESH_G23_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (i,s) in specs.iter().enumerate(){println!("FRESH_G23_BLOCK sub={} {:?}",i,s);}

    let drive=fixture::drive();
    let mut full=0usize;
    let mut per_seed=Vec::new();
    let mut combo_scores=[0usize;4];
    let mut memoryless=0usize;
    let mut single=0usize;
    let mut promoted=0usize;
    let mut and_promoted=0usize;
    let mut xor_promoted=0usize;
    let mut structure_violations=0usize;
    let mut leakage_violations=0usize;
    let mut atom_effect_violations=0usize;
    let mut wrong_operator_promotions=0usize;
    let mut irrelevant_promotions=0usize;
    let mut lesion=0usize;
    let mut phase=0usize;
    let mut restore=0usize;
    let mut unrelated=0usize;
    let mut restart=0usize;
    let mut fp_violations=0usize;
    let mut legacy_violations=0usize;
    let mut motor_mask=0u8;

    for (sub,spec) in specs.iter().enumerate(){
        motor_mask|=1u8<<spec.motors[0];
        motor_mask|=1u8<<spec.motors[1];

        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        assert!(evo.enable_phase_native_compositional_refinement());
        let x=spec.motors[0];let y=spec.motors[1];
        let base=spec.states[0];let goal_state=spec.states[1];let dead_state=spec.states[2];

        let d10=Fresh23Sample{combo:2,nuisance:false,layout:spec.layouts[0],pattern:0};
        let d11=Fresh23Sample{combo:3,nuisance:false,layout:spec.layouts[1],pattern:1};
        let s10=fresh23_raw(&l1,spec,base,d10);
        let s11=fresh23_raw(&l1,spec,base,d11);
        let inherited=fixture::scene(&l1,base,spec.layouts[0] as usize,&[]);
        if evo.phase_native_abstract_state(&s10)!=evo.phase_native_abstract_state(&inherited)
            || evo.phase_native_abstract_state(&s11)!=evo.phase_native_abstract_state(&inherited)
            || evo.active_concept_atom_ids(&s10)!=evo.active_concept_atom_ids(&inherited)
            || evo.active_concept_atom_ids(&s11)!=evo.active_concept_atom_ids(&inherited)
        {structure_violations+=1;}

        let d10_good=fresh23_truth(spec.op,true,false);
        fact(&mut evo,&s10,x,
            &fixture::scene(&l1,if d10_good{goal_state}else{dead_state},
                            spec.layouts[0] as usize,&[]));
        fact(&mut evo,&s11,x,
            &fixture::scene(&l1,if fresh23_truth(spec.op,true,true){goal_state}else{dead_state},
                            spec.layouts[1] as usize,&[]));

        let target=fresh23_program(spec.op,spec.bins[0],spec.bins[1]);
        let wrong=fresh23_wrong_program(spec.op,spec.bins[0],spec.bins[1]);
        let born=evo.phase_native_composition_witnesses();
        let Some(target_born)=born.iter().find(|w|w.program==target) else{
            structure_violations+=1;continue;
        };
        if target_born.eligible_observations!=0{leakage_violations+=1;}

        let mut stats=TrainingStats::default();
        for sample in &spec.train{
            let a=sample.combo&2!=0;let b=sample.combo&1!=0;
            let x_good=fresh23_truth(spec.op,a,b);
            stats.a.observe(a,x_good);stats.b.observe(b,x_good);
            let pre=fresh23_raw(&l1,spec,base,*sample);
            let post_x=fixture::scene(
                &l1,if x_good{goal_state}else{dead_state},
                sample.layout as usize,&[]);
            let post_y=fixture::scene(
                &l1,if x_good{dead_state}else{goal_state},
                sample.layout as usize,&[]);
            fact(&mut evo,&pre,x,&post_x);
            fact(&mut evo,&pre,y,&post_y);
        }

        // Irrelevant candidate pool on a separate base.
        let ir0=Fresh23Irrelevant{combo:2,good:true,layout:spec.layouts[0],pattern:0};
        let ir1=Fresh23Irrelevant{combo:3,good:false,layout:spec.layouts[1],pattern:1};
        fact(&mut evo,&fresh23_irrelevant_raw(&l1,spec,ir0),x,
             &fixture::scene(&l1,goal_state,spec.layouts[0] as usize,&[]));
        fact(&mut evo,&fresh23_irrelevant_raw(&l1,spec,ir1),x,
             &fixture::scene(&l1,dead_state,spec.layouts[1] as usize,&[]));
        for sample in &spec.irrelevant{
            let pre=fresh23_irrelevant_raw(&l1,spec,*sample);
            let post=fixture::scene(
                &l1,if sample.good{goal_state}else{dead_state},
                sample.layout as usize,&[]);
            fact(&mut evo,&pre,x,&post);
        }

        let ws=evo.phase_native_composition_witnesses();
        let base_cell=evo.phase_native_abstract_state(&inherited).unwrap().cell;
        let target_w=ws.iter().find(|w|w.base_cell==base_cell&&w.program==target);
        if let Some(w)=target_w{
            if w.promoted{
                promoted+=1;
                match spec.op{Fresh23Op::And=>and_promoted+=1,Fresh23Op::Xor=>xor_promoted+=1}
            }
            if matches!(w.program,PhasePerceptProgram::Atom(_)){structure_violations+=1;}
            if w.atom_effects.iter().any(|e|*e>0.25+1e-12){atom_effect_violations+=1;}
        }else{structure_violations+=1;}

        if ws.iter().any(|w|w.base_cell==base_cell&&w.program==wrong&&w.promoted){
            wrong_operator_promotions+=1;
        }
        let irr_plain=fixture::scene(&l1,spec.states[3],spec.layouts[0] as usize,&[]);
        let irr_cell=evo.phase_native_abstract_state(&irr_plain).unwrap().cell;
        let irr_ws=ws.iter().filter(|w|w.base_cell==irr_cell).collect::<Vec<_>>();
        if irr_ws.iter().any(|w|w.promoted){irrelevant_promotions+=1;}

        evo.set_planning_learning_enabled(false);
        let mut sub_full=0usize;
        let mut sub_mem=0usize;
        let mut a_single=0usize;
        let mut b_single=0usize;
        for sample in &spec.score{
            let a=sample.combo&2!=0;let b=sample.combo&1!=0;
            let expected=if fresh23_truth(spec.op,a,b){x}else{y};
            let current=fresh23_raw(&l1,spec,base,*sample);
            let goal_raw=fixture::scene(&l1,goal_state,sample.layout as usize,&[]);
            evo.observe_initial_real(&current,false);
            let fp=evo.phase_native_learned_fingerprint();
            let got=evo.phase_native_compositional_action(&goal_raw);
            let ok=got==(true,Some(expected));
            full+=usize::from(ok);sub_full+=usize::from(ok);
            combo_scores[sample.combo as usize]+=usize::from(ok);
            if evo.phase_native_learned_fingerprint()!=fp{fp_violations+=1;}

            let mut old=evo.clone();
            let m=old.plan_phase_native_abstract_goal(&current,&goal_raw,None)
                .map(|d|d.first_action)==Some(expected);
            memoryless+=usize::from(m);sub_mem+=usize::from(m);

            let pa=stats.a.predict_goal(a);let pb=stats.b.predict_goal(b);
            a_single+=usize::from((if pa{x}else{y})==expected);
            b_single+=usize::from((if pb{x}else{y})==expected);
        }
        single+=a_single.max(b_single);
        per_seed.push(sub_full);

        // Targeted physical interventions use a held-out sample where P=true.
        let true_sample=*spec.score.iter().find(|s|{
            let a=s.combo&2!=0;let b=s.combo&1!=0;
            fresh23_truth(spec.op,a,b)
        }).expect("true-side heldout sample");
        let current=fresh23_raw(&l1,spec,base,true_sample);
        let goal_raw=fixture::scene(&l1,goal_state,true_sample.layout as usize,&[]);
        let w=evo.phase_native_composition_witnesses().into_iter()
            .find(|w|w.base_cell==base_cell&&w.program==target&&w.promoted)
            .expect("fresh promoted target");
        let link=w.input_synapses[1][1];

        let mut damaged=evo.clone();
        damaged.observe_initial_real(&current,false);
        let saved=damaged.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
        lesion+=usize::from(
            damaged.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );
        damaged.restore_phase_native_synapse_for_control(link,saved);
        restore+=usize::from(
            damaged.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        let mut shifted=evo.clone();
        shifted.observe_initial_real(&current,false);
        shifted.perturb_phase_native_synapse_for_control(
            link,1.0,std::f32::consts::PI).unwrap();
        phase+=usize::from(
            shifted.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        if let Some(iw)=irr_ws.first(){
            let mut ir=evo.clone();
            ir.observe_initial_real(&current,false);
            ir.perturb_phase_native_synapse_for_control(
                iw.input_synapses[0][1],0.0,0.0).unwrap();
            unrelated+=usize::from(
                ir.phase_native_compositional_action(&goal_raw)==(true,Some(x))
            );
        }

        let checkpoint=evo.phase_native_checkpoint().unwrap();
        let mut restarted=EvoPhase::new(fixture::cfg(&evo));
        assert!(restarted.restore_phase_native_checkpoint(checkpoint));
        restarted.observe_initial_real(&current,false);
        restart+=usize::from(
            restarted.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        if evo.planning_transition_count()!=0||!evo.composite_concepts().is_empty(){
            legacy_violations+=1;
        }

        println!(
            "FRESH_G23_SUB sub={} op={:?} full={}/8 memoryless={}/8 single={}/8 target={:?}",
            sub,spec.op,sub_full,sub_mem,a_single.max(b_single),target_w
        );
    }

    let (lo,hi)=fresh23_wilson95(full,80);
    println!(
        "FRESH_G23_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} combo_scores={:?} promoted={}/10 and={}/5 xor={}/5 memoryless={}/80 single={}/80 structure_violations={} leakage_violations={} atom_effect_violations={} wrong_operator_promotions={} irrelevant_promotions={} lesion={}/10 phase={}/10 restore={}/10 unrelated={}/10 restart={}/10 fp_violations={} legacy_violations={} motor_mask={:#08b}",
        full,lo,hi,per_seed,combo_scores,promoted,and_promoted,xor_promoted,
        memoryless,single,structure_violations,leakage_violations,
        atom_effect_violations,wrong_operator_promotions,irrelevant_promotions,
        lesion,phase,restore,unrelated,restart,fp_violations,legacy_violations,
        motor_mask
    );

    assert_eq!(promoted,10);
    assert_eq!(and_promoted,5);
    assert_eq!(xor_promoted,5);
    assert_eq!(structure_violations,0);
    assert_eq!(leakage_violations,0);
    assert_eq!(atom_effect_violations,0);
    assert_eq!(wrong_operator_promotions,0);
    assert_eq!(irrelevant_promotions,0);
    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|x|*x>=7));
    assert!(combo_scores.iter().all(|x|*x>=19));
    assert!(memoryless<=44);
    assert!(single<=44);
    assert!(lesion<=1);
    assert!(phase<=1);
    assert!(restore>=9);
    assert!(unrelated>=9);
    assert!(restart>=9);
    assert_eq!(fp_violations,0);
    assert_eq!(legacy_violations,0);
    assert_eq!(motor_mask,0b11_1111);
}


#[test]
#[ignore="requires independent one-use FRESH-G23-2 CI authority"]
fn g23_fresh2_compositional_perceptual_pack(){
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required").parse().unwrap();
    let source_sha=std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_|"unknown".into());
    let (specs,digest)=fresh23_specs(authority);

    println!(
        "FRESH_G23_2_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (i,s) in specs.iter().enumerate(){println!("FRESH_G23_2_BLOCK sub={} {:?}",i,s);}

    let drive=fixture::drive();
    let mut full=0usize;
    let mut per_seed=Vec::new();
    let mut combo_scores=[0usize;4];
    let mut memoryless=0usize;
    let mut single=0usize;
    let mut and_single=0usize;
    let mut xor_single=0usize;
    let mut promoted=0usize;
    let mut and_promoted=0usize;
    let mut xor_promoted=0usize;
    let mut structure_violations=0usize;
    let mut leakage_violations=0usize;
    let mut atom_effect_violations=0usize;
    let mut wrong_operator_promotions=0usize;
    let mut irrelevant_promotions=0usize;
    let mut lesion=0usize;
    let mut phase=0usize;
    let mut restore=0usize;
    let mut unrelated=0usize;
    let mut restart=0usize;
    let mut fp_violations=0usize;
    let mut legacy_violations=0usize;
    let mut motor_mask=0u8;

    for (sub,spec) in specs.iter().enumerate(){
        motor_mask|=1u8<<spec.motors[0];
        motor_mask|=1u8<<spec.motors[1];

        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        assert!(evo.enable_phase_native_compositional_refinement());
        let x=spec.motors[0];let y=spec.motors[1];
        let base=spec.states[0];let goal_state=spec.states[1];let dead_state=spec.states[2];

        let d10=Fresh23Sample{combo:2,nuisance:false,layout:spec.layouts[0],pattern:0};
        let d11=Fresh23Sample{combo:3,nuisance:false,layout:spec.layouts[1],pattern:1};
        let s10=fresh23_raw(&l1,spec,base,d10);
        let s11=fresh23_raw(&l1,spec,base,d11);
        let inherited=fixture::scene(&l1,base,spec.layouts[0] as usize,&[]);
        if evo.phase_native_abstract_state(&s10)!=evo.phase_native_abstract_state(&inherited)
            || evo.phase_native_abstract_state(&s11)!=evo.phase_native_abstract_state(&inherited)
            || evo.active_concept_atom_ids(&s10)!=evo.active_concept_atom_ids(&inherited)
            || evo.active_concept_atom_ids(&s11)!=evo.active_concept_atom_ids(&inherited)
        {structure_violations+=1;}

        let d10_good=fresh23_truth(spec.op,true,false);
        fact(&mut evo,&s10,x,
            &fixture::scene(&l1,if d10_good{goal_state}else{dead_state},
                            spec.layouts[0] as usize,&[]));
        fact(&mut evo,&s11,x,
            &fixture::scene(&l1,if fresh23_truth(spec.op,true,true){goal_state}else{dead_state},
                            spec.layouts[1] as usize,&[]));

        let target=fresh23_program(spec.op,spec.bins[0],spec.bins[1]);
        let wrong=fresh23_wrong_program(spec.op,spec.bins[0],spec.bins[1]);
        let born=evo.phase_native_composition_witnesses();
        let Some(target_born)=born.iter().find(|w|w.program==target) else{
            structure_violations+=1;continue;
        };
        if target_born.eligible_observations!=0{leakage_violations+=1;}

        let mut stats=TrainingStats::default();
        for sample in &spec.train{
            let a=sample.combo&2!=0;let b=sample.combo&1!=0;
            let x_good=fresh23_truth(spec.op,a,b);
            stats.a.observe(a,x_good);stats.b.observe(b,x_good);
            let pre=fresh23_raw(&l1,spec,base,*sample);
            let post_x=fixture::scene(
                &l1,if x_good{goal_state}else{dead_state},
                sample.layout as usize,&[]);
            let post_y=fixture::scene(
                &l1,if x_good{dead_state}else{goal_state},
                sample.layout as usize,&[]);
            fact(&mut evo,&pre,x,&post_x);
            fact(&mut evo,&pre,y,&post_y);
        }

        // Irrelevant candidate pool on a separate base.
        let ir0=Fresh23Irrelevant{combo:2,good:true,layout:spec.layouts[0],pattern:0};
        let ir1=Fresh23Irrelevant{combo:3,good:false,layout:spec.layouts[1],pattern:1};
        fact(&mut evo,&fresh23_irrelevant_raw(&l1,spec,ir0),x,
             &fixture::scene(&l1,goal_state,spec.layouts[0] as usize,&[]));
        fact(&mut evo,&fresh23_irrelevant_raw(&l1,spec,ir1),x,
             &fixture::scene(&l1,dead_state,spec.layouts[1] as usize,&[]));
        for sample in &spec.irrelevant{
            let pre=fresh23_irrelevant_raw(&l1,spec,*sample);
            let post=fixture::scene(
                &l1,if sample.good{goal_state}else{dead_state},
                sample.layout as usize,&[]);
            fact(&mut evo,&pre,x,&post);
        }

        let ws=evo.phase_native_composition_witnesses();
        let base_cell=evo.phase_native_abstract_state(&inherited).unwrap().cell;
        let target_w=ws.iter().find(|w|w.base_cell==base_cell&&w.program==target);
        if let Some(w)=target_w{
            if w.promoted{
                promoted+=1;
                match spec.op{Fresh23Op::And=>and_promoted+=1,Fresh23Op::Xor=>xor_promoted+=1}
            }
            if matches!(w.program,PhasePerceptProgram::Atom(_)){structure_violations+=1;}
            if w.atom_effects.iter().any(|e|*e>0.25+1e-12){atom_effect_violations+=1;}
        }else{structure_violations+=1;}

        if ws.iter().any(|w|w.base_cell==base_cell&&w.program==wrong&&w.promoted){
            wrong_operator_promotions+=1;
        }
        let irr_plain=fixture::scene(&l1,spec.states[3],spec.layouts[0] as usize,&[]);
        let irr_cell=evo.phase_native_abstract_state(&irr_plain).unwrap().cell;
        let irr_ws=ws.iter().filter(|w|w.base_cell==irr_cell).collect::<Vec<_>>();
        if irr_ws.iter().any(|w|w.promoted){irrelevant_promotions+=1;}

        evo.set_planning_learning_enabled(false);
        let mut sub_full=0usize;
        let mut sub_mem=0usize;
        let mut a_single=0usize;
        let mut b_single=0usize;
        for sample in &spec.score{
            let a=sample.combo&2!=0;let b=sample.combo&1!=0;
            let expected=if fresh23_truth(spec.op,a,b){x}else{y};
            let current=fresh23_raw(&l1,spec,base,*sample);
            let goal_raw=fixture::scene(&l1,goal_state,sample.layout as usize,&[]);
            evo.observe_initial_real(&current,false);
            let fp=evo.phase_native_learned_fingerprint();
            let got=evo.phase_native_compositional_action(&goal_raw);
            let ok=got==(true,Some(expected));
            full+=usize::from(ok);sub_full+=usize::from(ok);
            combo_scores[sample.combo as usize]+=usize::from(ok);
            if evo.phase_native_learned_fingerprint()!=fp{fp_violations+=1;}

            let mut old=evo.clone();
            let m=old.plan_phase_native_abstract_goal(&current,&goal_raw,None)
                .map(|d|d.first_action)==Some(expected);
            memoryless+=usize::from(m);sub_mem+=usize::from(m);

            let pa=stats.a.predict_goal(a);let pb=stats.b.predict_goal(b);
            a_single+=usize::from((if pa{x}else{y})==expected);
            b_single+=usize::from((if pb{x}else{y})==expected);
        }
        let sub_single=a_single.max(b_single);
        single+=sub_single;
        match spec.op {
            Fresh23Op::And => and_single+=sub_single,
            Fresh23Op::Xor => xor_single+=sub_single,
        }
        per_seed.push(sub_full);

        // Targeted physical interventions use a held-out sample where P=true.
        let true_sample=*spec.score.iter().find(|s|{
            let a=s.combo&2!=0;let b=s.combo&1!=0;
            fresh23_truth(spec.op,a,b)
        }).expect("true-side heldout sample");
        let current=fresh23_raw(&l1,spec,base,true_sample);
        let goal_raw=fixture::scene(&l1,goal_state,true_sample.layout as usize,&[]);
        let w=evo.phase_native_composition_witnesses().into_iter()
            .find(|w|w.base_cell==base_cell&&w.program==target&&w.promoted)
            .expect("fresh promoted target");
        let link=w.input_synapses[1][1];

        let mut damaged=evo.clone();
        damaged.observe_initial_real(&current,false);
        let saved=damaged.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
        lesion+=usize::from(
            damaged.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );
        damaged.restore_phase_native_synapse_for_control(link,saved);
        restore+=usize::from(
            damaged.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        let mut shifted=evo.clone();
        shifted.observe_initial_real(&current,false);
        shifted.perturb_phase_native_synapse_for_control(
            link,1.0,std::f32::consts::PI).unwrap();
        phase+=usize::from(
            shifted.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        if let Some(iw)=irr_ws.first(){
            let mut ir=evo.clone();
            ir.observe_initial_real(&current,false);
            ir.perturb_phase_native_synapse_for_control(
                iw.input_synapses[0][1],0.0,0.0).unwrap();
            unrelated+=usize::from(
                ir.phase_native_compositional_action(&goal_raw)==(true,Some(x))
            );
        }

        let checkpoint=evo.phase_native_checkpoint().unwrap();
        let mut restarted=EvoPhase::new(fixture::cfg(&evo));
        assert!(restarted.restore_phase_native_checkpoint(checkpoint));
        restarted.observe_initial_real(&current,false);
        restart+=usize::from(
            restarted.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        if evo.planning_transition_count()!=0||!evo.composite_concepts().is_empty(){
            legacy_violations+=1;
        }

        println!(
            "FRESH_G23_2_SUB sub={} op={:?} full={}/8 memoryless={}/8 single={}/8 target={:?}",
            sub,spec.op,sub_full,sub_mem,a_single.max(b_single),target_w
        );
    }

    let (lo,hi)=fresh23_wilson95(full,80);
    println!(
        "FRESH_G23_2_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} combo_scores={:?} promoted={}/10 and={}/5 xor={}/5 memoryless={}/80 single={}/80 and_single={}/40 xor_single={}/40 structure_violations={} leakage_violations={} atom_effect_violations={} wrong_operator_promotions={} irrelevant_promotions={} lesion={}/10 phase={}/10 restore={}/10 unrelated={}/10 restart={}/10 fp_violations={} legacy_violations={} motor_mask={:#08b}",
        full,lo,hi,per_seed,combo_scores,promoted,and_promoted,xor_promoted,
        memoryless,single,and_single,xor_single,structure_violations,leakage_violations,
        atom_effect_violations,wrong_operator_promotions,irrelevant_promotions,
        lesion,phase,restore,unrelated,restart,fp_violations,legacy_violations,
        motor_mask
    );

    assert_eq!(promoted,10);
    assert_eq!(and_promoted,5);
    assert_eq!(xor_promoted,5);
    assert_eq!(structure_violations,0);
    assert_eq!(leakage_violations,0);
    assert_eq!(atom_effect_violations,0);
    assert_eq!(wrong_operator_promotions,0);
    assert_eq!(irrelevant_promotions,0);
    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|x|*x>=7));
    assert!(combo_scores.iter().all(|x|*x>=19));
    assert!(memoryless<=44);
    assert!(and_single<=30);
    assert!(xor_single<=20);
    assert!(single<=50);
    assert!(lesion<=1);
    assert!(phase<=1);
    assert!(restore>=9);
    assert!(unrelated>=9);
    assert!(restart>=9);
    assert_eq!(fp_violations,0);
    assert_eq!(legacy_violations,0);
    assert_eq!(motor_mask,0b11_1111);
}


#[test]
#[ignore="requires independent one-use FRESH-G23-3 CI authority"]
fn g23_fresh3_compositional_perceptual_pack(){
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required").parse().unwrap();
    let source_sha=std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_|"unknown".into());
    // Build inherited representation before sealing so marker safety is a
    // geometry-only invariant and an invalid pool consumes no authority pack.
    let drive=fixture::drive();
    let mut geometry=fixture::organism(&drive);
    let geometry_l1=fixture::train_rep(&mut geometry);
    let (specs,digest)=fresh23_specs_safe(authority,&geometry_l1);

    println!(
        "FRESH_G23_3_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (i,s) in specs.iter().enumerate(){println!("FRESH_G23_3_BLOCK sub={} {:?}",i,s);}

    let mut full=0usize;
    let mut per_seed=Vec::new();
    let mut combo_scores=[0usize;4];
    let mut memoryless=0usize;
    let mut single=0usize;
    let mut and_single=0usize;
    let mut xor_single=0usize;
    let mut promoted=0usize;
    let mut and_promoted=0usize;
    let mut xor_promoted=0usize;
    let mut structure_violations=0usize;
    let mut leakage_violations=0usize;
    let mut atom_effect_violations=0usize;
    let mut wrong_operator_promotions=0usize;
    let mut irrelevant_promotions=0usize;
    let mut lesion=0usize;
    let mut phase=0usize;
    let mut restore=0usize;
    let mut unrelated=0usize;
    let mut restart=0usize;
    let mut fp_violations=0usize;
    let mut legacy_violations=0usize;
    let mut motor_mask=0u8;

    for (sub,spec) in specs.iter().enumerate(){
        motor_mask|=1u8<<spec.motors[0];
        motor_mask|=1u8<<spec.motors[1];

        let mut evo=fixture::organism(&drive);
        let l1=fixture::train_rep(&mut evo);
        assert!(evo.enable_phase_native_compositional_refinement());
        let x=spec.motors[0];let y=spec.motors[1];
        let base=spec.states[0];let goal_state=spec.states[1];let dead_state=spec.states[2];

        let d10=Fresh23Sample{combo:2,nuisance:false,layout:spec.layouts[0],pattern:0};
        let d11=Fresh23Sample{combo:3,nuisance:false,layout:spec.layouts[1],pattern:1};
        let s10=fresh23_raw(&l1,spec,base,d10);
        let s11=fresh23_raw(&l1,spec,base,d11);
        let inherited=fixture::scene(&l1,base,spec.layouts[0] as usize,&[]);
        if evo.phase_native_abstract_state(&s10)!=evo.phase_native_abstract_state(&inherited)
            || evo.phase_native_abstract_state(&s11)!=evo.phase_native_abstract_state(&inherited)
            || evo.active_concept_atom_ids(&s10)!=evo.active_concept_atom_ids(&inherited)
            || evo.active_concept_atom_ids(&s11)!=evo.active_concept_atom_ids(&inherited)
        {structure_violations+=1;}

        let d10_good=fresh23_truth(spec.op,true,false);
        fact(&mut evo,&s10,x,
            &fixture::scene(&l1,if d10_good{goal_state}else{dead_state},
                            spec.layouts[0] as usize,&[]));
        fact(&mut evo,&s11,x,
            &fixture::scene(&l1,if fresh23_truth(spec.op,true,true){goal_state}else{dead_state},
                            spec.layouts[1] as usize,&[]));

        let target=fresh23_program(spec.op,spec.bins[0],spec.bins[1]);
        let wrong=fresh23_wrong_program(spec.op,spec.bins[0],spec.bins[1]);
        let born=evo.phase_native_composition_witnesses();
        let Some(target_born)=born.iter().find(|w|w.program==target) else{
            structure_violations+=1;continue;
        };
        if target_born.eligible_observations!=0{leakage_violations+=1;}

        let mut stats=TrainingStats::default();
        for sample in &spec.train{
            let a=sample.combo&2!=0;let b=sample.combo&1!=0;
            let x_good=fresh23_truth(spec.op,a,b);
            stats.a.observe(a,x_good);stats.b.observe(b,x_good);
            let pre=fresh23_raw(&l1,spec,base,*sample);
            let post_x=fixture::scene(
                &l1,if x_good{goal_state}else{dead_state},
                sample.layout as usize,&[]);
            let post_y=fixture::scene(
                &l1,if x_good{dead_state}else{goal_state},
                sample.layout as usize,&[]);
            fact(&mut evo,&pre,x,&post_x);
            fact(&mut evo,&pre,y,&post_y);
        }

        // Irrelevant candidate pool on a separate base.
        let ir0=Fresh23Irrelevant{combo:2,good:true,layout:spec.layouts[0],pattern:0};
        let ir1=Fresh23Irrelevant{combo:3,good:false,layout:spec.layouts[1],pattern:1};
        fact(&mut evo,&fresh23_irrelevant_raw(&l1,spec,ir0),x,
             &fixture::scene(&l1,goal_state,spec.layouts[0] as usize,&[]));
        fact(&mut evo,&fresh23_irrelevant_raw(&l1,spec,ir1),x,
             &fixture::scene(&l1,dead_state,spec.layouts[1] as usize,&[]));
        for sample in &spec.irrelevant{
            let pre=fresh23_irrelevant_raw(&l1,spec,*sample);
            let post=fixture::scene(
                &l1,if sample.good{goal_state}else{dead_state},
                sample.layout as usize,&[]);
            fact(&mut evo,&pre,x,&post);
        }

        let ws=evo.phase_native_composition_witnesses();
        let base_cell=evo.phase_native_abstract_state(&inherited).unwrap().cell;
        let target_w=ws.iter().find(|w|w.base_cell==base_cell&&w.program==target);
        if let Some(w)=target_w{
            if w.promoted{
                promoted+=1;
                match spec.op{Fresh23Op::And=>and_promoted+=1,Fresh23Op::Xor=>xor_promoted+=1}
            }
            if matches!(w.program,PhasePerceptProgram::Atom(_)){structure_violations+=1;}
            if w.atom_effects.iter().any(|e|*e>0.25+1e-12){atom_effect_violations+=1;}
        }else{structure_violations+=1;}

        if ws.iter().any(|w|w.base_cell==base_cell&&w.program==wrong&&w.promoted){
            wrong_operator_promotions+=1;
        }
        let irr_plain=fixture::scene(&l1,spec.states[3],spec.layouts[0] as usize,&[]);
        let irr_cell=evo.phase_native_abstract_state(&irr_plain).unwrap().cell;
        let irr_ws=ws.iter().filter(|w|w.base_cell==irr_cell).collect::<Vec<_>>();
        if irr_ws.iter().any(|w|w.promoted){irrelevant_promotions+=1;}

        evo.set_planning_learning_enabled(false);
        let mut sub_full=0usize;
        let mut sub_mem=0usize;
        let mut a_single=0usize;
        let mut b_single=0usize;
        for sample in &spec.score{
            let a=sample.combo&2!=0;let b=sample.combo&1!=0;
            let expected=if fresh23_truth(spec.op,a,b){x}else{y};
            let current=fresh23_raw(&l1,spec,base,*sample);
            let goal_raw=fixture::scene(&l1,goal_state,sample.layout as usize,&[]);
            evo.observe_initial_real(&current,false);
            let fp=evo.phase_native_learned_fingerprint();
            let got=evo.phase_native_compositional_action(&goal_raw);
            let ok=got==(true,Some(expected));
            full+=usize::from(ok);sub_full+=usize::from(ok);
            combo_scores[sample.combo as usize]+=usize::from(ok);
            if evo.phase_native_learned_fingerprint()!=fp{fp_violations+=1;}

            let mut old=evo.clone();
            let m=old.plan_phase_native_abstract_goal(&current,&goal_raw,None)
                .map(|d|d.first_action)==Some(expected);
            memoryless+=usize::from(m);sub_mem+=usize::from(m);

            let pa=stats.a.predict_goal(a);let pb=stats.b.predict_goal(b);
            a_single+=usize::from((if pa{x}else{y})==expected);
            b_single+=usize::from((if pb{x}else{y})==expected);
        }
        let sub_single=a_single.max(b_single);
        single+=sub_single;
        match spec.op {
            Fresh23Op::And => and_single+=sub_single,
            Fresh23Op::Xor => xor_single+=sub_single,
        }
        per_seed.push(sub_full);

        // Targeted physical interventions use a held-out sample where P=true.
        let true_sample=*spec.score.iter().find(|s|{
            let a=s.combo&2!=0;let b=s.combo&1!=0;
            fresh23_truth(spec.op,a,b)
        }).expect("true-side heldout sample");
        let current=fresh23_raw(&l1,spec,base,true_sample);
        let goal_raw=fixture::scene(&l1,goal_state,true_sample.layout as usize,&[]);
        let w=evo.phase_native_composition_witnesses().into_iter()
            .find(|w|w.base_cell==base_cell&&w.program==target&&w.promoted)
            .expect("fresh promoted target");
        let link=w.input_synapses[1][1];

        let mut damaged=evo.clone();
        damaged.observe_initial_real(&current,false);
        let saved=damaged.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
        lesion+=usize::from(
            damaged.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );
        damaged.restore_phase_native_synapse_for_control(link,saved);
        restore+=usize::from(
            damaged.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        let mut shifted=evo.clone();
        shifted.observe_initial_real(&current,false);
        shifted.perturb_phase_native_synapse_for_control(
            link,1.0,std::f32::consts::PI).unwrap();
        phase+=usize::from(
            shifted.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        if let Some(iw)=irr_ws.first(){
            let mut ir=evo.clone();
            ir.observe_initial_real(&current,false);
            ir.perturb_phase_native_synapse_for_control(
                iw.input_synapses[0][1],0.0,0.0).unwrap();
            unrelated+=usize::from(
                ir.phase_native_compositional_action(&goal_raw)==(true,Some(x))
            );
        }

        let checkpoint=evo.phase_native_checkpoint().unwrap();
        let mut restarted=EvoPhase::new(fixture::cfg(&evo));
        assert!(restarted.restore_phase_native_checkpoint(checkpoint));
        restarted.observe_initial_real(&current,false);
        restart+=usize::from(
            restarted.phase_native_compositional_action(&goal_raw)==(true,Some(x))
        );

        if evo.planning_transition_count()!=0||!evo.composite_concepts().is_empty(){
            legacy_violations+=1;
        }

        println!(
            "FRESH_G23_3_SUB sub={} op={:?} full={}/8 memoryless={}/8 single={}/8 target={:?}",
            sub,spec.op,sub_full,sub_mem,a_single.max(b_single),target_w
        );
    }

    let (lo,hi)=fresh23_wilson95(full,80);
    println!(
        "FRESH_G23_3_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} combo_scores={:?} promoted={}/10 and={}/5 xor={}/5 memoryless={}/80 single={}/80 and_single={}/40 xor_single={}/40 structure_violations={} leakage_violations={} atom_effect_violations={} wrong_operator_promotions={} irrelevant_promotions={} lesion={}/10 phase={}/10 restore={}/10 unrelated={}/10 restart={}/10 fp_violations={} legacy_violations={} motor_mask={:#08b}",
        full,lo,hi,per_seed,combo_scores,promoted,and_promoted,xor_promoted,
        memoryless,single,and_single,xor_single,structure_violations,leakage_violations,
        atom_effect_violations,wrong_operator_promotions,irrelevant_promotions,
        lesion,phase,restore,unrelated,restart,fp_violations,legacy_violations,
        motor_mask
    );

    assert_eq!(promoted,10);
    assert_eq!(and_promoted,5);
    assert_eq!(xor_promoted,5);
    assert_eq!(structure_violations,0);
    assert_eq!(leakage_violations,0);
    assert_eq!(atom_effect_violations,0);
    assert_eq!(wrong_operator_promotions,0);
    assert_eq!(irrelevant_promotions,0);
    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|x|*x>=7));
    assert!(combo_scores.iter().all(|x|*x>=19));
    assert!(memoryless<=44);
    assert!(and_single<=30);
    assert!(xor_single<=20);
    assert!(single<=50);
    assert!(lesion<=1);
    assert!(phase<=1);
    assert!(restore>=9);
    assert!(unrelated>=9);
    assert!(restart>=9);
    assert_eq!(fp_violations,0);
    assert_eq!(legacy_violations,0);
    assert_eq!(motor_mask,0b11_1111);
}
