# Autonomous inquiry: integrated mechanism and prospective evaluation

## What this experiment can establish

This work implements and tests a bounded, persistent inquiry process inside
AETERNA's existing causal memory and protected action loop. It tests whether a
supported contradiction can open a persistent question, whether competing
learned context predictions can prospectively direct a real experiment, and
whether factual evidence can resolve the question and inform subsequent goal
actions. It does not equate these abilities with consciousness or general
intelligence, nor does it claim invention of unrestricted explanatory theories.

This protocol and executable sources must be committed before the qualifying
seed is chosen. Development runs are explicitly labeled and cannot become the
qualifying result. Every qualifying world is reported, including worlds with
poor acquisition or absent mechanism witnesses. Negative findings are retained.
The previous integral-causal, retained-knowledge, and phase-context verdicts and
thresholds are not changed by this experiment.

## Existing organism and single action authority

The implementation remains in `ScientificRuntime` and its existing
`CausalMemory`. Ordinary probabilistic applicability inference is the common
context mechanism. Inquiry has no motor-role input, environment seed, hidden
rule label, privileged correct-action callback, separate controller, or actuator
permit. It contributes a prospective causal proposal to the existing U1
competition. Human Protection continues to authorize each selected action.
The environment executes exactly the action chosen by that protected loop.

Question creation requires a factual mismatch with support at least 8, a
predicted modal outcome probability at least .75, and actual outcome
probability below .25. Unknown predictions and ordinary entropy do not suffice.
The persistent question retains its originating PRE, action, prediction,
actual POST, support, and event. Experiments compare factual distributions of
the two learned applicability banks. High observation entropy shared by both
banks is not counted as information about which bank applies.

The pre-actuator `inquiry_decision` witness records the question, event, action,
bank probabilities, prospective outcome distributions, immediate information,
and expected information within the bounded horizon. Its event is the factual
transition's zero-based ID; the returned step event is that ID plus one.
The factual resolution requires repeated likelihood evidence for the same
inferred bank. An expired question is reported as expired, never as resolved.
Hypothetical planning and diagnostic reads do not update factual evidence.

## Opaque environment with two independent hidden factors

Each of 16 instances independently permutes five motor meanings and nine
observation symbols. All five actions are always offered at identical unit
cost. Each episode permits at most five actions. The actor receives a two-value
observation and one fixed goal symbol, without semantic labels.

| Evaluator-only role | Physical consequence |
|---|---|
| Cue sensor | Reports the episode's independent hidden bit |
| Diagnostic sensor | Reports the current persistent rule in opaque symbols |
| Noise sensor | Reports an independent, irrelevant random bit |
| Terminal actuator 0 | Succeeds if hidden bit XOR rule is 0 |
| Terminal actuator 1 | Succeeds if hidden bit XOR rule is 1 |

The diagnostic sensor can distinguish applicability contexts, whereas the cue
sensor is required for the episode-specific motor decision. The noise sensor
is a control for confusing unpredictability with reducible uncertainty.
A diagnostic reading is an ordinary physical observation available through a
chosen action, not a hidden rule label supplied to cognition. The evaluator
keeps the semantic interpretation private. The distinction is important:
this is a small designed environment with a useful available measurement, not
a claim that the organism invents new instruments or understands natural
language explanations.

On restart, only within-episode history resets. Learned models, native
controller state, applicability inference, inquiry state, evidence counters,
and cross-episode questions continue. All action selection is autonomous.
No acquisition or test action is prescribed by the evaluator.

## Common lifetime and paired continuations

Each world first acquires 768 ordinary-context episodes across eight
irregular segments, alternating between two rules. Segments have at least
64 episodes; the remaining 256 are divided by seeded positive random weights.
This provides an opportunity for both applicability banks and diagnostic laws
to be learned through actual, independently selected actions. It does not
guarantee either acquisition or diagnostic use.

The resulting complete live checkpoint is saved and restored byte-for-byte
into three arms:

| Arm | Persistent inquiry | Inquiry can contribute action proposal |
|---|---|---|
| `ordinary` | Absent | No |
| `active` | Enabled | Yes, through the existing U1 |
| `observe_only` | Enabled and factually updated | No |

Enabling preserves every learned sufficient statistic, native learned weight,
factual counter, journal, and ordinary context state. It only adds bounded
inquiry state and its reserved accounting. `observe_only` is the direct
intervention on directed experiment selection. Comparing `active` with
`observe_only` is the primary causal contrast; ordinary versus observe-only is
also reported to expose unintended behavioral changes.

Each arm's starting checkpoint independently seeds two continuations:

* **Switching:** 1,024 learning episodes across 32 irregular segments,
  alternating rules and beginning with the last acquisition rule. Each segment
  has at least 16 episodes; the remaining 512 are allocated by fresh private
  weights. Frequent irregular changes make the cost of diagnosis measurable.
* **Stationary:** 256 learning episodes with the last acquisition rule retained.
  It receives none of the switching continuation's experience. This measures
  unnecessary inquiry and disruption when applicability is stable.

The hidden episode bits and step-indexed noise are pre-generated and exactly
paired across arms. No boundary, episode number, future schedule, semantic
role, or scoring annotation enters the actor. Hidden annotations are written
to the evidence trace only after the corresponding action executes.

At the midpoint of every continuation, the complete live checkpoint is saved,
validated by exact roundtrip, and actually restored before the next episode.
Final live and separately frozen-clone checkpoints are saved. Freezing the
clone never changes the continuing organism. Behavioral episodes remain
learning-enabled in every arm; no post-hoc freeze convention changes a score.

## Resource contract

Every arm has the same external and model ceilings: five available unit-cost
actions, five actions per episode, history depth 3, 2,048 shared model entries,
2 MiB causal memory, 192 planning states, planning depth 5, 120,000 planning
work units, a 4,096-record journal, and a 100,000-unit lifetime external-cost
ceiling. Inquiry consumes its reserved state bytes within that same memory
limit. Actual action counts, useful/noise measurements, model entries, bytes,
computation work, and wall time are retained. Equal limits do not imply equal
actual CPU cost. Read-only evaluator belief queries are recorded separately
and cannot influence inference or action state.

## Independent integrity audit

The Python scorer reads the emitted traces and checkpoints independently. It
checks all physical outcomes against private schedules and opaque mappings,
step and event order, episode limits, exactly one journaled unit-cost factual
transition per selected action, action-source consistency, state capacity,
prospective forecast probability and logarithmic loss, report totals, and
checkpoint file fingerprints. It checks the identical common checkpoint and
normalizes only added inquiry state and its reserved byte accounting when
comparing arm starts. Every arm, world, timeout, and process error is retained.

A malformed run is `INVALID`. A valid run whose scientific gates fail is
`VALID_NEGATIVE`, not an implementation crash. Development outputs are
`DEVELOPMENT` and make no qualifying claim regardless of their score.

## Fixed primary gates

The primary unit is an independently recoded world, not a pooled episode.
The qualified comparison contains all 16 worlds.

**Directed inquiry benefit:** active versus observe-only must achieve a mean
paired switching success gain of at least **2 percentage points**, with a
strictly positive gain in at least **12 of 16 worlds**. Its stationary mean
regression may not exceed **2 percentage points**, and its total switching
action count may not exceed **115%** of observe-only's. These conjunctive gates
are fixed before the qualifying seed and cannot be loosened after results.

**Integrated factual chain:** at least **12 of 16 worlds** must contain a
verified chain: a supported factual contradiction opens a question; while
that same question is pending, a prospective inquiry witness with positive
expected information and positive immediate reducible disagreement is selected by U1 for the actual diagnostic action;
the resulting factual evidence precedes a resolved (not expired) question;
and a subsequent successful goal action occurs within eight episodes. The
question, diagnostic, resolution, and successful goal must all occupy the same
external schedule segment: a switch away and back never qualifies as unchanged
applicability. The diagnostic's semantic role is used only
for scoring after action. Terminal experiments and useful cue measurements
are reported separately and are not silently relabeled as diagnostic proofs.

These are separate conclusions. The overall behavioral-mechanistic claim
requires both. A mechanism can execute correctly without measurable benefit;
a behavioral gain without the specified chain does not demonstrate the claimed
reason for that gain.

## Evidence, interpretation, and limits

Evidence includes source and binary SHA256 provenance, private schedules,
all prospective/action/POST traces, all world summaries, independently scored
verdicts, and complete checkpoints. The protocol reports active, ordinary, and
observe-only success/action totals; paired differences; chain counts; question
open/resolved/expired counts; experiments selected versus actually executed;
and noise/diagnostic measurements. No result from this designed binary-rule
family establishes open-domain reasoning, new instrument design, arbitrary
compositional generalization, unlimited self-improvement, or autonomous goal
creation. Those remain separate empirical questions.
