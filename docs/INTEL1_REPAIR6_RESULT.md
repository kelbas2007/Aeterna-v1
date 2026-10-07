# INTEL-1 Repair-6 verification result

Date: 2026-10-07

Verdict: **FAIL / PARTIAL — G19 correctly defers over-sampled context-owned rivals, but base-level exploration still lacks predecessor-conditioned coverage.**

Workflow: `37671299953`.

Direct Repair-6 witness passed:
- over-sampled old candidate: context yielded and G19 rival probe returned None;
- under-sampled side: context anchor remained available and G19 rival probe remained permitted.

Arbitration trace then showed lower layers trying other actions via GoalDirectedAction, e.g. actions 1,2,3,4,5. However each newly tried action became known after one base-level observation and was not systematically retried under the alternate predecessor.

A useful context candidate can only be born when the same base/action yields different successors under distinct factual predecessors. Without predecessor-conditioned action coverage, that collision may never be observed.

In one diagnostic seed where old action 0 happened itself to be history-dependent, a correct [predecessor1, predecessor2] candidate did promote after 32 observations. In the preregistered system witness whose useful history actions were different, no target promotion occurred.

Thus Repair-6 solves redundant G19 probing but not the remaining discovery coverage gap. No new INTEL authority pack was consumed.
