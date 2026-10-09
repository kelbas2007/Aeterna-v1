#[path = "support/real_programs.rs"]
mod recorded;
use aeterna_v1::EvoPhase;
use std::io::{Read, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: acquired_primitives learn <new-model.json> | run <model.json>".into());
    }
    match args[0].as_str() {
        "learn" => {
            let (bytes, _, actions) = recorded::train(recorded::Corpus::Signals);
            let e = EvoPhase::from_online_checkpoint(&bytes)?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[1])?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            println!(
                "recorded_signal_training_actions={actions} acquired_operations={} saved={}",
                e.phase_primitives().len(),
                args[1]
            );
        }
        "run" => {
            let mut bytes = Vec::new();
            std::fs::File::open(&args[1])?
                .take(16 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() > 16 * 1024 * 1024 {
                return Err("checkpoint too large".into());
            }
            let mut e = EvoPhase::from_online_checkpoint(&bytes)?;
            if e.config().sensory_cells != 50
                || e.config().motor_cells != 3
                || e.phase_constructor_info().is_none()
            {
                return Err("requires acquired 50-feature motion-signal model".into());
            }
            e.set_planning_learning_enabled(false);
            let fingerprint = e.phase_native_learned_fingerprint();
            let (_, test, _, _) = recorded::data(recorded::Corpus::Signals);
            let mut correct = 0;
            let mut abstained = 0;
            for r in &test {
                match e
                    .phase_induction_predict(&r.input.iter().copied().map(Some).collect::<Vec<_>>())
                {
                    Some(p) => correct += usize::from(p.action == r.label),
                    None => abstained += 1,
                }
            }
            assert_eq!(fingerprint, e.phase_native_learned_fingerprint());
            println!("resumed_recorded_signals={correct}/{} abstained={abstained} authority=IMAGINED frozen=true",test.len());
        }
        _ => return Err("expected learn or run".into()),
    }
    Ok(())
}
