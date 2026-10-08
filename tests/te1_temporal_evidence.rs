// TE1 — newly introduced two-source temporal evidence accumulator.
// Uses the generic preregistered 24-class physical abstraction substrate.
// The source receives ONLY actual raw observations, never the evaluator's
// hidden majority/class bit.
#[allow(dead_code)]
mod fixture {
    include!("intel2_unified_worlds.rs");
    pub fn initial() -> (EvoPhase,[[usize;2];8]) {
        foundation::build24()
    }
    pub fn image(l1:&[[usize;2];8],class:usize,layout:usize) -> Vec<f32> {
        foundation::scene(l1,class,layout)
    }
}
use aeterna_v1::{EvoPhase};
use aeterna_v1::carrier::{PhaseTemporalEvidenceConfig};

fn qualified_episode_memory(
    evo:&mut EvoPhase,l1:&[[usize;2];8],classes:[usize;2]
)->[usize;2]{
    let cfg=PhaseTemporalEvidenceConfig{
        max_observations:8,minimum_observations:3,decisive_margin:0.125
    };
    assert!(evo.enable_phase_native_temporal_evidence(cfg));
    for (i,kind) in classes.iter().copied().enumerate(){
        assert!(evo.observe_phase_native_temporal_signal(
            &fixture::image(l1,kind,i)
        ));
    }
    let physical=evo.phase_native_temporal_evidence().unwrap();
    let ids=[
        physical.source_cells[0].unwrap(),
        physical.source_cells[1].unwrap()
    ];
    assert_ne!(ids[0],ids[1]);
    assert!(evo.begin_phase_native_temporal_episode());
    ids
}

fn supply(
    evo:&mut EvoPhase,l1:&[[usize;2];8],
    classes:[usize;2],cues:&[usize]
){
    for (i,&side) in cues.iter().enumerate(){
        assert!(evo.observe_phase_native_temporal_signal(
            &fixture::image(l1,classes[side],i%6)
        ),"factual observation {i} must enter recruited synapse");
    }
}

#[test]
fn te1_physically_accumulates_contradictory_raw_cues_and_restores(){
    let (mut evo,l1)=fixture::initial();
    let cue_classes=[13usize,4usize];
    let ids=qualified_episode_memory(&mut evo,&l1,cue_classes);
    supply(&mut evo,&l1,cue_classes,&[0,1,0,1,0,0,1,0]);
    let physical=evo.phase_native_temporal_evidence().unwrap();
    assert_eq!(physical.observations,8);
    assert_eq!(physical.winner_cell,Some(ids[0]));
    assert!(!physical.needs_more);
    let links=physical.synapses.map(|x|x.unwrap());
    assert!(physical.physical_evidence[0]>physical.physical_evidence[1]);
    let fingerprint=evo.phase_native_learned_fingerprint();
    for link in [links[0]] {
        let mut lesioned=evo.clone();
        let old=lesioned.perturb_phase_native_synapse_for_control(link,0.0,0.0)
            .expect("actual physical evidence link");
        let result=lesioned.phase_native_temporal_evidence().unwrap();
        assert_ne!(result.winner_cell,physical.winner_cell);
        lesioned.restore_phase_native_synapse_for_control(link,old);
        assert_eq!(lesioned.phase_native_learned_fingerprint(),fingerprint);
        assert_eq!(lesioned.phase_native_temporal_evidence().unwrap(),physical);

        let mut shifted=evo.clone();
        let saved=shifted.perturb_phase_native_synapse_for_control(
            link,1.0,std::f32::consts::PI
        ).unwrap();
        assert_ne!(shifted.phase_native_temporal_evidence().unwrap().winner_cell,
            physical.winner_cell);
        shifted.restore_phase_native_synapse_for_control(link,saved);
        assert_eq!(shifted.phase_native_temporal_evidence().unwrap(),physical);
    }
    let mut irrelevant=evo.clone();
    irrelevant.perturb_phase_native_synapse_for_control(
        links[1],0.0,0.0
    ).unwrap();
    assert_eq!(irrelevant.phase_native_temporal_evidence().unwrap().winner_cell,
        physical.winner_cell);
    let cp=evo.phase_native_checkpoint().unwrap();
    let mut new=EvoPhase::new(evo.config().clone());
    assert!(new.restore_phase_native_checkpoint(cp));
    assert_eq!(new.phase_native_temporal_evidence().unwrap(),physical);
    assert_eq!(evo.phase_native_temporal_evidence().unwrap(),physical);
    assert_eq!(evo.phase_native_learned_fingerprint(),fingerprint);

    assert!(evo.begin_phase_native_temporal_episode());
    let reset=evo.phase_native_temporal_evidence().unwrap();
    assert_eq!(reset.source_cells,physical.source_cells);
    assert_eq!(reset.synapses,physical.synapses);
    assert_eq!(reset.physical_evidence,[0.0,0.0]);
    assert_eq!(reset.winner_cell,None);
    assert!(reset.needs_more);
    // The same acquired cue addresses now support a *different* episode.
    supply(&mut evo,&l1,cue_classes,&[1,1,0,1,1]);
    assert_eq!(evo.phase_native_temporal_evidence().unwrap().winner_cell,
        Some(ids[1]));
    // Unregistered third raw observation cannot be mislabeled as either cue.
    let before=evo.phase_native_learned_fingerprint();
    assert!(!evo.observe_phase_native_temporal_signal(
        &fixture::image(&l1,22,4)
    ));
    assert_eq!(evo.phase_native_learned_fingerprint(),before);
    println!("TE1_PHYSICAL seq=8 winner={} lesion=true pi=true checkpoint=true reset=true unrelated=true",ids[0]);
}

#[test]
fn te1_fresh_balanced_noisy_series_vs_last_only(){
    let (mut evo,l1)=fixture::initial();
    let classes=[19usize,8usize];
    let addresses=qualified_episode_memory(&mut evo,&l1,classes);
    // This PRNG and class mapping are unrelated to FRONTIER-1's burned pack.
    let mut rng=fixture::Rng::new(0x711E_2026_1008_2026);
    let mut latent=(0..80).map(|i|i%2).collect::<Vec<_>>();
    fixture::shuffle(&mut rng,&mut latent);
    let mut full=0usize;
    let mut last_only=0usize;
    let mut abstentions=0usize;
    let mut past_majority=0usize;
    for (episode,&hidden) in latent.iter().enumerate(){
        assert!(evo.begin_phase_native_temporal_episode());
        let mut last=0usize;
        let mut ones=0usize;
        for read in 0..8 {
            let corrupted=(rng.next()%10)<3;
            last=hidden ^ usize::from(corrupted);
            ones+=last;
            assert!(evo.observe_phase_native_temporal_signal(
                &fixture::image(&l1,classes[last],(episode+read)%6)
            ));
        }
        let fingerprint=evo.phase_native_learned_fingerprint();
        let state=evo.phase_native_temporal_evidence().unwrap();
        assert_eq!(evo.phase_native_learned_fingerprint(),fingerprint);
        assert_eq!(state.observations,8);
        assert!(!state.needs_more);
        match state.winner_cell{
            Some(cell) => full+=usize::from(cell==addresses[hidden]),
            None => abstentions+=1,
        }
        last_only+=usize::from(last==hidden);
        let majority=if ones>4{Some(1)}else if ones<4{Some(0)}else{None};
        past_majority+=usize::from(majority==Some(hidden));
    }
    println!("TE1_NOISY full={full}/80 last_only={last_only}/80 majority={past_majority}/80 abstain={abstentions}/80 zero_accumulator=0");
    assert!(full>=70,"TE1 preregistered evidence integration criterion missed");
    assert!(full>=last_only+10,"TE1 preregistered last-only advantage missing");
}
