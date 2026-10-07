use aeterna_v1::{ConceptConfig, EvoConfig, EvoPhase, RasterFieldConfig};
use aeterna_v1::carrier::{
    PhaseDriveCheckpoint, PhaseDriveConfig, PhaseNativeCheckpoint, PhaseNativeConfig,
};

const W: usize = 20;
const H: usize = 20;
const SOURCE_BUDGET: usize = 160;
const TARGET_BUDGET: usize = 80;

const OFFSETS: [(usize,usize);16] = [
    (1,0),(0,1),(1,1),(2,0),
    (0,2),(2,1),(1,2),(2,2),
    (3,0),(0,3),(3,1),(1,3),
    (3,2),(2,3),(3,3),(4,0),
];

const LAYOUTS: [[(usize,usize);4];9] = [
    [(1,1),(11,1),(1,11),(11,11)],
    [(2,1),(12,1),(2,11),(12,11)],
    [(3,1),(13,1),(3,11),(13,11)],
    [(1,2),(11,2),(1,12),(11,12)],
    [(2,2),(12,2),(2,12),(12,12)],
    [(3,2),(13,2),(3,12),(13,12)],
    [(1,3),(11,3),(1,13),(11,13)],
    [(2,3),(12,3),(2,13),(12,13)],
    [(3,3),(13,3),(3,13),(13,13)],
];

const L1_PAIRS: [[usize;2];4] = [[0,7],[1,6],[2,5],[3,4]];
const STATE_PAIRS: [[usize;2];6] = [
    [0,1],[0,2],[0,3],[1,2],[1,3],[2,3],
];

#[derive(Debug, Clone)]
struct ResetWorld {
    length: usize,
    pair: [usize;2],
    immediate: usize,
    advance: Vec<usize>,
}

impl ResetWorld {
    fn step(&self, state:usize, action:usize)->(usize,f32){
        if state==0 && action==self.immediate {
            return (0,0.55);
        }
        if action==self.advance[state] {
            if state+1==self.length {
                return (0,1.0);
            }
            return (state+1,0.0);
        }
        (0,0.0)
    }

    fn delayed_first(&self)->usize{self.advance[0]}
}

fn worlds()->Vec<ResetWorld>{
    let pairs=[
        [0,1],[2,3],[4,5],[0,5],
        [1,4],[2,5],[0,3],[1,5],
        [2,4],[0,4],[1,3],[3,5],
    ];
    (0..12usize).map(|index|{
        let length=3+index%3;
        let pair=pairs[index];
        let immediate=pair[index%2];
        let delayed=if immediate==pair[0] {pair[1]} else {pair[0]};
        let mut advance=Vec::with_capacity(length);
        advance.push(delayed);
        for stage in 1..length {
            // Alternate the two relevant opaque motors; never encode route
            // knowledge in cognition.
            advance.push(pair[(index+stage+1)%2]);
        }
        ResetWorld{length,pair,immediate,advance}
    }).collect()
}

fn evo_config()->EvoConfig{
    EvoConfig {
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
    let mut r=vec![0.0;W*H];
    let (dx,dy)=OFFSETS[state];
    r[0]=1.0;
    r[dy*W+dx]=1.0;
    r
}

#[derive(Debug,Clone)]
struct SourceWorld{
    length:usize,
    advance:Vec<usize>,
}

impl SourceWorld{
    fn step(&self,state:usize,action:usize)->(usize,f32){
        let next=if action==self.advance[state] {
            state+1
        } else {
            0
        };
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
        let Some(action)=evo.choose_phase_native_autonomous_action() else {
            return false;
        };
        let (next,value)=world.step(state,action);
        assert!(evo.observe_phase_native_action_result(
            action,&source_raster(next),value
        ).is_some());
        state=next;
        if value>=1.0 {return true;}
    }
    false
}

fn train_drive()->PhaseDriveCheckpoint{
    let mut checkpoint=None;
    for (index,world) in source_worlds().iter().enumerate(){
        let mut evo=native_carrier();
        if let Some(previous)=checkpoint.take(){
            assert!(evo.restore_phase_native_drive_checkpoint(previous));
        } else {
            assert!(evo.enable_phase_native_learned_drive(PhaseDriveConfig{
                learning_rate:0.35,
                discount:0.90,
                learning_enabled:true,
                readout_enabled:true,
                phase_learning_enabled:true,
            }));
        }
        assert!(source_teacher(&mut evo,world),"source drive world {index}");
        checkpoint=evo.phase_native_drive_checkpoint();
    }
    let checkpoint=checkpoint.expect("learned drive checkpoint");
    let mut probe=native_carrier();
    assert!(probe.restore_phase_native_drive_checkpoint(checkpoint.clone()));
    let weights=probe.phase_native_drive_weights().unwrap();
    assert!(weights[0]>0.05);
    assert!(weights[1]>0.01);
    checkpoint
}

fn add_motif(r:&mut [f32],atom:usize,origin:(usize,usize)){
    let (x,y)=origin;
    let (dx,dy)=OFFSETS[atom];
    r[y*W+x]=1.0;
    r[(y+dy)*W+x+dx]=1.0;
}

fn pair_scene(pair:[usize;2],layout:[(usize,usize);4])->Vec<f32>{
    let mut r=vec![0.0;W*H];
    add_motif(&mut r,pair[0],layout[0]);
    add_motif(&mut r,pair[1],layout[1]);
    r
}

fn state_scene(state:usize,layout:[(usize,usize);4])->Vec<f32>{
    let pair=STATE_PAIRS[state];
    let mut r=vec![0.0;W*H];
    let mut cursor=0usize;
    for l1 in pair {
        for atom in L1_PAIRS[l1] {
            add_motif(&mut r,atom,layout[cursor]);
            cursor+=1;
        }
    }
    r
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

fn target_with_drive(checkpoint:&PhaseDriveCheckpoint)->EvoPhase{
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
    train_abstract_states(&mut evo);
    evo.set_concept_learning_enabled(false);

    assert_eq!(evo.phase_native_circuits().len(),0);
    assert_eq!(evo.planning_transition_count(),0);
    evo
}

fn target_zero_drive()->EvoPhase{
    let mut evo=native_carrier();
    assert!(evo.enable_phase_native_learned_drive(PhaseDriveConfig{
        learning_enabled:false,
        ..PhaseDriveConfig::default()
    }));
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
    train_abstract_states(&mut evo);
    evo.set_concept_learning_enabled(false);
    evo
}

fn train_abstract_states(evo:&mut EvoPhase){
    let factors=factorization_8();

    // L1 physical concepts.
    for pair in L1_PAIRS {
        for layout in LAYOUTS[..4].iter().copied(){
            let raster=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==0
                ));
            }
        }
    }

    for (round_index,round) in factors[1..5].iter().enumerate(){
        let layout=LAYOUTS[4+round_index];
        for raw_pair in *round {
            let pair=[raw_pair[0],raw_pair[1]];
            if L1_PAIRS.contains(&pair) {continue;}
            let raster=pair_scene(pair,layout);
            for action in [0usize,1usize] {
                assert!(evo.observe_phase_native_concept_factual(
                    &raster,action,action==1
                ));
            }
        }
    }

    assert_eq!(evo.concept_atoms().len(),8);
    assert_eq!(evo.phase_native_promoted_concept_count(),4);

    assert!(evo.enable_phase_native_depth_generic_abstraction(2));

    // Make every L1 supported-and-weak on abstraction motors.
    for l1 in 0..4usize {
        let scene=pair_scene(L1_PAIRS[l1],LAYOUTS[l1]);
        for rep in 0..4usize {
            let a_wins=rep%2==0;
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,2,a_wins
            ));
            assert!(evo.observe_phase_native_depth_generic_factual(
                &scene,3,!a_wins
            ));
        }
    }

    // Acquire all 6 unordered L1 pairs as physical L2 states.
    for cycle in 0..4usize {
        for state in 0..6usize {
            let scene=state_scene(state,LAYOUTS[cycle]);
            for action in [2usize,3usize] {
                assert!(evo.observe_phase_native_depth_generic_factual(
                    &scene,action,action==2
                ));
            }
            for child in STATE_PAIRS[state] {
                let single=pair_scene(
                    L1_PAIRS[child],
                    LAYOUTS[(cycle+child+4)%9]
                );
                for action in [2usize,3usize] {
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &single,action,action==3
                    ));
                }
            }
        }
    }

    assert_eq!(evo.phase_native_deep_candidate_count(2),6);
    assert_eq!(evo.phase_native_promoted_deep_count(2),6);

    for state in 0..6usize {
        let mut cell=None;
        for layout in LAYOUTS {
            let r=state_scene(state,layout);
            let s=evo.phase_native_abstract_state(&r)
                .expect("physical target abstract state");
            assert_eq!(s.level,2);
            if let Some(previous)=cell {assert_eq!(previous,s.cell);} else {cell=Some(s.cell);}
        }
    }
}

#[derive(Debug,Clone,Copy)]
struct Outcome{
    reward:bool,
    interactions:usize,
}

fn acquire(
    evo:&mut EvoPhase,
    world:&ResetWorld,
    direct_only:bool,
)->Outcome{
    let mut state=0usize;
    evo.observe_initial_real(&state_scene(state,LAYOUTS[4]),false);

    for interaction in 1..=TARGET_BUDGET {
        let action=if direct_only {
            evo.choose_phase_native_abstract_direct_action()
        } else {
            evo.choose_phase_native_abstract_learned_drive_action()
        };
        let Some(action)=action else {
            return Outcome{reward:false,interactions:interaction-1};
        };

        let (next,value)=world.step(state,action);
        let layout=LAYOUTS[4+(interaction%4)];
        let post=state_scene(next,layout);
        let accepted=evo.observe_phase_native_abstract_action_result(
            action,&post,value
        ).expect("factual abstract action result");
        if evo.config().structural_growth_enabled
            && evo.phase_native_circuits().len()>0
            && evo.phase_native_drive_weights()!=Some([0.0,0.0])
        {
            // FULL may update an existing circuit or recruit a new one.
            let _=accepted;
        }
        state=next;
        if value>=1.0 {
            return Outcome{reward:true,interactions:interaction};
        }
    }
    Outcome{reward:false,interactions:TARGET_BUDGET}
}

fn random_acquire(world:&ResetWorld,seed:u64)->Outcome{
    let mut x=seed^0xD1B5_4A32_D192_ED03;
    let mut state=0usize;
    for interaction in 1..=TARGET_BUDGET {
        x^=x>>12;
        x^=x<<25;
        x^=x>>27;
        let action=(x.wrapping_mul(0x2545_F491_4F6C_DD1D)%6) as usize;
        let (next,value)=world.step(state,action);
        state=next;
        if value>=1.0 {
            return Outcome{reward:true,interactions:interaction};
        }
    }
    Outcome{reward:false,interactions:TARGET_BUDGET}
}

fn heldout_delayed_plan(evo:&mut EvoPhase,world:&ResetWorld)->bool{
    let scene=state_scene(0,LAYOUTS[8]);
    evo.observe_initial_real(&scene,false);
    evo.plan_phase_native_abstract(&scene,None)
        .map(|decision|decision.first_action==world.delayed_first())
        .unwrap_or(false)
}

fn endpoint_violations(evo:&EvoPhase)->usize{
    let abstract_cells=evo.phase_native_deep_nodes().iter()
        .filter(|node|node.promoted && node.level==2)
        .map(|node|node.concept_cell)
        .collect::<std::collections::BTreeSet<_>>();
    evo.phase_native_circuits().iter().filter(|circuit|{
        let aff=evo.phase_native_synapse(circuit.afferent_synapse).unwrap();
        let succ=evo.phase_native_synapse(circuit.successor_synapse).unwrap();
        !abstract_cells.contains(&aff.from) || !abstract_cells.contains(&succ.to)
    }).count()
}

fn restore(checkpoint:PhaseNativeCheckpoint)->EvoPhase{
    let mut evo=EvoPhase::new(evo_config());
    assert!(evo.restore_phase_native_checkpoint(checkpoint));
    evo
}

#[test]
fn g16_learned_drive_autonomously_acquires_abstract_world_model(){
    let checkpoint=train_drive();

    let mut probe=native_carrier();
    assert!(probe.restore_phase_native_drive_checkpoint(checkpoint.clone()));
    let learned_weights=probe.phase_native_drive_weights().unwrap();
    assert!(learned_weights[0]>0.05);
    assert!(learned_weights[1]>0.01);

    let mut full_success=0usize;
    let mut full_plan=0usize;
    let mut restart_plan=0usize;
    let mut zero_success=0usize;
    let mut lesion_success=0usize;
    let mut direct_success=0usize;
    let mut random_success=0usize;
    let mut no_learning_plan=0usize;
    let mut no_growth_plan=0usize;
    let mut endpoint_errors=0usize;
    let mut total_cost=0usize;

    for (index,world) in worlds().iter().enumerate(){
        let mut full=target_with_drive(&checkpoint);
        assert_eq!(full.phase_native_circuits().len(),0);
        assert_eq!(full.planning_transition_count(),0);
        let weights_before=full.phase_native_drive_weights().unwrap();

        let outcome=acquire(&mut full,world,false);
        assert!(outcome.reward,"G16 FULL target {index} failed: {world:?}");
        assert!(outcome.interactions<=TARGET_BUDGET);
        assert_eq!(full.phase_native_drive_weights().unwrap(),weights_before);
        assert_eq!(full.planning_transition_count(),0);
        endpoint_errors+=endpoint_violations(&full);
        total_cost+=outcome.interactions;
        full_success+=1;

        full.set_planning_learning_enabled(false);
        full.set_phase_native_drive_readout_enabled(false);
        full_plan+=usize::from(heldout_delayed_plan(&mut full,world));

        let checkpoint_full=full.phase_native_checkpoint().expect("G16 checkpoint");
        let mut restarted=restore(checkpoint_full);
        restarted.set_planning_learning_enabled(false);
        restarted.set_phase_native_drive_readout_enabled(false);
        restart_plan+=usize::from(heldout_delayed_plan(&mut restarted,world));

        let mut zero=target_zero_drive();
        let z=acquire(&mut zero,world,false);
        zero_success+=usize::from(z.reward);

        let mut lesion=target_with_drive(&checkpoint);
        let frontier=lesion.phase_native_drive_synapses().unwrap()[1];
        lesion.perturb_phase_native_synapse_for_control(frontier,0.0,0.0)
            .expect("G16 frontier synapse");
        let l=acquire(&mut lesion,world,false);
        lesion_success+=usize::from(l.reward);

        let mut direct=target_with_drive(&checkpoint);
        let d=acquire(&mut direct,world,true);
        direct_success+=usize::from(d.reward);

        random_success+=usize::from(
            random_acquire(world,0xA37E_1600+index as u64).reward
        );

        let mut no_learning=target_with_drive(&checkpoint);
        no_learning.set_planning_learning_enabled(false);
        let _=acquire(&mut no_learning,world,false);
        no_learning_plan+=usize::from(heldout_delayed_plan(&mut no_learning,world));

        let mut no_growth=target_with_drive(&checkpoint);
        no_growth.set_structural_growth_for_control(false);
        let _=acquire(&mut no_growth,world,false);
        no_growth_plan+=usize::from(heldout_delayed_plan(&mut no_growth,world));
    }

    let mean=total_cost as f64/12.0;
    println!(
        "G16_AUTONOMOUS_ABSTRACT full_reward={}/12 full_plan={}/12 restart_plan={}/12 zero={}/12 frontier_lesion={}/12 direct={}/12 random={}/12 no_learning_plan={}/12 no_growth_plan={}/12 endpoint_errors={} mean_cost={:.3} drive_weights={:?}",
        full_success,full_plan,restart_plan,zero_success,lesion_success,
        direct_success,random_success,no_learning_plan,no_growth_plan,
        endpoint_errors,mean,learned_weights
    );

    assert_eq!(full_success,12);
    assert_eq!(full_plan,12);
    assert_eq!(restart_plan,12);
    assert_eq!(endpoint_errors,0);
    assert!(zero_success<=3);
    assert!(lesion_success<=3);
    assert!(direct_success<=4);
    assert_eq!(no_learning_plan,0);
    assert_eq!(no_growth_plan,0);
    assert!(mean<=45.0);
}

#[test]
fn g16_abstract_drive_selector_has_no_route_or_search_fallback(){
    let abstract_source=include_str!("../src/phase_abstract_planning.rs");
    let drive_source=include_str!("../src/phase_drive.rs");

    let start=abstract_source
        .find("pub fn choose_phase_native_abstract_learned_drive_action")
        .expect("G16 abstract drive selector");
    let selector=&abstract_source[start..];

    for forbidden in [
        "ResetWorld",
        "delayed_first",
        "advance",
        "immediate",
        "correct_action",
        "route",
        "EvoImaginationPlanner",
        "BinaryHeap",
        "VecDeque",
    ] {
        assert!(!selector.contains(forbidden),
            "G16 selector contains forbidden token {forbidden}");
    }

    for required in [
        "phase_native_abstract_state",
        "phase_drive_frontier_activity_for_cells",
        "phase_drive_features",
        "phase_drive_score",
        "pending_features",
        "phase_native_decision_from_cell",
    ] {
        assert!(selector.contains(required),
            "G16 selector missing dependency {required}");
    }

    assert!(drive_source.contains("phase_drive_frontier_activity_for_cells"));
    assert!(drive_source.contains("state_cells"));
}
