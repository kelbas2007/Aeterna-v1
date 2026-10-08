// TE2: learning opaque observation-producing motors from factual state
// transitions. No semantic sample/commit role ever enters production EvoPhase.
#[allow(dead_code)]
mod fixture {
    include!("intel2_unified_worlds.rs");
    pub fn cold()->(EvoPhase,[[usize;2];8]){foundation::build24()}
    pub fn scene(l1:&[[usize;2];8],class:usize,layout:usize)->Vec<f32>{
        foundation::scene(l1,class,layout)
    }
}
use aeterna_v1::{EvoPhase};
use aeterna_v1::carrier::PhaseTemporalEvidenceConfig;

struct GateRng(u64);
impl GateRng {
    fn new(seed:u64)->Self{Self(seed^0x1A7E_2200_2026_1007)}
    fn next(&mut self)->u64{
        let mut x=self.0;
        x^=x>>12;x^=x<<25;x^=x>>27;self.0=x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self,n:usize)->usize{(self.next()%n as u64) as usize}
}
fn shuffle<T>(rng:&mut GateRng,x:&mut[T]){
    for i in (1..x.len()).rev(){
        let j=rng.range(i+1);x.swap(i,j);
    }
}

#[test]
fn te2_opaque_sensing_action_is_acquired_physically_in_six_permutations(){
    let mut rng=GateRng::new(0xA7E7_2202_2026_1008);
    let mut targets=[0usize,1,2,3,4,5];
    shuffle(&mut rng,&mut targets);
    for (arm,&sensor_motor) in targets.iter().enumerate() {
        let (mut evo,l1)=fixture::cold();
        let mut identities=(0..24usize).collect::<Vec<_>>();
        shuffle(&mut rng,&mut identities);
        let classes=[identities[0],identities[1]];
        let unrelated=identities[2];
        assert!(evo.enable_phase_native_temporal_evidence(
            PhaseTemporalEvidenceConfig{
                max_observations:8,
                minimum_observations:3,
                decisive_margin:0.125,
            }
        ));
        for (i,&class) in classes.iter().enumerate(){
            assert!(evo.observe_phase_native_temporal_signal(
                &fixture::scene(&l1,class,i)
            ));
        }
        assert!(evo.begin_phase_native_temporal_episode());

        // Same factual exposure count for all six opaque actions.
        // The evaluator alone knows which action produces a new cue.
        for trial in 0..8usize {
            let start=classes[trial%2];
            let from=fixture::scene(&l1,start,trial%6);
            let mut actions=[0usize,1,2,3,4,5];
            shuffle(&mut rng,&mut actions);
            for (j,&action) in actions.iter().enumerate(){
                let after=if action==sensor_motor {
                    classes[1-trial%2]
                } else if (j+trial)%2==0 {
                    start
                } else {
                    unrelated
                };
                let post=fixture::scene(&l1,after,(j+trial+1)%6);
                assert!(evo.observe_phase_native_sensing_affordance(
                    action,&from,&post
                ));
            }
        }

        let original=evo.phase_native_learned_fingerprint();
        let early=evo.phase_native_temporal_evidence().unwrap();
        assert!(early.needs_more);
        assert_eq!(early.winner_cell,None);
        assert_eq!(early.observations,0);
        // Ambiguous evidence: both source cells were genuinely observed.
        assert!(evo.observe_phase_native_temporal_signal(
            &fixture::scene(&l1,classes[0],0)
        ));
        assert!(evo.observe_phase_native_temporal_signal(
            &fixture::scene(&l1,classes[1],1)
        ));
        let readout=evo.phase_native_temporal_evidence().unwrap();
        assert!(readout.needs_more);
        assert!(readout.winner_cell.is_none());
        let before=evo.phase_native_learned_fingerprint();
        let decision=evo.choose_phase_native_temporal_sensing_action()
            .expect("factual phase-supported repeated-observation motor");
        assert_eq!(decision.action,sensor_motor,
            "opaque action identity must be inferred from PRE/action/POST");
        assert!(decision.learned_affordance>0.5);
        assert!(decision.missing_evidence>0.0);
        assert_eq!(evo.phase_native_learned_fingerprint(),before);

        // Causal intervention on the actual action->project synapse.
        let mut severed=evo.clone();
        let old=severed.perturb_phase_native_synapse_for_control(
            decision.synapse,0.0,0.0
        ).unwrap();
        assert!(severed.choose_phase_native_temporal_sensing_action().is_none());
        severed.restore_phase_native_synapse_for_control(decision.synapse,old);
        assert_eq!(severed.choose_phase_native_temporal_sensing_action(),
            Some(decision));
        assert_eq!(severed.phase_native_learned_fingerprint(),before);

        let mut shifted=evo.clone();
        let saved=shifted.perturb_phase_native_synapse_for_control(
            decision.synapse,1.0,std::f32::consts::PI
        ).unwrap();
        assert!(shifted.choose_phase_native_temporal_sensing_action().is_none());
        shifted.restore_phase_native_synapse_for_control(decision.synapse,saved);
        assert_eq!(shifted.choose_phase_native_temporal_sensing_action(),
            Some(decision));

        let checkpoint=evo.phase_native_checkpoint().unwrap();
        let mut restarted=EvoPhase::new(evo.config().clone());
        assert!(restarted.restore_phase_native_checkpoint(checkpoint));
        assert_eq!(restarted.choose_phase_native_temporal_sensing_action(),
            Some(decision));

        // Once the physical evidence becomes sufficiently decisive,
        // further sensing no longer receives a native request.
        for _ in 0..3 {
            assert!(evo.observe_phase_native_temporal_signal(
                &fixture::scene(&l1,classes[0],2)
            ));
        }
        let known=evo.phase_native_temporal_evidence().unwrap();
        assert!(known.winner_cell.is_some());
        assert!(!known.needs_more);
        assert!(evo.choose_phase_native_temporal_sensing_action().is_none());

        // And a fresh episode can again require more information, without
        // losing the lifetime-acquired physical sensor-motor affordance.
        assert!(evo.begin_phase_native_temporal_episode());
        assert!(evo.observe_phase_native_temporal_signal(
            &fixture::scene(&l1,classes[1],3)
        ));
        assert_eq!(evo.choose_phase_native_temporal_sensing_action()
            .unwrap().action,sensor_motor);
        assert_ne!(evo.phase_native_learned_fingerprint(),original);
        println!(
            "TE2_ARM arm={} opaque_sensor={} physical_confidence={:.3} lesion=PASS pi=PASS restore=PASS restart=PASS stop=PASS",
            arm,sensor_motor,decision.learned_affordance
        );
    }
    println!("TE2_SENSING_PHYSICAL 6/6");
}
