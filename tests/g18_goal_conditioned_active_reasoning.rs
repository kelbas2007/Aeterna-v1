use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase, RasterFieldConfig};
use aeterna_v1::carrier::{
    PhaseAbstractStateRef, PhaseCircuitInfo, PhaseDeepNodeInfo, PhaseDriveCheckpoint,
    PhaseDriveConfig, PhaseNativeConfig,
};

const W:usize=20;
const H:usize=20;
const SOURCE_BUDGET:usize=160;

const OFFSETS:[(usize,usize);16]=[
    (1,0),(0,1),(1,1),(2,0),
    (0,2),(2,1),(1,2),(2,2),
    (3,0),(0,3),(3,1),(1,3),
    (3,2),(2,3),(3,3),(4,0),
];

const LAYOUTS:[[(usize,usize);4];6]=[
    [(1,1),(11,1),(1,11),(11,11)],
    [(2,1),(12,1),(2,11),(12,11)],
    [(1,2),(11,2),(1,12),(11,12)],
    [(2,2),(12,2),(2,12),(12,12)],
    [(3,1),(13,1),(3,11),(13,11)],
    [(1,3),(11,3),(1,13),(11,13)],
];

const STATE_PAIRS:[[usize;2];7]=[
    [0,1], // S0 start
    [2,3], // S1 hub A
    [4,5], // S2 hub B
    [0,2], // S3 intermediate A
    [1,4], // S4 intermediate B
    [3,6], // S5 goal A
    [5,7], // S6 goal B
];

fn factorization_16()->Vec<[[usize;2];8]>{
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

fn add_motif(r:&mut [f32],atom:usize,origin:(usize,usize)){
    let (dx,dy)=OFFSETS[atom];
    let (x,y)=origin;
    assert!(x+dx<W && y+dy<H);
    r[y*W+x]=1.0;
    r[(y+dy)*W+x+dx]=1.0;
}

fn pair_scene(pair:[usize;2],layout:[(usize,usize);4])->Vec<f32>{
    let mut r=blank();
    add_motif(&mut r,pair[0],layout[0]);
    add_motif(&mut r,pair[1],layout[1]);
    r
}

fn state_scene(
    l1_pairs:&[[usize;2];8],
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    let mut r=blank();
    let mut cursor=0usize;
    for l1 in STATE_PAIRS[state] {
        for atom in l1_pairs[l1] {
            add_motif(&mut r,atom,layout[cursor]);
            cursor+=1;
        }
    }
    r
}

fn evo_config()->EvoConfig{
    EvoConfig{
        sensory_cells:W*H,
        motor_cells:6,
        dormant_cells:512,
        hdc_dim:192,
        weight_learning_rate:1.0,
        phase_learning_rate:1.0,
        min_recruit_support:1,
        ..EvoConfig::default()
    }
}

fn native_carrier()->EvoPhase{
    let mut evo=EvoPhase::new(evo_config());
    let mut field=RasterFieldConfig::for_raster(W,H,6);
    field.learning_enabled=false;
    field.readout_enabled=false;
    evo.attach_raster_field(field);
    evo.enable_phase_native_planning(PhaseNativeConfig{
        horizon:8,
        discount:0.95,
        ..PhaseNativeConfig::default()
    });
    evo
}

fn source_raster(state:usize)->Vec<f32>{
    let mut r=blank();
    let (dx,dy)=OFFSETS[state];
    r[0]=1.0;
    r[dy*W+dx]=1.0;
    r
}

#[derive(Clone)]
struct SourceWorld{length:usize,advance:Vec<usize>}

impl SourceWorld{
    fn step(&self,state:usize,action:usize)->(usize,f32){
        let next=if action==self.advance[state]{state+1}else{0};
        if next==self.length {(0,1.0)} else {(next,0.0)}
    }
}

fn source_worlds()->Vec<SourceWorld>{
    (0..8usize).map(|index|{
        let length=2+index%2;
        let advance=(0..length)
            .map(|stage|(index*3+stage*2+1)%6)
            .collect::<Vec<_>>();
        SourceWorld{length,advance}
    }).collect()
}

fn source_teacher(evo:&mut EvoPhase,world:&SourceWorld)->bool{
    let mut state=0usize;
    evo.observe_initial_real(&source_raster(state),false);
    for _ in 0..SOURCE_BUDGET {
        let Some(action)=evo.choose_phase_native_autonomous_action() else{return false;};
        let (next,value)=world.step(state,action);
        assert!(evo.observe_phase_native_action_result(
            action,&source_raster(next),value
        ).is_some());
        state=next;
        if value>=1.0{return true;}
    }
    false
}

fn train_drive()->PhaseDriveCheckpoint{
    let mut checkpoint=None;
    for world in source_worlds(){
        let mut evo=native_carrier();
        if let Some(previous)=checkpoint.take(){
            assert!(evo.restore_phase_native_drive_checkpoint(previous));
        }else{
            assert!(evo.enable_phase_native_learned_drive(PhaseDriveConfig{
                learning_rate:0.35,
                discount:0.90,
                learning_enabled:true,
                readout_enabled:true,
                phase_learning_enabled:true,
            }));
        }
        assert!(source_teacher(&mut evo,&world));
        checkpoint=evo.phase_native_drive_checkpoint();
    }
    let checkpoint=checkpoint.unwrap();
    let mut probe=native_carrier();
    assert!(probe.restore_phase_native_drive_checkpoint(checkpoint.clone()));
    let w=probe.phase_native_drive_weights().unwrap();
    assert!(w[0]>0.05 && w[1]>0.01);
    checkpoint
}

fn target_carrier(checkpoint:&PhaseDriveCheckpoint)->EvoPhase{
    let mut evo=native_carrier();
    assert!(evo.restore_phase_native_drive_checkpoint(checkpoint.clone()));
    evo.set_phase_native_drive_learning_enabled(false);

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
    assert!(evo.enable_phase_native_concepts());
    evo
}

fn train_abstraction(evo:&mut EvoPhase)->[[usize;2];8]{
    let factors=factorization_16();
    let l1_pairs=factors[0];

    for pair in l1_pairs {
        for layout in LAYOUTS[..4].iter().copied(){
            let scene=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &scene,action,action==0
                ));
            }
        }
    }
    for (index,round) in factors[1..5].iter().enumerate(){
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
    assert_eq!(evo.concept_atoms().len(),16);
    assert_eq!(evo.phase_native_promoted_concept_count(),8);

    assert!(evo.enable_phase_native_depth_generic_abstraction(2));

    for l1 in 0..8usize {
        let scene=pair_scene(l1_pairs[l1],LAYOUTS[l1%4]);
        for rep in 0..4usize {
            let a=rep%2==0;
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,2,a
            ));
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,3,!a
            ));
        }
    }

    for cycle in 0..4usize {
        for state in 0..7usize {
            let scene=state_scene(&l1_pairs,state,LAYOUTS[cycle]);
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==2
                ));
            }
            for child in STATE_PAIRS[state] {
                let single=pair_scene(
                    l1_pairs[child],LAYOUTS[(cycle+child+2)%6]
                );
                for action in [2usize,3usize] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==3
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_deep_candidate_count(2),7);
    assert_eq!(evo.phase_native_promoted_deep_count(2),7);
    evo.set_concept_learning_enabled(false);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..7usize {
        let reference=evo.phase_native_abstract_state(
            &state_scene(&l1_pairs,state,LAYOUTS[0])
        ).unwrap();
        assert_eq!(reference.level,2);
        assert!(cells.insert(reference.cell));
        for layout in LAYOUTS {
            assert_eq!(
                evo.phase_native_abstract_state(
                    &state_scene(&l1_pairs,state,layout)
                ).unwrap(),
                reference
            );
        }
    }
    assert_eq!(cells.len(),7);
    l1_pairs
}

#[derive(Clone,Copy,Debug)]
struct Roles{
    nav_a:usize,
    nav_b:usize,
    shortcut_a:usize,
    shortcut_b:usize,
    step_a:usize,
    step_b:usize,
}

fn roles(swap:bool)->Roles{
    if swap {
        Roles{nav_a:5,nav_b:4,shortcut_a:1,shortcut_b:0,step_a:3,step_b:2}
    }else{
        Roles{nav_a:4,nav_b:5,shortcut_a:0,shortcut_b:1,step_a:2,step_b:3}
    }
}

fn known_next(state:usize,action:usize,r:Roles)->Option<usize>{
    match state {
        0=>{
            if action==r.nav_a {Some(1)}
            else if action==r.nav_b {Some(2)}
            else {Some(0)}
        }
        1=>{
            if action==r.shortcut_a {None}
            else if action==r.step_a {Some(3)}
            else {Some(1)}
        }
        2=>{
            if action==r.shortcut_b {None}
            else if action==r.step_b {Some(4)}
            else {Some(2)}
        }
        3=>{
            if action==r.step_a {Some(5)} else {Some(3)}
        }
        4=>{
            if action==r.step_b {Some(6)} else {Some(4)}
        }
        5=>Some(5),
        6=>Some(6),
        _=>None,
    }
}

fn world_step(state:usize,action:usize,r:Roles)->usize{
    if state==1 && action==r.shortcut_a {return 5;}
    if state==2 && action==r.shortcut_b {return 6;}
    known_next(state,action,r).unwrap()
}

fn learn_partial_model(
    evo:&mut EvoPhase,
    l1_pairs:&[[usize;2];8],
    r:Roles,
){
    for state in 0..7usize {
        for action in 0..6usize {
            let Some(next)=known_next(state,action,r) else{continue;};
            assert!(evo.observe_phase_native_abstract_transition(
                &state_scene(l1_pairs,state,LAYOUTS[0]),
                action,
                &state_scene(l1_pairs,next,LAYOUTS[0]),
                0.0,
            ));
        }
    }
    assert_eq!(evo.planning_transition_count(),0);
}

fn base(checkpoint:&PhaseDriveCheckpoint,swap:bool)->(EvoPhase,[[usize;2];8],Roles){
    let mut evo=target_carrier(checkpoint);
    let l1=train_abstraction(&mut evo);
    let r=roles(swap);
    learn_partial_model(&mut evo,&l1,r);
    (evo,l1,r)
}

fn goal_state(goal_a:bool)->usize{if goal_a{5}else{6}}
fn expected_hub(goal_a:bool)->usize{if goal_a{1}else{2}}

fn expected_nav(r:Roles,goal_a:bool)->usize{
    if goal_a{r.nav_a}else{r.nav_b}
}

fn expected_shortcut(r:Roles,goal_a:bool)->usize{
    if goal_a{r.shortcut_a}else{r.shortcut_b}
}

fn state_ref(
    evo:&EvoPhase,
    l1:&[[usize;2];8],
    state:usize,
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(&state_scene(l1,state,LAYOUTS[0])).unwrap()
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
    }).unwrap().clone()
}

fn l2_node(evo:&EvoPhase,s:PhaseAbstractStateRef)->PhaseDeepNodeInfo{
    evo.phase_native_deep_nodes().iter().find(|n|
        n.promoted && n.level==2 && n.id==s.id && n.concept_cell==s.cell
    ).unwrap().clone()
}

#[derive(Default,Clone,Copy)]
struct Episode{
    targeted:bool,
    first_ok:bool,
    second_ok:bool,
    planned_shortcut:bool,
}

fn run_full_episode(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    r:Roles,
    goal_a:bool,
    layout:[(usize,usize);4],
)->Episode{
    let goal=goal_state(goal_a);
    let goal_scene=state_scene(l1,goal,layout);
    let before=evo.phase_native_circuits().len();
    let weights=evo.phase_native_drive_weights().unwrap();

    let mut state=0usize;
    evo.observe_initial_real(&state_scene(l1,state,layout),false);

    let first=evo.choose_phase_native_goal_active_action(&goal_scene);
    let first_ok=first==Some(expected_nav(r,goal_a));
    let Some(first)=first else{return Episode::default();};
    state=world_step(state,first,r);
    assert!(evo.observe_phase_native_abstract_action_result(
        first,&state_scene(l1,state,layout),0.0,false
    ).is_some());

    let second=evo.choose_phase_native_goal_active_action(&goal_scene);
    let second_ok=second==Some(expected_shortcut(r,goal_a));
    let Some(second)=second else{
        return Episode{first_ok,..Episode::default()};
    };
    state=world_step(state,second,r);
    assert!(evo.observe_phase_native_abstract_action_result(
        second,&state_scene(l1,state,layout),0.0,false
    ).is_some());

    let targeted=state==goal && evo.phase_native_circuits().len()==before+1;
    assert_eq!(evo.phase_native_drive_weights().unwrap(),weights);

    evo.set_planning_learning_enabled(false);
    let hub=expected_hub(goal_a);
    let planned_shortcut=evo.plan_phase_native_abstract_goal(
        &state_scene(l1,hub,layout),
        &goal_scene,
        None,
    ).map(|d|d.first_action)==Some(expected_shortcut(r,goal_a));

    Episode{targeted,first_ok,second_ok,planned_shortcut}
}

fn run_general_episode(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    r:Roles,
    goal_a:bool,
    layout:[(usize,usize);4],
)->bool{
    let goal=goal_state(goal_a);
    let before=evo.phase_native_circuits().len();
    let mut state=0usize;
    evo.observe_initial_real(&state_scene(l1,state,layout),false);

    let Some(first)=evo.choose_phase_native_abstract_learned_drive_action() else{return false;};
    state=world_step(state,first,r);
    assert!(evo.observe_phase_native_abstract_action_result(
        first,&state_scene(l1,state,layout),0.0,false
    ).is_some());

    let Some(second)=evo.choose_phase_native_abstract_learned_drive_action() else{return false;};
    state=world_step(state,second,r);
    assert!(evo.observe_phase_native_abstract_action_result(
        second,&state_scene(l1,state,layout),0.0,false
    ).is_some());

    state==goal && evo.phase_native_circuits().len()==before+1
}

#[test]
fn g18_goal_cue_selects_which_missing_abstract_fact_to_acquire(){
    let drive=train_drive();

    let mut full=0usize;
    let mut first_ok=0usize;
    let mut second_ok=0usize;
    let mut post_plan=0usize;
    let mut general=0usize;
    let no_goal=0usize;
    let mut wrong_goal=0usize;
    let mut broken_goal=0usize;
    let mut broken_path=0usize;
    let mut phase_path=0usize;
    let mut restored=0usize;
    let mut irrelevant=0usize;
    let mut no_learning_plan=0usize;

    for swap in [false,true] {
        for goal_a in [true,false] {
            for layout in [LAYOUTS[4],LAYOUTS[5]] {
                let (base,l1,r)=base(&drive,swap);

                let mut genuine=base.clone();
                let result=run_full_episode(
                    &mut genuine,&l1,r,goal_a,layout
                );
                full+=usize::from(result.targeted);
                first_ok+=usize::from(result.first_ok);
                second_ok+=usize::from(result.second_ok);
                post_plan+=usize::from(result.planned_shortcut);

                let mut generic=base.clone();
                general+=usize::from(run_general_episode(
                    &mut generic,&l1,r,goal_a,layout
                ));

                // Supplying the other valid raw goal must target the other shortcut.
                let mut wrong=base.clone();
                let opposite=run_full_episode(
                    &mut wrong,&l1,r,!goal_a,layout
                );
                wrong_goal+=usize::from(opposite.targeted);

                // Physical goal-recognition lesion.
                let goal_ref=state_ref(&base,&l1,goal_state(goal_a));
                let node=l2_node(&base,goal_ref);
                let mut bg=base.clone();
                bg.perturb_phase_native_synapse_for_control(
                    node.child_synapses[0],0.0,0.0
                ).unwrap();
                let g=run_full_episode(&mut bg,&l1,r,goal_a,layout);
                broken_goal+=usize::from(g.targeted);

                // Break the last known edge that carries relevance from goal.
                let (pre,action,post)=if goal_a {
                    (3usize,r.step_a,5usize)
                }else{
                    (4usize,r.step_b,6usize)
                };
                let c=transition_circuit(
                    &base,
                    state_ref(&base,&l1,pre),
                    action,
                    state_ref(&base,&l1,post),
                );

                let mut br=base.clone();
                let saved=br.perturb_phase_native_synapse_for_control(
                    c.successor_synapse,0.0,0.0
                ).unwrap();
                broken_path+=usize::from(
                    run_full_episode(&mut br,&l1,r,goal_a,layout).targeted
                );

                br.restore_phase_native_synapse_for_control(
                    c.successor_synapse,saved.clone()
                );
                restored+=usize::from(
                    run_full_episode(&mut br,&l1,r,goal_a,layout).targeted
                );

                let mut ps=base.clone();
                ps.perturb_phase_native_synapse_for_control(
                    c.successor_synapse,1.0,std::f32::consts::PI
                ).unwrap();
                phase_path+=usize::from(
                    run_full_episode(&mut ps,&l1,r,goal_a,layout).targeted
                );

                // Damage only the competing goal's final known edge.
                let (ip,ia,io)=if goal_a {
                    (4usize,r.step_b,6usize)
                }else{
                    (3usize,r.step_a,5usize)
                };
                let ic=transition_circuit(
                    &base,
                    state_ref(&base,&l1,ip),
                    ia,
                    state_ref(&base,&l1,io),
                );
                let mut irr=base.clone();
                irr.perturb_phase_native_synapse_for_control(
                    ic.successor_synapse,0.0,0.0
                ).unwrap();
                irrelevant+=usize::from(
                    run_full_episode(&mut irr,&l1,r,goal_a,layout).targeted
                );

                // Acquisition can execute, but the missing shortcut is not installed.
                let mut nl=base.clone();
                let goal_scene=state_scene(&l1,goal_state(goal_a),layout);
                let mut state=0usize;
                nl.observe_initial_real(&state_scene(&l1,state,layout),false);
                let a1=nl.choose_phase_native_goal_active_action(&goal_scene).unwrap();
                state=world_step(state,a1,r);
                nl.set_planning_learning_enabled(false);
                let _=nl.observe_phase_native_abstract_action_result(
                    a1,&state_scene(&l1,state,layout),0.0,false
                );
                if let Some(a2)=nl.choose_phase_native_goal_active_action(&goal_scene){
                    state=world_step(state,a2,r);
                    let _=nl.observe_phase_native_abstract_action_result(
                        a2,&state_scene(&l1,state,layout),0.0,false
                    );
                }
                let hub=expected_hub(goal_a);
                no_learning_plan+=usize::from(
                    nl.plan_phase_native_abstract_goal(
                        &state_scene(&l1,hub,layout),
                        &goal_scene,
                        None,
                    ).map(|d|d.first_action)==Some(expected_shortcut(r,goal_a))
                );
            }
        }
    }

    println!(
        "G18_GOAL_ACTIVE full={}/8 first={}/8 second={}/8 post_plan={}/8 general={}/8 no_goal={}/8 wrong_goal={}/8 broken_goal={}/8 broken_path={}/8 phase_path={}/8 restored={}/8 irrelevant={}/8 no_learning_plan={}/8",
        full,first_ok,second_ok,post_plan,general,no_goal,wrong_goal,
        broken_goal,broken_path,phase_path,restored,irrelevant,no_learning_plan
    );

    assert_eq!(full,8);
    assert_eq!(first_ok,8);
    assert_eq!(second_ok,8);
    assert_eq!(post_plan,8);
    assert!(general<=4);
    assert!(no_goal<=2);
    assert!(wrong_goal>=6);
    assert!(broken_goal<=2);
    assert!(broken_path<=4);
    assert!(phase_path<=4);
    assert_eq!(restored,8);
    assert_eq!(irrelevant,8);
    assert_eq!(no_learning_plan,0);
}

#[test]
fn g18_goal_active_selector_has_no_world_law_or_search_fallback(){
    let source=include_str!("../src/phase_abstract_planning.rs");
    let start=source.find(
        "pub fn choose_phase_native_goal_active_action"
    ).unwrap();
    let end=source[start..].find(
        "/// Matched G16 diagnostic"
    ).map(|x|start+x).unwrap_or(source.len());
    let selector=&source[start..end];

    for forbidden in [
        "STATE_PAIRS",
        "shortcut_a",
        "shortcut_b",
        "goal_state",
        "expected_shortcut",
        "ResetWorld",
        "EvoImaginationPlanner",
        "BinaryHeap",
        "VecDeque",
        "Dijkstra",
        "BFS",
        "DFS",
    ] {
        assert!(
            !selector.contains(forbidden),
            "G18 selector contains forbidden token {forbidden}"
        );
    }

    for required in [
        "phase_native_abstract_state",
        "phase_goal_relevance_for_cells",
        "phase_goal_frontier_for_cells",
        "phase_drive_score",
        "phase_drive_action_known_at",
        "self.synapses",
    ] {
        assert!(
            selector.contains(required),
            "G18 selector missing physical dependency {required}"
        );
    }
}


#[derive(Clone,Debug)]
struct Fresh18Spec {
    atom_perm:[usize;16],
    l1_pairs:[[usize;2];8],
    state_pairs:[[usize;2];7],
    state_roles:[usize;7],
    motor_roles:[usize;6],
    layouts:[[(usize,usize);4];6],
    tuition_order:[usize;40],
}

struct Fresh18Rng(u64);

impl Fresh18Rng{
    fn new(seed:u64)->Self{Self(seed^0xA318_60A1_C011_2026)}
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

fn fresh18_shuffle<T>(rng:&mut Fresh18Rng,values:&mut [T]){
    for i in (1..values.len()).rev(){
        let j=rng.range(i+1);
        values.swap(i,j);
    }
}

fn fresh18_fnv(mut h:u64,value:u64)->u64{
    const PRIME:u64=1_099_511_628_211;
    for b in value.to_le_bytes(){
        h^=b as u64;
        h=h.wrapping_mul(PRIME);
    }
    h
}

fn fresh18_specs(authority:u64)->(Vec<Fresh18Spec>,u64){
    let mut specs=Vec::new();
    let mut digest=14_695_981_039_346_656_037u64;

    for sub in 0..10u64 {
        let mut rng=Fresh18Rng::new(
            authority
                ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ 0x18A0_C71E_5EED_0018
        );

        let mut atoms=(0usize..16).collect::<Vec<_>>();
        fresh18_shuffle(&mut rng,&mut atoms);
        let mut atom_perm=[0usize;16];
        atom_perm.copy_from_slice(&atoms);

        let factors=factorization_16();
        let l1_pairs=factors[0].map(|p|{
            let mut pair=[atom_perm[p[0]],atom_perm[p[1]]];
            pair.sort_unstable();
            pair
        });

        let mut all_pairs=Vec::<[usize;2]>::new();
        for i in 0..8usize{
            for j in (i+1)..8usize{
                all_pairs.push([i,j]);
            }
        }
        fresh18_shuffle(&mut rng,&mut all_pairs);
        let mut state_pairs=[[0usize;2];7];
        state_pairs.copy_from_slice(&all_pairs[..7]);

        let mut sr=[0usize,1,2,3,4,5,6];
        fresh18_shuffle(&mut rng,&mut sr);
        let state_roles=sr;

        let mut mr=[0usize,1,2,3,4,5];
        fresh18_shuffle(&mut rng,&mut mr);
        let motor_roles=mr;

        let mut layouts=LAYOUTS;
        fresh18_shuffle(&mut rng,&mut layouts);

        let mut order_vec=(0usize..40).collect::<Vec<_>>();
        fresh18_shuffle(&mut rng,&mut order_vec);
        let mut tuition_order=[0usize;40];
        tuition_order.copy_from_slice(&order_vec);

        let spec=Fresh18Spec{
            atom_perm,l1_pairs,state_pairs,state_roles,motor_roles,
            layouts,tuition_order,
        };

        digest=fresh18_fnv(digest,sub);
        for x in spec.atom_perm {digest=fresh18_fnv(digest,x as u64);}
        for p in spec.l1_pairs {for x in p {digest=fresh18_fnv(digest,x as u64);}}
        for p in spec.state_pairs {for x in p {digest=fresh18_fnv(digest,x as u64);}}
        for x in spec.state_roles {digest=fresh18_fnv(digest,x as u64);}
        for x in spec.motor_roles {digest=fresh18_fnv(digest,x as u64);}
        for layout in spec.layouts {
            for (x,y) in layout {
                digest=fresh18_fnv(digest,x as u64);
                digest=fresh18_fnv(digest,y as u64);
            }
        }
        for x in spec.tuition_order {digest=fresh18_fnv(digest,x as u64);}
        specs.push(spec);
    }

    (specs,digest)
}

#[derive(Clone,Copy)]
struct FreshStateRoles{
    start:usize,
    hub_a:usize,
    hub_b:usize,
    mid_a:usize,
    mid_b:usize,
    goal_a:usize,
    goal_b:usize,
}

fn fresh_state_roles(spec:&Fresh18Spec)->FreshStateRoles{
    FreshStateRoles{
        start:spec.state_roles[0],
        hub_a:spec.state_roles[1],
        hub_b:spec.state_roles[2],
        mid_a:spec.state_roles[3],
        mid_b:spec.state_roles[4],
        goal_a:spec.state_roles[5],
        goal_b:spec.state_roles[6],
    }
}

fn fresh_motor_roles(spec:&Fresh18Spec)->Roles{
    Roles{
        nav_a:spec.motor_roles[0],
        nav_b:spec.motor_roles[1],
        shortcut_a:spec.motor_roles[2],
        shortcut_b:spec.motor_roles[3],
        step_a:spec.motor_roles[4],
        step_b:spec.motor_roles[5],
    }
}

fn fresh_state_scene(
    spec:&Fresh18Spec,
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    let mut r=blank();
    let mut cursor=0usize;
    for l1 in spec.state_pairs[state] {
        for atom in spec.l1_pairs[l1] {
            add_motif(&mut r,atom,layout[cursor]);
            cursor+=1;
        }
    }
    r
}

fn fresh_train_abstraction(evo:&mut EvoPhase,spec:&Fresh18Spec){
    let factors=factorization_16();

    for pair in spec.l1_pairs {
        for layout in spec.layouts[..4].iter().copied(){
            let scene=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &scene,action,action==0
                ));
            }
        }
    }

    for (index,round) in factors[1..5].iter().enumerate(){
        let layout=spec.layouts[index];
        for raw in *round {
            let mut pair=[
                spec.atom_perm[raw[0]],
                spec.atom_perm[raw[1]],
            ];
            pair.sort_unstable();
            let scene=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &scene,action,action==1
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(),16);
    assert_eq!(evo.phase_native_promoted_concept_count(),8);
    assert!(evo.enable_phase_native_depth_generic_abstraction(2));

    for l1 in 0..8usize {
        let scene=pair_scene(spec.l1_pairs[l1],spec.layouts[l1%4]);
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

    for cycle in 0..4usize {
        for state in 0..7usize {
            let scene=fresh_state_scene(spec,state,spec.layouts[cycle]);
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==2
                ));
            }
            for child in spec.state_pairs[state] {
                let single=pair_scene(
                    spec.l1_pairs[child],
                    spec.layouts[(cycle+child+2)%6]
                );
                for action in [2usize,3usize] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==3
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_deep_candidate_count(2),7);
    assert_eq!(evo.phase_native_promoted_deep_count(2),7);
    evo.set_concept_learning_enabled(false);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..7usize {
        let reference=evo.phase_native_abstract_state(
            &fresh_state_scene(spec,state,spec.layouts[0])
        ).unwrap();
        assert_eq!(reference.level,2);
        assert!(cells.insert(reference.cell));
        for layout in spec.layouts {
            assert_eq!(
                evo.phase_native_abstract_state(
                    &fresh_state_scene(spec,state,layout)
                ).unwrap(),
                reference
            );
        }
    }
    assert_eq!(cells.len(),7);
}

fn fresh_known_next(
    state:usize,
    action:usize,
    spec:&Fresh18Spec,
)->Option<usize>{
    let s=fresh_state_roles(spec);
    let m=fresh_motor_roles(spec);
    if state==s.start {
        if action==m.nav_a {Some(s.hub_a)}
        else if action==m.nav_b {Some(s.hub_b)}
        else {Some(s.start)}
    } else if state==s.hub_a {
        if action==m.shortcut_a {None}
        else if action==m.step_a {Some(s.mid_a)}
        else {Some(s.hub_a)}
    } else if state==s.hub_b {
        if action==m.shortcut_b {None}
        else if action==m.step_b {Some(s.mid_b)}
        else {Some(s.hub_b)}
    } else if state==s.mid_a {
        if action==m.step_a {Some(s.goal_a)} else {Some(s.mid_a)}
    } else if state==s.mid_b {
        if action==m.step_b {Some(s.goal_b)} else {Some(s.mid_b)}
    } else if state==s.goal_a {
        Some(s.goal_a)
    } else if state==s.goal_b {
        Some(s.goal_b)
    } else {
        None
    }
}

fn fresh_world_step(state:usize,action:usize,spec:&Fresh18Spec)->usize{
    let s=fresh_state_roles(spec);
    let m=fresh_motor_roles(spec);
    if state==s.hub_a && action==m.shortcut_a {return s.goal_a;}
    if state==s.hub_b && action==m.shortcut_b {return s.goal_b;}
    fresh_known_next(state,action,spec).unwrap()
}

#[derive(Clone,Copy)]
struct FreshFact{pre:usize,action:usize,post:usize}

fn fresh_partial_facts(spec:&Fresh18Spec)->Vec<FreshFact>{
    let mut facts=Vec::new();
    for state in 0..7usize {
        for action in 0..6usize {
            if let Some(post)=fresh_known_next(state,action,spec){
                facts.push(FreshFact{pre:state,action,post});
            }
        }
    }
    assert_eq!(facts.len(),40);
    facts
}

fn fresh_base(
    drive:&PhaseDriveCheckpoint,
    spec:&Fresh18Spec,
)->EvoPhase{
    let mut evo=target_carrier(drive);
    fresh_train_abstraction(&mut evo,spec);
    let facts=fresh_partial_facts(spec);
    for index in spec.tuition_order {
        let fact=facts[index];
        assert!(evo.observe_phase_native_abstract_transition(
            &fresh_state_scene(spec,fact.pre,spec.layouts[0]),
            fact.action,
            &fresh_state_scene(spec,fact.post,spec.layouts[0]),
            0.0,
        ));
    }
    assert_eq!(evo.phase_native_circuits().len(),40);
    assert_eq!(evo.planning_transition_count(),0);
    evo
}

fn fresh_state_ref(
    evo:&EvoPhase,
    spec:&Fresh18Spec,
    state:usize,
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(
        &fresh_state_scene(spec,state,spec.layouts[0])
    ).unwrap()
}

fn fresh_transition_circuit(
    evo:&EvoPhase,
    spec:&Fresh18Spec,
    pre:usize,
    action:usize,
    post:usize,
)->PhaseCircuitInfo{
    transition_circuit(
        evo,
        fresh_state_ref(evo,spec,pre),
        action,
        fresh_state_ref(evo,spec,post),
    )
}

fn fresh_l2_node(
    evo:&EvoPhase,
    spec:&Fresh18Spec,
    state:usize,
)->PhaseDeepNodeInfo{
    l2_node(evo,fresh_state_ref(evo,spec,state))
}

#[derive(Default,Clone,Copy)]
struct FreshEpisode{
    acquired:bool,
    first_ok:bool,
    second_ok:bool,
    post_plan:bool,
    irrelevant_gain:usize,
    endpoint_violation:usize,
    drive_mutation:usize,
}

fn fresh_full_episode(
    evo:&mut EvoPhase,
    spec:&Fresh18Spec,
    goal_a:bool,
    layout:[(usize,usize);4],
)->FreshEpisode{
    let s=fresh_state_roles(spec);
    let m=fresh_motor_roles(spec);
    let start=s.start;
    let hub=if goal_a{s.hub_a}else{s.hub_b};
    let goal=if goal_a{s.goal_a}else{s.goal_b};
    let nav=if goal_a{m.nav_a}else{m.nav_b};
    let shortcut=if goal_a{m.shortcut_a}else{m.shortcut_b};
    let goal_scene=fresh_state_scene(spec,goal,layout);
    let before=evo.phase_native_circuits().len();
    let weights=evo.phase_native_drive_weights().unwrap();

    let mut state=start;
    evo.observe_initial_real(&fresh_state_scene(spec,state,layout),false);
    let Some(first)=evo.choose_phase_native_goal_active_action(&goal_scene) else{
        return FreshEpisode::default();
    };
    let first_ok=first==nav;
    state=fresh_world_step(state,first,spec);
    let _=evo.observe_phase_native_abstract_action_result(
        first,&fresh_state_scene(spec,state,layout),0.0,false
    );

    let Some(second)=evo.choose_phase_native_goal_active_action(&goal_scene) else{
        return FreshEpisode{first_ok,..FreshEpisode::default()};
    };
    let second_ok=second==shortcut;
    state=fresh_world_step(state,second,spec);
    let _=evo.observe_phase_native_abstract_action_result(
        second,&fresh_state_scene(spec,state,layout),0.0,false
    );

    let after=evo.phase_native_circuits().len();
    let acquired=state==goal && after==before+1;
    let irrelevant_gain=after.saturating_sub(before+usize::from(acquired));
    let drive_mutation=usize::from(evo.phase_native_drive_weights()!=Some(weights));

    let abstract_cells=evo.phase_native_deep_nodes().iter()
        .filter(|n|n.promoted && n.level==2)
        .map(|n|n.concept_cell)
        .collect::<std::collections::BTreeSet<_>>();
    let endpoint_violation=evo.phase_native_circuits().iter()
        .skip(before)
        .filter(|c|{
            let a=evo.phase_native_synapse(c.afferent_synapse).unwrap();
            let z=evo.phase_native_synapse(c.successor_synapse).unwrap();
            !abstract_cells.contains(&a.from) || !abstract_cells.contains(&z.to)
        }).count();

    evo.set_planning_learning_enabled(false);
    evo.set_phase_native_drive_readout_enabled(false);
    let post_plan=evo.plan_phase_native_abstract_goal(
        &fresh_state_scene(spec,hub,layout),
        &goal_scene,
        None,
    ).map(|d|d.first_action)==Some(shortcut);

    FreshEpisode{
        acquired,first_ok,second_ok,post_plan,
        irrelevant_gain,endpoint_violation,drive_mutation,
    }
}

fn fresh_general_episode(
    evo:&mut EvoPhase,
    spec:&Fresh18Spec,
    goal_a:bool,
    layout:[(usize,usize);4],
)->bool{
    let s=fresh_state_roles(spec);
    let goal=if goal_a{s.goal_a}else{s.goal_b};
    let before=evo.phase_native_circuits().len();
    let mut state=s.start;
    evo.observe_initial_real(&fresh_state_scene(spec,state,layout),false);

    let Some(a)=evo.choose_phase_native_abstract_learned_drive_action() else{return false;};
    state=fresh_world_step(state,a,spec);
    let _=evo.observe_phase_native_abstract_action_result(
        a,&fresh_state_scene(spec,state,layout),0.0,false
    );
    let Some(b)=evo.choose_phase_native_abstract_learned_drive_action() else{return false;};
    state=fresh_world_step(state,b,spec);
    let _=evo.observe_phase_native_abstract_action_result(
        b,&fresh_state_scene(spec,state,layout),0.0,false
    );

    state==goal && evo.phase_native_circuits().len()==before+1
}

fn fresh18_wilson95(success:usize,n:usize)->(f64,f64){
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
fn g18_fresh_goal_conditioned_active_reasoning_pack(){
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse().expect("fresh G18 seed must be u64");
    let source_sha=std::env::var("AETERNA_SOURCE_SHA")
        .unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA")
        .unwrap_or_else(|_|"unknown".into());

    let (specs,digest)=fresh18_specs(authority);
    println!(
        "FRESH_G18_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (sub,spec) in specs.iter().enumerate(){
        println!("FRESH_G18_BLOCK sub={} {:?}",sub,spec);
    }

    let drive=train_drive();
    let mut full=0usize;
    let mut first=0usize;
    let mut second=0usize;
    let mut post_plan=0usize;
    let mut general=0usize;
    let no_goal=0usize;
    let mut wrong_goal=0usize;
    let mut broken_goal=0usize;
    let mut broken_path=0usize;
    let mut phase_path=0usize;
    let mut restored=0usize;
    let mut irrelevant=0usize;
    let mut no_learning_plan=0usize;
    let mut irrelevant_gain=0usize;
    let mut drive_violations=0usize;
    let mut endpoint_violations=0usize;
    let mut per_seed=Vec::new();
    let mut motor_mask=0u8;

    for (sub,spec) in specs.iter().enumerate(){
        for m in spec.motor_roles {motor_mask|=1u8<<m;}
        let base=fresh_base(&drive,spec);
        let sr=fresh_state_roles(spec);
        let mr=fresh_motor_roles(spec);
        let mut sub_full=0usize;

        for goal_a in [true,false] {
            for layout in spec.layouts[2..6].iter().copied(){
                let mut evo=base.clone();
                let e=fresh_full_episode(&mut evo,spec,goal_a,layout);
                full+=usize::from(e.acquired);
                sub_full+=usize::from(e.acquired);
                first+=usize::from(e.first_ok);
                second+=usize::from(e.second_ok);
                post_plan+=usize::from(e.post_plan);
                irrelevant_gain+=e.irrelevant_gain;
                drive_violations+=e.drive_mutation;
                endpoint_violations+=e.endpoint_violation;

                let mut gen=base.clone();
                general+=usize::from(fresh_general_episode(
                    &mut gen,spec,goal_a,layout
                ));

                let mut wrong=base.clone();
                wrong_goal+=usize::from(
                    fresh_full_episode(&mut wrong,spec,!goal_a,layout).acquired
                );

                // NO_TRANSITION_LEARNING: selector may execute, but shortcut
                // cannot become part of the physical abstract model.
                let mut nl=base.clone();
                nl.set_planning_learning_enabled(false);
                let goal=if goal_a{sr.goal_a}else{sr.goal_b};
                let hub=if goal_a{sr.hub_a}else{sr.hub_b};
                let shortcut=if goal_a{mr.shortcut_a}else{mr.shortcut_b};
                let goal_scene=fresh_state_scene(spec,goal,layout);
                let mut state=sr.start;
                nl.observe_initial_real(&fresh_state_scene(spec,state,layout),false);
                if let Some(a)=nl.choose_phase_native_goal_active_action(&goal_scene){
                    state=fresh_world_step(state,a,spec);
                    let _=nl.observe_phase_native_abstract_action_result(
                        a,&fresh_state_scene(spec,state,layout),0.0,false
                    );
                    if let Some(b)=nl.choose_phase_native_goal_active_action(&goal_scene){
                        state=fresh_world_step(state,b,spec);
                        let _=nl.observe_phase_native_abstract_action_result(
                            b,&fresh_state_scene(spec,state,layout),0.0,false
                        );
                    }
                }
                no_learning_plan+=usize::from(
                    nl.plan_phase_native_abstract_goal(
                        &fresh_state_scene(spec,hub,layout),
                        &goal_scene,
                        None,
                    ).map(|d|d.first_action)==Some(shortcut)
                );
            }

            // Fresh causal interventions on first held-out layout per goal.
            let layout=spec.layouts[2];
            let goal=if goal_a{sr.goal_a}else{sr.goal_b};

            let mut bg=base.clone();
            let node=fresh_l2_node(&bg,spec,goal);
            bg.perturb_phase_native_synapse_for_control(
                node.child_synapses[0],0.0,0.0
            ).unwrap();
            broken_goal+=usize::from(
                fresh_full_episode(&mut bg,spec,goal_a,layout).acquired
            );

            let (pre,action,post)=if goal_a{
                (sr.mid_a,mr.step_a,sr.goal_a)
            }else{
                (sr.mid_b,mr.step_b,sr.goal_b)
            };
            let circuit=fresh_transition_circuit(
                &base,spec,pre,action,post
            );

            let mut br=base.clone();
            br.perturb_phase_native_synapse_for_control(
                circuit.successor_synapse,0.0,0.0
            ).unwrap();
            broken_path+=usize::from(
                fresh_full_episode(&mut br,spec,goal_a,layout).acquired
            );

            let mut ps=base.clone();
            ps.perturb_phase_native_synapse_for_control(
                circuit.successor_synapse,1.0,std::f32::consts::PI
            ).unwrap();
            phase_path+=usize::from(
                fresh_full_episode(&mut ps,spec,goal_a,layout).acquired
            );

            let mut rr=base.clone();
            let saved=rr.perturb_phase_native_synapse_for_control(
                circuit.successor_synapse,0.0,0.0
            ).unwrap();
            rr.restore_phase_native_synapse_for_control(
                circuit.successor_synapse,saved
            );
            restored+=usize::from(
                fresh_full_episode(&mut rr,spec,goal_a,layout).acquired
            );
        }

        // One irrelevant competing-branch lesion per seed.
        let ic=fresh_transition_circuit(
            &base,spec,sr.mid_b,mr.step_b,sr.goal_b
        );
        let mut irr=base.clone();
        irr.perturb_phase_native_synapse_for_control(
            ic.successor_synapse,0.0,0.0
        ).unwrap();
        irrelevant+=usize::from(
            fresh_full_episode(
                &mut irr,spec,true,spec.layouts[2]
            ).acquired
        );

        per_seed.push(sub_full);
        println!(
            "FRESH_G18_SUB sub={} full={}/8 general_so_far={} roles={:?} motors={:?}",
            sub,sub_full,general,sr_as_array(sr),spec.motor_roles
        );
    }

    let (lo,hi)=fresh18_wilson95(full,80);
    println!(
        "FRESH_G18_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} first={}/80 second={}/80 post_plan={}/80 general={}/80 no_goal={}/80 wrong_goal={}/80 broken_goal={}/20 broken_path={}/20 phase_path={}/20 restored={}/20 irrelevant={}/10 no_learning_plan={}/80 irrelevant_gain={} drive_violations={} endpoint_violations={} motor_mask={:#08b}",
        full,lo,hi,per_seed,first,second,post_plan,general,no_goal,
        wrong_goal,broken_goal,broken_path,phase_path,restored,irrelevant,
        no_learning_plan,irrelevant_gain,drive_violations,
        endpoint_violations,motor_mask
    );

    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|x|*x>=6));
    assert!(first>=76);
    assert!(second>=76);
    assert!(post_plan>=76);
    assert!(general<=48);
    assert!(no_goal<=8);
    assert!(wrong_goal>=72);
    assert!(broken_goal<=4);
    assert!(broken_path<=4);
    assert!(phase_path<=4);
    assert!(restored>=19);
    assert!(irrelevant>=9);
    assert_eq!(no_learning_plan,0);
    assert_eq!(irrelevant_gain,0);
    assert_eq!(drive_violations,0);
    assert_eq!(endpoint_violations,0);
    assert_eq!(motor_mask,0b11_1111);
}

fn sr_as_array(r:FreshStateRoles)->[usize;7]{
    [r.start,r.hub_a,r.hub_b,r.mid_a,r.mid_b,r.goal_a,r.goal_b]
}
