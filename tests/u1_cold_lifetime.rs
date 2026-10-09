#![allow(dead_code)]
// NEW development generator (not TE4 or FRONTIER-1): one uninterrupted A/B/A
// causal-law lifetime, random opaque cue/motor assignment, noisy sampling and
// no matched sensory/terminal tuition. Reports failures without relabeling them
// as passing cognition. Target thresholds are fixed below before the run.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::{PhaseTemporalEvidenceConfig,PhaseHypothesisEcologyConfig};
use aeterna_v1::scientific_runtime::{ScientificRuntime,StepOutcome};

#[derive(Clone,Copy)]
struct ColdRoles {
    cues:[usize;2],
    terminal:[usize;2],
    success:usize,
    failure:usize,
    sensor:usize,
}
#[derive(Clone)]
struct ColdEpisode {
    hidden:usize,
    noisy:[usize;11],
}
fn episodes(rng:&mut Rng,n:usize)->Vec<ColdEpisode> {
    assert_eq!(n%2,0);
    let mut hidden=(0..n).map(|i|i%2).collect::<Vec<_>>();
    shuffle(rng,&mut hidden);
    hidden.into_iter().map(|h|{
        let mut noisy=[0usize;11];
        for cue in &mut noisy {
            *cue=h ^ usize::from(rng.next()%10<3);
        }
        ColdEpisode{hidden:h,noisy}
    }).collect()
}
#[derive(Default)]
struct Measures {
    right:usize,commits:usize, samples:usize, multiple:usize,
    no_action:usize,blocked:usize, oracle_one:usize,oracle_three:usize,
    actions:usize,
}
fn run_episode(
    rt:&mut ScientificRuntime, l1:&[[usize;2];8], roles:ColdRoles,
    ep:&ColdEpisode,ordinal:usize, flipped:bool,
    totals:&mut Measures, capture_early:&mut bool
) {
    let goal=foundation::scene(l1,roles.success,(ordinal+3)%6);
    rt.observe_external(&foundation::scene(
        l1,roles.cues[ep.noisy[0]],ordinal%6
    )).unwrap();
    rt.set_goal(&goal).unwrap();
    totals.oracle_one+=usize::from(ep.noisy[0]==ep.hidden);
    let majority=ep.noisy[..3].iter().sum::<usize>()>=2;
    totals.oracle_three+=usize::from(usize::from(majority)==ep.hidden);

    let mut samples=0usize;
    let mut idx=0usize;
    let mut did_commit=false;
    let mut success=false;
    for step in 0..10 {
        if !*capture_early {
            let belief=rt.organism().phase_native_temporal_evidence();
            let sensor=rt.organism().choose_phase_native_temporal_sensing_action();
            let proposed=rt.inspect_unified_competition();
            if let (Some(belief),Some(sensor),Some(mut rivals))=(belief,sensor,proposed) {
                rivals.sort_by(|a,b|b.score.total_cmp(&a.score));
                if belief.needs_more && rivals.first().is_some_and(|w|w.action!=sensor.action) {
                    println!("U1_COLD_EARLY ordinal={} step={} observed={} margin={:.4} sensor={} affordance={:.4} rivals={:?}",
                        ordinal,step,belief.observations,belief.evidence_margin,
                        sensor.action,sensor.learned_affordance,
                        rivals.iter().take(6).collect::<Vec<_>>());
                    *capture_early=true;
                }
            }
        }
        let result=rt.step_unified(
            |_|Some(safe()),
            |motor|{
                if motor==roles.sensor {
                    samples+=1;
                    idx=(idx+1).min(ep.noisy.len()-1);
                    Ok((foundation::scene(
                        l1,roles.cues[ep.noisy[idx]],(ordinal+step+1)%6
                    ),0.0))
                } else if motor==roles.terminal[0] || motor==roles.terminal[1] {
                    did_commit=true;
                    let correct=roles.terminal[ep.hidden^usize::from(flipped)];
                    success=motor==correct;
                    Ok((foundation::scene(
                        l1,if success{roles.success}else{roles.failure},
                        (ordinal+step+1)%6
                    ),f32::from(success)))
                } else {
                    Ok((foundation::scene(
                        l1,roles.cues[ep.noisy[idx]],(ordinal+step+1)%6
                    ),0.0))
                }
            }
        );
        match result {
            Ok(StepOutcome::Executed{..})=>{totals.actions+=1;},
            Ok(StepOutcome::GoalReached)=>break,
            Ok(StepOutcome::Blocked(_))=>{totals.blocked+=1;break;},
            _=>{totals.no_action+=1;break;},
        }
        if did_commit {break;}
    }
    totals.right+=usize::from(success);
    totals.commits+=usize::from(did_commit);
    totals.samples+=samples;
    totals.multiple+=usize::from(samples>=2);
}

#[test]
fn u1_new_cold_no_curriculum_continual_lifetime_development() {
    // Separate from every used frozen authority seed and the TE4 generator.
    const SEED:u64=0xA5C3_2026_51A9_D13B;
    const PASS_CORRECT:usize=64;
    const PASS_MULTI:usize=40;
    let mut rng=Rng::new(SEED);
    let mut all_pass=true;
    for arm in 0..4usize {
        let (evo,l1)=foundation::build24();
        let mut classes=(0..24usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut classes);
        let mut motors=[0usize,1,2,3,4,5];
        shuffle(&mut rng,&mut motors);
        let roles=ColdRoles {
            cues:[classes[0],classes[1]],success:classes[2],failure:classes[3],
            sensor:motors[0],terminal:[motors[1],motors[2]],
        };
        let train_data=episodes(&mut rng,192);
        let frozen_data=episodes(&mut rng,80);
        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_context_refinement());
        assert!(rt.enable_perceptual_refinement());
        assert!(rt.enable_compositional_refinement());
        assert!(rt.enable_unified_cognition(
            meta_checkpoint(),
            PhaseHypothesisEcologyConfig {
                learning_rate:0.35,dormancy_threshold:0.05,
                learning_enabled:true,phase_learning_enabled:true
            }
        ));
        assert!(rt.enable_temporal_evidence(PhaseTemporalEvidenceConfig{
            max_observations:8,minimum_observations:3,decisive_margin:0.125
        }));
        assert!(rt.set_unified_online_learning(true));
        let mut train=Measures::default();
        let mut diagnostic=false;
        for (i,episode) in train_data.iter().enumerate() {
            // A(0..64) -> B(64..128) -> A(128..192), no change flag to cognition.
            run_episode(&mut rt,&l1,roles,episode,i,
                (64..128).contains(&i),&mut train,&mut diagnostic);
        }
        let source_count=rt.organism().phase_native_temporal_source_count();
        let sensor_value=rt.organism().phase_native_temporal_action_affordance(
            roles.sensor
        ).unwrap_or(0.0);
        let meta_updates=rt.organism().phase_native_meta_observations();
        assert!(rt.restart_cognition().is_ok());
        rt.set_model_learning_enabled(false);
        let frozen_meta=rt.organism().phase_native_meta_weights();
        let mut evaluated=Measures::default();
        for (i,episode) in frozen_data.iter().enumerate() {
            run_episode(&mut rt,&l1,roles,episode,1000+arm*100+i,
                false,&mut evaluated,&mut diagnostic);
        }
        assert_eq!(rt.organism().phase_native_meta_weights(),frozen_meta,
            "frozen U1 weights must not change in heldout");
        assert!(evaluated.actions<=800,"ten-action external bound");
        assert_eq!(evaluated.blocked,0,"safe test actuators");
        let pass=evaluated.right>=PASS_CORRECT
            &&evaluated.multiple>=PASS_MULTI
            &&source_count==2 &&sensor_value>1.0e-8
            &&evaluated.right>evaluated.oracle_one;
        all_pass &= pass;
        println!("U1_COLD_ARM={} sensor={} terminal={:?} cues={:?} train_correct={}/192 train_samples={} sources={} sensor_physical={:.5} meta_updates={} heldout_correct={}/80 heldout_commit={} heldout_samples={} episodes_multi={} oracle_one={}/80 oracle_three={}/80 no_action={} physical_safety={} development_pass={}",
            arm,roles.sensor,roles.terminal,roles.cues,train.right,
            train.samples,source_count,sensor_value,meta_updates,
            evaluated.right,evaluated.commits,evaluated.samples,evaluated.multiple,
            evaluated.oracle_one,evaluated.oracle_three,evaluated.no_action,
            evaluated.blocked==0,pass);
    }
    println!("U1_COLD_SUMMARY all_arms_pass={} verdict={}",
        all_pass,if all_pass{"DEVELOPMENT_PASS"}else{"DEVELOPMENT_FAIL"});
    // This is a negative-friendly developmental measurement. Never relabel
    // all_pass=false as qualification; CI checks program safety/consistency.
}
