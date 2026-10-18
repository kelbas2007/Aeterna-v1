//! All dependencies coexist. The adapter never selects a learning target,
//! injects a program/constructor or switches tasks. Development, not qualification.
use aeterna_v1::construction::autonomy::{ActionOffer, AutonomyPolicy};
use aeterna_v1::construction::epistemic::EvidencePolicy;
use aeterna_v1::construction::synthesis::SearchConfig;
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
struct World {
    state: i64,
    x: i64,
    y: i64,
    rng: u64,
    layout: [usize; 7],
}
impl World {
    fn initial(&self) -> Vec<i64> {
        self.encode([0, 0, 0, 0, self.x, self.y, 0])
    }
    fn encode(&self, v: [i64; 7]) -> Vec<i64> {
        self.layout.iter().map(|&j| v[j]).collect()
    }
    fn step(&mut self, a: usize) -> ExternalPost {
        let r = [
            self.state,
            self.state * self.x,
            self.state * self.y,
            self.state * (self.x + self.y),
        ];
        self.state = a as i64;
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.x = ((self.rng >> 32) % 7) as i64 - 3;
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.y = ((self.rng >> 32) % 7) as i64 - 3;
        let noise = if self.rng & 0x10000 == 0 {
            10000
        } else {
            -10000
        };
        ExternalPost {
            observation: self.encode([r[0], r[1], r[2], r[3], self.x, self.y, noise]),
            cost: 1,
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reports = Vec::new();
    for (index, layout) in [
        [0, 1, 2, 3, 4, 5, 6],
        [6, 3, 4, 1, 5, 0, 2],
        [2, 5, 1, 6, 0, 3, 4],
    ]
    .into_iter()
    .enumerate()
    {
        let mut w = World {
            state: 0,
            x: 1,
            y: 2,
            rng: 719 + index as u64 * 89,
            layout,
        };
        let cfg = Config {
            width: 7,
            actions: 3,
            minimum_training: 12,
            search_every: 8,
            shadow_steps: 4,
            journal_capacity: 4096,
            search: SearchConfig {
                work: 500_000,
                max_expr_nodes: 3,
                max_frontier: 2048,
                max_updates: 12,
                max_state: 0,
                deterministic: false,
            },
            ..Default::default()
        };
        let mut r = ProtectedConstructionRuntime::new(
            EvoPhase::new(EvoConfig {
                sensory_cells: 7,
                motor_cells: 3,
                ..Default::default()
            }),
            cfg,
            w.initial(),
        )?;
        r.enable_autonomy(AutonomyPolicy {
            ceiling: SearchConfig {
                work: 8_000_000,
                max_expr_nodes: 9,
                max_frontier: 32768,
                max_updates: 24,
                max_state: 3,
                deterministic: false,
            },
            failures_per_expansion: 1,
            evidence: Some(EvidencePolicy {
                max_samples: 64,
                ..Default::default()
            }),
            max_external_cost: 4096,
        })?;
        let offers = vec![
            ActionOffer {
                available: true,
                max_cost: 1
            };
            3
        ];
        for i in 0..3000 {
            r.step_autonomous(None, &offers, safe, |a| Ok(w.step(a)))?;
            if i % 64 == 63 {
                let b = r.checkpoint()?;
                r.restore_cognition(&b)?;
            }
            if i % 128 == 127 {
                eprintln!("layout={index}, t={}, {:?}", i + 1, r.learner().report());
            }
        }
        r.freeze_learning()?;
        let model = r.learner().program().fingerprint();
        let library = serde_json::to_vec(r.learner().library())?;
        let constructors = serde_json::to_vec(r.learner().registry())?;
        let mut errors = vec![0; 7];
        for _ in 0..128 {
            r.step_autonomous(None, &offers, safe, |a| Ok(w.step(a)))?;
            let row = r.learner().records().last().ok_or("missing record")?;
            for (j, e) in errors.iter_mut().enumerate() {
                *e += usize::from(row.prediction.center[j] != row.transition.post[j]);
            }
        }
        let echo = layout.iter().position(|&j| j == 0).ok_or("layout")?;
        let mut goal = vec![None; 7];
        goal[echo] = Some(2);
        let mut steps = 0;
        while r.learner().current()[echo] != 2 && steps < 5 {
            r.step_autonomous(Some(&goal), &offers, safe, |a| Ok(w.step(a)))?;
            steps += 1;
        }
        let solved = layout
            .iter()
            .enumerate()
            .filter(|(_, j)| **j < 4)
            .all(|(i, _)| errors[i] == 0);
        let preserved = model == r.learner().program().fingerprint()
            && library == serde_json::to_vec(r.learner().library())?
            && constructors == serde_json::to_vec(r.learner().registry())?;
        reports.push(serde_json::json!({"layout":layout,"training_steps":3000,"frozen_probe_steps":128,"frozen_errors_by_channel":errors,"learnable_channels_exact":solved,"goal_reached":r.learner().current()[echo]==2,"goal_steps":steps,"frozen_computation_unchanged":preserved,"program":r.learner().program(),"constructors":r.learner().registry().constructors(),"learner":r.learner().report(),"autonomy":r.learner().autonomy_report(),"curriculum":r.learner().curriculum(),"epistemic":r.learner().epistemic_report(),"recursive_reports":r.learner().registry().meta_ledger().reports()}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"kind":"integrated-autonomous-lifetime-development","independent_qualification":false,"injected_solutions":0,"injected_constructors":0,"task_switches":0,"authored_learning_target_sequence":false,"reports":&reports})
        )?
    );
    if reports.iter().any(|r| {
        r["learnable_channels_exact"] != true
            || r["frozen_computation_unchanged"] != true
            || r["goal_reached"] != true
    }) {
        return Err("unified developmental criterion failed; retain all layouts".into());
    }
    Ok(())
}
