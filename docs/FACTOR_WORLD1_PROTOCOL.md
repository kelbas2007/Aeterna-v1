# FACTOR-WORLD-1 — independent object properties, first development diagnostic

Date: 2026-10-10. Research branch: \`research/beyond-intel4\`.
Status: OPEN DEVELOPMENT DIAGNOSTIC. Not a frozen-source or authority gate.

## Problem
The prior COMPLEX-WORLD-FRESH1 PASS 6/6 learned whole-state causal edges in
one 14-state topology. A key/power/gate narrative is not an independently
factorized state representation. This test asks whether the same continuing
organism can reuse motor effects when key, battery, location and supply co-occur
for the first time. No preloaded transition table or action-role semantics is
passed to EvoPhase.

## Environment and heldout split
The evaluator owns four independent variables:
\`room ∈ {entrance, charger, exit}\`,
\`key ∈ {0,1}\`, \`power ∈ {0,1}\`,
\`supply ∈ {0,1}\`. There are 24 recognizable complete sensory classes.
The 24-to-sensory and six opaque action mappings are freshly permuted in each
arm. \`get-key\` and \`get-supply\` preserve every other property;
\`move\` and \`return\` preserve inventory and battery; \`charge\` changes
battery only; \`gate\` requires key and power, consumes power and preserves
supply. Action roles stay hidden from the organism.

Each of 4 independent organism lifetimes receives interleaved tasks in TWO
training contexts, with identical underlying motor effects:
(1) key available, supply absent, goal exit with key;
(2) supply available, key absent, goal charger with supply.
There are 160 episodes/context, at most 18 physical actions/episode.
The environment never allows joint \`key=1,supply=1\` in training; the
test asserts this split directly. This is a controlled curriculum of *world
availability and externally specified goals*, NOT motor/action tuition.

Heldout has BOTH objects available, starts with neither acquired and
requires exit with key AND supply after battery charging. The external
oracle's five-action path verifies the challenge is physically solvable,
but is used exclusively for scoring/feasibility. The organism receives the
raw goal and actual protected feedback; the model is restored and FROZEN.
The first causal plan must be available; at least 3 executed steps must
match readout plans; the goal must be reached within 14 protected actions
with zero unavailable or blocked actions. All 4 must pass for development
PASS. Every run must print the raw FAIL rather than relabeling CI success.

## Important interpretation
A pass would demonstrate a narrow compositional transfer under prelearned
24-class visual recognition; it would still not prove spontaneous
discovery of factor identities, SNN-invented search, dynamic battery
management, novel sensory ontology, or open-world intelligence.
A failure localizes the missing factor-conditioned causal rules; it
must NOT be repaired in this first diagnostic by injecting full-state
heldout examples, assigning action labels or making a hidden path available.

## Follow-on experiment (distinct preregistration required)
Without showing the heldout combination, learn factorwise invariant
PRE/action/POST effects, conditional requirements and preserved properties
inside the carrier. Retain contradictory facts, check causal link
lesion/restore and checkpoint continuity. Evaluate a NEW withheld
combination/topology and mid-episode battery discharge with true
online belief revision and protective execution, against:
whole-state graph, random exploration and evaluator-only symbolic oracle.
