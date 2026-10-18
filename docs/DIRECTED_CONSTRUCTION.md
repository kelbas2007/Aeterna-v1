# Directed construction (DC-0)

Status: opt-in bounded research implementation; CI results are recorded separately.
This does not replace the native EvoPhase runtime or qualify general intelligence.
Development branch: `research/beyond-intel4`. Do not alter frozen INTEL-4 evidence.

## Run

```bash
cargo test --locked --lib construction:: -- --nocapture
cargo run --locked --example directed_construction
# Optional checkpoint: refuses to overwrite an existing file.
cargo run --locked --example directed_construction -- fresh-construction-checkpoint.json
```

The example explores a hidden-state device through protected factual callbacks,
acquires a stateful program, validates it on subsequent observations, restores
cognition and attempts an observable goal. The hidden device variable is local
to the environment closure and is not supplied to the learner. This is an
AUTHORED repeatable development fixture, not a fresh independent qualification.

## Implementation contract

- `src/construction/vm.rs`: typed integer/Boolean expressions, checked arithmetic,
  conditionals, calls and fuel-bounded repetition. Simultaneous state updates.
  No filesystem, network, evaluator, task labels or authority instructions.
- `synthesis.rs`: discrepancy-derived jobs, provenance-linked witness pairs,
  raw channels not used by the parent, scoped deterministic alias constraints,
  typed-hole enumeration, exact Add/Sub/Xor inverse requirements, local output
  patches and acquired state updates. Queue/budget limits are explicit, not
  reported as proof that no program exists. Empirical coincidence never merges
  programs permanently.
- `constructors.rs`: binding-preserving structural generalization of successful
  edit traces into executable search schemas, guards synthesized in the same
  expression language, descendant trace provenance, pure callable library and
  no-library/same-library/constructor controls on single-use transfer batches.
- `runtime.rs`: private factual journal, forecasts committed before callback,
  discrete-Laplace probabilities, prospective frozen-model trials, per-record
  retention, version rollback without erasing experience, bounded checkpoints,
  disagreement/goal-directed finite lookahead and a protected external boundary.

## Connection to AETERNA

`ProtectedConstructionRuntime` owns an existing `EvoPhase` and consumes its
non-cloneable Human Protection permit before invoking the supplied actuator.
Cognition restoration cannot clear the live emergency latch. Legacy
`ScientificRuntime` and native selection are unchanged. This new mode's
synthesis, prediction and planning are SOFTWARE computations. Its capabilities
must NOT be attributed to physical SNN/EvoPhase synapses. No LLM is used in the
learning/execution loop. Human hazard evidence remains the host's responsibility.

The adapter supplies only raw integer channels, opaque action indices and
actual POSTs. It must not encode hidden state, goal solutions, task identifiers
or a desired program. `step_action` is an explicitly supervised intervention;
`step` selects its own action. Recorded resets are explicit host events, not
free assumed environment capabilities. Imagined POSTs are never learning data.

## Acceptance and costs

A search result is a proposal. It cannot promote itself using its training loss.
The runtime stages ONE candidate, commits both forecasts before each subsequent
external action, and compares frozen likelihoods after the configured number of
new observations. Training-prefix retention and description length are checked.
The consumed trial can subsequently become training data; another proposal
requires observations with newer IDs. A malformed POST or failed external
callback poisons the runtime rather than assuming the action did not happen.

The VM and search meter count explicit instruction/expansion/replay work, not
CPU cycles or a complete accounting of allocator/serialization overhead.
Compare wall time as well for performance claims. All library definitions are
counted. Constructor evaluation includes generation/evaluation work and its
recorded construction cost. No independent speedup claim is made by this code.
The separate broad search lane is offered one quarter of a hinted job's budget.

## Safety and storage

Journal limit defaults to 512 events (maximum 4096); capacity is checked before
actuation. No silent history eviction, automatic large artifact directory or
paid external service. Checkpoints are limited to 4 MiB; program types, references,
scale values and chronology are validated; dynamic state is recomputed from
history. FNV fingerprints detect accidental changes, NOT malicious forgery or
cryptographic authenticity. A checkpoint is trusted host input. Restoring a live
world requires the host to establish world continuity; equal observations alone
cannot establish equal hidden state. Action permissions are never checkpointed.

## Honest boundaries versus the full research specification

This first implementation covers integer/Boolean channels, finite typed search,
new recurrent cells, one-output local patches, executable patch generalization
and prospective controls. It does not yet provide byte/sequence perception,
arbitrary graph surgery, independently invented iteration algorithms, a native
SNN compiler, robust nonstationary causal identification, or unrestricted
recursive improvement. Repetition executes in the VM; its induction is not yet
part of the default grammar. Existing state transitions are retained while new
cells are added; arbitrary repair of an existing cell update is a further
search expansion. Structural growth is bounded. A constructor's presence is
NOT evidence of a new useful learning algorithm. `Registry::evaluate` explicitly
separates library reuse from extra search value and consumes its batch even if
an evaluation fails. Ordinary tests must preserve negative results.

## Specification provenance

Based on user-approved `AETERNA_directed_construction_v0_1.md` (16 October 2026),
SHA-256 `6b0ce3a83427e9be7bc791f514042931ddf8b4ff9bc8d4c5215951668052675c`.
The design's principal external antecedents are Push/Pushpop, Popper/Poppi,
Stitch/babble, active automata learning and failure-directed synthesis. This
implementation is newly written, not copied from those implementations; it does
not assert a novel scientific discovery. The original specification and its
18-source register remain the design provenance, not runtime test evidence.
