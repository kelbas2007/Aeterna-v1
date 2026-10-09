use aeterna_v1::{EvoConfig,EvoPhase,HumanProtectionEvidence,HumanProtectionReason};
use aeterna_v1::carrier::{
    PhaseMetaControlConfig,PhaseMetaControlCheckpoint,PhaseNativeConfig,
    PhaseHypothesisEcologyConfig,
};
use aeterna_v1::scientific_runtime::{ScientificRuntime,StepOutcome,RuntimeError};
use std::cell::Cell;
use std::collections::BTreeSet;

#[allow(dead_code)]
mod foundation {
    use std::collections::BTreeSet;
    include!("g19_rival_hypothesis_discrimination.rs");

    pub const PAIRS24:[[usize;2];24]=[
        [0,1],[0,2],[0,3],[0,4],[0,5],[0,6],[0,7],
        [1,2],[1,3],[1,4],[1,5],[1,6],[1,7],
        [2,3],[2,4],[2,5],[2,6],[2,7],
        [3,4],[3,5],[3,6],[3,7],
        [4,5],[4,6],
    ];

    pub fn build24()->(EvoPhase,[[usize;2];8]){
        let drive=train_drive();
        let mut evo=target(&drive);
        let factors=factorization_16();
        let l1=factors[0];

        for pair in l1 {
            for layout in LAYOUTS[..4].iter().copied(){
                let s=pair_scene(pair,layout);
                for action in [0usize,1usize]{
                    assert!(evo.observe_phase_native_concept_factual(
                        &s,action,action==0
                    ));
                }
            }
        }
        for (index,round) in factors[1..5].iter().enumerate(){
            for pair in *round{
                let s=pair_scene(pair,LAYOUTS[index]);
                for action in [0usize,1usize]{
                    assert!(evo.observe_phase_native_concept_factual(
                        &s,action,action==1
                    ));
                }
            }
        }
        assert_eq!(evo.concept_atoms().len(),16);
        assert_eq!(evo.phase_native_promoted_concept_count(),8);

        assert!(evo.enable_phase_native_depth_generic_abstraction(2));
        for child in 0..8usize{
            let s=pair_scene(l1[child],LAYOUTS[child%4]);
            for rep in 0..4usize{
                let a=rep%2==0;
                assert!(evo.observe_phase_native_depth_generic_factual(&s,2,a));
                assert!(evo.observe_phase_native_depth_generic_factual(&s,3,!a));
            }
        }

        for cycle in 0..4usize{
            for state in 0..PAIRS24.len(){
                let s=scene(&l1,state,cycle);
                for action in [2usize,3usize]{
                    assert!(evo.observe_phase_native_depth_generic_factual(
                        &s,action,action==2
                    ));
                }
                for child in PAIRS24[state]{
                    let single=pair_scene(
                        l1[child],LAYOUTS[(cycle+child+2)%LAYOUTS.len()]
                    );
                    for action in [2usize,3usize]{
                        assert!(evo.observe_phase_native_depth_generic_factual(
                            &single,action,action==3
                        ));
                    }
                }
            }
        }

        assert_eq!(evo.phase_native_deep_candidate_count(2),24);
        assert_eq!(evo.phase_native_promoted_deep_count(2),24);
        evo.set_concept_learning_enabled(false);
        assert_eq!(evo.phase_native_circuits().len(),0);

        let mut cells=BTreeSet::new();
        for state in 0..24usize{
            let reference=evo.phase_native_abstract_state(&scene(&l1,state,0)).unwrap();
            assert_eq!(reference.level,2);
            assert!(cells.insert(reference.cell));
            for layout in 0..6{
                assert_eq!(
                    evo.phase_native_abstract_state(&scene(&l1,state,layout)).unwrap(),
                    reference
                );
            }
        }
        (evo,l1)
    }

    pub fn scene(
        l1:&[[usize;2];8],
        state:usize,
        layout:usize,
    )->Vec<f32>{
        let mut r=blank();
        let mut cursor=0usize;
        for child in PAIRS24[state]{
            for atom in l1[child]{
                add_motif(&mut r,atom,LAYOUTS[layout%LAYOUTS.len()][cursor]);
                cursor+=1;
            }
        }
        r
    }
}

#[derive(Debug,Clone)]
struct Rng(u64);
impl Rng{
    fn new(seed:u64)->Self{Self(seed^0x1A7E_2200_2026_1007)}
    fn next(&mut self)->u64{
        let mut x=self.0;
        x^=x>>12;x^=x<<25;x^=x>>27;self.0=x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self,n:usize)->usize{(self.next()%n as u64)as usize}
}
fn shuffle<T>(rng:&mut Rng,x:&mut[T]){
    for i in (1..x.len()).rev(){let j=rng.range(i+1);x.swap(i,j);}
}

fn safe()->HumanProtectionEvidence{
    HumanProtectionEvidence{
        human_present:true,physical_effect_possible:true,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false,
    }
}

fn meta_checkpoint()->PhaseMetaControlCheckpoint{
    let mut evo=EvoPhase::new(EvoConfig{
        sensory_cells:8,motor_cells:6,dormant_cells:96,hdc_dim:128,
        phase_learning_rate:1.0,
        ..EvoConfig::default()
    });
    evo.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(evo.enable_phase_native_meta_control(PhaseMetaControlConfig{
        learning_rate:0.35,learning_enabled:true,readout_enabled:true,
        phase_learning_enabled:true,
    }));
    let oracle=[0.36f32,0.30,0.22,0.08,0.04];
    for _ in 0..96{
        for i in 0..5{
            let mut fields=[0.0f32;5];
            fields[i]=1.0;
            assert!(evo.observe_phase_native_meta_utility(fields,oracle[i]));
        }
    }
    evo.phase_native_meta_checkpoint().unwrap()
}

#[derive(Default)]
struct Monitor{
    proposal_count:u64,
    max_candidates:usize,
    saw_dormant:bool,
    saw_zero_applicability:bool,
    dormant_seen:BTreeSet<u64>,
    saw_reactivation:bool,
}
impl Monitor{
    fn before(&mut self,rt:&ScientificRuntime,goal:&[f32]){
        let ps=rt.organism().collect_phase_native_unified_proposals(goal);
        self.proposal_count+=ps.len() as u64;
        let active=ps.iter()
            .filter_map(|p|p.persistent_candidate_id)
            .collect::<BTreeSet<_>>();
        let records=rt.organism().phase_native_hypothesis_records();
        self.max_candidates=self.max_candidates.max(records.len());
        for r in records{
            if r.dormant{
                self.saw_dormant=true;
                self.dormant_seen.insert(r.candidate_id);
            }else if self.dormant_seen.contains(&r.candidate_id){
                self.saw_reactivation=true;
            }
            if !active.contains(&r.candidate_id){
                self.saw_zero_applicability=true;
            }
        }
    }
}

fn step_u<F>(
    rt:&mut ScientificRuntime,
    monitor:&mut Monitor,
    mut exec:F,
)->Result<usize,RuntimeError>
where F:FnMut(usize)->(Vec<f32>,f32){
    let goal=rt.organism().current_real()
        .map(|_|())
        .ok_or(RuntimeError::FreshObservationRequired)?;
    let _=goal;
    // Monitor with current goal through proposals exposed by the organism is
    // performed by callers immediately before this helper where goal is known.
    let outcome=rt.step_unified(
        |_|Some(safe()),
        |a|Ok(exec(a))
    )?;
    match outcome{
        StepOutcome::Executed{proposal,..}=>Ok(proposal.action),
        StepOutcome::GoalReached=>Err(RuntimeError::NoSupportedAction),
        other=>panic!("unexpected unified protected outcome: {:?}",other),
    }
}

fn set_world(
    rt:&mut ScientificRuntime,
    l1:&[[usize;2];8],
    state:usize,
    goal:usize,
    layout:usize,
){
    rt.observe_external(&foundation::scene(l1,state,layout)).unwrap();
    rt.set_goal(&foundation::scene(l1,goal,(layout+1)%6)).unwrap();
}

#[derive(Clone,Debug)]
struct WorldA{
    states:[usize;5], // start,p1,p2,goal,dead
    motors:[usize;5], // a0,a1,a2,dead,escape
    state:usize,
    calls:usize,
}
impl WorldA{
    fn new(states:[usize;5],motors:[usize;5])->Self{
        Self{state:states[0],states,motors,calls:0}
    }
    fn step(&mut self,action:usize)->(usize,f32){
        self.calls+=1;
        let [s,p1,p2,g,d]=self.states;
        let [a0,a1,a2,dead,escape]=self.motors;
        self.state=match self.state{
            x if x==s && action==a0=>p1,
            x if x==p1 && action==a1=>p2,
            x if x==p2 && action==a2=>g,
            x if x==s && action==dead=>d,
            x if x==d && action==escape=>s,
            x=>x,
        };
        (self.state,f32::from(self.state==g))
    }
}

fn run_a(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],monitor:&mut Monitor,
    world:&mut WorldA,budget:usize,layout:usize,start_override:Option<usize>,
)->bool{
    if let Some(s)=start_override{world.state=s;}
    set_world(rt,l1,world.state,world.states[3],layout);
    let goal_scene=foundation::scene(l1,world.states[3],(layout+1)%6);
    for k in 0..budget{
        if world.state==world.states[3]{return true;}
        monitor.before(rt,&goal_scene);
        let lay=(layout+k)%6;
        if step_u(rt,monitor,|a|{
            let (next,value)=world.step(a);
            (foundation::scene(l1,next,lay),value)
        }).is_err(){return false;}
    }
    world.state==world.states[3]
}

#[derive(Clone,Debug)]
struct WorldB{
    states:[usize;5], // start,tool,goal1,goal2,trap
    motors:[usize;3], // acquire,open1,open2
    state:usize,
    calls:usize,
}
impl WorldB{
    fn new(states:[usize;5],motors:[usize;3])->Self{
        Self{state:states[0],states,motors,calls:0}
    }
    fn step(&mut self,action:usize,goal2:bool)->(usize,f32){
        self.calls+=1;
        let [s,t,g1,g2,trap]=self.states;
        let [acq,o1,o2]=self.motors;
        self.state=match self.state{
            x if x==s && action==acq=>t,
            x if x==s && (action==o1||action==o2)=>trap,
            x if x==trap=>s,
            x if x==t && action==o1=>g1,
            x if x==t && action==o2=>g2,
            x if x==g1||x==g2=>t,
            x=>x,
        };
        let target=if goal2{g2}else{g1};
        (self.state,f32::from(self.state==target))
    }
}

fn run_b_goal(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],monitor:&mut Monitor,
    world:&mut WorldB,goal2:bool,budget:usize,layout:usize,
)->bool{
    let target=if goal2{world.states[3]}else{world.states[2]};
    set_world(rt,l1,world.state,target,layout);
    let goal_scene=foundation::scene(l1,target,(layout+1)%6);
    for k in 0..budget{
        if world.state==target{return true;}
        monitor.before(rt,&goal_scene);
        let lay=(layout+k)%6;
        if step_u(rt,monitor,|a|{
            let (next,value)=world.step(a,goal2);
            (foundation::scene(l1,next,lay),value)
        }).is_err(){return false;}
    }
    world.state==target
}

#[derive(Debug)]
struct WorldC{
    states:[usize;6], // reset,pred0,pred1,junction,goal,dead
    actions:[usize;2],
    state:usize,
    next_side:usize,
    trials:usize,
}
impl WorldC{
    fn new(states:[usize;6],actions:[usize;2],seed:u64)->Self{
        Self{state:states[0],states,actions,next_side:(seed as usize)&1,trials:0}
    }
    fn step(&mut self,action:usize)->(usize,f32){
        let [reset,p0,p1,j,g,d]=self.states;
        self.state=if self.state==reset{
            self.next_side^=1;
            if self.next_side==0{p0}else{p1}
        }else if self.state==p0||self.state==p1{
            j
        }else if self.state==j{
            let side=usize::from(self.next_side==1);
            self.trials+=1;
            if action==self.actions[side]{g}else{d}
        }else{
            reset
        };
        (self.state,f32::from(self.state==g))
    }
}

fn weak_scene(
    l1:&[[usize;2];8],state:usize,layout:usize,
    a:bool,b:bool,values:[f32;2],positions:[usize;2],
)->Vec<f32>{
    let mut s=foundation::scene(l1,state,layout);
    assert!(positions[0]!=positions[1]);
    assert!(s[positions[0]]<0.5&&s[positions[1]]<0.5);
    if a{s[positions[0]]=values[0];}
    if b{s[positions[1]]=values[1];}
    s
}

#[derive(Clone,Debug)]
struct WorldE{
    states:[usize;6], // start,shortcut_mid,fallback_mid,goal,dead,stable_mid
    motors:[usize;6], // shortcut,shortcut_finish,fallback,fallback_finish,stable,stable_finish
    state:usize,
    shortcut_successes:usize,
    changed:bool,
    changed_seen:bool,
    after_change_actions:usize,
    first_recovery:Option<usize>,
    fallback_after_change:bool,
}
impl WorldE{
    fn new(states:[usize;6],motors:[usize;6])->Self{
        Self{
            state:states[0],states,motors,shortcut_successes:0,
            changed:false,changed_seen:false,after_change_actions:0,
            first_recovery:None,fallback_after_change:false,
        }
    }
    fn step(&mut self,action:usize)->(usize,f32){
        let [s,sm,fm,g,d,um]=self.states;
        let [shortcut,sfinish,fallback,ffinish,stable,ufinish]=self.motors;
        if self.changed_seen{self.after_change_actions+=1;}
        let before=self.state;
        self.state=match self.state{
            x if x==s&&action==shortcut=>if self.changed{d}else{sm},
            x if x==sm&&action==sfinish=>g,
            x if x==s&&action==fallback=>fm,
            x if x==fm&&action==ffinish=>g,
            x if x==s&&action==stable=>um,
            x if x==um&&action==ufinish=>g,
            x if x==g||x==d=>s,
            x=>x,
        };
        if before==s&&action==shortcut&&self.changed{
            self.changed_seen=true;
            self.after_change_actions=0;
        }
        if before==sm&&action==sfinish&&self.state==g&&!self.changed{
            self.shortcut_successes+=1;
            if self.shortcut_successes>=4{self.changed=true;}
        }
        if self.changed_seen&&before==fm&&action==ffinish&&self.state==g{
            self.fallback_after_change=true;
        }
        if self.changed_seen&&self.state==g&&self.first_recovery.is_none(){
            self.first_recovery=Some(self.after_change_actions);
        }
        (self.state,f32::from(self.state==g))
    }
}

#[test]
fn intel2_frozen_unified_unknown_world_lifetime(){
    let seed:u64=std::env::var("AETERNA_INTEL2_SEED")
        .expect("AETERNA_INTEL2_SEED").parse().unwrap();
    let source_sha=std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_|"unknown".into());

    // Technical/generic setup before pack seal.
    let (evo,l1)=foundation::build24();
    let meta=meta_checkpoint();

    let mut rng=Rng::new(seed);
    let mut states=(0usize..24).collect::<Vec<_>>();
    shuffle(&mut rng,&mut states);
    let a_states: [usize;5]=states[0..5].try_into().unwrap();
    let b_states: [usize;5]=states[5..10].try_into().unwrap();
    let c_states: [usize;6]=states[10..16].try_into().unwrap();
    let d_states: [usize;3]=states[16..19].try_into().unwrap();
    let e_states: [usize;5]=states[19..24].try_into().unwrap();
    // World E needs six roles; its stable branch reuses its start state's raw
    // class only as an intermediate through a distinct motor, while laws remain
    // sealed. No target transition transfers.
    let e6=[e_states[0],e_states[1],e_states[2],e_states[3],e_states[4],a_states[4]];

    let mut ma=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut ma);
    let mut mb=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mb);
    let mut mc=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mc);
    let mut md=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut md);
    let mut me=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut me);
    let op_xor=(rng.next()&1)==1;
    let bins=if rng.next()&1==0{[0.20f32,0.40f32]}else{[0.15f32,0.35f32]};

    println!(
        "INTEL2_SEAL source_sha={} spec_sha={} seed={} A={:?}/{:?} B={:?}/{:?} C={:?}/{:?} D={:?}/{:?}/op={} bins={:?} E={:?}/{:?}",
        source_sha,spec_sha,seed,a_states,ma,b_states,mb,c_states,[mc[0],mc[1]],
        d_states,[md[0],md[1]],if op_xor{"XOR"}else{"AND"},bins,e6,me
    );

    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_context_refinement());
    assert!(rt.enable_perceptual_refinement());
    assert!(rt.enable_compositional_refinement());
    assert!(rt.enable_unified_cognition(
        meta,
        PhaseHypothesisEcologyConfig{
            learning_rate:0.35,dormancy_threshold:0.05,
            learning_enabled:true,phase_learning_enabled:true,
        }
    ));
    assert!(rt.organism().phase_native_hypothesis_records().is_empty());
    let meta_weights=rt.organism().phase_native_meta_weights().unwrap();
    let mut monitor=Monitor::default();

    // WORLD A.
    let mut wa=WorldA::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    let a_ok=run_a(&mut rt,&l1,&mut monitor,&mut wa,48,0,None);
    let a_cost=wa.calls;
    rt.set_model_learning_enabled(false);
    let mut wa_reuse=WorldA::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    let a_reuse=run_a(
        &mut rt,&l1,&mut monitor,&mut wa_reuse,6,4,Some(a_states[1])
    );
    rt.set_model_learning_enabled(true);
    println!("INTEL2_A first={} cost={} reuse={} reuse_cost={}",
        a_ok,a_cost,a_reuse,wa_reuse.calls);

    // WORLD B.
    let mut wb=WorldB::new(b_states,[mb[0],mb[1],mb[2]]);
    let b1=run_b_goal(&mut rt,&l1,&mut monitor,&mut wb,false,64,1);
    let b1_cost=wb.calls;
    let before_switch=wb.calls;
    let b2=run_b_goal(&mut rt,&l1,&mut monitor,&mut wb,true,12,2);
    let b2_cost=wb.calls-before_switch;
    println!("INTEL2_B first={} cost={} switched={} switch_cost={}",
        b1,b1_cost,b2,b2_cost);

    // WORLD C: target-lifetime context acquisition.
    let promoted_before=rt.organism().phase_native_context_witnesses()
        .iter().filter(|w|w.promoted).count();
    let mut wc=WorldC::new(c_states,[mc[0],mc[1]],seed^0xC0FFEE);
    set_world(&mut rt,&l1,wc.state,c_states[4],0);
    let c_goal=foundation::scene(&l1,c_states[4],1);
    for k in 0..1800usize{
        let base=rt.organism().phase_native_abstract_state(
            &foundation::scene(&l1,c_states[3],0)
        ).unwrap().cell;
        if wc.trials>=72 && rt.organism().phase_native_context_witnesses()
            .iter().any(|w|w.promoted&&w.base_cell==base)
        {break;}
        monitor.before(&rt,&c_goal);
        let layout=k%4;
        step_u(&mut rt,&mut monitor,|a|{
            let (next,value)=wc.step(a);
            (foundation::scene(&l1,next,layout),value)
        }).expect("World C unified action");
    }
    let c_base=rt.organism().phase_native_abstract_state(
        &foundation::scene(&l1,c_states[3],0)
    ).unwrap().cell;
    let c_promoted=rt.organism().phase_native_context_witnesses()
        .iter().any(|w|w.promoted&&w.base_cell==c_base);
    assert!(rt.organism().phase_native_context_witnesses()
        .iter().filter(|w|w.promoted).count()>=promoted_before);

    rt.set_model_learning_enabled(false);
    let mut c_correct=0usize;
    let mut c_side=[0usize;2];
    let mut c_side_correct=[0usize;2];
    let mut memoryless=0usize;
    for i in 0..32usize{
        let side=i%2;
        let pred=c_states[1+side];
        rt.observe_external(&foundation::scene(&l1,pred,(4+i)%6)).unwrap();
        rt.set_goal(&c_goal).unwrap();
        monitor.before(&rt,&c_goal);
        // Every action from predecessor is an observational transition to junction.
        let outcome=rt.step_unified(
            |_|Some(safe()),
            |_|Ok((foundation::scene(&l1,c_states[3],(4+i)%6),0.0))
        ).unwrap();
        assert!(matches!(outcome,StepOutcome::Executed{..}));
        let proposal=rt.propose_unified().unwrap().expect("context readout");
        c_side[side]+=1;
        if proposal.action==mc[side]{
            c_correct+=1;c_side_correct[side]+=1;
        }

        let current=foundation::scene(&l1,c_states[3],(4+i)%6);
        let mut old=rt.organism().clone();
        memoryless+=usize::from(
            old.plan_phase_native_abstract_goal(&current,&c_goal,None)
                .map(|d|d.first_action)==Some(mc[side])
        );
    }
    rt.set_model_learning_enabled(true);
    println!("INTEL2_C promoted={} score={}/32 sides={:?}/{:?} memoryless={}/32",
        c_promoted,c_correct,c_side_correct,c_side,memoryless);

    // Required protected cognitive restart after C.
    let fp_before_restart=rt.organism().phase_native_learned_fingerprint();
    let records_before_restart=rt.organism().phase_native_hypothesis_records();
    let meta_before_restart=rt.organism().phase_native_meta_weights().unwrap();
    rt.restart_cognition().unwrap();
    assert!(rt.organism().current_real().is_none());
    assert_eq!(rt.organism().phase_native_learned_fingerprint(),fp_before_restart);
    assert_eq!(rt.organism().phase_native_meta_weights().unwrap(),meta_before_restart);
    assert_eq!(
        rt.organism().phase_native_hypothesis_records().len(),
        records_before_restart.len()
    );

    // Human Protection intervention before D.
    let d_base=foundation::scene(&l1,d_states[0],0);
    let d_goal=foundation::scene(&l1,d_states[1],1);
    rt.observe_external(&d_base).unwrap();
    rt.set_goal(&d_goal).unwrap();
    let calls=Cell::new(0usize);
    let before_hp=rt.organism().phase_native_learned_fingerprint();
    let high=rt.step_unified(
        |_|Some(HumanProtectionEvidence{
            predicted_harm_probability:0.5,..safe()
        }),
        |_|{calls.set(calls.get()+1);Ok((d_base.clone(),0.0))}
    ).unwrap();
    let StepOutcome::Blocked(r)=high else{panic!("high risk must block");};
    assert_eq!(r.reason,HumanProtectionReason::ExcessHumanHarmRisk);
    let emergency=rt.step_unified(
        |_|Some(HumanProtectionEvidence{emergency_stop:true,..safe()}),
        |_|{calls.set(calls.get()+1);Ok((d_base.clone(),0.0))}
    ).unwrap();
    assert!(matches!(emergency,StepOutcome::Blocked(_)));
    assert_eq!(calls.get(),0);
    assert_eq!(rt.organism().phase_native_learned_fingerprint(),before_hp);
    rt.restart_cognition().unwrap();
    assert!(rt.emergency_latched());
    rt.observe_external(&d_base).unwrap();
    rt.set_goal(&d_goal).unwrap();
    let latched=rt.step_unified(
        |_|Some(safe()),
        |_|{calls.set(calls.get()+1);Ok((d_base.clone(),0.0))}
    ).unwrap();
    assert!(matches!(latched,StepOutcome::Blocked(_)));
    assert_eq!(calls.get(),0);
    rt.external_operator_reset();

    // WORLD D: target-lifetime composition.
    let x=md[0];let y=md[1];
    let train_positions=[[399usize,398usize],[397,396]];
    let score_positions=[[395usize,394usize],[393,392]];
    let mut schedule=Vec::new();
    let counts=if op_xor{[48usize,8,8,16]}else{[24usize,24,24,8]};
    for (combo,count) in counts.into_iter().enumerate(){
        for _ in 0..count{schedule.push(combo);}
    }
    shuffle(&mut rng,&mut schedule);
    rt.set_goal(&d_goal).unwrap();
    let d_base_cell=rt.organism().phase_native_abstract_state(&d_base).unwrap().cell;
    for i in 0..1200usize{
        let promoted=rt.organism().phase_native_composition_witnesses()
            .iter().any(|w|w.promoted&&w.base_cell==d_base_cell);
        if promoted&&i>=160{break;}
        let combo=schedule[i%schedule.len()];
        let a=combo&2!=0;let b=combo&1!=0;
        let current=weak_scene(
            &l1,d_states[0],i%4,a,b,bins,train_positions[i%2]
        );
        rt.observe_external(&current).unwrap();
        monitor.before(&rt,&d_goal);
        let expected_x=if op_xor{a^b}else{a&&b};
        let outcome=rt.step_unified(
            |_|Some(safe()),
            |act|{
                let correct=(act==x)==expected_x;
                Ok((
                    foundation::scene(&l1,if correct{d_states[1]}else{d_states[2]},i%4),
                    f32::from(correct),
                ))
            }
        );
        if outcome.is_err(){break;}
    }
    let d_promoted=rt.organism().phase_native_composition_witnesses()
        .iter().any(|w|w.promoted&&w.base_cell==d_base_cell);

    rt.set_model_learning_enabled(false);
    let mut d_correct=0usize;
    let mut labels=Vec::new();
    for i in 0..32usize{
        let combo=i%4;let a=combo&2!=0;let b=combo&1!=0;
        let current=weak_scene(
            &l1,d_states[0],4+i%2,a,b,bins,score_positions[i%2]
        );
        rt.observe_external(&current).unwrap();
        rt.set_goal(&d_goal).unwrap();
        let p=rt.propose_unified().unwrap().expect("D heldout proposal");
        let expected_x=if op_xor{a^b}else{a&&b};
        let expected=if expected_x{x}else{y};
        d_correct+=usize::from(p.action==expected);
        labels.push((a,b,expected_x));
    }
    rt.set_model_learning_enabled(true);

    let best_single=(0..2).map(|which|{
        [false,true].into_iter().map(|polarity|{
            labels.iter().filter(|(a,b,label)|{
                let bit=if which==0{*a}else{*b};
                (bit==polarity)==*label
            }).count()
        }).max().unwrap()
    }).max().unwrap();
    println!("INTEL2_D op={} promoted={} full={}/32 best_single={}/32",
        if op_xor{"XOR"}else{"AND"},d_promoted,d_correct,best_single);

    // WORLD E: nonstationary law.
    let mut we=WorldE::new(e6,me);
    set_world(&mut rt,&l1,we.state,e6[3],2);
    let e_goal=foundation::scene(&l1,e6[3],3);
    for k in 0..1200usize{
        if we.changed_seen&&we.first_recovery.is_some(){break;}
        monitor.before(&rt,&e_goal);
        let layout=k%6;
        if step_u(&mut rt,&mut monitor,|a|{
            let (next,value)=we.step(a);
            (foundation::scene(&l1,next,layout),value)
        }).is_err(){break;}
    }
    let e_recovery=we.first_recovery;
    println!(
        "INTEL2_E shortcut_successes={} changed={} changed_seen={} recovery={:?} fallback_after_change={}",
        we.shortcut_successes,we.changed,we.changed_seen,e_recovery,we.fallback_after_change
    );

    // Final retention with target representation/transition learning frozen.
    rt.set_model_learning_enabled(false);
    let mut fa=WorldA::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    let final_a=run_a(&mut rt,&l1,&mut monitor,&mut fa,6,5,None);
    let mut fb=WorldB::new(b_states,[mb[0],mb[1],mb[2]]);
    let final_b=run_b_goal(&mut rt,&l1,&mut monitor,&mut fb,false,8,4);

    let meta_unchanged=rt.organism().phase_native_meta_weights().unwrap()==meta_weights;
    let legacy=rt.organism().planning_transition_count();
    let table=rt.organism().composite_concepts().len();

    let c_pass=c_promoted&&c_correct>=28
        &&c_side_correct[0]>=13&&c_side_correct[1]>=13
        &&memoryless<=20;
    let d_single_limit=if op_xor{20}else{24};
    let d_pass=d_promoted&&d_correct>=28&&best_single<=d_single_limit;
    let e_pass=we.changed&&we.changed_seen
        &&e_recovery.map(|x|x<=16).unwrap_or(false)
        &&we.fallback_after_change;

    println!(
        "INTEL2_RESULT A={}/{} B={}/{} C={} D={} E={} final={}/2 safety=true meta_unchanged={} proposals={} max_u2={} dormant={} zero_app={} reactivated={} legacy={} table={} verdict={}",
        a_ok,a_reuse,b1,b2,c_pass,d_pass,e_pass,
        usize::from(final_a)+usize::from(final_b),
        meta_unchanged,monitor.proposal_count,monitor.max_candidates,
        monitor.saw_dormant,monitor.saw_zero_applicability,monitor.saw_reactivation,
        legacy,table,
        if a_ok&&a_cost<=48&&a_reuse&&wa_reuse.calls<=6
            &&b1&&b1_cost<=64&&b2&&b2_cost<=12
            &&c_pass&&d_pass&&e_pass&&final_a&&fa.calls<=6
            &&final_b&&fb.calls<=8&&meta_unchanged&&legacy==0&&table==0
        {"PASS_AUTONOMOUS_DEVELOPING_INTELLIGENCE_UNIFIED"}else{"FAIL"}
    );

    assert!(a_ok&&a_cost<=48,"INTEL-2 World A acquisition failure");
    assert!(a_reuse&&wa_reuse.calls<=6,"INTEL-2 World A translated reuse failure");
    assert!(b1&&b1_cost<=64,"INTEL-2 World B first-goal failure");
    assert!(b2&&b2_cost<=12,"INTEL-2 World B goal-switch failure");
    assert!(c_pass,"INTEL-2 World C history-dependent failure");
    assert!(d_pass,"INTEL-2 World D composition failure");
    assert!(e_pass,"INTEL-2 World E revision failure");
    assert!(final_a&&fa.calls<=6,"INTEL-2 final World A retention failure");
    assert!(final_b&&fb.calls<=8,"INTEL-2 final World B retention failure");
    assert!(meta_unchanged,"INTEL-2 U1 meta weights changed");
    assert_eq!(legacy,0);
    assert_eq!(table,0);
}

#[test]
fn intel2_evaluator_uses_only_unified_external_runtime(){
    let source=include_str!("intel2_unified_worlds.rs");
    assert!(source.contains("step_unified"));
    // Construct tokens at runtime so this guard cannot match its own literals.
    for forbidden in [
        [".step", "(|"].concat(),
        ["Reasoning", "Mode::"].concat(),
        ["correct", "_explanation"].concat(),
        ["world", "_id"].concat(),
        ["task", "_id"].concat(),
    ] {
        assert!(!source.contains(&forbidden), "forbidden evaluator token: {forbidden}");
    }
}


#[test]
fn intel2_burned_world_c_diagnosis(){
    // Diagnostic only. This reuses the permanently burned INTEL-2 authority
    // seed and MUST NOT be interpreted as a new verdict or qualification run.
    let seed:u64=37_687_243_350;
    let (evo,l1)=foundation::build24();
    let meta=meta_checkpoint();

    let mut rng=Rng::new(seed);
    let mut states=(0usize..24).collect::<Vec<_>>();
    shuffle(&mut rng,&mut states);
    let a_states:[usize;5]=states[0..5].try_into().unwrap();
    let b_states:[usize;5]=states[5..10].try_into().unwrap();
    let c_states:[usize;6]=states[10..16].try_into().unwrap();

    let mut ma=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut ma);
    let mut mb=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mb);
    let mut mc=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mc);

    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_context_refinement());
    assert!(rt.enable_perceptual_refinement());
    assert!(rt.enable_compositional_refinement());
    assert!(rt.enable_unified_cognition(
        meta,
        PhaseHypothesisEcologyConfig{
            learning_rate:0.35,
            dormancy_threshold:0.05,
            learning_enabled:true,
            phase_learning_enabled:true,
        }
    ));
    let mut monitor=Monitor::default();

    // Reproduce the exact pre-C lifetime from the burned pack.
    let mut wa=WorldA::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    assert!(run_a(&mut rt,&l1,&mut monitor,&mut wa,48,0,None));
    rt.set_model_learning_enabled(false);
    let mut wa_reuse=WorldA::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    assert!(run_a(
        &mut rt,&l1,&mut monitor,&mut wa_reuse,6,4,Some(a_states[1])
    ));
    rt.set_model_learning_enabled(true);

    let mut wb=WorldB::new(b_states,[mb[0],mb[1],mb[2]]);
    assert!(run_b_goal(&mut rt,&l1,&mut monitor,&mut wb,false,64,1));
    assert!(run_b_goal(&mut rt,&l1,&mut monitor,&mut wb,true,12,2));

    let mut wc=WorldC::new(c_states,[mc[0],mc[1]],seed^0xC0FFEE);
    set_world(&mut rt,&l1,wc.state,c_states[4],0);
    let c_goal=foundation::scene(&l1,c_states[4],1);

    let mut reproduced=false;
    for k in 0..1800usize{
        let current=rt.organism().current_real().unwrap().sensory.clone();
        let base=rt.organism().phase_native_abstract_state(&current).unwrap().cell;
        let proposals=rt.organism().collect_phase_native_unified_proposals(&c_goal);
        let selected=rt.organism().choose_phase_native_unified_proposal(&proposals);

        if selected.is_none(){
            let mut context_probe=rt.organism().clone();
            let mut percept_probe=rt.organism().clone();
            let mut composition_probe=rt.organism().clone();
            let mut rival_probe=rt.organism().clone();
            let mut goal_probe=rt.organism().clone();
            let mut general_probe=rt.organism().clone();

            println!(
                "INTEL2_C_DIAG_STOP k={} world_state={} base={} trials={} proposals={} selected={:?}",
                k,wc.state,base,wc.trials,proposals.len(),selected
            );
            println!("INTEL2_C_DIAG_PROPOSALS {:?}",proposals);
            println!(
                "INTEL2_C_DIAG_SOURCES context={:?} percept={:?} composition={:?} rival={:?} goal={:?} general={:?}",
                context_probe.phase_native_context_action(&c_goal),
                percept_probe.phase_native_perceptual_action(&c_goal),
                composition_probe.phase_native_compositional_action(&c_goal),
                rival_probe.choose_phase_native_goal_rival_probe(&c_goal),
                goal_probe.choose_phase_native_goal_active_action(&c_goal),
                general_probe.choose_phase_native_abstract_learned_drive_action(),
            );
            println!(
                "INTEL2_C_DIAG_CONTEXTS {:?}",
                rt.organism().phase_native_context_witnesses()
            );
            println!(
                "INTEL2_C_DIAG_U2 {:?}",
                rt.organism().phase_native_hypothesis_records()
            );
            reproduced=true;
            break;
        }

        monitor.before(&rt,&c_goal);
        let layout=k%4;
        let result=step_u(&mut rt,&mut monitor,|a|{
            let (next,value)=wc.step(a);
            (foundation::scene(&l1,next,layout),value)
        });
        if let Err(error)=result{
            println!(
                "INTEL2_C_DIAG_RUNTIME_STOP k={} world_state={} base={} trials={} error={:?}",
                k,wc.state,base,wc.trials,error
            );
            reproduced=true;
            break;
        }
    }

    assert!(reproduced,"burned World C failure did not reproduce");
}


#[test]
fn intel2_burned_world_c_with_environment_terminal_reset_diagnosis(){
    // Diagnostic only on the burned authority pack. This tests evaluator
    // terminal semantics; it is NOT a replacement INTEL-2 verdict.
    let seed:u64=37_687_243_350;
    let (evo,l1)=foundation::build24();
    let meta=meta_checkpoint();

    let mut rng=Rng::new(seed);
    let mut states=(0usize..24).collect::<Vec<_>>();
    shuffle(&mut rng,&mut states);
    let a_states:[usize;5]=states[0..5].try_into().unwrap();
    let b_states:[usize;5]=states[5..10].try_into().unwrap();
    let c_states:[usize;6]=states[10..16].try_into().unwrap();

    let mut ma=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut ma);
    let mut mb=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mb);
    let mut mc=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mc);

    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_context_refinement());
    assert!(rt.enable_perceptual_refinement());
    assert!(rt.enable_compositional_refinement());
    assert!(rt.enable_unified_cognition(
        meta,
        PhaseHypothesisEcologyConfig{
            learning_rate:0.35,
            dormancy_threshold:0.05,
            learning_enabled:true,
            phase_learning_enabled:true,
        }
    ));
    let mut monitor=Monitor::default();

    let mut wa=WorldA::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    assert!(run_a(&mut rt,&l1,&mut monitor,&mut wa,48,0,None));
    rt.set_model_learning_enabled(false);
    let mut wa_reuse=WorldA::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    assert!(run_a(
        &mut rt,&l1,&mut monitor,&mut wa_reuse,6,4,Some(a_states[1])
    ));
    rt.set_model_learning_enabled(true);

    let mut wb=WorldB::new(b_states,[mb[0],mb[1],mb[2]]);
    assert!(run_b_goal(&mut rt,&l1,&mut monitor,&mut wb,false,64,1));
    assert!(run_b_goal(&mut rt,&l1,&mut monitor,&mut wb,true,12,2));

    let mut wc=WorldC::new(c_states,[mc[0],mc[1]],seed^0xC0FFEE);
    set_world(&mut rt,&l1,wc.state,c_states[4],0);
    let c_goal=foundation::scene(&l1,c_states[4],1);
    let c_base=rt.organism().phase_native_abstract_state(
        &foundation::scene(&l1,c_states[3],0)
    ).unwrap().cell;

    let recruited_before_c=rt.organism().recruited_relays();
    let dormant_capacity=rt.organism().config().dormant_cells;
    let mut junction_actions=[[0usize;6];2];
    let mut unsupported=None;
    let mut environment_resets=0usize;
    for k in 0..1800usize{
        if wc.trials>=72 && rt.organism().phase_native_context_witnesses()
            .iter().any(|w|w.promoted&&w.base_cell==c_base)
        {break;}

        if wc.state==c_states[4] || wc.state==c_states[5] {
            // Terminal transition is an environment episode boundary, not a
            // cognitive action. Preserve the organism; provide fresh factual
            // reset observation exactly as an external environment may do.
            wc.state=c_states[0];
            rt.observe_external(
                &foundation::scene(&l1,wc.state,(k+1)%6)
            ).unwrap();
            rt.set_goal(&c_goal).unwrap();
            environment_resets+=1;
            continue;
        }

        monitor.before(&rt,&c_goal);
        let layout=k%4;
        let at_junction=wc.state==c_states[3];
        let side=usize::from(wc.next_side==1);
        if at_junction && junction_actions.iter().flatten().sum::<usize>() < 12 {
            let proposals=rt.organism().collect_phase_native_unified_proposals(&c_goal);
            let selected=rt.organism().choose_phase_native_unified_proposal(&proposals);
            let mut context_probe=rt.organism().clone();
            let mut percept_probe=rt.organism().clone();
            let mut composition_probe=rt.organism().clone();
            let mut rival_probe=rt.organism().clone();
            let mut goal_probe=rt.organism().clone();
            let mut general_probe=rt.organism().clone();
            println!("INTEL2_C_DRIVE_WEIGHTS {:?}",rt.organism().phase_native_drive_weights());
            println!(
                "INTEL2_C_JUNCTION_TRACE trial={} side={} direct_unknown={:?} context={:?} percept={:?} composition={:?} rival={:?} goal={:?} general={:?} selected={:?} proposals={:?}",
                wc.trials,side,
                rt.organism().choose_phase_native_abstract_direct_action(),
                context_probe.phase_native_context_action(&c_goal),
                percept_probe.phase_native_perceptual_action(&c_goal),
                composition_probe.phase_native_compositional_action(&c_goal),
                rival_probe.choose_phase_native_goal_rival_probe(&c_goal),
                goal_probe.choose_phase_native_goal_active_action(&c_goal),
                general_probe.choose_phase_native_abstract_learned_drive_action(),
                selected,proposals
            );
        }
        if let Err(error)=step_u(&mut rt,&mut monitor,|a|{
            if at_junction { junction_actions[side][a]+=1; }
            let (next,value)=wc.step(a);
            (foundation::scene(&l1,next,layout),value)
        }){
            unsupported=Some((k,wc.state,error));
            break;
        }
    }

    let c_promoted=rt.organism().phase_native_context_witnesses()
        .iter().any(|w|w.promoted&&w.base_cell==c_base);
    println!(
        "INTEL2_C_RESET_DIAG trained_trials={} environment_resets={} promoted={} unsupported={:?} recruited_before_c={}/{} recruited_after_c={} junction_actions={:?} contexts={:?} u2={:?}",
        wc.trials,environment_resets,c_promoted,unsupported,
        recruited_before_c,dormant_capacity,rt.organism().recruited_relays(),
        junction_actions,
        rt.organism().phase_native_context_witnesses(),
        rt.organism().phase_native_hypothesis_records()
    );

    assert!(unsupported.is_none(),"corrected episodic C still lost action support");
    assert!(c_promoted,"corrected episodic C failed to promote context");

    rt.set_model_learning_enabled(false);
    let mut c_correct=0usize;
    let mut c_side=[0usize;2];
    let mut c_side_correct=[0usize;2];
    let mut memoryless=0usize;
    for i in 0..32usize{
        let side=i%2;
        let pred=c_states[1+side];
        rt.observe_external(&foundation::scene(&l1,pred,(4+i)%6)).unwrap();
        rt.set_goal(&c_goal).unwrap();
        monitor.before(&rt,&c_goal);
        let outcome=rt.step_unified(
            |_|Some(safe()),
            |_|Ok((foundation::scene(&l1,c_states[3],(4+i)%6),0.0))
        ).unwrap();
        assert!(matches!(outcome,StepOutcome::Executed{..}));
        let proposal=rt.propose_unified().unwrap().expect("context readout");
        c_side[side]+=1;
        if proposal.action==mc[side]{
            c_correct+=1;
            c_side_correct[side]+=1;
        }

        let current=foundation::scene(&l1,c_states[3],(4+i)%6);
        let mut old=rt.organism().clone();
        memoryless+=usize::from(
            old.plan_phase_native_abstract_goal(&current,&c_goal,None)
                .map(|d|d.first_action)==Some(mc[side])
        );
    }
    println!(
        "INTEL2_C_RESET_SCORE promoted={} score={}/32 sides={:?}/{:?} memoryless={}/32",
        c_promoted,c_correct,c_side_correct,c_side,memoryless
    );
    assert!(c_correct>=28);
    assert!(c_side_correct[0]>=13&&c_side_correct[1]>=13);
    assert!(memoryless<=20);
}
