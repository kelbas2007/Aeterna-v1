# AETERNA-v1 — ARCHITECTURE DIAGNOSIS AFTER INTEL-1

Date: 2026-10-07
Status: architecture review after final INTEL-1R4 FAIL

## Executive conclusion

The dominant failure is no longer lack of individual cognitive capability.

It is **meta-control fragmentation**.

AETERNA-v1 has acquired mechanisms for exploration, goal-conditioned planning, rival hypotheses, contextual refinement, perceptual refinement, compositional refinement, continual revision and protected execution.

But the decision about **which cognitive process gets authority now** is still largely an authored orchestration problem.

That is the central architectural mismatch.

## 1. The host still owns cognitive mode arbitration

`ScientificRuntime::propose()` is a fixed priority chain:

```text
compositional refinement
 -> perceptual refinement
 -> contextual refinement
 -> rival discrimination
 -> goal-directed active reasoning
 -> general epistemic exploration
```

The representations, predictions and many action values are carrier-owned.

But the priority relation between whole classes of thought is not learned or negotiated by the organism. It is authored in Rust.

INTEL failures repeatedly appeared at these boundaries:

- disconnected goal versus general exploration;
- exploration versus frozen exploitation;
- one enabled learner starving another of facts;
- stale contextual experiment versus rival probing;
- contextual probing versus predecessor-specific coverage;
- multiple promoted contexts versus action consensus.

This pattern is more important than any one failed rule.

## 2. Representation learners are separate hypothesis silos

Contextual, perceptual and compositional refinement each maintain their own:

- discovery records;
- candidate inventories;
- promotion/retirement state;
- applicability rules;
- action requests.

They share a carrier substrate but do not participate in one common competition over explanatory adequacy.

As a result, one subsystem can hold an old but technically applicable hypothesis and monopolize behavior even when another kind of explanation would better reduce the current residual.

This is why adding another local arbitration rule keeps moving the bottleneck rather than eliminating it.

## 3. The organism lacks one common currency for cognitive control

Current modules reason with different local signals:

- novelty/frontier gain;
- goal relevance;
- rival disagreement;
- evidence gates;
- contextual side balance;
- supported plan value.

There is no single carrier-owned quantity that asks:

> Which internal operation has the highest expected value of computation now?

Such a quantity must include at least:

- expected reduction in decision-relevant uncertainty;
- expected improvement in goal value;
- confidence/evidence quality;
- cost of the experiment/thought;
- structural novelty;
- interference/resource cost;
- reversibility/safety constraints.

Without a common currency, mode arbitration remains host policy.

## 4. Candidate lifecycle is not lifetime-aware enough

A long life accumulates:

- old unpromoted candidates;
- promoted candidates from different laws;
- provenance from prior worlds;
- rival transitions;
- context-specific coverage gaps.

Preserving provenance is correct. But preservation without learned utility/competition produces cognitive clutter.

The architecture needs explicit carrier-owned mechanisms for:

- hypothesis utility;
- dormancy;
- reactivation;
- consolidation;
- retirement;
- dependency-aware forgetting;
- resource budgets.

Not because memory is too small, but because **attention and structural authority are finite**.

## 5. "One substrate" is not yet "one cognition"

The project goal says the same EvoPhase network should support perception, prediction, hypotheses, experiment, imagination, action, consolidation and revision.

Mechanistically many of these now use the same cells/synapses.

But software-level ownership still distinguishes separate organs with separate APIs:

- `phase_native_context_action`;
- `phase_native_perceptual_action`;
- `phase_native_compositional_action`;
- `choose_phase_native_goal_rival_probe`;
- `choose_phase_native_goal_active_action`;
- `choose_phase_native_abstract_learned_drive_action`.

This means substrate unification has advanced further than **control unification**.

INTEL-1 exposes exactly that gap.

## 6. The next architecture should not add another organ

The next research architecture should replace the fixed mode stack with a **unified carrier-owned hypothesis/action workspace**.

A useful target abstraction is:

```text
Factual observation
  -> active explanatory structures
  -> each structure predicts:
       consequences
       applicability
       uncertainty
       goal relevance
       information value
       computation/action cost
  -> physical competition among structures/actions
  -> one selected internal/external operation
  -> factual or imagined consequence
  -> evidence/revision
```

Context, perceptual feature, composition, rival law and route should become different instances of one explanatory structure interface, not separate top-level organs competing through Rust priority order.

## 7. Proposed architectural invariant for the next line

The host may still own:

- raw I/O;
- persistence;
- serialization;
- Human Protection;
- hard resource ceilings;
- deterministic audit.

The host must NOT own:

- which reasoning mode runs next;
- which hypothesis class has priority;
- whether to explore versus exploit;
- which candidate deserves attention;
- which abstraction family should explain residual;
- which thought/experiment is worth its cost.

Those decisions must emerge from carrier state and learned evidence.

## 8. Concrete redesign target: Unified Cognitive Competition

Do not add G24.

Create a new architecture branch/version only if development resumes.

Core object:

`CognitiveProposal`

Every acquired structure may locally emit a proposal containing only carrier-derived quantities:

- physical source/reference;
- candidate operation/action;
- predicted goal value;
- epistemic value;
- confidence;
- contradiction pressure;
- cost/resource demand;
- safety class (never permission to bypass Human Protection).

A shared physical competition layer selects among proposals.

Important: this is NOT a host table that computes a weighted score over named modules. The proposal fields and competition weights must themselves be embodied/learned in the carrier, with lesion/transfer tests.

The winning operation may be:

- external motor;
- gather evidence;
- test rival;
- refine state;
- compose representation;
- simulate future;
- consolidate;
- retire/dormant a hypothesis.

Thus "what to think about next" becomes part of the learned organism.

## 9. Required evidence before claiming improvement over v1

The redesigned line should NOT replay G10-G23 one by one.

First prove only three architecture properties:

1. **Unified arbitration ownership**
   - lesion/phase intervention on the meta-control carrier changes which cognitive operation wins;
   - deleting host priority ordering does not destroy operation selection.

2. **Long-lifetime hypothesis ecology**
   - old candidates can remain stored without monopolizing behavior;
   - useful old candidates reactivate when evidence returns;
   - irrelevant candidates become dormant/retired from carrier-owned utility.

3. **Cross-mechanism factual competition**
   - when history, raw feature, rival law and ordinary transition are all plausible explanations of one residual, the organism allocates experiments among them based on evidence/goal value rather than API priority.

Only after those pass should a new **INTEL-2** use qualitatively new worlds.

## 10. What to preserve from AETERNA-v1

Do NOT throw away the qualified mechanisms.

Preserve as reusable substrate components:

- physical phase-native transition/value recurrence;
- persistent checkpointing;
- generic abstraction depth;
- learned exploration drive;
- goal relevance;
- rival transition provenance/revision;
- evidence gates;
- current-sensory descriptor extraction/composition;
- contextual history;
- Human Protection.

The likely error is not that these parts are fake.

The likely error is the way they are assembled into a controller.

## 11. Final scientific stance

AETERNA-v1 is a strong experimental cognitive architecture with multiple causally tested acquired mechanisms.

It is **not yet a demonstrated autonomous developing intelligence under INTEL-1**.

The most defensible next scientific hypothesis is:

> Intelligence failure is dominated by host-authored meta-control fragmentation, not by absence of another domain-specific cognitive primitive.

That hypothesis should be tested by architectural unification, not by Repair-9.
