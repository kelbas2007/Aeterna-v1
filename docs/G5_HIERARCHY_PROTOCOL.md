# G5 — HIERARCHICAL REUSE OF ACQUIRED MACROS

Status: **PRE-REGISTERED BEFORE G5 IMPLEMENTATION**

Date: 2026-10-06

## Question

Can an already acquired EvoPhase macro become a child building block of a newly acquired higher-level macro, so that later behavior reuses the learned child as a unit rather than flattening its primitive motor sequence back into host logic?

G5 tests hierarchy formation and reuse. It does not claim arbitrary recursion or universal program induction.

## Starting state

Both GENUINE and NO_HIERARCHY begin with the same two independently acquired child macros.

Each child:
- was itself absent before its own tuition;
- contains learned relational branch structure and opaque motors;
- transfers across unseen spatial bindings;
- is identified only by its EvoPhase carrier ID.

The evaluator never gives semantic skill names to cognition.

## Parent tuition

A parent episode contains:
- a raw relational outer cue;
- two child-macro invocation opportunities;
- factual Need only after the complete child sequence.

A fixed generic high-level curriculum tries all length-2 child sequences under each outer cue. The evaluator executes the actual learned child macros; it does not substitute their primitive action bodies.

Only successful factual parent episodes are eligible for consolidation.

GENUINE may consolidate:
```text
outer relational trace -> acquired child ID -> acquired child ID
```

NO_HIERARCHY receives the identical outer observations, child inventory, attempted child sequences and factual Need, but cannot promote a parent assembly that references child IDs.

## Ownership rule

A parent stores references to acquired EvoPhase macro IDs, not copied Rust action scripts.

Removing or revising a child must therefore affect the parent through the child reference rather than requiring the parent body to be rewritten by host code.

## Held-out evaluation

Held-out outer cues use absolute translations absent from parent tuition.

Learning is frozen.

GENUINE:
- parent readout ON;
- child macros available.

NO_PARENT_READOUT:
- same acquired parent and children;
- parent readout OFF.

NO_HIERARCHY:
- same child macros and parent tuition facts;
- no acquired parent.

The non-hierarchical controls use a generic enumeration of available child sequences with no evaluator hint.

## Cost metric

Primary mechanism metric:
- number of child-macro candidate sequence evaluations / attempted child invocations required before factual success.

Secondary:
- primitive physical motor actions executed by the actual child macros.

G5 is not accepted merely because a parent object exists; it must reduce one of these costs on held-out tasks.

## Acceptance

PASS requires all:

1. Both arms start with the same acquired child macro IDs.
2. Parent factual tuition ledgers are identical.
3. GENUINE acquires at least one parent macro referencing child IDs; NO_HIERARCHY acquires none.
4. The parent contains no copied primitive action sequence as its child representation.
5. At least one child ID used by the parent existed before parent tuition.
6. On fresh held-out outer bindings, parent readout invokes the correct acquired child sequence before final Need is known.
7. The invoked child macros execute their own learned bodies.
8. GENUINE has lower candidate-search or physical interaction cost than matched NO_HIERARCHY / NO_PARENT_READOUT.
9. Random opaque relabeling of child IDs does not change success when references are permuted consistently.
10. G0-G4 regressions remain PASS.
11. Rust tests and release build PASS.
12. Parent tuition and held-out costs are reported.

## Failure conditions

FAIL if:
- the parent stores a flattened primitive motor script instead of child references;
- evaluator tells cognition the correct child sequence;
- parent success disappears under consistent child-ID permutation;
- hierarchy provides no measurable cost advantage;
- a host-side planner chooses child order.

## Next gate after PASS

G6 — learned exploration strategy: the organism must acquire a reusable *way of choosing informative experience* that transfers across a new family of worlds, rather than using a fixed disagreement argmax supplied as substrate policy.
