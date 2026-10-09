# Multi-step physical evidence acquisition — design contract

Date: 2026-10-09. Branch: `research/beyond-intel4`.
Status: OPEN DEVELOPMENT, not a sealed scientific pass.

## Behavioral target

A single EvoPhase organism, after external factual observations, must autonomously discover a sequence of **unknown, opaque motor actions of variable length** (2, 3, 4, 5+) that reaches a genuinely informative raw cue. A readout must carry unfinished information seeking across intermediate physical states, resume after checkpoint/restart, and revise when an expected transition is contradicted.

This is *not* a fourth hard-coded action slot, a host-supplied plan, a fixed number of samples, or a second software owner of cognitive policy. The active memory and factual action-transition associations belong to the existing EvoPhase carrier. Non-rewarding intermediate actions gain value **only because a physically conducting causal route to an acquired cue exists**.

## Proposed opt-in integration

- Learn generic, bounded factual PRE-abstract-state/action/POST-abstract-state transitions in one persistent carrier (a source->motor and motor->destination pair of physical phase-sensitive synapses; no hidden or learned task labels).
- Store just transition addresses, support, per-context action coverage and episode-local physical state. No imported correct route. Discover new transitions through generic under-tested motor sampling when evidence remains insufficient; physical HumanProtection still arbitrates actuation.
- Infer a useful causal path by bounded carrier traversal to acquired raw-cue cells; select the next motor given *current external state*, never a pre-generated sequence.
- Require physically supported consecutive links before a motor can count a factual new cue. A random no-op, imagined fact, or terminal reward cannot masquerade as an independent sample.
- Bounded path length, no cycles, abort/revise on factual mismatch, no special arm/read roles or length-specific fields. Checkpoint contains learned physical links; transient step context is cleared at new episodes/restarts.
- Keep existing two-step TE path and one-step Fresh2 default unchanged unless this experimental opt-in is used. No modifications to `main` or `intel4-frozen-unified`.

## Development controls to create first

1. Different sequences of length 2, 3, 4 and 5 with shuffled opaque motor identities and markers; the very same implementation works at every length. No source-specific priority.
2. Both mandatory physical links for any intermediate action can be lesioned separately; path disappears or changes, and restoration returns it. Checkpoint preserves learned graph. A non-cue next state must not count as an additional cue.
3. A new, untaught sequence is acquired through `ScientificRuntime::step_unified` and real PRE/POST, not an evaluator motor schedule; measure found traces, correct final actions and timeouts.
4. Fixed-budget comparisons to no-sensing and single-cue controls. Set fresh structurally independent challenge criteria **before** a later frozen source/sealed first attempt; do not relabel the previous consumed STRUCTURE-1 FAIL.

## Current scientific baseline

First-attempt STRUCTURE-1 (two-action physical prerequisite) `37948186490`: FAIL 0/4. Later open repair `37951476712`: 2/4 on a different seed, learned two-action sensing 4/4. Existing Fresh2 independent within the direct-sensor task family: 4/4 PASS. None proves variable-depth multi-step transfer.
