# LC1 / RSP3 unified organism — review corrections for induction width and sketch preflight

**Date:** 2026-10-18. **Scope:** PR #8 review findings on `27eeec08`.

This is a correctness fix for bounded LC1 within the existing shared Learner and
ScientificRuntime. The RSP2 policy, producer admission, checkpoints, lessons,
and external permissions are unchanged. No new AGI result is asserted.

## P2: induction only supports six input columns

Generic `SynthesisTask::validate()` accepts 1–32 columns because other
mechanisms handle larger inputs. LC1's `induction::infer` currently accepts
1–6. The public `validate_completion_task` now checks
`MAX_LC1_INPUTS = 6`, in addition to `MAX_LC1_SUPPORT_ROWS = 128`, before
any LC1 runtime work reservation, for both explicit and automatic completion.

The regression validates that six-column inputs remain admissible, seven-
column and thirty-two-column generic tasks remain valid to other consumers,
but LC1 rejects them with `Error::Capacity`. Both `RequirementMemory` and
`ScientificRuntime` interfaces are checked; the exact joint checkpoint and
work ledger are identical before/after each rejection. A supported task still
completes with a forward-verified executable program afterward.

## P2: invalid explicit sketches must not reserve all work

The syntax/type checks formerly performed inside
`RequirementMemory::complete` are now also exposed through
`validate_completion_sketch`, called by the runtime **before**
`computation_work += limits.work`. Both direct and runtime calls use the same
validation. The test checks five malformed sketches, including a non-binary
expression, incorrect hole ID, multiple holes, Boolean operand, and a
non-integer operation. Each yields `InvalidProgram`; neither acquired state
nor metered total changes. The original correct sketch still solves afterward.

This preflight deals with malformed instructions before search; other errors
that genuinely arise during charged search continue to follow the existing
failure-accounting contract. It does not extend the induction grammar.

## Executed local verification (Rust 1.99.0, offline)

- `cargo check --locked --offline --all-targets`: PASS.
- Five touched test suites: **56/56** cases PASS (7 experiential, 5 human
  protection, 13 LC1, 21 RSP2 repair, 10 unified runtime).
- `python scripts/project_audit.py`: PASS, original cognitive source identity
  preserved; RSP2 search-policy SHA256 remains
  `aafe66f572983b9172e733a0abb7eccee2ba740af4eeec5ef36529782409bcb0`.
- `python scripts/check_docs.py`: 319 Markdown docs, no broken local links.

These are software quality checks, not independent evidence of recursive
self-improvement. Full Linux/Windows PR Actions must pass on this exact
revision before merging into `main`.
