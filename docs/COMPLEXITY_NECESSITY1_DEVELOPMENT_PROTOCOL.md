# COMPLEXITY-NECESSITY-1 — preserve simplest adequate cognition

Date: 2026-10-20. Developer-visible protocol BEFORE any mechanism changes.
Parent source: `bb991976d4cbf48186531afd5d661ab8c1eb3851`.
This is ENGINEERING development only; old one-use results are immutable.

## Why current variable-depth is inadequate

Cold development `39372704369`: 36 worlds, no teacher motor actions,
new depth-2..5 candidate: 467 solved/864 and 8,910 real actions, fixed
3-motor memory: 307/864 and 11,059 actions. Family 4-action tasks:
20→179 successes; five-action tasks only 3→7; **existing 3-action family
REGRESSED 140→137**. A length-5 option completed only once.
The direct-goal family was unchanged. This is a useful new capacity but
not stable general improvement; deeper suggestions can displace learned
simpler competence. A single residual is not evidence that MORE capacity
is the best solution.

## Normative complexity need decision

The agent's existing protected, chronological journal defines a
conditional prediction problem. Let `H_L` be the shortest supported
recent motor history length L. The organism should choose among:

- RETAIN: a currently evidence-consistent shorter behavioral explanation
  exists, so no added history is needed for the same goal and sensor PRE.
- INVESTIGATE: shorter history has contradictory physical outcomes, but
  the longer candidate is unsupported or predicted improvement does not
  exceed complexity cost. No optimistic terminal goal or motor permit.
- EXTEND: one more historically real predecessor resolves the
  SAME-PRE, SAME-shorter-suffix observed success/failure contradiction.
  Actual shorter sequence must have failed on its own at least once, not
  just be a rhetorical claim of insufficiency. Only bounded
  2..planning_depth(<=5) search may run.

A matched contrast needs the exact same initial visible sensor state,
same shorter action suffix, chronological real steps not crossing
external restart, and different observed terminal POST. The full
longer sequence must have at least one real goal success and ZERO real
contradictory repetitions. Direct-effect goals do not license extension.

For candidate full history length L versus shorter L-1, estimate
success probability with Laplace smoothing from real empirical counts:
`p_full=(full_successes+1)/(full_successes+full_failures+2)`,
`p_short=(short_successes+1)/(short_successes+short_failures+2)`.
An extension is justified only if `p_full-p_short` exceeds
`0.03*(L-3)` (a declared authored simplicity tax representing each
additional real motor action) AND has a real shorter-suffix failure.
For L<=3 no complexity extension decision is made. This is an
explicit heuristic, not statistically calibrated posterior confidence,
and an observed positive full route still requires a physically
executed U1 decision before it counts as acquired capability.

## Minimal opt-in integration

Implement as **a separate opt-in complexity need gate** in the SAME
Learner/U1/TemporalOptionLane. Baseline 3-motor option remains available
and preferred if it still has uncontradicted real support. Once it
receives a REAL contradictory physical outcome, the corpus marks
its inability to predict and a longer model may compete.

- The old `enable_variable_depth_options` behavior stays unchanged;
  new opt-in requires that mode and is enabled BEFORE first real action.
- No extra independent agent, no forced exploration, no motor-role
  semantics, no hidden-stage field and no changes to RSP2/LC1/HumanProtection.
- Preserve one macro-choice U1 credit AFTER real terminal POST, physical
  cost, external reset/veto cancellation and checkpoint validation.
- Charge both simpler and extended model scans inside the same fixed
  20,000-work evidence budget (5,000 fixed, remainder for expansion).
- Report diagnostic decision (simple, extended, no support), selected
  depth, and actual completed outcomes separately.
- Never consume or relabel a previously used scientific seed.

## Frozen BEFORE coding: validation

On existing 3-motor family: candidate gate MUST not regress goal
success vs the variable-depth path, and ideally restores the original
fixed-depth result. On 4-motor and 5-motor families, demonstrate real
native selection/completion and compare cold success, actual actions,
per-family regression, actual contradictory outcomes, and whether
complexity was truly necessary. A direct-progress negative control
MUST not adopt artificial complexity.

Tests include checkpoint roundtrip, stale goal, newly unavailable motor,
contradictory real POST and a new HumanProtection veto. Compare three
separate cold arms: fixed three, un-gated variable, and gated variable.
All raw positive and negative results preserved. No scientific PASS
claim for developer-visible test tuning.

## Longer-term reusable admission contract

The *same concept* can eventually govern other architectural growth:
a finite predicted-behavior error, insufficient existing representation,
an evidence-supported proposed additional computational degree of
freedom, measured external cost, prospective heldout confirmation,
nonregression/rollback. This first implementation covers **causal
history depth only**, not Rust/SNN structural self-modification or AGI.
