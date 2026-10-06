# FRESH-P3 — One-use statistical qualification of autonomous acquisition

Status: **SEALED BEFORE FIRST AUTHORITY RUN**

Date: 2026-10-06

This specification qualifies only the bounded autonomous reset-chain capability preregistered in `docs/PHASE_NATIVE_P3_PROTOCOL.md`.

## Authority

The first scored pack is valid only when:
- source is committed before the run;
- this specification is committed before the run;
- `AETERNA_FRESH_SEED == GITHUB_RUN_ID`;
- `AETERNA_SOURCE_SHA == GITHUB_SHA`;
- the specification blob SHA is logged;
- run attempt is 1.

The GitHub Actions run ID is expanded into 10 deterministic sub-seeds. The complete generated world descriptions and pack digest are printed before any organism acquisition or scoring begins.

The first observed pack is burned regardless of PASS/FAIL. Any source/spec change requires a new authority run and cannot reuse the old pack as fresh evidence.

## Pack

10 sub-seeds x 8 worlds = **80 worlds**.

For every world the authority seed determines:
- chain length in 3..=5;
- an independent opaque advancing motor in 0..3 at every chain state;
- detour insertion point after initial acquisition;
- detour reconnecting motor;
- held-out absolute raster translation;
- random-baseline seed.

The generated law is never supplied to cognition.

## Arms

### FULL
Cold EvoPhase:
- P1/P2 phase-native carrier enabled;
- no world circuits/receptors initially;
- autonomous P3 action selection;
- max 60 physical acquisition interactions;
- first reward ends initial acquisition.

Then:
- freeze learning;
- solve from the authority-derived unseen translation;
- create opaque phase-native checkpoint;
- restore it into a newly constructed cold EvoPhase;
- solve again with learning frozen.

Then changed-law phase:
- re-enable learning in the acquired organism;
- insert the unseen detour;
- max 40 additional autonomous interactions;
- freeze and solve the revised world;
- checkpoint/restore the revised state and solve again.

### DIRECT_ONLY
Same cold carrier and world, but novelty cannot propagate backward from deeper frontier receptors. It may try only unknown actions at its current receptor.

### RANDOM_ACTION
Same hidden world and 60-interaction budget. A seeded uniform random action is provided by the evaluator only as a diagnostic external baseline. It is not AETERNA cognition.

### Structural/plastic controls
On every fresh world:
- NO_LEARNING;
- NO_STRUCTURAL_GROWTH;
- ZERO_PHASE_LEARNING;
- ZERO_WEIGHT_LEARNING.

They receive no transition curriculum and interact only through their selected physical actions.

### FROZEN_CHANGED_LAW
A matched copy of the acquired pre-change organism has learning disabled and is exposed to the changed-law environment. It must not receive the detour solution.

## Metrics

Report before scoring:
- source SHA;
- spec SHA;
- authority seed;
- pack digest;
- every generated world description.

Report after scoring:
- FULL first-reward acquisition success / 80;
- FULL frozen translated exploitation success / 80;
- restored-checkpoint exploitation success / 80;
- changed-law autonomous repair success / 80;
- revised restored-checkpoint success / 80;
- DIRECT_ONLY and RANDOM_ACTION success / 80;
- each structural/plastic control success / 80;
- FROZEN_CHANGED_LAW success / 80;
- per-sub-seed counts;
- Wilson 95% interval for FULL initial acquisition;
- mean and standard deviation of physical interactions to first reward;
- mean and standard deviation of changed-law repair interactions;
- legacy graph transition count.

## Frozen PASS thresholds

FRESH-P3 PASS requires all:

1. N = 80 across exactly 10 sub-seeds.
2. FULL initial autonomous acquisition >= 76/80.
3. Wilson 95% lower bound for FULL initial acquisition >= 0.87.
4. Every sub-seed FULL initial acquisition >= 6/8.
5. Frozen translated exploitation succeeds for every initially acquired FULL world.
6. Checkpoint-restored translated exploitation succeeds for every initially acquired FULL world.
7. Mean initial acquisition cost <= 45 physical interactions.
8. DIRECT_ONLY success is at least 0.30 absolute below FULL.
9. RANDOM_ACTION success is at least 0.25 absolute below FULL.
10. None of NO_LEARNING / NO_STRUCTURAL_GROWTH / ZERO_PHASE_LEARNING / ZERO_WEIGHT_LEARNING may reach >= 50% success.
11. Changed-law autonomous repair >= 72/80.
12. Every sub-seed changed-law repair >= 6/8.
13. Mean changed-law repair cost <= 25 additional physical interactions.
14. Revised frozen exploitation and revised checkpoint-restored exploitation succeed for every repaired FULL world.
15. FROZEN_CHANGED_LAW success is strictly below learned changed-law repair.
16. Legacy graph transition count remains zero in FULL.
17. P0/P1/P2 and all prior ordinary regressions pass.
18. Release build passes.

A technical compile/workflow/observer failure is not a cognitive FAIL. A completed fresh scored run that misses any threshold is a burned scientific FAIL.

## Claim boundary

PASS would establish bounded autonomous acquisition, physical frontier seeking, factual detour repair and checkpoint persistence in this deterministic reset-chain family.

It would not establish general autonomous science, stochastic-world reasoning, arbitrary concept formation, language, AGI or consciousness.
