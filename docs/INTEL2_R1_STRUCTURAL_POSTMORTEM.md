# INTEL-2R1 STRUCTURAL POSTMORTEM

Date: 2026-10-08
Verdict run: `37728670318`
Diagnostic replay: `37729057004`
Frozen cognitive SHA: `07b44fb8e00837568bc9655e760eb031d1944dff`

## Status

INTEL-2R1 remains FAIL.

This postmortem replays the already burned pack only to diagnose architecture. It does not change the verdict or create new qualification evidence.

## C — history representation never bootstrapped

At the aliased junction there were 600 scored training visits, balanced across the two hidden predecessor histories:

- side 0 visits: 300;
- side 1 visits: 300.

Action histogram at the junction:

```text
side 0: [300,0,0,0,0,0]
side 1: [300,0,0,0,0,0]
```

The sealed useful history actions were motors 3 and 5.

The organism chose motor 0 on **600/600** junction visits and never sampled either useful discriminating action.

Result:
- context candidates: **none**;
- promoted context: false;
- held-out score: 0/32.

Therefore C failed *before* evidence-gated context learning. The failure is action/experiment allocation starvation, not a weak G21 evidence threshold.

## D — rejected first hypothesis blocked alternative structural search

Across 1200 raw-pattern trials the action histogram was:

```text
00: [720,0,0,0,0,0]
01: [120,0,0,0,0,0]
10: [120,0,0,0,0,0]
11: [240,0,0,0,0,0]
```

The organism again selected motor 0 on **1200/1200** trials.

One composition candidate was born:

`Atom(WeakAmplitudeBin(3))`

It received 128 future observations and was correctly rejected:

- log evidence: **-2.185721**;
- promoted: false;
- retired: true;
- atom effect ~0.1713.

However no replacement composition candidate was generated on that base state after retirement.

The XOR structure required by the sealed world therefore never entered competition.

This exposes a one-shot candidate-generation constraint: the first explanatory hypothesis can be rejected correctly yet still permanently block later alternative hypotheses for the same residual.

## U2 credit mismatch

At the end of D the U2 ecology contained two stored records.

One had:
- 128 utility observations;
- weight/authority ~**0.99999994**;
- active.

Yet the associated representation search on the current base contained only a retired, nonpredictive candidate.

The frozen unified runtime credits a selected persistent hypothesis whenever the *global* knowledge snapshot gains any circuit/revision/candidate/evidence observation.

Thus evidence accumulation — including evidence that ultimately rejects a hypothesis — can give positive U2 usefulness to the selected hypothesis simply because some global learned state changed.

This is a provenance/credit-assignment error.

## Meta-control self-lock

The frozen U1 common currency includes a positively weighted confidence field.

At a new state:
- an unknown action may have high epistemic novelty but zero support confidence;
- the first sampled action gains support/confidence;
- if that action also reaches further unknown frontier, it retains high epistemic value;
- additive confidence then makes it systematically dominate still-unknown sibling actions.

The C and D histograms are the empirical signature: one arbitrary action monopolized hundreds of opportunities while alternatives remained untested.

Confidence is therefore being treated partly as utility, although confidence should normally modulate reliability of a predicted benefit rather than itself constitute benefit.

## E — not usable as clean architecture evidence

E showed:
- shortcut successes 0;
- law change not triggered.

The action histogram already showed limited start-state coverage before the first goal path.

However the E loop inherits the same terminal-goal lifecycle pattern that invalidated original World C: after reaching goal, the helper can stop rather than begin another episode.

Because C and D independently establish genuine frozen-core failures, E is not needed for the FAIL verdict and should not be overinterpreted.

A future system test must preregister terminal episode handling for every repeated-episode world.

## Architecture diagnosis

U1-U3 solved **competition among existing proposals**.

INTEL-2R1 shows the deeper missing capability:

> the organism does not yet own a persistent developmental agenda for creating, testing, replacing and completing hypotheses over multiple actions.

Three coupled deficiencies dominate:

1. **myopic action arbitration / self-locking confidence**;
2. **global instead of hypothesis-local causal credit**;
3. **one-shot hypothesis birth with no replacement search after rejection**.

A fourth likely requirement is temporal experiment commitment: evidence-gated structures often need a sequence of deliberately balanced observations, not a new one-step competition from scratch each action.

## Next architecture hypothesis

Do not patch the sealed INTEL-2R1 worlds.

Create a new architecture line centered on **carrier-owned cognitive projects / developmental agenda**:

- persistent unresolved residual projects;
- project-local evidence obligations;
- expected information/goal gain over multiple actions;
- confidence as reliability, not additive reward;
- project-local causal credit only when that project's evidence/prediction improves;
- automatic alternative-hypothesis generation after rejection;
- dormancy/reactivation without deletion;
- model-validity maintenance for high-impact learned laws.

Only after architecture-level gates on these properties should another independent system verdict be attempted.
