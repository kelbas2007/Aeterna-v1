# CAUSAL-CONTINUAL-4 — interrupted negative comparison

The first frozen implementation was `46b18fed76070d24db0e84a76afd869a78a4b2c1`.
Binary SHA256: `09ba8b39948670a8b82cb9cff01b07d9e950c637f4dae2ce8f799fe695ffdd54`.
Its ordinary development tests passed, but the held comparison exposed
failures with opaque numeric sensor codes. On seed 230001 the new controller
failed the ten-step route; on seed 230002 it solved only 38/64 noisy-sensor
episodes, below the 75% gate for that sample.

The comparison was stopped after 110 completed lives out of the planned 384.
This is an explicit **protocol deviation and negative development outcome**,
not a completed qualification. All completed lifetime traces are retained;
the interrupted lifetime's unfinished trace is unavailable. All seeds
230000..230031 are withdrawn from future fresh qualification, including
the unexecuted suffix. No cohort-wide PASS, confidence claim or cost advantage
may be inferred from this partial experiment.

[Source/binary commitment](evidence/causal4-negative/commitment.json),
[partial summary](evidence/causal4-negative/summary.json),
[completed factual traces](evidence/causal4-negative/actual_traces.jsonl.gz).

Diagnosis: the new-mode feedback still inherited numeric distance-to-goal
progress, and competing legacy recall/VM proposals still inherited numerical
goal proximity. Neither is meaningful for arbitrarily renamed categories.
In addition, intermediate information and route decisions received immediate
zero utility before their delayed episode credit, undermining their future
goal value. The repair uses exact goal membership and actual external reward,
calibrates competing immediate motor forecasts against the same acquired
context distribution, and defers eligible information/route decision utility
until the real episode outcome. This does not assign motor authority to the
model; native U1 still selects each action.

The exposed failures become ordinary open regression cases. Any new held
comparison must use [CAUSAL-CONTINUAL-4B](CAUSAL_CONTINUAL4B_PROTOCOL.md), a new
source/binary commitment and the untouched 240000..240031 seed range.
