#!/usr/bin/env bash
# Ordinary regressions, including causal controls. One-use authority packs
# and explicitly negative research diagnostics have separate workflows.
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
sha256sum --check tests/data/iris.sha256
sha256sum --check tests/data/digits.sha256
sha256sum --check tests/data/gunpoint.sha256

cargo test --locked --release --lib \
    --test online_learning --test online_rules --test expanded_rules --test partial_observation \
    --test inverse_inference --test adaptive_rules --test uncertain_learning --test iris_grounded \
    --test vector_learning --test real_digits --test perception_cli \
    --test induced_programs --test induction_mechanism \
    --test acquired_primitives --test real_programs \
    --test phase_native_execution --test phase_forward_execution \
    --test phase_native_autonomy --test phase_native_continual \
    --test g20_continual_reasoning --test g21_contextual_state_refinement \
    --test g22_perceptual_variable_invention --test u1_meta_control \
    --test u2_hypothesis_ecology --test u3_cross_mechanism_allocation --test human_protection
cargo test --locked --release --test e_model_revalidation e_model_revalidation_depends_on_factual_local_support
cargo test --locked --release --test e_model_revalidation e_goal_route_coverage_comes_from_acquired_physical_successors
cargo test --locked --release --test composition_succession composition_replaces_rejected
cargo test --locked --release --test g23_compositional_perceptual_function g23_synthesizes_composed
cargo test --locked --release --test unified_runtime_integration unified_runtime_
cargo test --locked --release --test intel2_unified_worlds intel2_evaluator_uses_only_unified_external_runtime -- --exact
cargo test --locked --release --test intel4_unified_worlds intel4_evaluator_uses_only_unified_external_runtime -- --exact
cargo test --locked --release --test intel2_refined_state_controls refined_state_controls_preserve_context_through_unified_competition -- --exact
cargo test --locked --release --test te1_temporal_evidence te1_physically_accumulates_contradictory_raw_cues_and_restores -- --exact
cargo test --locked --release --test te2_sensing_affordance te2_opaque_sensing_action_is_acquired_physically_in_six_permutations -- --exact
cargo test --locked --release --test te3_unified_sensing te3_
cargo test --locked --release --test te5_belief_action te5_six_independent_cue_motor_mappings_are_learned_and_physically_necessary -- --exact
cargo test --locked --release --test te5_unified_decisions te5_unified_selects_acquired_sensing_then_belief_grounded_goal_motor -- --exact
cargo test --locked --release --test u1_live_credit u1_online_credit_depends_on_protected_factual_sensing_and_freezes -- --exact
cargo test --locked --release --test te6_cold_probe_control te6_autonomous_motor_coverage_discovers_sensing_affordance_physically -- --exact
cargo test --locked --release --test te6_cold_probe_control te6_terminal_experiment_moves_away_from_factually_checked_motor -- --exact
cargo test --locked --release --test te6_cold_probe_control te6_rival_physical_outcomes_make_organism_seek_more_evidence -- --exact
cargo test --locked --release --test te6_cold_probe_control te6_decisive_reward_does_not_create_unsupported_rival_policy -- --exact
cargo test --locked --release --test u1_cold_lifetime u1_new_cold_no_curriculum_continual_lifetime_development -- --exact --nocapture
cargo test --locked --release --test structure1_chain_mechanism chain_motor_affordance_is_physical_contextual_and_checkpointed -- --exact
cargo test --locked --release --test multistep_physical_path physical_sensing_paths_of_variable_depth_are_reused_and_lesionable -- --exact
cargo test --locked --release --test multistep_physical_path completed_goal_is_not_an_unexplored_multistep_information_frontier -- --exact
cargo test --locked --release --test multistep_physical_path factual_terminal_trials_preserve_belief_conditioned_ucb_in_multistep_carrier -- --exact
# New goal-replanning mechanism: learned factual transition graph, physical
# synapse lesion, protected 4-action goal and changed-law 6-action recovery.
# Frozen first-attempt fresh seed remains in its separate one-use workflow.
cargo test --locked --release --test goal_replan_physical goal_replan_composes_physical_routes_after_unannounced_causal_drift -- --exact
python3 scripts/g21_statistical_crosscheck.py
python3 scripts/check_docs.py
cargo build --locked --release --bins --examples
