# AETERNA PR9 — selective factual learning, evidence-based abstention

Date: 2026-10-19. **EXPOSED DEVELOPMENT DIAGNOSTIC, NOT NEW QUALIFICATION.**

## Scientific problem

The prior candidate used a `factual-probe` proposal with high authored novelty
scores (0.85/0.85) whenever a factual effect could not be prospectively
validated. On conjunctive worlds this repeatedly displaced otherwise effective
ordinary U1 exploration. The same prior candidate performed worse than U1 in
one repeated-prerequisite development world (14/24 vs 24/24), establishing
**negative transfer**. That is not acceptable as evidence of general intelligence.

## Correction — calibrated acquired knowledge or abstention

Keep the existing SAME `Learner` journal, checkpoint, HumanProtection and U1.
The data-driven numerical predictor already validates a learned effect against
three later executed real transitions and checks its stability between the
older fitting prefix and newer evidence; contradictory or under-supported
consequences are `None`. Such a candidate may join U1 only if verified.
Remove the competing, high-scoring `factual-probe` hypothesis whenever
verification fails. Unknown remains unknown; ordinary U1 still selects and
executes exploration under its existing safety permissions. There is no
new second controller, motor role label, training schedule or privileged
knowledge injected by this repair. Planning/prospective checks remain metered.

## Paired tests (all cohorts now EXPOSED)

The same deterministic synthetic evaluators, opaque role permutations,
initial conditions, resources and action budgets were applied to both arms:

| World and budget | Selective factual + U1 | Original U1 | Inference |
|---|---:|---:|---|
| Five-sensor conjunction/alarm, 120 role permutations, 144 actions | 120/120, **4,182** actions, 0 untrusted interventions | 120/120, **4,182** actions | No-regression on disclosed family; no acquired-conjunction advantage |
| Four-sensor prerequisite family A, 24 permutations, 96 actions | 24/24, **580** actions | 24/24, **580** actions | No-regression |
| Four-sensor reset/trap family B, 24 permutations, 96 actions | 24/24, **686** actions | 24/24, **686** actions | No-regression |
| Three-sensor motor roles, 24 permutations, 32 actions | 19/24, **486** actions | 19/24, **486** actions | Conservative fallback loses earlier one-family advantage |
| Three-sensor roles, 24 permutations, 96 actions | 23/24, **671** actions | 23/24, **671** actions | No-regression, not full success |

The previous candidate's 22/24 vs 19/24 single-family advantage at a 32
step budget **no longer holds**, and must not appear as a current result.
This conservative repair solves a reliability problem, not the absent
conjunctive world model or autonomous goal creation.

Positive control: a real sequence of executed numeric +1 observations,
checkpoint persistence, and prospective validation still creates a useful
`factual-experience` proposal: 10/12 post-restore U1 choices used the
validated consequence, and the same journal subsequently rejected the
assertion after three contradictory +10 observed effects. This is narrow,
not robust transfer evidence.

## Reproduction and boundaries

```
cargo test --locked --offline --test unified_self_directed_lifetime --test factual_transfer_development --test factual_selective_holdout -- --nocapture
python scripts/project_audit.py
cargo check --locked --offline --all-targets
```

Exact GitHub/Linux/Windows CI remains necessary. All current synthetic
worlds are exposed developmental diagnostics. Do not optimize against them
and then call the same seeds held-out. Next independent tests must
pre-register diverse unseen causal structures, multiple simultaneous guards,
resource depletion, and sequential external domains, with a no-regression
criterion versus existing U1 and meaningful savings when the learned
predictor is actually validated. No change to RSP2 policy admission is made.
