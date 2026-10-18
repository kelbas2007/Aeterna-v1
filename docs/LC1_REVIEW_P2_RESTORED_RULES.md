# LC1 PR #8 — sketch limit and acquired-rule restoration (18 October 2026)

Codex review: [oversized skeleton](https://github.com/kelbas2007/Aeterna-v1/pull/8#discussion_r4308653537), [restored-rule acquisition contract](https://github.com/kelbas2007/Aeterna-v1/pull/8#discussion_r4308653540).

## Repair

- A skeleton with more nodes than the *final completed program* limit is impossible to shrink via replacing one Hole. The preflight now rejects it with InvalidProgram **before** reserving any work in the joint ScientificRuntime ledger.
- Rule restoration now uses the **same deterministic acquisition routine** as observed-operation learning: at least 12 records, at least 5 distinct values of each of left, right and output, one consistent right value for each (left, output) pair, and exact match to the **first** candidate selected from the frozen allowed grammar. Validation reconstructs the full ordered rule set from forward-validated records, rather than only checking that an arbitrary small expression fits them. Real previously acquired checkpoint records remain valid. RSP2 source and permission gates are unchanged.

## Reproducible failure and correction

On pre-fix source, both new tests failed:

- `lc1_oversized_but_well_formed_sketch_rejected_before_budget_charge`: accepted a 7-node skeleton under a 5-node output limit. After repair: InvalidProgram, byte-identical checkpoint and no work charge. A skeleton at exactly the limit remains eligible for further verification.
- `lc1_restored_rule_cannot_claim_acquisition_from_undiverse_records`: a modified checkpoint containing 35 copies of the same valid Sub execution, plus an inverse rule, passed the previous validation even though it could never meet the five-distinct-values learning gate. After repair: InvalidData for rule validation and Learner checkpoint restore, **even after recomputing the checkpoint envelope checksum**.
- `lc1_checkpoint_rejects_fitting_but_noncanonical_inverse`: an alternative fitting Add inverse, never selected by the canonical learner, is refused on restore. Original acquired rules still validate.

A recomputed checksum protects bytes but does not authenticate whether observations occurred in the outside world. This patch validates internal consistency with the documented learning algorithm, **not cryptographic provenance of real experiences**. Final Linux and Windows GitHub CI is required before merging PR #8. No broad external intelligence or recursive-improvement outcome is claimed.
