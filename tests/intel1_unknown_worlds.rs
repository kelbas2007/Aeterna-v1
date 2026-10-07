use aeterna_v1::{EvoPhase, HumanProtectionEvidence, HumanProtectionReason};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use std::cell::Cell;

#[allow(dead_code)]
mod foundation {
    include!("g19_rival_hypothesis_discrimination.rs");

    pub fn build() -> (EvoPhase, [[usize;2];8]) {
        let drive=train_drive();
        let mut evo=target(&drive);
        let l1=train_abstraction(&mut evo);
        assert_eq!(evo.phase_native_circuits().len(),0);
        (evo,l1)
    }
    pub fn scene(l1:&[[usize;2];8],state:usize,layout:usize)->Vec<f32>{
        state_scene(l1,state,LAYOUTS[layout%LAYOUTS.len()])
    }
}

#[derive(Debug)]
struct Rng(u64);
impl Rng {
    fn new(seed:u64)->Self{Self(seed^0x1A7E_1100_2026_1007)}
    fn next(&mut self)->u64{
        let mut x=self.0;x^=x>>12;x^=x<<25;x^=x>>27;self.0=x;
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
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false
    }
}

fn step_runtime<F>(
    rt:&mut ScientificRuntime,
    mut exec:F,
)->usize where F:FnMut(usize)->Vec<f32>{
    let outcome=rt.step(|_|Some(safe()),|a|Ok(exec(a))).unwrap();
    match outcome{
        StepOutcome::Executed{proposal,..}=>proposal.action,
        other=>panic!("unexpected protected step: {:?}",other),
    }
}

#[derive(Debug)]
struct ChainWorld{
    state:usize,goal:usize,path:Vec<(usize,usize,usize)>,calls:usize,
}
impl ChainWorld{
    fn step(&mut self,action:usize)->usize{
        self.calls+=1;
        if let Some((_,_,to))=self.path.iter().find(|(from,a,_)|*from==self.state&&*a==action){
            self.state=*to;
        }
        self.state
    }
}

fn run_chain(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],world:&mut ChainWorld,
    budget:usize,layout_base:usize
)->bool{
    rt.observe_external(&foundation::scene(l1,world.state,layout_base)).unwrap();
    rt.set_goal(&foundation::scene(l1,world.goal,layout_base+1)).unwrap();
    for k in 0..budget{
        if world.state==world.goal{return true;}
        let layout=(layout_base+k)%6;
        step_runtime(rt,|a|{
            let next=world.step(a);
            foundation::scene(l1,next,layout)
        });
    }
    world.state==world.goal
}

fn protection_intervention(rt:&mut ScientificRuntime,l1:&[[usize;2];8],state:usize){
    rt.observe_external(&foundation::scene(l1,state,0)).unwrap();
    rt.set_goal(&foundation::scene(l1,7,1)).unwrap();
    let before=rt.organism().phase_native_learned_fingerprint();
    let frame=rt.organism().current_real().unwrap().clone();
    let calls=Cell::new(0);

    let high=rt.step(|_|Some(HumanProtectionEvidence{
        predicted_harm_probability:0.5,..safe()
    }),|_|{calls.set(calls.get()+1);Ok(foundation::scene(l1,state,0))}).unwrap();
    let StepOutcome::Blocked(r)=high else{panic!("high risk must block");};
    assert_eq!(r.reason,HumanProtectionReason::ExcessHumanHarmRisk);

    let missing=rt.step(|_|None,|_|{
        calls.set(calls.get()+1);Ok(foundation::scene(l1,state,0))
    }).unwrap();
    let StepOutcome::Blocked(r)=missing else{panic!("missing evidence must block");};
    assert_eq!(r.reason,HumanProtectionReason::InvalidEvidence);

    let emergency=rt.step(|_|Some(HumanProtectionEvidence{
        emergency_stop:true,..safe()
    }),|_|{calls.set(calls.get()+1);Ok(foundation::scene(l1,state,0))}).unwrap();
    assert!(matches!(emergency,StepOutcome::Blocked(_)));
    assert_eq!(calls.get(),0);
    assert_eq!(rt.organism().phase_native_learned_fingerprint(),before);
    let after=rt.organism().current_real().unwrap();
    assert_eq!(after.sensory,frame.sensory);
    assert_eq!(after.tick,frame.tick);

    rt.restart_cognition().unwrap();
    assert!(rt.emergency_latched());
    rt.observe_external(&foundation::scene(l1,state,0)).unwrap();
    let blocked=rt.step(|_|Some(safe()),|_|{
        calls.set(calls.get()+1);Ok(foundation::scene(l1,state,0))
    }).unwrap();
    assert!(matches!(blocked,StepOutcome::Blocked(_)));
    assert_eq!(calls.get(),0);
    rt.external_operator_reset();
}

#[derive(Debug)]
struct HistoryWorld{
    state:usize,context:bool,calls:usize,rng:Rng,
    actions:[usize;2],correct:usize,total:usize,score_correct:usize,ctx:[usize;4],
}
impl HistoryWorld{
    fn new(seed:u64,actions:[usize;2])->Self{
        Self{state:5,context:false,calls:0,rng:Rng::new(seed),actions,
            correct:0,total:0,score_correct:0,ctx:[0;4]}
    }
    fn step(&mut self,action:usize)->usize{
        self.calls+=1;
        self.state=match self.state{
            5=>{self.context=self.rng.range(2)==1;if self.context{2}else{1}},
            1|2=>0,
            0=>{
                let expected=self.actions[usize::from(self.context)];
                self.total+=1;
                if self.total>64{
                    self.ctx[usize::from(self.context)*2+1]+=1;
                    if action==expected{
                        self.score_correct+=1;
                        self.ctx[usize::from(self.context)*2]+=1;
                    }
                }
                if action==expected{7}else{8}
            }
            7|8=>5,
            _=>5,
        };
        self.state
    }
}

fn weak_scene(
    l1:&[[usize;2];8],state:usize,layout:usize,
    a:bool,b:bool,bins:[f32;2],
)->Vec<f32>{
    let mut s=foundation::scene(l1,state,layout);
    let pos=[399usize,398usize];
    assert!(s[pos[0]]<0.5&&s[pos[1]]<0.5);
    if a{s[pos[0]]=bins[0];}
    if b{s[pos[1]]=bins[1];}
    s
}

#[test]
fn intel1_frozen_core_unknown_world_lifetime(){
    let seed:u64=std::env::var("AETERNA_INTEL1_SEED")
        .expect("AETERNA_INTEL1_SEED").parse().unwrap();
    let mut rng=Rng::new(seed);
    let mut motors=[0usize,1,2,3,4,5];shuffle(&mut rng,&mut motors);

    println!("INTEL1_SEAL seed={} motors={:?} history_actions={:?} comp_op={} drift_change_after=4",
        seed,motors,[motors[4],motors[5]],if seed&1==0{"AND"}else{"XOR"});

    let (evo,l1)=foundation::build();
    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_context_refinement());
    assert!(rt.enable_perceptual_refinement());
    assert!(rt.enable_compositional_refinement());

    // WORLD 1: navigation.
    let mut w1=ChainWorld{
        state:0,goal:7,calls:0,
        path:vec![(0,motors[0],1),(1,motors[1],2),(2,motors[2],7)],
    };
    let w1_ok=run_chain(&mut rt,&l1,&mut w1,80,0);
    rt.set_model_learning_enabled(false);
    let mut w1_revisit=ChainWorld{state:0,goal:7,calls:0,path:w1.path.clone()};
    let w1_reuse=run_chain(&mut rt,&l1,&mut w1_revisit,20,4);
    rt.set_model_learning_enabled(true);
    println!("INTEL1_W1 first={} cost={} reuse={} reuse_cost={}",
        w1_ok,w1.calls,w1_reuse,w1_revisit.calls);

    // WORLD 2: causal machine / ordered intervention sequence.
    let mut w2=ChainWorld{
        state:3,goal:8,calls:0,
        path:vec![(3,motors[3],4),(4,motors[0],6),(6,motors[1],8)],
    };
    let w2_ok=run_chain(&mut rt,&l1,&mut w2,100,1);
    rt.set_model_learning_enabled(false);
    let mut w2_revisit=ChainWorld{state:3,goal:8,calls:0,path:w2.path.clone()};
    let w2_reuse=run_chain(&mut rt,&l1,&mut w2_revisit,20,5);
    rt.set_model_learning_enabled(true);
    println!("INTEL1_W2 first={} cost={} reuse={} reuse_cost={}",
        w2_ok,w2.calls,w2_reuse,w2_revisit.calls);

    protection_intervention(&mut rt,&l1,5);

    // WORLD 3: same junction image, different factual history.
    let mut hw=HistoryWorld::new(seed^0x33,[motors[4],motors[5]]);
    rt.observe_external(&foundation::scene(&l1,hw.state,0)).unwrap();
    rt.set_goal(&foundation::scene(&l1,7,1)).unwrap();
    for k in 0..900usize{
        let layout=k%4;
        step_runtime(&mut rt,|a|{
            let next=hw.step(a);
            foundation::scene(&l1,next,layout)
        });
        if hw.total>=96{break;}
    }
    let witnesses=rt.organism().phase_native_context_witnesses();
    let promoted=witnesses.iter().any(|w|w.promoted);
    let scored=hw.ctx[1]+hw.ctx[3];
    println!("INTEL1_W3 promoted={} score={}/{} ctx={:?} witnesses={:?}",
        promoted,hw.score_correct,scored,hw.ctx,witnesses);

    // Protected restart is required regardless of W3 result.
    let fp=rt.organism().phase_native_learned_fingerprint();
    rt.restart_cognition().unwrap();
    assert_eq!(rt.organism().phase_native_learned_fingerprint(),fp);
    rt.observe_external(&foundation::scene(&l1,5,0)).unwrap();

    // WORLD 4: current-sensory AND/XOR composition.
    let op_xor=seed&1==1;
    let bins=[0.20f32,0.40f32];
    let cx=motors[0];let cy=motors[1];
    let mut comp_correct=0usize;let mut comp_scored=0usize;
    rt.set_goal(&foundation::scene(&l1,7,2)).unwrap();
    for i in 0..420usize{
        let combo=i%4;let a=combo&2!=0;let b=combo&1!=0;
        let current=weak_scene(&l1,3,i%4,a,b,bins);
        rt.observe_external(&current).unwrap();
        let expected_x=if op_xor{a^b}else{a&&b};
        let outcome=rt.step(|_|Some(safe()),|act|{
            let good=(act==cx)==expected_x;
            Ok(foundation::scene(&l1,if good{7}else{8},i%4))
        }).unwrap();
        if i>=388{
            comp_scored+=1;
            if let StepOutcome::Executed{proposal,..}=outcome{
                let expected=if expected_x{cx}else{cy};
                comp_correct+=usize::from(proposal.action==expected);
            }
        }
    }
    let comps=rt.organism().phase_native_composition_witnesses();
    let comp_promoted=comps.iter().any(|w|w.promoted);
    println!("INTEL1_W4 promoted={} score={}/{} witnesses={}",
        comp_promoted,comp_correct,comp_scored,comps.len());

    // WORLD 5: one useful shortcut changes after four factual successes.
    rt.set_model_learning_enabled(true);
    let mut drift_state=5usize;
    let shortcut=motors[2];let step=motors[3];let fallback=motors[4];let finish=motors[5];
    let mut successes=0usize;let mut changed=false;let mut post_change_first=None;
    rt.observe_external(&foundation::scene(&l1,drift_state,0)).unwrap();
    rt.set_goal(&foundation::scene(&l1,7,1)).unwrap();
    let mut since_change=0usize;
    for i in 0..240usize{
        let before=drift_state;
        step_runtime(&mut rt,|act|{
            drift_state=match before{
                5 if act==shortcut=>if changed{8}else{6},
                5 if act==fallback=>4,
                6 if act==step=>7,
                4 if act==finish=>7,
                7|8=>5,
                _=>before,
            };
            foundation::scene(&l1,drift_state,i%6)
        });
        if changed{since_change+=1;}
        if drift_state==7{
            successes+=1;
            if successes==4&&!changed{changed=true;since_change=0;}
            else if changed&&post_change_first.is_none(){post_change_first=Some(since_change);}
        }
        if changed&&successes>=8{break;}
    }
    println!("INTEL1_W5 successes={} changed={} first_repair={:?}",
        successes,changed,post_change_first);

    // Retention revisits after all worlds.
    rt.set_model_learning_enabled(false);
    let mut r1=ChainWorld{state:0,goal:7,calls:0,path:w1.path.clone()};
    let retain1=run_chain(&mut rt,&l1,&mut r1,20,4);
    let mut r2=ChainWorld{state:3,goal:8,calls:0,path:w2.path.clone()};
    let retain2=run_chain(&mut rt,&l1,&mut r2,20,5);

    let w3_pass=promoted&&scored>=32&&hw.score_correct>=28
        && hw.ctx[0]>=13&&hw.ctx[2]>=13;
    let w4_pass=comp_promoted&&comp_scored==32&&comp_correct>=28;
    let w5_pass=changed&&successes>=8&&post_change_first.map(|x|x<=16).unwrap_or(false);

    println!(
        "INTEL1_RESULT W1={} W2={} W3={} W4={} W5={} retention={}/2 safety=true legacy={} table={} verdict={}",
        w1_ok&&w1_reuse,w2_ok&&w2_reuse,w3_pass,w4_pass,w5_pass,
        usize::from(retain1)+usize::from(retain2),
        rt.organism().planning_transition_count(),
        rt.organism().composite_concepts().len(),
        if w1_ok&&w1_reuse&&w2_ok&&w2_reuse&&w3_pass&&w4_pass&&w5_pass&&retain1&&retain2
            {"PASS_AUTONOMOUS_DEVELOPING_INTELLIGENCE"}else{"FAIL"}
    );

    assert!(w1_ok&&w1_reuse,"INTEL-1 W1 failure");
    assert!(w2_ok&&w2_reuse,"INTEL-1 W2 failure");
    assert!(w3_pass,"INTEL-1 earliest causal bottleneck: history-dependent world");
    assert!(w4_pass,"INTEL-1 W4 composition failure");
    assert!(w5_pass,"INTEL-1 W5 revision failure");
    assert!(retain1&&retain2,"INTEL-1 retention failure");
    assert_eq!(rt.organism().planning_transition_count(),0);
    assert!(rt.organism().composite_concepts().is_empty());
}
