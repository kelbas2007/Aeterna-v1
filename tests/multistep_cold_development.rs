#![allow(dead_code)]
// Open variable-depth development, NOT a frozen independent scientific gate.
// Cognitive code receives only actual raw PRE/action/POST, bounded reward and
// generic goal. The hidden side, causal stage, marker classes and motor roles
// are private to this evaluator. Each arm has its own uninterrupted lifetime.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[derive(Clone)]
struct Episode {hidden:usize, cues:[usize;19]}
fn episodes(rng:&mut Rng,n:usize)->Vec<Episode>{
    let mut sides=(0..n).map(|i|i%2).collect::<Vec<_>>();
    shuffle(rng,&mut sides);
    sides.into_iter().map(|hidden|{
        let mut cues=[0usize;19];
        for cue in &mut cues {
            *cue=hidden^usize::from(rng.next()%10<3);
        }
        Episode{hidden,cues}
    }).collect()
}
#[derive(Clone)]
struct World {
    depth:usize,
    cue:[usize;2],
    markers:Vec<usize>,
    chain:Vec<usize>,
    terminal:[usize;2],
    success:usize,
    failure:usize,
}
#[derive(Default)]
struct Measures{
    correct:usize,commits:usize,actions:usize,blocked:usize,unavailable:usize,
    sampled:usize,repeated:usize,completed_chains:usize,first_cue_oracle:usize,
    triple_cue_oracle:usize,
    entered_stage:[usize;6], staged_attempts:[usize;6],
    first_motor_counts:[usize;6],
}
fn perform(rt:&mut ScientificRuntime,l1:&[[usize;2];8],w:&World,
    ep:&Episode,ordinal:usize,flipped:bool,m:&mut Measures)
{
    rt.observe_external(&foundation::scene(
        l1,w.cue[ep.cues[0]],ordinal%6
    )).unwrap();
    rt.set_goal(&foundation::scene(
        l1,w.success,(ordinal+3)%6
    )).unwrap();
    m.first_cue_oracle+=usize::from(ep.cues[0]==ep.hidden);
    m.triple_cue_oracle+=usize::from(
        usize::from(ep.cues[..3].iter().sum::<usize>()>=2)==ep.hidden
    );
    let mut state=0usize;
    let mut index=0usize;
    let mut sampled=0usize;
    let mut commit=false;
    let mut correct=false;
    let mut current_cue=ep.cues[0];
    for step in 0..16usize {
        if commit {break;}
        let stage_before=state;
        let result=rt.step_unified(
            |_|Some(safe()),
            |motor|{
                if step==0 {m.first_motor_counts[motor]+=1;}
                m.staged_attempts[stage_before]+=1;
                let layout=(ordinal+step+1)%6;
                if motor==w.terminal[0]||motor==w.terminal[1]{
                    commit=true;
                    correct=motor==w.terminal[ep.hidden^usize::from(flipped)];
                    return Ok((foundation::scene(
                        l1,if correct{w.success}else{w.failure},layout
                    ),f32::from(correct)));
                }
                if motor==w.chain[state] {
                    state+=1;
                    m.entered_stage[state]+=1;
                    if state==w.depth {
                        state=0;
                        index=(index+1).min(ep.cues.len()-1);
                        current_cue=ep.cues[index];
                        sampled+=1;
                        m.completed_chains+=1;
                        return Ok((foundation::scene(
                            l1,w.cue[current_cue],layout
                        ),0.0));
                    }
                    return Ok((foundation::scene(
                        l1,w.markers[state-1],layout
                    ),0.0));
                }
                let stay=if state==0 {w.cue[current_cue]} else {
                    w.markers[state-1]
                };
                Ok((foundation::scene(l1,stay,layout),0.0))
            }
        );
        match result{
            Ok(StepOutcome::Executed{..})=>m.actions+=1,
            Ok(StepOutcome::GoalReached)=>break,
            Ok(StepOutcome::Blocked(_))=>{m.blocked+=1;break;},
            _=>{m.unavailable+=1;break;},
        }
    }
    m.correct+=usize::from(correct);
    m.commits+=usize::from(commit);
    m.sampled+=sampled;
    m.repeated+=usize::from(sampled>=2);
}
#[test]
fn multistep_unknown_depth_cold_open_development(){
    const SEED:u64=0xA17E_2026_D3E5_0084;
    const PASS_CORRECT:usize=64;
    const PASS_MULTI:usize=30;
    let mut rng=Rng::new(SEED);
    let mut passes=0usize;
    for depth in 2..=5usize {
        let (evo,l1)=foundation::build24();
        let mut states=(0..24usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut states);
        let mut motors=(0..6usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut motors);
        // Six opaque motors suffice for a five-action chain: a motor is
        // reused in a different physical state. Two terminal motors remain.
        let chain=(0..depth).map(|i|motors[i%4]).collect::<Vec<_>>();
        let w=World{
            depth,chain,
            cue:[states[0],states[1]],
            markers:states[2..2+depth-1].to_vec(),
            success:states[10],failure:states[11],
            terminal:[motors[4],motors[5]]
        };
        let training=episodes(&mut rng,192);
        let heldout=episodes(&mut rng,80);
        let mut rt=ScientificRuntime::new(evo).unwrap();
        assert!(rt.enable_context_refinement());
        assert!(rt.enable_perceptual_refinement());
        assert!(rt.enable_compositional_refinement());
        assert!(rt.enable_unified_cognition(
            meta_checkpoint(),
            PhaseHypothesisEcologyConfig{
                learning_rate:0.35,dormancy_threshold:0.05,
                learning_enabled:true,phase_learning_enabled:true
            }
        ));
        assert!(rt.enable_temporal_evidence(PhaseTemporalEvidenceConfig{
            max_observations:8,minimum_observations:3,decisive_margin:0.125
        }));
        assert!(rt.set_unified_online_learning(true));
        assert!(rt.set_temporal_autonomous_probe(true));
        assert!(rt.set_temporal_multistep(true));
        let mut before=Measures::default();
        for (i,ep) in training.iter().enumerate() {
            perform(&mut rt,&l1,&w,ep,i,(64..128).contains(&i),&mut before);
        }
        let edges=rt.organism().phase_native_temporal_transition_count();
        let read=rt.organism().phase_native_temporal_action_affordance(
            *w.chain.last().unwrap()
        ).unwrap_or(0.0);
        assert!(rt.restart_cognition().is_ok());
        assert!(rt.organism().phase_native_temporal_multistep_enabled());
        rt.set_model_learning_enabled(false);
        let frozen_weights=rt.organism().phase_native_meta_weights();
        let mut after=Measures::default();
        for (i,ep) in heldout.iter().enumerate(){
            perform(&mut rt,&l1,&w,ep,1000+100*depth+i,false,&mut after);
        }
        let stable=rt.organism().phase_native_meta_weights()==frozen_weights;
        let pass=after.correct>=PASS_CORRECT
            && after.correct>after.first_cue_oracle
            && after.repeated>=PASS_MULTI &&after.blocked==0
            &&after.unavailable==0 &&stable &&read>1.0e-8;
        passes+=usize::from(pass);
        println!("MULTISTEP_STAGES depth={} train_reached={:?} train_attempts={:?} train_first_motors={:?} held_reached={:?}",
            depth,before.entered_stage,before.staged_attempts,
            before.first_motor_counts,after.entered_stage);
        println!("MULTISTEP_COLD depth={} chain={:?} terminal={:?} train_correct={}/192 train_chains={} transition_links={} final_sensor={:.6} heldout_correct={}/80 heldout_commit={} heldout_samples={} heldout_multi={} firstcue={}/80 threecue={}/80 blocked={} unavailable={} u1_frozen={} development_pass={}",
            depth,w.chain,w.terminal,before.correct,before.completed_chains,
            edges,read,after.correct,after.commits,after.sampled,after.repeated,
            after.first_cue_oracle,after.triple_cue_oracle,after.blocked,
            after.unavailable,stable,pass
        );
    }
    println!("MULTISTEP_COLD_SUMMARY passed={}/4 verdict={}",
        passes,if passes==4{"DEVELOPMENT_PASS"}else{"DEVELOPMENT_FAIL"});
    // Negative-friendly: CI must report a failed scientific criterion
    // independently of whether this diagnostic executable exits 0.
}
