# G8-FRESH-2 — SEALED PLANNING QUALIFICATION WITH OBSERVABILITY CONTRACT

Status: **PRE-REGISTERED AFTER G8-FRESH-1 FAIL AND BEFORE FRESH-2 RUN**

Date: 2026-10-06

## Why FRESH-1 failed

G8-FRESH-1 was burned at run 37515610684. The first depth-4 world exposed a shorter high-value imagined path than the evaluator's physical route. The pack therefore failed the depth/model-integrity witness before complete scoring.

The immediate failure class is state/transition aliasing under the planning observation trace or an equivalent model-binding defect.

FRESH-2 does not reuse that pack.

## Scope correction

G7 FRESH-2 already statistically qualified the robust perception substrate on its frozen open-world family.

G8-FRESH-2 therefore isolates **planning conditional on an observable carrier state**. It does not re-claim perception robustness.

A planning world is well-posed only when the carrier observation representation can distinguish the evaluator's route states under the nuisance scheduled for that world.

## One-use fresh authority

- source/spec are committed before execution;
- external GitHub Actions GITHUB_RUN_ID is the authority seed;
- 10 deterministic sub-seeds;
- 8 worlds per sub-seed;
- N=80;
- pack digest emitted before scoring;
- after first observation the pack is permanently burned;
- any code/spec change requires a new run ID.

## World family

Each world has:
- four opaque motor IDs;
- route depth in {2,3,4};
- immediate trap reward 0.30;
- delayed goal reward 1.0;
- dead alternatives;
- fresh relational state codes;
- clean factual local-model tuition;
- held-out absolute translation;
- scheduled nuisance: clean, distractor, dropout, 90-degree rotation, or uniform scale.

## Frozen observability seal

The generator may reject a candidate world **before the pack is sealed** only for representation-level identifiability.

For all evaluator states used in one world:

1. clean carrier traces must be pairwise below the planner match threshold;
2. the held-out nuisance trace of each state must match its own clean trace at or above threshold;
3. that nuisance trace must remain below threshold against every other clean state.

This seal is evaluated without actions, rewards, plans or success outcomes.

It does not select worlds by planner performance. It prevents a nominally fully observable planning benchmark from silently becoming a POMDP because two evaluator states collapse to the same inherited carrier observation.

The number of rejected candidate state sets is reported.

## Arms

FULL_IMAGINATION:
- carrier-owned IMAGINED rollout to depth 4.

DEPTH1:
- same model, imagined rollout restricted to one transition.

NO_IMAGINATION:
- same learned transition state, immediate MODEL reward only;
- no imagined rollout nodes.

SHUFFLED_MODEL:
- same transition inventory;
- successor bindings permuted by the existing causal control.

## Model-integrity witness

For every scored FULL world:

- the selected first action must equal the opaque delayed-route motor;
- selected imagined depth before the first physical action must be at least the evaluator's true route depth;
- pure imagination must not mutate REAL;
- physical execution must follow the learned route rather than an evaluator shortcut.

No per-world assertion may terminate scoring early. Integrity violations are counted and the full sealed pack is scored before final PASS/FAIL.

## Acceptance

PASS requires all:

1. 80/80 worlds are scored after sealing.
2. FULL >=72/80.
3. Wilson 95% lower bound >0.80.
4. zero model-integrity/depth violations among FULL successes.
5. each route depth 2,3,4 has >=80% success.
6. each single nuisance class has >=70% success.
7. FULL exceeds DEPTH1 by >=40 percentage points.
8. FULL exceeds NO_IMAGINATION by >=40 percentage points.
9. FULL exceeds SHUFFLED_MODEL by >=30 percentage points.
10. successful first actions follow opaque motor permutations rather than one fixed numeric ID.
11. all G0-G8 mechanism regressions and ownership audits PASS.
12. optimized release build PASS.

## Interpretation

A PASS qualifies bounded delayed-reward planning **given an identifiable carrier observation state and an acquired local transition model**.

It does not establish:
- planning under perceptual aliasing / POMDP uncertainty;
- unrestricted horizons;
- arbitrary theorem proving;
- language reasoning;
- AGI.
