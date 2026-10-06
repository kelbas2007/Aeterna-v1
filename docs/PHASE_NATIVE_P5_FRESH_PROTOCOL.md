# FRESH-P5 — STATISTICAL QUALIFICATION OF CONTINUAL SELF-DIRECTED RETENTION

Status: **PRE-REGISTERED BEFORE FRESH-P5 IMPLEMENTATION/RUN**

Date: 2026-10-06

## Claim

Can one persistent phase-native Aeterna-v1 organism, carrying only a previously learned P4 exploration drive at target start, autonomously acquire several independently generated worlds in sequence, retain earlier world models after later learning, selectively repair one changed world, and preserve the accumulated lifetime through one full checkpoint/restart?

This qualifies bounded continual retention/revision. It does not qualify unbounded lifelong learning or AGI.

## Fresh authority

The first valid qualification attempt uses an external GitHub Actions authority.

Before the run:
- production source is committed;
- this protocol is committed;
- evaluator implementation is committed;
- no authority-derived pack has been observed.

At first-attempt execution:
- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- `GITHUB_RUN_ATTEMPT == 1`;
- exactly 10 sub-seeds are derived from the authority seed.

For every sub-seed the evaluator generates and prints the entire source/target law pack, target encounter order, changed-world index and held-out translations **before any training or scoring starts**. These values are mixed into a pack digest.

Once printed, the pack is permanently burned regardless of PASS/FAIL.

## Per-sub-seed lifetime

Each sub-seed creates its own independent lifetime.

### Learned-drive meta-tuition

- 8 independently cold source worlds;
- source length sampled from 2..=3;
- 3 opaque motor IDs;
- advancing motor independently sampled for every state;
- P3 bootstrap selector is allowed only in source meta-tuition;
- only the P4 drive checkpoint transfers between source organisms.

Then instantiate one cold target EvoPhase and restore **only** the learned drive.

Required before target experience:
- phase-native target receptors = 0;
- target transition circuits = 0;
- legacy graph transitions = 0;
- P4 drive learning is frozen.

### Persistent target lifetime

Generate 4 target worlds:
- chain length sampled from 3..=4;
- every state receives an independently sampled advancing motor;
- wrong action resets to that world's state 0;
- terminal factual value appears only at the goal;
- each world is rendered through a distinct raw relational family;
- no world/task identity is passed to cognition.

The encounter order of the four raw world families is randomly permuted per sub-seed.

The same target EvoPhase remains alive for all four acquisitions.

Per target world:
- acquisition budget = 60 physical interactions;
- every acquisition action comes from `choose_phase_native_learned_drive_action`;
- only factual POST raster and factual terminal value are supplied;
- no injected transition tuple or route is supplied.

After each new world, learning is frozen and every world acquired so far is revisited at an authority-derived held-out translation. These intermediate revisits are diagnostic and must all succeed, but they are not part of the primary N=80 score.

## Primary pre-change retention score

After all 4 worlds are acquired:
- freeze P1/P2 learning;
- solve all 4 worlds at fresh authority-derived translations;
- these 4 solves are scored.

Across 10 sub-seeds: **40 primary pre-change retention episodes**.

## Fresh changed-law phase

For every sub-seed, the authority pack selects one of the four acquired worlds.

That world changes by inserting one previously unseen detour state after one nonterminal transition:
- the original advancing motor now reaches the detour;
- an authority-derived opaque motor from the detour reaches the old successor;
- all other worlds remain unchanged.

The same persistent organism:
- receives no change flag or world ID;
- gets <=40 learned-drive acquisition interactions to repair the changed world;
- drive learning remains frozen;
- ordinary phase-native world-model learning is enabled.

A matched **FROZEN_CHANGED** clone receives the changed world with P1/P2 learning disabled.

After repair, freeze learning and solve all four worlds again.

These 4 post-change solves are scored. Across 10 sub-seeds: **40 primary post-change retention/revision episodes**.

Thus primary FULL score is exactly **N=80**.

## Checkpoint requirement

After post-change evaluation:
- create one full phase-native checkpoint from the accumulated organism;
- restore it into a newly constructed EvoPhase;
- REAL state must start empty;
- learning remains frozen;
- solve all four worlds once more at independently authority-derived translations.

Checkpoint-restored solves are a separate required persistence metric (N=40), not included in primary N=80.

No per-world checkpoint swap is permitted.

## Controls

### RESET_BETWEEN_WORLDS

For each target world create a fresh carrier with the same learned drive. Retain only the carrier trained on the final encountered world. It is then asked to solve the other three worlds without relearning.

This tests whether primary retention requires one persistent organism.

### ZERO_DRIVE_PERSISTENT

One persistent target carrier with zero drive weights and identical 60-action budgets.

### NO_GROWTH_PERSISTENT

One persistent target carrier with structural growth disabled.

### FROZEN_CHANGED

Clone the fully acquired pre-change FULL organism. Disable P1/P2 learning before presenting the changed world. It must not receive the repaired checkpoint.

## Statistics and reporting

Report:
- source SHA;
- spec SHA;
- authority seed;
- pack digest;
- all generated source/target laws, encounter orders, changed-world choices and translations before scoring;
- primary FULL successes / 80 and Wilson 95% interval;
- per-sub-seed primary successes / 8;
- pre-change retention successes / 40;
- post-change retention successes / 40;
- checkpoint-restored successes / 40;
- changed-world repair success / 10;
- FROZEN_CHANGED success / 10;
- RESET_BETWEEN earlier-world retention / 30;
- ZERO_DRIVE target acquisitions / 40;
- NO_GROWTH target acquisitions / 40;
- mean and sample SD of FULL target acquisition cost;
- mean and sample SD of changed-world repair cost;
- meta-drive tuition cost;
- receptor/circuit count after each target acquisition;
- proof target drive weights remain frozen;
- proof legacy graph transitions remain zero.

## Frozen PASS thresholds

FRESH-P5 PASS requires all:

1. Exactly 10 sub-seeds and exactly N=80 primary FULL episodes.
2. All 40 FULL target worlds are autonomously acquired within 60 interactions.
3. Every intermediate frozen revisit succeeds.
4. Pre-change retention = **40/40**.
5. Changed-world autonomous repair succeeds in **>=9/10** sub-seeds.
6. Post-change retention succeeds in **>=39/40** episodes.
7. Primary FULL total succeeds in **>=79/80**.
8. Wilson 95% lower bound for primary FULL is **>=0.93**.
9. Every sub-seed primary score is **>=7/8**.
10. Checkpoint-restored retention succeeds in **>=39/40** episodes.
11. FROZEN_CHANGED succeeds in **<=2/10** changed worlds.
12. RESET_BETWEEN_WORLDS earlier-world retention succeeds in **<=6/30**.
13. ZERO_DRIVE_PERSISTENT autonomously acquires **<=20/40** target worlds.
14. NO_GROWTH_PERSISTENT autonomously acquires **<=4/40** target worlds.
15. Mean FULL target acquisition cost is **<=35** physical interactions.
16. Mean changed-world repair cost is **<=30** physical interactions across successful repairs.
17. Target P4 drive weights remain unchanged throughout each target lifetime.
18. Legacy graph transition count remains zero throughout FULL.
19. Source guard / ownership checks and P0–P5 deterministic regressions PASS.
20. Release build PASS.

A completed authority-scored run missing any threshold is a scientific FAIL and the pack remains burned. Workflow/compile errors before pack generation are technical failures only.

## Interpretation boundary

PASS would establish bounded statistical continual retention and selective revision in one persistent phase-native organism across four sequential randomized deterministic worlds.

It would still not establish:
- unbounded memory lifetime;
- autonomous memory compression/consolidation policy;
- stochastic/POMDP continual learning;
- arbitrary task-boundary invention;
- arbitrary domains;
- general intelligence, AGI or consciousness.
