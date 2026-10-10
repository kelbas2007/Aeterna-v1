# COMPLEX-WORLD-1.1 — open evaluator correction after negative result

Date: 2026-10-10; research branch `research/beyond-intel4`.

## Historical first result (do not erase)
[First completed COMPLEX-WORLD-1 run 38025843171](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38025843171):
4/4 discovered the 14-state factual graph (17 edges),
4/4 completed frozen nine-step goals, physical necessary-link lesion and
checkpoint PASS, ~113–120 training trap encounters. But the *changed-law*
scenario yielded **zero actual contradicted motor observations in all four
arms**, because the shortest normal route and detour were equally long.
Thus `COMPLEX_WORLD1_SUMMARY passed=0/4 verdict=DEVELOPMENT_FAIL` stands.

## Why the benchmark itself was not decisive
Original evaluator routes:
`3 → 4 → 5` (main, 2 actions) and
`3 → 10 → 5` (detour, 2 actions).
A physical planner legitimately chooses either equal-length route before
a drift; this does NOT prove revision when main 3→4 is later blocked.
The failure is a benchmark-design ambiguity, not yet a demonstrated
architectural inability to replan.

## Precisely bounded open amendment
For **COMPLEX-WORLD-1.1**, retain the same seed, 14 states, six opaque
motor permutations, traps, 256×24 acquisition budget and all PASS thresholds.
Amend ONLY the evaluator's detour endpoint:
`3 → 10 → 4 → 5`; the physical action at 10 now leads to *generator
access* (state 4), not directly to *powered* state (5).
This makes the detour **3 actions** against **2 main actions**.
The original intact shortest path is still nine actions, the longer
detour is ten; after failed main 3→4, replan should require eleven
executed actions (including the failure). The return to state 4 is
physically observable, and action (m4) has different state-conditioned
effects at states 10 and 4. No changes to EvoPhase source are needed.

Criterion remains discovery of >=11 factual transitions, trap exposure,
frozen nine-step goal, intact physically lesionable route, changed-law
actual no-op **observed before detour**, goal regained through the
alternate route within 20 executed protected steps, and zero blocked/
unsupported actions. This remains OPEN iterative development, not a
new preregistered first-attempt scientific gate.
