use aeterna_v1::EvoPhase;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn invoke(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_perception"))
        .args(args)
        .output()
        .unwrap()
}
fn ok(args: &[&str]) -> String {
    let result = invoke(args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap()
}
fn name(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn actual_file_training_png_jpeg_inversion_correction_and_restart() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch = Scratch(
        std::env::temp_dir().join(format!("aeterna-perception-{}-{nonce}", std::process::id())),
    );
    std::fs::create_dir(&scratch.0).unwrap();
    let model = scratch.0.join("learned.json");
    let corrected = scratch.0.join("corrected.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/digits.csv");
    assert!(ok(&["train", name(&fixture), name(&model)]).contains("trained_records=1433"));
    let original = std::fs::read(&model).unwrap();
    let pixels = include_str!("data/digits.csv")
        .lines()
        .next()
        .unwrap()
        .split(',')
        .take(64)
        .map(|x| (x.parse::<f32>().unwrap() * 255.0 / 16.0).round() as u8)
        .collect();
    // This is a held-out recorded digit, exported to actual encoded image files.
    let gray = image::GrayImage::from_raw(8, 8, pixels).unwrap();
    let png = scratch.0.join("digit.png");
    let jpeg = scratch.0.join("digit.jpg");
    let inverted = scratch.0.join("white-background.png");
    gray.save(&png).unwrap();
    gray.save(&jpeg).unwrap();
    let mut white = gray.clone();
    image::imageops::invert(&mut white);
    white.save(&inverted).unwrap();
    for image in [&png, &jpeg] {
        let prediction = ok(&["recognize", name(&model), name(image)]);
        assert!(prediction.contains("digit=0 "), "{prediction}");
        assert!(prediction.contains("authority=IMAGINED"));
    }
    assert!(ok(&["recognize", name(&model), name(&inverted), "--invert"]).contains("digit=0 "));
    assert_eq!(std::fs::read(&model).unwrap(), original);
    assert!(
        ok(&["correct", name(&model), name(&png), "2", name(&corrected)])
            .contains("corrected_annotation=2")
    );
    let old = EvoPhase::from_online_checkpoint(&original).unwrap();
    let new = EvoPhase::from_online_checkpoint(&std::fs::read(&corrected).unwrap()).unwrap();
    assert_ne!(
        old.phase_native_learned_fingerprint(),
        new.phase_native_learned_fingerprint()
    );
    assert!(
        new.phase_vector_info().unwrap().observations
            > old.phase_vector_info().unwrap().observations
    );
    assert_eq!(std::fs::read(&model).unwrap(), original);
    assert!(
        !invoke(&["correct", name(&model), name(&png), "2", name(&model)])
            .status
            .success()
    );
    assert!(!invoke(&["train", name(&fixture), name(&model)])
        .status
        .success());
    let malformed = scratch.0.join("malformed.png");
    std::fs::write(&malformed, b"not an image").unwrap();
    assert!(!invoke(&["recognize", name(&model), name(&malformed)])
        .status
        .success());
    assert_eq!(std::fs::read(&model).unwrap(), original);
}

#[test]
fn acquired_program_image_mode_has_no_prototype_fallback() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "aeterna-program-images-{}-{nonce}",
        std::process::id()
    )));
    std::fs::create_dir(&scratch.0).unwrap();
    let model = scratch.0.join("programs.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/digits.csv");
    let trained = ok(&["train", name(&fixture), name(&model), "--programs"]);
    assert!(
        trained.contains("engine=acquired_programs") && trained.contains("trained_records=1433")
    );
    let bytes = std::fs::read(&model).unwrap();
    let e = EvoPhase::from_online_checkpoint(&bytes).unwrap();
    assert!(e.phase_constructor_info().unwrap().factual_updates > 0);
    assert!(!e.phase_primitives().is_empty());
    assert_eq!(e.phase_vector_info().unwrap().active_prototypes, 0);
    let png = scratch.0.join("actual-heldout.png");
    let pixels = include_str!("data/digits.csv")
        .lines()
        .next()
        .unwrap()
        .split(',')
        .take(64)
        .map(|s| (s.parse::<f32>().unwrap() * 255.0 / 16.0).round() as u8)
        .collect();
    image::GrayImage::from_raw(8, 8, pixels)
        .unwrap()
        .save(&png)
        .unwrap();
    let output = ok(&["recognize", name(&model), name(&png)]);
    assert!(output.contains("engine=acquired_programs"));
    assert!(output.contains("abstained=true") || output.contains("authority=IMAGINED"));
    assert_eq!(std::fs::read(&model).unwrap(), bytes);
    assert!(
        !invoke(&["train", name(&fixture), name(&model), "--programs"])
            .status
            .success()
    );
}
