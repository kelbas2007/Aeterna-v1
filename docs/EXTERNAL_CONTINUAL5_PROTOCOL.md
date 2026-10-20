# EXTERNAL-CONTINUAL-5: protocol before implementation and scoring

This is a development and external validation study, not a qualification of
general intelligence. All controllers and encoders are authored algorithms.
Learning must use actual protected interactions, without supplied goal vectors,
answer labels in observations, game source, hidden maps, or solution scripts.

## Representation and control

One canonical Rust executable, its existing protected U1 actuator, and one
persisted learner are used. An optional bounded reward memory generalizes
between sensory observations and propagates rewards along real episodes.
Perception retains spatial components of categorical scenes and numerical
features of real images/signals. Raw observations and all action intentions,
costs, rewards, remaining deadlines, failures and checkpoints are recorded.
Neither model rollouts nor held-out answers become training facts.

## Development / scoring boundary

MiniGrid development uses Empty-5x5 and DoorKey-5x5 seeds 810000..810007.
The old public ARC ls20 game is repeatable development, not a fresh exam.
Only official training partitions of Fashion-MNIST and UCI HAR may be decoded
before the final source, binary, configuration and this protocol are frozen.
Public training-partition validation is allowed and must be labeled development.
Downloads of unopened test archives for integrity checking do not expose answers.

Before scoring, commit the implementation and save source / binary / data hashes
in a commitment. Score once; any repair after disclosure requires a new protocol
and fresh data. Repeated paired retention scoring of the same held examples is
explicitly a longitudinal measurement, not independent new evidence.

## External environments

Use minigrid 3.1.0 / gymnasium 1.2.0. For each of DoorKey-8x8,
MultiRoom-N2-S4 and Unlock, train from a cold learner for 32 episodes, seeds
830000..830031; freeze and restore before 16 episodes, seeds 820000..820015.
The interface supplies only public categorical egocentric images and all seven
motors. No mission, hidden pose/map, key/door semantics, scripted exploration or
training curriculum is supplied. Every episode has at most 256 actions.

Use official arc-agi 0.9.9 / arcengine 0.9.3, offline public games ft09 and vc33.
For each game, train for eight episodes, seeds 840100..840107; freeze/restore
before eight episodes, seeds 840000..840007, at most 256 actions each. Only raw
public frames, available actions, completion rewards and episode boundaries are
available. A finite click interface may expose up to eleven generic observed
component centroids, alongside the five ordinary actions. This restriction is
reported, and game source or a game-specific solution must not be inspected.

Compare full reward learning, the same reward learner with one-step horizon,
the previous causal learner without reward generalization, and uniform random,
with identical observations and maximum action budgets. Report actual actions,
successes / completed levels, distinct observations and per-family results.
MiniGrid target: at least 25% success overall and above both legacy and random.
ARC target: at least one held-out completed level and more than random. A failure
is published as a failure, not converted into success by counting exploration.

## Real recorded sensor streams

Fashion-MNIST: first 3000 official training images, then first 1000 official test
images. Labels are obtainable only by an executed answer action and its actual
binary correctness feedback. One paid image acquisition plus up to ten distinct
answer attempts in training; one acquisition and one answer in testing.

UCI HAR: first 3000 official training windows, then first 1000 official test
windows. Official train/test subjects are disjoint. Nine real 128-sample inertial
channels can be measured separately; six opaque answer motors report actual
correctness. At most nine acquisitions plus six distinct answers in training;
at most four acquisitions and one answer in testing. Report balanced accuracy,
macro F1, measured channels, action costs and errors, not only raw accuracy.

Run one continuous A (images) -> B (signals) learner, without replacing its
knowledge. Frozen restored copies measure A immediately after A training and
again after B training, and B after B training. No held-out feedback reaches the
continuing training learner. Namespace identifies the public sensor interface,
never a class or subject. Compare with matched 3-nearest-neighbor classification
on the same paid measurements and acquired training examples; random and old
causal controls are also reported. Any alternative sensor corruption is authored
and must be described as such, rather than as naturally observed device drift.

Targets: image accuracy >=65%; signal balanced accuracy >=65%; retained image
accuracy >=90% of its pre-B value. These targets do not assert superiority to
3NN. Show checkpoint/model sizes and distinguish bounded encoded knowledge
from process RSS. Persist and restore the complete learner, not an external
Python classifier as the native policy.

## Evidence and limitations

Publish commitment, summaries, complete compressed raw trajectories, commands,
dependency/data hashes and all negative outcomes. Never rerun an old exam and
label it fresh. MiniGrid and ARC are third-party virtual environments; images
and HAR are real recorded measurements, not a live robot or a physical world.
