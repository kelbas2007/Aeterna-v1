# U1 — CARRIER-OWNED META-CONTROL COMPETITION

Status: **PRE-REGISTERED BEFORE U1 IMPLEMENTATION**
Date: 2026-10-07

## Purpose

Test the first architectural claim of the post-INTEL redesign:

> when several already-qualified cognitive operations are simultaneously plausible, the winning operation is selected by one shared physical competition over task-agnostic proposal fields rather than by Rust module priority.

U1 is opt-in. Existing AETERNA-v1 default behavior and its closed INTEL-1 FAIL record remain unchanged.

## Proposal abstraction

A cognitive proposal contains:

- opaque operation reference;
- opaque external motor action when applicable;
- carrier-derived goal-value estimate;
- carrier-derived epistemic value;
- carrier-derived contradiction pressure;
- carrier-derived confidence/evidence quality;
- bounded resource cost.

Proposal source/module identity may be present only for diagnostics. It MUST NOT enter the competition score or tie-break.

## Physical competition

Enable a dedicated phase-native meta-control state with:

- one physical feature cell per proposal field;
- one physical utility cell;
- learned phase-sensitive synapses from feature cells to utility;
- no task/world/module-specific weights.

All meta weights start at zero.

A generic factual meta-learning API updates the physical weights from whether the selected cognitive operation produced bounded useful progress:
- factual goal progress;
- factual structural information gain;
- factual contradiction resolution;
- bounded cost penalty.

The API receives proposal fields plus factual utility outcome, but no world ID, module ID or correct-operation label.

## Development training

Train the common meta weights on a small source curriculum containing mixed proposal situations. Only the physical meta-control checkpoint transfers.

Target U1 begins with new proposal tuples and opaque operation IDs.

## Target mixed-competition witness

At least three proposal classes are simultaneously available:

1. general epistemic acquisition;
2. rival-discrimination probe;
3. representation-refinement experiment.

Across target cases, the useful operation changes because the proposal fields change, not because its class/name changes.

PASS requires:
- FULL selects the utility-dominant operation in 12/12 target competitions;
- a fixed authored module-priority baseline <=6/12;
- a source-ID-only baseline <=6/12;
- zero-meta-weight control <=4/12;
- all three operation classes win at least twice.

## Causal ownership controls

For four target competitions:
- lesion one necessary learned meta-control synapse -> original winner lost in >=3/4;
- pi phase shift same synapse -> original winner lost in >=3/4;
- exact restore -> original winner returns 4/4;
- unrelated meta synapse lesion preserves winner >=3/4.

Competition must leave REAL factual state unchanged.

## Source guard

Production competition source must not contain:
- ContextualRefinement;
- PerceptualRefinement;
- CompositionalRefinement;
- RivalDiscrimination;
- GeneralEpistemic;
- GoalDirectedAction;
- world/task/evaluator IDs;
- hard-coded proposal ordering by source;
- correct_operation.

It may operate only on numeric proposal fields, opaque proposal IDs/actions, physical meta cells/synapses and learned weights.

## Integration witness

An opt-in runtime path may enumerate proposals from existing mechanisms, but:
- enumeration order must be randomized/reversed in matched tests without changing winner;
- source identity must not be visible to the competition function;
- winner must equal direct competition over the same opaque proposal set.

## Regression requirements

- closed v1 INTEL verdict documents unchanged;
- existing G20-G23 and Human Protection regressions PASS;
- default ScientificRuntime path unchanged unless unified competition is explicitly enabled;
- Release build PASS.

## Interpretation

U1 PASS establishes bounded carrier-owned arbitration among heterogeneous cognitive proposals.

It does NOT establish:
- full hypothesis ecology;
- internal thought sequencing;
- learned proposal generation;
- autonomous goal invention;
- INTEL-2 or AGI.

Those require U2/U3 and then a new frozen INTEL-2.
