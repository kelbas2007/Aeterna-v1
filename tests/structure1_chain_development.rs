#![allow(dead_code)]
// STRUCTURE-1: open developmental after frozen first-attempt, two-step active
// sensing diagnostic. Based on independent burned STRUCTURE-1 negative witness; this source is NOT a frozen scientific qualification.
// Unlike Fresh2, no single opaque motor can produce a new cue:
// arm -> observable ready state -> distinct read motor -> noisy cue.
// No motor roles or hidden labels are given to EvoPhase.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[derive(Clone,Copy)]
struct Roles {
    cue:[usize;2], ready:usize, success:usize, failure:usize,
    arm:usize, read:usize, terminal:[usize;2]
}
#[derive(Clone)]
struct Episode { hidden:usize, noisy:[usize;13] }
fn make_episodes(rng:&mut Rng,n:usize)->Vec<Episode>{
    assert_eq!(n%2,0);
    let mut hidden=(0..n).map(|i|i%2).collect::<Vec<_>>();
    shuffle(rng,&mut hidden);
    hidden.into_iter().map(|h|{
        let mut noisy=[0usize;13];
        for x in &mut noisy {*x=h^usize::from(rng.next()%10<3);}
        Episode{hidden:h,noisy}
    }).collect()
}
#[derive(Default)]
struct Metrics {
    right:usize, commits:usize, actions:usize, unavailable:usize,
    blocked:usize, arm_actions:usize, read_actions:usize,
    successful_reads:usize, two_extra:usize,
    firstcue_oracle:usize, threecue_oracle:usize,
}
fn one_episode(
    rt:&mut ScientificRuntime,l1:&[[usize;2];8],roles:Roles,ep:&Episode,
    ordinal:usize,flipped:bool,m:&mut Metrics
) {
    let first=foundation::scene(l1,roles.cue[ep.noisy[0]],ordinal%6);
    let goal=foundation::scene(l1,roles.success,(ordinal+3)%6);
    rt.observe_external(&first).expect("factual episode start");
    rt.set_goal(&goal).expect("recognizable raw goal");
    m.firstcue_oracle+=usize::from(ep.noisy[0]==ep.hidden);
    let majority=ep.noisy[..3].iter().sum::<usize>()>=2;
    m.threecue_oracle+=usize::from(usize::from(majority)==ep.hidden);
    let mut ready=false;
    let mut idx=0usize;
    let mut new_cues=0usize;
    let mut commit=false;
    let mut success=false;
    let mut current=ep.noisy[0];
    for step in 0..12usize {
        if commit {break;}
        let result=rt.step_unified(
            |_|Some(safe()),
            |motor|{
                let layout=(ordinal+step+1)%6;
                if motor==roles.arm {
                    m.arm_actions+=1;
                    ready=true;
                    // The READY state is external and visible, not a hidden
                    // mode supplied to native cognition.
                    return Ok((foundation::scene(l1,roles.ready,layout),0.0));
                }
                if motor==roles.read {
                    m.read_actions+=1;
                    if ready {
                        ready=false;
                        idx=(idx+1).min(ep.noisy.len()-1);
                        current=ep.noisy[idx];
                        new_cues+=1;
                        m.successful_reads+=1;
                        return Ok((
                            foundation::scene(l1,roles.cue[current],layout),0.0
                        ));
                    }
                    return Ok((foundation::scene(l1,roles.cue[current],layout),0.0));
                }
                if motor==roles.terminal[0] || motor==roles.terminal[1]{
                    commit=true;
                    success=motor==roles.terminal[
                        ep.hidden^usize::from(flipped)
                    ];
                    return Ok((foundation::scene(l1,if success{
                        roles.success
                    }else{
                        roles.failure
                    },layout),f32::from(success)));
                }
                // Irrelevant opaque motors cannot magically release cues.
                let unchanged=if ready {roles.ready} else {roles.cue[current]};
                Ok((foundation::scene(l1,unchanged,layout),0.0))
            }
        );
        match result {
            Ok(StepOutcome::Executed{..})=>m.actions+=1,
            Ok(StepOutcome::GoalReached)=>break,
            Ok(StepOutcome::Blocked(_))=>{m.blocked+=1;break;},
            _=>{m.unavailable+=1;break;},
        }
    }
    m.commits+=usize::from(commit);
    m.right+=usize::from(success);
    m.two_extra+=usize::from(new_cues>=2);
}

#[test]
fn structure1_acquired_chained_sensing_open_development(){
    const SEED:u64=0x57A7_2026_5E71_011A;
    const PASS_CORRECT:usize=64;
    const PASS_TWO_EXTRA:usize=32;
    let mut rng=Rng::new(SEED);
    let mut passed=0usize;
    for arm_i in 0..4usize {
        let (evo,l1)=foundation::build24();
        let mut classes=(0..24usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut classes);
        let mut motors=[0usize,1,2,3,4,5];
        shuffle(&mut rng,&mut motors);
        let roles=Roles{
            cue:[classes[0],classes[1]],ready:classes[2],
            success:classes[3],failure:classes[4],
            arm:motors[0],read:motors[1],
            terminal:[motors[2],motors[3]],
        };
        assert_ne!(roles.arm,roles.read);
        assert!(!roles.cue.contains(&roles.ready));
        let train=make_episodes(&mut rng,192);
        let heldout=make_episodes(&mut rng,80);
        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_context_refinement());
        assert!(rt.enable_perceptual_refinement());
        assert!(rt.enable_compositional_refinement());
        assert!(rt.enable_unified_cognition(
            meta_checkpoint(),
            PhaseHypothesisEcologyConfig {
                learning_rate:0.35,dormancy_threshold:0.05,
                learning_enabled:true,phase_learning_enabled:true,
            }
        ));
        assert!(rt.enable_temporal_evidence(PhaseTemporalEvidenceConfig{
            max_observations:8,minimum_observations:3,decisive_margin:0.125,
        }));
        assert!(rt.set_unified_online_learning(true));
        assert!(rt.set_temporal_autonomous_probe(true));
        assert!(rt.set_temporal_chain_learning(true));
        let mut learning=Metrics::default();
        for (i,episode) in train.iter().enumerate(){
            one_episode(&mut rt,&l1,roles,episode,i,(64..128).contains(&i),
                &mut learning);
        }
        let acquired_cue_count=rt.organism().phase_native_temporal_source_count();
        let acquired_read=rt.organism()
            .phase_native_temporal_action_affordance(roles.read)
            .unwrap_or(0.0);
        assert!(rt.restart_cognition().is_ok());
        rt.set_model_learning_enabled(false);
        let weights_before=rt.organism().phase_native_meta_weights();
        let mut frozen=Metrics::default();
        for (i,episode) in heldout.iter().enumerate(){
            one_episode(&mut rt,&l1,roles,episode,1000+arm_i*100+i,
                false,&mut frozen);
        }
        let weights_stable=weights_before==rt.organism().phase_native_meta_weights();
        let pass=weights_stable && frozen.right>=PASS_CORRECT
            && frozen.right>frozen.firstcue_oracle
            && frozen.two_extra>=PASS_TWO_EXTRA
            && frozen.unavailable==0 && frozen.blocked==0
            && acquired_cue_count==2 && acquired_read>1.0e-8;
        passed+=usize::from(pass);
        println!("STRUCTURE1_CHAIN_DEV_ARM arm={} motor_arm={} motor_read={} terminal={:?} cue={:?} ready={} train_right={}/192 train_arm={} train_read={} train_new_cues={} learnt_read={:.6} frozen_right={}/80 frozen_commits={} frozen_arm={} frozen_read={} frozen_actual_new_cues={} episodes_2plus={} firstcue_oracle={}/80 threecue_oracle={}/80 unavailable={} blocked={} unchanged_u1={} pass={}",
            arm_i,roles.arm,roles.read,roles.terminal,roles.cue,roles.ready,
            learning.right,learning.arm_actions,learning.read_actions,
            learning.successful_reads,acquired_read,
            frozen.right,frozen.commits,frozen.arm_actions,frozen.read_actions,
            frozen.successful_reads,frozen.two_extra,
            frozen.firstcue_oracle,frozen.threecue_oracle,
            frozen.unavailable,frozen.blocked,weights_stable,pass);
    }
    println!("STRUCTURE1_CHAIN_DEV_SUMMARY passed={}/4 verdict={}",
        passed,if passed==4{"STRUCTURE1_CHAIN_DEVELOPMENT_PASS"}else{
            "STRUCTURE1_CHAIN_DEVELOPMENT_FAIL"
        });
    // Scientific failure must remain visible in log while allowing the
    // regression/diagnostic runner to preserve COMPLETE negative evidence.
}
