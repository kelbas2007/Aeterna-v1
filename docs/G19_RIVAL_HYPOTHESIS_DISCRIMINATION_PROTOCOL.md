# G19 — GOAL-RELEVANT PHYSICAL RIVAL-HYPOTHESIS DISCRIMINATION

Status: **PRE-REGISTERED BEFORE G19 IMPLEMENTATION**

Date: 2026-10-07

## Why G19 exists

G18 can choose which *unknown* fact matters for the current goal.

G19 introduces a different epistemic condition:

- the transition is not simply unknown;
- AETERNA already carries **multiple supported physical predictions** for the same
  acquired abstract state + opaque action;
- the predictions came from contradictory factual prior experience;
- a new physical experiment must distinguish them;
- the factual result must suppress the contradicted physical prediction;
- planning must then use the revised model.

This is a bounded first step from frontier exploration toward causal hypothesis testing.

## Scope of the G19 claim

G19 qualifies **rival transition hypotheses**, not yet a coherent arbitrary world-theory object.

A rival hypothesis is an existing supported phase-native transition circuit with:

- the same acquired abstract pre-cell;
- the same opaque motor;
- a different acquired abstract successor cell.

The rival predictions themselves therefore reside in ordinary physical transition
cells/synapses already qualified by P1/G15.

No host-side `WorldHypothesis` table is used by G19 production cognition.

## Architectural causal chain

```text
raw current observation
 -> acquired current abstract cell

raw requested goal
 -> acquired goal abstract cell
 -> physical backward goal relevance

same pre + same motor
 -> multiple supported physical successor predictions
 -> goal-relevance disagreement between predicted successors
 -> choose maximally goal-discriminating opaque action

execute factual experiment
 -> factual successor abstract cell
 -> strengthen matching physical prediction
 -> physically suppress contradicted same-action predictions
 -> reset/present factual start observation
 -> goal-conditioned physical replanning
```

## Rival formation

Development tuition uses factual prior episodes only.

For one ambiguous acquired start state S0, prior factual experience has produced
two supported successor circuits for PROBE_A and two for PROBE_B.

No hidden-context/world label is stored.

The two rival successors of PROBE_A differ strongly in relevance to GOAL_A but are
equally irrelevant to GOAL_B.

The two rival successors of PROBE_B differ strongly in relevance to GOAL_B but are
equally irrelevant to GOAL_A.

Thus:

- for requested GOAL_A, PROBE_A is informative and PROBE_B is not;
- for requested GOAL_B, PROBE_B is informative and PROBE_A is not.

Both actions contain real physical predictive disagreement; only one disagreement
matters to the current goal.

## Development abstract states

Use the frozen 20x20 physical abstraction substrate.

Acquired L2 states:

- S0 = ambiguous current state;
- A_GOOD = one PROBE_A prediction, physically near GOAL_A;
- A_DEAD = other PROBE_A prediction, goal-A irrelevant;
- B_GOOD = one PROBE_B prediction, physically near GOAL_B;
- B_DEAD = other PROBE_B prediction, goal-B irrelevant;
- FALL_A = known fallback state toward GOAL_A;
- FALL_B = known fallback state toward GOAL_B;
- GOAL_A;
- GOAL_B.

All evaluator names are external aliases only.

## Known goal topology

Supported zero-valued physical transitions include:

- A_GOOD -> GOAL_A;
- B_GOOD -> GOAL_B;
- FALL_A -> GOAL_A;
- FALL_B -> GOAL_B;
- S0 -> FALL_A through FALLBACK_A;
- S0 -> FALL_B through FALLBACK_B;
- self-loops / neutral transitions needed to keep all motors physically represented.

A_DEAD and B_DEAD do not physically lead to either requested goal within horizon.

All factual transition outcome values are 0.

## Prior contradictory factual experience

Before the test episode, ordinary factual transition learning observes:

- S0 + PROBE_A -> A_GOOD;
- S0 + PROBE_A -> A_DEAD;
- S0 + PROBE_B -> B_GOOD;
- S0 + PROBE_B -> B_DEAD.

Therefore both PROBE_A and PROBE_B have two supported physical rival successor circuits.

No experiment result for the current episode has yet been seen.

## Current hidden factual law

The evaluator selects one current law per episode.

For the requested goal's relevant probe, the actual successor alternates across
development cases between GOOD and DEAD.

This matters because after discrimination:

- GOOD result should make the probe route preferable to the longer fallback;
- DEAD result should suppress the false GOOD prediction and make fallback preferable.

Thus success requires belief revision, not merely selecting a probe.

## Goal-relevant disagreement

For each opaque action leaving current abstract cell:

1. gather supported physical successor predictions;
2. compute current raw-goal relevance for each predicted successor through the
   known physical model;
3. epistemic discrimination score is the spread between the strongest and weakest
   goal relevance among distinct supported successor predictions;
4. choose the action with the largest positive spread.

Tie-break may use only generic supported physical evidence and opaque motor index.

No state ID, goal ID, hidden-law ID, correct probe, route table or evaluator hint.

## Factual rival revision

After executing the chosen probe:

- the matching transition circuit is reinforced through ordinary factual learning;
- supported same-pre/same-action circuits whose successor differs from factual POST
  receive a physical contradiction update;
- contradiction must reduce their executable conductance below supported-prediction
  threshold;
- no rival is removed by host list deletion;
- revision is visible in actual synapse weights / circuit revision metadata.

## Development score

Test:

- two opaque motor permutations;
- two requested goals;
- both factual outcomes for the requested probe (GOOD and DEAD).

Total FULL discrimination episodes: **8**.

Each episode starts from the same frozen rival model clone.

After one physical probe, externally present a fresh factual S0 observation and plan
to the requested raw goal.

## Controls

1. **FULL_GOAL_DISAGREEMENT**
   - uses raw current + raw goal;
   - physical rival predictions;
   - goal-relevance disagreement.

2. **NOVELTY_ONLY**
   - ignores rival disagreement;
   - chooses by ordinary goal-conditioned unknown/frontier machinery.
   - Since both probes are already modeled, it must not receive a hypothesis signal.

3. **DISAGREEMENT_NO_GOAL**
   - chooses action with largest physical successor disagreement but ignores goal relevance.
   - Cannot reliably pick the requested-goal probe.

4. **WRONG_GOAL**
   - supply the opposite raw goal cue;
   - must choose the opposite goal's discriminating probe.

5. **BROKEN_GOAL_RECOGNITION**
   - lesion physical lower abstraction beneath goal cue.

6. **BROKEN_RELEVANCE_ROUTE**
   - lesion a known physical transition carrying relevance from requested goal to GOOD successor.

7. **PI_PHASE_RELEVANCE_ROUTE**
   - shift same route synapse by pi.

8. **NO_RIVAL_REVISION**
   - experiment may execute but contradicted prediction is not suppressed.
   - DEAD-outcome replanning must fail often enough to establish revision necessity.

9. **BROKEN_RIVAL_PREDICTION**
   - lesion one rival prediction before probe selection.
   - discrimination signal for that action must collapse.

10. **RESTORE**
    - exact restore of lesion/phase intervention recovers selection without retraining.

11. **IRRELEVANT_RIVAL_LESION**
    - lesion rival only on competing-goal probe;
    - requested-goal experiment remains correct.

## Ownership requirements

G19 production selector/revision may use only:

- physically recognized current/goal abstract cells;
- supported phase-native transition circuits;
- actual successor cells;
- actual synapse conductance/coherence;
- circuit support/revision;
- generic motor indices.

It may not use:

- `EvoEpistemicState`;
- `WorldHypothesis`;
- `HypothesisPrediction`;
- host graph/search;
- evaluator state/goal/law labels.

## Mechanism PASS thresholds

PASS requires all:

1. FULL selects requested-goal discriminating probe in **8/8**.
2. Factual result suppresses contradicted same-action rival in **8/8**.
3. GOOD-outcome post-probe goal plan chooses probe shortcut in **4/4**.
4. DEAD-outcome post-probe goal plan chooses known fallback in **4/4**.
5. WRONG_GOAL selects opposite-goal probe in **8/8**.
6. NOVELTY_ONLY requested-probe score <= **2/8**.
7. DISAGREEMENT_NO_GOAL requested-probe score <= **4/8**.
8. BROKEN_GOAL_RECOGNITION requested probe <= **2/8**.
9. BROKEN_RELEVANCE_ROUTE requested probe <= **4/8**.
10. PI_PHASE_RELEVANCE_ROUTE requested probe <= **4/8**.
11. NO_RIVAL_REVISION DEAD-outcome correct replanning <= **1/4**.
12. BROKEN_RIVAL_PREDICTION collapses that action's discrimination in both motor permutations.
13. Exact RESTORE returns FULL probe selection in **8/8**.
14. IRRELEVANT_RIVAL_LESION preserves requested probe in both motor permutations.
15. All rival circuits begin/end on acquired abstract cells.
16. REAL changes only through factual interaction APIs.
17. Goal planning and probe scoring do not mutate learned fingerprint.
18. Legacy graph transitions = 0.
19. Source guard PASS.
20. Human Protection v1.1 regression PASS.
21. G0-G18 / P1-P5 regressions PASS.
22. Release build PASS.

## Fresh qualification

A separate one-use fresh G19 pack must randomize:

- abstraction hierarchy;
- ambiguous start state;
- goal states;
- GOOD/DEAD/fallback state identities;
- probe/fallback opaque motor roles;
- prior rival tuition order;
- current factual law;
- raw tuition/held-out bindings.

At least:

- 10 sub-seeds;
- 80 goal-conditioned rival-discrimination episodes;
- balanced GOOD/DEAD factual outcomes;
- Wilson95 interval;
- fresh causal lesions/revision controls.

## Interpretation boundary

PASS establishes bounded physical rival-prediction discrimination and factual revision.

It does not yet establish:

- coherent multi-variable world theories;
- spontaneous hypothesis invention beyond contradictory transition experience;
- Bayesian uncertainty calibration;
- stochastic/POMDP hypothesis testing;
- unrestricted scientific reasoning;
- AGI or consciousness.
