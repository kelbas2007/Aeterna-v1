use aeterna_v1::carrier::PhaseNativeConfig;
use aeterna_v1::{EvoConfig, EvoPhase};

fn newborn(memory: bool) -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: 32,
        motor_cells: 3,
        dormant_cells: 16,
        hdc_dim: 32,
        ..EvoConfig::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_general_policy());
    if memory {
        assert!(e.enable_phase_native_developmental_memory());
    }
    assert!(e.enable_phase_native_context_value_learning());
    e
}

fn cue(which: usize) -> Vec<f32> {
    (0..32).map(|i| f32::from(i % 4 == which)).collect()
}

// The evaluator defines a world; the carrier picks every motor itself.
// A preparatory motor exposes an identical blank decision frame. Only
// the previously seen cue determines which final action actually rewards.
fn train(memory: bool) -> EvoPhase {
    let mut e = newborn(memory);
    let blank = vec![0.0; 32];
    for episode in 0..160 {
        let which = episode % 2;
        let first = cue(which);
        e.begin_phase_native_general_episode();
        assert!(e.observe_phase_native_general_initial(&first));
        let mut observed = first;
        let mut prepared = false;
        for _ in 0..16 {
            let action = e
                .choose_phase_native_general_action(&observed)
                .unwrap()
                .action;
            let reward = if prepared && action == which {
                1.0
            } else {
                0.0
            };
            let post = if action == 2 || prepared {
                blank.clone()
            } else {
                observed.clone()
            };
            assert!(e.observe_phase_native_general_transition(action, &observed, &post, reward));
            prepared |= action == 2;
            observed = post;
            if reward > 0.0 {
                break;
            }
        }
    }
    e.set_planning_learning_enabled(false);
    e
}

#[test]
fn self_selected_delayed_reward_changes_action_with_earlier_cue() {
    let mut e = train(true);
    let mut actions = Vec::new();
    for which in 0..2 {
        e.begin_phase_native_general_episode();
        let first = cue(which);
        assert!(e.observe_phase_native_general_initial(&first));
        assert_eq!(
            e.choose_phase_native_general_action(&first).unwrap().action,
            2
        );
        assert!(e.observe_phase_native_general_transition(2, &first, &vec![0.0; 32], 0.0));
        actions.push(
            e.choose_phase_native_general_action(&vec![0.0; 32])
                .unwrap()
                .action,
        );
    }
    assert_eq!(
        actions,
        vec![0, 1],
        "same present, different factual past must change action"
    );

    let link = e.phase_native_value_memory_link().unwrap();
    let saved = e
        .perturb_phase_native_synapse_for_control(link, 0.0, 0.0)
        .unwrap();
    let mut lesioned = Vec::new();
    for which in 0..2 {
        e.begin_phase_native_general_episode();
        let first = cue(which);
        e.observe_phase_native_general_initial(&first);
        e.observe_phase_native_general_transition(2, &first, &vec![0.0; 32], 0.0);
        lesioned.push(
            e.choose_phase_native_general_action(&vec![0.0; 32])
                .unwrap()
                .action,
        );
    }
    assert_eq!(
        lesioned[0], lesioned[1],
        "memory lesion must destroy history dependence"
    );
    e.restore_phase_native_synapse_for_control(link, saved);
}

#[test]
fn checkpoint_keeps_knowledge_and_frozen_episodes_do_not_train() {
    let e = train(true);
    let count = e.phase_native_value_state_count();
    let checkpoint = e.phase_native_checkpoint().unwrap();
    let mut restored = newborn(true);
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    assert_eq!(restored.phase_native_value_state_count(), count);
    restored.set_planning_learning_enabled(false);
    let fingerprint = restored.phase_native_learned_fingerprint();
    restored.begin_phase_native_general_episode();
    let first = cue(1);
    restored.observe_phase_native_general_initial(&first);
    restored.observe_phase_native_general_transition(2, &first, &vec![0.0; 32], 0.0);
    assert_eq!(
        restored
            .choose_phase_native_general_action(&vec![0.0; 32])
            .unwrap()
            .action,
        1
    );
    assert_eq!(restored.phase_native_learned_fingerprint(), fingerprint);
    for motor in 0..3 {
        let link = restored.phase_native_general_synapse(motor).unwrap();
        restored
            .perturb_phase_native_synapse_for_control(link, 0.0, 0.0)
            .unwrap();
    }
    assert!(restored
        .choose_phase_native_general_action(&first)
        .is_none());
}

#[test]
fn learned_state_storage_is_bounded_and_frozen_new_inputs_do_not_allocate() {
    let mut e = newborn(false);
    let encode = |number: usize| {
        (0..32)
            .map(|bit| ((number >> bit) & 1) as f32)
            .collect::<Vec<_>>()
    };
    for state in 0..2200 {
        assert!(e.observe_phase_native_general_transition(
            0,
            &encode(state),
            &encode(state + 1),
            0.0
        ));
    }
    assert_eq!(e.phase_native_value_state_count(), 2048);
    // Revisit the oldest surviving state while inserting an unseen POST:
    // capacity eviction must not invalidate the active factual PRE address.
    assert!(e.observe_phase_native_general_transition(0,&encode(153),&encode(9000),0.0));
    assert_eq!(e.phase_native_value_state_count(),2048);
    e.set_planning_learning_enabled(false);
    let fingerprint = e.phase_native_learned_fingerprint();
    e.begin_phase_native_general_episode();
    e.observe_phase_native_general_initial(&encode(9000));
    e.observe_phase_native_general_transition(1, &encode(9000), &encode(9001), 0.0);
    assert_eq!(e.phase_native_value_state_count(), 2048);
    assert_eq!(e.phase_native_learned_fingerprint(), fingerprint);
}

#[test]
fn frozen_policy_recovers_when_a_previously_rewarded_action_now_stalls() {
    let mut e = newborn(false);
    let observed = cue(0);
    // Factual former world: opaque motor 0 completes the task.
    for _ in 0..24 {
        e.begin_phase_native_general_episode();
        e.observe_phase_native_general_initial(&observed);
        e.observe_phase_native_general_transition(0, &observed, &observed, 1.0);
    }
    e.set_planning_learning_enabled(false);
    e.begin_phase_native_general_episode();
    e.observe_phase_native_general_initial(&observed);
    let fingerprint = e.phase_native_learned_fingerprint();
    assert_eq!(
        e.choose_phase_native_general_action(&observed)
            .unwrap()
            .action,
        0
    );
    // Factual changed world: that old motor no longer changes anything.
    // The evaluator supplies neither the correct replacement nor a route.
    let mut escaped = false;
    for _ in 0..32 {
        let action = e
            .choose_phase_native_general_action(&observed)
            .unwrap()
            .action;
        let post = if action == 0 {
            observed.clone()
        } else {
            cue(1)
        };
        e.observe_phase_native_general_transition(action, &observed, &post, 0.0);
        if action != 0 {
            escaped = true;
            break;
        }
    }
    assert!(
        escaped,
        "a frozen model must not freeze a lifetime in a repeated action loop"
    );
    assert_eq!(e.phase_native_learned_fingerprint(), fingerprint);
}
