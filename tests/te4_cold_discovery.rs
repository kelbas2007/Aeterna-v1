#![allow(dead_code)]
// TE4: independent development diagnostic of *cold*, self-selected exploration
// in a noisy hidden-cause world. This does NOT reuse the FRONTIER-1 sealed
// authority seed, classes or motor mapping and supplies no matched tuition.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[derive(Debug, Clone, Copy)]
struct TE4Roles {
    sources: [usize;2],
    success: usize,
    failure: usize,
    sensor: usize,
    commit: [usize;2],
}

#[derive(Debug, Clone)]
struct TE4Episode {
    hidden: usize,
    observations: [usize;11],
}
fn te4_episodes(rng:&mut Rng, n:usize)->Vec<TE4Episode>{
    assert_eq!(n%2,0);
    let mut modes=(0..n).map(|i|i%2).collect::<Vec<_>>();
    shuffle(rng,&mut modes);
    modes.into_iter().map(|hidden|{
        let mut observations=[0usize;11];
        for observation in &mut observations {
            let inverted=rng.next()%10<3;
            *observation=hidden^usize::from(inverted);
        }
        TE4Episode{hidden,observations}
    }).collect()
}

#[derive(Debug, Default)]
struct TE4Metrics{
    episodes:usize,
    commits:usize,
    right:usize,
    samples:usize,
    two_plus:usize,
    actions:usize,
    unable:usize,
    blocked:usize,
    last_observation_oracle:usize,
}

fn te4_episode(
    rt:&mut ScientificRuntime,
    l1:&[[usize;2];8],
    roles:TE4Roles,
    episode:&TE4Episode,
    episode_index:usize,
    metrics:&mut TE4Metrics,
){
    metrics.episodes+=1;
    let mut idx=0usize;
    let mut committed=false;
    let mut success=false;
    let mut samples=0usize;
    let goal_raw=foundation::scene(l1,roles.success,(episode_index+1)%6);
    let first_raw=foundation::scene(l1,roles.sources[episode.observations[0]],episode_index%6);
    rt.observe_external(&first_raw).expect("external real cue");
    rt.set_goal(&goal_raw).expect("valid raw goal");
    for k in 0..10usize{
        if committed{break;}
        let action=rt.step_unified(
            |_|Some(safe()),
            |motor|{
                if motor==roles.sensor {
                    samples+=1;
                    idx=(idx+1).min(episode.observations.len()-1);
                    Ok((foundation::scene(
                        l1,roles.sources[episode.observations[idx]],
                        (episode_index+k+2)%6
                    ),0.0))
                } else if motor==roles.commit[0]||motor==roles.commit[1]{
                    committed=true;
                    success=(motor==roles.commit[episode.hidden]);
                    Ok((foundation::scene(
                        l1,if success{roles.success}else{roles.failure},
                        (episode_index+k+2)%6
                    ),f32::from(success)))
                }else{
                    Ok((foundation::scene(
                        l1,roles.sources[episode.observations[idx]],
                        (episode_index+k+2)%6
                    ),0.0))
                }
            },
        );
        match action {
            Ok(StepOutcome::Executed{..})=>{metrics.actions+=1;},
            Ok(StepOutcome::Blocked(_))=>{metrics.blocked+=1;break;},
            Ok(StepOutcome::GoalReached)=>break,
            _=>{metrics.unable+=1;break;},
        }
    }
    metrics.samples+=samples;
    metrics.two_plus+=usize::from(samples>=2);
    metrics.commits+=usize::from(committed);
    metrics.right+=usize::from(success);
    metrics.last_observation_oracle+=
        usize::from(episode.observations[idx]==episode.hidden);
}

#[test]
fn te4_cold_no_tuition_one_lifetime_four_independent_worlds(){
    // This is a diagnostic generator selected independently of FRONTIER-1.
    const BASE:u64=0xC01D_2026_A77E_0044;
    let mut rng=Rng::new(BASE);
    let mut all_pass=true;
    for arm in 0..4usize {
        let (evo,l1)=foundation::build24();
        let mut classes=(0..24usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut classes);
        let mut motors=[0usize,1,2,3,4,5];
        shuffle(&mut rng,&mut motors);
        let roles=TE4Roles{
            sources:[classes[0],classes[1]],
            success:classes[2],
            failure:classes[3],
            sensor:motors[0],
            commit:[motors[1],motors[2]],
        };
        let training=te4_episodes(&mut rng,128);
        let heldout=te4_episodes(&mut rng,80);
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
            max_observations:8,minimum_observations:3,decisive_margin:0.125
        }));
        assert_eq!(rt.organism().phase_native_temporal_source_count(),0);
        let mut train=TE4Metrics::default();
        for (i,ep) in training.iter().enumerate(){
            te4_episode(&mut rt,&l1,roles,ep,i,&mut train);
        }
        let learned_sensor=rt.organism().phase_native_temporal_action_affordance(
            roles.sensor
        ).unwrap_or(0.0);
        let sources=rt.organism().phase_native_temporal_source_count();
        let checkpoint=rt.organism().phase_native_checkpoint().unwrap();
        // The checkpoint survives an actual cold cognitive restart while
        // requiring a new raw observation before future actuation.
        assert!(rt.restart_cognition().is_ok());
        let restored_source_count=rt.organism().phase_native_temporal_source_count();
        assert_eq!(restored_source_count,sources);
        // Do not silently train the cold organism during evaluation.
        rt.set_model_learning_enabled(false);
        let mut held=TE4Metrics::default();
        for (i,ep) in heldout.iter().enumerate(){
            te4_episode(&mut rt,&l1,roles,ep,i+1000+arm*83,&mut held);
        }
        let last_oracle=held.last_observation_oracle.max(80-held.last_observation_oracle);
        let pass=held.right>=64&&held.two_plus>=40
            &&learned_sensor>1.0e-8
            &&held.right>last_oracle
            &&held.unable==0
            &&held.blocked==0
            &&sources==2;
        all_pass &= pass;
        println!(
            "TE4_COLD_ARM arm={} sensor={} commits={:?} cue_classes={:?} train_right={}/128 train_commits={} train_samples={} train_unavailable={} physical_sources={} learned_sensor={:.6} heldout={}/80 heldout_commits={} heldout_samples={} episodes_2plus={} lastcue_oracle={}/80 heldout_unavailable={} blocked={} pass={}",
            arm,roles.sensor,roles.commit,roles.sources,train.right,
            train.commits,train.samples,train.unable,sources,learned_sensor,
            held.right,held.commits,held.samples,held.two_plus,last_oracle,
            held.unable,held.blocked,pass,
        );
        let _checkpoint_was_built=checkpoint;
    }
    println!("TE4_COLD_SUMMARY four_arms_pass={} verdict={}",
        all_pass,if all_pass{"DEVELOPMENT_PASS"}else{"DEVELOPMENT_FAIL"});
    assert!(all_pass,"cold autonomous stochastic evidence acquisition is not yet qualified");
}
