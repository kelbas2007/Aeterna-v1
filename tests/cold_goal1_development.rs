#![allow(dead_code)]
// COLD-GOAL-1 open diagnostic. No PRE/motor/POST tuition is provided.
// All goal-relevant physical links must be created from protected,
// organism-selected factual action consequences in the SAME persistent carrier.
// An experimenter knows world transition truth but never passes role IDs,
// graph edges or a teaching schedule into EvoPhase.
include!("intel2_unified_worlds.rs");
#[derive(Clone)]
struct CausalWorld {
    states:[usize;6],
    motors:[usize;6],
}
impl CausalWorld {
    fn factual(&self,l1:&[[usize;2];8],state:usize,layout:usize)->Vec<f32>{
        foundation::scene(l1,self.states[state],layout)
    }
    fn transition(&self,from:usize,motor:usize,drift:bool)->usize{
        let m=self.motors;
        match (from,motor) {
            (0,a) if a==m[0]=>1,
            (1,a) if a==m[1]&&!drift=>2,
            (2,a) if a==m[2]=>3,
            (3,a) if a==m[3]=>5,
            (1,a) if a==m[4]=>4,
            (4,a) if a==m[5]=>2,
            _=>from
        }
    }
}

fn episode(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],w:&CausalWorld,
    drift:bool,max_steps:usize
)->(bool,Vec<usize>,usize) {
    let mut current=0usize;
    rt.observe_external(&w.factual(l1,0,0)).unwrap();
    rt.set_goal(&w.factual(l1,5,2)).unwrap();
    let mut actions=Vec::new();
    let mut surprises=0usize;
    for step in 0..max_steps{
        if current==5 {break;}
        let planned=rt.organism().choose_phase_native_temporal_goal_plan(
            &w.factual(l1,5,2)
        );
        let prior=current;
        let mut taken=None;
        let outcome=rt.step_unified(
            |_|Some(safe()),
            |motor|{
                taken=Some(motor);
                current=w.transition(current,motor,drift);
                Ok((w.factual(l1,current,(step+1)%6),
                    if current==5{1.0}else{0.0}))
            }
        ).unwrap();
        match outcome {
            StepOutcome::Executed{..}=>{
                let motor=taken.expect("protected actuator");
                actions.push(motor);
                if let Some(plan)=planned {
                    if plan.action==motor && prior==current {surprises+=1;}
                }
                println!("GOAL_REPLAN_STEP drift={} step={} prior={} motor={} next={} native={:?}",
                    drift,step,prior,motor,current,planned);
            }
            other=>panic!("supported physical goal path must execute, got {:?}",other)
        }
    }
    (current==5,actions,surprises)
}


#[derive(Default)]
struct ColdGoalStats{
    actions:usize,
    goals:usize,
    unavailable:usize,
    blocked:usize,
    exploratory:usize,
    observed_new_states:usize,
    proposals:usize,
}
fn run_self_directed(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],
    world:&CausalWorld,ordinal:usize,limit:usize,metrics:&mut ColdGoalStats
)->bool {
    let goal=world.factual(l1,5,(ordinal+3)%6);
    rt.observe_external(&world.factual(l1,0,ordinal%6)).unwrap();
    rt.set_goal(&goal).unwrap();
    let mut state=0usize;
    for step in 0..limit{
        if state==5 {break;}
        let proposal=rt.organism().choose_phase_native_temporal_goal_frontier();
        let planned=rt.organism().choose_phase_native_temporal_goal_plan(&goal);
        let before=state;
        const DRIFT:bool=false;
        let execution=rt.step_unified(
            |_|Some(safe()),
            |action|{
                state=world.transition(state,action,DRIFT);
                Ok((world.factual(l1,state,(ordinal+step+1)%6),
                    if state==5{1.0}else{0.0}))
            }
        );
        match execution {
            Ok(StepOutcome::Executed{proposal:chosen,..})=>{
                metrics.actions+=1;
                metrics.exploratory+=usize::from(
                    proposal.is_some_and(|p|p.0==chosen.action)
                );
                metrics.proposals+=usize::from(planned.is_some());
                metrics.observed_new_states+=usize::from(state!=before);
                if ordinal<2 {
                    println!("COLD_GOAL_EARLY ep={} step={} prev={} action={} after={} frontier={:?} path={:?}",
                        ordinal,step,before,chosen.action,state,proposal,planned);
                }
            }
            Ok(StepOutcome::GoalReached)=>break,
            Ok(StepOutcome::Blocked(_))=>{metrics.blocked+=1;break;}
            _=>{metrics.unavailable+=1;break;}
        }
    }
    if state==5{metrics.goals+=1;}
    state==5
}

#[test]
fn cold_goal1_unguided_causal_model_to_goal_and_drift_development(){
    const SEED:u64=0xC01D_2026_6E7A_0001;
    let mut rng=Rng::new(SEED);
    let mut all_good=0usize;
    for arm in 0..4usize {
        let (evo,l1)=foundation::build24();
        let mut states=(0..24usize).collect::<Vec<_>>();
        let mut motors=(0..6usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut states);
        shuffle(&mut rng,&mut motors);
        let w=CausalWorld{
            states:states[..6].try_into().unwrap(),
            motors:motors.try_into().unwrap(),
        };
        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_temporal_evidence(
            aeterna_v1::carrier::PhaseTemporalEvidenceConfig{
                max_observations:8,minimum_observations:3,decisive_margin:0.125
            }
        ));
        assert!(rt.set_temporal_multistep(true));
        assert!(rt.set_temporal_goal_replanning(true));
        assert!(rt.set_temporal_cold_goal_acquisition(true));
        assert!(rt.enable_unified_cognition(
            meta_checkpoint(),
            PhaseHypothesisEcologyConfig{
                learning_rate:0.35,dormancy_threshold:0.05,
                learning_enabled:true,phase_learning_enabled:true
            }
        ));
        let mut train=ColdGoalStats::default();
        for ep in 0..192{
            run_self_directed(&mut rt,&l1,&w,ep,16,&mut train);
        }
        let acquired=rt.organism().phase_native_temporal_transition_count();
        let goal=w.factual(&l1,5,0);
        rt.observe_external(&w.factual(&l1,0,1)).unwrap();
        rt.set_goal(&goal).unwrap();
        let initial_route=rt.organism().choose_phase_native_temporal_goal_plan(&goal);
        assert!(rt.restart_cognition().is_ok());
        rt.set_model_learning_enabled(false);
        let mut held=ColdGoalStats::default();
        let intact=run_self_directed(&mut rt,&l1,&w,999+arm,12,&mut held);
        rt.set_model_learning_enabled(true);
        let (repaired,actual,surprises)=episode(&mut rt,&l1,&w,true,12);
        let pass=acquired>=6 && intact && repaired
            && surprises>=1 && actual.contains(&w.motors[4])
            &&actual.contains(&w.motors[5])
            &&train.blocked==0&&train.unavailable==0
            &&held.blocked==0&&held.unavailable==0;
        all_good+=usize::from(pass);
        println!("COLD_GOAL_ARM arm={} motors={:?} states={:?} acquired_edges={} train_goals={}/192 train_actions={} exploratory={} new_states={} train_blocked={} train_unavailable={} first_plan={:?} frozen_goal={} frozen_actions={} repaired={} surprises={} detour={:?} pass={}",
            arm,w.motors,w.states,acquired,train.goals,train.actions,
            train.exploratory,train.observed_new_states,
            train.blocked,train.unavailable,initial_route,
            intact,held.actions,repaired,surprises,actual,pass);
    }
    println!("COLD_GOAL_SUMMARY passed={}/4 verdict={}",
        all_good,if all_good==4{"DEVELOPMENT_PASS"}else{"DEVELOPMENT_FAIL"});
    // The development test deliberately retains FAIL records without
    // labelling green CI a scientific qualification.
}
