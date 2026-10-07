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
