# RELATIONAL-HYPOTHESES-2 — avoid discarding ambiguous real perceptions

Date 2026-10-10. Open preregistered development iteration;
NOT a frozen independent scientific authority.

Prior exact [RELATIONAL-WORKSPACE-1](RELATIONAL_WORKSPACE1_PROTOCOL.md)
run [38074563658](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38074563658):
native controlled relation+lesion tests 2/2 passed,
but real independent Farama 3.1.0 MemoryS7 paired
causal cue interventions failed. On 32 balanced
pairs (64 episodes) relation and source-identical
no-relation both got 32/64, zero changed exits,
zero 2-of-2 correctly solved pairs, with only 8/32
cue-exposed pairs. Critically the first relation
selector required exactly ONE rare candidate in
the first visual frame, and it had **0/32 initial
bindings** across actual real scenes. Thus it
discarded every ambiguous observation before
forming a causal memory.

## Fundamental representational difference

Instead of guessing one rare initial foreground or
refusing all object memory when multiple candidates
exist, retain a bounded SET (up to 12) of observed
rare visual hypotheses from the first actual partial
view. Each later factual view compares multiple
observed candidate appearances to EACH prior
hypothesis; a 96D distributed motor-learner input
contains only "same/different wrt earlier candidate",
relative visual location and uncertainty-slot identity,
without the absolute MiniGrid category code, semantic
word, target exit, motor ID or route. The generic
learned U1/HP policy remains the ONLY action source,
with exactly its existing causal learning rule.
This is an explicit programmer-defined operation for
partial belief/relation, NOT self-invented child logic
or RGB object recognition.

A hypothesis exists only if its corresponding
appearance was ACTUALLY visible. It is reset at the
start of each real episode and excluded from native
checkpoint and learned fingerprint; physical phase
synapse lesion kills the relational evidence.
The external harness never tells the organism which
of the candidate hypotheses is the original cue.
False foreground selection is possible; we must
not call generic hypothesis presence "cue understood".

## Grounded independent new outcomes

Use Farama \`MiniGrid-MemoryS7-v0\`, fixed type-coded
public partial observation, exact independent physics.
Train seeds 160000..160127 (128 actual episodes), then
native restart/frozen learned policy and 32 NEW matched
counterfactual layouts 161000..161031 (64 episodes).
Each map is tested with original starting cue and the
physically swapped key↔ball cue with corresponding
physical success/failure exits exchanged.
No hidden state, correct motor, correct branch or
cue label enters native process. Temporary evaluator
cue exchanges measure availability and are restored
before a real motor is requested.

Compare source-identical:
A. original native general policy + full actual
autobiographical episode replay, no new relation representation;
B. identical learner + ambiguous relational hypothesis context.
The balanced paired scorer must reject constant-side
strategies even if they obtain 32/64 game rewards.

Open cognitive PASS: >=33/64 true terminal rewards, >=4/32
pairs correctly solving BOTH cue variants with actually
exposed cue, >=4 paired factual RELATION readout changes,
and strictly more both-correct and exit-switch pairs
than no-relation control. Also 0 successful
both-variant pairs in which the relevant cue never
reached public observation (otherwise invalid/leak).

Native controls must show subject hypothesis retention
when first image has multiple rare appearances, then
eight occluded views, then identical present image with
different earlier rare appearances, producing different
relation features; initial cue content MUST be grounded
in actual prior perception. Physical phase-link lesion,
restore and checkpoint protections remain necessary.
No green CI substitutes for actual score.

Even if experiments PASS, general real-world transfer
to different visual encodings and proof of self-initiated
information search remain UNTESTED. The first serious
obstacle is still missing cue in many start locations,
which no memory mechanism can solve without exploring
for information.
