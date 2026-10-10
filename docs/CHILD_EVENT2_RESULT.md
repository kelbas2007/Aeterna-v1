# CHILD-EVENT-2 — source-immutable replication definitively rejects cue-conditioned memory

Date: 2026-10-10. **REPLICATION BEHAVIOR FAIL**.
[Exact GitHub Actions run 38068884815](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38068884815).
The workflow checked out the **identical original source commit**
\`855b58c1d3e680509ce629b02731d78f2a453eb0\`
from the first CHILD-EVENT-1 experiment. No learned-controller
parameter or branch mechanism was changed between runs.

The independent Farama MiniGrid MemoryS7 experiment used
new training seeds 120000..120127 (128 real episodes)
and new heldout seeds 121000..121023 (24), never
consumed in the previous 110000/111000 experiments.
The same physical U1/Human Protection gate and same
public category-coded partial observation, max 200 actual
motor steps per episode, native checkpoint and
frozen holdout. The only two cognitive options:
generic learned frame policy or the *same* general learner
with autobiographical success/failure sequence memory.
External hidden positions \`success_pos\`,
\`failure_pos\` and \`agent_pos\` are used ONLY
in the evaluation harness after actual simulator
actions; they were never given to the Rust organism.

## Factual outcome

| Frozen external MemoryS7 metric | General frame policy | Actual positive/negative episode-memory policy |
|---|---:|---:|
| Completed full task /24 | 7 | 7 |
| Matched seeded random motor | 6 | 6 |
| Real train successes /128 | 59 | 59 |
| Distinct rewarded experience associations | 0 | 64 |
| Failed experience counts | 0 | 376 |
| Distinct heldout chosen exit positions | 1 | **1** |
| Specific chosen exit on heldout | lower (5,4) 24/24 | **lower (5,4) 24/24** |

The hidden scorer found **17/24** heldout goals
at the opposite upper exit (5,2), only **7/24**
at lower (5,4). Thus the optimal completely
MEMORYLESS constant-exit strategy would have
achieved **17/24** — more than twice the
learned 7/24. Both organisms nevertheless
chose lower (5,4) on every heldout world.

Exact true scorer:
\`CHILD_EVENT2_CAUSAL_BRANCH_AUDIT episodic=7/24
no_memory=7/24 random=6/24 best_fixed_side=17/24
distinct_chosen_exits=1 correct_goal_branches=1
chosen={(5, 4): 24} verdict=BEHAVIOR_FAIL\`.

Contrast with first previously audited run:
original episode memory 14/24 vs no-memory
10/24, but episodic agent chose lower 24/24
and frame agent upper 24/24, benefiting only
from a heldout goal-side imbalance. Both
immature claims are now refuted by the same
rigorous behavioral criterion.

## Scientific conclusion

The source-protected SAME code on independent
new seeds did NOT generalize, did NOT adjust
the selected branch based on the first observed
object, and did NOT beat the best fixed-side
blind rule. **The original 14/24 should NEVER
be claimed as actual causal history use**.
Physical learned event memory is real
(reward/negative associations and checkpoint),
but it doesn't supply a reliable inner
representation of how early cue relates to
the future branch decision.

A continuous record of experiences and
retrospective reward assignment alone is
not sufficient for childlike relational
reasoning. The correct *fundamental*
research direction is active relational
binding and comparison: stabilize the
identity of a currently perceived object,
retain its relationship to the current
objective through occlusion, compare to
a newly perceived candidate, and update
causal predictive beliefs on contradiction.
The learning algorithm must be tested
for induced relations and transfer across
different visual worlds; do not prewire
MiniGrid object indices, hardcode "choose
upper/lower", tweak one consumed seed, or
mislabel any synthetic control as AGI.

Original native tests of checkpoint, frozen
weights, physical motor lesion and recall
remain valid *component tests*, but the
actual autonomous external goal remains
unproven. No merge to main.
