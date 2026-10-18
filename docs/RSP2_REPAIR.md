# RSP2: admission, residual feedback and producer selection

Base: `research/beyond-intel4`, commit
`81adeb621bc511e6f65b65d628bc78ab4b6ad2a4`. This change is prepared in the
isolated local branch `codex/rsp1-search-repair`. Historical RSP1 results are
unchanged. This document describes implemented behavior; measured outcomes are
in [RSP2_REPAIR_RESULTS.md](RSP2_REPAIR_RESULTS.md).

## Global adoption

`improve_recursive_search` now takes an `AdmissionSuite`, not a flat task list:

- `fresh`: at least eight support/probe cases;
- `transfer`: at least eight cases from other structures in the curriculum;
- `producers`: two to four independent producer comparisons.

Search receives only support examples. Probe examples are withheld until a
program has been produced. A solve requires both a support fit and correct probe
predictions. A candidate cannot replace the current policy if it loses any
previously solved fresh/transfer case or any retained case. With this protection,
adoption requires more verified solves or at least a 5% cost-score improvement.

Every accepted fresh/transfer solution enters the retained case set with its
actual search seed. Later changes are evaluated on this set automatically; the
caller cannot omit it. Checkpoint validation reconstructs the retained set from
the admission records, including all additions. Capacity is 128 cases; exhaustion
refuses the change instead of discarding earlier obligations.

Task input sets consumed by fresh, transfer and producer examinations are
single-use and cannot later become proposal training data unchanged. Keys ignore
labels and row ordering; they are not a global ban on every individual input row
recurring in a different task. Support/probe rows within each case are disjoint.
The engine cannot prove semantic diversity from opaque numerical examples: the
included curriculum supplies different expression structures and keeps their
identities outside policy inputs. These sample checks do not guarantee behavior
on every possible future problem.

This patch protects global adoption; it does not add a portfolio of conditional
specialists. A candidate that fails global admission leaves the incumbent active.

## Observed residuals

The original ten search features remain in their original positions. Positions
10–19 expose observed sample count, exact-match fraction, signed residual
balance/mean, the input most associated with the residual, a signed association
score, a pair of inputs whose ordering separates residuals, its signed strength,
and an observation-kind marker. No hidden target program, family ID, reference
answer outside support examples, or examination outcome enters these features.

Diagnostics are requested only when policy code reads the corresponding features.
Predictions and residuals are computed together once per evaluated candidate and
stored on its search node. Input association and ordering calculations are lazy;
their operations are charged to the same work budget. Integer arithmetic and
bounded residuals handle extreme predictions without overflow.

Policies using feedback can also choose a concrete mutation operand and arithmetic
operation. A negative variation value encodes `-(1 + edit + 8 * detail)`, where
`detail = leaf_index + (input_width + 7) * arithmetic_op`. Edits 0, 1 and 3 use
this choice; other edits retain their existing semantics. Leaf indices enumerate
inputs followed by the existing seven literal choices. Arithmetic operations use
the existing Add/Sub/Mul/Xor/Rem ordering. Policy expressions must acquire these
choices through search. No authored strategy body is installed. The independent
random/FIFO exploration reserves remain active.

## Producer selection during learning

A candidate that passes direct, transfer and retention checks becomes a finalist.
Both incumbent and finalist then produce a child from the same cold target,
training data, seeds, library and budgets. Each child is tested on fresh
support/probe cases unavailable during its construction. Each comparison must
preserve child solve count, and the aggregate must improve solves or cost by the
registered rule. A solver improvement alone can no longer authorize adoption.

All production, child search and probe work is charged, including rejected
candidates. The cost score includes child construction cost; accumulated lifetime
work also includes acquisition and admission. This is an instrumented software
counter, not a claim about complete CPU instructions or wall-clock acceleration.
Expensive child-production checks run only for direct-test finalists.

## Runtime and compatibility

The existing Learner Registry and ScientificRuntime own the state. The JSONL
`improve_recursive_search` operation requires the new structured `gate` object.
No additional cognitive subsystem or external-action authority is introduced.

New states carry `admission_version = 2`. Legacy RSP1 states retain their old
serialization/validation and can still be loaded, inspected and used to solve.
Further training on a legacy admission state returns `Stale`; its historical
local admissions cannot be presented as having passed the stronger contract.
Existing checkpoints without recursive-search state remain unchanged.

## Verification

Focused tests:

```text
cargo test --locked --test recursive_search_policy --test recursive_search_repair
```

The repair tests exercise real searches with equal scalar loss but different
residual bindings; operand selection from those bindings; diagnostic charging;
hidden probe separation; global replacement rejected after a real old-task or
transfer regression; producer execution from an identical target; identical
producers rejected as an improvement; checkpoint/admission integrity; and an
actual accepted policy whose retained cases survive a runtime checkpoint and
are automatically examined at the next admission.

The integrated engineering check uses four predeclared lifetimes (201–204), eight
attempts each, fixed resources and cold/loss-plus-size controls. It records all
attempts, exact runtime checkpoints, retained cases, costs and held-out results.
Families 8–11 are reserved for that check's post-training new-structure exam.

```text
cargo run --locked --example recursive_search -- repair-check rsp2-results/run-01
```

The command refuses an existing result directory and writes the protocol and
actual search-policy source before executing any lifetime. Passing engineering
checks does not by itself establish a better learned search policy.
