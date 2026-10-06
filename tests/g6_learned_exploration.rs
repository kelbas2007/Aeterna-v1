mod g6_support;

use aeterna_v1::ExplorationConfig;
use g6_support::{
    actual_post, all_orders, carrier, install_world_models, probes_for_order, relation, train_strategy,
};

#[test]
fn g6_learns_exploration_strategy_and_transfers_to_three_rival_family() {
    let mut evo = carrier();
    let tuition_cost = train_strategy(&mut evo);

    assert_eq!(tuition_cost, 32, "tuition physical probe cost must be explicit");
    assert_eq!(evo.exploration_observations(), 32);

    let weights = evo.exploration_weights().expect("strategy state");
    assert!(
        weights[0] > 0.10,
        "factual information gain must learn positive value for prediction disagreement"
    );

    evo.set_exploration_learning_enabled(false);
    evo.set_exploration_readout_enabled(true);

    // New family after strategy learning is frozen: three rivals instead of two.
    for world in 0..8usize {
        let family = 100 + world;
        let informative = (world + 1) % 4;
        let partial = (informative + 1) % 4;
        let pre_code = 5_000 + world as u64;
        install_world_models(
            &mut evo,
            family,
            pre_code,
            informative,
            Some(partial),
            3,
        );
    }

    let mut learned_first_correct = 0usize;
    let mut zero_first_correct = 0usize;
    let mut oracle_first_correct = 0usize;
    let mut learned_probe_sum = 0usize;
    let mut random_probe_sum = 0usize;
    let mut random_cases = 0usize;

    for world in 0..8usize {
        let family = 100 + world;
        let informative = (world + 1) % 4;
        let partial = (informative + 1) % 4;
        let pre_code = 5_000 + world as u64;
        let law = world % 3;
        let pre = relation(pre_code, 7, 3);

        // Learned path: no direct G2 disagreement-argmax call.
        let mut learned = evo.clone();
        assert_eq!(learned.begin_epistemic_episode(&pre), 3);
        let action = learned
            .choose_learned_exploration_probe()
            .expect("learned strategy readout");
        learned_first_correct += usize::from(action == informative);

        let post = actual_post(family, action, law, informative, Some(partial), 3);
        let remaining = learned.observe_epistemic_probe(action, &post);
        learned_probe_sum += if remaining == 1 { 1 } else { 2 };

        // Same world models, but strategy state reset to zero.
        let mut zero = evo.clone();
        zero.enable_exploration_strategy(ExplorationConfig {
            learning_rate: 0.18,
            learning_enabled: false,
            readout_enabled: true,
        });
        assert_eq!(zero.begin_epistemic_episode(&pre), 3);
        let zero_action = zero
            .choose_learned_exploration_probe()
            .expect("zero strategy readout");
        zero_first_correct += usize::from(zero_action == informative);

        // Direct disagreement is a diagnostic ceiling only.
        let mut oracle = evo.clone();
        assert_eq!(oracle.begin_epistemic_episode(&pre), 3);
        let oracle_action = oracle.choose_epistemic_probe(true);
        oracle_first_correct += usize::from(oracle_action == informative);

        // Strong random-order baseline: all 24 permutations.
        for order in all_orders() {
            random_probe_sum += probes_for_order(
                &evo,
                &pre,
                family,
                informative,
                partial,
                law,
                order,
            );
            random_cases += 1;
        }
    }

    let learned_mean = learned_probe_sum as f64 / 8.0;
    let random_mean = random_probe_sum as f64 / random_cases as f64;

    assert!(
        learned_first_correct >= 7,
        "learned strategy must choose the maximally informative probe first in >=7/8"
    );
    assert!(
        zero_first_correct < learned_first_correct,
        "zero-strategy matched control must be strictly worse"
    );
    assert!(
        oracle_first_correct >= learned_first_correct,
        "learned strategy cannot exceed the direct-disagreement oracle ceiling"
    );
    assert!(
        learned_mean < random_mean,
        "learned strategy must beat mean physical probe cost across all random orders"
    );
}
