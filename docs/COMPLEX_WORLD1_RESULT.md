# COMPLEX-WORLD-1 / 1.1 — self-discovered gated labyrinth

Date: 2026-10-10
Branch: `research/beyond-intel4`
Status: **OPEN DEVELOPMENT PASS 4/4 on corrected experiment**.
First-experiment FAIL preserved. Fresh seeded check tracked separately in
[protocol](COMPLEX_WORLD_FRESH1_PROTOCOL.md) and its dedicated CI runner.

## Truly more complex synthetic world

14 distinguishable raw/abstract states, six opaque motor commands reused
depending on state, physically learned graph with 17 nontrivial factual
transition links. The first mandatory route is nine actions
(start→foyer→key room→key acquired→generator access→powered key holder
→gate approach→secured corridor→exit hall→goal).
Two distractor classes (premature power-first trap and recoverable dead end),
recovery loops and alternate generator route test that motor outcomes
depend on actual physical state.

No target-world transition labels or motor schedule were imported. Only
the externally specified recognizable raw goal, protected action execution,
factual PRE/action/POST and terminal bounded reward were supplied.
The generic cognitive visual foundation and U1 weights remain transferred
from prior experience, as in COLD-GOAL-1.

### First open negative result (immutable)

[Run 38025843171](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38025843171):
all four acquired **17/17** factual changes and each completed
the intact frozen goal in exactly **9 actions**. 113–120 trap-state visits
during acquisition. However under hidden drift, the organism chose an
equally short detour *before* seeing the altered main link. Zero arms
executed a factual contradicted main link. Therefore
`COMPLEX_WORLD1_SUMMARY passed=0/4 verdict=DEVELOPMENT_FAIL`.
The CI job itself exited successfully because the test retained all
negative diagnostic evidence; this is NOT a scientific PASS.

### Explicit open benchmark correction

[Recorded before rerun](COMPLEX_WORLD1_AMENDMENT.md): change only the
detour endpoint from 10→5 to 10→4, preserving the same opaque motor IDs,
seed and 14 states. Thus the normal nine-step route is now strictly shorter
than the ten-step alternate route. Nothing in EvoPhase cognitive source
was changed for this correction.

### Corrected open result

[Run 38026192943](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38026192943)
reported **COMPLEX_WORLD11_SUMMARY passed=4/4 verdict=DEVELOPMENT_PASS**.

| Open arm | Self-acquired nontrivial edges | Successful training goals /256 | Intact frozen goal | Actual blocked transition | Actual 3→10→4 detour | Drift actions | Verdict |
|---|---:|---:|---|---:|---|---:|---|
| 0 | 17 | 236 | 9 actions | 1 | yes, both edges | 16 | PASS |
| 1 | 17 | 236 | 9 actions | 1 | yes, both edges | 14 | PASS |
| 2 | 17 | 235 | 9 actions | 1 | yes, both edges | 16 | PASS |
| 3 | 17 | 236 | 9 actions | 1 | yes, both edges | 15 | PASS |

In each world necessary-link causal physical lesion/restore passed,
checkpoint retained the learned route, U1 weights remained frozen during
the held-out intact goal, and there were zero blocked or unsupported actions.
The changed-law recovery did not receive a law-change flag: the predicted
main edge really returned the same physical state, and the ensuing route
used both actual detour transitions. This is a bounded research
demonstration, not open-world AGI.

## First-attempt source-frozen Fresh-1 — six new assignments

[Predeclared protocol](COMPLEX_WORLD_FRESH1_PROTOCOL.md) fixed a different
unused seed `0xB17E_2026_FE51_1001`, six fresh role assignments, the
source tree `d8c8114e5480fafd03dd22bddac0bb405bf88752:src`,
all criteria and the exact test blob BEFORE first execution.
[Fresh-1 run 38026527091](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38026527091)
verified source/test/seed and reported
`COMPLEX_WORLD_FRESH1_SUMMARY passed=6/6 verdict=DEVELOPMENT_PASS`.
This is the completed first attempt, not repeated evaluation of the
consumed open development seed.

| Fresh arm | Factual links learned | Training goals /256 | Frozen goal actions | Actual blocked main edge | Detour state transitions observed | Goal recovery actions | Verdict |
|---|---:|---:|---:|---:|---|---:|---|
| 0 | 17 | 233 | 9 | 1 | 2/2 | 13 | PASS |
| 1 | 17 | 235 | 9 | 1 | 2/2 | 14 | PASS |
| 2 | 17 | 235 | 9 | 1 | 2/2 | 16 | PASS |
| 3 | 17 | 236 | 9 | 1 | 2/2 | 11 | PASS |
| 4 | 17 | 236 | 9 | 1 | 2/2 | 16 | PASS |
| 5 | 17 | 236 | 9 | 1 | 2/2 | 14 | PASS |

All six: at least 116 actual training trap entries, physical first
synapse lesion/restore passed, checkpoint and frozen U1 weights passed,
and zero blocked/unsupported external actions. The factual failed main
edge was actually executed first, then both detour edges were traversed,
and the original goal reached within the predeclared 20-act limit.
No motor/transition roles or change flags were passed to EvoPhase.

This supports **bounded transfer within this 14-state topology**, not a
newly generated unseen graph topology or independent factorized key/power
world.

## Remaining fundamental limits

Despite key/power/gate metaphor, each composite causal state is represented
as a **single recognized abstract state**, not a proof that EvoPhase
factorizes independent key, electrical power and location variables and
recombines them in unseen configurations. This task has deterministic
state transitions and 14 known representable classes; it is not a 3D
interactive environment, noisy partial world, unsupervised image recognition,
or dynamically invented intrinsic goal. The graph search and novelty
frontier remain explicit bounded Rust algorithms on physically stored
EvoPhase synapses. Formal test of newly composed independent prerequisites,
noisy state ambiguity, multiple concurrent subgoals and a truly novel graph
topology still required.
