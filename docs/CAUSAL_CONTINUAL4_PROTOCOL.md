# CAUSAL-CONTINUAL-4 — four measurable autonomy repairs

2026-10-20. Development preregistration, before scoring external evaluation.
Starting main: `a5b22cc4fb0114cbd117e4d0846fae7ae4ecd162`. Historical one-use
qualifications and negative cohorts are not replayed or relabeled.

1. Predictive contexts are learned from real PRE/action/POST only. Preserve
   multiple outcomes rather than deleting a contradictory transition. Compare
   prequentially scored history hypotheses, measure mutual information between
   hypotheses and prospective observations, and distinguish aleatoric noise
   from information which can change a goal decision. Charge action costs.
2. Actual native U1 remains the sole motor selector. Experiments and routes are
   MODEL proposals. Only real completed goals can award delayed experiment
   utility; new observations alone cannot award goal-control utility. Test
   protected useful-query tasks and no-information controls, with opaque motors.
3. Bound raw working memory, consolidate sufficient statistics and preserve
   absolute event ids, cost debt, contradictory outcomes, VM replay anchors and
   checkpoint safety. Continue beyond 4096 real actions in one learner, restore
   and continue again, reject tampered metadata; report any forgetting.
4. Expose horizon/work, journal and motor counts in JSONL. Demonstrate at least
   twelve sensors, more than four motors and a ten-action goal path. The new
   finite horizon is configurable up to 32; no infinite-memory/AGI claim.

All local unit/development cases may be inspected for diagnosis. A later frozen
external comparison must identify source/binary hashes before held scoring.
Use new seeds 230000..230031 for held opaque role and observation assignments,
at identical real action and planning budgets. Compare new native U1 against
the legacy selector and appropriate new-mode ablations; require non-regression
on direct-goal tasks. Keep failed comparisons and actual action costs. These
authored worlds are engineering controls, not independent real-world AGI.

## Frozen comparison definition (before any held scoring)

`examples/causal_continual_eval.rs` evaluates all 32 seeds, four families and
three arms (`legacy`, `causal`, `horizon_one`). Every arm has four available
unit-cost motors, 120,000 planning-work units per actual action, the same
episode limits and training resets. Only the horizon-one ablation reduces
the planning horizon; both full arms use 12. The generator supplies no action
labels or hidden state to the learner. Freeze and checkpoint/restore before
held episodes; verify that learned causal evidence stays fixed throughout.

- Ten-step route: 12 sensors; 260 training episodes, 9 actions each, varying
  initial stage; one held episode from stage zero, at most 36 actions.
- Noisy sensor: 600 training and 64 held episodes, at most four actions each;
  paid 90%-accurate query, irrelevant random motor and two terminal guesses.
- Uninformative sensor: identical limits but query cue is independent noise.
- Direct goal: 128 training and 64 held episodes, one action each.

Engineering gates fixed before scoring: causal route success at least 24/32
and better than horizon-one; causal noisy-sensor success at least 75% and
at least 10 percentage points above legacy; direct-goal success no more than
two percentage points below legacy. Always report noise-control scores,
real costs, all worlds and all ablations, including failures. Noise-control
fluctuations cannot count as an information-discovery success. These gates
do not establish general hidden-state recovery or independently sourced
external-world transfer. Record source revision and SHA256 of the executable
before invocation, and retain all actual PRE/action/POST traces.

Required verification: the integrated `scripts/check_project.py` suite, adapter
contracts, old checkpoint compatibility, protection and existing native role
parity, plus the new tests. Changes are opt-in; existing source-bound scores
remain tied to their original source. Full new behavior is admitted only on
actual outcome evidence, never a passing compile or a claimed imagined result.
