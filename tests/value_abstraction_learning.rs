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
