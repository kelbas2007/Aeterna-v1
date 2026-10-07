# INTEL-1 — first frozen-core verdict

Date: 2026-10-07

## Verdict

**FAIL — earliest causal bottleneck: no bootstrap action when the supplied goal is disconnected from the currently known model.**

Run: `37658761202`  
Frozen cognitive baseline: `308aba7c06fa89613276ef90da725be963f2d25f`  
Authority seed / burned INTEL pack: `37658761202`.

The workflow verified that `src/**`, `Cargo.toml` and `Cargo.lock` exactly matched the frozen qualified G23 baseline. Historical G23/G22/G21/Human Protection regressions passed before the verdict test.

The evaluator printed:

```text
INTEL1_SEAL seed=37658761202 motors=[2, 1, 4, 5, 3, 0]
history_actions=[3, 0] comp_op=AND drift_change_after=4
```

The organism then failed before completing World 1 with:

```text
NoSupportedAction
```

No W1 success metric was printed; later INTEL worlds were not scored.

## Diagnosis

At an entirely new target world, the current abstract state had no known physical transition path establishing relevance to the externally supplied goal.

The frozen runtime arbitration attempted:

1. promoted compositional/current-sensory representation;
2. promoted perceptual representation;
3. promoted history-context representation;
4. rival-hypothesis probe;
5. goal-conditioned active selector.

The goal-conditioned selector is intentionally selective: if the known physical model contains no path that connects the current state to the goal, the propagated goal-relevance field is zero. With no useful goal-conditioned score and no exploitation plan, it returns no action.

The already-qualified G16 generic learned-drive selector can bootstrap an unknown model by exploring direct/reachable epistemic frontier even when no goal connection is known, but the unified ScientificRuntime did not fall back to it.

Thus the first INTEL failure is not “cannot explore” and not “cannot plan to a goal” separately. It is an **arbitration/bootstrap gap between those acquired capabilities**:

> when goal relevance cannot yet be computed, fall back to general epistemic acquisition until a goal-relevant model becomes available.

## Repair rule

No G24/G25 is opened.

The only permitted cognitive repair before the next INTEL qualification is a task-agnostic fallback in ScientificRuntime:

`goal-conditioned selector unavailable -> existing G16 learned-drive general epistemic selector`.

Forbidden:
- world labels;
- state IDs;
- special W1 logic;
- evaluator motor/route knowledge;
- changed G23/G22/G21 mechanisms;
- threshold changes.

After that narrow repair, all historical regressions must pass, a new cognitive SHA is frozen, and a new independently seeded INTEL run is required. The first INTEL pack remains burned and FAIL.
