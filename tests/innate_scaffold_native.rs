use aeterna_v1::{EvoConfig,EvoPhase};
use aeterna_v1::carrier::PhaseNativeConfig;

fn cold() -> EvoPhase {
    let mut organism=EvoPhase::new(EvoConfig {
        sensory_cells:16,motor_cells:3,dormant_cells:32,
        hdc_dim:32,..EvoConfig::default()
    });
    organism.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(organism.enable_phase_native_innate_scaffold());
    organism
}

#[test]
fn new_organism_has_learning_priors_not_memories_or_object_labels(){
    let mut organism=cold();
    let at_birth=organism.phase_native_innate_readout().unwrap();
    assert!(at_birth.continuity_prior>0.0);
    assert!(at_birth.orienting_prior>0.0);
    assert!(at_birth.agency_prior>0.0);
    assert!(at_birth.habituation_prior>0.0);
    assert!(at_birth.approximate_magnitude_prior>0.0);
    assert!(at_birth.regulation_prior>0.0);
    assert!(at_birth.social_readiness>0.0);
    assert!(at_birth.speech_readiness>0.0);
    assert_eq!(at_birth.observed_events,0);
    assert_eq!(at_birth.acquired_associations,0);
    assert_eq!(organism.phase_native_grounded_words(),0);
    assert_eq!(organism.phase_native_grounded_categories(),0);
    assert!(organism.phase_native_innate_action_bias(0)>0.0);

    // Disabling ACTUAL attention synapse causally removes novelty drive.
    let synapse=organism.phase_native_innate_synapse(1).unwrap();
    let before=organism.phase_native_innate_action_bias(0);
    let original=organism.perturb_phase_native_synapse_for_control(
        synapse,0.0,0.0).unwrap();
    let disabled=organism.phase_native_innate_action_bias(0);
    assert!(disabled<before);
    organism.restore_phase_native_synapse_for_control(synapse,original);
    assert_eq!(organism.phase_native_innate_action_bias(0),before);
}

#[test]
fn factual_effects_change_agency_and_checkpoint_preserves_only_learned_knowledge(){
    let mut organism=cold();
    let quiet=vec![0.0;16];
    let moving=vec![1.0;16];
    assert!(organism.begin_phase_native_innate_episode(&quiet));
    assert!(organism.observe_phase_native_innate_contingency(1,&quiet,&moving));
    assert!(organism.observe_phase_native_innate_contingency(1,&quiet,&moving));
    assert!(organism.observe_phase_native_innate_contingency(2,&quiet,&quiet));
    let acquired=organism.phase_native_innate_readout().unwrap();
    assert_eq!(acquired.observed_events,3);
    assert_eq!(acquired.acquired_associations,2);
    assert!(organism.phase_native_innate_action_bias(1)
        >organism.phase_native_innate_action_bias(2));

    let checkpoint=organism.phase_native_checkpoint().unwrap();
    let mut resumed=EvoPhase::new(organism.config().clone());
    assert!(resumed.restore_phase_native_checkpoint(checkpoint));
    assert_eq!(resumed.phase_native_innate_readout().unwrap().observed_events,3);
    assert_eq!(resumed.phase_native_innate_readout().unwrap().acquired_associations,2);

    resumed.set_planning_learning_enabled(false);
    let signature=resumed.phase_native_learned_fingerprint();
    assert!(resumed.begin_phase_native_innate_episode(&moving));
    assert!(resumed.observe_phase_native_innate_contingency(1,&moving,&quiet));
    assert_eq!(resumed.phase_native_learned_fingerprint(),signature);
    assert_eq!(resumed.phase_native_innate_readout().unwrap().observed_events,3);
    println!("INNATE_SCAFFOLD_PASS initial_empty=true innate_priors=8 learned_action_effects=2 checkpoint=true frozen=true attention_lesion=true");
}
