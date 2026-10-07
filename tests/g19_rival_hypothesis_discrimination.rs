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

const STATE_PAIRS:[[usize;2];9]=[
    [0,1], // S0
    [2,3], // A_GOOD
    [4,5], // A_DEAD
    [6,7], // B_GOOD
    [0,2], // B_DEAD
    [1,3], // FALL_A
    [4,6], // FALL_B
    [5,7], // GOAL_A
    [1,6], // GOAL_B
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
    l1:&[[usize;2];8],
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    let mut r=blank();
    let mut cursor=0usize;
    for child in STATE_PAIRS[state] {
        for atom in l1[child] {
            add_motif(&mut r,atom,layout[cursor]);
            cursor+=1;
        }
    }
    r
}

fn config()->EvoConfig{
    EvoConfig{
        sensory_cells:W*H,
        motor_cells:6,
        dormant_cells:640,
        hdc_dim:192,
        weight_learning_rate:1.0,
        phase_learning_rate:1.0,
        min_recruit_support:1,
        ..EvoConfig::default()
    }
}

fn native_carrier()->EvoPhase{
    let mut evo=EvoPhase::new(config());
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
        let mut state=0usize;
        evo.observe_initial_real(&source_raster(state),false);
        let mut solved=false;
        for _ in 0..SOURCE_BUDGET {
            let Some(action)=evo.choose_phase_native_autonomous_action() else{break;};
            let (next,value)=world.step(state,action);
            assert!(evo.observe_phase_native_action_result(
                action,&source_raster(next),value
            ).is_some());
            state=next;
            if value>=1.0 {solved=true;break;}
        }
        assert!(solved);
        checkpoint=evo.phase_native_drive_checkpoint();
    }
    checkpoint.unwrap()
}

fn target(drive:&PhaseDriveCheckpoint)->EvoPhase{
    let mut evo=native_carrier();
    assert!(evo.restore_phase_native_drive_checkpoint(drive.clone()));
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
    let l1=factors[0];

    for pair in l1 {
        for layout in LAYOUTS[..4].iter().copied(){
            let s=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &s,action,action==0
                ));
            }
        }
    }
    for (index,round) in factors[1..5].iter().enumerate(){
        for pair in *round {
            let s=pair_scene(pair,LAYOUTS[index]);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &s,action,action==1
                ));
            }
        }
    }
    assert_eq!(evo.concept_atoms().len(),16);
    assert_eq!(evo.phase_native_promoted_concept_count(),8);

    assert!(evo.enable_phase_native_depth_generic_abstraction(2));

    for child in 0..8usize {
        let s=pair_scene(l1[child],LAYOUTS[child%4]);
        for rep in 0..4usize {
            let first=rep%2==0;
            assert!(evo.observe_phase_native_depth_generic_factual(
                &s,2,first
            ));
            assert!(evo.observe_phase_native_depth_generic_factual(
                &s,3,!first
            ));
        }
    }

    for cycle in 0..4usize {
        for state in 0..9usize {
            let s=state_scene(&l1,state,LAYOUTS[cycle]);
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &s,action,action==2
                ));
            }
            for child in STATE_PAIRS[state] {
                let single=pair_scene(
                    l1[child],LAYOUTS[(cycle+child+2)%6]
                );
                for action in [2usize,3usize] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==3
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_deep_candidate_count(2),9);
    assert_eq!(evo.phase_native_promoted_deep_count(2),9);
    evo.set_concept_learning_enabled(false);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..9usize {
        let reference=evo.phase_native_abstract_state(
            &state_scene(&l1,state,LAYOUTS[0])
        ).unwrap();
        assert_eq!(reference.level,2);
        assert!(cells.insert(reference.cell));
        for layout in LAYOUTS {
            assert_eq!(
                evo.phase_native_abstract_state(
                    &state_scene(&l1,state,layout)
                ).unwrap(),
                reference
            );
        }
    }
    assert_eq!(cells.len(),9);
    l1
}

#[derive(Clone,Copy)]
struct Roles{
    probe_a:usize,
    probe_b:usize,
    fallback_a:usize,
    fallback_b:usize,
    step_a:usize,
    step_b:usize,
}

fn roles(swap:bool)->Roles{
    if swap{
        Roles{probe_a:1,probe_b:0,fallback_a:3,fallback_b:2,step_a:5,step_b:4}
    }else{
        Roles{probe_a:0,probe_b:1,fallback_a:2,fallback_b:3,step_a:4,step_b:5}
    }
}

fn state_ref(
    evo:&EvoPhase,
    l1:&[[usize;2];8],
    state:usize,
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(
        &state_scene(l1,state,LAYOUTS[0])
    ).unwrap()
}

fn transition_circuit(
    evo:&EvoPhase,
    from:PhaseAbstractStateRef,
    action:usize,
    to:PhaseAbstractStateRef,
)->PhaseCircuitInfo{
    let motor=evo.config().sensory_cells+action;
    evo.phase_native_circuits().iter().find(|c|{
        let a=evo.phase_native_synapse(c.afferent_synapse).unwrap();
        let z=evo.phase_native_synapse(c.successor_synapse).unwrap();
        let m=evo.phase_native_synapse(c.motor_synapse).unwrap();
        a.from==from.cell && z.to==to.cell && m.to==motor
    }).unwrap().clone()
}

fn l2_node(evo:&EvoPhase,s:PhaseAbstractStateRef)->PhaseDeepNodeInfo{
    evo.phase_native_deep_nodes().iter().find(|n|
        n.promoted && n.level==2 && n.id==s.id && n.concept_cell==s.cell
    ).unwrap().clone()
}

fn learn(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    pre:usize,
    action:usize,
    post:usize,
){
    assert!(evo.observe_phase_native_abstract_transition(
        &state_scene(l1,pre,LAYOUTS[0]),
        action,
        &state_scene(l1,post,LAYOUTS[0]),
        0.0,
    ));
}

fn learn_full_model(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    r:Roles,
){
    // Rival factual predictions from prior experience.
    learn(evo,l1,0,r.probe_a,1);
    learn(evo,l1,0,r.probe_a,2);
    learn(evo,l1,0,r.probe_b,3);
    learn(evo,l1,0,r.probe_b,4);

    // Longer known fallbacks.
    learn(evo,l1,0,r.fallback_a,5);
    learn(evo,l1,5,r.step_a,1);
    learn(evo,l1,1,r.step_a,7);

    learn(evo,l1,0,r.fallback_b,6);
    learn(evo,l1,6,r.step_b,3);
    learn(evo,l1,3,r.step_b,8);

    // Remaining start motors are known neutral/self transitions.
    learn(evo,l1,0,r.step_a,0);
    learn(evo,l1,0,r.step_b,0);

    // Fully model successor/dead/goal states so G18 novelty has no hidden
    // frontier signal to exploit.
    for state in 1..9usize {
        for action in 0..6usize {
            let already=evo.phase_native_circuits().iter().any(|c|{
                let a=evo.phase_native_synapse(c.afferent_synapse).unwrap();
                let m=evo.phase_native_synapse(c.motor_synapse).unwrap();
                a.from==state_ref(evo,l1,state).cell
                    && m.to==evo.config().sensory_cells+action
            });
            if !already {
                learn(evo,l1,state,action,state);
            }
        }
    }

    assert_eq!(evo.planning_transition_count(),0);
}

fn base(drive:&PhaseDriveCheckpoint,swap:bool)->(EvoPhase,[[usize;2];8],Roles){
    let mut evo=target(drive);
    let l1=train_abstraction(&mut evo);
    let r=roles(swap);
    learn_full_model(&mut evo,&l1,r);
    (evo,l1,r)
}

fn goal_state(goal_a:bool)->usize{if goal_a{7}else{8}}
fn good_state(goal_a:bool)->usize{if goal_a{1}else{3}}
fn dead_state(goal_a:bool)->usize{if goal_a{2}else{4}}
fn probe(r:Roles,goal_a:bool)->usize{if goal_a{r.probe_a}else{r.probe_b}}
fn fallback(r:Roles,goal_a:bool)->usize{if goal_a{r.fallback_a}else{r.fallback_b}}

fn post_plan_action(
    evo:&mut EvoPhase,
    l1:&[[usize;2];8],
    goal_a:bool,
    layout:[(usize,usize);4],
)->Option<usize>{
    evo.observe_initial_real(&state_scene(l1,0,layout),false);
    evo.plan_phase_native_abstract_goal(
        &state_scene(l1,0,layout),
        &state_scene(l1,goal_state(goal_a),layout),
        None,
    ).map(|d|d.first_action)
}

#[test]
fn g19_goal_relevant_rivals_drive_probe_then_factual_revision(){
    let drive=train_drive();

    let mut full_probe=0usize;
    let mut suppressed=0usize;
    let mut good_plan=0usize;
    let mut dead_plan=0usize;
    let mut wrong_goal=0usize;
    let mut novelty=0usize;
    let mut no_goal=0usize;
    let mut broken_goal=0usize;
    let mut broken_relevance=0usize;
    let mut phase_relevance=0usize;
    let mut no_revision_dead=0usize;
    let mut rival_collapse=0usize;
    let mut restored=0usize;
    let mut irrelevant=0usize;

    for swap in [false,true] {
        for goal_a in [true,false] {
            for actual_good in [true,false] {
                let (base,l1,r)=base(&drive,swap);
                let current=state_scene(&l1,0,LAYOUTS[4]);
                let goal=state_scene(&l1,goal_state(goal_a),LAYOUTS[5]);
                let expected_probe=probe(r,goal_a);

                let mut full=base.clone();
                full.observe_initial_real(&current,false);
                let before_fp=full.phase_native_learned_fingerprint();
                let chosen=full.choose_phase_native_goal_rival_probe(&goal);
                let after_fp=full.phase_native_learned_fingerprint();
                assert_eq!(before_fp,after_fp);
                full_probe+=usize::from(chosen==Some(expected_probe));

                let factual=if actual_good{
                    good_state(goal_a)
                }else{
                    dead_state(goal_a)
                };
                let suppressed_now=full.observe_phase_native_rival_probe_result(
                    expected_probe,
                    &state_scene(&l1,factual,LAYOUTS[4]),
                ).unwrap();
                suppressed+=usize::from(suppressed_now==1);

                let score_after=full.phase_native_goal_rival_disagreement_for_control(
                    &state_scene(&l1,0,LAYOUTS[4]),
                    &goal,
                    expected_probe,
                ).unwrap();
                assert!(score_after<=1.0e-8);

                let planned=post_plan_action(
                    &mut full,&l1,goal_a,LAYOUTS[5]
                );
                if actual_good {
                    good_plan+=usize::from(planned==Some(expected_probe));
                }else{
                    dead_plan+=usize::from(planned==Some(fallback(r,goal_a)));
                }

                // Opposite valid goal selects opposite probe.
                let mut wrong=base.clone();
                wrong.observe_initial_real(&current,false);
                let wrong_goal_scene=state_scene(
                    &l1,goal_state(!goal_a),LAYOUTS[5]
                );
                wrong_goal+=usize::from(
                    wrong.choose_phase_native_goal_rival_probe(&wrong_goal_scene)
                        ==Some(probe(r,!goal_a))
                );

                // Novelty-only has no hypothesis signal because all actions
                // are already physically modeled.
                let novelty_choice=base.clone();
                let mut n=novelty_choice;
                n.observe_initial_real(&current,false);
                novelty+=usize::from(
                    n.choose_phase_native_goal_unknown_probe_only_for_control(&goal)
                        ==Some(expected_probe)
                );

                let mut ng=base.clone();
                ng.observe_initial_real(&current,false);
                no_goal+=usize::from(
                    ng.choose_phase_native_rival_probe_no_goal_for_control()
                        ==Some(expected_probe)
                );

                // Goal-recognition lesion.
                let goal_ref=state_ref(&base,&l1,goal_state(goal_a));
                let node=l2_node(&base,goal_ref);
                let mut bg=base.clone();
                bg.perturb_phase_native_synapse_for_control(
                    node.child_synapses[0],0.0,0.0
                ).unwrap();
                bg.observe_initial_real(&current,false);
                broken_goal+=usize::from(
                    bg.choose_phase_native_goal_rival_probe(&goal)
                        ==Some(expected_probe)
                );

                // Break GOOD->GOAL relevance path.
                let step=if goal_a{r.step_a}else{r.step_b};
                let route=transition_circuit(
                    &base,
                    state_ref(&base,&l1,good_state(goal_a)),
                    step,
                    goal_ref,
                );
                let mut br=base.clone();
                let saved=br.perturb_phase_native_synapse_for_control(
                    route.successor_synapse,0.0,0.0
                ).unwrap();
                br.observe_initial_real(&current,false);
                broken_relevance+=usize::from(
                    br.choose_phase_native_goal_rival_probe(&goal)
                        ==Some(expected_probe)
                );

                br.restore_phase_native_synapse_for_control(
                    route.successor_synapse,saved.clone()
                );
                br.observe_initial_real(&current,false);
                restored+=usize::from(
                    br.choose_phase_native_goal_rival_probe(&goal)
                        ==Some(expected_probe)
                );

                let mut ps=base.clone();
                ps.perturb_phase_native_synapse_for_control(
                    route.successor_synapse,1.0,std::f32::consts::PI
                ).unwrap();
                ps.observe_initial_real(&current,false);
                phase_relevance+=usize::from(
                    ps.choose_phase_native_goal_rival_probe(&goal)
                        ==Some(expected_probe)
                );

                // DEAD fact without rival revision: optimistic GOOD prediction
                // remains and should keep shortcut falsely attractive.
                if !actual_good {
                    let mut nr=base.clone();
                    nr.observe_initial_real(&current,false);
                    let dead=state_scene(&l1,dead_state(goal_a),LAYOUTS[4]);
                    assert!(nr.observe_phase_native_abstract_action_result(
                        expected_probe,&dead,0.0,false
                    ).is_some());
                    let planned=post_plan_action(
                        &mut nr,&l1,goal_a,LAYOUTS[5]
                    );
                    no_revision_dead+=usize::from(
                        planned==Some(fallback(r,goal_a))
                    );
                }

                // Remove one of the requested action's rival predictions:
                // discrimination signal must collapse.
                let competitor=transition_circuit(
                    &base,
                    state_ref(&base,&l1,0),
                    expected_probe,
                    state_ref(&base,&l1,dead_state(goal_a)),
                );
                let mut bc=base.clone();
                let old=bc.perturb_phase_native_synapse_for_control(
                    competitor.successor_synapse,0.0,0.0
                ).unwrap();
                let collapsed=bc.phase_native_goal_rival_disagreement_for_control(
                    &current,&goal,expected_probe
                ).unwrap();
                rival_collapse+=usize::from(collapsed<=1.0e-8);
                bc.restore_phase_native_synapse_for_control(
                    competitor.successor_synapse,old
                );

                // Competing goal's rival lesion is irrelevant.
                let other_probe=probe(r,!goal_a);
                let other_dead=dead_state(!goal_a);
                let other=transition_circuit(
                    &base,
                    state_ref(&base,&l1,0),
                    other_probe,
                    state_ref(&base,&l1,other_dead),
                );
                let mut irr=base.clone();
                irr.perturb_phase_native_synapse_for_control(
                    other.successor_synapse,0.0,0.0
                ).unwrap();
                irr.observe_initial_real(&current,false);
                irrelevant+=usize::from(
                    irr.choose_phase_native_goal_rival_probe(&goal)
                        ==Some(expected_probe)
                );
            }
        }
    }

    println!(
        "G19_RIVAL full_probe={}/8 suppressed={}/8 good_plan={}/4 dead_plan={}/4 wrong_goal={}/8 novelty={}/8 no_goal={}/8 broken_goal={}/8 broken_relevance={}/8 phase_relevance={}/8 no_revision_dead={}/4 rival_collapse={}/8 restored={}/8 irrelevant={}/8",
        full_probe,suppressed,good_plan,dead_plan,wrong_goal,novelty,no_goal,
        broken_goal,broken_relevance,phase_relevance,no_revision_dead,
        rival_collapse,restored,irrelevant
    );

    assert_eq!(full_probe,8);
    assert_eq!(suppressed,8);
    assert_eq!(good_plan,4);
    assert_eq!(dead_plan,4);
    assert_eq!(wrong_goal,8);
    assert!(novelty<=2);
    assert!(no_goal<=4);
    assert!(broken_goal<=2);
    assert!(broken_relevance<=4);
    assert!(phase_relevance<=4);
    assert!(no_revision_dead<=1);
    assert_eq!(rival_collapse,8);
    assert_eq!(restored,8);
    assert_eq!(irrelevant,8);
}

#[test]
fn g19_selector_has_no_host_hypothesis_table_or_search_fallback(){
    let source=include_str!("../src/phase_abstract_planning.rs");
    let start=source.find(
        "pub fn choose_phase_native_goal_rival_probe"
    ).unwrap();
    let end=source[start..].find(
        "/// Matched G16 diagnostic"
    ).map(|x|start+x).unwrap_or(source.len());
    let block=&source[start..end];

    for forbidden in [
        "EvoEpistemicState",
        "WorldHypothesis",
        "HypothesisPrediction",
        "STATE_PAIRS",
        "good_state",
        "dead_state",
        "correct_probe",
        "EvoImaginationPlanner",
        "BinaryHeap",
        "VecDeque",
        "Dijkstra",
        "BFS",
        "DFS",
    ] {
        assert!(
            !block.contains(forbidden),
            "G19 production block contains forbidden token {forbidden}"
        );
    }

    for required in [
        "phase_native_abstract_state",
        "phase_goal_relevance_for_cells",
        "phase_supported_rival_successors",
        "phase_goal_rival_disagreement_score",
        "self.synapses",
        "circuit.revision",
        "successor.weight = 0.0",
    ] {
        assert!(
            block.contains(required),
            "G19 production block missing physical dependency {required}"
        );
    }
}


#[derive(Clone,Debug)]
struct Fresh19Spec {
    atom_perm:[usize;16],
    l1_pairs:[[usize;2];8],
    state_pairs:[[usize;2];9],
    state_roles:[usize;9],
    motor_roles:[usize;6],
    layouts:[[(usize,usize);4];6],
    tuition_order:Vec<usize>,
}

struct Fresh19Rng(u64);

impl Fresh19Rng{
    fn new(seed:u64)->Self{Self(seed^0xA319_19A1_C011_2026)}
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

fn fresh19_shuffle<T>(rng:&mut Fresh19Rng,values:&mut [T]){
    for i in (1..values.len()).rev(){
        let j=rng.range(i+1);
        values.swap(i,j);
    }
}

fn fresh19_fnv(mut h:u64,value:u64)->u64{
    const PRIME:u64=1_099_511_628_211;
    for b in value.to_le_bytes(){
        h^=b as u64;
        h=h.wrapping_mul(PRIME);
    }
    h
}

#[derive(Clone,Copy,Debug)]
struct Fresh19StateRoles{
    start:usize,
    a_good:usize,
    a_dead:usize,
    b_good:usize,
    b_dead:usize,
    fall_a:usize,
    fall_b:usize,
    goal_a:usize,
    goal_b:usize,
}

fn fresh19_state_roles(spec:&Fresh19Spec)->Fresh19StateRoles{
    Fresh19StateRoles{
        start:spec.state_roles[0],
        a_good:spec.state_roles[1],
        a_dead:spec.state_roles[2],
        b_good:spec.state_roles[3],
        b_dead:spec.state_roles[4],
        fall_a:spec.state_roles[5],
        fall_b:spec.state_roles[6],
        goal_a:spec.state_roles[7],
        goal_b:spec.state_roles[8],
    }
}

fn fresh19_motor_roles(spec:&Fresh19Spec)->Roles{
    Roles{
        probe_a:spec.motor_roles[0],
        probe_b:spec.motor_roles[1],
        fallback_a:spec.motor_roles[2],
        fallback_b:spec.motor_roles[3],
        step_a:spec.motor_roles[4],
        step_b:spec.motor_roles[5],
    }
}

#[derive(Clone,Copy,Debug)]
struct Fresh19Fact{pre:usize,action:usize,post:usize}

fn fresh19_model_facts(spec:&Fresh19Spec)->Vec<Fresh19Fact>{
    let s=fresh19_state_roles(spec);
    let r=fresh19_motor_roles(spec);
    let mut facts=vec![
        Fresh19Fact{pre:s.start,action:r.probe_a,post:s.a_good},
        Fresh19Fact{pre:s.start,action:r.probe_a,post:s.a_dead},
        Fresh19Fact{pre:s.start,action:r.probe_b,post:s.b_good},
        Fresh19Fact{pre:s.start,action:r.probe_b,post:s.b_dead},
        Fresh19Fact{pre:s.a_good,action:r.step_a,post:s.goal_a},
        Fresh19Fact{pre:s.b_good,action:r.step_b,post:s.goal_b},
        Fresh19Fact{pre:s.start,action:r.fallback_a,post:s.fall_a},
        Fresh19Fact{pre:s.fall_a,action:r.step_a,post:s.a_good},
        Fresh19Fact{pre:s.start,action:r.fallback_b,post:s.fall_b},
        Fresh19Fact{pre:s.fall_b,action:r.step_b,post:s.b_good},
    ];

    // Fill every still-unrepresented state/action slot with a neutral self-loop.
    for state in 0..9usize {
        for action in 0..6usize {
            if facts.iter().any(|f|f.pre==state && f.action==action){
                continue;
            }
            facts.push(Fresh19Fact{pre:state,action,post:state});
        }
    }

    // 54 state/action slots plus one extra rival successor for each probe.
    assert_eq!(facts.len(),56);
    facts
}

fn fresh19_specs(authority:u64)->(Vec<Fresh19Spec>,u64){
    let mut specs=Vec::new();
    let mut digest=14_695_981_039_346_656_037u64;

    for sub in 0..10u64 {
        let mut rng=Fresh19Rng::new(
            authority
                ^ sub.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ 0x19A0_C71E_5EED_0019
        );

        let mut atoms=(0usize..16).collect::<Vec<_>>();
        fresh19_shuffle(&mut rng,&mut atoms);
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
        fresh19_shuffle(&mut rng,&mut all_pairs);
        let mut state_pairs=[[0usize;2];9];
        state_pairs.copy_from_slice(&all_pairs[..9]);

        let mut sr=[0usize,1,2,3,4,5,6,7,8];
        fresh19_shuffle(&mut rng,&mut sr);

        let mut mr=[0usize,1,2,3,4,5];
        fresh19_shuffle(&mut rng,&mut mr);

        let mut layouts=LAYOUTS;
        fresh19_shuffle(&mut rng,&mut layouts);

        let mut provisional=Fresh19Spec{
            atom_perm,
            l1_pairs,
            state_pairs,
            state_roles:sr,
            motor_roles:mr,
            layouts,
            tuition_order:Vec::new(),
        };
        let fact_len=fresh19_model_facts(&provisional).len();
        let mut order=(0usize..fact_len).collect::<Vec<_>>();
        fresh19_shuffle(&mut rng,&mut order);
        provisional.tuition_order=order;

        digest=fresh19_fnv(digest,sub);
        for x in provisional.atom_perm {digest=fresh19_fnv(digest,x as u64);}
        for p in provisional.l1_pairs {for x in p {digest=fresh19_fnv(digest,x as u64);}}
        for p in provisional.state_pairs {for x in p {digest=fresh19_fnv(digest,x as u64);}}
        for x in provisional.state_roles {digest=fresh19_fnv(digest,x as u64);}
        for x in provisional.motor_roles {digest=fresh19_fnv(digest,x as u64);}
        for layout in provisional.layouts {
            for (x,y) in layout {
                digest=fresh19_fnv(digest,x as u64);
                digest=fresh19_fnv(digest,y as u64);
            }
        }
        for &x in &provisional.tuition_order {
            digest=fresh19_fnv(digest,x as u64);
        }
        specs.push(provisional);
    }
    (specs,digest)
}

fn fresh19_state_scene(
    spec:&Fresh19Spec,
    state:usize,
    layout:[(usize,usize);4],
)->Vec<f32>{
    let mut r=blank();
    let mut cursor=0usize;
    for child in spec.state_pairs[state] {
        for atom in spec.l1_pairs[child] {
            add_motif(&mut r,atom,layout[cursor]);
            cursor+=1;
        }
    }
    r
}

fn fresh19_train_abstraction(evo:&mut EvoPhase,spec:&Fresh19Spec){
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
        for raw in *round {
            let mut pair=[
                spec.atom_perm[raw[0]],
                spec.atom_perm[raw[1]],
            ];
            pair.sort_unstable();
            let scene=pair_scene(pair,spec.layouts[index]);
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

    for child in 0..8usize {
        let scene=pair_scene(
            spec.l1_pairs[child],
            spec.layouts[child%4],
        );
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
        for state in 0..9usize {
            let scene=fresh19_state_scene(spec,state,spec.layouts[cycle]);
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==2
                ));
            }
            for child in spec.state_pairs[state] {
                let single=pair_scene(
                    spec.l1_pairs[child],
                    spec.layouts[(cycle+child+2)%6],
                );
                for action in [2usize,3usize] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==3
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_deep_candidate_count(2),9);
    assert_eq!(evo.phase_native_promoted_deep_count(2),9);
    evo.set_concept_learning_enabled(false);

    let mut cells=std::collections::BTreeSet::new();
    for state in 0..9usize {
        let reference=evo.phase_native_abstract_state(
            &fresh19_state_scene(spec,state,spec.layouts[0])
        ).unwrap();
        assert_eq!(reference.level,2);
        assert!(cells.insert(reference.cell));
        for layout in spec.layouts {
            assert_eq!(
                evo.phase_native_abstract_state(
                    &fresh19_state_scene(spec,state,layout)
                ).unwrap(),
                reference
            );
        }
    }
    assert_eq!(cells.len(),9);
}

fn fresh19_base(
    drive:&PhaseDriveCheckpoint,
    spec:&Fresh19Spec,
)->EvoPhase{
    let mut evo=target(drive);
    fresh19_train_abstraction(&mut evo,spec);
    let facts=fresh19_model_facts(spec);
    for &index in &spec.tuition_order {
        let fact=facts[index];
        assert!(evo.observe_phase_native_abstract_transition(
            &fresh19_state_scene(spec,fact.pre,spec.layouts[0]),
            fact.action,
            &fresh19_state_scene(spec,fact.post,spec.layouts[0]),
            0.0,
        ));
    }
    assert_eq!(evo.phase_native_circuits().len(),56);
    assert_eq!(evo.planning_transition_count(),0);
    evo
}

fn fresh19_state_ref(
    evo:&EvoPhase,
    spec:&Fresh19Spec,
    state:usize,
)->PhaseAbstractStateRef{
    evo.phase_native_abstract_state(
        &fresh19_state_scene(spec,state,spec.layouts[0])
    ).unwrap()
}

fn fresh19_circuit(
    evo:&EvoPhase,
    spec:&Fresh19Spec,
    pre:usize,
    action:usize,
    post:usize,
)->PhaseCircuitInfo{
    transition_circuit(
        evo,
        fresh19_state_ref(evo,spec,pre),
        action,
        fresh19_state_ref(evo,spec,post),
    )
}

fn fresh19_l2_node(
    evo:&EvoPhase,
    spec:&Fresh19Spec,
    state:usize,
)->PhaseDeepNodeInfo{
    l2_node(evo,fresh19_state_ref(evo,spec,state))
}

#[derive(Default,Clone,Copy)]
struct Fresh19Episode{
    probe_ok:bool,
    suppressed_ok:bool,
    plan_ok:bool,
    fp_violation:usize,
    endpoint_violation:usize,
}

fn fresh19_episode(
    evo:&mut EvoPhase,
    spec:&Fresh19Spec,
    goal_a:bool,
    actual_good:bool,
    layout:[(usize,usize);4],
)->Fresh19Episode{
    let s=fresh19_state_roles(spec);
    let r=fresh19_motor_roles(spec);
    let goal=if goal_a{s.goal_a}else{s.goal_b};
    let good=if goal_a{s.a_good}else{s.b_good};
    let dead=if goal_a{s.a_dead}else{s.b_dead};
    let probe=if goal_a{r.probe_a}else{r.probe_b};
    let fallback=if goal_a{r.fallback_a}else{r.fallback_b};

    let current=fresh19_state_scene(spec,s.start,layout);
    let goal_scene=fresh19_state_scene(spec,goal,layout);
    evo.observe_initial_real(&current,false);

    let fp_before=evo.phase_native_learned_fingerprint();
    let selected=evo.choose_phase_native_goal_rival_probe(&goal_scene);
    let fp_after=evo.phase_native_learned_fingerprint();
    let fp_violation=usize::from(fp_before!=fp_after);
    let probe_ok=selected==Some(probe);

    let actual=if actual_good{good}else{dead};
    let suppressed=evo.observe_phase_native_rival_probe_result(
        probe,
        &fresh19_state_scene(spec,actual,layout),
    ).unwrap_or(0);
    let suppressed_ok=suppressed==1;

    let abstract_cells=evo.phase_native_deep_nodes().iter()
        .filter(|n|n.promoted && n.level==2)
        .map(|n|n.concept_cell)
        .collect::<std::collections::BTreeSet<_>>();
    let endpoint_violation=evo.phase_native_circuits().iter()
        .filter(|c|{
            let a=evo.phase_native_synapse(c.afferent_synapse).unwrap();
            let z=evo.phase_native_synapse(c.successor_synapse).unwrap();
            !abstract_cells.contains(&a.from) || !abstract_cells.contains(&z.to)
        }).count();

    evo.observe_initial_real(&current,false);
    let fp_plan_before=evo.phase_native_learned_fingerprint();
    let plan=evo.plan_phase_native_abstract_goal(
        &current,&goal_scene,None
    ).map(|d|d.first_action);
    let fp_plan_after=evo.phase_native_learned_fingerprint();
    let expected=if actual_good{probe}else{fallback};
    let plan_ok=plan==Some(expected);

    Fresh19Episode{
        probe_ok,
        suppressed_ok,
        plan_ok,
        fp_violation:fp_violation+usize::from(fp_plan_before!=fp_plan_after),
        endpoint_violation,
    }
}

fn fresh19_wilson95(success:usize,n:usize)->(f64,f64){
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
fn g19_fresh_rival_hypothesis_discrimination_pack(){
    let authority:u64=std::env::var("AETERNA_FRESH_SEED")
        .expect("AETERNA_FRESH_SEED required")
        .parse().expect("fresh G19 seed must be u64");
    let source_sha=std::env::var("AETERNA_SOURCE_SHA")
        .unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA")
        .unwrap_or_else(|_|"unknown".into());

    let (specs,digest)=fresh19_specs(authority);
    println!(
        "FRESH_G19_SEAL source_sha={} spec_sha={} authority_seed={} pack_digest={:016x}",
        source_sha,spec_sha,authority,digest
    );
    for (sub,spec) in specs.iter().enumerate(){
        println!("FRESH_G19_BLOCK sub={} {:?}",sub,spec);
    }

    let drive=train_drive();
    let mut full=0usize;
    let mut suppressed=0usize;
    let mut good_plan=0usize;
    let mut dead_plan=0usize;
    let mut wrong_goal=0usize;
    let mut novelty=0usize;
    let mut no_goal=0usize;
    let mut broken_goal=0usize;
    let mut broken_relevance=0usize;
    let mut phase_relevance=0usize;
    let mut no_revision_dead=0usize;
    let mut rival_collapse=0usize;
    let mut restored=0usize;
    let mut irrelevant=0usize;
    let mut endpoint_violations=0usize;
    let mut fp_violations=0usize;
    let mut legacy_violations=0usize;
    let mut motor_mask=0u8;
    let mut per_seed=Vec::new();

    for (sub,spec) in specs.iter().enumerate(){
        for m in spec.motor_roles {motor_mask|=1u8<<m;}
        let base=fresh19_base(&drive,spec);
        let s=fresh19_state_roles(spec);
        let r=fresh19_motor_roles(spec);
        if base.planning_transition_count()!=0 {legacy_violations+=1;}
        let mut sub_full=0usize;

        for goal_a in [true,false] {
            for actual_good in [true,false] {
                for layout in spec.layouts[2..4].iter().copied(){
                    let mut evo=base.clone();
                    let e=fresh19_episode(
                        &mut evo,spec,goal_a,actual_good,layout
                    );
                    full+=usize::from(e.probe_ok);
                    sub_full+=usize::from(e.probe_ok);
                    suppressed+=usize::from(e.suppressed_ok);
                    if actual_good {
                        good_plan+=usize::from(e.plan_ok);
                    }else{
                        dead_plan+=usize::from(e.plan_ok);
                    }
                    endpoint_violations+=e.endpoint_violation;
                    fp_violations+=e.fp_violation;

                    let start=fresh19_state_scene(spec,s.start,layout);
                    let goal=if goal_a{s.goal_a}else{s.goal_b};
                    let goal_scene=fresh19_state_scene(spec,goal,layout);
                    let expected_probe=if goal_a{r.probe_a}else{r.probe_b};

                    let mut wrong=base.clone();
                    wrong.observe_initial_real(&start,false);
                    let other_goal=if goal_a{s.goal_b}else{s.goal_a};
                    wrong_goal+=usize::from(
                        wrong.choose_phase_native_goal_rival_probe(
                            &fresh19_state_scene(spec,other_goal,layout)
                        )==Some(if goal_a{r.probe_b}else{r.probe_a})
                    );

                    let mut nov=base.clone();
                    nov.observe_initial_real(&start,false);
                    novelty+=usize::from(
                        nov.choose_phase_native_goal_unknown_probe_only_for_control(
                            &goal_scene
                        )==Some(expected_probe)
                    );

                    let mut ng=base.clone();
                    ng.observe_initial_real(&start,false);
                    no_goal+=usize::from(
                        ng.choose_phase_native_rival_probe_no_goal_for_control()
                            ==Some(expected_probe)
                    );
                }
            }

            let layout=spec.layouts[2];
            let start=fresh19_state_scene(spec,s.start,layout);
            let goal=if goal_a{s.goal_a}else{s.goal_b};
            let goal_scene=fresh19_state_scene(spec,goal,layout);
            let good=if goal_a{s.a_good}else{s.b_good};
            let dead=if goal_a{s.a_dead}else{s.b_dead};
            let probe=if goal_a{r.probe_a}else{r.probe_b};
            let fallback=if goal_a{r.fallback_a}else{r.fallback_b};
            let step=if goal_a{r.step_a}else{r.step_b};

            let mut bg=base.clone();
            let node=fresh19_l2_node(&bg,spec,goal);
            bg.perturb_phase_native_synapse_for_control(
                node.child_synapses[0],0.0,0.0
            ).unwrap();
            bg.observe_initial_real(&start,false);
            broken_goal+=usize::from(
                bg.choose_phase_native_goal_rival_probe(&goal_scene)
                    ==Some(probe)
            );

            let route=fresh19_circuit(&base,spec,good,step,goal);
            let mut br=base.clone();
            let saved=br.perturb_phase_native_synapse_for_control(
                route.successor_synapse,0.0,0.0
            ).unwrap();
            br.observe_initial_real(&start,false);
            broken_relevance+=usize::from(
                br.choose_phase_native_goal_rival_probe(&goal_scene)
                    ==Some(probe)
            );

            br.restore_phase_native_synapse_for_control(
                route.successor_synapse,saved.clone()
            );
            br.observe_initial_real(&start,false);
            restored+=usize::from(
                br.choose_phase_native_goal_rival_probe(&goal_scene)
                    ==Some(probe)
            );

            let mut ps=base.clone();
            ps.perturb_phase_native_synapse_for_control(
                route.successor_synapse,1.0,std::f32::consts::PI
            ).unwrap();
            ps.observe_initial_real(&start,false);
            phase_relevance+=usize::from(
                ps.choose_phase_native_goal_rival_probe(&goal_scene)
                    ==Some(probe)
            );

            // DEAD factual result without contradiction suppression.
            let mut nr=base.clone();
            nr.observe_initial_real(&start,false);
            assert!(nr.observe_phase_native_abstract_action_result(
                probe,
                &fresh19_state_scene(spec,dead,layout),
                0.0,
                false,
            ).is_some());
            nr.observe_initial_real(&start,false);
            no_revision_dead+=usize::from(
                nr.plan_phase_native_abstract_goal(
                    &start,&goal_scene,None
                ).map(|d|d.first_action)==Some(fallback)
            );

            // Remove one rival prediction and require discrimination collapse.
            let rival=fresh19_circuit(&base,spec,s.start,probe,dead);
            let mut rc=base.clone();
            rc.perturb_phase_native_synapse_for_control(
                rival.successor_synapse,0.0,0.0
            ).unwrap();
            let score=rc.phase_native_goal_rival_disagreement_for_control(
                &start,&goal_scene,probe
            ).unwrap();
            rival_collapse+=usize::from(score<=1.0e-8);
        }

        // Irrelevant competing-goal rival lesion.
        let layout=spec.layouts[2];
        let start=fresh19_state_scene(spec,s.start,layout);
        let goal_scene=fresh19_state_scene(spec,s.goal_a,layout);
        let rival=fresh19_circuit(
            &base,spec,s.start,r.probe_b,s.b_dead
        );
        let mut irr=base.clone();
        irr.perturb_phase_native_synapse_for_control(
            rival.successor_synapse,0.0,0.0
        ).unwrap();
        irr.observe_initial_real(&start,false);
        irrelevant+=usize::from(
            irr.choose_phase_native_goal_rival_probe(&goal_scene)
                ==Some(r.probe_a)
        );

        per_seed.push(sub_full);
        println!(
            "FRESH_G19_SUB sub={} full={}/8 roles={:?} motors={:?}",
            sub,sub_full,spec.state_roles,spec.motor_roles
        );
    }

    let (lo,hi)=fresh19_wilson95(full,80);
    println!(
        "FRESH_G19_RESULT full={}/80 wilson95=[{:.6},{:.6}] per_seed={:?} suppressed={}/80 good_plan={}/40 dead_plan={}/40 wrong_goal={}/80 novelty={}/80 no_goal={}/80 broken_goal={}/20 broken_relevance={}/20 phase_relevance={}/20 no_revision_dead={}/20 rival_collapse={}/20 restored={}/20 irrelevant={}/10 endpoint_violations={} fp_violations={} legacy_violations={} motor_mask={:#08b}",
        full,lo,hi,per_seed,suppressed,good_plan,dead_plan,wrong_goal,
        novelty,no_goal,broken_goal,broken_relevance,phase_relevance,
        no_revision_dead,rival_collapse,restored,irrelevant,
        endpoint_violations,fp_violations,legacy_violations,motor_mask
    );

    assert!(full>=76);
    assert!(lo>=0.87);
    assert!(per_seed.iter().all(|x|*x>=6));
    assert!(suppressed>=76);
    assert!(good_plan>=38);
    assert!(dead_plan>=38);
    assert!(wrong_goal>=72);
    assert!(novelty<=20);
    assert!(no_goal<=48);
    assert!(broken_goal<=4);
    assert!(broken_relevance<=4);
    assert!(phase_relevance<=4);
    assert!(no_revision_dead<=4);
    assert!(rival_collapse>=19);
    assert!(restored>=19);
    assert!(irrelevant>=9);
    assert_eq!(endpoint_violations,0);
    assert_eq!(fp_violations,0);
    assert_eq!(legacy_violations,0);
    assert_eq!(motor_mask,0b11_1111);
}
