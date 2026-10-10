#![allow(dead_code)]
// COMPLEX-WORLD-1.1: OPEN development (benchmark correction recorded separately) of cold causal goal reasoning with a
// structurally new 14-state, 9-step gated labyrinth, distractions and drift.
// Evaluator has the hidden transition table; EvoPhase is provided only
// its recognizable raw goal, opaque actions and factual protected POSTs.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[derive(Clone)]
struct Labyrinth{
    states:[usize;14],
    motor:[usize;6],
}
impl Labyrinth{
    fn raw(&self,l1:&[[usize;2];8],state:usize,layout:usize)->Vec<f32>{
        foundation::scene(l1,self.states[state],layout)
    }
    fn next(&self,from:usize,action:usize,broken:bool)->usize{
        let m=self.motor;
        match (from,action) {
            // Main path demands first KEY, then POWER, then gated corridor.
            (0,a) if a==m[0]=>1,
            (1,a) if a==m[1]=>2,
            (2,a) if a==m[2]=>3,
            (3,a) if a==m[3]&&!broken=>4,
            (4,a) if a==m[4]=>5,
            (5,a) if a==m[5]=>6,
            (6,a) if a==m[0]=>7,
            (7,a) if a==m[1]=>8,
            (8,a) if a==m[2]=>9,
            // Alternative power access once KEY has already been acquired.
            (3,a) if a==m[5]=>10,
            (10,a) if a==m[4]=>4,
            // Misleading premature power-before-key branch and trap.
            (1,a) if a==m[3]=>11,
            (11,a) if a==m[1]=>12,
            (12,a) if a==m[2]=>1,
            // False corridor, recoverable dead end.
            (1,a) if a==m[4]=>13,
            (13,a) if a==m[0]=>1,
            // Reversible nonproductive gate loop.
            (7,a) if a==m[4]=>6,
            _=>from,
        }
    }
}
#[derive(Default)]
struct Measurements{
    success:usize,
    actions:usize,
    blocked:usize,
    unsupported:usize,
    changes:usize,
    exploratory:usize,
    hit_traps:usize,
    detour_entry:usize,
    detour_exit:usize,
    drift_noop:usize,
}
fn act_until_goal(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],w:&Labyrinth,
    episode:usize,budget:usize,broken:bool,s:&mut Measurements,
    log:bool
)->(bool,Vec<usize>,usize){
    let mut state=0usize;
    let desired=w.raw(l1,9,(episode+2)%6);
    rt.observe_external(&w.raw(l1,state,episode%6)).unwrap();
    rt.set_goal(&desired).unwrap();
    let mut actions=Vec::new();
    let mut surprises=0usize;
    for step in 0..budget{
        if state==9 {break;}
        let before=state;
        let plan=rt.organism().choose_phase_native_temporal_goal_plan(&desired);
        let explore=rt.organism().choose_phase_native_temporal_goal_frontier();
        let mut selected=None;
        let result=rt.step_unified(
            |_|Some(safe()),
            |motor|{
                selected=Some(motor);
                state=w.next(state,motor,broken);
                Ok((w.raw(l1,state,(episode+step+1)%6),
                    if state==9{1.0}else{0.0}))
            }
        );
        match result{
            Ok(StepOutcome::Executed{proposal,..})=>{
                assert_eq!(Some(proposal.action),selected);
                actions.push(proposal.action);
                s.actions+=1;
                s.changes+=usize::from(before!=state);
                s.hit_traps+=usize::from([11,12,13].contains(&state));
                s.detour_entry+=usize::from(
                    before==3&&state==10&&proposal.action==w.motor[5]
                );
                s.detour_exit+=usize::from(
                    before==10&&state==4&&proposal.action==w.motor[4]
                );
                s.drift_noop+=usize::from(
                    broken&&before==3&&state==3
                        &&proposal.action==w.motor[3]
                );
                s.exploratory+=usize::from(explore.is_some_and(
                    |p|p.0==proposal.action
                ));
                if before==state && plan.is_some_and(
                    |p|p.action==proposal.action
                ) { surprises+=1; }
                if log {
                    println!("COMPLEX11_STEP ep={} drift={} step={} pre={} action={} post={} planned={:?} frontier={:?}",
                        episode,broken,step,before,proposal.action,state,plan,explore);
                }
            }
            Ok(StepOutcome::GoalReached)=>break,
            Ok(StepOutcome::Blocked(_))=>{s.blocked+=1;break;}
            _=>{s.unsupported+=1;break;}
        }
    }
    let reached=state==9;
    s.success+=usize::from(reached);
    (reached,actions,surprises)
}
#[test]
fn complex_world1_cold_gated_labyrinth_and_drift_development(){
    const SEED:u64=0xB17E_2026_0C0D_0101;
    const ARMS:usize=4;
    const TRAIN:usize=256;
    const BUDGET:usize=24;
    let mut rng=Rng::new(SEED);
    let mut passed=0usize;
    for arm in 0..ARMS {
        let (evo,l1)=foundation::build24();
        let mut shuffled_states=(0..24usize).collect::<Vec<_>>();
        let mut motors=(0..6usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut shuffled_states);
        shuffle(&mut rng,&mut motors);
        let w=Labyrinth{
            states:shuffled_states[..14].try_into().unwrap(),
            motor:motors.try_into().unwrap(),
        };
        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_temporal_evidence(PhaseTemporalEvidenceConfig{
            max_observations:8,minimum_observations:3,decisive_margin:0.125
        }));
        assert!(rt.set_temporal_multistep(true));
        assert!(rt.set_temporal_goal_replanning(true));
        assert!(rt.set_temporal_cold_goal_acquisition(true));
        assert!(rt.enable_unified_cognition(meta_checkpoint(),
            PhaseHypothesisEcologyConfig{
                learning_rate:0.35,dormancy_threshold:0.05,
                learning_enabled:true,phase_learning_enabled:true
            }
        ));
        let mut learning=Measurements::default();
        for ep in 0..TRAIN {
            let _=act_until_goal(&mut rt,&l1,&w,ep,BUDGET,false,
                &mut learning,ep==0);
        }
        let edge_count=rt.organism().phase_native_temporal_transition_count();
        let goal=w.raw(&l1,9,0);
        rt.observe_external(&w.raw(&l1,0,1)).unwrap();
        rt.set_goal(&goal).unwrap();
        let first=rt.organism().choose_phase_native_temporal_goal_plan(&goal);
        let physical_test=if let Some(plan)=first{
            let mut damaged=rt.organism().clone();
            let saved=damaged.perturb_phase_native_synapse_for_control(
                plan.synapse,0.0,0.0).unwrap();
            let absent=damaged.choose_phase_native_temporal_goal_plan(&goal)
                .is_none();
            damaged.restore_phase_native_synapse_for_control(plan.synapse,saved);
            absent && damaged.choose_phase_native_temporal_goal_plan(&goal)
                ==Some(plan)
        }else{false};
        assert!(rt.restart_cognition().is_ok());
        rt.set_model_learning_enabled(false);
        let frozen_weights=rt.organism().phase_native_meta_weights();
        let mut held=Measurements::default();
        let (intact,path,_)=act_until_goal(
            &mut rt,&l1,&w,1000+arm,16,false,&mut held,true
        );
        let stable=frozen_weights==rt.organism().phase_native_meta_weights();
        rt.set_model_learning_enabled(true);
        let mut drift=Measurements::default();
        let (recovered,alternate,surprises)=act_until_goal(
            &mut rt,&l1,&w,2000+arm,20,true,&mut drift,true
        );
        let both_detour=drift.detour_entry>=1 && drift.detour_exit>=1
            &&drift.drift_noop>=1;
        let pass=edge_count>=11 && learning.hit_traps>0
            &&first.is_some_and(|p|p.steps==9)
            &&physical_test &&stable &&intact && path.len()==9
            &&recovered && surprises>=1 && both_detour
            &&learning.blocked==0&&learning.unsupported==0
            &&held.blocked==0&&held.unsupported==0
            &&drift.blocked==0&&drift.unsupported==0;
        passed+=usize::from(pass);
        println!("COMPLEX_WORLD11_ARM arm={} motors={:?} edges={} train_goals={}/{} train_actions={} changed={} traps={} frontier={} first_plan={:?} frozen_goal={} frozen_len={} physical_lesion={} meta_frozen={} drift_goal={} drift_len={} surprise={} actual_blocked_edge={} detour_entry={} detour_exit={} detour={} blocked={} unavailable={} pass={}",
            arm,w.motor,edge_count,learning.success,TRAIN,
            learning.actions,learning.changes,learning.hit_traps,
            learning.exploratory,first,intact,path.len(),physical_test,
            stable,recovered,alternate.len(),surprises,drift.drift_noop,
            drift.detour_entry,drift.detour_exit,both_detour,
            learning.blocked+held.blocked+drift.blocked,
            learning.unsupported+held.unsupported+drift.unsupported,
            pass);
    }
    println!("COMPLEX_WORLD11_SUMMARY passed={}/{} verdict={}",
        passed,ARMS,if passed==ARMS{"DEVELOPMENT_PASS"}else{"DEVELOPMENT_FAIL"});
    // An open negative diagnostic is still valid science; never relabel
    // compilation SUCCESS as cognitive PASS.
}
