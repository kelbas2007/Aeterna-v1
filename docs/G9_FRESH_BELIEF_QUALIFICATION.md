# G9-FRESH — ONE-USE STATISTICAL QUALIFICATION OF HISTORY-CONDITIONED BELIEF PLANNING

Status: **PRE-REGISTERED BEFORE FRESH RUN**

Date: 2026-10-06

## Claim under test

Given factual action/observation experience, an EvoPhase-owned recurrent HDC/FHRR belief state can preserve task-relevant history through a deliberately aliased current observation and support correct bounded planning better than matched current-observation-only controls.

This is not a claim of general POMDP solving.

## Fresh authority

The sealed qualification runs only with `AETERNA_RUN_FRESH_G9=1`.

- authority seed = external GitHub Actions `GITHUB_RUN_ID`;
- source SHA = `GITHUB_SHA`;
- 10 deterministic sub-seeds;
- 8 worlds per sub-seed;
- N=80;
- pack digest emitted before scoring;
- pack burned after first observation;
- any source/spec change requires a new run ID.

## Randomized world family

Each world independently randomizes:
- two visible cue relations;
- one deliberately shared ambiguous corridor relation;
- goal and dead-end relations;
- opaque 4-motor permutation;
- hidden context A or B;
- history length in {1,2,3};
- held-out absolute translation.

The context label exists only in the evaluator.

For both contexts:
- after the visible cue, the physical agent traverses a chain of one to three identical ambiguous corridor observations;
- the continuation motor is the same opaque motor;
- after the final ambiguous observation, the correct terminal motor depends on the earlier visible cue/context;
- wrong terminal motor gives factual failure.

Thus the current final corridor raster is identical across contexts; only factual history distinguishes them.

## Pre-seal representation integrity

The generator may reject a candidate relation-code set only when unintended clean carrier-trace collisions occur among:
- cue A;
- cue B;
- corridor;
- goal;
- dead.

The intended corridor alias across hidden contexts is mandatory and never rejected.

No action, reward, plan, success result or hidden policy is used by this integrity check. Rejection count is logged.

## Tuition

For each fresh world, all three arms receive the same factual training curriculum over both contexts and multiple absolute translations.

Every opaque action is tried from every relevant stage.

FULL_BELIEF:
- learns transitions over the recurrent carrier belief state.

OBSERVATION_ONLY:
- learns the same factual transitions using only the current observation trace.

RESET_HISTORY:
- uses the same recurrent mechanism but resets it to the current observation at each stage, deliberately deleting temporal identity.

Learning is frozen before held-out scoring.

## Recurrent state

The inherited generic update is:

```text
belief_0 = encode(obs_0)
belief_t+1 = PERMUTE(belief_t) ⊗ ACTION_ROLE[a_t] ⊗ encode(obs_t+1)
```

No latent context ID enters this state.

## Outcomes

Report:
- success / 80 and Wilson 95% CI;
- per-sub-seed success;
- per-history-length success;
- FULL vs OBSERVATION_ONLY vs RESET_HISTORY;
- belief-separation violations at the final aliased corridor;
- authority/REAL-firewall violations;
- successful opaque terminal motor IDs;
- mean physical actions;
- candidate-code rejection count.

## Frozen acceptance

PASS requires all:

1. FULL_BELIEF >=72/80.
2. Wilson 95% lower bound for FULL >0.80.
3. each history length 1,2,3 has >=80% FULL success.
4. OBSERVATION_ONLY <=48/80.
5. RESET_HISTORY <=48/80.
6. FULL exceeds OBSERVATION_ONLY by >=30 percentage points.
7. FULL exceeds RESET_HISTORY by >=30 percentage points.
8. zero belief-separation violations: the two context histories must remain below the carrier match threshold at the identical final corridor observation.
9. zero authority/REAL-firewall violations during planning.
10. successful terminal actions span at least three opaque numeric motor IDs.
11. all existing regressions and ownership audits PASS.
12. optimized release build PASS.

## Interpretation

A PASS qualifies a bounded recurrent carrier memory/belief mechanism across fresh randomized partially observable episodes and history lengths up to three.

It does not establish:
- arbitrary hidden-state inference;
- long-horizon POMDP planning;
- learned memory architecture;
- unrestricted concept invention;
- AGI.
