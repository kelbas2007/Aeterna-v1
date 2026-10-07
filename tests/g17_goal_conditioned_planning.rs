use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase};
use aeterna_v1::carrier::{
    PhaseAbstractStateRef, PhaseCircuitInfo, PhaseDeepNodeInfo, PhaseNativeConfig,
};

const W: usize = 20;
const H: usize = 20;

const OFFSETS: [(usize, usize); 8] = [
    (1,0),(0,1),(1,1),(2,0),
    (0,2),(2,1),(1,2),(2,2),
];

const LAYOUTS: [[(usize,usize);4]; 6] = [
    [(1,1),(11,1),(1,11),(11,11)],
    [(2,1),(12,1),(2,11),(12,11)],
    [(1,2),(11,2),(1,12),(11,12)],
    [(2,2),(12,2),(2,12),(12,12)],
    [(3,1),(13,1),(3,11),(13,11)],
    [(1,3),(11,3),(1,13),(11,13)],
];

fn factorization_8() -> Vec<[[usize;2];4]> {
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

fn blank()->Vec<f32>{vec![0.0;W*H]}

fn add_motif(raster:&mut [f32],atom:usize,origin:(usize,usize)){
    let (x,y)=origin;
    let (dx,dy)=OFFSETS[atom];
    assert!(x+dx<W && y+dy<H);
    raster[y*W+x]=1.0;
    raster[(y+dy)*W+x+dx]=1.0;
}

fn pair_scene(pair:[usize;2],layout:[(usize,usize);4])->Vec<f32>{
    let mut r=blank();
    add_motif(&mut r,pair[0],layout[0]);
    add_motif(&mut r,pair[1],layout[1]);
    r
}

fn state_scene(
    l1_pairs:&[[usize;2];4],
    state_pairs:&[[usize;2];6],
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    let mut r=blank();
    let mut cursor=0usize;
    for l1 in state_pairs[state] {
        for atom in l1_pairs[l1] {
            add_motif(&mut r,atom,layout[cursor]);
            cursor+=1;
        }
    }
    r
}

fn carrier()->EvoPhase{
    let cfg=EvoConfig {
        sensory_cells:W*H,
        motor_cells:6,
        dormant_cells:384,
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

fn train_abstraction()->(EvoPhase,[[usize;2];4],[[usize;2];6]){
    let factors=factorization_8();
    let l1_pairs=factors[0];

    let mut evo=carrier();

    // Four useful L1 pairs at support 8.
    for pair in l1_pairs {
        for layout in LAYOUTS[..4].iter().copied() {
            let scene=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &scene,action,action==0
                ));
            }
        }
    }

    // Four balancing perfect matchings make each primitive child weak.
    for (index,round) in factors[1..5].iter().enumerate() {
        let layout=LAYOUTS[index];
        for pair in *round {
            let scene=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &scene,action,action==1
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(),8);
    assert_eq!(evo.phase_native_promoted_concept_count(),4);
    assert!(evo.composite_concepts().is_empty());

    assert!(evo.enable_phase_native_depth_generic_abstraction(2));

    // Establish supported weak evidence for every L1 on Stage-2 motors.
    for l1 in 0..4usize {
        let scene=pair_scene(l1_pairs[l1],LAYOUTS[l1]);
        for rep in 0..4usize {
            let first=rep%2==0;
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,2,first
            ));
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,3,!first
            ));
        }
    }
    assert_eq!(evo.phase_native_deep_candidate_count(2),0);

    let state_pairs=[
        [0usize,1usize],[0,2],[0,3],[1,2],[1,3],[2,3],
    ];

    // Promote all six L2 combinations while keeping individual L1 evidence weak.
    for cycle in 0..4usize {
        for state in 0..6usize {
            let scene=state_scene(&l1_pairs,&state_pairs,state,LAYOUTS[cycle]);
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==2
                ));
            }
            for child in state_pairs[state] {
                let single=pair_scene(
                    l1_pairs[child],
                    LAYOUTS[(cycle+child+2)%6],
                );
                for action in [2usize,3usize] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==3
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_promoted_deep_count(2),6);
    evo.set_concept_learning_enabled(false);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..6usize {
        let reference=evo.phase_native_abstract_state(
            &state_scene(&l1_pairs,&state_pairs,state,LAYOUTS[0])
        ).expect("G17 acquired L2 state");
        assert_eq!(reference.level,2);
        assert!(cells.insert(reference.cell));
        for layout in LAYOUTS {
            let held=evo.phase_native_abstract_state(
                &state_scene(&l1_pairs,&state_pairs,state,layout)
            ).expect("G17 held binding recognizes L2 state");
            assert_eq!(held,reference);
        }
    }
    assert_eq!(cells.len(),6);
    (evo,l1_pairs,state_pairs)
}

fn actions(swap:bool)->(usize,usize){
    if swap {(5,4)} else {(4,5)}
}

fn learn_zero_value_model(
    evo:&mut EvoPhase,
    l1_pairs:&[[usize;2];4],
    state_pairs:&[[usize;2];6],
    swap:bool,
){
    let (a,b)=actions(swap);
    let s=(0..6usize).map(|state|
        state_scene(l1_pairs,state_pairs,state,LAYOUTS[0])
    ).collect::<Vec<_>>();

    // Goal A = state 4, goal B = state 5.
    // From the same start S0 the first motor differs by goal.
    let facts=[
        (0usize,a,1usize),(0,b,2),
        (1,a,4),(1,b,3),
        (2,a,3),(2,b,5),
        (3,a,3),(3,b,3),
        (4,a,4),(4,b,4),
        (5,a,5),(5,b,5),
    ];

    for (pre,action,post) in facts {
        assert!(evo.observe_phase_native_abstract_transition(
            &s[pre],action,&s[post],0.0
        ));
    }
    assert_eq!(evo.phase_native_circuits().len(),12);

    // G17 model is goal-agnostic: factual outcome weights are zero.
    for circuit in evo.phase_native_circuits() {
        let outcome=evo.phase_native_synapse(circuit.outcome_synapse).unwrap();
        assert!(outcome.weight.abs()<=1.0e-7);
    }
}

fn state_ref(
    evo:&EvoPhase,
    l1_pairs:&[[usize;2];4],
    state_pairs:&[[usize;2];6],
    state:usize,
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(
        &state_scene(l1_pairs,state_pairs,state,LAYOUTS[0])
    ).expect("G17 state ref")
}

fn transition_circuit(
    evo:&EvoPhase,
    from:PhaseAbstractStateRef,
    action:usize,
    to:PhaseAbstractStateRef,
)->PhaseCircuitInfo{
    let motor=evo.config().sensory_cells+action;
    evo.phase_native_circuits().iter().find(|c|{
        let aff=evo.phase_native_synapse(c.afferent_synapse).unwrap();
        let succ=evo.phase_native_synapse(c.successor_synapse).unwrap();
        let out=evo.phase_native_synapse(c.motor_synapse).unwrap();
        aff.from==from.cell && succ.to==to.cell && out.to==motor
    }).expect("G17 physical transition").clone()
}

fn l2_node(evo:&EvoPhase,state:PhaseAbstractStateRef)->PhaseDeepNodeInfo{
    evo.phase_native_deep_nodes().iter().find(|n|
        n.promoted && n.level==2 && n.id==state.id && n.concept_cell==state.cell
    ).expect("G17 L2 node").clone()
}

fn expected_for_goal(swap:bool,goal:usize)->usize{
    let (a,b)=actions(swap);
    match goal {
        4=>a,
        5=>b,
        _=>panic!("development goal must be state 4 or 5"),
    }
}

fn score_goal(
    evo:&mut EvoPhase,
    l1_pairs:&[[usize;2];4],
    state_pairs:&[[usize;2];6],
    swap:bool,
    depth:Option<usize>,
)->usize{
    let mut score=0usize;
    for goal in [4usize,5usize] {
        let expected=expected_for_goal(swap,goal);
        for layout in [LAYOUTS[4],LAYOUTS[5]] {
            let current=state_scene(l1_pairs,state_pairs,0,layout);
            let goal_scene=state_scene(l1_pairs,state_pairs,goal,layout);
            evo.observe_initial_real(&current,false);
            let real_before=evo.current_real().unwrap().clone();
            let fp_before=evo.phase_native_learned_fingerprint();

            let decision=evo.plan_phase_native_abstract_goal(
                &current,&goal_scene,depth
            );

            let real_after=evo.current_real().unwrap();
            let fp_after=evo.phase_native_learned_fingerprint();
            assert_eq!(real_after.sensory,real_before.sensory);
            assert_eq!(real_after.need,real_before.need);
            assert_eq!(real_after.tick,real_before.tick);
            assert_eq!(fp_after,fp_before);

            score+=usize::from(
                decision.map(|d|d.first_action)==Some(expected)
            );
        }
    }
    score
}

#[test]
fn g17_same_physical_world_model_changes_action_when_raw_goal_changes(){
    let (base,l1_pairs,state_pairs)=train_abstraction();

    let mut full_total=0usize;
    let mut depth1_total=0usize;
    let mut no_goal_specific=0usize;
    let mut wrong_goal_follows_other=0usize;
    let mut broken_goal_total=0usize;
    let mut broken_route_total=0usize;
    let mut phase_total=0usize;
    let mut restored_total=0usize;
    let mut irrelevant_ok=0usize;

    for swap in [false,true] {
        let mut full=base.clone();
        learn_zero_value_model(&mut full,&l1_pairs,&state_pairs,swap);

        let cells=(0..6usize).map(|state|
            state_ref(&full,&l1_pairs,&state_pairs,state).cell
        ).collect::<std::collections::BTreeSet<_>>();
        for circuit in full.phase_native_circuits() {
            let aff=full.phase_native_synapse(circuit.afferent_synapse).unwrap();
            let succ=full.phase_native_synapse(circuit.successor_synapse).unwrap();
            assert!(cells.contains(&aff.from));
            assert!(cells.contains(&succ.to));
        }

        full.set_planning_learning_enabled(false);
        let baseline=score_goal(
            &mut full,&l1_pairs,&state_pairs,swap,None
        );
        assert_eq!(baseline,4);
        full_total+=baseline;

        let mut shallow=full.clone();
        depth1_total+=score_goal(
            &mut shallow,&l1_pairs,&state_pairs,swap,Some(1)
        );

        // No goal: ordinary value planner has only zero outcomes.
        for layout in [LAYOUTS[4],LAYOUTS[5]] {
            let current=state_scene(&l1_pairs,&state_pairs,0,layout);
            for goal in [4usize,5usize] {
                let expected=expected_for_goal(swap,goal);
                no_goal_specific+=usize::from(
                    full.plan_phase_native_abstract(&current,None)
                        .map(|d|d.first_action)==Some(expected)
                );
            }
        }

        // Supplying the opposite valid goal must select that other goal's motor.
        for requested in [4usize,5usize] {
            let other=if requested==4 {5}else{4};
            let other_expected=expected_for_goal(swap,other);
            for layout in [LAYOUTS[4],LAYOUTS[5]] {
                let current=state_scene(&l1_pairs,&state_pairs,0,layout);
                let wrong_goal=state_scene(
                    &l1_pairs,&state_pairs,other,layout
                );
                wrong_goal_follows_other+=usize::from(
                    full.plan_phase_native_abstract_goal(
                        &current,&wrong_goal,None
                    ).map(|d|d.first_action)==Some(other_expected)
                );
            }
        }

        let s0=state_ref(&full,&l1_pairs,&state_pairs,0);
        let s1=state_ref(&full,&l1_pairs,&state_pairs,1);
        let s2=state_ref(&full,&l1_pairs,&state_pairs,2);
        let g4=state_ref(&full,&l1_pairs,&state_pairs,4);

        // Break physical recognition beneath goal A.
        let goal_node=l2_node(&full,g4);
        let mut broken_goal=full.clone();
        broken_goal.perturb_phase_native_synapse_for_control(
            goal_node.child_synapses[0],0.0,0.0
        ).expect("G17 necessary goal-recognition synapse");
        for layout in [LAYOUTS[4],LAYOUTS[5]] {
            let current=state_scene(&l1_pairs,&state_pairs,0,layout);
            let goal=state_scene(&l1_pairs,&state_pairs,4,layout);
            broken_goal_total+=usize::from(
                broken_goal.plan_phase_native_abstract_goal(
                    &current,&goal,None
                ).map(|d|d.first_action)==Some(expected_for_goal(swap,4))
            );
        }

        // Break S0 -> S1, which is only needed for goal A.
        let (a,b)=actions(swap);
        let selected=transition_circuit(&full,s0,a,s1);
        let mut broken=full.clone();
        let saved=broken.perturb_phase_native_synapse_for_control(
            selected.successor_synapse,0.0,0.0
        ).expect("G17 necessary goal-route synapse");

        for layout in [LAYOUTS[4],LAYOUTS[5]] {
            let current=state_scene(&l1_pairs,&state_pairs,0,layout);
            let goal=state_scene(&l1_pairs,&state_pairs,4,layout);
            broken_route_total+=usize::from(
                broken.plan_phase_native_abstract_goal(
                    &current,&goal,None
                ).map(|d|d.first_action)==Some(a)
            );
        }

        broken.restore_phase_native_synapse_for_control(
            selected.successor_synapse,saved.clone()
        );
        restored_total+=score_goal(
            &mut broken,&l1_pairs,&state_pairs,swap,None
        );

        let mut shifted=full.clone();
        shifted.perturb_phase_native_synapse_for_control(
            selected.successor_synapse,1.0,std::f32::consts::PI
        ).expect("G17 phase intervention");
        for layout in [LAYOUTS[4],LAYOUTS[5]] {
            let current=state_scene(&l1_pairs,&state_pairs,0,layout);
            let goal=state_scene(&l1_pairs,&state_pairs,4,layout);
            phase_total+=usize::from(
                shifted.plan_phase_native_abstract_goal(
                    &current,&goal,None
                ).map(|d|d.first_action)==Some(a)
            );
        }

        // Damage only competing goal-B branch; goal A must remain correct.
        let competing=transition_circuit(&full,s0,b,s2);
        let mut irrelevant=full.clone();
        irrelevant.perturb_phase_native_synapse_for_control(
            competing.successor_synapse,0.0,0.0
        ).expect("G17 competing-goal transition");
        let current=state_scene(&l1_pairs,&state_pairs,0,LAYOUTS[4]);
        let goal=state_scene(&l1_pairs,&state_pairs,4,LAYOUTS[4]);
        irrelevant_ok+=usize::from(
            irrelevant.plan_phase_native_abstract_goal(
                &current,&goal,None
            ).map(|d|d.first_action)==Some(a)
        );
    }

    println!(
        "G17_GOAL full={}/8 depth1={}/8 no_goal_specific={}/8 wrong_goal_follows_other={}/8 broken_goal={}/4 broken_route={}/4 phase_shift={}/4 restored={}/8 irrelevant={}/2",
        full_total,depth1_total,no_goal_specific,wrong_goal_follows_other,
        broken_goal_total,broken_route_total,phase_total,restored_total,
        irrelevant_ok
    );

    assert_eq!(full_total,8);
    assert!(depth1_total<=2);
    assert!(no_goal_specific<=2);
    assert!(wrong_goal_follows_other>=6);
    assert!(broken_goal_total<=2);
    assert!(broken_route_total<=2);
    assert!(phase_total<=2);
    assert_eq!(restored_total,8);
    assert_eq!(irrelevant_ok,2);
    assert_eq!(base.planning_transition_count(),0);
}

#[test]
fn g17_goal_planner_has_no_goal_id_graph_or_answer_fallback(){
    let source=include_str!("../src/phase_abstract_planning.rs");
    let start=source.find("pub fn plan_phase_native_abstract_goal")
        .expect("G17 goal planner");
    let end=source[start..].find("pub fn plan_phase_native_abstract(")
        .map(|offset|start+offset)
        .unwrap_or(source.len());
    let goal_source=&source[start..end];

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
        "S4",
        "S5",
        "goal_id",
        "correct_action",
        "dijkstra",
        "bfs",
        "dfs",
    ] {
        assert!(
            !goal_source.contains(forbidden),
            "G17 goal planner contains forbidden token {forbidden}"
        );
    }

    for required in [
        "phase_native_abstract_state",
        "goal.cell",
        "conductance",
        "successor_synapse",
        "afferent_synapse",
        "motor_synapse",
        "Authority::Imagined",
    ] {
        assert!(
            goal_source.contains(required),
            "G17 goal planner missing physical dependency {required}"
        );
    }
}


#[derive(Debug, Clone)]
struct FreshG17Spec {
    selected_atoms: [usize;8],
    l1_pairs: [[usize;2];4],
    state_pairs: [[usize;2];6],
    roles: [usize;6],
    motors: [usize;6],
    layouts: [[(usize,usize);4];6],
    transition_order: [usize;12],
}

struct FreshG17Rng(u64);

impl FreshG17Rng {
    fn new(seed:u64)->Self{Self(seed^0xA37E_1717_C011_2026)}
    fn next(&mut self)->u64{
        let mut x=self.0;
        x^=x>>12;
        x^=x<<25;
        x^=x>>27;
        self.0=x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self,upper:usize)->usize{
        (self.next()%upper as u64) as usize
    }
}

fn fresh17_shuffle<T>(rng:&mut FreshG17Rng,values:&mut [T]){
    for i in (1..values.len()).rev(){
        let j=rng.range(i+1);
        values.swap(i,j);
    }
}

fn fresh17_fnv(mut h:u64,value:u64)->u64{
    const PRIME:u64=1_099_511_628_211;
    for byte in value.to_le_bytes(){
        h^=byte as u64;
        h=h.wrapping_mul(PRIME);
    }
    h
}

fn fresh17_specs(authority:u64)->(Vec<FreshG17Spec>,u64){
    let mut specs=Vec::new();
    let mut digest=14_695_981_039_346_656_037u64;
    let factors=factorization_8();

    for sub in 0..10u64 {
        let mut rng=FreshG17Rng::new(
            authority
                ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ 0x1717_5EED_D1B5_4A32
        );

        let mut selected_atoms=[0usize,1,2,3,4,5,6,7];
        fresh17_shuffle(&mut rng,&mut selected_atoms);

        let l1_pairs=factors[0].map(|p|{
            let mut pair=[selected_atoms[p[0]],selected_atoms[p[1]]];
            pair.sort_unstable();
            pair
        });

        let mut state_pairs=[
            [0usize,1usize],[0,2],[0,3],[1,2],[1,3],[2,3],
        ];
        fresh17_shuffle(&mut rng,&mut state_pairs);

        let mut roles=[0usize,1,2,3,4,5];
        fresh17_shuffle(&mut rng,&mut roles);

        let mut motors=[0usize,1,2,3,4,5];
        fresh17_shuffle(&mut rng,&mut motors);

        let mut layouts=LAYOUTS;
        fresh17_shuffle(&mut rng,&mut layouts);

        let mut transition_order=[0usize;12];
        for (index,slot) in transition_order.iter_mut().enumerate(){
            *slot=index;
        }
        fresh17_shuffle(&mut rng,&mut transition_order);

        let spec=FreshG17Spec {
            selected_atoms,l1_pairs,state_pairs,roles,motors,layouts,
            transition_order,
        };

        digest=fresh17_fnv(digest,sub);
        for value in spec.selected_atoms {
            digest=fresh17_fnv(digest,value as u64);
        }
        for pair in spec.l1_pairs {
            for value in pair {digest=fresh17_fnv(digest,value as u64);}
        }
        for pair in spec.state_pairs {
            for value in pair {digest=fresh17_fnv(digest,value as u64);}
        }
        for value in spec.roles {digest=fresh17_fnv(digest,value as u64);}
        for value in spec.motors {digest=fresh17_fnv(digest,value as u64);}
        for layout in spec.layouts {
            for (x,y) in layout {
                digest=fresh17_fnv(digest,x as u64);
                digest=fresh17_fnv(digest,y as u64);
            }
        }
        for value in spec.transition_order {
            digest=fresh17_fnv(digest,value as u64);
        }

        specs.push(spec);
    }

    (specs,digest)
}

fn fresh17_pair_scene(
    pair:[usize;2],
    layout:[(usize,usize);4],
)->Vec<f32>{
    pair_scene(pair,layout)
}

fn fresh17_state_scene(
    spec:&FreshG17Spec,
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    state_scene(&spec.l1_pairs,&spec.state_pairs,state,layout)
}

fn fresh17_train_abstraction(spec:&FreshG17Spec)->EvoPhase{
    let factors=factorization_8();
    let mut evo=carrier();

    let foundation_good=spec.motors[0];
    let foundation_bad=spec.motors[1];

    for pair in spec.l1_pairs {
        for layout in spec.layouts[..4].iter().copied(){
            let scene=fresh17_pair_scene(pair,layout);
            for action in [foundation_good,foundation_bad] {
                assert!(evo.observe_phase_native_concept_factual(
                    &scene,action,action==foundation_good
                ));
            }
        }
    }

    for (round_index,round) in factors[1..5].iter().enumerate(){
        let layout=spec.layouts[(round_index+2)%6];
        for raw_pair in *round {
            let mut pair=[
                spec.selected_atoms[raw_pair[0]],
                spec.selected_atoms[raw_pair[1]],
            ];
            pair.sort_unstable();
            let scene=fresh17_pair_scene(pair,layout);
            for action in [foundation_good,foundation_bad] {
                assert!(evo.observe_phase_native_concept_factual(
                    &scene,action,action==foundation_bad
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(),8);
    assert_eq!(evo.phase_native_promoted_concept_count(),4);
    assert!(evo.composite_concepts().is_empty());

    assert!(evo.enable_phase_native_depth_generic_abstraction(2));
    let stage2_a=spec.motors[2];
    let stage2_b=spec.motors[3];

    for l1 in 0..4usize {
        let scene=fresh17_pair_scene(
            spec.l1_pairs[l1],
            spec.layouts[l1%6],
        );
        for rep in 0..4usize {
            let a_wins=rep%2==0;
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,stage2_a,a_wins
            ));
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,stage2_b,!a_wins
            ));
        }
    }
    assert_eq!(evo.phase_native_deep_candidate_count(2),0);

    for cycle in 0..4usize {
        for state in 0..6usize {
            let scene=fresh17_state_scene(
                spec,state,spec.layouts[cycle%6]
            );
            for action in [stage2_a,stage2_b] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==stage2_a
                ));
            }
            for child in spec.state_pairs[state] {
                let single=fresh17_pair_scene(
                    spec.l1_pairs[child],
                    spec.layouts[(cycle+child+2)%6],
                );
                for action in [stage2_a,stage2_b] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==stage2_b
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_promoted_deep_count(2),6);
    evo.set_concept_learning_enabled(false);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..6usize {
        let first=evo.phase_native_abstract_state(
            &fresh17_state_scene(spec,state,spec.layouts[0])
        ).expect("fresh G17 L2 state");
        assert_eq!(first.level,2);
        assert!(cells.insert(first.cell));
        for layout in spec.layouts {
            let other=evo.phase_native_abstract_state(
                &fresh17_state_scene(spec,state,layout)
            ).expect("fresh G17 held state");
            assert_eq!(other,first);
        }
    }
    assert_eq!(cells.len(),6);
    evo
}

#[derive(Clone,Copy)]
struct FreshG17Fact {
    pre:usize,
    action:usize,
    post:usize,
}

fn fresh17_facts(spec:&FreshG17Spec)->[FreshG17Fact;12]{
    let start=spec.roles[0];
    let mid_a=spec.roles[1];
    let mid_b=spec.roles[2];
    let goal_a=spec.roles[3];
    let goal_b=spec.roles[4];
    let distractor=spec.roles[5];
    let a=spec.motors[4];
    let b=spec.motors[5];

    [
        FreshG17Fact{pre:start,action:a,post:mid_a},
        FreshG17Fact{pre:start,action:b,post:mid_b},
        FreshG17Fact{pre:mid_a,action:a,post:goal_a},
        FreshG17Fact{pre:mid_a,action:b,post:distractor},
        FreshG17Fact{pre:mid_b,action:a,post:distractor},
        FreshG17Fact{pre:mid_b,action:b,post:goal_b},
        FreshG17Fact{pre:goal_a,action:a,post:goal_a},
        FreshG17Fact{pre:goal_a,action:b,post:goal_a},
        FreshG17Fact{pre:goal_b,action:a,post:goal_b},
        FreshG17Fact{pre:goal_b,action:b,post:goal_b},
        FreshG17Fact{pre:distractor,action:a,post:distractor},
        FreshG17Fact{pre:distractor,action:b,post:distractor},
    ]
}

fn fresh17_learn_model(evo:&mut EvoPhase,spec:&FreshG17Spec){
    let facts=fresh17_facts(spec);
    let tuition=spec.layouts[0];
    for index in spec.transition_order {
        let fact=facts[index];
        let pre=fresh17_state_scene(spec,fact.pre,tuition);
        let post=fresh17_state_scene(spec,fact.post,tuition);
        assert!(evo.observe_phase_native_abstract_transition(
            &pre,fact.action,&post,0.0
        ));
    }
    assert_eq!(evo.phase_native_circuits().len(),12);
}

fn fresh17_ref(
    evo:&EvoPhase,spec:&FreshG17Spec,state:usize
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(
        &fresh17_state_scene(spec,state,spec.layouts[0])
    ).expect("fresh G17 state ref")
}

fn fresh17_transition(
    evo:&EvoPhase,
    from:PhaseAbstractStateRef,
    action:usize,
    to:PhaseAbstractStateRef,
)->PhaseCircuitInfo{
    transition_circuit(evo,from,action,to)
}

fn fresh17_l2_node(
    evo:&EvoPhase,state:PhaseAbstractStateRef
)->PhaseDeepNodeInfo{
    l2_node(evo,state)
}

fn fresh17_wilson95(success:usize,n:usize)->(f64,f64){
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
fn g17_fresh_goal_conditioned_planning_pack(){
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse().expect("fresh G17 seed must be u64");
    let source_sha=std::env::var("AETERNA_SOURCE_SHA")
        .unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA")
        .unwrap_or_else(|_|"unknown".into());

    let (specs,digest)=fresh17_specs(authority);
    println!(
        "FRESH_G17_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (sub,spec) in specs.iter().enumerate(){
        println!("FRESH_G17_BLOCK sub={} {:?}",sub,spec);
    }

    let mut full=0usize;
    let mut per_seed=Vec::new();
    let mut switch_violations=0usize;
    let mut depth1=0usize;
    let mut no_goal=0usize;
    let mut wrong_goal=0usize;
    let mut broken_goal=0usize;
    let mut broken_route=0usize;
    let mut phase_shift=0usize;
    let mut restored=0usize;
    let mut irrelevant=0usize;
    let mut structure_violations=0usize;
    let mut outcome_violations=0usize;
    let mut endpoint_violations=0usize;
    let mut real_mutations=0usize;
    let mut fingerprint_mutations=0usize;
    let mut legacy_violations=0usize;
    let mut motor_mask=0u8;

    for (sub,spec) in specs.iter().enumerate(){
        for motor in spec.motors {motor_mask|=1u8<<motor;}

        let mut evo=fresh17_train_abstraction(spec);
        if evo.concept_atoms().len()!=8
            || evo.phase_native_promoted_concept_count()!=4
            || evo.phase_native_promoted_deep_count(2)!=6
            || !evo.composite_concepts().is_empty()
        {
            structure_violations+=1;
        }

        fresh17_learn_model(&mut evo,spec);

        let refs=(0..6usize)
            .map(|state|fresh17_ref(&evo,spec,state))
            .collect::<Vec<_>>();
        let cells=refs.iter()
            .map(|r|r.cell)
            .collect::<std::collections::BTreeSet<_>>();
        for circuit in evo.phase_native_circuits(){
            let aff=evo.phase_native_synapse(circuit.afferent_synapse).unwrap();
            let succ=evo.phase_native_synapse(circuit.successor_synapse).unwrap();
            let outcome=evo.phase_native_synapse(circuit.outcome_synapse).unwrap();
            if !cells.contains(&aff.from) || !cells.contains(&succ.to) {
                endpoint_violations+=1;
            }
            if outcome.weight.abs()>1.0e-7 {
                outcome_violations+=1;
            }
        }
        if evo.planning_transition_count()!=0 {legacy_violations+=1;}

        evo.set_planning_learning_enabled(false);

        let start=spec.roles[0];
        let goal_a=spec.roles[3];
        let goal_b=spec.roles[4];
        let mid_a=spec.roles[1];
        let mid_b=spec.roles[2];
        let action_a=spec.motors[4];
        let action_b=spec.motors[5];

        let mut sub_full=0usize;
        let mut goal_a_actions=Vec::new();
        let mut goal_b_actions=Vec::new();

        for goal in [goal_a,goal_b] {
            let expected=if goal==goal_a {action_a}else{action_b};
            for layout in spec.layouts[1..5].iter().copied(){
                let current=fresh17_state_scene(spec,start,layout);
                let goal_scene=fresh17_state_scene(spec,goal,layout);
                evo.observe_initial_real(&current,false);
                let real_before=evo.current_real().unwrap().clone();
                let fp_before=evo.phase_native_learned_fingerprint();

                let decision=evo.plan_phase_native_abstract_goal(
                    &current,&goal_scene,None
                );

                let real_after=evo.current_real().unwrap();
                let fp_after=evo.phase_native_learned_fingerprint();
                if real_after.sensory!=real_before.sensory
                    || real_after.need!=real_before.need
                    || real_after.tick!=real_before.tick
                {real_mutations+=1;}
                if fp_after!=fp_before {fingerprint_mutations+=1;}

                let action=decision.map(|d|d.first_action);
                let ok=action==Some(expected);
                full+=usize::from(ok);
                sub_full+=usize::from(ok);
                if goal==goal_a {goal_a_actions.push(action);}
                else {goal_b_actions.push(action);}

                let mut shallow=evo.clone();
                depth1+=usize::from(
                    shallow.plan_phase_native_abstract_goal(
                        &current,&goal_scene,Some(1)
                    ).map(|d|d.first_action)==Some(expected)
                );

                no_goal+=usize::from(
                    evo.plan_phase_native_abstract(&current,None)
                        .map(|d|d.first_action)==Some(expected)
                );

                let other=if goal==goal_a {goal_b}else{goal_a};
                let other_expected=if other==goal_a {action_a}else{action_b};
                let wrong=fresh17_state_scene(spec,other,layout);
                wrong_goal+=usize::from(
                    evo.plan_phase_native_abstract_goal(
                        &current,&wrong,None
                    ).map(|d|d.first_action)==Some(other_expected)
                );
            }
        }

        if goal_a_actions.iter().any(|a|*a!=Some(action_a))
            || goal_b_actions.iter().any(|a|*a!=Some(action_b))
            || action_a==action_b
        {
            switch_violations+=1;
        }
        per_seed.push(sub_full);

        let s0=refs[start];
        let mid_a_ref=refs[mid_a];
        let mid_b_ref=refs[mid_b];
        let goal_a_ref=refs[goal_a];

        let goal_node=fresh17_l2_node(&evo,goal_a_ref);
        let mut bg=evo.clone();
        bg.perturb_phase_native_synapse_for_control(
            goal_node.child_synapses[0],0.0,0.0
        ).expect("fresh G17 goal-recognition synapse");
        for layout in spec.layouts[1..3].iter().copied(){
            let current=fresh17_state_scene(spec,start,layout);
            let goal=fresh17_state_scene(spec,goal_a,layout);
            broken_goal+=usize::from(
                bg.plan_phase_native_abstract_goal(
                    &current,&goal,None
                ).map(|d|d.first_action)==Some(action_a)
            );
        }

        let selected=fresh17_transition(
            &evo,s0,action_a,mid_a_ref
        );
        let mut br=evo.clone();
        let saved=br.perturb_phase_native_synapse_for_control(
            selected.successor_synapse,0.0,0.0
        ).expect("fresh G17 route synapse");
        for layout in spec.layouts[1..3].iter().copied(){
            let current=fresh17_state_scene(spec,start,layout);
            let goal=fresh17_state_scene(spec,goal_a,layout);
            broken_route+=usize::from(
                br.plan_phase_native_abstract_goal(
                    &current,&goal,None
                ).map(|d|d.first_action)==Some(action_a)
            );
        }

        br.restore_phase_native_synapse_for_control(
            selected.successor_synapse,saved.clone()
        );
        for layout in spec.layouts[1..3].iter().copied(){
            let current=fresh17_state_scene(spec,start,layout);
            let goal=fresh17_state_scene(spec,goal_a,layout);
            restored+=usize::from(
                br.plan_phase_native_abstract_goal(
                    &current,&goal,None
                ).map(|d|d.first_action)==Some(action_a)
            );
        }

        let mut shifted=evo.clone();
        shifted.perturb_phase_native_synapse_for_control(
            selected.successor_synapse,1.0,std::f32::consts::PI
        ).expect("fresh G17 phase route");
        for layout in spec.layouts[1..3].iter().copied(){
            let current=fresh17_state_scene(spec,start,layout);
            let goal=fresh17_state_scene(spec,goal_a,layout);
            phase_shift+=usize::from(
                shifted.plan_phase_native_abstract_goal(
                    &current,&goal,None
                ).map(|d|d.first_action)==Some(action_a)
            );
        }

        let competing=fresh17_transition(
            &evo,s0,action_b,mid_b_ref
        );
        let mut unrelated_evo=evo.clone();
        unrelated_evo.perturb_phase_native_synapse_for_control(
            competing.successor_synapse,0.0,0.0
        ).expect("fresh G17 competing route");
        let current=fresh17_state_scene(spec,start,spec.layouts[1]);
        let goal=fresh17_state_scene(spec,goal_a,spec.layouts[1]);
        irrelevant+=usize::from(
            unrelated_evo.plan_phase_native_abstract_goal(
                &current,&goal,None
            ).map(|d|d.first_action)==Some(action_a)
        );

        println!(
            "FRESH_G17_SUB sub={} full={}/8 start={} goal_a={} goal_b={} actions=[{},{}]",
            sub,sub_full,start,goal_a,goal_b,action_a,action_b
        );
    }

    let (lo,hi)=fresh17_wilson95(full,80);
    println!(
        "FRESH_G17_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} switch_violations={} depth1={}/80 no_goal={}/80 wrong_goal={}/80 broken_goal={}/20 broken_route={}/20 phase_shift={}/20 restored={}/20 irrelevant={}/10 structure_violations={} outcome_violations={} endpoint_violations={} real_mutations={} fingerprint_mutations={} legacy_violations={} motor_mask={:#08b}",
        full,lo,hi,per_seed,switch_violations,depth1,no_goal,wrong_goal,
        broken_goal,broken_route,phase_shift,restored,irrelevant,
        structure_violations,outcome_violations,endpoint_violations,
        real_mutations,fingerprint_mutations,legacy_violations,motor_mask
    );

    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|score|*score>=6));
    assert_eq!(switch_violations,0);
    assert!(depth1<=10);
    assert!(no_goal<=20);
    assert!(wrong_goal>=70);
    assert!(broken_goal<=4);
    assert!(broken_route<=4);
    assert!(phase_shift<=4);
    assert!(restored>=19);
    assert!(irrelevant>=9);
    assert_eq!(structure_violations,0);
    assert_eq!(outcome_violations,0);
    assert_eq!(endpoint_violations,0);
    assert_eq!(real_mutations,0);
    assert_eq!(fingerprint_mutations,0);
    assert_eq!(legacy_violations,0);
    assert_eq!(motor_mask,0b11_1111);
}
