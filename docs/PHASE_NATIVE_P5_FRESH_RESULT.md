# FRESH-P5 — continual self-directed retention qualification

Date: 2026-10-07

## Verdict

**PASS — one-use fresh statistical qualification of bounded continual retention and selective revision.**

Workflow: `37569003124`  
Source: `d7e33bb53cf63b1e21a96463bc1eea5b728f45e7`  
Spec blob SHA: `a2bc2717f94e3840576015265116a4c7b646234b`  
Authority seed: `37569003124`  
Burned pack digest: `f7d409553cf1de82`

The entire authority-derived pack was printed before scoring. Run attempt was 1.

## Fresh lifetime result

10 independent sub-seeds were evaluated. Each lifetime:

- meta-learned the P4 exploration drive on 8 independently cold short source worlds;
- instantiated one cold target organism carrying only that drive;
- autonomously acquired 4 independently generated target worlds sequentially in one persistent EvoPhase;
- revisited earlier worlds after later learning;
- repaired one authority-selected changed world without a change flag or world ID;
- retained the three unchanged worlds;
- checkpointed the entire accumulated organism once and restored it into a newly constructed EvoPhase.

Observed:

- primary FULL: **80/80**;
- Wilson 95% CI: **[0.954182, 1.000000]**;
- per-sub-seed primary score: **[8,8,8,8,8,8,8,8,8,8]**;
- autonomous target acquisition: **40/40**;
- intermediate frozen revisits: **100/100**;
- pre-change retention: **40/40**;
- changed-world autonomous repair: **10/10**;
- post-change retention: **40/40**;
- whole-lifetime checkpoint restore: **40/40**;
- FROZEN_CHANGED: **0/10**;
- RESET_BETWEEN_WORLDS earlier-world retention: **0/30**;
- ZERO_DRIVE_PERSISTENT acquisition: **1/40**;
- NO_GROWTH_PERSISTENT acquisition: **0/40**;
- drive-weight violations during target lifetime: **0**;
- legacy graph-transition violations: **0**.

Costs:

- mean target acquisition cost: **14.875** physical interactions, SD **6.382**;
- mean changed-world repair cost: **9.400**, SD **5.700**;
- P4 source meta-tuition cost across the pack: **669** interactions.

All frozen acceptance thresholds passed. Full optimized regressions and Release build passed on the exact scored source.

## Causal interpretation

This result supports a bounded claim that one persistent phase-native organism can:

1. carry a learned exploration drive into a cold lifetime;
2. autonomously acquire several independently sampled worlds in sequence;
3. preserve earlier learned models after later learning;
4. selectively repair a contradicted world while retaining untouched worlds;
5. preserve the accumulated lifetime through one ordinary checkpoint/restart.

The controls rule out explaining the result by per-world reset, zero drive, disabled structural growth, a frozen stale model, or the legacy graph planner.

## Boundary

This does **not** establish:

- unbounded memory lifetime;
- autonomous memory compression/consolidation;
- stochastic or general POMDP continual learning;
- arbitrary domain transfer;
- autonomous invention of representational primitives;
- unrestricted concept invention;
- AGI or consciousness.

The next active architectural gate is **G10 autonomous composite concept construction**: a new reusable internal concept must be formed from acquired lower-level carrier units because no individual child is predictive enough.
