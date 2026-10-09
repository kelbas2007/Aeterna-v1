//! Application I/O only. Image decoding and the externally annotated outcome
//! live here; all acquired pattern memory and action selection live in EvoPhase.
use aeterna_v1::carrier::{PhaseNativeConfig, PhaseOnlineConfig, PhaseVectorConfig};
use aeterna_v1::scientific_runtime::{ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::io::{Read, Write};
use std::path::Path;
type Error = Box<dyn std::error::Error>;
fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn read_bounded(path: &str, max: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err("file exceeds supported byte limit".into());
    }
    Ok(bytes)
}
fn save_new(path: &str, e: &EvoPhase) -> Result<(), Error> {
    let bytes = e.online_checkpoint_bytes()?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}
fn fresh() -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: 64,
        motor_cells: 11,
        dormant_cells: 356,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_vector_learning(PhaseVectorConfig::default()));
    e
}
fn load(path: &str) -> Result<EvoPhase, Error> {
    let e = EvoPhase::from_online_checkpoint(&read_bounded(path, 16 * 1024 * 1024)?)?;
    if !e.phase_vector_enabled() || e.config().sensory_cells != 64 || e.config().motor_cells != 11 {
        return Err("requires a 64-feature, 11-motor perception checkpoint".into());
    }
    Ok(e)
}
fn image_pixels(path: &str, invert: bool) -> Result<Vec<f32>, Error> {
    let bytes = read_bounded(path, 8 * 1024 * 1024)?;
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let gray = reader.decode()?.to_luma8();
    if gray.width() == 0 || gray.height() == 0 {
        return Err("empty image".into());
    }
    let small = image::imageops::resize(&gray, 8, 8, image::imageops::FilterType::Triangle);
    Ok(small
        .pixels()
        .map(|p| {
            let x = f32::from(p[0]) / 255.0;
            if invert {
                1.0 - x
            } else {
                x
            }
        })
        .collect())
}
fn annotated_record(line: &str) -> Result<(Vec<f32>, usize), Error> {
    let values = line
        .split(',')
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != 65
        || values[..64]
            .iter()
            .any(|x| !x.is_finite() || !(0.0..=16.0).contains(x))
        || !values[64].is_finite()
        || !(0.0..=9.0).contains(&values[64])
        || values[64].fract() != 0.0
    {
        return Err("CSV needs 64 pixel intensities 0..16 and one digit annotation 0..9".into());
    }
    Ok((
        values[..64].iter().map(|x| x / 16.0).collect(),
        values[64] as usize,
    ))
}
fn teach(rt: &mut ScientificRuntime, pixels: &[f32], annotation: usize) -> Result<usize, Error> {
    rt.observe_external(pixels)?;
    let mut attempts = 0;
    for _ in 0..11 {
        if rt.goal_reached()? {
            break;
        }
        let result = rt.step_partial(
            |_| Some(safe()),
            |a| {
                attempts += 1;
                Ok((
                    pixels.iter().copied().map(Some).collect(),
                    f32::from(a == annotation),
                ))
            },
        )?;
        if !matches!(result, StepOutcome::Executed { .. }) {
            return Err("feedback did not execute".into());
        }
    }
    if !rt.goal_reached()? {
        return Err("factual correction budget exhausted".into());
    }
    Ok(attempts)
}
fn run() -> Result<(), Error> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("train") if args.len()==3 => {
            if Path::new(&args[2]).exists() {return Err("output checkpoint already exists".into());}
            let data=String::from_utf8(read_bounded(&args[1],8*1024*1024)?)?;
            let mut rt=ScientificRuntime::new(fresh())?;rt.set_outcome_goal(1.0)?;
            let mut counts=[0;10];let mut records=0;let mut actions=0;
            for (line_no,line) in data.lines().enumerate() {
                let (pixels,digit)=annotated_record(line).map_err(|e|format!("line {}: {e}",line_no+1))?;
                let heldout=counts[digit]%5==0;counts[digit]+=1;if heldout {continue;}
                actions+=teach(&mut rt,&pixels,digit)?;records+=1;
            }
            if records==0 {return Err("no training records".into());}save_new(&args[2],rt.organism())?;
            println!("trained_records={records} actual_actions={actions} occupied_prototypes={} saved={}",rt.organism().phase_vector_info().unwrap().active_prototypes,args[2]);
        }
        Some("recognize") if args.len()==3 || (args.len()==4 && args[3]=="--invert") => {
            let pixels=image_pixels(&args[2],args.len()==4)?;
            let mut rt=ScientificRuntime::new(load(&args[1])?)?;rt.set_model_learning_enabled(false);rt.set_outcome_goal(1.0)?;
            rt.observe_external(&pixels)?;
            let before=rt.organism().phase_native_learned_fingerprint();
            if let Some(p)=rt.organism().phase_vector_predict(&pixels.iter().copied().map(Some).collect::<Vec<_>>()) {
                println!("digit={} distance={:.5} vote_margin={:.5} authority=IMAGINED",p.action,p.nearest_distance,p.vote_margin);
            }else{println!("abstained=true reason=insufficient_pattern_support");}
            assert_eq!(before,rt.organism().phase_native_learned_fingerprint());
        }
        Some("correct") if args.len()==5 || (args.len()==6 && args[5]=="--invert") => {
            if Path::new(&args[4]).exists() {return Err("output checkpoint already exists".into());}
            let label=args[3].parse::<usize>()?;if label>9 {return Err("digit annotation must be 0..9".into());}
            let pixels=image_pixels(&args[2],args.len()==6)?;let mut rt=ScientificRuntime::new(load(&args[1])?)?;
            rt.set_model_learning_enabled(true);rt.set_outcome_goal(1.0)?;
            let actions=teach(&mut rt,&pixels,label)?;save_new(&args[4],rt.organism())?;
            println!("corrected_annotation={label} actual_feedback_actions={actions} saved={}",args[4]);
        }
        _=>return Err("usage: perception train <digits.csv> <new-checkpoint.json> | recognize <checkpoint.json> <image.png|jpg|pgm> [--invert] | correct <checkpoint.json> <image> <digit> <new-checkpoint.json> [--invert]".into()),
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
