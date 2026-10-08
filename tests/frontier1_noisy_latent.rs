#![allow(dead_code)]
// FRONTIER-1: genuinely stochastic latent cause with repeated noisy
// observation. The evaluator holds latent truth; EvoPhase never receives it.
include!("intel2_unified_worlds.rs");

#[derive(Debug, Clone, Copy)]
struct F1Roles {
    cue: [usize; 2],
    goal: usize,
    dead: usize,
    sample: usize,
    commit: [usize; 2],
}

#[derive(Debug, Clone)]
struct F1Episode {
    hidden: usize,
    cues: [usize; 8],
}

fn f1_episodes(rng: &mut Rng, count: usize) -> Vec<F1Episode> {
    assert_eq!(count % 2, 0);
    let mut sides = (0..count).map(|i| i % 2).collect::<Vec<_>>();
    shuffle(rng, &mut sides);
    sides.into_iter().map(|hidden| {
        let mut cues = [0usize;8];
        for c in &mut cues {
            let erroneous = rng.next() % 10 < 3;
            *c = hidden ^ usize::from(erroneous);
        }
        F1Episode {hidden,cues}
    }).collect()
}

#[derive(Debug,Default)]
struct F1Metrics {
    success: usize,
    commits: usize,
    samples: usize,
    actions: usize,
    supported: usize,
    last_cue_match: usize,
    majority8: usize,
    episodes_multiple_samples: usize,
}

fn f1_execute_episode(
    rt: &mut ScientificRuntime,
    l1: &[[usize;2];8],
    roles: F1Roles,
    ep: &F1Episode,
    layout: usize,
    data: &mut F1Metrics,
) {
    let goal_image = foundation::scene(l1,roles.goal,(layout+1)%6);
    let mut read_index = 0usize;
    let mut sample_count = 0usize;
    let mut has_committed = false;
    let mut success = false;

    // A reset is an external new observation, never a taught terminal arc.
    rt.observe_external(&foundation::scene(
        l1,roles.cue[ep.cues[0]],layout%6
    )).expect("valid raw F1 observation");
    rt.set_goal(&goal_image).expect("valid F1 raw goal");

    for k in 0..10usize {
        let response = rt.step_unified(
            |_|Some(safe()),
            |motor|{
                if motor == roles.sample {
                    sample_count += 1;
                    read_index = (read_index+1).min(7);
                    (Ok((foundation::scene(
                        l1,roles.cue[ep.cues[read_index]],(layout+k+1)%6
                    ),0.0)))
                } else if motor == roles.commit[0] || motor == roles.commit[1] {
                    has_committed=true;
                    success = motor==roles.commit[ep.hidden];
                    Ok((foundation::scene(
                        l1,if success{roles.goal}else{roles.dead},
                        (layout+k+1)%6
                    ),f32::from(success)))
                } else {
                    // The other opaque motors are factual nonterminal actions,
                    // not hidden-mode observations or simulator hints.
                    Ok((foundation::scene(
                        l1,roles.cue[ep.cues[read_index]],(layout+k+1)%6
                    ),0.0))
                }
            }
        );
        match response {
            Ok(StepOutcome::Executed{..}) => {
                data.supported += 1;
                data.actions += 1;
                if has_committed { break; }
            }
            Ok(StepOutcome::GoalReached) => break,
            _ => break,
        }
    }

    data.samples += sample_count;
    data.commits += usize::from(has_committed);
    data.success += usize::from(success);
    data.episodes_multiple_samples += usize::from(sample_count>=2);
    // Strong oracle uses the last actually visible (possibly re-sampled) cue.
    data.last_cue_match += usize::from(ep.cues[read_index]==ep.hidden);
    let votes_one=ep.cues.iter().filter(|&&cue|cue==1).count();
    let majority_side=usize::from(votes_one>=5);
    data.majority8+=usize::from(majority_side==ep.hidden);
}

#[test]
fn frontier1_source_guard(){
    let source=include_str!("frontier1_noisy_latent.rs");
    assert!(source.contains("step_unified"));
    assert!(source.contains("f1_execute_episode"));
    let forbidden=[
        ["world","_id"].concat(),
        ["task","_id"].concat(),
        ["rt",".step("].concat(),
    ];
    for token in forbidden {
        assert!(!source.contains(&token),
            "forbidden host cognitive shortcut: {token}");
    }
}

#[test]
fn frontier1_frozen_stochastic_latent_evidence_lifetime(){
    let seed:u64=std::env::var("AETERNA_FRONTIER1_SEED")
        .expect("authority seed").parse().unwrap();
    let source_sha=std::env::var("AETERNA_SOURCE_SHA")
        .unwrap_or_else(|_|"unknown".into());
    let spec_sha=std::env::var("AETERNA_SPEC_SHA")
        .unwrap_or_else(|_|"unknown".into());

    let (evo,l1)=foundation::build24();
    let checkpoint=meta_checkpoint();
    let mut rng=Rng::new(seed);
    let mut classes=(0..24usize).collect::<Vec<_>>();
    shuffle(&mut rng,&mut classes);
    let mut motors=[0usize,1,2,3,4,5];
    shuffle(&mut rng,&mut motors);
    let roles=F1Roles{
        cue:[classes[0],classes[1]],
        goal:classes[2],
        dead:classes[3],
        sample:motors[0],
        commit:[motors[1],motors[2]],
    };
    let training=f1_episodes(&mut rng,320);
    let heldout=f1_episodes(&mut rng,80);

    // Seal before any target observation/action. Neither target mapping nor
    // the random process enters the cognitive runtime.
    println!(
        "FRONTIER1_SEAL source_sha={} spec_sha={} seed={} cues={:?} goal={} dead={} sample={} commits={:?} train=320 heldout=80",
        source_sha,spec_sha,seed,roles.cue,roles.goal,roles.dead,
        roles.sample,roles.commit
    );

    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_context_refinement());
    assert!(rt.enable_perceptual_refinement());
    assert!(rt.enable_compositional_refinement());
    assert!(rt.enable_unified_cognition(
        checkpoint,PhaseHypothesisEcologyConfig{
            learning_rate:0.35,dormancy_threshold:0.05,
            learning_enabled:true,phase_learning_enabled:true
        }
    ));
    let mut train=F1Metrics::default();
    for (i,episode) in training.iter().enumerate(){
        f1_execute_episode(
            &mut rt,&l1,roles,episode,i%6,&mut train
        );
    }
    let prior_fingerprint=rt.organism().phase_native_learned_fingerprint();
    rt.restart_cognition().unwrap();
    assert_eq!(rt.organism().phase_native_learned_fingerprint(),
        prior_fingerprint);
    rt.set_model_learning_enabled(false);
    let mut eval=F1Metrics::default();
    for (i,episode) in heldout.iter().enumerate(){
        f1_execute_episode(
            &mut rt,&l1,roles,episode,(i+3)%6,&mut eval
        );
    }
    let onecue=eval.last_cue_match.max(80-eval.last_cue_match);
    let mean_action=eval.actions as f64/80.0;
    let mean_sample=eval.samples as f64/80.0;
    let legacy=rt.organism().planning_transition_count();
    let answer_tables=rt.organism().composite_concepts().len();
    println!(
        "FRONTIER1_RESULT training={}/320 train_commits={} heldout={}/80 commits={} onecue_oracle={}/80 majority8_oracle={}/80 episodes_2plus_samples={} mean_actions={:.3} mean_samples={:.3} legacy={} answer_tables={} verdict={}",
        train.success,train.commits,eval.success,eval.commits,
        onecue,eval.majority8,eval.episodes_multiple_samples,
        mean_action,mean_sample,legacy,answer_tables,
        if eval.success>=64 && onecue<=60
            && eval.success>onecue
            && eval.episodes_multiple_samples>=40
            && legacy==0 && answer_tables==0 {
            "PASS_BOUNDED_STOCHASTIC_EVIDENCE_INTEGRATION"
        } else { "FAIL" }
    );
    assert!(eval.success>=64,"FRONTIER1 failed unseen noisy latent episodes");
    assert!(onecue<=60,"FRONTIER1 pack lacks single-cue baseline separation");
    assert!(eval.success>onecue,"FRONTIER1 no temporal-evidence advantage");
    assert!(eval.episodes_multiple_samples>=40,"FRONTIER1 no active repeated sensing");
    assert_eq!(legacy,0);
    assert_eq!(answer_tables,0);
}
