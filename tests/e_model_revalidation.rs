// Prospective generic physical controls for EvoPhase's factual-model
// revalidation and competing goal-path coverage. Uses the already-qualified
// perceptual substrate, not any World-E law, action assignment or switch flag.
#[allow(dead_code)]
mod fixture {
    include!("intel2_unified_worlds.rs");
    pub fn build() -> (EvoPhase, [[usize;2];8]) { foundation::build24() }
    pub fn raster(l1:&[[usize;2];8], state:usize) -> Vec<f32> {
        foundation::scene(l1,state,0)
    }
    pub fn meta() -> PhaseMetaControlCheckpoint { meta_checkpoint() }
}
use aeterna_v1::carrier::PhaseHypothesisEcologyConfig;
use aeterna_v1::scientific_runtime::ScientificRuntime;

fn unify(evo:aeterna_v1::EvoPhase, meta:aeterna_v1::carrier::PhaseMetaControlCheckpoint,
    raw:&[f32],goal:&[f32])->ScientificRuntime {
    let mut rt=ScientificRuntime::new(evo).unwrap();
    assert!(rt.enable_unified_cognition(
        meta, PhaseHypothesisEcologyConfig {
            learning_rate:0.35, dormancy_threshold:0.05,
            learning_enabled:true, phase_learning_enabled:true
        }
    ));
    rt.observe_external(raw).unwrap();
    rt.set_goal(goal).unwrap();
    rt
}

#[test]
fn e_model_revalidation_depends_on_factual_local_support(){
    let (mut evo,l1)=fixture::build();
    let raw=fixture::raster(&l1,0);
    let goal=fixture::raster(&l1,7);
    for motor in 0..6 {
        assert!(evo.observe_phase_native_abstract_transition(&raw,motor,&raw,0.0));
    }
    let meta=fixture::meta();
    let baseline=unify(evo.clone(),meta.clone(),&raw,&goal);
    let first=baseline.organism().collect_phase_native_unified_proposals(&goal);
    let picked=baseline.organism().choose_phase_native_unified_proposal(&first)
        .expect("bounded native revalidation must prevent exhausted-model stall");
    assert_eq!(first.len(),1,"fully modelled dead region must not invent routes");
    assert!(first[0].proposal.fields[1] > 0.0);
    assert_eq!(picked.action,first[0].proposal.action);
    let first_motor=picked.action;

    // Add factual support to the initially least-supported motor.
    // No goal link, task tag, feature answer or external planner is inserted.
    for _ in 0..5 {
        assert!(evo.observe_phase_native_abstract_transition(
            &raw,first_motor,&raw,0.0
        ));
    }
    let changed=unify(evo,meta,&raw,&goal);
    let next=changed.organism().collect_phase_native_unified_proposals(&goal);
    assert_eq!(next.len(),1);
    assert_ne!(next[0].proposal.action, first_motor,
        "new physical evidence must change the preferred revalidation target");

    let original=baseline.organism().clone();
    let cell=original.phase_native_abstract_state(&raw).unwrap().cell;
    let linked=original.phase_native_circuits().iter().find(|c|{
        let aff=original.phase_native_synapse(c.afferent_synapse).unwrap();
        let motor=original.phase_native_synapse(c.motor_synapse).unwrap();
        aff.from==cell && motor.to==original.config().sensory_cells+first_motor
    }).expect("physical state-action record");
    let link=linked.afferent_synapse;
    let mut lesioned=original.clone();
    let saved=lesioned.perturb_phase_native_synapse_for_control(link,0.0,0.0).unwrap();
    let altered=lesioned.collect_phase_native_unified_proposals(&goal);
    assert!(!altered.iter().any(|p|p.proposal.proposal_id==first[0].proposal.proposal_id),
        "damaged native transition must not masquerade as a validated recheck");
    lesioned.restore_phase_native_synapse_for_control(link,saved);
    assert_eq!(lesioned.phase_native_learned_fingerprint(),
        original.phase_native_learned_fingerprint());
    assert_eq!(lesioned.collect_phase_native_unified_proposals(&goal),first);
    println!("E_REVALIDATION_GENERIC initial={} after_support={} causal=PASS restore=PASS",
        first_motor,next[0].proposal.action);
}

#[test]
fn e_goal_route_coverage_comes_from_acquired_physical_successors(){
    let (mut evo,l1)=fixture::build();
    let start=fixture::raster(&l1,12);
    let mid_a=fixture::raster(&l1,13);
    let mid_b=fixture::raster(&l1,14);
    let goal=fixture::raster(&l1,15);

    // Two task-independent factual routes of equal length. The first
    // acquires much greater confidence than the other; other motors merely
    // return to the same base state.
    for _ in 0..8 {
        assert!(evo.observe_phase_native_abstract_transition(&start,0,&mid_a,0.0));
        assert!(evo.observe_phase_native_abstract_transition(&mid_a,4,&goal,0.0));
    }
    assert!(evo.observe_phase_native_abstract_transition(&start,3,&mid_b,0.0));
    assert!(evo.observe_phase_native_abstract_transition(&mid_b,5,&goal,0.0));
    for motor in [1,2,4,5] {
        assert!(evo.observe_phase_native_abstract_transition(
            &start,motor,&start,0.0
        ));
    }

    let rt=unify(evo,fixture::meta(),&start,&goal);
    let proposals=rt.organism().collect_phase_native_unified_proposals(&goal);
    let candidate=proposals.iter().find(|p|{
        p.proposal.action==3
            && p.persistent_candidate_id.is_none()
            && p.proposal.fields[0]>0.01
            && p.proposal.fields[1]>0.01
    }).expect("under-covered physical route must have benefit and evidence debt");
    let expected=candidate.proposal;
    let original=rt.organism().clone();
    let base=original.phase_native_abstract_state(&start).unwrap().cell;
    let mid=original.phase_native_abstract_state(&mid_b).unwrap().cell;
    let circuit=original.phase_native_circuits().iter().find(|c|{
        let aff=original.phase_native_synapse(c.afferent_synapse).unwrap();
        let motor=original.phase_native_synapse(c.motor_synapse).unwrap();
        let succ=original.phase_native_synapse(c.successor_synapse).unwrap();
        aff.from==base && succ.to==mid
          && motor.to==original.config().sensory_cells+3
    }).expect("learned competing route must be physically addressed");
    let mut lesion=original.clone();
    let old=lesion.perturb_phase_native_synapse_for_control(
        circuit.successor_synapse,0.0,0.0
    ).unwrap();
    assert!(!lesion.collect_phase_native_unified_proposals(&goal).iter()
        .any(|p|p.proposal.proposal_id==expected.proposal_id
            && p.proposal.fields[0]>=expected.fields[0]
            && p.proposal.fields[1]>=expected.fields[1]));
    lesion.restore_phase_native_synapse_for_control(circuit.successor_synapse,old);
    let mut phase_shifted=original.clone();
    let phase_saved=phase_shifted.perturb_phase_native_synapse_for_control(
        circuit.successor_synapse,1.0,std::f32::consts::PI
    ).unwrap();
    assert!(!phase_shifted.collect_phase_native_unified_proposals(&goal)
        .iter().any(|p|p.proposal.proposal_id==expected.proposal_id
            && p.proposal.fields[0]>=expected.fields[0]
            && p.proposal.fields[1]>=expected.fields[1]),
        "pi shifted native route must not retain the same goal-valued proposal");
    phase_shifted.restore_phase_native_synapse_for_control(
        circuit.successor_synapse,phase_saved
    );
    assert_eq!(phase_shifted.phase_native_learned_fingerprint(),
        original.phase_native_learned_fingerprint());
    assert_eq!(phase_shifted.collect_phase_native_unified_proposals(&goal),
        proposals);
    assert_eq!(lesion.phase_native_learned_fingerprint(),
        original.phase_native_learned_fingerprint());
    assert_eq!(lesion.collect_phase_native_unified_proposals(&goal),proposals);
    println!("E_COVERAGE_GENERIC alternative=3 local_physical_support=PASS lesion=PASS restore=PASS");
}
