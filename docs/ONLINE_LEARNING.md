# Online acquisition and persistent knowledge

This development extension starts from `research/beyond-intel4` at
`6714987`. It adds an opt-in mode for deterministic, fully observed tasks.
It does not change the recorded INTEL-4, FRONTIER-1, TE1 or TE4 verdicts.

An additional, explicitly selected [learned-rule mode](LEARNED_RULES.md) now
supports generalizing circular channel transformations and choosing experiments.
The P1/P3 receptor-based behavior described below remains available independently.
The current checkpoint writer uses v6; it also reads v1–v5 snapshots.
The optional [partial-observation extension](PARTIAL_OBSERVATION.md) adds
explicit missing channels and informative actions over acquired circular rules.
The [expanded vocabulary](EXPANDED_RULES.md) supports constants, integer gains,
signed sums and differences, and explicit expansion of an existing rule model.

## Behavior

Previously `ScientificRuntime` required every observation and goal to resolve
through a pretrained abstract representation. The online mode acquires bounded
raw sensor receptors from actual initial observations and actual action results.
A goal may describe an observation the agent has never encountered. Setting that
goal does not create knowledge, an observation, a route, or a successful outcome.

Action selection and transition learning remain inside EvoPhase. P3 explores
unmodelled actions and propagates frontier value through acquired synapses; P1
propagates a requested goal through the acquired transition circuits. No U1
teacher weights, supplied transition curriculum, hidden state IDs, motor roles,
or external graph solver are given to the organism. The external adapter still
supplies sensors, goal observations, action consequences and hazard evidence.

This mode is mutually exclusive at initialization with an already acquired
abstract vocabulary. Its single P1/P3 proposal does not use pretrained U1/U2
competition. It is a cold acquisition path, not a claim that U1–U3 or autonomous
composition have been learned from scratch. The old qualified paths remain
available without enabling online mode.

## Run the demonstration

Use the repository's Rust toolchain and fetch the locked dependencies:

```bash
cargo fetch --locked
cargo run --locked --release --example online_learning -- learn /tmp/aeterna-online.json
cargo run --locked --release --example online_learning -- run /tmp/aeterna-online.json
```

`learn` creates a new checkpoint file and refuses to overwrite an existing one.
`run` starts a separate process, restores the learned model, freezes learning,
receives a fresh observation, and reaches the same goal. It verifies that learned
knowledge did not change during frozen evaluation. The environment implementation
is confined to the example; its transition mapping is never passed to cognition.

In the cloud environment, first run
`source /workspace/.aeterna-env/activate.sh` if Cargo is not on PATH.

## Library entry points

1. Construct `EvoPhase` with the required sensory/motor dimensions and relay capacity.
2. Enable `PhaseNativeConfig`, then `PhaseOnlineConfig` using
   `enable_phase_native_online_learning` before observing a target.
3. Wrap it in `ScientificRuntime`, call `observe_external` and `set_goal`.
4. Execute `step_unified` (or `step`) with external assessment and execution callbacks.
5. Use `set_model_learning_enabled(false)` for frozen evaluation.

The example uses a 32-step planning horizon, 1.0 weight/phase learning rates and
one-observation support. These are inherited substrate settings, not acquired
meta-learning. Recognition uses maximum per-channel distance under the native
match threshold. No translation invariance, noise robustness, or learned feature
metric is claimed by this raw encoding. All-zero observations are valid.

Only factual observation boundaries may allocate receptors. Malformed sensor
values, blocked actions, requested goals and imagined proposals cannot teach
transitions. Unknown observations in frozen mode are reported as
`UnknownRepresentation`; exhausted capacity in learning mode is reported as
`RepresentationCapacity`. Physical resource bounds are explicit.

## Persistent state

`EvoPhase::online_checkpoint_bytes` produces a versioned JSON snapshot;
`EvoPhase::from_online_checkpoint` validates and reconstructs it. Serde is used for
data encoding only. `Cargo.lock` pins dependencies and `rust-toolchain.toml` pins
the tested toolchain.

Snapshots contain online sensor receptors, cells, synapses, physical circuit
metadata and their configuration. They exclude current REAL observations, goals,
runtime audit state and actuator authority. Mixed modes that the format cannot
represent are rejected instead of silently losing state. The decoder bounds input
size at 16 MiB and validates dimensions, numeric ranges, allocation budgets and
every circuit address before constructing the carrier. This is structural data
validation, not authentication of the file's author.

`ScientificRuntime::restore_online_checkpoint` replaces cognition atomically,
preserves the live protection latch and current learning/freeze setting, and
requires fresh external sensing. A process-level deployment must preserve its
external safety authority separately; cognitive files cannot authorize actuation.

## Validation and limits

`tests/online_learning.rs` exercises 64 deterministic directed graphs with
5–16 nodes, independently shuffled raw sensor patterns, node-dependent opaque
motor permutations, cycles, goal changes and frozen reuse after cognitive restart.
Every target begins without acquired receptors, transitions or transferred meta
weights. Additional tests cover changed-transition repair, a physical pi-shift
and exact restore, no-growth/capacity controls, frozen learning, blocked callbacks,
invalid observations, persistence and invalid checkpoint rejection.

These are deterministic development regressions authored with the implementation,
not an independent authority pack. Their 64/64 acquisition and restart results
do not establish transfer outside this finite-state class, general intelligence,
learned exploration principles, arbitrary program invention, or an advantage over
tabular model-based methods. Planning horizon, state capacity and sensor tolerance
remain explicit configuration choices.

Before this change, the existing TE4 cold stochastic diagnostic was executed
unchanged: held-out results were **40/80, 55/80, 40/80 and 38/80**, with repeated
sensing in **0, 0, 0 and 28** episodes, respectively. Its verdict was
`DEVELOPMENT_FAIL`. The online mode does not solve temporal uncertainty or claim
to repair that diagnostic. Its generator, budgets and assertions are unchanged.

## Ordinary continuous integration

`bash scripts/check.sh` runs online tests, library tests, P1/P2/P3/P5 controls,
the qualified-main integration selections, TE1–TE3 mechanism checks, Human
Protection, the statistical cross-check and release binaries/examples. The
development workflow runs on pull requests and source pushes, and exercises the
example across two processes.

The selected tests intentionally exclude one-use authority packs and known
negative research diagnostics; their existing dedicated workflows remain intact.
Ignored and filtered counts are reported by Cargo. A raw `cargo test --all-targets`
still includes those experiments and is not the ordinary regression command.
The INTEL-2 source guard now builds its forbidden strings at runtime, fixing its
previous match against its own assertion without weakening what it checks.

Local verification of this extension completed with **82 passed, 0 failed,
8 ignored and 79 filtered test invocations across 24 test binaries**. Imported
fixtures can execute in multiple binaries; these counts are not independent
world samples. All compilation targets and Clippy correctness checks completed;
existing unused-code/style warnings remain. The new GitHub workflow has not yet
been executed remotely.

The example was executed as two separate OS processes with the same disk file:

```text
mode=learn goal_reached=true actions=18 learned_states=11 circuits=18 frozen_unchanged=false
mode=run goal_reached=true actions=4 learned_states=11 circuits=18 frozen_unchanged=true
```

The first process acquires knowledge; the second reuses it without factual model
updates. This small demonstration is a persistence smoke test, not a performance
benchmark or scientific qualification.
