use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase};
use aeterna_v1::carrier::{PhaseDeepChildRef, PhaseDeepNodeInfo, PhaseNativeConfig};

const W: usize = 20;
const H: usize = 20;

const G13_OFFSETS: [(usize, usize); 16] = [
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

const L3_TUITION_LAYOUT: [(usize,usize);8] = [
    (13,1),(2,16),(9,13),(14,12),
    (8,3),(0,3),(2,0),(15,0),
];

const L3_HELDOUT_LAYOUTS: [[(usize,usize);8];2] = [
    [
        (16,10),(6,8),(15,17),(0,15),
        (1,4),(17,0),(9,4),(2,0),
    ],
    [
        (14,17),(4,8),(0,1),(8,3),
        (12,10),(4,13),(17,8),(0,17),
    ],
];

fn factorization_16() -> Vec<[[usize;2];8]> {
    let mut ring = (0usize..16).collect::<Vec<_>>();
    let mut rounds = Vec::new();
    for _ in 0..15 {
        let mut pairs = [[0usize;2];8];
        for i in 0..8 {
            let a = ring[i];
            let b = ring[15-i];
            pairs[i] = if a < b { [a,b] } else { [b,a] };
        }
        pairs.sort_unstable();
        rounds.push(pairs);

        let last = ring.pop().unwrap();
        ring.insert(1,last);
    }
    rounds
}

fn blank() -> Vec<f32> {
    vec![0.0; W*H]
}

fn add_motif(raster: &mut [f32], atom: usize, origin:(usize,usize)) {
    let (x,y)=origin;
    let (dx,dy)=G13_OFFSETS[atom];
    assert!(x+dx<W && y+dy<H);
    raster[y*W+x]=1.0;
    raster[(y+dy)*W+x+dx]=1.0;
}

fn pair_scene(pair:[usize;2], binding:((usize,usize),(usize,usize))) -> Vec<f32> {
    let mut r=blank();
    add_motif(&mut r,pair[0],binding.0);
    add_motif(&mut r,pair[1],binding.1);
    r
}

fn concept_scene(
    l1_pairs:&[[usize;2];8],
    concepts:&[usize],
    origins:&[(usize,usize)],
) -> Vec<f32> {
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

fn carrier() -> EvoPhase {
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
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_concepts());
    evo
}

fn train_l1(evo:&mut EvoPhase) -> [[usize;2];8] {
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

fn active_l1_ids(evo:&EvoPhase,sensory:&[f32]) -> Vec<u64> {
    let atoms=evo.active_concept_atom_ids(sensory);
    let mut ids=evo.phase_native_concept_circuits()
        .iter()
        .filter(|c| c.promoted && c.child_ids.iter().all(|id| atoms.binary_search(id).is_ok()))
        .map(|c|c.concept_id)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn promoted_deep_for_scene(evo:&EvoPhase,sensory:&[f32],level:u8) -> Vec<PhaseDeepNodeInfo> {
    let l1=active_l1_ids(evo,sensory);
    let mut active_refs=l1.iter().filter_map(|id| {
        evo.phase_native_concept_circuits().iter()
            .find(|c|c.promoted && c.concept_id==*id)
            .map(|c|PhaseDeepChildRef{level:1,id:*id,cell:c.concept_cell})
    }).collect::<Vec<_>>();

    for current in 2..=level {
        let prior=active_refs.iter().copied()
            .filter(|r|r.level+1==current).collect::<Vec<_>>();
        let mut added=Vec::new();
        for node in evo.phase_native_deep_nodes().iter()
            .filter(|n|n.promoted && n.level==current)
        {
            if node.children.iter().all(|wanted|prior.iter().any(|r|
                r.level==wanted.level && r.id==wanted.id && r.cell==wanted.cell
            )) {
                added.push(PhaseDeepChildRef{level:current,id:node.id,cell:node.concept_cell});
            }
        }
        active_refs.extend(added);
    }

    let active_keys=active_refs.iter().map(|r|(r.level,r.id,r.cell))
        .collect::<std::collections::BTreeSet<_>>();
    evo.phase_native_deep_nodes().iter()
        .filter(|n| n.promoted && n.level==level
            && active_keys.contains(&(level,n.id,n.concept_cell)))
        .cloned().collect()
}

fn balanced_child_tuition(
    evo:&mut EvoPhase,
    sensory:&[f32],
    actions:[usize;2],
) {
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

const L2_TARGETS:[[usize;2];4]=[[0,1],[2,3],[4,5],[6,7]];

fn l2_pair_scene(l1_pairs:&[[usize;2];8],target_index:usize,layout:[(usize,usize);4])->Vec<f32>{
    let pair=L2_TARGETS[target_index];
    concept_scene(l1_pairs,&[pair[0],pair[1]],&layout)
}

fn l1_single_scene(l1_pairs:&[[usize;2];8],index:usize,binding:((usize,usize),(usize,usize)))->Vec<f32>{
    pair_scene(l1_pairs[index],binding)
}

fn train_l2(evo:&mut EvoPhase,l1_pairs:&[[usize;2];8]) {
    for index in 0..8usize {
        let scene=l1_single_scene(l1_pairs,index,STAGE1_BINDINGS[index%4]);
        balanced_child_tuition(evo,&scene,[2,3]);
    }
    assert_eq!(evo.phase_native_deep_candidate_count(2),0);

    for cycle in 0..4usize {
        for target in 0..4usize {
            let scene=l2_pair_scene(l1_pairs,target,L2_LAYOUTS[cycle]);
            let correct=if target%2==0 {2}else{3};
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                ));
            }

            for child in L2_TARGETS[target] {
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
    assert_eq!(evo.phase_native_deep_candidate_count(3),0);
}

fn l2_single_scene(l1_pairs:&[[usize;2];8],l2_index:usize,layout:[(usize,usize);4])->Vec<f32>{
    l2_pair_scene(l1_pairs,l2_index,layout)
}

const L3_TARGETS:[[usize;2];2]=[[0,1],[2,3]];

fn l3_pair_scene(
    l1_pairs:&[[usize;2];8],
    target_index:usize,
    layout:[(usize,usize);8],
)->Vec<f32>{
    let l2s=L3_TARGETS[target_index];
    let mut l1s=Vec::new();
    for l2 in l2s {
        l1s.extend_from_slice(&L2_TARGETS[l2]);
    }
    concept_scene(l1_pairs,&l1s,&layout)
}

fn train_l3(evo:&mut EvoPhase,l1_pairs:&[[usize;2];8],swap:bool,require_accept:bool) {
    for l2 in 0..4usize {
        let scene=l2_single_scene(l1_pairs,l2,L2_LAYOUTS[l2]);
        balanced_child_tuition(evo,&scene,[4,5]);
    }
    assert_eq!(evo.phase_native_deep_candidate_count(3),0);

    for cycle in 0..4usize {
        for target in 0..2usize {
            let scene=l3_pair_scene(l1_pairs,target,L3_TUITION_LAYOUT);
            let class_one=target==1;
            let correct=if class_one^swap {5}else{4};
            for action in [4usize,5usize] {
                let accepted=evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                );
                if require_accept { assert!(accepted); }
            }

            for l2 in L3_TARGETS[target] {
                let single=l2_single_scene(
                    l1_pairs,l2,L2_LAYOUTS[(cycle+l2)%4]
                );
                let opposite=if correct==4 {5}else{4};
                for action in [4usize,5usize] {
                    let accepted=evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==opposite
                    );
                    if require_accept { assert!(accepted); }
                }
            }
        }
    }
}

fn freeze(evo:&mut EvoPhase){
    evo.set_concept_learning_enabled(false);
    evo.set_planning_learning_enabled(false);
}

fn score_l3(evo:&EvoPhase,l1_pairs:&[[usize;2];8],swap:bool)->usize{
    let mut score=0usize;
    for target in 0..2usize {
        let class_one=target==1;
        let expected=if class_one^swap {5}else{4};
        for layout in L3_HELDOUT_LAYOUTS {
            let scene=l3_pair_scene(l1_pairs,target,layout);
            score+=usize::from(
                evo.choose_phase_native_depth_generic_action(&scene)==Some(expected)
            );
        }
    }
    score
}

fn l3_node_for_scene(evo:&EvoPhase,scene:&[f32])->PhaseDeepNodeInfo{
    let active=promoted_deep_for_scene(evo,scene,3);
    assert_eq!(active.len(),1);
    active[0].clone()
}

fn verify_levels(evo:&EvoPhase){
    let l1=evo.phase_native_concept_circuits().iter()
        .filter(|c|c.promoted)
        .map(|c|(c.concept_id,c.concept_cell))
        .collect::<std::collections::BTreeMap<_,_>>();
    let l2=evo.phase_native_deep_nodes().iter()
        .filter(|n|n.promoted && n.level==2)
        .map(|n|(n.id,n.concept_cell))
        .collect::<std::collections::BTreeMap<_,_>>();

    for node in evo.phase_native_deep_nodes().iter().filter(|n|n.promoted) {
        if node.level==2 {
            assert!(node.children.iter().all(|child|
                child.level==1 && l1.get(&child.id)==Some(&child.cell)
            ));
        } else if node.level==3 {
            assert!(node.children.iter().all(|child|
                child.level==2 && l2.get(&child.id)==Some(&child.cell)
            ));
        } else {
            panic!("unexpected promoted deep level {}",node.level);
        }
    }
}

#[test]
fn g13_one_generic_engine_builds_and_executes_depth_three() {
    let mut stage1=carrier();
    let l1_pairs=train_l1(&mut stage1);

    let mut stage2=stage1.clone();
    assert!(stage2.enable_phase_native_depth_generic_abstraction(3));
    train_l2(&mut stage2,&l1_pairs);
    verify_levels(&stage2);

    let mut full_total=0usize;
    let mut max2_total=0usize;
    let mut no_engine_total=0usize;
    let mut zero_phase_total=0usize;
    let mut zero_weight_total=0usize;
    let mut no_growth_total=0usize;
    let mut lesion_total=0usize;
    let mut phase_total=0usize;
    let mut restored_total=0usize;
    let mut unrelated_ok=0usize;
    let mut lower_lesion_ok=0usize;

    for swap in [false,true] {
        let mut full=stage2.clone();
        train_l3(&mut full,&l1_pairs,swap,true);
        assert_eq!(full.phase_native_deep_candidate_count(3),2);
        assert_eq!(full.phase_native_promoted_deep_count(3),2);
        verify_levels(&full);
        freeze(&mut full);
        let baseline=score_l3(&full,&l1_pairs,swap);
        assert_eq!(baseline,4);
        full_total+=baseline;

        let mut max2=stage1.clone();
        assert!(max2.enable_phase_native_depth_generic_abstraction(2));
        train_l2(&mut max2,&l1_pairs);
        train_l3(&mut max2,&l1_pairs,swap,true);
        freeze(&mut max2);
        max2_total+=score_l3(&max2,&l1_pairs,swap);

        let mut no_engine=stage1.clone();
        freeze(&mut no_engine);
        no_engine_total+=score_l3(&no_engine,&l1_pairs,swap);

        let mut zp=stage2.clone();
        zp.set_learning_rates_for_control(1.0,0.0);
        train_l3(&mut zp,&l1_pairs,swap,true);
        freeze(&mut zp);
        zero_phase_total+=score_l3(&zp,&l1_pairs,swap);

        let mut zw=stage2.clone();
        zw.set_learning_rates_for_control(0.0,1.0);
        train_l3(&mut zw,&l1_pairs,swap,true);
        freeze(&mut zw);
        zero_weight_total+=score_l3(&zw,&l1_pairs,swap);

        let mut ng=stage2.clone();
        ng.set_structural_growth_for_control(false);
        train_l3(&mut ng,&l1_pairs,swap,false);
        freeze(&mut ng);
        no_growth_total+=score_l3(&ng,&l1_pairs,swap);

        let target_scene=l3_pair_scene(&l1_pairs,0,L3_HELDOUT_LAYOUTS[0]);
        let node=l3_node_for_scene(&full,&target_scene);

        let mut lesioned=full.clone();
        let saved=lesioned.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],0.0,0.0
        ).expect("G13 necessary L2->L3 synapse");
        lesion_total+=score_l3(&lesioned,&l1_pairs,swap);
        lesioned.restore_phase_native_synapse_for_control(
            node.child_synapses[0],saved
        );
        restored_total+=score_l3(&lesioned,&l1_pairs,swap);

        let mut shifted=full.clone();
        shifted.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],1.0,std::f32::consts::PI
        ).expect("G13 phase intervention");
        phase_total+=score_l3(&shifted,&l1_pairs,swap);

        let other_scene=l3_pair_scene(&l1_pairs,1,L3_HELDOUT_LAYOUTS[0]);
        let other=l3_node_for_scene(&full,&other_scene);
        let mut unrelated=full.clone();
        unrelated.perturb_phase_native_synapse_for_control(
            other.child_synapses[0],0.0,0.0
        ).expect("G13 unrelated L3 synapse");
        let expected=if swap {5}else{4};
        unrelated_ok+=usize::from(
            unrelated.choose_phase_native_depth_generic_action(&target_scene)==Some(expected)
        );

        let l2_child=node.children[0];
        let l2_node=full.phase_native_deep_nodes().iter()
            .find(|n|n.promoted && n.level==2 && n.id==l2_child.id)
            .expect("G13 lower L2 node").clone();
        let mut lower=full.clone();
        lower.perturb_phase_native_synapse_for_control(
            l2_node.child_synapses[0],0.0,0.0
        ).expect("G13 lower L1->L2 dependency");
        lower_lesion_ok+=usize::from(
            lower.choose_phase_native_depth_generic_action(&target_scene)==Some(expected)
        );
    }

    println!(
        "G13_DEPTH3 full={}/8 max2={}/8 no_engine={}/8 zero_phase={}/8 zero_weight={}/8 no_growth={}/8 lesion={}/8 phase_shift={}/8 restored={}/8 unrelated={}/2 lower_lesion_success={}/2 l1={} l2={} l3={}",
        full_total,max2_total,no_engine_total,zero_phase_total,zero_weight_total,
        no_growth_total,lesion_total,phase_total,restored_total,unrelated_ok,
        lower_lesion_ok,stage2.phase_native_promoted_concept_count(),
        stage2.phase_native_promoted_deep_count(2),2
    );

    assert_eq!(stage2.phase_native_promoted_concept_count(),8);
    assert_eq!(stage2.phase_native_promoted_deep_count(2),4);
    assert_eq!(full_total,8);
    assert!(max2_total<=4);
    assert!(no_engine_total<=4);
    assert!(zero_phase_total<=4);
    assert!(zero_weight_total<=4);
    assert!(no_growth_total<=4);
    assert!(lesion_total<=6);
    assert!(phase_total<=6);
    assert_eq!(restored_total,8);
    assert_eq!(unrelated_ok,2);
    assert_eq!(lower_lesion_ok,0);
}

#[test]
fn g13_generic_engine_has_no_level_specific_task_branch_or_search_fallback(){
    let source=include_str!("../src/phase_deep_abstraction.rs");
    for forbidden in [
        "level == 2",
        "level==2",
        "level == 3",
        "level==3",
        "observe_level2",
        "observe_level3",
        "L2_TARGETS",
        "L3_TARGETS",
        "correct_action",
        "EvoImaginationPlanner",
        "BinaryHeap",
        "VecDeque",
    ] {
        assert!(!source.contains(forbidden),"G13 source contains forbidden token {forbidden}");
    }

    for required in [
        "highest_level",
        "target_level",
        "saturating_add(1)",
        "deep.max_level",
        "child.level",
        "child_synapses",
        "motor_synapses",
        "conductance",
    ] {
        assert!(source.contains(required),"G13 generic dependency missing {required}");
    }
}
