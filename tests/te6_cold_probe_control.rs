#![allow(dead_code)]
// Generic causal control: no hidden motor labels enter EvoPhase. Unknown
// physical motor evidence is resolved by the organism using actual coverage,
// not an evaluator-side motor-try schedule. This is NOT scientific authority.
include!("intel2_unified_worlds.rs");
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

#[test]
fn te6_autonomous_motor_coverage_discovers_sensing_affordance_physically() {
    for (&cue0,&cue1) in [(2usize,9usize),(13,20),(7,21)].iter().map(|(a,b)|(a,b)) {
        let (mut evo,l1)=foundation::build24();
        assert!(evo.enable_phase_native_temporal_evidence(
            PhaseTemporalEvidenceConfig{max_observations:8,minimum_observations:3,
                decisive_margin:0.125}));
        for cue in [cue0,cue1] {
            assert!(evo.observe_phase_native_temporal_signal(
                &foundation::scene(&l1,cue,0)));
        }
        assert!(evo.begin_phase_native_temporal_episode());
        assert!(evo.set_phase_native_temporal_autonomous_probe(true));
        let first=evo.choose_phase_native_temporal_unknown_probe().unwrap();
        assert_eq!(first.1,1.0);
        assert!(evo.observe_phase_native_sensing_affordance(first.0,
            &foundation::scene(&l1,cue0,0),
            &foundation::scene(&l1,cue0,1)));
        let second=evo.choose_phase_native_temporal_unknown_probe().unwrap();
        assert_ne!(second.0,first.0,
            "lack of factual information must move next hypothesis");
        // The *physical* sensor can be any opaque motor. Only a factual
        // cue-to-cue transition earns it a conducting affordance.
        let sensor=second.0;
        assert!(evo.observe_phase_native_sensing_affordance(sensor,
            &foundation::scene(&l1,cue0,0),
            &foundation::scene(&l1,cue1,1)));
        let sensory=evo.choose_phase_native_temporal_sensing_action().unwrap();
        assert_eq!(sensory.action,sensor);
        assert!(sensory.learned_affordance>0.0);
        assert!(evo.phase_native_temporal_action_evidence_admissible(sensor));
        assert!(!evo.phase_native_temporal_action_evidence_admissible(
            (sensor+1)%6
        ),"unknown alternatives cannot masquerade as sufficiently informed");

        assert!(evo.choose_phase_native_temporal_unknown_probe().is_none());
        let checkpoint=evo.phase_native_checkpoint().unwrap();
        let mut restored=EvoPhase::new(evo.config().clone());
        assert!(restored.restore_phase_native_checkpoint(checkpoint));
        assert!(restored.phase_native_temporal_autonomous_probe());
        assert_eq!(restored.choose_phase_native_temporal_sensing_action(),Some(sensory));
        let saved=restored.perturb_phase_native_synapse_for_control(
            sensory.synapse,0.0,0.0).unwrap();
        assert!(restored.choose_phase_native_temporal_sensing_action().is_none());
        assert!(restored.choose_phase_native_temporal_unknown_probe().is_some(),
            "missing conducting affordance must reopen investigation");
        restored.restore_phase_native_synapse_for_control(sensory.synapse,saved);
        assert_eq!(restored.choose_phase_native_temporal_sensing_action(),Some(sensory));
        for _ in 0..3 {
            assert!(restored.observe_phase_native_temporal_signal(
                &foundation::scene(&l1,cue0,0)));
        }
        assert!(!restored.phase_native_temporal_evidence().unwrap().needs_more);
        assert!(restored.phase_native_temporal_action_evidence_admissible(
            (sensor+1)%6
        ),"decisive factual belief releases ordinary goal actions");
        // The newly acquired physical goal outcome, not a label or fixed
        // motor, must now determine which terminal choice is admissible.
        let terminal=(sensor+2)%6;
        let post=foundation::scene(&l1,
            if cue0==10 || cue1==10 {11} else {10},1);
        assert!(restored.observe_phase_native_temporal_outcome(
            terminal,&post,1.0
        ));
        let policy=restored.choose_phase_native_temporal_outcome_action()
            .expect("positive physically conducting reward synapse");
        assert_eq!(policy.action,terminal);
        assert!(restored.phase_native_temporal_action_evidence_admissible(terminal));
        assert!(!restored.phase_native_temporal_action_evidence_admissible(
            (terminal+1)%6
        ));
        let old=restored.perturb_phase_native_synapse_for_control(
            policy.synapse,0.0,0.0).unwrap();
        assert!(restored.choose_phase_native_temporal_outcome_action().is_none());
        assert!(restored.phase_native_temporal_action_evidence_admissible(
            (terminal+1)%6
        ),"lesion removes physically unsupported terminal policy");
        restored.restore_phase_native_synapse_for_control(policy.synapse,old);
        assert_eq!(restored.choose_phase_native_temporal_outcome_action(),Some(policy));
    }
    println!("TE6_COVERAGE unknown_probe=true physical_sensor=true lesion=true restart=true");
}
