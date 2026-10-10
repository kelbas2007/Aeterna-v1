//! Grounded object memory: unseen room/layout, same visual pattern,
 //! deictic lexical teaching, checkpoint, negative control and phase lesion.
use aeterna_v1::{EvoConfig, EvoPhase};
use aeterna_v1::carrier::PhaseNativeConfig;

const W: usize = 7;
const H: usize = 7;
const DIM: usize = W * H * 3 * 4;
fn put(raw: &mut [f32], tile: usize, kind: u8, color: u8, state: u8) {
    for (channel, value) in [kind, color, state].into_iter().enumerate() {
        for bit in 0..4 {
            raw[tile * 12 + channel * 4 + bit] = ((value >> bit) & 1) as f32;
        }
    }
}
fn frame(tile: usize, object: (u8, u8, u8), decoy: (usize, u8, u8, u8)) -> Vec<f32> {
    let mut v = vec![0.0; DIM];
    put(&mut v, tile, object.0, object.1, object.2);
    put(&mut v, decoy.0, decoy.1, decoy.2, decoy.3);
    v
}

#[test]
fn visual_referent_persists_across_worlds_and_causal_physical_word_link() {
    let mut original = EvoPhase::new(EvoConfig {
        sensory_cells: DIM, motor_cells: 7, dormant_cells: 128,
        hdc_dim: 64, ..EvoConfig::default()
    });
    original.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(original.enable_phase_native_grounded_objects(7, 7, 3, 4, 2));

    // A genuinely external teacher points at tile 9 and says an
    // arbitrary token. No object type, action rule or semantic ID supplied.
    let learn = frame(9, (5, 2, 0), (22, 4, 2, 0));
    original.observe_initial_real(&learn, false);
    assert!(original.observe_phase_native_object_view(&learn));
    assert!(original.teach_phase_native_pointed_word("kiva", 9));
    assert_eq!(original.phase_native_grounded_words(), 1);
    assert_eq!(original.phase_native_grounded_frames(), 1);

    // World B: same appearance at another position, different dynamic
    // state, different distractor. No new lesson supplied.
    let transfer = frame(37, (5, 2, 3), (4, 6, 3, 2));
    assert_eq!(
        original.locate_phase_native_grounded_word("kiva", &transfer)
            .iter().map(|r| r.tile_index).collect::<Vec<_>>(),
        vec![37]
    );
    assert!(original.locate_phase_native_grounded_word("unknown", &transfer).is_empty());

    // Negative control: changed visual category must not inherit word.
    let mismatch = frame(37, (5, 3, 3), (4, 6, 3, 2));
    assert!(original.locate_phase_native_grounded_word("kiva", &mismatch).is_empty());

    let checkpoint = original.phase_native_checkpoint().unwrap();
    let mut restored = EvoPhase::new(original.config().clone());
    assert!(restored.restore_phase_native_checkpoint(checkpoint));
    assert!(restored.current_real().is_none(), "no fictional factual frame after restart");
    assert_eq!(
        restored.locate_phase_native_grounded_word("kiva", &transfer)
            .iter().map(|r| r.tile_index).collect::<Vec<_>>(),
        vec![37]
    );
    let synapse = restored.phase_native_grounded_word_synapse("kiva").unwrap();
    let saved = restored.perturb_phase_native_synapse_for_control(
        synapse, 0.0, 0.0).unwrap();
    assert!(restored.locate_phase_native_grounded_word("kiva", &transfer).is_empty());
    restored.restore_phase_native_synapse_for_control(synapse, saved);
    assert_eq!(restored.locate_phase_native_grounded_word("kiva", &transfer).len(), 1);
    println!("GROUNDED_OBJECT_MEMORY_PASS words=1 unseen_position=37 novel_state=3 checkpoint=true lesion=true");
}

#[test]
fn grounded_word_never_appears_without_pointed_supervision() {
    let mut organism = EvoPhase::new(EvoConfig {
        sensory_cells: DIM, motor_cells: 7, dormant_cells: 128,
        hdc_dim: 64, ..EvoConfig::default()
    });
    organism.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(organism.enable_phase_native_grounded_objects(7, 7, 3, 4, 2));
    let v = frame(9, (5, 2, 0), (22, 4, 2, 0));
    organism.observe_initial_real(&v, false);
    assert!(organism.observe_phase_native_object_view(&v));
    assert_eq!(organism.phase_native_grounded_words(), 0);
    assert!(organism.locate_phase_native_grounded_word("kiva", &v).is_empty());
    // A teacher pointing outside the observed tile array cannot teach.
    assert!(!organism.teach_phase_native_pointed_word("kiva", 777));
}
