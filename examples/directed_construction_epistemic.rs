//! Development evidence-gate exercise, not an independent intelligence test.
//! Existing synthesis chooses its candidate; no program is injected into runtime.
use aeterna_v1::construction::epistemic::EvidencePolicy;
use aeterna_v1::construction::synthesis::{replay, SearchConfig};
use aeterna_v1::construction::vm::Meter;
use aeterna_v1::construction::{Config, ExternalPost, ProtectedConstructionRuntime};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn make(enabled: bool) -> Result<ProtectedConstructionRuntime, Box<dyn std::error::Error>> {
    let cfg = Config {
        minimum_training: 32,
        search_every: 32,
        shadow_steps: 4,
        journal_capacity: 2048,
        search: SearchConfig {
            work: 50_000,
            max_expr_nodes: 1,
            max_state: 0,
            max_updates: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut r = ProtectedConstructionRuntime::new(
        EvoPhase::new(EvoConfig {
            sensory_cells: 1,
            motor_cells: 2,
            ..Default::default()
        }),
        cfg,
        vec![0],
    )?;
    if enabled {
        r.enable_epistemic(EvidencePolicy::default())?;
    }
    Ok(r)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut legacy = make(false)?;
    let mut revised = make(true)?;
    for i in 0..256 {
        let y = i64::from(i % 5 != 0);
        for r in [&mut legacy, &mut revised] {
            r.step_action(0, safe, |_| {
                Ok(ExternalPost {
                    observation: vec![y],
                    cost: 1,
                })
            })?;
        }
        if i % 7 == 0 {
            let bytes = revised.checkpoint()?;
            revised.restore_cognition(&bytes)?;
        }
    }
    // Both models are frozen. This pseudo-random evaluation stream cannot train
    // either learner and is not reused for changing the candidate or threshold.
    let mut results = Vec::new();
    for r in [&legacy, &revised] {
        let history: Vec<_> = r
            .learner()
            .records()
            .iter()
            .map(|x| x.transition.clone())
            .collect();
        let mut m = Meter::new(1_000_000);
        let mut state = replay(
            r.learner().program(),
            &history,
            r.learner().library(),
            &mut m,
        )?
        .1;
        let mut input = r.learner().current().to_vec();
        let mut seed = 0x37a9c52d_u64;
        let mut errors = 0;
        for _ in 0..1000 {
            let (p, s) =
                r.learner()
                    .program()
                    .step(&input, 0, &state, r.learner().library(), &mut m)?;
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let y = i64::from((seed >> 32) % 5 != 0);
            errors += usize::from(p != [y]);
            input = vec![y];
            state = s;
        }
        results.push(errors);
    }
    let report = revised
        .learner()
        .epistemic_report()
        .ok_or("missing evidence")?;
    if legacy.learner().report().promotions != 0
        || revised.learner().report().promotions == 0
        || results[1] >= results[0]
    {
        return Err(format!(
            "development contrast failed: errors={results:?}, evidence={report:?}"
        )
        .into());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "kind":"EP1-integrated-development","independent_qualification":false,
            "training_stream":"authored 4/5 ones; first zero is an incumbent lucky observation",
            "search_scope":"existing bounded synthesis; one-node expressions, no new memory in this fixture",
            "external_actions":"supervised action zero, screened by unchanged EvoPhase protection",
            "training_events":256,"frozen_probe_events":1000,"legacy_frozen_errors":results[0],"ep1_frozen_errors":results[1],
            "legacy":legacy.learner().report(),"ep1":revised.learner().report(),"evidence":report,
            "factual_first_zero_retained":revised.learner().records()[0].transition.post==[0]
        }))?
    );
    Ok(())
}
