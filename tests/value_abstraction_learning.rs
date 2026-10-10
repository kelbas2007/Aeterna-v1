use aeterna_v1::carrier::PhaseNativeConfig;
use aeterna_v1::{EvoConfig, EvoPhase};

fn newborn() -> EvoPhase {
    let mut e=EvoPhase::new(EvoConfig { sensory_cells:32, motor_cells:3,
        dormant_cells:16, hdc_dim:32, ..EvoConfig::default() });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_general_policy());
    assert!(e.enable_phase_native_context_value_learning());
    assert!(e.enable_phase_native_value_abstraction());
    e
}

fn frame(bit:usize, which:usize, nuisance:usize, unseen:bool)->Vec<f32> {
    let mut raw=vec![0.0;32];
    raw[bit]=which as f32;
    for i in 0..6 {raw[16+i]=((nuisance>>i)&1) as f32;}
    raw[31]=f32::from(unseen);
    raw
}

fn train(bit:usize, reverse:bool)->EvoPhase {
    let mut e=newborn();
    // Every motor is selected by the carrier. Only factual success returns
    // reward; the acquired tree never receives the world's predicate or motor.
    for _round in 0..12 {
        for nuisance in 0..32 {
            for which in 0..2 {
                let raw=frame(bit,which,nuisance,false);
                e.begin_phase_native_general_episode();
                e.observe_phase_native_general_initial(&raw);
                for _ in 0..12 {
                    let action=e.choose_phase_native_general_action(&raw).unwrap().action;
                    let reward=f32::from(action==(which^usize::from(reverse)));
                    assert!(e.observe_phase_native_general_transition(action,&raw,&raw,reward));
                    if reward>0.0 {break;}
                }
            }
        }
    }
    e.set_planning_learning_enabled(false);
    e
}

#[test]
fn acquired_predicates_ignore_unseen_nuisance_and_follow_different_causal_bits() {
    // Two different unknown causal laws with the same constructor, including
    // reversed motors. No authored bit-0 classifier can satisfy both.
    for (bit,reverse) in [(0,false),(7,true)] {
        let mut e=train(bit,reverse);
        let (nodes,fits,support)=e.phase_native_value_abstraction_status();
        assert!((3..=255).contains(&nodes));
        assert!(fits>0 && support>=8);
        let fingerprint=e.phase_native_learned_fingerprint();
        for nuisance in 32..48 {
            for which in 0..2 {
                let raw=frame(bit,which,nuisance,true);
                e.begin_phase_native_general_episode();
                e.observe_phase_native_general_initial(&raw);
                assert_eq!(e.choose_phase_native_general_action(&raw).unwrap().action,
                    which^usize::from(reverse));
            }
        }
        assert_eq!(e.phase_native_learned_fingerprint(),fingerprint);
    }
}

#[test]
fn predicate_access_is_causal_and_checkpoint_restores_frozen_transfer() {
    let e=train(7,true);
    let checkpoint=e.phase_native_checkpoint().unwrap();
    let mut restored=newborn();
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    restored.set_planning_learning_enabled(false);
    let link=restored.phase_native_value_abstraction_link().unwrap();
    let saved=restored.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
    let mut differences=0;
    for nuisance in 32..48 {
        for which in 0..2 {
            let raw=frame(7,which,nuisance,true);
            restored.begin_phase_native_general_episode();
            restored.observe_phase_native_general_initial(&raw);
            differences+=usize::from(restored.choose_phase_native_general_action(&raw).unwrap().action!=(which^1));
        }
    }
    assert!(differences>0,"cutting the actual predicate-access link must remove transfer");
    restored.restore_phase_native_synapse_for_control(link,saved);
    let fingerprint=restored.phase_native_learned_fingerprint();
    for which in 0..2 {
        let raw=frame(7,which,47,true);
        restored.begin_phase_native_general_episode();
        restored.observe_phase_native_general_initial(&raw);
        assert_eq!(restored.choose_phase_native_general_action(&raw).unwrap().action,which^1);
    }
    assert_eq!(restored.phase_native_learned_fingerprint(),fingerprint);
}

#[test]
fn late_enable_cannot_relabel_existing_learned_states_as_raw_evidence() {
    let mut e=EvoPhase::new(EvoConfig { sensory_cells:32,motor_cells:3,dormant_cells:16,
        hdc_dim:32,..EvoConfig::default() });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    e.enable_phase_native_general_policy();
    e.enable_phase_native_context_value_learning();
    let raw=vec![0.0;32];
    assert!(e.observe_phase_native_general_transition(0,&raw,&raw,1.0));
    assert!(!e.enable_phase_native_value_abstraction());
}

#[test]
fn transferred_predicates_use_factual_past_and_lose_it_after_memory_lesion() {
    let mut e=EvoPhase::new(EvoConfig {sensory_cells:32,motor_cells:3,dormant_cells:16,
        hdc_dim:32,..EvoConfig::default()});
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    e.enable_phase_native_general_policy();
    e.enable_phase_native_developmental_memory();
    e.enable_phase_native_context_value_learning();
    e.enable_phase_native_value_abstraction();
    for _ in 0..12 {
        for nuisance in 0..16 {
            for which in 0..2 {
                let mut first=frame(0,which,nuisance,false);first[4]=1.0;
                let blank=frame(0,0,nuisance,false);
                e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&first);
                let mut raw=first;let mut prepared=false;
                for _ in 0..24 {
                    let action=e.choose_phase_native_general_action(&raw).unwrap().action;
                    let reward=f32::from(prepared && action==which);
                    let post=if prepared || action==2 {blank.clone()} else {raw.clone()};
                    e.observe_phase_native_general_transition(action,&raw,&post,reward);
                    prepared|=action==2;raw=post;
                    if reward>0.0 {break;}
                }
            }
        }
    }
    e.set_planning_learning_enabled(false);
    for nuisance in 32..40 {
        for which in 0..2 {
            let mut first=frame(0,which,nuisance,true);first[4]=1.0;
            let blank=frame(0,0,nuisance,true);
            e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&first);
            assert_eq!(e.choose_phase_native_general_action(&first).unwrap().action,2);
            e.observe_phase_native_general_transition(2,&first,&blank,0.0);
            assert_eq!(e.choose_phase_native_general_action(&blank).unwrap().action,which);
        }
    }
    let link=e.phase_native_value_memory_link().unwrap();
    e.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
    let mut actions=Vec::new();
    for which in 0..2 {
        let mut first=frame(0,which,35,true);first[4]=1.0;
        let blank=frame(0,0,35,true);
        e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&first);
        e.observe_phase_native_general_transition(2,&first,&blank,0.0);
        actions.push(e.choose_phase_native_general_action(&blank).unwrap().action);
    }
    assert_eq!(actions[0],actions[1]);
}

#[test]
fn representation_is_selected_from_acquired_evidence_without_declaring_sensor_units() {
    let mut e=newborn();
    let encode=|a:usize,b:usize,new:bool| {
        let mut raw=vec![0.0;32];
        for bit in 0..4 {raw[bit]=((a>>bit)&1) as f32;raw[4+bit]=((b>>bit)&1) as f32;}
        raw[31]=f32::from(new);raw
    };
    for _ in 0..8 {
        for a in 0..16 {
            for b in 0..16 {
                if a==b {continue;}
                let raw=encode(a,b,false);let correct=usize::from(a<b);
                e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&raw);
                for _ in 0..12 {
                    let action=e.choose_phase_native_general_action(&raw).unwrap().action;
                    let reward=f32::from(action==correct);
                    e.observe_phase_native_general_transition(action,&raw,&raw,reward);
                    if reward>0.0 {break;}
                }
            }
        }
    }
    e.set_planning_learning_enabled(false);
    let (width,cases,correct,total)=e.phase_native_value_representation_status();
    assert!((1..=8).contains(&width),"a comparison representation should outperform sparse predicates on internal validation");
    assert!(cases<=512 && correct>0 && total>0);
    let mut success=0;let mut count=0;
    for a in 0..16 {
        for b in 0..16 {
            if a==b {continue;}
            let raw=encode(a,b,true);e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&raw);
            success+=usize::from(e.choose_phase_native_general_action(&raw).unwrap().action==usize::from(a<b));count+=1;
        }
    }
    assert!(success*100>=count*90,"acquired representation must survive a previously unseen independent sensor");
}

#[test]
fn factual_effect_models_reuse_a_skill_at_new_frames_with_opaque_permuted_motors() {
    for (goal_motor,change_motor) in [(0,1),(2,0)] {
        let mut e=newborn();
        for episode in 0..512 {
            let mut raw=frame(0,episode%2,(episode/2)%32,false);
            e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&raw);
            for _ in 0..24 {
                let action=e.choose_phase_native_general_action(&raw).unwrap().action;
                let mut post=raw.clone();let mut reward=0.0;
                if action==goal_motor && raw[0]==1.0 {post[1]=1.0;reward=1.0;}
                else if action==change_motor {post[0]=1.0-raw[0];}
                e.observe_phase_native_general_transition(action,&raw,&post,reward);raw=post;
                if reward>0.0 {break;}
            }
        }
        e.set_planning_learning_enabled(false);
        let (active,nodes)=e.phase_native_value_effect_status();
        assert!(active && nodes>0 && nodes<=3*63);
        let fingerprint=e.phase_native_learned_fingerprint();
        for nuisance in 32..48 {
            for available in 0..2 {
                let raw=frame(0,available,nuisance,true);
                e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&raw);
                assert_eq!(e.choose_phase_native_general_action(&raw).unwrap().action,
                    if available==1 {goal_motor}else{change_motor});
            }
        }
        assert_eq!(e.phase_native_learned_fingerprint(),fingerprint);
    }
}

#[test]
fn unsupported_sensor_tuple_is_uncertain_and_a_factual_failed_probe_revises_only_episode_belief() {
    let mut e=EvoPhase::new(EvoConfig {sensory_cells:32,motor_cells:3,dormant_cells:16,
        hdc_dim:32,..EvoConfig::default()});
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    e.enable_phase_native_general_policy();e.enable_phase_native_developmental_memory();
    assert!(e.enable_phase_native_relational_workspace(8,1,2,2,1));
    e.enable_phase_native_context_value_learning();e.enable_phase_native_value_abstraction();
    let observed=|available:bool,nuisance:usize| {
        let mut raw=frame(0,0,nuisance,false);
        if available {raw[0]=1.0;} else {raw[1]=1.0;raw[2]=1.0;}
        raw
    };
    for episode in 0..512 {
        let mut raw=observed(episode%2==0,(episode/2)%32);
        e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&raw);
        e.phase_native_relational_initial(&raw);
        for _ in 0..24 {
            let action=e.choose_phase_native_general_action(&raw).unwrap().action;
            let mut post=raw.clone();let mut reward=0.0;
            if action==2 && raw[0]==1.0 {post[8]=1.0;reward=1.0;}
            else if action==0 {
                let next=observed(raw[0]==0.0,(episode/2)%32);
                post[0]=next[0];post[1]=next[1];post[2]=next[2];
            }
            e.observe_phase_native_general_transition(action,&raw,&post,reward);
            e.phase_native_relational_factual_post(&post);raw=post;
            if reward>0.0 {break;}
        }
    }
    e.set_planning_learning_enabled(false);
    assert!(e.phase_native_value_effect_status().0);
    // The novel tuple has the old component correlated with failure, but
    // the complete appearance has never been observed. No novel label or
    // expected result is given to cognition. It must test rather than assume.
    let mut novel=frame(0,0,35,true);novel[2]=1.0;
    e.begin_phase_native_general_episode();e.observe_phase_native_general_initial(&novel);
    e.phase_native_relational_initial(&novel);
    let fingerprint=e.phase_native_learned_fingerprint();
    assert_eq!(e.choose_phase_native_general_action(&novel).unwrap().action,2);
    // This actual intervention fails to change the sensed frame. Only the
    // episode belief is revised; frozen knowledge is not silently trained.
    assert!(e.observe_phase_native_general_transition(2,&novel,&novel,0.0));
    e.phase_native_relational_factual_post(&novel);
    assert_eq!(e.choose_phase_native_general_action(&novel).unwrap().action,0);
    assert_eq!(e.phase_native_learned_fingerprint(),fingerprint);
}
