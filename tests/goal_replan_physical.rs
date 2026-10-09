#![allow(dead_code)]
// GOAL-REPLAN-1 open integration test. A previously acquired physical
// state/action graph is reassembled into an untaught composite route.
// The trusted world reports ONLY factual PRE/POST and bounded real reward.
// Actual goal action selection goes through protected step_unified, not here.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[derive(Clone)]
struct CausalWorld {
    states:[usize;6],  // start, gate, primary, join, detour, goal
    motors:[usize;6],  // S->A, A->B, B->C, C->G, A->D, D->B
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
    fn demonstrate_components(
        &self,evo:&mut EvoPhase,l1:&[[usize;2];8]
    ){
        // A factual collection of EDGE experiences, never a guided full
        // demonstration of the goal route or of the composed detour.
        for (from,motor,to) in [
            (0,self.motors[0],1),(1,self.motors[1],2),
            (2,self.motors[2],3),(3,self.motors[3],5),
            (1,self.motors[4],4),(4,self.motors[5],2)
        ] {
            assert!(evo.observe_phase_native_sensing_affordance(
                motor,&self.factual(l1,from,0),&self.factual(l1,to,1)
            ));
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

#[test]
fn goal_replan_composes_physical_routes_after_unannounced_causal_drift(){
    let mut rng=Rng::new(0x6A17_2026_00AB_1101);
    let mut intact=0usize;
    let mut repaired=0usize;
    for arm in 0..4 {
        let (mut evo,l1)=foundation::build24();
        assert!(evo.enable_phase_native_temporal_evidence(
            PhaseTemporalEvidenceConfig{
                max_observations:8,minimum_observations:3,
                decisive_margin:0.125
            }
        ));
        assert!(evo.set_phase_native_temporal_multistep(true));
        assert!(evo.set_phase_native_temporal_goal_replanning(true));
        let mut states=(0..24usize).collect::<Vec<_>>();
        let mut motors=(0..6usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut states);
        shuffle(&mut rng,&mut motors);
        let w=CausalWorld{
            states:states[..6].try_into().unwrap(),
            motors:motors.try_into().unwrap(),
        };
        w.demonstrate_components(&mut evo,&l1);
        assert_eq!(evo.phase_native_temporal_transition_count(),6);
        let checkpoint=evo.phase_native_checkpoint().unwrap();
        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_unified_cognition(
            meta_checkpoint(),
            PhaseHypothesisEcologyConfig {
                learning_rate:0.35,dormancy_threshold:0.05,
                learning_enabled:true,phase_learning_enabled:true
            }
        ));
        rt.observe_external(&w.factual(&l1,0,0)).unwrap();
        let first=rt.organism().choose_phase_native_temporal_goal_plan(
            &w.factual(&l1,5,2)
        ).unwrap();
        assert_eq!(first.action,w.motors[0]);
        assert_eq!(first.steps,4);
        let mut damaged=rt.organism().clone();
        let saved=damaged.perturb_phase_native_synapse_for_control(
            first.synapse,0.0,0.0).unwrap();
        assert!(damaged.choose_phase_native_temporal_goal_plan(
            &w.factual(&l1,5,2)
        ).is_none(),"causal lesion must destroy every path from start");
        damaged.restore_phase_native_synapse_for_control(first.synapse,saved);
        assert_eq!(damaged.choose_phase_native_temporal_goal_plan(
            &w.factual(&l1,5,2)
        ).unwrap(),first);
        let (pass,primary,_)=episode(&mut rt,&l1,&w,false,8);
        intact+=usize::from(pass);
        // Same organism, new episode, no external law-change flag.
        let (recovered,changed,surprises)=episode(&mut rt,&l1,&w,true,10);
        repaired+=usize::from(recovered);
        assert!(rt.organism().phase_native_temporal_goal_replanning_enabled());
        assert!(rt.organism().phase_native_temporal_transition_count()>=6);
        let weight_snapshot=rt.organism().phase_native_meta_weights();
        rt.restart_cognition().unwrap();
        assert!(rt.organism().phase_native_temporal_goal_replanning_enabled());
        assert_eq!(rt.organism().phase_native_meta_weights(),weight_snapshot);
        // Native source checkpoint must be restorable independently too.
        let mut old=EvoPhase::new(rt.organism().config().clone());
        assert!(old.restore_phase_native_checkpoint(checkpoint));
        println!("GOAL_REPLAN_ARM arm={} intact={} drift_recovered={} surprises={} primary={:?} changed={:?} original_four={} checkpoint=true",
            arm,pass,recovered,surprises,primary,changed,first.steps);
        assert_eq!(primary.len(),4,"learned shortest four-step goal path");
        assert!(pass,"untaught four-step goal route must execute");
        assert!(recovered,"unexpected law change must trigger alternate path");
        assert!(surprises>=1,"repair must depend on factual contradicted PRE/POST");
        assert!(changed.contains(&w.motors[4])
            &&changed.contains(&w.motors[5]),"acquired detour must be composed");
    }
    println!("GOAL_REPLAN_SUMMARY intact={}/4 drift_recovered={}/4 mechanism_pass={}",
        intact,repaired,intact==4&&repaired==4);
    assert_eq!((intact,repaired),(4,4));
}
