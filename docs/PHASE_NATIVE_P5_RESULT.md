# P5 — continual self-directed learning under interference

Date: 2026-10-06

## Verdict

**P5 deterministic mechanism PASS after preserving an initial real continual-readout failure.**

P5 tests one persistent phase-native organism across several sequential worlds. It does not construct a fresh target carrier between worlds and does not receive evaluator world/task IDs.

Protocol preregistration: `docs/PHASE_NATIVE_P5_PROTOCOL.md` at commit `4d8902cba98fa377ec62ad4dda3e2b5bf026db59`.

## Preserved failure sequence

The first P5 execution did not pass.

### Run 37532474106

Two distinct problems appeared:

1. a test-only source guard falsely matched its own forbidden-token literal; this was a technical observer bug;
2. the substantive continual test failed with:
   `domain 0 forgotten after learning domain 1`.

The second failure was retained and investigated rather than weakening the P5 criterion.

### Diagnostic runs

Run `37533268179`:
- maximum cross-domain carrier-trace similarity = **0.077763** at match threshold 0.97;
- therefore A/B were not being merged by representation aliasing;
- A still failed after B.

Run `37533755925`:
- direct P1 planning from A start still selected the correct first motor 1;
- predicted value remained 0.9025;
- P2 ordinary action+prediction loop abstained before action;
- therefore the retained policy/value path survived while forward readout failed.

Run `37534252495`:
- after learning B, **0 of 606** tracked pre-existing A/drive/decoder synapses changed;
- A forward readout still failed;
- this ruled out destructive overwrite of A synaptic parameters.

Run `37534474528`:
- disabling newly added B synapses did not restore the old A forward readout;
- the remaining interfering state was the shared cell substrate rather than B transition content.

## Root cause

In `EvoPhase::load_real`, every active raw sensory channel historically received a permanent `+0.01` phase increment on every factual observation.

P2 decoder phase offsets are learned relative to the physical sensory-cell phase. Across later worlds, the shared sensory reference therefore drifted even though old decoder synapses were untouched. Old knowledge remained structurally present but lost phase coherence during forward readout.

This is a continual-substrate defect: observation history was rewriting the reference frame against which long-lived synapses had been learned.

## Fix

Production commit: `797e366d64a186bc7c36d283f84fff3dfe57e8f7`.

In phase-native mode:
- REAL sensory input still changes sensory-cell **charge**;
- sensory-cell intrinsic phase is kept stable;
- adaptation remains in learned synaptic weights / phase offsets.

The legacy non-native carrier keeps its historical small sensory phase-adaptation behavior.

This does not add a task ID, world ID, replay table, route cache, memory bank, or evaluator hint.

## Passing mechanism run

Workflow: `37534699260`  
Tested source: `de2f931b5a6b03995f62f8723011cfca89581be0`

Observed:

- cross-domain max trace similarity: **0.077763**;
- pre-existing A synapses mutated while learning B: **0 / 606**;
- meta-drive tuition cost: **61** physical interactions;
- persistent target acquisition costs A/B/C/D: **[9, 24, 12, 20]**;
- total persistent acquisition cost: **65**;
- frozen retention revisit actions: **34**;
- changed-B autonomous repair cost: **9**;
- receptor count after A/B/C/D: **[4, 9, 13, 18]**;
- circuit count after A/B/C/D: **[7, 19, 27, 38]**;
- ZERO_DRIVE_PERSISTENT: **0/4**, total budget spent 240;
- NO_GROWTH_PERSISTENT: **0/4**;
- legacy graph transition count remained zero;
- source guard PASS;
- one full phase-native checkpoint restored the post-repair organism into a newly constructed EvoPhase and all four worlds remained solvable;
- full optimized regression suite PASS;
- Release build PASS.

The final learned fingerprint reported by the witness was `8307d12ebd207c6d`.

## What P5 establishes

Within the fixed deterministic development witness:

1. one persistent organism can autonomously acquire several distinct worlds sequentially;
2. later learning need not erase earlier acquired world models;
3. retention is not achieved by swapping per-world checkpoints;
4. one changed world can be selectively repaired while untouched worlds remain usable;
5. the whole accumulated phase-native state survives one ordinary checkpoint/restart;
6. a learned P4 exploration drive remains useful across this continuous lifetime;
7. stable phase-native sensory reference is necessary for long-lived decoder coherence.

## What P5 does not establish

This mechanism result is not yet statistical continual-learning qualification.

It does not establish:
- unbounded lifetime retention;
- large-scale memory consolidation;
- stochastic/POMDP continual learning;
- autonomous discovery of task boundaries;
- cross-domain representation invention;
- absence of interference at larger scale;
- AGI or consciousness.

The next valid P5 claim requires a one-use fresh authority pack with randomized world laws/order and at least 80 scored retention/revision episodes.
