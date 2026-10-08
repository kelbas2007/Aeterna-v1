# TE5 — phase-native belief-conditioned action outcome ownership

Status: PREREGISTERED BEFORE COGNITIVE SOURCE CHANGE
Date: 2026-10-08
Branch: `research/beyond-intel4`

## Causal gap (documented before new implementation)

TE4 cold four-arm developmental replay at [37841337318](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37841337318) failed, despite earlier TE1, TE2 and TE3 physical mechanism qualification. A cold EvoPhase sometimes discovered and chose the sampling action, but even 124 additional held-out samples in arm 3 gave only 38/80 correct goal commitments; other arms failed to maintain consistent active sensing.

A separate *learned association from accumulated physical belief to factual goal action value* is missing. TE1's two physical cue→hub synapses encode uncertain evidence; TE2's physical motor→hub synapses encode repeatable sensing affordance; neither yet learns which terminal motor is useful given that evidence.

## Allowed generic TE5 mechanism

1. New `PhaseTemporalOutcomeLink` records physical addresses only: acquired cue-source physical cell, opaque external motor and its physical cue→motor value synapse, plus factual support count. No correct-action map or world/task ID.
2. The project observes only a bounded factual outcome `[0,1]`, a genuine POST representation, the executed opaque motor and the *pre-action* phase evidence for two cue sources.
3. Eligible terminal evidence is a factual POST outside both acquired cue identities; no special stop flag/hidden target class. Repeated sensing that produces either cue cannot create positive terminal-outcome links.
4. Factual credit is apportioned to the two acquired cue sources by normalized physical accumulated evidence; a single cue is weak but valid source evidence. Update physical value synapse using bounded locally weighted reward prediction error. Each motor/cue link is modifiable by further contradicting experience.
5. On an adequately decisive physical belief, propose the motor with strongest **phase-conducting** learned outcome value for the current cue winner. Unlearned, lesioned, π-shifted, tied and low-value outcomes abstain rather than invent an action. A tie/low-margin belief must seek information or abstain.
6. No pre-supplied latent class, label, observation reliability, sensor motor ID, target specific action mapping, manually selected goal route, assigned world phase or output action.
7. Checkpoint/restart must preserve physical policy links; lesion and phase shift of necessary cue→motor synapse must abolish the learned decision, exact restore recover it without learning. Continue to keep transient evidence reset separate from long-lived outcome value.
8. Integration into `ScientificRuntime::step_unified` is permissible only after causal mechanism tests: signal reward credit after real permitted execution and validated POST; learned decision goes through the same U1 comparison and Human Protection, with no host priority.

## Mechanism qualification, independent of TE4 burned generator

Six new arbitrary physical cue/motor assignments; each action gets balanced factual experience under two raw cue sources with opposite reward outcomes. The evaluator supplies factual success/failure, not the correct motor label. After training freeze value learning; physical evidence readout for each cue must select the factual useful motor **12/12**. Lesion/π/restore must cause/recover readout. Change the action/reward mapping and factual learning must revise existing values rather than preserving a stale answer.

Old TE1 physical memory, TE2 sensor affordance, TE3 U1 protected sensing, C, G20–G23, U1–U3 and software Human Protection must remain passing. This qualifies belief-conditioned outcome *mechanism* only. It does NOT qualify cold autonomous discovery or any stochastic end-to-end inference. No threshold or world adaptation after TE4 diagnostic failure should be called an independent frozen PASS.
