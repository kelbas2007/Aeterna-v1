# G12 — SELF-TRIGGERED PHYSICAL ABSTRACTION ESCALATION

Status: **PRE-REGISTERED BEFORE G12 IMPLEMENTATION**

Date: 2026-10-07

## Question

Can one Aeterna-v1 organism decide from factual residual evidence **when an existing abstraction level is no longer sufficient**, and autonomously route learning to a higher physical concept level without an evaluator/task-boundary signal or a host call saying "learn recursively now"?

G12 is not about whether depth-2 recursion can work; FRESH-G11 already established that. G12 asks whether the organism can **trigger the escalation itself**.

## Architectural requirement

The ordinary learning loop receives only:

- raw sensory raster;
- executed opaque motor ID;
- factual Need.

There is one adaptive concept-learning entry point.

The evaluator must not tell cognition:

- which abstraction level to update;
- when a task phase changes;
- which lower concepts are relevant;
- whether a new composite should be created;
- the target pair/class.

The internal routing rule may use only already-acquired physical concept activity, physical action-conditioned evidence, support, residual/predictiveness thresholds and structural capacity.

## Required behavior

The same persistent organism must show this sequence:

1. acquire reusable physical level-1 concepts from raw relations;
2. experience a regime where those existing concepts individually predict a later action well;
3. **do not** allocate/promote a higher concept while the child explanation is adequate;
4. receive later contradictory/balancing facts, with no change flag;
5. detect that individual child evidence has become insufficient;
6. begin physical level-2 pair learning automatically;
7. promote only the useful higher-level combinations;
8. use them on held-out raw scenes.

The evaluator never calls a "recursive tuition" API in FULL.

## Generic adaptive routing

A new generic API may be enabled once at organism construction, analogous to enabling plasticity.

For every factual raw scene:

1. recognize/acquire primitive relation atoms;
2. determine which previously promoted physical concepts are active;
3. if fewer than two promoted concepts cover the scene, update the lower physical level;
4. if two or more promoted concepts are active:
   - update their own action-conditioned physical evidence;
   - test whether their evidence for the current action is sufficiently supported and individually strong;
   - if an existing child explanation is strong, do **not** recruit a higher pair;
   - if all active children are sufficiently supported but weak, route the unexplained factual residual into a physical higher-level candidate;
   - promote that candidate only under the already frozen support/joint-evidence/weak-child rules.

No dedicated table composite action readout or legacy planner/search is allowed in this path.

## Development world

Use the same eight generic local relation offsets as G11 and acquire four disjoint physical level-1 concepts.

### Adequate-child regime

Before any level-2 structure exists:

- each acquired L1 concept is presented alone at several absolute bindings;
- top motor channels are 2 and 3;
- L0/L1 individually support motor 2;
- L2/L3 individually support motor 3;
- support is sufficient for the child's prediction to be considered reliable.

Then present paired same-class scenes:

- {L0,L1}
- {L2,L3}

The adaptive router must observe two active L1 concepts but create/promote **zero** L2 structures because the children already explain the outcome.

### Residual regime

Without resetting the organism or supplying a change flag, switch factual statistics to the balanced G11 top task:

- class 0: {L0,L1}, {L2,L3}
- class 1: {L0,L2}, {L1,L3}

Continue enough factual experience for each individual L1 concept to become non-predictive for top actions while the pair evidence remains predictive.

The adaptive router must then autonomously begin higher-level physical learning and promote the four useful L2 concepts.

No explicit recursive-enable or recursive-observe call is permitted after the initial generic auto-abstraction capability is enabled.

## Controls

1. **FULL_AUTO**
   - one generic factual concept API;
   - self-triggered level selection/growth enabled.

2. **NO_ESCALATION**
   - identical raw experience;
   - higher-level structural escalation disabled;
   - L1 evidence still updates.

3. **ALWAYS_ESCALATE**
   - diagnostic structural-economy baseline;
   - recruits higher pair candidates whenever >=2 L1 concepts are active, regardless of child sufficiency.

4. **FROZEN_SIMPLE**
   - child evidence is frozen after the adequate-child regime;
   - tests whether revision of the existing explanation is necessary before escalation.

5. **ZERO_PHASE_ESCALATION**
   - Stage-1 L1 acquired normally;
   - phase learning disabled before residual regime.

6. **ZERO_WEIGHT_ESCALATION**
   - Stage-1 L1 acquired normally;
   - weight learning disabled before residual regime.

## Observability and structural metrics

Record:

- L1 promoted count;
- L2 candidate/promoted count after adequate-child regime;
- first factual observation index at which child evidence crosses the weak threshold;
- first observation index at which an L2 physical candidate is recruited;
- first promotion index;
- total extra recruited cells/synapses;
- max child evidence before and after residual regime;
- winning joint evidence;
- held-out success.

FULL must have:
- zero L2 candidate/promoted structures throughout the adequate-child regime;
- first L2 recruitment only after supported child evidence becomes weak.

## Causal interventions

After FULL promotion:

- necessary L1->L2 synapse lesion;
- pi phase shift;
- exact restore without retraining;
- lower atom->L1 lesion;
- unrelated L2 lesion.

## Mechanism PASS thresholds

PASS requires all:

1. Stage-1: exactly 8 primitive atom units and 4 promoted physical L1 concepts.
2. At end of adequate-child regime: **0** L2 candidates and **0** L2 promoted concepts.
3. FULL eventually promotes exactly **4** useful L2 concepts after residual evidence.
4. No L2 candidate is recruited before its active children have sufficient support and absolute current-action evidence <= **0.20**.
5. FULL held-out recursive score = **8/8**.
6. NO_ESCALATION <= **4/8**.
7. FROZEN_SIMPLE <= **4/8**.
8. ZERO_PHASE_ESCALATION <= **4/8**.
9. ZERO_WEIGHT_ESCALATION <= **4/8**.
10. ALWAYS_ESCALATE may solve, but must recruit strictly more pre-residual L2 structures than FULL.
11. Necessary L1->L2 lesion loses >=2/8 decisions.
12. Pi phase shift loses >=2/8.
13. Exact restore returns **8/8** without retraining.
14. Lower atom->L1 lesion removes the dependent target in both motor permutations.
15. Unrelated L2 lesion preserves the targeted decision in both permutations.
16. No evaluator phase/task label or recursive-tuition call exists in FULL source path.
17. G0-G11 regressions PASS.
18. Release build PASS.

## Fresh qualification after mechanism PASS

Use a new one-use authority pack with:

- >=10 sub-seeds;
- >=80 held-out decisions;
- randomized relation permutation;
- randomized L1 target matching;
- randomized motor roles;
- randomized simple-regime mapping;
- randomized balanced residual-regime hierarchy;
- randomized duration/order around the weak-evidence crossing;
- held-out absolute bindings;
- causal interventions;
- structural-economy comparison.

## Interpretation boundary

PASS would establish bounded **self-triggered abstraction escalation** under changing factual evidence.

It would still not establish arbitrary-depth autonomous recursion, open-ended concept discovery, natural-language semantics, AGI or consciousness.
