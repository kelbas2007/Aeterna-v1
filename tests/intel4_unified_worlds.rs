#![allow(dead_code)]

include!("intel2_unified_worlds.rs");

#[derive(Clone,Debug)]
struct WorldA3{
    states:[usize;5], // start,p1,p2,goal,lure
    motors:[usize;5], // enter,advance,finish,lure,escape
    state:usize,
    calls:usize,
}
impl WorldA3{
    fn new(states:[usize;5],motors:[usize;5])->Self{
        Self{state:states[0],states,motors,calls:0}
    }
    fn step(&mut self,action:usize)->(usize,f32){
        self.calls+=1;
        let [s,p1,p2,g,d]=self.states;
        let [enter,advance,finish,lure,escape]=self.motors;
        self.state=match self.state{
            x if x==s&&action==enter=>p1,
            x if x==p1&&action==advance=>p2,
            x if x==p2&&action==finish=>g,
            x if x==p1&&action==lure=>d,
            x if x==d&&action==escape=>p1,
            x=>x,
        };
        (self.state,f32::from(self.state==g))
    }
}

fn run_a3(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],monitor:&mut Monitor,
    world:&mut WorldA3,budget:usize,layout:usize,start_override:Option<usize>,
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
struct WorldB3{
    states:[usize;5], // start,tool,armed,goal1,goal2
    motors:[usize;5], // acquire,arm,use1,use2,return
    state:usize,
    calls:usize,
}
impl WorldB3{
    fn new(states:[usize;5],motors:[usize;5])->Self{
        Self{state:states[0],states,motors,calls:0}
    }
    fn step(&mut self,action:usize,goal2:bool)->(usize,f32){
        self.calls+=1;
        let [s,t,a,g1,g2]=self.states;
        let [acq,arm,use1,use2,ret]=self.motors;
        self.state=match self.state{
            x if x==s&&action==acq=>t,
            x if x==t&&action==arm=>a,
            x if x==s&&(action==use1||action==use2)=>s,
            x if x==t&&(action==use1||action==use2)=>s,
            x if x==a&&action==use1=>g1,
            x if x==a&&action==use2=>g2,
            x if (x==g1||x==g2)&&action==ret=>a,
            x=>x,
        };
        let target=if goal2{g2}else{g1};
        (self.state,f32::from(self.state==target))
    }
}

fn run_b3_goal(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],monitor:&mut Monitor,
    world:&mut WorldB3,goal2:bool,budget:usize,layout:usize,
)->bool{
    let target=if goal2{world.states[4]}else{world.states[3]};
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

#[derive(Clone,Debug)]
struct WorldE3{
    states:[usize;5], // start,fast_mid,slow_mid,goal,dead
    motors:[usize;4], // fast,fast_finish,slow,slow_finish
    state:usize,
    fast_successes:usize,
    changed:bool,
    changed_seen:bool,
    after_change_actions:usize,
    first_recovery:Option<usize>,
    slow_after_change:bool,
}
impl WorldE3{
    fn new(states:[usize;5],motors:[usize;4])->Self{
        Self{
            state:states[0],states,motors,
            fast_successes:0,changed:false,changed_seen:false,
            after_change_actions:0,first_recovery:None,slow_after_change:false,
        }
    }
    fn step(&mut self,action:usize)->(usize,f32){
        let [s,fm,sm,g,d]=self.states;
        let [fast,ff,slow,sf]=self.motors;
        if self.changed_seen{self.after_change_actions+=1;}
        let before=self.state;
        self.state=match self.state{
            x if x==s&&action==fast=>if self.changed{d}else{fm},
            x if x==fm&&action==ff=>g,
            x if x==s&&action==slow=>sm,
            x if x==sm&&action==sf=>g,
            x if x==g||x==d=>s,
            x=>x,
        };
        if before==s&&action==fast&&self.changed{
            self.changed_seen=true;
            self.after_change_actions=0;
        }
        if before==fm&&action==ff&&self.state==g&&!self.changed{
            self.fast_successes+=1;
            if self.fast_successes>=4{self.changed=true;}
        }
        if self.changed_seen&&before==sm&&action==sf&&self.state==g{
            self.slow_after_change=true;
        }
        if self.changed_seen&&self.state==g&&self.first_recovery.is_none(){
            self.first_recovery=Some(self.after_change_actions);
        }
        (self.state,f32::from(self.state==g))
    }
}

#[test]
fn intel4_frozen_unified_fresh_lifetime(){
    let seed:u64=std::env::var("AETERNA_INTEL4_SEED")
        .expect("AETERNA_INTEL4_SEED").parse().unwrap();
    let source_sha=std::env::var("AETERNA_SOURCE_SHA").unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA").unwrap_or_else(|_|"unknown".into());

    // Generic pre-target substrate and transferred meta-control only.
    let (evo,l1)=foundation::build24();
    let meta=meta_checkpoint();

    let mut rng=Rng::new(seed);
    let mut states=(0usize..24).collect::<Vec<_>>();
    shuffle(&mut rng,&mut states);
    let a_states:[usize;5]=states[0..5].try_into().unwrap();
    let b_states:[usize;5]=states[5..10].try_into().unwrap();
    let c_states:[usize;6]=states[10..16].try_into().unwrap();
    let d_states:[usize;3]=states[16..19].try_into().unwrap();
    let e_states:[usize;5]=states[19..24].try_into().unwrap();

    let mut ma=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut ma);
    let mut mb=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mb);
    let mut mc=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut mc);
    let mut md=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut md);
    let mut me=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut me);
    let op_xor=(rng.next()&1)==1;
    let bins=if rng.next()&1==0{[0.18f32,0.38f32]}else{[0.16f32,0.36f32]};

    println!(
        "INTEL4_SEAL source_sha={} spec_sha={} seed={} A={:?}/{:?} B={:?}/{:?} C={:?}/{:?} D={:?}/{:?}/op={} bins={:?} E={:?}/{:?}",
        source_sha,spec_sha,seed,
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]],
        b_states,[mb[0],mb[1],mb[2],mb[3],mb[4]],
        c_states,[mc[0],mc[1]],
        d_states,[md[0],md[1]],if op_xor{"XOR"}else{"AND"},bins,
        e_states,[me[0],me[1],me[2],me[3]]
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

    // WORLD A — lure from an intermediate state.
    let mut wa=WorldA3::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    let a_ok=run_a3(&mut rt,&l1,&mut monitor,&mut wa,56,0,None);
    let a_cost=wa.calls;
    rt.set_model_learning_enabled(false);
    let mut wa_reuse=WorldA3::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    let a_reuse=run_a3(
        &mut rt,&l1,&mut monitor,&mut wa_reuse,7,4,Some(a_states[1])
    );
    let a_reuse_cost=wa_reuse.calls;
    rt.set_model_learning_enabled(true);
    println!(
        "INTEL4_A first={} cost={} reuse={} reuse_cost={}",
        a_ok,a_cost,a_reuse,a_reuse_cost
    );

    // WORLD B — acquire -> arm -> terminal, then goal switch.
    let mut wb=WorldB3::new(
        b_states,[mb[0],mb[1],mb[2],mb[3],mb[4]]
    );
    let b1=run_b3_goal(&mut rt,&l1,&mut monitor,&mut wb,false,72,1);
    let b1_cost=wb.calls;
    let before_switch=wb.calls;
    let b2=run_b3_goal(&mut rt,&l1,&mut monitor,&mut wb,true,16,2);
    let b2_cost=wb.calls-before_switch;
    println!(
        "INTEL4_B first={} cost={} switched={} switch_cost={}",
        b1,b1_cost,b2,b2_cost
    );

    // WORLD C — new aliased-history pack.
    let promoted_before=rt.organism().phase_native_context_witnesses()
        .iter().filter(|w|w.promoted).count();
    let mut wc=WorldC::new(c_states,[mc[0],mc[1]],seed^0xC300_C0DE);
    set_world(&mut rt,&l1,wc.state,c_states[4],0);
    let c_goal=foundation::scene(&l1,c_states[4],1);
    let c_base=rt.organism().phase_native_abstract_state(
        &foundation::scene(&l1,c_states[3],0)
    ).unwrap().cell;
    let mut c_unsupported=None;
    let mut c_resets=0usize;
    for k in 0..2400usize{
        if wc.trials>=96 && rt.organism().phase_native_context_witnesses()
            .iter().any(|w|w.promoted&&w.base_cell==c_base)
        {break;}

        if wc.state==c_states[4]||wc.state==c_states[5]{
            wc.state=c_states[0];
            rt.observe_external(&foundation::scene(&l1,wc.state,(k+2)%6)).unwrap();
            rt.set_goal(&c_goal).unwrap();
            c_resets+=1;
            continue;
        }

        monitor.before(&rt,&c_goal);
        let layout=k%6;
        if let Err(error)=step_u(&mut rt,&mut monitor,|a|{
            let (next,value)=wc.step(a);
            (foundation::scene(&l1,next,layout),value)
        }){
            c_unsupported=Some(error);
            break;
        }
    }
    let c_promoted=rt.organism().phase_native_context_witnesses()
        .iter().any(|w|w.promoted&&w.base_cell==c_base);

    rt.set_model_learning_enabled(false);
    let mut c_correct=0usize;
    let mut c_side=[0usize;2];
    let mut c_side_correct=[0usize;2];
    let mut memoryless=0usize;
    for i in 0..40usize{
        let side=i%2;
        let pred=c_states[1+side];
        rt.observe_external(&foundation::scene(&l1,pred,(i+3)%6)).unwrap();
        rt.set_goal(&c_goal).unwrap();
        let outcome=rt.step_unified(
            |_|Some(safe()),
            |_|Ok((foundation::scene(&l1,c_states[3],(i+4)%6),0.0))
        ).unwrap();
        assert!(matches!(outcome,StepOutcome::Executed{..}));
        let proposal=rt.propose_unified().unwrap().expect("INTEL-4 C readout");
        c_side[side]+=1;
        if proposal.action==mc[side]{
            c_correct+=1;
            c_side_correct[side]+=1;
        }

        let current=foundation::scene(&l1,c_states[3],(i+4)%6);
        let mut old=rt.organism().clone();
        memoryless+=usize::from(
            old.plan_phase_native_abstract_goal(&current,&c_goal,None)
                .map(|d|d.first_action)==Some(mc[side])
        );
    }
    rt.set_model_learning_enabled(true);
    println!(
        "INTEL4_C trials={} resets={} promoted={} unsupported={:?} full={}/40 sides={:?}/{:?} memoryless={}/40",
        wc.trials,c_resets,c_promoted,c_unsupported,c_correct,
        c_side_correct,c_side,memoryless
    );

    // Protected cognitive restart after C.
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

    // WORLD D — fresh marker locations / operator.
    let x=md[0];let y=md[1];
    let train_positions=[[391usize,390usize],[389,388]];
    let score_positions=[[387usize,386usize],[385,384]];
    let mut schedule=Vec::new();
    let counts=if op_xor{[48usize,8,8,16]}else{[24usize,24,24,8]};
    for (combo,count) in counts.into_iter().enumerate(){
        for _ in 0..count{schedule.push(combo);}
    }
    shuffle(&mut rng,&mut schedule);
    rt.set_goal(&d_goal).unwrap();
    let d_base_cell=rt.organism().phase_native_abstract_state(&d_base).unwrap().cell;
    for i in 0..1400usize{
        // Factual learning continues after representation promotion:
        // a composed predicate must still acquire each required motor.
        // No label or action tutor is supplied.
        let combo=schedule[i%schedule.len()];
        let a=combo&2!=0;let b=combo&1!=0;
        let current=weak_scene(
            &l1,d_states[0],i%6,a,b,bins,train_positions[i%2]
        );
        rt.observe_external(&current).unwrap();
        rt.set_goal(&d_goal).unwrap();
        let expected_x=if op_xor{a^b}else{a&&b};
        let outcome=rt.step_unified(
            |_|Some(safe()),
            |act|{
                let correct=if expected_x {act==x} else {act==y};
                Ok((
                    foundation::scene(
                        &l1,if correct{d_states[1]}else{d_states[2]},i%6
                    ),
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
    for i in 0..40usize{
        let combo=i%4;let a=combo&2!=0;let b=combo&1!=0;
        let current=weak_scene(
            &l1,d_states[0],4+i%2,a,b,bins,score_positions[i%2]
        );
        rt.observe_external(&current).unwrap();
        rt.set_goal(&d_goal).unwrap();
        let p=rt.propose_unified().unwrap().expect("INTEL-4 D readout");
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
    println!(
        "INTEL4_D op={} promoted={} full={}/40 best_single={}/40",
        if op_xor{"XOR"}else{"AND"},d_promoted,d_correct,best_single
    );

    // WORLD E — fast route changes; slow route stays factual.
    let mut we=WorldE3::new(
        e_states,[me[0],me[1],me[2],me[3]]
    );
    set_world(&mut rt,&l1,we.state,e_states[3],2);
    let e_goal=foundation::scene(&l1,e_states[3],3);
    let mut e_unsupported=None;
    let mut e_resets=0usize;
    let mut first_changed_model=None;
    let mut e_actions=0usize;
    for k in 0..1600usize{
        if we.changed_seen&&we.first_recovery.is_some(){break;}
        if we.state==e_states[3] || we.state==e_states[4] {
            // Only the external environment starts another episode. No
            // fabricated action or learned terminal->reset law is inserted.
            we.state=e_states[0];
            rt.observe_external(&foundation::scene(&l1,we.state,(k+1)%6)).unwrap();
            rt.set_goal(&e_goal).unwrap();
            e_resets+=1;
            continue;
        }
        let from=we.state;
        let already_changed=we.changed;
        let before_revisions=rt.organism().phase_native_circuits().iter()
            .map(|c|(c.revision,c.counterexamples.len() as u64))
            .fold((0u64,0u64),|acc,x|(acc.0+x.0,acc.1+x.1));
        monitor.before(&rt,&e_goal);
        let layout=k%6;
        if let Err(error)=step_u(&mut rt,&mut monitor,|a|{
            if already_changed && from==e_states[0] && a==me[0]
                && first_changed_model.is_none()
            {
                first_changed_model=Some(before_revisions);
            }
            e_actions+=1;
            let (next,value)=we.step(a);
            (foundation::scene(&l1,next,layout),value)
        }){
            e_unsupported=Some(error);
            break;
        }
    }
    let after_revisions=rt.organism().phase_native_circuits().iter()
        .map(|c|(c.revision,c.counterexamples.len() as u64))
        .fold((0u64,0u64),|acc,x|(acc.0+x.0,acc.1+x.1));
    let e_revision=first_changed_model
        .map(|before|after_revisions.0>before.0
            && after_revisions.1>before.1)
        .unwrap_or(false);
    println!(
        "INTEL4_E fast_successes={} changed={} seen={} recovery={:?} slow_after_change={} revision={} actions={} resets={} unsupported={:?}",
        we.fast_successes,we.changed,we.changed_seen,we.first_recovery,
        we.slow_after_change,e_revision,e_actions,e_resets,e_unsupported
    );

    // Final frozen retention.
    rt.set_model_learning_enabled(false);
    let mut fa=WorldA3::new(
        a_states,[ma[0],ma[1],ma[2],ma[3],ma[4]]
    );
    let final_a=run_a3(&mut rt,&l1,&mut monitor,&mut fa,7,5,None);
    let final_a_cost=fa.calls;
    let mut fb=WorldB3::new(
        b_states,[mb[0],mb[1],mb[2],mb[3],mb[4]]
    );
    let final_b=run_b3_goal(&mut rt,&l1,&mut monitor,&mut fb,false,10,4);
    let final_b_cost=fb.calls;

    let meta_unchanged=rt.organism().phase_native_meta_weights().unwrap()==meta_weights;
    let legacy=rt.organism().planning_transition_count();
    let table=rt.organism().composite_concepts().len();

    let c_pass=c_unsupported.is_none()
        &&c_promoted&&c_correct>=36
        &&c_side_correct[0]>=18&&c_side_correct[1]>=18
        &&memoryless<=24;
    let d_single_limit=if op_xor{20}else{30};
    let d_pass=d_promoted&&d_correct>=36&&best_single<=d_single_limit;
    let e_pass=we.changed&&we.changed_seen
        &&we.first_recovery.map(|x|x<=18).unwrap_or(false)
        &&we.slow_after_change&&e_revision&&e_unsupported.is_none();

    let verdict=
        a_ok&&a_cost<=56&&a_reuse&&a_reuse_cost<=7
        &&b1&&b1_cost<=72&&b2&&b2_cost<=16
        &&c_pass&&d_pass&&e_pass
        &&final_a&&final_a_cost<=7
        &&final_b&&final_b_cost<=10
        &&meta_unchanged&&legacy==0&&table==0;

    println!(
        "INTEL4_RESULT A={}/{} B={}/{} C={} D={} E={} final={}/2 safety=true meta_unchanged={} proposals={} max_u2={} dormant={} zero_app={} reactivated={} legacy={} table={} verdict={}",
        a_ok,a_reuse,b1,b2,c_pass,d_pass,e_pass,
        usize::from(final_a)+usize::from(final_b),
        meta_unchanged,monitor.proposal_count,monitor.max_candidates,
        monitor.saw_dormant,monitor.saw_zero_applicability,monitor.saw_reactivation,
        legacy,table,
        if verdict{"PASS_AUTONOMOUS_DEVELOPING_INTELLIGENCE_UNIFIED_V2"}else{"FAIL"}
    );

    assert!(a_ok&&a_cost<=56,"INTEL-4 World A acquisition failure");
    assert!(a_reuse&&a_reuse_cost<=7,"INTEL-4 World A reuse failure");
    assert!(b1&&b1_cost<=72,"INTEL-4 World B first-goal failure");
    assert!(b2&&b2_cost<=16,"INTEL-4 World B goal-switch failure");
    assert!(c_pass,"INTEL-4 World C history-dependent failure");
    assert!(d_pass,"INTEL-4 World D composition failure");
    assert!(e_pass,"INTEL-4 World E revision failure");
    assert!(final_a&&final_a_cost<=7,"INTEL-4 final A retention failure");
    assert!(final_b&&final_b_cost<=10,"INTEL-4 final B retention failure");
    assert!(meta_unchanged,"INTEL-4 U1 meta weights changed");
    assert_eq!(legacy,0);
    assert_eq!(table,0);
}

#[test]
fn intel4_evaluator_uses_only_unified_external_runtime(){
    let source=include_str!("intel4_unified_worlds.rs");
    assert!(source.contains("step_unified"));

    // Assemble forbidden evaluator tokens at runtime so the source guard does
    // not match its own string literals.
    let forbidden=[
        ["rt",".step("].concat(),
        ["Reasoning","Mode::"].concat(),
        ["world","_id"].concat(),
        ["task","_id"].concat(),
        ["correct","_action"].concat(),
    ];
    for token in forbidden {
        assert!(!source.contains(&token),"forbidden evaluator token: {token}");
    }
}
