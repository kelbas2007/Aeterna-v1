//! Learn a program from actual action feedback, save and resume separately.
#[path = "support/induction.rs"]
mod induction;
use aeterna_v1::EvoPhase;
use std::io::{Read, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 || !matches!(args[1].as_str(), "learn" | "run") {
        return Err("usage: induced_programs <learn|run> <new-checkpoint.json>".into());
    }
    let bytes = if args[1] == "learn" {
        if std::path::Path::new(&args[2]).exists() {
            return Err("output checkpoint already exists".into());
        }
        let (bytes, _, actions) = induction::train(induction::Function::Parity, 1);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[2])?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        println!("actual_tuition_actions={actions}");
        bytes
    } else {
        let mut bytes = Vec::new();
        std::fs::File::open(&args[2])?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("checkpoint exceeds byte limit".into());
        }
        bytes
    };
    let e = EvoPhase::from_online_checkpoint(&bytes)?;
    if !e.phase_induction_enabled() || e.config().sensory_cells != 5 || e.config().motor_cells != 3
    {
        return Err("requires this example's acquired program checkpoint".into());
    }
    let r = induction::evaluate(&bytes, induction::Function::Parity, 1, None, 0);
    assert_eq!(r.correct, r.tasks);
    assert_eq!(r.sensing_first, r.tasks);
    println!(
        "mode={} heldout={}/{} actions={} acquired_nodes={} frozen_knowledge=true",
        args[1], r.correct, r.tasks, r.actions, r.nodes
    );
    for p in e.phase_induction_programs() {
        println!(
            "motor={} generation={} future_checks={} nodes={}",
            p.action,
            p.generation,
            p.future_checks,
            p.nodes.len()
        );
        for n in p.nodes {
            println!(
                "cell={} input={:?} threshold={:?} children={:?} outcome={:?}",
                n.cell, n.input, n.threshold, n.children, n.outcome
            );
        }
    }
    Ok(())
}
