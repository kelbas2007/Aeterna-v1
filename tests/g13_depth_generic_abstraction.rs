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


#[derive(Clone, Debug)]
struct FreshG13Block {
    relation_perm: [usize;16],
    l1_pairs: [[usize;2];8],
    balancing: [[[usize;2];8];4],
    l2_targets: [[usize;2];4],
    l3_targets: [[usize;2];2],
    motors: [usize;6],
    pair_order: [usize;4],
    l2_layout_order: [usize;4],
    l2_orders: [[usize;4];4],
    l3_orders: [[usize;2];4],
    l3_tuition: [[[ (usize,usize);8 ];4];2],
    l3_heldout: [[[ (usize,usize);8 ];4];2],
    layout_rejections: usize,
}

struct FreshG13Rng(u64);

impl FreshG13Rng {
    fn new(seed:u64)->Self { Self(seed ^ 0xD313_A37E_5EED_0020) }
    fn next(&mut self)->u64 {
        let mut x=self.0;
        x^=x>>12; x^=x<<25; x^=x>>27;
        self.0=x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self,upper:usize)->usize {
        (self.next()%upper as u64) as usize
    }
}

fn fresh13_shuffle<T>(rng:&mut FreshG13Rng, values:&mut [T]) {
    for i in (1..values.len()).rev() {
        let j=rng.range(i+1);
        values.swap(i,j);
    }
}

fn fresh13_pair(a:usize,b:usize)->[usize;2] {
    if a<=b {[a,b]} else {[b,a]}
}

fn fresh13_layout_valid(atoms:&[usize], origins:&[(usize,usize)])->bool {
    if atoms.len()!=origins.len() { return false; }
    let mut points=Vec::<((usize,usize),usize)>::new();

    for (owner,(&atom,&origin)) in atoms.iter().zip(origins).enumerate() {
        let (x,y)=origin;
        let (dx,dy)=G13_OFFSETS[atom];
        if x+dx>=W || y+dy>=H { return false; }
        for point in [(x,y),(x+dx,y+dy)] {
            if points.iter().any(|(existing,_)|*existing==point) { return false; }
            points.push((point,owner));
        }
    }

    for i in 0..points.len() {
        for j in (i+1)..points.len() {
            if points[i].1==points[j].1 { continue; }
            let a=points[i].0;
            let b=points[j].0;
            if a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))>4 { continue; }
            let (from,to)=if a<=b {(a,b)} else {(b,a)};
            let dx=to.0 as isize-from.0 as isize;
            let dy=to.1 as isize-from.1 as isize;
            if dx>=0 && dy>=0
                && G13_OFFSETS.iter().any(|&(x,y)|x==dx as usize && y==dy as usize)
            {
                return false;
            }
        }
    }
    true
}

fn fresh13_unique_layout(
    atoms:&[usize],
    rng:&mut FreshG13Rng,
    used:&mut Vec<[(usize,usize);8]>,
    rejections:&mut usize,
)->[(usize,usize);8] {
    assert_eq!(atoms.len(),8);
    for _ in 0..100_000usize {
        let mut candidate=Vec::with_capacity(8);
        for &atom in atoms {
            let (dx,dy)=G13_OFFSETS[atom];
            candidate.push((rng.range(W-dx),rng.range(H-dy)));
        }
        let array:[(usize,usize);8]=candidate.try_into().unwrap();
        if used.contains(&array) || !fresh13_layout_valid(atoms,&array) {
            *rejections+=1;
            continue;
        }
        used.push(array);
        return array;
    }
    panic!("frozen geometry-only G13 layout search exhausted");
}

fn fresh13_l3_atoms(block:&FreshG13Block,target:usize)->Vec<usize> {
    let mut atoms=Vec::with_capacity(8);
    for l2 in block.l3_targets[target] {
        for l1 in block.l2_targets[l2] {
            atoms.extend_from_slice(&block.l1_pairs[l1]);
        }
    }
    atoms
}

fn fresh13_map_round(
    round:[[usize;2];8],
    perm:&[usize;16],
)->[[usize;2];8] {
    let mut out=[[0usize;2];8];
    for (index,pair) in round.into_iter().enumerate() {
        out[index]=fresh13_pair(perm[pair[0]],perm[pair[1]]);
    }
    out.sort_unstable();
    out
}

fn fresh13_block(authority:u64,sub:u64)->FreshG13Block {
    let mut rng=FreshG13Rng::new(
        authority
            ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ 0x1313_D33F_C011_771B
    );

    let mut relation_perm:[usize;16]=std::array::from_fn(|i|i);
    fresh13_shuffle(&mut rng,&mut relation_perm);

    let factors=factorization_16();
    let l1_pairs=fresh13_map_round(factors[0],&relation_perm);
    let balancing:[[[usize;2];8];4]=std::array::from_fn(|i|
        fresh13_map_round(factors[i+1],&relation_perm)
    );

    let mut l1_order:[usize;8]=std::array::from_fn(|i|i);
    fresh13_shuffle(&mut rng,&mut l1_order);
    let l2_targets:[ [usize;2];4 ]=std::array::from_fn(|i|
        fresh13_pair(l1_order[2*i],l1_order[2*i+1])
    );

    let mut l2_order:[usize;4]=[0,1,2,3];
    fresh13_shuffle(&mut rng,&mut l2_order);
    let l3_targets:[ [usize;2];2 ]=std::array::from_fn(|i|
        fresh13_pair(l2_order[2*i],l2_order[2*i+1])
    );

    let mut motors:[usize;6]=[0,1,2,3,4,5];
    fresh13_shuffle(&mut rng,&mut motors);

    let mut pair_order=[0usize,1,2,3];
    fresh13_shuffle(&mut rng,&mut pair_order);
    let mut l2_layout_order=[0usize,1,2,3];
    fresh13_shuffle(&mut rng,&mut l2_layout_order);

    let mut l2_orders=[[0usize;4];4];
    for cycle in 0..4 {
        l2_orders[cycle]=[0,1,2,3];
        fresh13_shuffle(&mut rng,&mut l2_orders[cycle]);
    }

    let mut l3_orders=[[0usize;2];4];
    for cycle in 0..4 {
        l3_orders[cycle]=[0,1];
        fresh13_shuffle(&mut rng,&mut l3_orders[cycle]);
    }

    let mut placeholder=FreshG13Block {
        relation_perm,
        l1_pairs,
        balancing,
        l2_targets,
        l3_targets,
        motors,
        pair_order,
        l2_layout_order,
        l2_orders,
        l3_orders,
        l3_tuition:[[[(0,0);8];4];2],
        l3_heldout:[[[(0,0);8];4];2],
        layout_rejections:0,
    };

    let mut rejected=0usize;
    for target in 0..2 {
        let atoms=fresh13_l3_atoms(&placeholder,target);
        let mut used=Vec::new();
        for cycle in 0..4 {
            placeholder.l3_tuition[target][cycle]=
                fresh13_unique_layout(&atoms,&mut rng,&mut used,&mut rejected);
        }
        for holdout in 0..4 {
            placeholder.l3_heldout[target][holdout]=
                fresh13_unique_layout(&atoms,&mut rng,&mut used,&mut rejected);
        }
    }
    placeholder.layout_rejections=rejected;
    placeholder
}

fn fresh13_fnv(mut hash:u64,value:u64)->u64 {
    const PRIME:u64=1_099_511_628_211;
    for byte in value.to_le_bytes() {
        hash^=byte as u64;
        hash=hash.wrapping_mul(PRIME);
    }
    hash
}

fn fresh13_digest(blocks:&[FreshG13Block])->u64 {
    let mut h=14_695_981_039_346_656_037u64;
    for (sub,block) in blocks.iter().enumerate() {
        h=fresh13_fnv(h,sub as u64);
        for &v in &block.relation_perm { h=fresh13_fnv(h,v as u64); }
        for pair in block.l1_pairs { for v in pair { h=fresh13_fnv(h,v as u64); } }
        for pair in block.l2_targets { for v in pair { h=fresh13_fnv(h,v as u64); } }
        for pair in block.l3_targets { for v in pair { h=fresh13_fnv(h,v as u64); } }
        for &v in &block.motors { h=fresh13_fnv(h,v as u64); }
        for target in 0..2 {
            for layout in block.l3_tuition[target] {
                for (x,y) in layout {
                    h=fresh13_fnv(h,x as u64);
                    h=fresh13_fnv(h,y as u64);
                }
            }
            for layout in block.l3_heldout[target] {
                for (x,y) in layout {
                    h=fresh13_fnv(h,x as u64);
                    h=fresh13_fnv(h,y as u64);
                }
            }
        }
    }
    h
}

fn fresh13_train_l1(evo:&mut EvoPhase,block:&FreshG13Block) {
    let foundation=[block.motors[0],block.motors[1]];
    for pair in block.l1_pairs {
        for &binding_index in &block.pair_order {
            let raster=pair_scene(pair,STAGE1_BINDINGS[binding_index]);
            for action in foundation {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==foundation[0]
                ));
            }
        }
    }

    for round_index in 0..4 {
        let binding=NEGATIVE_BINDINGS[block.pair_order[round_index]];
        for pair in block.balancing[round_index] {
            let raster=pair_scene(pair,binding);
            for action in foundation {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==foundation[1]
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(),16);
    assert_eq!(evo.composite_concepts().len(),0);
    assert_eq!(evo.phase_native_promoted_concept_count(),8);
}

fn fresh13_l1_single(
    block:&FreshG13Block,index:usize,binding_index:usize
)->Vec<f32> {
    pair_scene(block.l1_pairs[index],STAGE1_BINDINGS[binding_index%4])
}

fn fresh13_l2_scene(
    block:&FreshG13Block,target:usize,layout_index:usize
)->Vec<f32> {
    let pair=block.l2_targets[target];
    concept_scene(
        &block.l1_pairs,
        &[pair[0],pair[1]],
        &L2_LAYOUTS[layout_index%4],
    )
}

fn fresh13_balanced(
    evo:&mut EvoPhase,
    sensory:&[f32],
    actions:[usize;2],
) {
    for rep in 0..4 {
        let first=rep%2==0;
        assert!(evo.observe_phase_native_depth_generic_factual(
            sensory,actions[0],first
        ));
        assert!(evo.observe_phase_native_depth_generic_factual(
            sensory,actions[1],!first
        ));
    }
}

fn fresh13_train_l2(
    evo:&mut EvoPhase,
    block:&FreshG13Block,
) {
    let actions=[block.motors[2],block.motors[3]];

    for index in 0..8 {
        let single=fresh13_l1_single(
            block,index,block.pair_order[index%4]
        );
        fresh13_balanced(evo,&single,actions);
    }
    assert_eq!(evo.phase_native_deep_candidate_count(2),0);

    for cycle in 0..4 {
        let layout=block.l2_layout_order[cycle];
        for target in block.l2_orders[cycle] {
            let scene=fresh13_l2_scene(block,target,layout);
            let correct=if target%2==0 {actions[0]} else {actions[1]};
            for action in actions {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                ));
            }

            let opposite=if correct==actions[0] {actions[1]} else {actions[0]};
            for child in block.l2_targets[target] {
                let single=fresh13_l1_single(
                    block,child,block.pair_order[(cycle+child)%4]
                );
                for action in actions {
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

fn fresh13_l2_single(
    block:&FreshG13Block,l2:usize,layout_index:usize
)->Vec<f32> {
    fresh13_l2_scene(block,l2,layout_index)
}

fn fresh13_l3_scene(
    block:&FreshG13Block,
    target:usize,
    layout:[(usize,usize);8],
)->Vec<f32> {
    let mut l1s=Vec::with_capacity(4);
    for l2 in block.l3_targets[target] {
        l1s.extend_from_slice(&block.l2_targets[l2]);
    }
    concept_scene(&block.l1_pairs,&l1s,&layout)
}

fn fresh13_train_l3(
    evo:&mut EvoPhase,
    block:&FreshG13Block,
    require_accept:bool,
) {
    let actions=[block.motors[4],block.motors[5]];

    for l2 in 0..4 {
        let single=fresh13_l2_single(
            block,l2,block.l2_layout_order[l2]
        );
        fresh13_balanced(evo,&single,actions);
    }
    assert_eq!(evo.phase_native_deep_candidate_count(3),0);

    for cycle in 0..4 {
        for target in block.l3_orders[cycle] {
            let scene=fresh13_l3_scene(
                block,target,block.l3_tuition[target][cycle]
            );
            let correct=if target==0 {actions[0]} else {actions[1]};
            for action in actions {
                let accepted=evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                );
                if require_accept { assert!(accepted); }
            }

            let opposite=if correct==actions[0] {actions[1]} else {actions[0]};
            for l2 in block.l3_targets[target] {
                let single=fresh13_l2_single(
                    block,l2,block.l2_layout_order[(cycle+l2)%4]
                );
                for action in actions {
                    let accepted=evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==opposite
                    );
                    if require_accept { assert!(accepted); }
                }
            }
        }
    }
}

fn fresh13_score(
    evo:&EvoPhase,
    block:&FreshG13Block,
)->usize {
    let actions=[block.motors[4],block.motors[5]];
    let mut score=0usize;
    for target in 0..2 {
        let expected=if target==0 {actions[0]} else {actions[1]};
        for layout in block.l3_heldout[target] {
            let scene=fresh13_l3_scene(block,target,layout);
            score+=usize::from(
                evo.choose_phase_native_depth_generic_action(&scene)==Some(expected)
            );
        }
    }
    score
}

fn fresh13_reference_violations(evo:&EvoPhase)->usize {
    let l1=evo.phase_native_concept_circuits().iter()
        .filter(|c|c.promoted)
        .map(|c|(c.concept_id,c.concept_cell))
        .collect::<std::collections::BTreeMap<_,_>>();
    let l2=evo.phase_native_deep_nodes().iter()
        .filter(|n|n.promoted && n.level==2)
        .map(|n|(n.id,n.concept_cell))
        .collect::<std::collections::BTreeMap<_,_>>();
    let mut violations=0usize;

    for node in evo.phase_native_deep_nodes().iter().filter(|n|n.promoted) {
        if node.level==2 {
            for child in node.children {
                if child.level!=1 || l1.get(&child.id)!=Some(&child.cell) {
                    violations+=1;
                }
            }
        } else if node.level==3 {
            for child in node.children {
                if child.level!=2 || l2.get(&child.id)!=Some(&child.cell) {
                    violations+=1;
                }
            }
        } else {
            violations+=1;
        }
    }
    violations
}

fn fresh13_l3_node(
    evo:&EvoPhase,
    block:&FreshG13Block,
    target:usize,
    layout:[(usize,usize);8],
)->PhaseDeepNodeInfo {
    let scene=fresh13_l3_scene(block,target,layout);
    let active=promoted_deep_for_scene(evo,&scene,3);
    assert_eq!(active.len(),1);
    active[0].clone()
}

fn fresh13_wilson95(success:usize,n:usize)->(f64,f64) {
    let z=1.959_963_984_540_054f64;
    let n=n as f64;
    let p=success as f64/n;
    let denom=1.0+z*z/n;
    let center=(p+z*z/(2.0*n))/denom;
    let half=z*(p*(1.0-p)/n+z*z/(4.0*n*n)).sqrt()/denom;
    (center-half,center+half)
}

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from first-attempt CI"]
fn g13_fresh_depth_generic_abstraction_pack() {
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse().expect("fresh G13 seed must be u64");
    let source_sha=std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_|"unknown".into());

    let blocks=(0..10u64)
        .map(|sub|fresh13_block(authority,sub))
        .collect::<Vec<_>>();
    let digest=fresh13_digest(&blocks);

    println!(
        "FRESH_G13_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (sub,block) in blocks.iter().enumerate() {
        println!("FRESH_G13_BLOCK sub={} {:?}",sub,block);
    }

    let mut full=0usize;
    let mut max2=0usize;
    let mut no_engine=0usize;
    let mut zero_phase=0usize;
    let mut zero_weight=0usize;
    let mut no_growth=0usize;
    let mut per_seed=Vec::new();
    let mut stage1_violations=0usize;
    let mut stage2_violations=0usize;
    let mut stage3_violations=0usize;
    let mut reference_violations=0usize;
    let mut lesion_ok=0usize;
    let mut phase_ok=0usize;
    let mut restored_ok=0usize;
    let mut unrelated_ok=0usize;
    let mut lower_ok=0usize;
    let mut motor_mask=0u8;
    let mut layout_rejections=0usize;

    for (sub,block) in blocks.iter().enumerate() {
        for motor in block.motors { motor_mask|=1u8<<motor; }
        layout_rejections+=block.layout_rejections;

        let mut stage1=carrier();
        fresh13_train_l1(&mut stage1,block);
        stage1_violations+=usize::from(
            stage1.concept_atoms().len()!=16
                || stage1.phase_native_promoted_concept_count()!=8
                || !stage1.composite_concepts().is_empty()
        );

        let mut stage2=stage1.clone();
        assert!(stage2.enable_phase_native_depth_generic_abstraction(3));
        fresh13_train_l2(&mut stage2,block);
        stage2_violations+=usize::from(
            stage2.phase_native_promoted_deep_count(2)!=4
                || stage2.phase_native_deep_candidate_count(3)!=0
        );

        let mut full_evo=stage2.clone();
        fresh13_train_l3(&mut full_evo,block,true);
        stage3_violations+=usize::from(
            full_evo.phase_native_deep_candidate_count(3)!=2
                || full_evo.phase_native_promoted_deep_count(3)!=2
        );
        reference_violations+=fresh13_reference_violations(&full_evo);
        freeze(&mut full_evo);
        let score=fresh13_score(&full_evo,block);
        full+=score;
        per_seed.push(score);

        let mut max2_evo=stage1.clone();
        assert!(max2_evo.enable_phase_native_depth_generic_abstraction(2));
        fresh13_train_l2(&mut max2_evo,block);
        fresh13_train_l3(&mut max2_evo,block,true);
        freeze(&mut max2_evo);
        max2+=fresh13_score(&max2_evo,block);

        let mut no_engine_evo=stage1.clone();
        freeze(&mut no_engine_evo);
        no_engine+=fresh13_score(&no_engine_evo,block);

        let mut zp=stage2.clone();
        zp.set_learning_rates_for_control(1.0,0.0);
        fresh13_train_l3(&mut zp,block,true);
        freeze(&mut zp);
        zero_phase+=fresh13_score(&zp,block);

        let mut zw=stage2.clone();
        zw.set_learning_rates_for_control(0.0,1.0);
        fresh13_train_l3(&mut zw,block,true);
        freeze(&mut zw);
        zero_weight+=fresh13_score(&zw,block);

        let mut ng=stage2.clone();
        ng.set_structural_growth_for_control(false);
        fresh13_train_l3(&mut ng,block,false);
        freeze(&mut ng);
        no_growth+=fresh13_score(&ng,block);

        let actions=[block.motors[4],block.motors[5]];
        let expected=actions[0];
        let node=fresh13_l3_node(
            &full_evo,block,0,block.l3_heldout[0][0]
        );

        let mut lesioned=full_evo.clone();
        let saved=lesioned.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],0.0,0.0
        ).expect("fresh G13 necessary L2->L3 synapse");
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(block,0,block.l3_heldout[0][holdout]);
            lesion_ok+=usize::from(
                lesioned.choose_phase_native_depth_generic_action(&scene)==Some(expected)
            );
        }
        lesioned.restore_phase_native_synapse_for_control(
            node.child_synapses[0],saved
        );
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(block,0,block.l3_heldout[0][holdout]);
            restored_ok+=usize::from(
                lesioned.choose_phase_native_depth_generic_action(&scene)==Some(expected)
            );
        }

        let mut shifted=full_evo.clone();
        shifted.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],1.0,std::f32::consts::PI
        ).expect("fresh G13 phase intervention");
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(block,0,block.l3_heldout[0][holdout]);
            phase_ok+=usize::from(
                shifted.choose_phase_native_depth_generic_action(&scene)==Some(expected)
            );
        }

        let other=fresh13_l3_node(
            &full_evo,block,1,block.l3_heldout[1][0]
        );
        let mut unrelated=full_evo.clone();
        unrelated.perturb_phase_native_synapse_for_control(
            other.child_synapses[0],0.0,0.0
        ).expect("fresh G13 unrelated L3 synapse");
        let target_scene=fresh13_l3_scene(block,0,block.l3_heldout[0][0]);
        unrelated_ok+=usize::from(
            unrelated.choose_phase_native_depth_generic_action(&target_scene)==Some(expected)
        );

        let l2_child=node.children[0];
        let l2_node=full_evo.phase_native_deep_nodes().iter()
            .find(|n|n.promoted && n.level==2 && n.id==l2_child.id)
            .expect("fresh G13 lower L2 node").clone();
        let mut lower=full_evo.clone();
        lower.perturb_phase_native_synapse_for_control(
            l2_node.child_synapses[0],0.0,0.0
        ).expect("fresh G13 lower L1->L2 dependency");
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(block,0,block.l3_heldout[0][holdout]);
            lower_ok+=usize::from(
                lower.choose_phase_native_depth_generic_action(&scene)==Some(expected)
            );
        }

        println!(
            "FRESH_G13_SUB sub={} full={}/8 l1={} l2={} l3={} refs={} rejected_layouts={}",
            sub,score,
            full_evo.phase_native_promoted_concept_count(),
            full_evo.phase_native_promoted_deep_count(2),
            full_evo.phase_native_promoted_deep_count(3),
            fresh13_reference_violations(&full_evo),
            block.layout_rejections
        );
    }

    let (lo,hi)=fresh13_wilson95(full,80);
    println!(
        "FRESH_G13_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} max2={}/80 no_engine={}/80 zero_phase={}/80 zero_weight={}/80 no_growth={}/80 stage1_violations={} stage2_violations={} stage3_violations={} reference_violations={} lesion={}/20 phase_shift={}/20 restored={}/20 unrelated={}/10 lower_lesion_success={}/20 motor_mask={:#08b} layout_rejections={}",
        full,lo,hi,per_seed,max2,no_engine,zero_phase,zero_weight,no_growth,
        stage1_violations,stage2_violations,stage3_violations,reference_violations,
        lesion_ok,phase_ok,restored_ok,unrelated_ok,lower_ok,motor_mask,layout_rejections
    );

    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|score|*score>=6));
    assert_eq!(stage1_violations,0);
    assert_eq!(stage2_violations,0);
    assert_eq!(stage3_violations,0);
    assert_eq!(reference_violations,0);
    assert!(max2<=40);
    assert!(no_engine<=40);
    assert!(zero_phase<=40);
    assert!(zero_weight<=40);
    assert!(no_growth<=40);
    assert!(lesion_ok<=4);
    assert!(phase_ok<=4);
    assert!(restored_ok>=19);
    assert!(unrelated_ok>=9);
    assert!(lower_ok<=1);
    assert_eq!(motor_mask,0b11_1111);
}


const G14_L2_CONFIRM_LAYOUT:[(usize,usize);4]=[
    (3,3),(13,3),(3,13),(13,13),
];

const G14_L2_HELDOUT_LAYOUTS:[[(usize,usize);4];2]=[
    [(3,1),(13,1),(3,11),(13,11)],
    [(1,3),(11,3),(1,13),(11,13)],
];

const G14_L3_CONFIRM_LAYOUT:[(usize,usize);8]=[
    (13,7),(6,13),(1,4),(14,0),
    (0,11),(7,3),(17,11),(9,1),
];

fn g14_highest_promoted_level(evo:&EvoPhase)->u8 {
    evo.phase_native_deep_nodes().iter()
        .filter(|node|node.promoted)
        .map(|node|node.level)
        .max()
        .unwrap_or(1)
}

fn g14_confirm_l2(evo:&mut EvoPhase,l1_pairs:&[[usize;2];8]) {
    for _ in 0..4 {
        for target in 0..4usize {
            let scene=l2_pair_scene(l1_pairs,target,G14_L2_CONFIRM_LAYOUT);
            let correct=if target%2==0 {2}else{3};
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                ));
            }
        }
    }
}

fn g14_score_l2(evo:&EvoPhase,l1_pairs:&[[usize;2];8])->usize {
    let mut score=0usize;
    for target in 0..4usize {
        let expected=if target%2==0 {2}else{3};
        let layout=G14_L2_HELDOUT_LAYOUTS[target%2];
        let scene=l2_pair_scene(l1_pairs,target,layout);
        score+=usize::from(
            evo.choose_phase_native_depth_generic_action(&scene)==Some(expected)
        );
    }
    score
}

fn g14_confirm_l3(evo:&mut EvoPhase,l1_pairs:&[[usize;2];8],swap:bool) {
    for _ in 0..4 {
        for target in 0..2usize {
            let scene=l3_pair_scene(l1_pairs,target,G14_L3_CONFIRM_LAYOUT);
            let class_one=target==1;
            let correct=if class_one^swap {5}else{4};
            for action in [4usize,5usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                ));
            }
        }
    }
}

#[test]
fn g14_open_depth_stops_when_sufficient_and_grows_when_needed() {
    let mut stage1=carrier();
    let l1_pairs=train_l1(&mut stage1);

    let mut open_stage2=stage1.clone();
    assert!(open_stage2.enable_phase_native_open_depth_abstraction());
    assert!(
        open_stage2.phase_native_deep_max_level().unwrap()>=8,
        "open-depth safety ceiling must be architectural, not task depth"
    );
    train_l2(&mut open_stage2,&l1_pairs);
    assert_eq!(open_stage2.phase_native_promoted_deep_count(2),4);
    assert_eq!(open_stage2.phase_native_deep_candidate_count(3),0);

    let mut simple=open_stage2.clone();
    g14_confirm_l2(&mut simple,&l1_pairs);
    assert_eq!(
        simple.phase_native_deep_candidate_count(3),0,
        "sufficient L2 explanation must not trigger needless L3"
    );
    assert_eq!(g14_highest_promoted_level(&simple),2);
    freeze(&mut simple);
    let simple_score=g14_score_l2(&simple,&l1_pairs);
    assert_eq!(simple_score,4);

    let mut deep_total=0usize;
    let mut cap2_total=0usize;
    let mut no_engine_total=0usize;
    let mut lesion_total=0usize;
    let mut phase_total=0usize;
    let mut restored_total=0usize;
    let mut lower_lesion_ok=0usize;

    for swap in [false,true] {
        let mut deep=open_stage2.clone();
        train_l3(&mut deep,&l1_pairs,swap,true);
        assert_eq!(deep.phase_native_promoted_deep_count(3),2);
        g14_confirm_l3(&mut deep,&l1_pairs,swap);
        assert_eq!(
            deep.phase_native_deep_candidate_count(4),0,
            "sufficient L3 explanation must not trigger needless L4"
        );
        assert_eq!(g14_highest_promoted_level(&deep),3);
        freeze(&mut deep);
        let baseline=score_l3(&deep,&l1_pairs,swap);
        assert_eq!(baseline,4);
        deep_total+=baseline;

        let mut cap2=stage1.clone();
        assert!(cap2.enable_phase_native_depth_generic_abstraction(2));
        train_l2(&mut cap2,&l1_pairs);
        train_l3(&mut cap2,&l1_pairs,swap,true);
        freeze(&mut cap2);
        cap2_total+=score_l3(&cap2,&l1_pairs,swap);

        let mut no_engine=stage1.clone();
        freeze(&mut no_engine);
        no_engine_total+=score_l3(&no_engine,&l1_pairs,swap);

        let target=l3_pair_scene(&l1_pairs,0,L3_HELDOUT_LAYOUTS[0]);
        let node=l3_node_for_scene(&deep,&target);

        let mut lesioned=deep.clone();
        let saved=lesioned.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],0.0,0.0
        ).expect("G14 necessary L2->L3 synapse");
        lesion_total+=score_l3(&lesioned,&l1_pairs,swap);
        lesioned.restore_phase_native_synapse_for_control(
            node.child_synapses[0],saved
        );
        restored_total+=score_l3(&lesioned,&l1_pairs,swap);

        let mut shifted=deep.clone();
        shifted.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],1.0,std::f32::consts::PI
        ).expect("G14 phase intervention");
        phase_total+=score_l3(&shifted,&l1_pairs,swap);

        let lower_ref=node.children[0];
        let lower_node=deep.phase_native_deep_nodes().iter()
            .find(|candidate|
                candidate.promoted
                    && candidate.level==2
                    && candidate.id==lower_ref.id
            )
            .expect("G14 lower L2 node").clone();
        let mut lower=deep.clone();
        lower.perturb_phase_native_synapse_for_control(
            lower_node.child_synapses[0],0.0,0.0
        ).expect("G14 lower L1->L2 synapse");
        let expected=if swap {5}else{4};
        lower_lesion_ok+=usize::from(
            lower.choose_phase_native_depth_generic_action(&target)==Some(expected)
        );
    }

    println!(
        "G14_SELF_DEPTH safety={} simple={}/4 simple_highest={} simple_l3_candidates={} deep={}/8 cap2={}/8 no_engine={}/8 lesion={}/8 phase_shift={}/8 restored={}/8 lower_lesion_success={}/2",
        open_stage2.phase_native_deep_max_level().unwrap(),
        simple_score,
        g14_highest_promoted_level(&simple),
        simple.phase_native_deep_candidate_count(3),
        deep_total,
        cap2_total,
        no_engine_total,
        lesion_total,
        phase_total,
        restored_total,
        lower_lesion_ok
    );

    assert_eq!(simple_score,4);
    assert_eq!(g14_highest_promoted_level(&simple),2);
    assert_eq!(simple.phase_native_deep_candidate_count(3),0);
    assert_eq!(deep_total,8);
    assert!(cap2_total<=4);
    assert!(no_engine_total<=4);
    assert!(lesion_total<=6);
    assert!(phase_total<=6);
    assert_eq!(restored_total,8);
    assert_eq!(lower_lesion_ok,0);
}

#[test]
fn g14_open_depth_entrypoint_has_no_task_depth_argument_or_hidden_two_three() {
    let source=include_str!("../src/phase_deep_abstraction.rs");
    let start=source.find("pub fn enable_phase_native_open_depth_abstraction")
        .expect("G14 open-depth entrypoint");
    let end=source[start..].find("pub fn enable_phase_native_depth_generic_abstraction")
        .map(|offset|start+offset)
        .expect("legacy explicit-depth entrypoint follows G14");
    let entry=&source[start..end];

    for forbidden in [
        "max_level:",
        "(2)",
        "(3)",
        "G14_L2",
        "G14_L3",
        "correct_action",
    ] {
        assert!(
            !entry.contains(forbidden),
            "G14 open-depth entrypoint contains task-depth token {forbidden}"
        );
    }
    assert!(entry.contains("OPEN_DEPTH_SAFETY_CEILING"));
}


#[derive(Clone, Debug)]
struct FreshG14Block {
    base: FreshG13Block,
    simple_confirm: [[(usize,usize);4];4],
    simple_heldout: [[(usize,usize);4];4],
    simple_layout_rejections: usize,
}

fn fresh14_l2_atoms(base:&FreshG13Block,target:usize)->[usize;4] {
    let pair=base.l2_targets[target];
    [
        base.l1_pairs[pair[0]][0],
        base.l1_pairs[pair[0]][1],
        base.l1_pairs[pair[1]][0],
        base.l1_pairs[pair[1]][1],
    ]
}

fn fresh14_unique_layout4(
    atoms:&[usize;4],
    rng:&mut FreshG13Rng,
    used:&mut Vec<[(usize,usize);4]>,
    rejections:&mut usize,
)->[(usize,usize);4] {
    for _ in 0..100_000usize {
        let mut candidate=Vec::with_capacity(4);
        for &atom in atoms {
            let (dx,dy)=G13_OFFSETS[atom];
            candidate.push((rng.range(W-dx),rng.range(H-dy)));
        }
        let array:[(usize,usize);4]=candidate.try_into().unwrap();
        if used.contains(&array) || !fresh13_layout_valid(atoms,&array) {
            *rejections+=1;
            continue;
        }
        used.push(array);
        return array;
    }
    panic!("frozen geometry-only G14 L2 layout search exhausted");
}

fn fresh14_block(authority:u64,sub:u64)->FreshG14Block {
    let base=fresh13_block(
        authority ^ 0x14A4_D33F_C011_5EED,
        sub
    );
    let mut rng=FreshG13Rng::new(
        authority
            ^ sub.wrapping_mul(0xD1B5_4A32_D192_ED03)
            ^ 0x1414_0A37_E5E0_0014
    );
    let mut simple_confirm=[[(0usize,0usize);4];4];
    let mut simple_heldout=[[(0usize,0usize);4];4];
    let mut rejections=0usize;

    for target in 0..4 {
        let atoms=fresh14_l2_atoms(&base,target);
        let mut used=Vec::new();
        simple_confirm[target]=fresh14_unique_layout4(
            &atoms,&mut rng,&mut used,&mut rejections
        );
        simple_heldout[target]=fresh14_unique_layout4(
            &atoms,&mut rng,&mut used,&mut rejections
        );
    }

    FreshG14Block {
        base,
        simple_confirm,
        simple_heldout,
        simple_layout_rejections:rejections,
    }
}

fn fresh14_digest(blocks:&[FreshG14Block])->u64 {
    let mut h=14_695_981_039_346_656_037u64;
    for (sub,block) in blocks.iter().enumerate() {
        h=fresh13_fnv(h,sub as u64);
        for &v in &block.base.relation_perm { h=fresh13_fnv(h,v as u64); }
        for pair in block.base.l1_pairs { for v in pair { h=fresh13_fnv(h,v as u64); } }
        for pair in block.base.l2_targets { for v in pair { h=fresh13_fnv(h,v as u64); } }
        for pair in block.base.l3_targets { for v in pair { h=fresh13_fnv(h,v as u64); } }
        for &v in &block.base.motors { h=fresh13_fnv(h,v as u64); }
        for target in 0..4 {
            for (x,y) in block.simple_confirm[target] {
                h=fresh13_fnv(h,x as u64);
                h=fresh13_fnv(h,y as u64);
            }
            for (x,y) in block.simple_heldout[target] {
                h=fresh13_fnv(h,x as u64);
                h=fresh13_fnv(h,y as u64);
            }
        }
        for target in 0..2 {
            for layout in block.base.l3_tuition[target] {
                for (x,y) in layout {
                    h=fresh13_fnv(h,x as u64);
                    h=fresh13_fnv(h,y as u64);
                }
            }
            for layout in block.base.l3_heldout[target] {
                for (x,y) in layout {
                    h=fresh13_fnv(h,x as u64);
                    h=fresh13_fnv(h,y as u64);
                }
            }
        }
    }
    h
}

fn fresh14_l2_scene(
    block:&FreshG14Block,
    target:usize,
    layout:[(usize,usize);4],
)->Vec<f32> {
    let pair=block.base.l2_targets[target];
    concept_scene(
        &block.base.l1_pairs,
        &[pair[0],pair[1]],
        &layout,
    )
}

fn fresh14_confirm_simple(evo:&mut EvoPhase,block:&FreshG14Block) {
    let actions=[block.base.motors[2],block.base.motors[3]];
    for _ in 0..4 {
        for target in 0..4 {
            let scene=fresh14_l2_scene(
                block,target,block.simple_confirm[target]
            );
            let correct=if target%2==0 {actions[0]} else {actions[1]};
            for action in actions {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                ));
            }
        }
    }
}

fn fresh14_score_simple(evo:&EvoPhase,block:&FreshG14Block)->usize {
    let actions=[block.base.motors[2],block.base.motors[3]];
    let mut score=0usize;
    for target in 0..4 {
        let scene=fresh14_l2_scene(
            block,target,block.simple_heldout[target]
        );
        let expected=if target%2==0 {actions[0]} else {actions[1]};
        score+=usize::from(
            evo.choose_phase_native_depth_generic_action(&scene)==Some(expected)
        );
    }
    score
}

fn fresh14_confirm_deep(evo:&mut EvoPhase,block:&FreshG14Block) {
    let actions=[block.base.motors[4],block.base.motors[5]];
    for _ in 0..4 {
        for target in 0..2 {
            let scene=fresh13_l3_scene(
                &block.base,target,block.base.l3_tuition[target][0]
            );
            let correct=if target==0 {actions[0]} else {actions[1]};
            for action in actions {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==correct
                ));
            }
        }
    }
}

fn fresh14_wilson95(success:usize,n:usize)->(f64,f64) {
    fresh13_wilson95(success,n)
}

#[test]
#[ignore = "requires one-use AETERNA_FRESH_SEED from first-attempt CI"]
fn g14_fresh_self_selected_depth_pack() {
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse().expect("fresh G14 seed must be u64");
    let source_sha=std::env::var("AETERNA_SOURCE_SHA")
        .unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA")
        .unwrap_or_else(|_|"unknown".into());

    let blocks=(0..10u64)
        .map(|sub|fresh14_block(authority,sub))
        .collect::<Vec<_>>();
    let digest=fresh14_digest(&blocks);

    println!(
        "FRESH_G14_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (sub,block) in blocks.iter().enumerate() {
        println!("FRESH_G14_BLOCK sub={} {:?}",sub,block);
    }

    let mut simple_total=0usize;
    let mut deep_total=0usize;
    let mut simple_per_seed=Vec::new();
    let mut deep_per_seed=Vec::new();
    let mut simple_depth_violations=0usize;
    let mut simple_l3_violations=0usize;
    let mut deep_depth_violations=0usize;
    let mut deep_l4_violations=0usize;
    let mut safety_violations=0usize;
    let mut cap2_total=0usize;
    let mut no_engine_total=0usize;
    let mut lesion_ok=0usize;
    let mut phase_ok=0usize;
    let mut restored_ok=0usize;
    let mut lower_ok=0usize;
    let mut layout_rejections=0usize;

    for (sub,block) in blocks.iter().enumerate() {
        layout_rejections+=
            block.simple_layout_rejections+block.base.layout_rejections;

        let mut stage1=carrier();
        fresh13_train_l1(&mut stage1,&block.base);

        let mut open_stage2=stage1.clone();
        assert!(open_stage2.enable_phase_native_open_depth_abstraction());
        let ceiling=open_stage2.phase_native_deep_max_level().unwrap();
        safety_violations+=usize::from(ceiling<8);
        fresh13_train_l2(&mut open_stage2,&block.base);

        let mut simple=open_stage2.clone();
        fresh14_confirm_simple(&mut simple,block);
        simple_l3_violations+=usize::from(
            simple.phase_native_deep_candidate_count(3)!=0
        );
        simple_depth_violations+=usize::from(
            g14_highest_promoted_level(&simple)!=2
        );
        freeze(&mut simple);
        let simple_score=fresh14_score_simple(&simple,block);
        simple_total+=simple_score;
        simple_per_seed.push(simple_score);

        let mut deep=open_stage2.clone();
        fresh13_train_l3(&mut deep,&block.base,true);
        fresh14_confirm_deep(&mut deep,block);
        deep_l4_violations+=usize::from(
            deep.phase_native_deep_candidate_count(4)!=0
        );
        deep_depth_violations+=usize::from(
            g14_highest_promoted_level(&deep)!=3
        );
        freeze(&mut deep);
        let deep_score=fresh13_score(&deep,&block.base);
        deep_total+=deep_score;
        deep_per_seed.push(deep_score);

        let mut cap2=stage1.clone();
        assert!(cap2.enable_phase_native_depth_generic_abstraction(2));
        fresh13_train_l2(&mut cap2,&block.base);
        fresh13_train_l3(&mut cap2,&block.base,true);
        freeze(&mut cap2);
        cap2_total+=fresh13_score(&cap2,&block.base);

        let mut no_engine=stage1.clone();
        freeze(&mut no_engine);
        no_engine_total+=fresh13_score(&no_engine,&block.base);

        let actions=[block.base.motors[4],block.base.motors[5]];
        let expected=actions[0];
        let node=fresh13_l3_node(
            &deep,&block.base,0,block.base.l3_heldout[0][0]
        );

        let mut lesioned=deep.clone();
        let saved=lesioned.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],0.0,0.0
        ).expect("fresh G14 necessary L2->L3 synapse");
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(
                &block.base,0,block.base.l3_heldout[0][holdout]
            );
            lesion_ok+=usize::from(
                lesioned.choose_phase_native_depth_generic_action(&scene)
                    ==Some(expected)
            );
        }

        lesioned.restore_phase_native_synapse_for_control(
            node.child_synapses[0],saved
        );
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(
                &block.base,0,block.base.l3_heldout[0][holdout]
            );
            restored_ok+=usize::from(
                lesioned.choose_phase_native_depth_generic_action(&scene)
                    ==Some(expected)
            );
        }

        let mut shifted=deep.clone();
        shifted.perturb_phase_native_synapse_for_control(
            node.child_synapses[0],1.0,std::f32::consts::PI
        ).expect("fresh G14 phase intervention");
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(
                &block.base,0,block.base.l3_heldout[0][holdout]
            );
            phase_ok+=usize::from(
                shifted.choose_phase_native_depth_generic_action(&scene)
                    ==Some(expected)
            );
        }

        let l2_child=node.children[0];
        let l2_node=deep.phase_native_deep_nodes().iter()
            .find(|n|n.promoted && n.level==2 && n.id==l2_child.id)
            .expect("fresh G14 lower L2 node").clone();
        let mut lower=deep.clone();
        lower.perturb_phase_native_synapse_for_control(
            l2_node.child_synapses[0],0.0,0.0
        ).expect("fresh G14 lower L1->L2 dependency");
        for holdout in 0..2 {
            let scene=fresh13_l3_scene(
                &block.base,0,block.base.l3_heldout[0][holdout]
            );
            lower_ok+=usize::from(
                lower.choose_phase_native_depth_generic_action(&scene)
                    ==Some(expected)
            );
        }

        println!(
            "FRESH_G14_SUB sub={} safety={} simple={}/4 simple_highest={} simple_l3={} deep={}/8 deep_highest={} deep_l4={} rejected_layouts={}",
            sub,
            ceiling,
            simple_score,
            g14_highest_promoted_level(&simple),
            simple.phase_native_deep_candidate_count(3),
            deep_score,
            g14_highest_promoted_level(&deep),
            deep.phase_native_deep_candidate_count(4),
            block.simple_layout_rejections+block.base.layout_rejections,
        );
    }

    let (deep_lo,deep_hi)=fresh14_wilson95(deep_total,80);
    println!(
        "FRESH_G14_RESULT simple={}/40 simple_per_seed={:?} simple_depth_violations={} simple_l3_violations={} deep={}/80 wilson95=[{:.6},{:.6}] deep_per_seed={:?} deep_depth_violations={} deep_l4_violations={} safety_violations={} cap2={}/80 no_engine={}/80 lesion={}/20 phase_shift={}/20 restored={}/20 lower_lesion_success={}/20 layout_rejections={}",
        simple_total,
        simple_per_seed,
        simple_depth_violations,
        simple_l3_violations,
        deep_total,
        deep_lo,
        deep_hi,
        deep_per_seed,
        deep_depth_violations,
        deep_l4_violations,
        safety_violations,
        cap2_total,
        no_engine_total,
        lesion_ok,
        phase_ok,
        restored_ok,
        lower_ok,
        layout_rejections
    );

    assert!(simple_total>=38);
    assert!(simple_per_seed.iter().all(|score|*score>=3));
    assert_eq!(simple_depth_violations,0);
    assert_eq!(simple_l3_violations,0);
    assert!(deep_total>=76);
    assert!(deep_lo>=0.87);
    assert!(deep_per_seed.iter().all(|score|*score>=6));
    assert_eq!(deep_depth_violations,0);
    assert_eq!(deep_l4_violations,0);
    assert_eq!(safety_violations,0);
    assert!(cap2_total<=40);
    assert!(no_engine_total<=40);
    assert!(lesion_ok<=4);
    assert!(phase_ok<=4);
    assert!(restored_ok>=19);
    assert!(lower_ok<=1);
}
