# LC1 V7 — bounded inputs and error-aware work accounting

Date: 2026-10-18. Integration candidate based on PR #8 commit `3140c183`.
This is a **software safety/compatibility repair**, not a new external
qualification, AGI claim or improvement in the number of acquired policies.

## Original reviewed findings

1. Deep sketches previously reached recursive `Expr::nodes()` before any
   depth-bounded validation. LC1 now uses a **nonrecursive**, bounded traversal
   over the entire sketch, rejecting excess depth (48), excessive nodes and
   large `Call` argument fan-out *before* recursive `ty`, `first_hole`, or
   `references_valid` are consulted. All subsequently used recursive helpers
   therefore receive a structurally bounded input.
2. `InductionConfig.max_nodes` previously applied `.max(3)` even when the
   final program had only 1–2 remaining nodes. It now passes the true residual
   node budget, which the existing induction engine accepts from 1 upward.
3. `scripts/check_project.py` used the hardcoded `<repo>/target` location for
   the joint LC1 demo despite `CARGO_TARGET_DIR`. It now reads the effective
   `target_directory` from `cargo metadata`. A regression invokes real Cargo
   metadata with a custom target directory before running the project suite.

## Corrective change beyond the original three findings

The earlier LC1 runtime had a systemic error-accounting problem:

    reserve all limits.work -> invoke LC1 -> refund only when return is Ok

A valid request could start execution, encounter arithmetic overflow and
return Err. The old runtime then kept the **entire reserved allowance** even
if only a few VM instructions had run. More preflight checks could never cover
all such dynamic errors.

LC1 now exposes an internal `(Result<CompletionReport>, metered_work)` contract
for both explicit completion and automatic discovery. The unified runtime
settles the same work reservation on **every** return path; a failed VM
operation is charged only for the instructions actually reached. Nested
discovery sums metered costs from each candidate even if it is rejected.
The common settlement helper is used by successful RSP2 solves too, without
modifying the RSP2 search engine or its admission gates. **RSP2 internal
errors remain conservatively charged at their reserved budget** until an
independently reviewed RSP2 error-metering API is implemented; this patch does
not pretend that all runtime mechanisms now expose exact failure costs.

The underlying `induction::infer` does not expose its measured work on `Err`.
When that subsolver returns an error, LC1 conservatively charges only the
*delegated remaining induction allowance* (which is bounded by the outer
reservation). This is explicitly not a CPU-time measurement or guaranteed
exact historical work accounting.

## Executed evidence

Local Rust 1.99.0, offline vendored dependencies:

- `cargo check --locked --offline --all-targets`: **PASS**.
- Six focused suites (LC1, RSP2 policy repair, RSP3 experience, human
  protection, unified runtime and the new adversarial contract): **68/68 PASS**.
- `scripts/test_check_project_target.py`: **PASS**, using a real overridden
  `CARGO_TARGET_DIR` and Cargo's metadata.
- `scripts/project_audit.py`: **PASS**, verifying original RSP2 source hash
  `aafe66f572983b9172e733a0abb7eccee2ba740af4eeec5ef36529782409bcb0`
  and the original 81-file source provenance.
- `scripts/check_docs.py`: **PASS**, 322 Markdown documents including this report.
- Adversarial test builds an 8,192-level expression: bounded preflight
  rejects it without a recursive walk; valid tiny sketches still work.
- The one-node fragment budget produces a bounded unsolved report instead of
  searching three-node fragments that cannot fit the final program.
- On an arithmetic-overflow task, the organism charged **7 metered VM
  instructions rather than the 2,000,000-unit reservation**, and could
  immediately solve another LC1 task in the same state.

The overall `scripts/check_project.py --offline` execution was attempted, but
its long persistent-session tests exceeded the local runner's timeout.
Consequently **a local full-suite PASS is not claimed**. Full Linux and
Windows GitHub Actions on the final exact PR code remain required before
merging to `main`.

No hidden probe answers, world identifiers, external action permissions, or
new cognitive authority were introduced. The study remains bounded numeric
computation construction, not human-level or sustained recursive improvement.
