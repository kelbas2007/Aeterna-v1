# Internally inferred context and phase dynamics — frozen comparison

## Question and scope

The retained-knowledge experiment ended with mixed results: better return to
rule B, worse reacquisition of A, and evidence that incompatible old and new
outcomes can occupy one predictive context. This experiment tests two distinct
hypotheses, without changing either previous integral verdict:

1. Inferring a persistent latent context inside the existing causal-memory loop
   improves continued behavior relative to unchanged AETERNA.
2. This particular coupled phase implementation adds practical value relative
   to equally bounded ordinary probabilistic context inference.

Baseline AETERNA already contains phase-native planning and meta-control. The
comparison is specifically about an additional **context inference mechanism**,
not phase cognition versus a phase-free organism. The equations below are an
engineering hypothesis, not a biological model of DNA and not a universal test
of every possible oscillatory architecture.

All changes are confined to the isolated branch
`experiment/integral-phase-context-20261123`, based on
`b714f40c83bb6ba2a9c383d67f3687a67b748ab9`. Main stays at
`01a35d0c69c3dec0340c753456ada4a7b7f221da`; the separate Aeterna Codex project
is outside this work. No production merge is authorized or included.

## Arms

| Arm | Context inference |
|---|---|
| `baseline` | Existing runtime, optional experimental state absent |
| `ordinary` | Four ordinary Bayesian evidence filters |
| `phase` | Four coupled relative phase variables |
| `phase_scrambled` | Same phase dynamics with independent relative-phase perturbations |
| `phase_uncoupled` | Same phase dynamics, coupling coefficient zero |

The four contextual arms have the same two unlabeled banks, candidate
likelihoods, admission rule, total table capacity, history hypotheses, readout
hysteresis, learning rules, existing native controller and action authority.
Only their persistent inference dynamics differs. Every action, including
information gathering, is selected by the existing protected runtime loop.

## Integration and information boundary

Optional state lives in the existing `CausalMemory`, inside its actual
observe/predict/plan/learn path. There is no evaluator-side selector, instructed
research sequence, alternative actor, oracle rule label or new actuator permit.

The mechanism receives the actual PRE, chosen action, within-episode history
and actual POST. Candidate likelihoods are calculated from counts that existed
before that POST was learned. The previously recorded prospective forecast
cannot be changed retroactively. After factual observation, latent inference
may change and the actual transition may be assigned to its inferred bank.

The inferred bank is used consistently for beliefs, planning support and
factual learning. Ordinary episode resets clear within-episode history but
preserve the inferred cross-episode state. Hypothetical planning branches never
update it. Read-only diagnostic belief queries never update it.

No hidden bit, motor role, true rule, segment identifier, episode number,
switch flag, dwell time, world seed or future schedule is passed to the
inference mechanism. Evaluation metadata is recorded only outside the actor.

## Shared memory and initialization

All arms retain the existing limits: four equal unit-cost actions, at most
four actions per episode, history depth 3, 2,048 model entries, 2 MiB causal
model budget, 192 planning states and 120,000 planning work units. Journal
capacity is 4,096 and the lifetime external-cost ceiling is 100,000.

The banks share **one count map and one total capacity limit**. The contextual
controller reserves its fixed state allowance inside the same byte ceiling.
There is no complete extra table per bank. The baseline can use its original
entire table budget. Actual bytes, entries, evictions, action counts and work
are reported. Equal ceilings are not a claim of equal actual CPU time.

On enabling a contextual arm, every old sufficient statistic remains in bank
0 under its original key. Bank 1 is dormant. There is no duplication of old
facts, selective replay, reset of old counts, change to native learned weights
or loss of factual/learning counters. Enabling is allowed only at a quiescent,
unfrozen runtime boundary. The optional state is absent by default, preserving
the old checkpoint encoding and baseline path.

All contextual arms use the same two-supported-surprise admission rule. A
strong surprise requires active-bank predictive probability below .25 with
support at least 8; two such surprises admit bank 1. A supported probability
at least .5 clears the surprise streak; neutral observations leave it intact.
Birth initializes the inferred state toward the new bank identically across
routers and assigns the triggering POST there. It skips the normal dynamical
update and scrambling for that POST; subsequent POSTs follow the chosen
equations. It is triggered by actual outcomes, never an evaluator boundary.

Channel evidence is used when both banks have support at least 8, or when one
has support at least 8 and assigns the actual outcome probability below .25
while the alternative has no observations and uses unknown likelihood .5.
Other support combinations, including 1–7 observations in one bank, are
neutral for routing. This prevents known common sensor
transitions from automatically overpowering a newly admitted context merely
because the second bank has not yet observed them. Exact support, depth
fallback and pooling logic are fixed in the committed source.

## Inference dynamics

Each of the four existing history-depth hypotheses supplies a pre-learning
log-likelihood contrast between banks. Contrasts are clipped to [-4,4]. Both
routers use the same existing history-depth weights to pool their readout.
Switch hysteresis uses readout -0.2 / +0.2, with bank 0 favored by positive
values. Bank birth and admission are shared operations, not evidence for a
phase-specific advantage.

The ordinary comparator keeps four posterior probabilities. For each factual
POST, each channel applies a fixed symmetric transition prior h=.01 and a
Bayesian likelihood update. This is a persistent probabilistic comparator,
not a memoryless threshold. Its initial probability is matched to the phase
readout through q_i=(1+cos(theta_i))/2.

The phase mechanism holds four relative angles. With normalized contrast
u_i=clip(log(p0_i/p1_i),-4,4)/4, it integrates:

    d theta_i / dt = omega_i
                    + kappa/3 * sum_{j!=i} sin(theta_j-theta_i)
                    - g*u_i*sin(theta_i)

Fixed parameters: omega=(-.03,-.01,.01,.03), kappa=.35, g=2,
eight synchronous Euler substeps with dt=.125 per actual POST. Initial
angles are (.15,.25,.35,.45), with the shared sign change at bank birth.
The phase readout is the history-weighted mean of cos(theta_i).

The uncoupled arm sets kappa=0. The scrambled arm applies deterministic
outcome-independent perturbations to relative angles, preserving unit
magnitudes. Its seed and progression are internal and do not encode hidden
switch times. The perturbation rule and ordering are fixed in source.

Scrambling can harm any stateful representation. Its effect alone therefore
cannot establish phase superiority. The ordinary comparator and uncoupled
control are necessary to interpret a phase-specific claim. A common rotation
of all phases **including their reference** is checked as an implementation
invariant, not counted as behavioral evidence.

These four coupled variables are not simply a sine transform of one scalar
posterior. They can nevertheless synchronize or receive redundant evidence;
channel inputs and dynamical readout are retained to inspect that limitation.

## Two explicitly separated suites

### Primary: 16 fresh instances with shared prior experience

The new GitHub run ID supplies the frozen evaluator's seed derivation. Sixteen
fresh opaque recodings and independent schedules are generated after source
and protocol are committed. Each instance first runs 384 ordinary autonomous
episodes under rule 0 with the unchanged runtime. Its complete live checkpoint
is saved and restored identically into every arm before optional inference
is enabled. This is shared acquisition for a new comparison; it is not the
historical first organism and does not count as five independent acquisitions.

### Secondary: 8 actual historical retained states

Load the eight live **repeat** continuation checkpoints from the archived run
45235768424 directly. Archive SHA256:

`bf231f8834d9f4b954bf1bcb064931836763dac76d51bbc8d8c73de48cf3c571`.

Paths inside that archive are
`retained-evidence/repeat/checkpoints/world-N/continuation-unfrozen.checkpoint.json`.
Their world identities use original master seed 45231338992 and indices 0–7.
The common initial true rule is 1, where that previous life ended. All arms
start from the same exact bytes with all historical knowledge and counters.
The new hidden/noise/switch schedules are fresh, derived from the new run ID.

This is direct continued life, not another reconstruction. These eight known
instances are reported separately and never pooled into the 16-world primary
qualification.

## Physical environment and schedules

The task family remains one binary sensor/actuator world with opaque symbols:
one informative sensor, one irrelevant-noise sensor and two terminal actuators.
The correct actuator depends on the hidden bit and the current hidden rule.
Motor meanings, observation codes and changes are not told to the runtime.

For every suite/instance/arm:

* **Switching condition:** 2,048 learning episodes, eight segments and seven
  unannounced reversals, starting with the suite's initial rule. Every segment
  has at least 128 episodes. The remaining 1,024 episodes are distributed by
  evaluator-private positive random weights and a fixed integer-remainder rule.
  Total episodes and exogenous schedules are exactly paired across arms.
* **Stationary condition:** a separate clone of the same original common
  checkpoint gets 512 learning episodes with the initial rule unchanged. It
  receives neither switching-condition experience nor evaluator change hints.

The schedules govern the external world, not the system's research actions.
Neither actions nor useful measurement times are scripted. Episode start
observations, equal offers and the goal are the same across true regimes.
The irregular boundaries prevent a fixed oscillator period from predicting
the switch schedule. Stationary behavior measures unnecessary self-switching.

All primary episodes retain learning. No post-hoc change in freeze convention
may improve a verdict. Frozen inference and frozen-model invariants are checked
with focused integrity tests separately; they are not a new held-out ability
claim for this study.

## Metrics and fixed gates

The primary unit is a world instance, not an individual episode. The main score
is success rate across **all 2,048 switching episodes**, with a paired
per-world comparison. Report all worlds and failed prerequisites.

For a comparison X versus Y, the prespecified improvement gate requires:

1. Mean paired gain at least 5 percentage points over the 16 primary worlds.
2. A strictly higher score in at least 13 of the 16 paired worlds.

Separate conclusions:

* **Ordinary context benefit:** ordinary versus baseline passes that gate.
* **Phase benefit:** phase passes that gate versus baseline, ordinary and
  scrambled, has no stationary mean regression over 2 percentage points versus
  ordinary, and uses no more than 110% of ordinary's switching action count.
* **Coupling contribution:** phase versus uncoupled passes the improvement gate;
  this is an additional distinct result, not silently substituted for phase
  versus ordinary.

Every comparison is reported even if another gate fails. Exact paired sign-test
probabilities are descriptive; no episode-level independence, selective world
exclusion or newly chosen threshold is allowed. These gates are operational
criteria for this bounded study, not a broad cognitive qualification.

Secondary measures: first 32 episodes following each actual reversal, last 32
episodes of each segment, per-segment reuse, stationary success and context
changes, actions, useful/noise sensor choices, timeouts, prequential selected
action log loss, internal inference work, planning work, actual elapsed time,
model bytes/entries/evictions and complete final state persistence. The field
`extra_inference_work` charges a fixed conservative allowance of 8,192 units
per factual POST; it is an upper-bound accounting convention, not a measurement
of actual arithmetic operations or CPU time. Actual elapsed time is separate.

Neither original integral FAIL (4/8) nor retained-repeat FAIL (0/8 new complete
chains) is recomputed or overwritten. This study does not claim to have passed
the older exact-context causal-revision chain gate through better average
behavior.

## Integrity, freezing and evidence

Before any fresh evaluation, the disabled-mode implementation must run the
unchanged original evaluator for original world 0. Full JSONL must match the
pinned original archive byte-for-byte and its JSON summary must match. Failure
aborts the new study as INVALID. This known-world replay is an integrity check,
not independent ability evidence.

Additional gates cover legacy checkpoint loading, exact new checkpoint round
trips, persistence across episode reset, factual-only inference updates,
unchanged model fingerprints during frozen inference, read-only belief calls,
total capacity bounds, self-selected action/callback/journal identity and common
initial states. Actual forecasts are logged before actions and outcomes.

Freeze source, constants, protocol, schedule generator and scoring before
behavioral execution. Use Rust 1.99.0 and locked dependencies. One first-attempt
run only; do not tune on primary results or repeat for a preferred verdict.
Build and focused synthetic integrity checks may precede source freeze; no
fresh behavioral pilot or parameter sweep is included.

All arm/condition traces, common/final checkpoints, schedules, source/executable
hashes, process exits and negative results are preserved. A scorer can validate
and recompute results from copied evidence without executing an organism.
Malformed/incomplete execution gives INVALID; a valid missed capability gate
is a negative result. Both remain in the evidence.

The maximum model/action/planning limits are equal; inference arithmetic has
a separate fixed bounded update and conservative work charge. No claimed
speed or energy advantage follows
from running mathematical phase equations on an ordinary CPU.
