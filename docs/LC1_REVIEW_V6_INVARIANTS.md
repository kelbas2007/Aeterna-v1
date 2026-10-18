# LC1 PR #8: preflight, acquired-work and freeze invariants (2026-10-18)

**Scope:** Follow-up to Codex review on commit `afb5e6c8`; this is a defect
repair, **not** a fresh intelligence result or a new independent test cohort.
The canonical RSP2 search-policy and RSP3 learner source are unchanged.

## Reviewed problems and corrections

1. **Generated impossible sketches:** LC1's automatic discovery generated
   five-node sketches even when `limits.nodes` was three or four. Those are
   filtered *before* calling `complete`, rather than turning a bounded search
   into an `InvalidProgram` failure. The solver returns a normal unsolved
   report accounting for completed, feasible attempts.
2. **Invalid VM references:** `known.ty` is not sufficient to validate
   `Input(99)`, `State(0)`, or `Arg(0)` in a stateless, no-argument completion
   context. The explicit-sketch preflight calls
   `known.references_valid(task_width, 0, 0, library)` *before* the shared
   runtime reserves work. Rejected requests leave checkpoints and work intact.
3. **Forged zero-cost learned work:** Restore-time
   `RequirementMemory::validate` already reconstructs the first canonical
   rule from its complete observation set. It now additionally requires the
   serialized LC1 `work` to be **at least** the work spent by that exact
   deterministic final replay. Equality is **not** required: legitimate
   training can include multiple earlier acquisition passes.
4. **Book/organism freeze mismatch:** Restored memory's `frozen` flag must
   equal whether its owning `Learner.frozen_at` exists. Both mismatched
   directions are rejected, rather than only checking that a frozen learner
   must contain a frozen book.
5. **Additional cross-ledger bound:** LC1-only recorded work cannot exceed
   the owning Learner's cumulative work. This is an added shared-ledger
   consistency check, not a claim of cryptographic proof of past operations.

## Executed evidence

All five new regression tests **failed on the original code** (each tested
before its fix):

- `lc1_discovery_skips_impossible_generated_skeletons_without_discarding_work_receipt`
- `lc1_known_references_are_checked_before_shared_budget_reservation`
- `lc1_restored_ledger_cannot_claim_free_acquisition`
- `lc1_frozen_book_must_match_learner_freeze_both_directions`
- `lc1_restored_book_work_cannot_exceed_parent_learner_ledger`

After the fixes, **65/65 distinct selected integration tests passed** under
Rust 1.99.0, offline. Distribution: learned_requirements **22**,
recursive_search_repair **21**, experiential **5**, unified_runtime **10**,
human_protection **7**. In addition, `cargo check --locked --offline
--all-targets` succeeded, the exact-source audit passed (81 original core
files, 77 unmodified), and the RSP2 search-policy source retained SHA256
`aafe66f572983b9172e733a0abb7eccee2ba740af4eeec5ef36529782409bcb0`.

An older engineering test that inserted a legitimate acquired book into an
otherwise fresh learner was updated to set the parent's actual LC1 work to
match the inserted book. The previous test fixture violated the new invariant
by forging a valid-looking book with zero parent work. Its original intended
undiverse-record attack remains independently checked.

## Boundaries

These checks prove consistency with the known deterministic local acquisition
algorithm and account, not that external observations are cryptographically
authenticated or that AETERNA can learn arbitrary mathematical operations.
The original LC1 development tasks remain exposed. None of these repairs
establish AGI, real-world grounding or sustained recursive self-improvement.
The patch must be verified by full GitHub Linux/Windows CI on the **exact
published PR head** before it can be merged to `main`.
