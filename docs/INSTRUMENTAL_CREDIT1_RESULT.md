# INSTRUMENTAL-CREDIT1 — engineering controls pass, unscheduled goal policy NEGATIVE

Recorded after first explicit diagnostic run [39315607292](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39315607292) on 2026-10-19. **Not an independent scientific result.**

Preregistered before code at commit `58e93a1568e182f7dc81f658f46490cb0f674b0a`. The opt-in `InstrumentalCredit` separates factual transition acquisition from U1 goal utility: no real goal progress ⇒ no native goal credit; real advance ⇒ native U1 goal credit; selected verified invisible preparation is eligible for one-step delayed credit only after a later real goal advance. A new temporary credit field and counters are included in the same joint checkpoint; old behavior is opt-out.

The engineering tests passed: twelve truly executed no-progress actions accumulated 12 factual records without changing instrumental weights; the next actual progress event trained native U1 goal utility; HumanProtection veto never minted a virtual success, and exact checkpoint restoration passed. Main source audit eventually bound the new JSONL endpoint after a transitory SHA mismatch was corrected.

## Exposed unforced outcome: negative

Each arm had 192 consecutive REAL protected, U1-selected actions, **all four motors offered throughout**, in an authored hidden-ready world, then a frozen goal of one new progress unit (max 24 actions). No manual training motor schedule. Two distinct opaque role permutations were used.

| Roles | Goal credit rule | Cold causal probes selected | Actual training progress steps | Holdout solved? | Holdout actions | History goal selected |
|---|---|---:|---:|---|---:|---:|
| 1,0,3,2 | Old immediate novelty/goal utility | 35 | 12 | YES | 11 | 0 |
| 1,0,3,2 | New instrumental-only credit | 40 | 10 | **NO** | 24 | 0 |
| 3,2,1,0 | Old immediate novelty/goal utility | 33 | 7 | YES | 16 | 0 |
| 3,2,1,0 | New instrumental-only credit | 3 | 4 | **NO** | 24 | 0 |

The experimental mode correctly counted 10 and 4 actual goal-progress events and 182 and 188 factual-only steps. No stage produced a confirmed retrospective preparation credit. The lower instrumental weights in this regime did NOT translate to more successful action selection.

**Verdict:** Mechanism PASS (engineering), goal-policy benefit **NEGATIVE** on these authored diagnostic cases; do not relabel it a behavioral breakthrough. The learned history plan was not selected in either treatment. The four world/arm cases are now EXPOSED and must not serve as a fresh scientific benchmark.

## Revised research diagnosis

A long free exploration lifetime with a goal of progress=2000, followed by freezing the controller and suddenly demanding progress=current+1, is a poor way to assess credit for *completed goal sequences*. Both regimes were learning mainly from uncompleted goals. Next developer-authored fix is a truthful external EPISODE RESTART observation, not a force-selected motor. During each new episode the old goal remains achievable, all motor options are freely offered and U1 can learn from genuine completed trajectories. Restarts preserve the same Learner journal and explicitly sever the short predecessor context (no false causal linkage across episodes). A proper autonomous end-to-end test must compare early and late episodes, as well as matched no-probe control.

PR #9 remains draft. RSP2 and HumanProtection unchanged.
