# G23 — EVIDENCE-GATED COMPOSITIONAL PERCEPTUAL FUNCTION SYNTHESIS

Status: **PREREGISTERED BEFORE G23 IMPLEMENTATION AND OUTCOME OBSERVATION**

Date: 2026-10-07
Base source: `940001406c8be475716f7ebb2812c330f9e59084` plus the already qualified G22 mechanism source lineage.

## Scientific question

Can one continuing AETERNA form a new operational perceptual predicate by **composing multiple raw descriptors**, when no single descriptor is sufficient to predict the relevant consequence, and validate that composed predicate only on later factual experience?

G22 selects one useful raw descriptor from a bounded authored descriptor family. G23 must not be solvable by any single G22 descriptor. The useful distinction must live in a new composed function over multiple descriptors.

This is a bounded synthesis mechanism experiment. It is not a claim of arbitrary program synthesis, unrestricted concept invention, world-first novelty or AGI.

## Primitive descriptor source

Use the G22 generic raw descriptor extractor unchanged:
- subthreshold amplitude bins;
- wide translation-invariant pair offsets.

G23 does not add task-specific primitive descriptor kinds.

## Compositional grammar

The native learner may synthesize only from descriptors that were actually observed in conflicting factual frames.

Frozen depth-2 grammar:

- `Atom(f)`
- `And(f1,f2)` for distinct descriptors;
- `Xor(f1,f2)` for distinct descriptors.

Programs are canonicalized by descriptor order. No evaluator feature labels, marker coordinates, state IDs, outcome labels or correct action enter program generation.

The evaluator never supplies which operator or which descriptors are useful.

This grammar is generic but authored; G23 does not claim open-ended programming.

## Candidate generation

Maintain a bounded factual discovery buffer containing:
- inherited base abstract cell;
- opaque action;
- acquired successor cell;
- raw primitive descriptor set.

When the same inherited base/action has conflicting successors, enumerate depth-2 programs over the **union of descriptors observed in those conflicting frames**.

A program is eligible only if:
1. it takes different truth values on the two discovery frames;
2. neither of its constituent atoms alone has already passed the G22 evidence gate for this base/action;
3. no simpler `Atom` program separates the candidate's future evidence at promotion time;
4. total program candidates per lifetime <=16.

Discovery observations select programs but contribute zero validation evidence.

If more than one eligible program exists, choose deterministically by:
- smallest AST node count;
- then operator order Atom < And < Xor;
- then canonical descriptor ordering.

No outcome-aware tie break is allowed after candidate birth.

## Native physical representation

For one selected binary program P:

- recruit one physical program cell representing P=true;
- recruit one physical complement cell representing P=false;
- recruit two refined-state cells;
- each refined state receives one base-state synapse plus one program-side synapse;
- raw sensing evaluates the acquired program on the current frame and supplies current only to the corresponding acquired program/complement cell;
- ordinary native transition circuits learn consequences from the refined-state cells.

The host/runtime never receives the program truth value.

The program AST and evidence state belong to PhaseNativeState/checkpoint.

## Evidence gate

Use the same future-only binary evidence gate and alpha spending contract as G21/G22:

- >=32 future anchor observations;
- >=8 observations for each side;
- >=4 side switches;
- absolute difference in successor rate >=0.60;
- log evidence >= log(16/0.01);
- no third successor;
- retire at 128 anchor observations or third successor.

In addition, at promotion:
- each constituent atom considered alone must have absolute empirical successor-rate difference <=0.25;
- the composed program must meet the >=0.60 effect threshold.

This is the central G23 anti-shortcut criterion.

## Deterministic witness

The inherited concept abstraction sees one junction state.

Current raw frame contains two weak binary primitive cues A and B, each encoded by different G22 subthreshold amplitude bins. Their absolute positions vary between learning and scoring.

The world consequence for anchor action X is determined by **XOR(A,B)**:
- 00 -> dead;
- 01 -> goal;
- 10 -> goal;
- 11 -> dead.

A second opaque action Y has the complementary consequence.

Training/scoring schedules are balanced so:
- P(goal | A=0) = P(goal | A=1) = 0.5;
- P(goal | B=0) = P(goal | B=1) = 0.5;
- therefore no single cue can exceed the frozen atom-effect bound;
- XOR(A,B) perfectly separates the two successor classes.

The learner is not told XOR. It sees only raw frames, actions and actual acquired successor states.

At least one nuisance raw descriptor varies independently and may appear in candidate unions.

## Scoring

After promotion:
- freeze learning;
- score 64 held-out translated junction decisions per opaque motor permutation;
- all four A/B combinations appear equally;
- marker positions differ from discovery/training positions;
- goal cue is unchanged.

Total descriptive FULL decisions: 128.

Matched controls:
1. old inherited/memoryless abstraction;
2. best single-atom G22 readout computed from the same acquired factual supports;
3. forced AND-only grammar;
4. program-cell weight lesion;
5. program-cell pi-phase shift;
6. exact restore;
7. unrelated nuisance-program lesion;
8. checkpoint/restart with fresh current sensing.

## Mechanism acceptance

PASS requires:

- no G23 program candidate before a factual successor collision;
- discovery observations contribute zero validation samples;
- selected useful program is structurally a depth-2 composition, not Atom;
- useful program promotes only after frozen future-only evidence gate;
- at promotion, each constituent atom effect <=0.25 while program effect >=0.60;
- FULL >=60/64 in each motor permutation;
- inherited/memoryless comparator <=40/64 per permutation;
- best single-atom comparator <=40/64 per permutation;
- AND-only matched control <=40/64 in the XOR witness;
- all four A/B combinations score >=15/16 per permutation;
- program-cell weight lesion and pi shift each make affected refined readout unavailable;
- exact restore recovers correct action and learned fingerprint;
- unrelated nuisance-program lesion preserves target decision;
- checkpoint/restart preserves program/evidence/state and fresh current raw sensing restores discrimination;
- at least one irrelevant composition candidate is retired/unpromoted under future evidence;
- legacy graph transitions 0;
- dedicated table composites remain unused;
- source guard finds no XOR truth table, A/B labels, marker coordinates, correct action map, host graph/search or evaluator state IDs in production;
- Human Protection, G20, G21, G22 and full ordinary regressions PASS;
- Release build PASS.

## Fresh qualification boundary

A later fresh G23 protocol must independently randomize:
- primitive descriptor identities and positions;
- operator target across at least AND/XOR;
- nuisance descriptors;
- opaque motor/successor assignments;
- inherited hierarchy and goal state;
- factual schedule/order.

No fresh qualification is consumed by this deterministic mechanism stage.

## Explicit limits

- Grammar depth is fixed at 2.
- Operators are programmer-authored.
- Primitive descriptor family remains the bounded G22 family.
- Deterministic fully observed consequences only.
- This is not arbitrary symbolic program induction, semantics, language concept learning or AGI.
