# COLD-EPISODES1 — unscheduled protected goal-reaching across truthful external restarts

Date: 2026-10-19. Development-only pre-implementation contract; NO blind transfer or AGI claim.

## Why
SELF-PROBE verified 57 U1-selected epistemic actions from 384 cold training steps, but final goal success was 1/2 vs 2/2. Instrumental-credit opt-in made neither of two new disclosed cold variants succeed. Both protocols trained for 192 long steps with a distant goal (progress=2000), then replaced it with an unrelated achievable target. They do not test learning credit from completed successful/failed goal sequences.

## Implement minimal honest EPISODE boundary
`ScientificRuntime::observe_constructing_external_restart(real_observation)` will report a world-initiated reset to the **same** existing constructing Learner. It must:
- accept only the ACTUAL new observation supplied by a trusted external world; never predict or invent POSTs;
- call the existing Learner `record_external_restart`, which preserves the historical protected PRE/action/POST records but marks the first next transition `restart=true` so action-history inference cannot cross an episode boundary;
- reset the native sensory readout to the current actual new external observation; preserve U1 weights, registry, model, causally earned info, external-action cost and the original human-protection audit;
- sever incomplete candidate temporal credit and unfinished goal episodes, without assigning a failure label to an interrupted action sequence;
- fail closed on poisoned/pending learner or invalid observation width; not touch the physical actuator or reset the protection audit or resource budget;
- preserve the full joint checkpoint in the exact old restoration format.

## Exposed developmental benchmark
An evaluator-only hidden precondition world begins with progress=0 and readiness=false. During EACH episode all four opaque motors remain available. U1 freely chooses up to 24 protected actions toward the SAME externally specified reachable goal=progress1. Actual selected motors get REAL POSTs with cost1; the world may fail to progress due hidden readiness. After success or exhaustion the evaluator resets its virtual world to a new physical episode and passes only the real initial numeric observation to the new API. There is NEVER an externally selected educational motor.

Compare four cold starts across multiple held-out-for-this-fixture motor orders:
A) original self-directed factual baseline, no hidden-history proposal, no pair probes;
B) learned history proposal, no active probing;
C) learned history proposal plus U1-selected pair probes;
D) C with goal-only instrumental credit (developer-authored, trial can reject it).

Use identical initial cold state, same physical rules, agent-offered actions, cost and per-episode budget. Report individual episodes, first/late success counts, executed primitive actions, actual U1-selected probes and learned history plans, total journal length and meta weights. No synthetic POST, motor meaning or teacher-selected training actions reach the learner. Test physical veto and checkpoint restart correctness separately.

Engineering criterion: API real-restart integrity + source-bound logs + honest descriptive early/late results. It is NOT required to pass an invented success advantage: negatives remain negative, and a successful result requires a later disjoint one-use protocol before any scientific promotion.

## Limitations
The environment owner still determines objectives and physical episodic resets. The reset is environmental control, NOT an action chosen by organism. A one-step hidden history is insufficient for arbitrary keys/counters/world composition. Need independent external benchmark later; never reuse previous T-SR1/T-SR2/AC2 worlds as 'fresh'.
