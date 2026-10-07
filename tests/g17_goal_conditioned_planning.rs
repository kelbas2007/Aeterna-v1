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
