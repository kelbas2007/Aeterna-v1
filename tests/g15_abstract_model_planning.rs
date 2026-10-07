use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase};
use aeterna_v1::carrier::{
    PhaseAbstractStateRef, PhaseCircuitInfo, PhaseDeepNodeInfo, PhaseNativeConfig,
};

const W: usize = 20;
const H: usize = 20;

const OFFSETS: [(usize, usize); 16] = [
    (1,0),(0,1),(1,1),(2,0),
    (0,2),(2,1),(1,2),(2,2),
    (3,0),(0,3),(3,1),(1,3),
    (3,2),(2,3),(3,3),(4,0),
];

const STAGE1_BINDINGS: [((usize,usize),(usize,usize));4] = [
    ((1,1),(11,11)),
    ((2,1),(12,11)),
    ((1,2),(11,12)),
    ((2,2),(12,12)),
];

const NEGATIVE_BINDINGS: [((usize,usize),(usize,usize));4] = [
    ((1,10),(11,1)),
    ((2,10),(12,1)),
    ((1,11),(11,2)),
    ((2,11),(12,2)),
];

const L2_LAYOUTS: [[(usize,usize);4];4] = [
    [(1,1),(11,1),(1,11),(11,11)],
    [(2,1),(12,1),(2,11),(12,11)],
    [(1,2),(11,2),(1,12),(11,12)],
    [(2,2),(12,2),(2,12),(12,12)],
];

const TRANSITION_TUITION_LAYOUT: [(usize,usize);4] = [
    (3,3),(13,3),(3,13),(13,13),
];

const HELDOUT_LAYOUTS: [[(usize,usize);4];4] = L2_LAYOUTS;

const L2_TARGETS: [[usize;2];4] = [[0,1],[2,3],[4,5],[6,7]];

fn factorization_16() -> Vec<[[usize;2];8]> {
    let mut ring=(0usize..16).collect::<Vec<_>>();
    let mut rounds=Vec::new();
    for _ in 0..15 {
        let mut pairs=[[0usize;2];8];
        for i in 0..8 {
            let a=ring[i];
            let b=ring[15-i];
            pairs[i]=if a<b {[a,b]} else {[b,a]};
        }
        pairs.sort_unstable();
        rounds.push(pairs);
        let last=ring.pop().unwrap();
        ring.insert(1,last);
    }
    rounds
}

fn blank()->Vec<f32>{vec![0.0;W*H]}

fn add_motif(raster:&mut [f32], atom:usize, origin:(usize,usize)){
    let (x,y)=origin;
    let (dx,dy)=OFFSETS[atom];
    assert!(x+dx<W && y+dy<H);
    raster[y*W+x]=1.0;
    raster[(y+dy)*W+x+dx]=1.0;
}

fn pair_scene(pair:[usize;2],binding:((usize,usize),(usize,usize)))->Vec<f32>{
    let mut r=blank();
    add_motif(&mut r,pair[0],binding.0);
    add_motif(&mut r,pair[1],binding.1);
    r
}

fn concept_scene(
    l1_pairs:&[[usize;2];8],
    concepts:&[usize],
    origins:&[(usize,usize)],
)->Vec<f32>{
    assert_eq!(origins.len(),concepts.len()*2);
    let mut r=blank();
    let mut cursor=0usize;
    for &concept in concepts {
        for atom in l1_pairs[concept] {
            add_motif(&mut r,atom,origins[cursor]);
            cursor+=1;
        }
    }
    r
}

fn carrier()->EvoPhase{
    let cfg=EvoConfig {
        sensory_cells:W*H,
        motor_cells:6,
        dormant_cells:512,
        hdc_dim:192,
        weight_learning_rate:1.0,
        phase_learning_rate:1.0,
        min_recruit_support:1,
        ..EvoConfig::default()
    };
    let mut evo=EvoPhase::new(cfg);
    let mut concept=ConceptConfig::for_raster(W,H,6,192);
    concept.local_radius=4;
    concept.atom_match_threshold=0.97;
    concept.min_action_support=4;
    concept.min_composite_support=8;
    concept.child_predictiveness_ceiling=0.20;
    concept.composite_promotion_threshold=0.60;
    concept.readout_enabled=false;
    concept.learning_enabled=true;
    evo.enable_concept_memory(concept);
    evo.enable_phase_native_planning(PhaseNativeConfig {
        horizon:6,
        discount:0.95,
        ..PhaseNativeConfig::default()
    });
    assert!(evo.enable_phase_native_concepts());
    evo
}

fn train_l1(evo:&mut EvoPhase)->[[usize;2];8]{
    let factors=factorization_16();
    let targets=factors[0];

    for pair in targets {
        for binding in STAGE1_BINDINGS {
            let raster=pair_scene(pair,binding);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==0
                ));
            }
        }
    }

    for (round,binding) in factors[1..5].iter().zip(NEGATIVE_BINDINGS) {
        for pair in *round {
            let raster=pair_scene(pair,binding);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==1
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(),16);
    assert_eq!(evo.composite_concepts().len(),0);
    assert_eq!(evo.phase_native_promoted_concept_count(),8);
    targets
}

fn balanced_child_tuition(evo:&mut EvoPhase,sensory:&[f32],actions:[usize;2]){
    for rep in 0..4usize {
        let first_wins=rep%2==0;
        assert!(evo.observe_phase_native_depth_generic_factual(
            sensory,actions[0],first_wins
        ));
        assert!(evo.observe_phase_native_depth_generic_factual(
            sensory,actions[1],!first_wins
        ));
    }
}

fn l1_single_scene(
    l1_pairs:&[[usize;2];8],
    index:usize,
    binding:((usize,usize),(usize,usize)),
)->Vec<f32>{
    pair_scene(l1_pairs[index],binding)
}

fn l2_scene(
    l1_pairs:&[[usize;2];8],
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    let pair=L2_TARGETS[state];
    concept_scene(l1_pairs,&[pair[0],pair[1]],&layout)
}

fn train_l2_states(evo:&mut EvoPhase,l1_pairs:&[[usize;2];8]){
    assert!(evo.enable_phase_native_depth_generic_abstraction(2));

    for index in 0..8usize {
        let scene=l1_single_scene(l1_pairs,index,STAGE1_BINDINGS[index%4]);
        balanced_child_tuition(evo,&scene,[2,3]);
    }
    assert_eq!(evo.phase_native_deep_candidate_count(2),0);

    for cycle in 0..4usize {
        for state in 0..4usize {
            let scene=l2_scene(l1_pairs,state,L2_LAYOUTS[cycle]);
            let correct=if state%2==0 {2}else{3};
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                ));
            }
            for child in L2_TARGETS[state] {
                let single=l1_single_scene(
                    l1_pairs,child,STAGE1_BINDINGS[(cycle+child)%4]
                );
                let opposite=if correct==2 {3}else{2};
                for action in [2usize,3usize] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==opposite
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_deep_candidate_count(2),4);
    assert_eq!(evo.phase_native_promoted_deep_count(2),4);
}

fn build_abstract_base()->(EvoPhase,[[usize;2];8]){
    let mut evo=carrier();
    let l1_pairs=train_l1(&mut evo);
    train_l2_states(&mut evo,&l1_pairs);
    evo.set_concept_learning_enabled(false);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..4usize {
        let tuition=l2_scene(&l1_pairs,state,TRANSITION_TUITION_LAYOUT);
        let state_ref=evo.phase_native_abstract_state(&tuition)
            .expect("tuition binding must activate one abstract state");
        assert_eq!(state_ref.level,2);
        assert!(cells.insert(state_ref.cell));

        for layout in HELDOUT_LAYOUTS {
            let held=l2_scene(&l1_pairs,state,layout);
            let held_ref=evo.phase_native_abstract_state(&held)
                .expect("held-out binding must reuse the same abstract state");
            assert_eq!(held_ref,state_ref);
        }
    }
    assert_eq!(cells.len(),4);
    assert_eq!(evo.phase_native_circuits().len(),0);
    (evo,l1_pairs)
}

fn planning_actions(swap:bool)->(usize,usize){
    if swap {(5,4)} else {(4,5)}
}

fn learn_world(
    evo:&mut EvoPhase,
    l1_pairs:&[[usize;2];8],
    swap:bool,
){
    let (immediate,delayed)=planning_actions(swap);
    let s=(0..4usize)
        .map(|state|l2_scene(l1_pairs,state,TRANSITION_TUITION_LAYOUT))
        .collect::<Vec<_>>();

    let facts=[
        (0usize,immediate,3usize,0.55f32),
        (0,delayed,1,0.0),
        (1,immediate,2,0.0),
        (1,delayed,3,0.0),
        (2,immediate,3,0.0),
        (2,delayed,3,1.0),
        (3,immediate,3,0.0),
        (3,delayed,3,0.0),
    ];

    for (pre,action,post,value) in facts {
        assert!(evo.observe_phase_native_abstract_transition(
            &s[pre],action,&s[post],value
        ));
    }
    assert_eq!(evo.phase_native_circuits().len(),8);
}

fn state_ref(
    evo:&EvoPhase,
    l1_pairs:&[[usize;2];8],
    state:usize,
    layout:[(usize,usize);4],
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(&l2_scene(l1_pairs,state,layout))
        .expect("abstract state recognition")
}

fn transition_circuit(
    evo:&EvoPhase,
    from:PhaseAbstractStateRef,
    action:usize,
    to:PhaseAbstractStateRef,
)->PhaseCircuitInfo{
    let motor_cell=evo.config().sensory_cells+action;
    evo.phase_native_circuits()
        .iter()
        .find(|circuit|{
            let aff=evo.phase_native_synapse(circuit.afferent_synapse).unwrap();
            let succ=evo.phase_native_synapse(circuit.successor_synapse).unwrap();
            let motor=evo.phase_native_synapse(circuit.motor_synapse).unwrap();
            aff.from==from.cell && succ.to==to.cell && motor.to==motor_cell
        })
        .expect("physical abstract transition circuit")
        .clone()
}

fn l2_node_for_state(
    evo:&EvoPhase,
    state:PhaseAbstractStateRef,
)->PhaseDeepNodeInfo{
    evo.phase_native_deep_nodes()
        .iter()
        .find(|node|{
            node.promoted && node.level==state.level
                && node.id==state.id && node.concept_cell==state.cell
        })
        .expect("physical L2 state node")
        .clone()
}

fn score_full(
    evo:&mut EvoPhase,
    l1_pairs:&[[usize;2];8],
    swap:bool,
)->(usize,usize){
    let (_,delayed)=planning_actions(swap);
    let mut correct=0usize;
    let mut deep=0usize;

    for layout in HELDOUT_LAYOUTS {
        let sensory=l2_scene(l1_pairs,0,layout);
        evo.observe_initial_real(&sensory,false);
        let before_real=evo.current_real().unwrap().clone();
        let before_fingerprint=evo.phase_native_learned_fingerprint();

        let decision=evo.plan_phase_native_abstract(&sensory,None)
            .expect("FULL abstract planning decision");

        let after_real=evo.current_real().unwrap();
        let after_fingerprint=evo.phase_native_learned_fingerprint();

        assert_eq!(after_real.sensory,before_real.sensory);
        assert_eq!(after_real.need,before_real.need);
        assert_eq!(after_real.tick,before_real.tick);
        assert_eq!(after_fingerprint,before_fingerprint);

        correct+=usize::from(decision.first_action==delayed);
        deep+=usize::from(decision.selected_depth>=3);
    }
    (correct,deep)
}

fn score_depth1(
    evo:&mut EvoPhase,
    l1_pairs:&[[usize;2];8],
    swap:bool,
)->usize{
    let (_,delayed)=planning_actions(swap);
    HELDOUT_LAYOUTS.into_iter().map(|layout|{
        let sensory=l2_scene(l1_pairs,0,layout);
        usize::from(
            evo.plan_phase_native_abstract(&sensory,Some(1))
                .map(|d|d.first_action)==Some(delayed)
        )
    }).sum()
}

fn score_delayed(
    evo:&mut EvoPhase,
    l1_pairs:&[[usize;2];8],
    swap:bool,
)->usize{
    let (_,delayed)=planning_actions(swap);
    HELDOUT_LAYOUTS.into_iter().map(|layout|{
        let sensory=l2_scene(l1_pairs,0,layout);
        usize::from(
            evo.plan_phase_native_abstract(&sensory,None)
                .map(|d|d.first_action)==Some(delayed)
        )
    }).sum()
}

#[test]
fn g15_acquired_abstract_states_support_multistep_physical_planning(){
    let (abstract_base,l1_pairs)=build_abstract_base();

    let mut full_total=0usize;
    let mut depth3_total=0usize;
    let mut depth1_delayed=0usize;
    let mut no_model_delayed=0usize;
    let mut broken_state_delayed=0usize;
    let mut broken_transition_delayed=0usize;
    let mut phase_shift_delayed=0usize;
    let mut restored_total=0usize;
    let mut irrelevant_ok=0usize;

    for swap in [false,true] {
        let mut no_model=abstract_base.clone();
        no_model.set_planning_learning_enabled(false);
        no_model_delayed+=score_delayed(&mut no_model,&l1_pairs,swap);

        let mut full=abstract_base.clone();
        learn_world(&mut full,&l1_pairs,swap);

        let s0=state_ref(&full,&l1_pairs,0,TRANSITION_TUITION_LAYOUT);
        let s1=state_ref(&full,&l1_pairs,1,TRANSITION_TUITION_LAYOUT);
        let s3=state_ref(&full,&l1_pairs,3,TRANSITION_TUITION_LAYOUT);
        let (immediate,delayed)=planning_actions(swap);

        // Every learned P1 circuit must start/end on an acquired abstract cell.
        let abstract_cells=(0..4usize)
            .map(|state|state_ref(
                &full,&l1_pairs,state,TRANSITION_TUITION_LAYOUT
            ).cell)
            .collect::<std::collections::BTreeSet<_>>();
        for circuit in full.phase_native_circuits() {
            let aff=full.phase_native_synapse(circuit.afferent_synapse).unwrap();
            let succ=full.phase_native_synapse(circuit.successor_synapse).unwrap();
            assert!(abstract_cells.contains(&aff.from));
            assert!(abstract_cells.contains(&succ.to));
        }

        full.set_planning_learning_enabled(false);
        let (correct,deep)=score_full(&mut full,&l1_pairs,swap);
        full_total+=correct;
        depth3_total+=deep;

        let mut depth1=full.clone();
        depth1_delayed+=score_depth1(&mut depth1,&l1_pairs,swap);

        // Break the acquired physical L2 state itself.
        let node=l2_node_for_state(&full,s0);
        let mut broken_state=full.clone();
        broken_state
            .perturb_phase_native_synapse_for_control(
                node.child_synapses[0],0.0,0.0
            )
            .expect("necessary lower abstract-state synapse");
        broken_state_delayed+=score_delayed(
            &mut broken_state,&l1_pairs,swap
        );

        // Break the necessary abstract S0 --delayed--> S1 transition.
        let delayed_circuit=transition_circuit(&full,s0,delayed,s1);
        let mut broken_transition=full.clone();
        let saved=broken_transition
            .perturb_phase_native_synapse_for_control(
                delayed_circuit.successor_synapse,0.0,0.0
            )
            .expect("necessary abstract successor synapse");
        broken_transition_delayed+=score_delayed(
            &mut broken_transition,&l1_pairs,swap
        );

        broken_transition.restore_phase_native_synapse_for_control(
            delayed_circuit.successor_synapse,saved.clone()
        );
        restored_total+=score_delayed(
            &mut broken_transition,&l1_pairs,swap
        );

        let mut shifted=full.clone();
        shifted
            .perturb_phase_native_synapse_for_control(
                delayed_circuit.successor_synapse,
                1.0,
                std::f32::consts::PI,
            )
            .expect("necessary abstract phase synapse");
        phase_shift_delayed+=score_delayed(&mut shifted,&l1_pairs,swap);

        // Damage the immediate distractor branch, not the selected delayed path.
        let irrelevant=transition_circuit(&full,s0,immediate,s3);
        let mut irrelevant_lesion=full.clone();
        irrelevant_lesion
            .perturb_phase_native_synapse_for_control(
                irrelevant.successor_synapse,0.0,0.0
            )
            .expect("irrelevant abstract transition");
        let held=l2_scene(&l1_pairs,0,HELDOUT_LAYOUTS[0]);
        irrelevant_ok+=usize::from(
            irrelevant_lesion
                .plan_phase_native_abstract(&held,None)
                .map(|d|d.first_action)==Some(delayed)
        );
    }

    println!(
        "G15_ABSTRACT_PLAN full={}/8 depth_ge3={}/8 depth1_delayed={}/8 no_model={}/8 broken_state={}/8 broken_transition={}/8 phase_shift={}/8 restored={}/8 irrelevant={}/2",
        full_total,depth3_total,depth1_delayed,no_model_delayed,
        broken_state_delayed,broken_transition_delayed,phase_shift_delayed,
        restored_total,irrelevant_ok
    );

    assert_eq!(full_total,8);
    assert_eq!(depth3_total,8);
    assert!(depth1_delayed<=1);
    assert_eq!(no_model_delayed,0);
    assert!(broken_state_delayed<=6);
    assert!(broken_transition_delayed<=6);
    assert!(phase_shift_delayed<=6);
    assert_eq!(restored_total,8);
    assert_eq!(irrelevant_ok,2);
}

#[test]
fn g15_abstract_planner_has_no_graph_or_answer_fallback(){
    let source=include_str!("../src/phase_abstract_planning.rs");
    for forbidden in [
        "EvoImaginationPlanner",
        "HashMap",
        "BTreeMap",
        "VecDeque",
        "BinaryHeap",
        "S0",
        "S1",
        "S2",
        "S3",
        "correct_action",
        "route_list",
        "dijkstra",
    ] {
        assert!(
            !source.contains(forbidden),
            "G15 source contains forbidden token {forbidden}"
        );
    }
    for required in [
        "conductance",
        "native_cell_observation",
        "phase_native_decision_from_cell",
        "phase_native_abstract_state",
        "child_synapses",
        "self.synapses",
    ] {
        assert!(
            source.contains(required),
            "G15 physical dependency missing {required}"
        );
    }
}


#[derive(Clone, Debug)]
struct FreshG15Spec {
    selected_atoms: [usize;8],
    l1_pairs: [[usize;2];4],
    state_pairs: [[usize;2];6],
    motors: [usize;6],
    route_len: usize,
    immediate_milli: u32,
    route_actions: Vec<usize>,
    layouts: [[(usize,usize);4];9],
    transition_order: [usize;12],
}

struct FreshG15Rng(u64);

impl FreshG15Rng {
    fn new(seed:u64)->Self { Self(seed ^ 0xA815_7A6E_C011_2026) }
    fn next(&mut self)->u64{
        let mut x=self.0;
        x^=x>>12;
        x^=x<<25;
        x^=x>>27;
        self.0=x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self,upper:usize)->usize{(self.next()%upper as u64) as usize}
}

fn fresh_shuffle<T>(rng:&mut FreshG15Rng,values:&mut [T]){
    for i in (1..values.len()).rev(){
        let j=rng.range(i+1);
        values.swap(i,j);
    }
}

fn factorization_8()->Vec<[[usize;2];4]>{
    let mut ring=(0usize..8).collect::<Vec<_>>();
    let mut rounds=Vec::new();
    for _ in 0..7 {
        let mut pairs=[[0usize;2];4];
        for i in 0..4 {
            let a=ring[i];
            let b=ring[7-i];
            pairs[i]=if a<b {[a,b]} else {[b,a]};
        }
        pairs.sort_unstable();
        rounds.push(pairs);
        let last=ring.pop().unwrap();
        ring.insert(1,last);
    }
    rounds
}

fn fresh_layout_bank()->[[(usize,usize);4];9]{
    let mut out=[[(0usize,0usize);4];9];
    let mut index=0usize;
    for dy in 0..3usize {
        for dx in 0..3usize {
            out[index]=[
                (1+dx,1+dy),
                (11+dx,1+dy),
                (1+dx,11+dy),
                (11+dx,11+dy),
            ];
            index+=1;
        }
    }
    out
}

fn fresh_layout_valid(layout:&[(usize,usize);4],atoms:&[usize])->bool{
    let mut pixels=std::collections::BTreeSet::new();
    let mut per_motif=Vec::new();
    for (slot,&atom) in atoms.iter().enumerate() {
        let origin=layout[slot];
        let delta=OFFSETS[atom];
        let p0=origin;
        let p1=(origin.0+delta.0,origin.1+delta.1);
        if p1.0>=W || p1.1>=H || p0==p1 { return false; }
        if !pixels.insert(p0) || !pixels.insert(p1) { return false; }
        per_motif.push([p0,p1]);
    }
    for i in 0..per_motif.len() {
        for j in (i+1)..per_motif.len() {
            for a in per_motif[i] {
                for b in per_motif[j] {
                    if a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))<=4 {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn fresh_fnv(mut h:u64,value:u64)->u64{
    const PRIME:u64=1_099_511_628_211;
    for byte in value.to_le_bytes(){
        h^=byte as u64;
        h=h.wrapping_mul(PRIME);
    }
    h
}

fn fresh_g15_specs(authority:u64)->(Vec<FreshG15Spec>,u64,usize){
    let mut specs=Vec::new();
    let mut digest=14_695_981_039_346_656_037u64;
    let mut rejected=0usize;

    for sub in 0..10u64 {
        let mut rng=FreshG15Rng::new(
            authority
                ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ 0x6150_15AB_57AC_7001
        );

        let mut pool=(0usize..16).collect::<Vec<_>>();
        fresh_shuffle(&mut rng,&mut pool);
        let selected_atoms: [usize;8]=pool[..8].try_into().unwrap();

        let rounds=factorization_8();
        let l1_pairs=rounds[0].map(|p|{
            let mut pair=[selected_atoms[p[0]],selected_atoms[p[1]]];
            pair.sort_unstable();
            pair
        });

        let mut state_pairs=[
            [0usize,1usize],[0,2],[0,3],[1,2],[1,3],[2,3],
        ];
        fresh_shuffle(&mut rng,&mut state_pairs);

        let mut motors=[0usize,1,2,3,4,5];
        fresh_shuffle(&mut rng,&mut motors);

        let route_len=2+rng.range(3);
        let immediate_milli=500+rng.range(201) as u32;
        let mut route_actions=Vec::new();
        for _ in 0..route_len {
            route_actions.push(if rng.range(2)==0 { motors[4] } else { motors[5] });
        }

        let mut layouts=fresh_layout_bank();
        fresh_shuffle(&mut rng,&mut layouts);
        for layout in &layouts {
            if !fresh_layout_valid(layout,&[
                l1_pairs[0][0],l1_pairs[0][1],
                l1_pairs[1][0],l1_pairs[1][1],
            ]) {
                rejected+=1;
            }
        }

        let mut transition_order=[0usize;12];
        for (i,x) in transition_order.iter_mut().enumerate(){*x=i;}
        fresh_shuffle(&mut rng,&mut transition_order);

        let spec=FreshG15Spec {
            selected_atoms,l1_pairs,state_pairs,motors,route_len,
            immediate_milli,route_actions,layouts,transition_order,
        };

        digest=fresh_fnv(digest,sub);
        for value in spec.selected_atoms {digest=fresh_fnv(digest,value as u64);}
        for pair in spec.l1_pairs {for value in pair {digest=fresh_fnv(digest,value as u64);}}
        for pair in spec.state_pairs {for value in pair {digest=fresh_fnv(digest,value as u64);}}
        for value in spec.motors {digest=fresh_fnv(digest,value as u64);}
        digest=fresh_fnv(digest,spec.route_len as u64);
        digest=fresh_fnv(digest,spec.immediate_milli as u64);
        for value in &spec.route_actions {digest=fresh_fnv(digest,*value as u64);}
        for layout in spec.layouts {
            for (x,y) in layout {
                digest=fresh_fnv(digest,x as u64);
                digest=fresh_fnv(digest,y as u64);
            }
        }
        for value in spec.transition_order {digest=fresh_fnv(digest,value as u64);}
        specs.push(spec);
    }
    (specs,digest,rejected)
}

fn fresh_pair_scene(pair:[usize;2],layout:[(usize,usize);4])->Vec<f32>{
    pair_scene(pair,(layout[0],layout[1]))
}

fn fresh_state_scene(
    l1_pairs:&[[usize;2];4],
    state_pairs:&[[usize;2];6],
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    let pair=state_pairs[state];
    let mut raster=blank();
    let mut cursor=0usize;
    for l1 in pair {
        for atom in l1_pairs[l1] {
            add_motif(&mut raster,atom,layout[cursor]);
            cursor+=1;
        }
    }
    raster
}

fn fresh_carrier()->EvoPhase{
    carrier()
}

fn fresh_train_l1(evo:&mut EvoPhase,spec:&FreshG15Spec){
    let rounds=factorization_8();
    let good=spec.motors[0];
    let bad=spec.motors[1];

    for pair in spec.l1_pairs {
        for layout in spec.layouts[..4].iter().copied() {
            let raster=fresh_pair_scene(pair,layout);
            for action in [good,bad] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==good
                ));
            }
        }
    }

    for (round_index,round) in rounds[1..5].iter().enumerate() {
        let layout=spec.layouts[4+round_index];
        for p in *round {
            let mut pair=[
                spec.selected_atoms[p[0]],
                spec.selected_atoms[p[1]],
            ];
            pair.sort_unstable();
            let raster=fresh_pair_scene(pair,layout);
            for action in [good,bad] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==bad
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(),8);
    assert_eq!(evo.composite_concepts().len(),0);
    assert_eq!(evo.phase_native_promoted_concept_count(),4);
}

fn fresh_single_l1(
    spec:&FreshG15Spec,
    l1:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    fresh_pair_scene(spec.l1_pairs[l1],layout)
}

fn fresh_train_states(evo:&mut EvoPhase,spec:&FreshG15Spec){
    assert!(evo.enable_phase_native_depth_generic_abstraction(2));
    let a=spec.motors[2];
    let b=spec.motors[3];

    for l1 in 0..4usize {
        let scene=fresh_single_l1(spec,l1,spec.layouts[l1]);
        for rep in 0..4usize {
            let a_wins=rep%2==0;
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,a,a_wins
            ));
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,b,!a_wins
            ));
        }
    }
    assert_eq!(evo.phase_native_deep_candidate_count(2),0);

    for cycle in 0..4usize {
        for state in 0..6usize {
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,state,spec.layouts[cycle]
            );
            for action in [a,b] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==a
                ));
            }
            for child in spec.state_pairs[state] {
                let single=fresh_single_l1(
                    spec,child,spec.layouts[(cycle+child+4)%9]
                );
                for action in [a,b] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==b
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_deep_candidate_count(2),6);
    assert_eq!(evo.phase_native_promoted_deep_count(2),6);
}

fn fresh_build_base(spec:&FreshG15Spec)->EvoPhase{
    let mut evo=fresh_carrier();
    fresh_train_l1(&mut evo,spec);
    fresh_train_states(&mut evo,spec);
    evo.set_concept_learning_enabled(false);
    assert_eq!(evo.phase_native_circuits().len(),0);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..6usize {
        for layout in spec.layouts {
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,state,layout
            );
            let state_ref=evo.phase_native_abstract_state(&scene)
                .expect("fresh L2 state must be recognized");
            assert_eq!(state_ref.level,2);
            if layout==spec.layouts[0] {
                assert!(cells.insert(state_ref.cell));
            }
        }
    }
    assert_eq!(cells.len(),6);
    evo
}

#[derive(Clone,Copy)]
struct FreshFact {
    pre:usize,
    action:usize,
    post:usize,
    value:f32,
}

fn fresh_world_facts(spec:&FreshG15Spec)->[FreshFact;12]{
    let immediate=spec.motors[4];
    let delayed=spec.motors[5];
    let terminal=5usize;

    let mut facts=[FreshFact{pre:0,action:0,post:0,value:0.0};12];
    let mut cursor=0usize;
    for state in 0..6usize {
        for action in [immediate,delayed] {
            facts[cursor]=FreshFact {
                pre:state,action,post:terminal,value:0.0
            };
            cursor+=1;
        }
    }

    let set_fact=|facts:&mut [FreshFact;12],pre:usize,action:usize,post:usize,value:f32|{
        let slot=facts.iter_mut()
            .find(|fact|fact.pre==pre && fact.action==action)
            .expect("fresh transition slot");
        slot.post=post;
        slot.value=value;
    };

    set_fact(
        &mut facts,0,immediate,terminal,
        spec.immediate_milli as f32/1000.0
    );
    set_fact(&mut facts,0,delayed,1,0.0);

    for step in 1..=spec.route_len {
        let action=spec.route_actions[step-1];
        let post=step+1;
        let value=if step==spec.route_len {1.0}else{0.0};
        set_fact(&mut facts,step,action,post,value);
    }

    facts
}

fn fresh_learn_world(evo:&mut EvoPhase,spec:&FreshG15Spec){
    let facts=fresh_world_facts(spec);
    let tuition=spec.layouts[0];
    for index in spec.transition_order {
        let fact=facts[index];
        let pre=fresh_state_scene(
            &spec.l1_pairs,&spec.state_pairs,fact.pre,tuition
        );
        let post=fresh_state_scene(
            &spec.l1_pairs,&spec.state_pairs,fact.post,tuition
        );
        assert!(evo.observe_phase_native_abstract_transition(
            &pre,fact.action,&post,fact.value
        ));
    }
    assert_eq!(evo.phase_native_circuits().len(),12);
}

fn fresh_state_ref(
    evo:&EvoPhase,spec:&FreshG15Spec,state:usize
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(&fresh_state_scene(
        &spec.l1_pairs,&spec.state_pairs,state,spec.layouts[0]
    )).expect("fresh abstract state ref")
}

fn fresh_transition_circuit(
    evo:&EvoPhase,
    from:PhaseAbstractStateRef,
    action:usize,
    to:PhaseAbstractStateRef,
)->PhaseCircuitInfo{
    let motor_cell=evo.config().sensory_cells+action;
    evo.phase_native_circuits().iter()
        .find(|circuit|{
            let aff=evo.phase_native_synapse(circuit.afferent_synapse).unwrap();
            let succ=evo.phase_native_synapse(circuit.successor_synapse).unwrap();
            let motor=evo.phase_native_synapse(circuit.motor_synapse).unwrap();
            aff.from==from.cell && succ.to==to.cell && motor.to==motor_cell
        })
        .expect("fresh physical abstract transition")
        .clone()
}

fn fresh_l2_node(evo:&EvoPhase,state:PhaseAbstractStateRef)->PhaseDeepNodeInfo{
    evo.phase_native_deep_nodes().iter()
        .find(|node|node.promoted && node.level==2
            && node.id==state.id && node.concept_cell==state.cell)
        .expect("fresh physical L2 node")
        .clone()
}

fn fresh_wilson95(success:usize,n:usize)->(f64,f64){
    let z=1.959_963_984_540_054_f64;
    let n=n as f64;
    let p=success as f64/n;
    let denom=1.0+z*z/n;
    let center=(p+z*z/(2.0*n))/denom;
    let half=z*(p*(1.0-p)/n+z*z/(4.0*n*n)).sqrt()/denom;
    (center-half,center+half)
}

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from first-attempt CI"]
fn g15_fresh_abstract_model_planning_pack(){
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse().expect("fresh G15 seed must be u64");
    let source_sha=std::env::var("AETERNA_SOURCE_SHA")
        .unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA")
        .unwrap_or_else(|_|"unknown".into());

    let (specs,digest,layout_rejects)=fresh_g15_specs(authority);
    println!(
        "FRESH_G15_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x} layout_rejects={}",
        source_sha,spec_sha,authority,digest,layout_rejects
    );
    for (sub,spec) in specs.iter().enumerate(){
        println!("FRESH_G15_BLOCK sub={} {:?}",sub,spec);
    }

    let mut full=0usize;
    let mut per_seed=Vec::new();
    let mut depth_violations=0usize;
    let mut depth1=0usize;
    let mut no_model=0usize;
    let mut broken_state=0usize;
    let mut broken_transition=0usize;
    let mut phase_shift=0usize;
    let mut restored=0usize;
    let mut irrelevant=0usize;
    let mut structure_violations=0usize;
    let mut endpoint_violations=0usize;
    let mut real_mutations=0usize;
    let mut fingerprint_mutations=0usize;
    let mut legacy_violations=0usize;
    let mut motor_mask=0u8;

    for (sub,spec) in specs.iter().enumerate(){
        for motor in spec.motors { motor_mask|=1u8<<motor; }

        let base=fresh_build_base(spec);
        if base.concept_atoms().len()!=8
            || base.phase_native_promoted_concept_count()!=4
            || base.phase_native_promoted_deep_count(2)!=6
            || !base.composite_concepts().is_empty()
        {
            structure_violations+=1;
        }

        let l1=base.phase_native_concept_circuits().iter()
            .filter(|c|c.promoted)
            .map(|c|(c.concept_id,c.concept_cell))
            .collect::<std::collections::BTreeMap<_,_>>();
        for node in base.phase_native_deep_nodes().iter()
            .filter(|n|n.promoted && n.level==2)
        {
            if !node.children.iter().all(|child|
                child.level==1 && l1.get(&child.id)==Some(&child.cell)
            ){
                structure_violations+=1;
            }
        }

        let mut no_model_evo=base.clone();
        no_model_evo.set_planning_learning_enabled(false);
        for layout in spec.layouts[1..].iter().copied(){
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,0,layout
            );
            no_model+=usize::from(
                no_model_evo.plan_phase_native_abstract(&scene,None)
                    .map(|d|d.first_action)==Some(spec.motors[5])
            );
        }

        let mut evo=base.clone();
        fresh_learn_world(&mut evo,spec);

        let state_refs=(0..6usize)
            .map(|s|fresh_state_ref(&evo,spec,s))
            .collect::<Vec<_>>();
        let abstract_cells=state_refs.iter()
            .map(|r|r.cell)
            .collect::<std::collections::BTreeSet<_>>();
        for circuit in evo.phase_native_circuits(){
            let aff=evo.phase_native_synapse(circuit.afferent_synapse).unwrap();
            let succ=evo.phase_native_synapse(circuit.successor_synapse).unwrap();
            if !abstract_cells.contains(&aff.from)
                || !abstract_cells.contains(&succ.to)
            {
                endpoint_violations+=1;
            }
        }
        if evo.planning_transition_count()!=0 {legacy_violations+=1;}

        evo.set_planning_learning_enabled(false);

        let mut sub_full=0usize;
        for layout in spec.layouts[1..].iter().copied(){
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,0,layout
            );
            evo.observe_initial_real(&scene,false);
            let real_before=evo.current_real().unwrap().clone();
            let fp_before=evo.phase_native_learned_fingerprint();
            let decision=evo.plan_phase_native_abstract(&scene,None)
                .expect("fresh FULL abstract plan");
            let real_after=evo.current_real().unwrap();
            let fp_after=evo.phase_native_learned_fingerprint();

            if real_after.sensory!=real_before.sensory
                || real_after.need!=real_before.need
                || real_after.tick!=real_before.tick
            {real_mutations+=1;}
            if fp_after!=fp_before {fingerprint_mutations+=1;}

            let ok=decision.first_action==spec.motors[5];
            full+=usize::from(ok);
            sub_full+=usize::from(ok);
            if ok && decision.selected_depth<spec.route_len+1 {
                depth_violations+=1;
            }

            let mut shallow=evo.clone();
            depth1+=usize::from(
                shallow.plan_phase_native_abstract(&scene,Some(1))
                    .map(|d|d.first_action)==Some(spec.motors[5])
            );
        }
        per_seed.push(sub_full);

        let s0=state_refs[0];
        let s1=state_refs[1];

        let node=fresh_l2_node(&evo,s0);
        let mut broken_state_evo=evo.clone();
        broken_state_evo.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],0.0,0.0
        ).expect("fresh necessary state synapse");
        for layout in spec.layouts[1..3].iter().copied(){
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,0,layout
            );
            broken_state+=usize::from(
                broken_state_evo.plan_phase_native_abstract(&scene,None)
                    .map(|d|d.first_action)==Some(spec.motors[5])
            );
        }

        let delayed_circuit=fresh_transition_circuit(
            &evo,s0,spec.motors[5],s1
        );
        let mut broken=evo.clone();
        let saved=broken.perturb_phase_native_synapse_for_control(
            delayed_circuit.successor_synapse,0.0,0.0
        ).expect("fresh necessary abstract transition");
        for layout in spec.layouts[1..3].iter().copied(){
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,0,layout
            );
            broken_transition+=usize::from(
                broken.plan_phase_native_abstract(&scene,None)
                    .map(|d|d.first_action)==Some(spec.motors[5])
            );
        }

        broken.restore_phase_native_synapse_for_control(
            delayed_circuit.successor_synapse,saved.clone()
        );
        for layout in spec.layouts[1..3].iter().copied(){
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,0,layout
            );
            restored+=usize::from(
                broken.plan_phase_native_abstract(&scene,None)
                    .map(|d|d.first_action)==Some(spec.motors[5])
            );
        }

        let mut shifted=evo.clone();
        shifted.perturb_phase_native_synapse_for_control(
            delayed_circuit.successor_synapse,1.0,std::f32::consts::PI
        ).expect("fresh phase-shift transition");
        for layout in spec.layouts[1..3].iter().copied(){
            let scene=fresh_state_scene(
                &spec.l1_pairs,&spec.state_pairs,0,layout
            );
            phase_shift+=usize::from(
                shifted.plan_phase_native_abstract(&scene,None)
                    .map(|d|d.first_action)==Some(spec.motors[5])
            );
        }

        let immediate_circuit=fresh_transition_circuit(
            &evo,s0,spec.motors[4],state_refs[5]
        );
        let mut unrelated_evo=evo.clone();
        unrelated_evo.perturb_phase_native_synapse_for_control(
            immediate_circuit.successor_synapse,0.0,0.0
        ).expect("fresh irrelevant transition");
        let scene=fresh_state_scene(
            &spec.l1_pairs,&spec.state_pairs,0,spec.layouts[1]
        );
        irrelevant+=usize::from(
            unrelated_evo.plan_phase_native_abstract(&scene,None)
                .map(|d|d.first_action)==Some(spec.motors[5])
        );

        println!(
            "FRESH_G15_SUB sub={} full={}/8 route_len={} immediate={:.3} circuits={} route_actions={:?}",
            sub,sub_full,spec.route_len,
            spec.immediate_milli as f32/1000.0,
            evo.phase_native_circuits().len(),
            spec.route_actions
        );
    }

    let (lo,hi)=fresh_wilson95(full,80);
    println!(
        "FRESH_G15_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} depth_violations={} depth1={}/80 no_model={}/80 broken_state={}/20 broken_transition={}/20 phase_shift={}/20 restored={}/20 irrelevant={}/10 structure_violations={} endpoint_violations={} real_mutations={} fingerprint_mutations={} legacy_violations={} motor_mask={:#08b} layout_rejects={}",
        full,lo,hi,per_seed,depth_violations,depth1,no_model,
        broken_state,broken_transition,phase_shift,restored,irrelevant,
        structure_violations,endpoint_violations,real_mutations,
        fingerprint_mutations,legacy_violations,motor_mask,layout_rejects
    );

    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|score|*score>=6));
    assert_eq!(depth_violations,0);
    assert!(depth1<=4);
    assert_eq!(no_model,0);
    assert_eq!(structure_violations,0);
    assert_eq!(endpoint_violations,0);
    assert!(broken_state<=4);
    assert!(broken_transition<=4);
    assert!(phase_shift<=4);
    assert!(restored>=19);
    assert!(irrelevant>=9);
    assert_eq!(real_mutations,0);
    assert_eq!(fingerprint_mutations,0);
    assert_eq!(legacy_violations,0);
    assert_eq!(motor_mask,0b11_1111);
}
