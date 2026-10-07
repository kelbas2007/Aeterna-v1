# FRESH-G16 — AUTONOMOUS ABSTRACT MODEL ACQUISITION QUALIFICATION

Status: **PRE-REGISTERED BEFORE FRESH-G16 EVALUATOR/RUN**

Date: 2026-10-07

## Claim

Can a transferred learned exploration drive autonomously acquire the missing physical abstract transition model, discover delayed reward, preserve the model across restart, and then exploit it on unseen raw bindings across fresh abstract hierarchies and motor assignments?

This statistically qualifies the bounded G16 mechanism. It does not qualify arbitrary active learning or AGI.

## Frozen production behavior

The ordinary FULL selector uses only:

- the inherited P4 DIRECT_UNMODELLED feature;
- the inherited P4 REACHABLE_FRONTIER feature;
- their transferred learned physical drive weights;
- actual acquired abstract transition circuits;
- a deterministic equal-score tie-break that prefers the opaque motor with greater supported physical transition evidence elsewhere in the currently acquired abstract model.

The tie-break is not a third weighted feature and receives no world/state/depth/correct-action metadata.

## Fresh authority

First valid GitHub Actions attempt only:

- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- run attempt = 1;
- exactly 10 deterministic sub-seeds.

Before target interaction/scoring, each sub-seed prints and hashes:

- selected/permuted primitive relation set;
- four acquired L1 concept pairs;
- six L2 abstract-state pair identities;
- target world laws;
- two relevant planning motors per world;
- immediate/delayed start permutation;
- chain length;
- advancing motor at every chain state;
- raw target/held-out binding plan;
- seeded-random control seed.

The first observed pack is permanently burned.

## Source meta-drive

Use the same bounded P4/G16 source meta-drive procedure.

Only the learned drive checkpoint transfers.

No source receptor, transition circuit, decoder, REAL state, abstract hierarchy or world law transfers.

Target drive learning is frozen.

## Fresh abstract substrate

Per sub-seed, authority:

1. permutes the 16 frozen local relation offsets;
2. selects 8;
3. pairs them into 4 physical L1 concepts;
4. forms all 6 unordered pairs of those L1 concepts as promoted physical L2 abstract states.

Raw substrate:

- 20x20;
- 6 opaque motors;
- HDC dimension 192;
- local relation radius 4;
- planning horizon 8;
- discount 0.95.

Before every target world:

- restore only the drive checkpoint;
- independently acquire the authority-selected abstract hierarchy;
- assert native abstract transition circuit count = 0;
- assert legacy graph transition count = 0;
- freeze abstract representation learning;
- freeze drive learning.

## Fresh target worlds

Per sub-seed: **4 target worlds**.

Total acquisition worlds: **40**.

For every world:

- chain length sampled in 3..=5;
- choose 2 distinct opaque planning motors from all 6;
- one start motor is immediate terminal 0.55;
- the other is the delayed first motor;
- each intermediate state independently selects one of the two relevant motors as the advancing motor;
- wrong relevant motor resets to S0 with value 0;
- the four irrelevant motors also reset to S0 with value 0;
- final advancing transition yields factual value 1.0;
- episode reset is external and is represented only through the frozen physical episode-boundary mechanism.

Interaction budget: **80**.

The cognition API receives no world law or route metadata.

## FULL acquisition

For every physical interaction:

1. observe current raw factual state;
2. choose action with `choose_phase_native_abstract_learned_drive_action`;
3. execute in evaluator world;
4. receive raw factual POST + bounded factual value + episode-boundary flag;
5. update through `observe_phase_native_abstract_action_result`.

Stop acquisition when factual value 1.0 is first reached or budget is exhausted.

## Held-out exploitation

After acquisition:

- freeze transition learning;
- disable drive readout;
- score **2 unseen raw bindings** of S0;
- no transition relearning.

Total held-out FULL N = **80**.

Then checkpoint/restart the acquired organism and score the first held-out binding again.

Total restart decisions = **40**.

## Matched controls

On the exact same fresh worlds:

1. **ZERO_DRIVE**
   - same abstract substrate/transition learner;
   - drive weights zero.

2. **FRONTIER_LESION**
   - learned drive restored;
   - REACHABLE_FRONTIER physical drive synapse lesioned.

3. **DIRECT_ONLY**
   - local unmodelled action exploration only;
   - no reachable-frontier propagation.

4. **SEEDED_RANDOM**
   - seeded uniform motor action;
   - diagnostic only.

5. **NO_TRANSITION_LEARNING**
   - factual interaction occurs;
   - target transition learning disabled.

6. **NO_GROWTH**
   - structural transition recruitment disabled.

## Ownership / persistence requirements

For every FULL world:

- target starts with 0 native transition circuits;
- target starts with 0 legacy graph transitions;
- transferred drive weights remain exactly unchanged;
- every acquired target transition begins/ends on acquired L2 abstract cells;
- held-out raw bindings require no transition relearning;
- restart retains the acquired abstract transition model;
- source guard PASS.

## Reporting

Report:

- source/spec/authority/pack digest;
- all sealed hierarchy/world laws before scoring;
- FULL acquisition success /40;
- FULL held-out plan /80 and Wilson95;
- FULL restart plan /40;
- per-sub-seed acquisition /4 and held-out /8;
- interactions to first delayed reward per world;
- mean and sample SD FULL cost;
- ZERO_DRIVE reward /40;
- FRONTIER_LESION reward /40;
- DIRECT_ONLY reward /40;
- SEEDED_RANDOM reward /40 and cost;
- NO_TRANSITION_LEARNING held-out plans /80;
- NO_GROWTH held-out plans /80;
- drive-weight mutation violations;
- transition endpoint violations;
- legacy graph violations;
- restart violations;
- all six motor-role participation mask;
- source guard result;
- Human Protection regression result;
- full regression/Release result.

## Frozen PASS thresholds

FRESH-G16 PASS requires all:

1. Exactly 10 sub-seeds and 40 target worlds.
2. FULL reaches factual delayed value 1.0 in **40/40**.
3. FULL held-out delayed plan >= **76/80**.
4. Wilson 95% lower bound for held-out plan >= **0.87**.
5. Every sub-seed acquisition = **4/4**.
6. Every sub-seed held-out plan >= **6/8**.
7. Restart delayed plan >= **39/40**.
8. Mean FULL interactions to first delayed reward <= **45.0**.
9. ZERO_DRIVE reaches delayed reward in <= **10/40**.
10. FRONTIER_LESION reaches delayed reward in <= **10/40**.
11. DIRECT_ONLY reaches delayed reward in <= **14/40**.
12. NO_TRANSITION_LEARNING held-out delayed plans = **0/80**.
13. NO_GROWTH held-out delayed plans = **0/80**.
14. Drive-weight mutation violations = **0**.
15. Abstract transition endpoint violations = **0**.
16. Legacy graph violations = **0**.
17. Restart/model-retention violations = **0**.
18. All six opaque motors participate across the sealed target pack.
19. Source guard PASS.
20. Human Protection mechanism regression PASS.
21. G0-G15 / P1-P5 regressions PASS.
22. Release build PASS.

A compile/workflow failure before `FRESH_G16_SEAL` is technical. A completed authority-scored run missing any frozen threshold is a burned scientific FAIL.

## Interpretation boundary

PASS establishes statistical bounded autonomous acquisition of a physical abstract transition model followed by restart-persistent abstract planning.

PASS still does not establish:

- goal-conditioned information value;
- stochastic/POMDP active learning;
- autonomous invention of the exploration feature vocabulary;
- open-ended goal invention;
- arbitrary-world intelligence;
- AGI or consciousness.
