# P5 — CONTINUAL SELF-DIRECTED LEARNING UNDER INTERFERENCE

Status: **PRE-REGISTERED BEFORE P5 IMPLEMENTATION**

Date: 2026-10-06

## Question

Can one persistent Aeterna-v1 organism, with a previously learned P4 exploration drive, autonomously acquire several distinct worlds in sequence, retain earlier world models while later worlds are learned, selectively repair one changed world, and still recover the untouched older capabilities without evaluator task-identity labels?

P5 targets catastrophic overwrite and episodic cold-start dependence. It is not a claim of lifelong AGI.

## Persistent-organism rule

After P4 drive installation there is exactly one target `EvoPhase` instance.

The evaluator may reset the external environment to the start of another world, but must not:
- construct a fresh organism between target worlds;
- clear phase-native receptors/circuits between worlds;
- provide world/domain IDs to cognition;
- provide hidden transition laws;
- provide the correct action;
- inject transition tuples;
- reload a per-world checkpoint before ordinary target acquisition.

The organism distinguishes worlds only from raw sensory structure and its own retained state.

## Input / environment

Development witness:
- 4 sequential deterministic reset-chain worlds;
- three opaque motor IDs;
- each world length 3–4;
- wrong action resets to that world's start;
- terminal factual value only at goal;
- every world uses a distinct raw relational state family;
- absolute raster translation may change on revisit;
- advancing action is independently assigned by state and world.

No semantic world identity enters production cognition.

## Exploration state

Before the four persistent target worlds:
- meta-train the P4 drive on at least 8 independent short source worlds;
- create one cold target organism;
- restore only the learned P4 drive;
- verify target world receptors/circuits are initially zero;
- freeze P4 drive learning during the four-world continual target phase.

Thus P5 tests retained/adapted world knowledge, not further drive fitting.

## Sequential acquisition

For worlds A, B, C, D in order:

1. present only that world's raw start raster;
2. use `choose_phase_native_learned_drive_action` for every acquisition action;
3. feed only factual POST raster and factual value;
4. stop acquisition at first factual goal or 60 interactions;
5. do not clear acquired world state.

After each newly acquired world:
- freeze P1/P2 learning;
- revisit every world acquired so far at a held-out absolute translation;
- require factual goal using the acquired phase-native policy/model;
- re-enable learning only after retention evaluation.

Retention evaluation must not call P3 exploration.

## Changed-law repair

After A–D have all been acquired and retention-qualified:

- change exactly one transition in world B by inserting a new detour state;
- keep A, C and D unchanged;
- expose the same persistent organism to changed B;
- allow <=40 learned-drive acquisition interactions;
- no task label or detour solution is supplied.

Then freeze learning and require:
- revised B succeeds;
- untouched A, C, D still succeed;
- at least one pre-change B prediction/action is measurably revised;
- at least one explicitly checked unchanged transition retains its factual prediction.

## Persistence

After changed-law repair:
- create one opaque full phase-native checkpoint;
- restore into one newly constructed EvoPhase;
- current REAL state must not be restored;
- with learning frozen, A, revised B, C and D must all solve from held-out translations.

Checkpoint is tested only after the persistent organism has experienced all worlds; it is not used to swap per-world memories.

## Controls

1. **RESET_BETWEEN_WORLDS**
   - fresh cold target carrier for each A–D world;
   - receives the same learned P4 drive;
   - demonstrates acquisition competence but has no cross-world retained memory;
   - when asked to solve an earlier world without relearning, it must not be credited as retention.

2. **FROZEN_CHANGED_B**
   - clone of the persistent pre-change organism;
   - P1/P2 learning disabled during changed B;
   - tests that repair requires factual plasticity.

3. **ZERO_DRIVE_PERSISTENT**
   - one persistent carrier with zero P4 drive weights;
   - same 60-action per-world acquisition budget;
   - diagnostic matched exploration control.

4. **NO_GROWTH_PERSISTENT**
   - one persistent carrier with structural growth disabled;
   - same observations selected by its own policy; no injected tuples.

## Costs

Report separately:
- P4 meta-tuition cost;
- acquisition cost for A/B/C/D;
- retention revisit physical actions;
- changed-B repair interactions;
- number of retained receptors/circuits after each world;
- checkpoint size proxy / learned fingerprint.

## Deterministic PASS thresholds

P5 mechanism PASS requires all:

1. One persistent FULL organism is used across A–D acquisition.
2. Drive-only restore begins target phase with receptors=0 and circuits=0.
3. P4 drive learning is frozen throughout persistent target acquisition.
4. FULL reaches first goal in all 4 worlds within 60 interactions each.
5. After learning B, A still solves frozen without relearning.
6. After learning C, A and B still solve frozen.
7. After learning D, A/B/C/D all solve frozen.
8. Every retained frozen solve uses <= world_length + 1 physical actions.
9. Changed B is autonomously repaired within 40 interactions.
10. FROZEN_CHANGED_B fails changed B or is strictly worse than repaired FULL.
11. After B repair, unchanged A/C/D all still solve.
12. At least one unchanged transition preserves its factual prediction after B repair.
13. Revised B plus A/C/D all survive one full checkpoint/restore into a newly constructed EvoPhase.
14. RESET_BETWEEN_WORLDS does not possess earlier world capability when only the final cold world carrier is retained.
15. ZERO_DRIVE_PERSISTENT is strictly worse than FULL in total acquisitions or interaction cost.
16. NO_GROWTH_PERSISTENT does not match FULL 4/4 acquisition+retention.
17. Legacy graph transition count remains zero in FULL.
18. Mechanical ownership/source guards and all P0–P4 regressions PASS.
19. Release build PASS.

## Advancement rule

A deterministic PASS is only a mechanism witness.

Fresh P5 statistical qualification requires a separately preregistered one-use authority pack with randomized world laws/order and at least 80 scored retention/revision episodes.

## Interpretation boundary

P5 PASS would establish bounded continual retention and selective revision in one persistent phase-native organism across several sequential deterministic worlds.

It would still not establish:
- unbounded lifetime memory;
- optimal consolidation/replay;
- stochastic/POMDP continual learning;
- automatic invention of task boundaries;
- arbitrary domains;
- general intelligence, AGI, or consciousness.
