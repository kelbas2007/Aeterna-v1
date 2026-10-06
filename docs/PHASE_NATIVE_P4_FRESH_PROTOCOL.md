# FRESH-P4 — STATISTICAL QUALIFICATION OF LEARNED PHASE-NATIVE EXPLORATION DRIVE

Status: **PRE-REGISTERED BEFORE FRESH-P4 IMPLEMENTATION/RUN**

Date: 2026-10-06

## Claim

Does the P4 phase-native exploration drive, learned only from factual model gain on independently cold short source worlds, transfer by itself into cold carriers and improve autonomous acquisition on many unseen longer reset-chain worlds?

This qualifies transfer of a learned exploration drive. It does not qualify general curiosity, arbitrary domains, lifelong learning, or AGI.

## Fresh authority

The qualification obeys the one-use authority rule.

Before execution:
- source is committed;
- this protocol is committed;
- no authority-derived source or target world has been observed.

At first-attempt GitHub Actions execution:
- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- run attempt must equal 1;
- 10 deterministic sub-seeds are derived from the authority seed.

For every sub-seed, all generated source and target laws are printed and mixed into a pack digest **before any acquisition or scoring starts**.

The first observed pack is burned regardless of PASS/FAIL. Any code/spec change requires a new run ID and a new pack.

## Pack structure

Exactly 10 sub-seeds.

Per sub-seed:

### Meta-tuition source family
- 8 independently cold worlds;
- chain length sampled in 2..=3;
- three opaque motors;
- advancing motor sampled independently at every chain state;
- wrong action resets to state 0;
- terminal factual value only at the end.

The P3 selector is allowed only as the source-world bootstrap action generator.
After each source world, only the P4 drive checkpoint transfers to the next cold carrier.
No source receptor, transition circuit, decoded sensory pattern or REAL state transfers.

### Fresh target family
- 8 independently cold worlds;
- chain length sampled in 4..=5;
- advancing motor sampled independently at every chain state;
- target uses the separate target raster bank;
- 60 physical-interaction acquisition budget.

Thus total scored target N = 80.

Before every target:
- instantiate a new cold EvoPhase;
- restore only the learned drive checkpoint for that sub-seed;
- verify receptor count = 0;
- verify circuit count = 0;
- verify legacy graph transition count = 0;
- freeze drive learning;
- keep ordinary P1/P2 factual world-model learning enabled.

All target acquisition actions for FULL come from `choose_phase_native_learned_drive_action`.
The P3 hand-written selector is forbidden in the FULL target arm.

## Arms

On the exact same 80 target worlds:

1. **FULL_LEARNED_DRIVE**
   - restored learned P4 drive;
   - target drive learning frozen.

2. **ZERO_DRIVE**
   - same P4 selector and cold P1/P2 carrier;
   - P4 drive weights never trained.

3. **FRONTIER_LESION**
   - restored learned checkpoint;
   - learned REACHABLE_FRONTIER physical drive synapse set to zero after restore.

4. **ZERO_PHASE_DRIVE**
   - source meta-tuition repeated with drive phase learning disabled;
   - only resulting drive checkpoint transfers.

5. **RANDOM_ACTION**
   - seeded external uniform random action baseline;
   - diagnostic only, not cognition.

6. **P3_TEACHER**
   - hand-written P3 selector on targets;
   - ceiling/diagnostic only and cannot count as P4 success.

## Metrics

Report:
- source SHA;
- spec SHA;
- authority seed;
- pack digest;
- every generated source/target world law before scoring;
- N and success counts for all arms;
- Wilson 95% interval for FULL;
- per-sub-seed FULL success;
- mean and sample SD of target physical interactions for FULL, lesion, zero-phase and random;
- learned drive weights and drive observation count for each sub-seed;
- total source meta-tuition physical interactions;
- proof that target drive weights remain frozen;
- proof that target carriers start with zero world receptors/circuits/legacy transitions.

## Frozen PASS thresholds

FRESH-P4 PASS requires all:

1. Exactly N=80 target worlds across exactly 10 sub-seeds.
2. Every sub-seed source meta-tuition completes all 8 source worlds.
3. Learned DIRECT_UNMODELLED and REACHABLE_FRONTIER weights are both >0.05 in every sub-seed.
4. FULL_LEARNED_DRIVE succeeds in >=76/80 targets.
5. Wilson 95% lower bound for FULL is >=0.87.
6. Every sub-seed FULL succeeds in >=6/8 targets.
7. ZERO_DRIVE succeeds in <=40/80.
8. FRONTIER_LESION is at least 0.20 absolute below FULL success, or its mean physical cost is >=10 interactions worse.
9. ZERO_PHASE_DRIVE is at least 0.20 absolute below FULL success, or its mean physical cost is >=10 interactions worse.
10. RANDOM_ACTION is at least 0.20 absolute below FULL success, or its mean physical cost is strictly worse.
11. P3_TEACHER solves >=76/80 as a sanity ceiling.
12. Mean FULL target acquisition cost <=35 physical interactions.
13. Target drive weights are unchanged from their restored values in every FULL target.
14. Every FULL target begins with receptors=0, circuits=0 and legacy graph transitions=0 before first observation.
15. P4 source guard and all prior regressions pass.
16. Release build passes.

A compile/workflow/observer problem before scoring is technical. A completed fresh scored run missing any cognitive threshold is a burned scientific FAIL.

## Interpretation boundary

PASS would establish statistical transfer of a learned physical exploration drive across independently sampled short-to-long deterministic reset-chain worlds.

PASS would still not establish:
- invention of the epistemic feature vocabulary;
- noisy/stochastic exploration;
- POMDP exploration;
- continual retention across many interfering domains;
- arbitrary concept formation;
- general intelligence, AGI, or consciousness.
