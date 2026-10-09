#![allow(dead_code)]
// Open mechanism witness, not a cold autonomous intelligence qualification.
// The learner sees factual PRE/action/POST observations only. All motor names,
// intermediate classes and path lengths below are evaluator-private controls.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[test]
fn physical_sensing_paths_of_variable_depth_are_reused_and_lesionable(){
    let mut rng=Rng::new(0x9D47_2026_5E12_4481);
    for depth in 2..=5usize {
        for arm in 0..3usize {
            let (mut evo,l1)=foundation::build24();
            assert!(evo.enable_phase_native_temporal_evidence(
                PhaseTemporalEvidenceConfig{
                    max_observations:8,minimum_observations:3,
                    decisive_margin:0.125,
                }
            ));
            assert!(evo.set_phase_native_temporal_multistep(true));
            assert!(evo.set_phase_native_temporal_autonomous_probe(true));
            let mut states=(0..24usize).collect::<Vec<_>>();
            shuffle(&mut rng,&mut states);
            let cues=[states[0],states[1]];
            let mut waypoints=vec![cues[0]];
            waypoints.extend_from_slice(&states[2..2+depth-1]);
            waypoints.push(cues[1]);
            let mut motors=(0..6usize).collect::<Vec<_>>();
            shuffle(&mut rng,&mut motors);
            let motors=&motors[..depth];
            for cue in cues{
                assert!(evo.observe_phase_native_temporal_signal(
                    &foundation::scene(&l1,cue,0)
                ));
            }
            assert!(evo.begin_phase_native_temporal_episode());
            // Factual demonstrations exercise carrier storage only; the
            // independent challenge must later test self-directed discovery.
            for step in 0..depth {
                let before=foundation::scene(&l1,waypoints[step],0);
                let after=foundation::scene(&l1,waypoints[step+1],1);
                assert!(evo.observe_phase_native_sensing_affordance(
                    motors[step],&before,&after
                ));
                assert_eq!(
                    evo.phase_native_temporal_factual_sample_from(
                        motors[step],&before,&after
                    ),step+1==depth,
                    "only a physically complete path to a cue yields a sample"
                );
            }
            assert_eq!(evo.phase_native_temporal_transition_count(),depth);
            assert!(evo.phase_native_temporal_action_affordance(
                motors[depth-1]
            ).unwrap_or(0.0)>0.0);
            assert!(evo.begin_phase_native_temporal_episode());
            assert!(evo.observe_phase_native_temporal_signal(
                &foundation::scene(&l1,cues[0],0)
            ));
            let checkpoint=evo.phase_native_checkpoint().unwrap();
            let mut restored=EvoPhase::new(evo.config().clone());
            assert!(restored.restore_phase_native_checkpoint(checkpoint));
            assert!(restored.phase_native_temporal_multistep_enabled());
            assert_eq!(restored.phase_native_temporal_transition_count(),depth);
            let mut synapses=Vec::new();
            for step in 0..depth {
                restored.observe_initial_real(
                    &foundation::scene(&l1,waypoints[step],0),false
                );
                let decision=restored.choose_phase_native_temporal_sensing_action()
                    .expect("variable depth physical path");
                assert_eq!(decision.action,motors[step],
                    "must select next motor in present factual state");
                synapses.push(decision.synapse);
            }
            for synapse in synapses {
                let saved=restored.perturb_phase_native_synapse_for_control(
                    synapse,0.0,0.0
                ).expect("phase-native witness");
                restored.observe_initial_real(
                    &foundation::scene(&l1,cues[0],0),false
                );
                assert!(restored.choose_phase_native_temporal_sensing_action()
                    .is_none(),"lesion of any essential link invalidates route");
                restored.restore_phase_native_synapse_for_control(
                    synapse,saved
                );
                assert!(restored.choose_phase_native_temporal_sensing_action()
                    .is_some(),"restoring the phase synapse restores route");
            }
            println!("MULTISTEP_PHYSICAL depth={} arm={} links={} checkpoint=true lesions=true samples_factual=true",
                depth,arm,restored.phase_native_temporal_transition_count());
        }
    }
}


#[test]
fn completed_goal_is_not_an_unexplored_multistep_information_frontier(){
    let (mut evo,l1)=foundation::build24();
    assert!(evo.enable_phase_native_temporal_evidence(
        PhaseTemporalEvidenceConfig{
            max_observations:8,minimum_observations:3,decisive_margin:0.125
        }
    ));
    for cue in [2usize,9usize]{
        assert!(evo.observe_phase_native_temporal_signal(
            &foundation::scene(&l1,cue,0)));
    }
    assert!(evo.begin_phase_native_temporal_episode());
    assert!(evo.observe_phase_native_temporal_signal(
        &foundation::scene(&l1,2,0)));
    assert!(evo.set_phase_native_temporal_autonomous_probe(true));
    assert!(evo.set_phase_native_temporal_multistep(true));
    let first=foundation::scene(&l1,2,0);
    let outcome=foundation::scene(&l1,10,2);
    evo.observe_initial_real(&first,false);
    let (motor,_)=evo.choose_phase_native_temporal_unknown_probe()
        .expect("a cold unfamiliar motor");
    // A partial or missing reward is insufficient to declare completion.
    assert!(!evo.observe_phase_native_temporal_full_goal_outcome(
        motor,&outcome,0.5
    ));
    assert!(evo.observe_phase_native_sensing_affordance(
        motor,&first,&outcome
    ));
    assert_eq!(evo.phase_native_temporal_rewarded_action_count(),0);
    // The successful goal is a factual externally returned 1.0,
    // not a motor label or a hidden evaluator class.
    assert!(evo.observe_phase_native_temporal_full_goal_outcome(
        motor,&outcome,1.0
    ));
    assert_eq!(evo.phase_native_temporal_rewarded_action_count(),1);
    let other=evo.choose_phase_native_temporal_unknown_probe().unwrap();
    assert_ne!(other.0,motor,
        "an achieved goal may not masquerade as an information frontier");
    let checkpoint=evo.phase_native_checkpoint().unwrap();
    let mut recovered=EvoPhase::new(evo.config().clone());
    assert!(recovered.restore_phase_native_checkpoint(checkpoint));
    assert_eq!(recovered.phase_native_temporal_rewarded_action_count(),1);
    let id=recovered.phase_native_temporal_reward_synapse(motor).unwrap();
    let saved=recovered.perturb_phase_native_synapse_for_control(
        id,0.0,0.0
    ).unwrap();
    assert_eq!(recovered.phase_native_temporal_rewarded_action_count(),0,
        "motor semantic authority must depend on physical evidence");
    recovered.restore_phase_native_synapse_for_control(id,saved);
    assert_eq!(recovered.phase_native_temporal_rewarded_action_count(),1);
    println!("MULTISTEP_GOAL_WITNESS actual_reward=true physical_lesion=true checkpoint=true no_label=true");
}


#[test]
fn factual_terminal_trials_preserve_belief_conditioned_ucb_in_multistep_carrier(){
    let (mut evo,l1)=foundation::build24();
    assert!(evo.enable_phase_native_temporal_evidence(
        PhaseTemporalEvidenceConfig{
            max_observations:8,minimum_observations:3,decisive_margin:0.125
        }
    ));
    let raw=foundation::scene(&l1,2,0);
    for cue in [2usize,9usize] {
        assert!(evo.observe_phase_native_temporal_signal(
            &foundation::scene(&l1,cue,0)
        ));
    }
    assert!(evo.begin_phase_native_temporal_episode());
    assert!(evo.set_phase_native_temporal_autonomous_probe(true));
    assert!(evo.set_phase_native_temporal_multistep(true));
    for _ in 0..3 {
        assert!(evo.observe_phase_native_temporal_signal(&raw));
    }
    assert!(evo.phase_native_temporal_evidence().unwrap().winner_cell.is_some());
    let first=evo.choose_phase_native_temporal_outcome_probe()
        .expect("opaque terminal motor under conclusive belief").0;
    let actual_post=foundation::scene(&l1,10,2);
    assert!(evo.observe_phase_native_temporal_outcome(
        first,&actual_post,0.0
    ));
    assert!(evo.observe_phase_native_sensing_affordance(
        first,&raw,&actual_post
    ));
    let next=evo.choose_phase_native_temporal_outcome_probe()
        .expect("observe untried alternative").0;
    assert_ne!(first,next,
        "factual failure must count against the executed motor in its physical belief context");
    println!("MULTISTEP_TERMINAL_CREDIT first={} next={} failed_motor_counted=true",first,next);
}
