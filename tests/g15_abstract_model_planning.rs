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
