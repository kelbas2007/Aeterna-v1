# U2 — CARRIER-OWNED LIFETIME HYPOTHESIS ECOLOGY

Status: **PRE-REGISTERED BEFORE U2 IMPLEMENTATION**
Date: 2026-10-07
Prerequisite: U1 physical meta-control is present on current `main`. The clean-branch U1 PASS is supporting mechanism evidence only; current `main` must pass U1/U2 together with its current post-INTEL Repair-5–8 state before U2 can be qualified.

## Question

Can one persistent carrier maintain several competing hypotheses across a long lifetime so that:

- stale or irrelevant hypotheses lose action authority without being deleted;
- useful new hypotheses gain authority from factual outcomes;
- previously useful dormant hypotheses reactivate when supporting evidence returns;
- candidate selection is determined by physical learned utility rather than candidate age, insertion order, type or host priority?

U2 tests shared candidate lifecycle, not another domain-specific reasoning organ.

## Generic ecology representation

The phase-native meta-control state may own a bounded hypothesis ecology.

Each registered opaque candidate receives:

- one recruited physical candidate cell;
- one phase-sensitive utility synapse to a shared ecology/attention cell;
- factual observation count;
- immutable opaque candidate ID only for provenance lookup.

No hypothesis class/type enters the authority score.

Candidate authority is:

```text
applicability × physical conductance(candidate -> ecology)
```

where applicability is a bounded carrier-derived [0,1] input supplied by the hypothesis itself.

Dormancy is not a host deletion flag. A candidate is dormant when its physical authority falls below a frozen generic threshold.

## Utility learning

All candidate utility synapses start at zero.

A generic factual update accepts:

- opaque candidate ID;
- bounded factual usefulness in [0,1].

No world ID, task ID, candidate class, correct-candidate label or change flag is accepted.

Utility learning uses local prediction error on the candidate's physical synapse.

Positive factual usefulness raises authority.
Repeated zero usefulness lowers previously learned authority.
Provenance/candidate identity remains present.

## Lifetime witness

Register exactly four opaque candidates:

- one initially useful candidate;
- one candidate that becomes stale;
- one irrelevant candidate;
- one later-useful candidate.

The IDs are randomly permuted in the evaluator and have no score meaning.

### Regime A — initial experience
Factual outcomes make two candidates initially useful, with candidate A strongest.

Required:
- A chosen on 8/8 readouts;
- no deletion/retirement;
- all four provenance records remain.

### Regime B — changed evidence, no change signal
Without any explicit change/reset signal:
- old stale candidate receives repeated factual usefulness 0;
- irrelevant candidate remains 0;
- later-useful candidate receives usefulness 1;
- original useful candidate receives no new evidence and remains stored.

Required:
- later-useful candidate chosen 8/8;
- stale + irrelevant candidates dormant;
- original useful candidate remains stored, not overwritten/deleted;
- selection independent of candidate enumeration order.

### Regime C — evidence returns
Again without a world-ID/change flag:
- original candidate receives new usefulness 1 observations.

Required:
- original candidate reactivates and is chosen 8/8;
- candidate address/ID/provenance unchanged;
- no re-registration allowed.

## U1 coupling

Create two otherwise identical U1 cognitive proposals whose confidence/evidence field is populated only from the U2 physical hypothesis authority.

Required:
- higher-authority hypothesis proposal wins;
- after stale decay, winner switches;
- after reactivation, winner switches back;
- U1 meta weights are unchanged throughout U2 lifetime.

This tests that ecology affects common meta-control rather than creating another host priority list.

## Causal controls

On the currently winning candidate:

- weight lesion of its utility synapse -> winner lost in >=3/4 matched decisions;
- pi phase shift -> winner lost in >=3/4;
- exact restore -> original winner 4/4;
- unrelated candidate-synapse lesion -> winner preserved >=3/4.

## Persistence

Native checkpoint/restart must preserve:

- all candidate provenance records;
- learned utility weights;
- dormancy/reactivation status implied by weights;
- U1 meta weights.

No current REAL observation is restored.

## Source guard

Production ecology source must not contain:

- world_id / task_id;
- ContextualRefinement / PerceptualRefinement / CompositionalRefinement;
- RivalDiscrimination / GeneralEpistemic;
- stale_candidate / useful_candidate semantic names;
- correct_candidate;
- candidate-type weights;
- insertion-order priority.

## PASS

U2 PASS requires all lifetime, U1 coupling, causal and persistence criteria above plus:

- U1 regression PASS;
- G20-G23 PASS;
- Human Protection PASS;
- Release build PASS.

## Interpretation

U2 PASS would establish bounded carrier-owned lifecycle/attention utility for opaque hypotheses.

It would not yet prove that context/perceptual/rival structures automatically register themselves into the ecology. U3 must connect cross-mechanism residuals/proposals to the shared system and test evidence allocation across explanation classes.

No INTEL-2 before U1-U3 all pass.
