# G8-FRESH — ONE-USE STATISTICAL QUALIFICATION OF IMAGINED PLANNING

Status: **PRE-REGISTERED BEFORE FRESH QUALIFICATION RUN**

Date: 2026-10-06

## Claim under test

Given a factual local transition model represented in EvoPhase carrier state, bounded IMAGINED rollout can select a delayed-reward route before the first physical action and transfer across fresh opaque motor permutations, route depths and raw-raster nuisance transformations.

This qualification does not claim unrestricted planning or AGI.

## Fresh authority

The qualification pack is generated only when `AETERNA_RUN_FRESH_G8=1`.

- authority seed = external GitHub Actions `GITHUB_RUN_ID`;
- source SHA = `GITHUB_SHA`;
- 10 deterministic sub-seeds;
- 8 worlds per sub-seed;
- total N = 80;
- pack digest is printed before scoring;
- any observed pack is permanently burned after the run;
- any design/code change requires a new run ID and a new pack.

## World generator

Each world has:
- four opaque motor IDs, permuted independently per sub-seed;
- one immediate-reward trap;
- one delayed route;
- at least one dead action;
- route depth in {2,3,4};
- distinct raw relational states generated from fresh state codes;
- held-out absolute translation not used in factual model tuition.

The delayed goal reward is 1.0.
The immediate trap reward is 0.30.
With discount 0.95, the delayed route remains higher-value through depth 4.

## Perception / nuisance schedule

Each sub-seed contains eight worlds:
- four clean worlds;
- one distractor world;
- one dropout world;
- one 90-degree rotation world;
- one uniform scale world.

Nuisance transforms are applied only at held-out execution. Factual transition tuition is clean.

The carrier uses the already-implemented robust high-level relational trace. No pack candidate is rejected based on carrier success or robust-trace similarity. Only exact duplicate clean raw states inside one world are regenerated so the external world is visually well-posed.

## Factual model tuition

For each world, the organism receives the same clean local factual transition graph at preregistered tuition translations.

The evaluator does not provide:
- route labels;
- goal distance;
- correct first action;
- hidden coordinates;
- a best-plan API.

Learning is frozen before held-out scoring.

## Arms

### FULL_IMAGINATION
Uses bounded EvoPhase IMAGINED rollout up to depth 4.

### DEPTH1
Same acquired transitions; IMAGINED rollout restricted to one edge.

### NO_IMAGINATION
Same acquired transitions; selects the highest immediate MODEL reward without constructing IMAGINED rollout state.

### SHUFFLED_MODEL
Same transition inventory and planner, but learned successor bindings are permuted by the existing causal control.

## Primary outcomes

For every world:
- held-out factual success;
- first physical action;
- executed physical action count;
- rollout nodes for FULL/DEPTH1/SHUFFLED;
- selected planning depth.

Report:
- successes / 80;
- Wilson 95% CI;
- per-sub-seed successes;
- per-route-depth successes;
- clean and per-nuisance successes;
- mean physical actions;
- mean rollout nodes.

## Frozen acceptance thresholds

G8-FRESH PASS requires all:

1. FULL >= 72/80 (90%).
2. Wilson 95% lower bound for FULL > 0.80.
3. Every route depth 2, 3 and 4 has >=80% FULL success.
4. Each single nuisance category has >=70% FULL success.
5. FULL exceeds DEPTH1 by at least 40 percentage points.
6. FULL exceeds NO_IMAGINATION by at least 40 percentage points.
7. FULL exceeds SHUFFLED_MODEL by at least 30 percentage points.
8. FULL's chosen first action follows the opaque permutation: no fixed numeric motor may account for the result.
9. Pure imagination leaves REAL state unchanged in sampled worlds.
10. All existing regression tests and mechanical ownership audits PASS.
11. Optimized release build PASS.

## Interpretation if FAIL

The first observed pack is not tuned.

A FAIL is classified from the evidence:
- representation/perception failure;
- transition aliasing;
- insufficient rollout depth/budget;
- value propagation failure;
- successor-binding failure;
- ownership/audit failure.

After any implementation change, the failed pack remains burned and a new external run ID is mandatory.
