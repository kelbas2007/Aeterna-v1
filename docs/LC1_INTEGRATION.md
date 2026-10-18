# LC1 is the central computation-acquisition loop (limited implemented stage)

LC1 is **not** an isolated AGI or a new controller. Acquired right-operand inverse
rules live in the **existing Learner** alongside its RSP2 search and the
ScientificRuntime's EvoPhase/U1 and School state. The persisted checkpoint
contains all available knowledge; old checkpoints without LC1 remain readable.

## Concrete executable loop

```
record executed operation (left, right, output)
  -> learn a backward requirement from many executions in the existing VM
  -> observe desired output on unfamiliar task supports
  -> hypothesize a missing right-hand computation (or accept a known sketch)
  -> infer a candidate expression for the needed intermediate values
  -> execute the complete candidate in the original forward VM
  -> return a bounded hypothesis or explicit refusal
```

All op-to-inverse maps are **learned from executions**, using a small, authored
pure-VM hypothesis grammar; none are prefilled at startup. A learned rule
predicts `right = f(left, desired_output)`; before proposing a completion the
program checks that the forward operator really reconstructs the desired
output, and after building a full expression checks **all** support rows.

When the learned rule is not identifiable, such as conflicting right operands
for the same `0 * right = 0`, LC1 must decline that rule. Incorrect or missing
inverses do not remove the existing RSP2/induction search paths.

## JSONL API (aeterna-v1 --jsonl)

```
{"op":"init"}
{"op":"enable_recursive_search","config":{"object":{"work":100000,"evaluations":30,"max_nodes":7,"frontier":64,"edits_per_expansion":8},"meta":{"work":100000,"evaluations":30,"max_nodes":7,"frontier":64,"edits_per_expansion":8},"max_lifetime_work":1000000}}
{"op":"enable_missing_computation_learning"}
{"op":"observe_executed_computations","observations":[{"op":"Sub","left":11,"right":4,"output":7}, ...]}
{"op":"complete_missing_computation","task":{"examples":[{"input":[1,2,3,4],"output":-9}, ...],"library":{"functions":[]}}}
{"op":"missing_computation_report"}
{"op":"checkpoint"}
```

`complete_missing_computation` optionally accepts a `sketch` Expr containing a
single typed right-side Hole(0); if absent, LC1 generates a **bounded** set of
simple possible contexts of that missing subcomputation. The completion
returns `used_acquired_inverse`, `forward_verified`, `program`, work and how
many hypotheses were tried. The hypothesis search is **not** an unbounded
self-directed discovery of arbitrary unknown abstractions.

The same explicit `freeze` that locks RSP2 and school also locks LC1 learning.
An emergency stop and external-action authorization are unchanged. None of
these APIs grant a candidate program any physical/online action right.

## Verified engineering scope and scientific limitations

- Rust 1.99.0 local: unit and cross-component tests check the actual VM,
  numerical holdouts, missing-rule refusal, malformed inputs, atomic intake,
  nonidentifiability, exact checkpoint restore and joint coexistence with RSP2.
- `scripts/lc1_joint_demo.py` exercises two independent application processes
  and restores the same LC1/U1/RSP2 checkpoint without injecting hidden probes.
- The design uses **recorded external/teacher executions** to learn. It is not
  proof that SNN invented arithmetic or autonomously collected its own teacher.
- The scripted four-input arithmetic family is an **authored engineering demo**,
  not a new external SyGuS or ARC-AGI score.
- No LC1 acceptance into RSP2 meta-producer policy is asserted. Gating remains
  separate and must continue to pass producer/fresh/transfer/retention tests.
- Conditional inverse rules, generic recursion over arbitrary holes, better
  autonomous problem selection and stable meta-improvement remain open.

See [LC1 original research protocol](LC1_PROTOCOL.md) and
[RSP3 experiential search research](RSP3_EXPERIENCE_RESULTS.md).
