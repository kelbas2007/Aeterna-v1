# INTEL-1 Repair-7 — predecessor-conditioned epistemic coverage

Status: **PRE-REGISTERED BEFORE REPAIR-7 CODE CHANGE**

Date: 2026-10-07

## Preserved history

INTEL-1 and R1-R4 remain failed burned verdicts.
Repair-5 and Repair-6 are retained as partial generic arbitration improvements; neither is relabeled PASS.

## Earliest remaining bottleneck

Context discovery requires a factual collision:

same acquired base + same opaque action + different factual predecessor -> different successor.

Generic base-level exploration currently treats an action as known after observing it once at the base, regardless of predecessor history. Therefore a different predecessor may never receive the same action, preventing the collision needed to discover history dependence.

## Allowed repair

Extend G21's existing factual predecessor memory with bounded **predecessor-conditioned action coverage**.

When context refinement is enabled and a current base has factual evidence of at least one *other* predecessor:

1. for the current factual predecessor, inspect G21 discovery records for this base;
2. if an opaque action has never been factually executed from this base under the current predecessor, that action is a context-coverage frontier;
3. if no promoted context hypothesis currently resolves the state, and no information-balanced unpromoted candidate currently has a higher-priority anchor request, G21 may request one such uncovered action;
4. after the action is factually executed, it is covered for that base/predecessor/action tuple;
5. coverage is bounded by the existing six motor actions and G21 discovery capacity.

No hidden context ID is created. The predecessor is the already-qualified physical previous-base memory.

## Priority

Within G21:
1. promoted coherent context readout;
2. information-balanced anchor of an existing unpromoted hypothesis;
3. predecessor-conditioned uncovered action;
4. otherwise yield to rival/goal/general reasoning.

## Forbidden

No world type, task counter, evaluator state ID, authority action, correct action, goal answer, hidden-context label, alternating parity, reset-on-world-change or per-world module switch.

## Repair witness

1. Same base visited under two factual predecessors.
2. An action executed only under predecessor A is considered covered for A but still uncovered for B.
3. G21 requests that action under B when no higher-priority context hypothesis needs probing.
4. After execution under B, that tuple is no longer a coverage frontier.
5. Persistent all-refiners history world with stale earlier candidates promotes a useful context hypothesis and scores >=60/64, >=28/32 each side.
6. Matched no-context control <=40/64.
7. Repair-6 direct rival deferral PASS.
8. Repair-5 side-balance PASS.
9. Repair-4 coexistence PASS.
10. Repair-3 parent support remains exactly 1.
11. Repair-1/2, G21/G22/G23, G20, Human Protection and Release PASS.

## Freeze/rerun

If Repair-7 passes, freeze a new cognitive SHA and run a new independent INTEL verdict. No G24/G25.
