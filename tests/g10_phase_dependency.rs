use aeterna_v1::{ConceptConfig, EvoConceptMemory, EvoConfig, EvoPhase};

#[derive(Clone, Copy)]
enum K { A, B, C, D }

fn delta(k: K) -> (usize, usize) {
    match k {
        K::A => (1,0),
        K::B => (0,1),
        K::C => (1,1),
        K::D => (2,0),
    }
}

fn cases() -> [(K,K,bool);4] {
    [(K::A,K::B,false),(K::C,K::D,false),(K::A,K::C,true),(K::B,K::D,true)]
}

fn scene(a: K, b: K, left: (usize,usize), right: (usize,usize)) -> Vec<f32> {
    let mut r=vec![0.0f32;144];
    for (k,(x,y)) in [(a,left),(b,right)] {
        let (dx,dy)=delta(k);
        r[y*12+x]=1.0;
        r[(y+dy)*12+x+dx]=1.0;
    }
    r
}

fn expected(class_one: bool, swap: bool) -> usize {
    usize::from(class_one ^ swap)
}

fn train_memory(swap: bool) -> EvoConceptMemory {
    let mut cfg=ConceptConfig::for_raster(12,12,2,192);
    cfg.readout_enabled=false;
    let mut memory=EvoConceptMemory::new(cfg);
    let bindings=[
        ((1usize,1usize),(8usize,8usize)),
        ((2,1),(7,7)),
        ((1,3),(8,5)),
        ((3,1),(6,7)),
    ];
    for (l,r) in bindings {
        for (a,b,class_one) in cases() {
            let raster=scene(a,b,l,r);
            for action in 0..2 {
                memory.observe_factual(&raster,action,action==expected(class_one,swap));
            }
        }
    }
    memory.set_learning_enabled(false);
    memory.set_readout_enabled(true);
    memory
}

fn train_zero_physical(swap: bool) -> EvoPhase {
    let cfg=EvoConfig {
        sensory_cells:144,
        motor_cells:2,
        dormant_cells:0,
        hdc_dim:192,
        weight_learning_rate:0.0,
        phase_learning_rate:0.0,
        structural_growth_enabled:false,
        ..EvoConfig::default()
    };
    let mut evo=EvoPhase::new(cfg);
    let mut concept=ConceptConfig::for_raster(12,12,2,192);
    concept.readout_enabled=false;
    evo.enable_concept_memory(concept);
    let bindings=[
        ((1usize,1usize),(8usize,8usize)),
        ((2,1),(7,7)),
        ((1,3),(8,5)),
        ((3,1),(6,7)),
    ];
    for (l,r) in bindings {
        for (a,b,class_one) in cases() {
            let raster=scene(a,b,l,r);
            for action in 0..2 {
                evo.observe_concept_factual(&raster,action,action==expected(class_one,swap));
            }
        }
    }
    evo.set_concept_learning_enabled(false);
    evo.set_concept_readout_enabled(true);
    evo
}

fn heldout_score_memory(memory: &EvoConceptMemory, swap: bool) -> usize {
    let layouts=[
        ((3usize,1usize),(6usize,8usize)),
        ((1,4),(8,2)),
        ((3,3),(7,7)),
        ((2,4),(7,1)),
    ];
    cases().into_iter().zip(layouts).map(|((a,b,class_one),(l,r))| {
        usize::from(memory.choose_composite_action(&scene(a,b,l,r))==Some(expected(class_one,swap)))
    }).sum()
}

fn heldout_score_evo(evo: &EvoPhase, swap: bool) -> usize {
    let layouts=[
        ((3usize,1usize),(6usize,8usize)),
        ((1,4),(8,2)),
        ((3,3),(7,7)),
        ((2,4),(7,1)),
    ];
    cases().into_iter().zip(layouts).map(|((a,b,class_one),(l,r))| {
        usize::from(evo.choose_composite_concept_action(&scene(a,b,l,r))==Some(expected(class_one,swap)))
    }).sum()
}

#[test]
fn current_g10_readout_does_not_require_phase_cell_network_execution() {
    let mut standalone_total=0usize;
    let mut zero_physical_total=0usize;

    for swap in [false,true] {
        let standalone=train_memory(swap);
        let zero=train_zero_physical(swap);

        assert_eq!(standalone.atoms().len(),4);
        assert_eq!(standalone.composites().len(),4);
        assert_eq!(zero.recruited_relays(),0);
        assert_eq!(zero.branches().len(),0);

        standalone_total+=heldout_score_memory(&standalone,swap);
        zero_physical_total+=heldout_score_evo(&zero,swap);
    }

    println!(
        "G10_PHASE_AUDIT standalone={}/8 zero_physical={}/8 verdict=NEGATIVE_PHASE_DEPENDENCY_WITNESS",
        standalone_total, zero_physical_total
    );

    assert_eq!(standalone_total,8);
    assert_eq!(zero_physical_total,8);
}
