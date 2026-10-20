# PR9 post-merge Codex review repair (2026-10-20)

Codex review email dated 2026-10-20, reviewed `829ea8b3`,
identified four reproducible robustness problems in PR #9 source, and one
potential one-use test rerun. This is a focused software safety/accounting
repair FROM the merged canonical `main` commit `3677cdc0`.

1. **Domain transition checkpoint**: entering a new construction domain
   creates a fresh Learner with history flags off. Any old TemporalOptionLane
   is now revoked/cleared before saving a checkpoint, preventing
   `from_constructing_checkpoint` rejecting a successfully entered domain.
2. **Untrusted width**: JSONL `cfg.validate()` now runs BEFORE creating
   its implicit `vec![0; cfg.width]`. Missing observation plus huge width
   fails with ordinary Capacity instead of risking out-of-memory.
3. **Behavioral-canary work ledger**: record an arm's actual accumulated
   computational work after EACH `step_constructing` return, even when
   the step returns an error/veto. The one-use attempt remains consumed.
4. **Multi-motor cost**: quote every learned autobiographical route
   action, including repeats. A route exceeding remaining external cost
   is not offered as a native U1 option; later offer changes still revoke.
5. **One-use rerun allegation**: original PR9 exact-head agency job
   `39393707630` had its `Evaluate new held-out worlds` step
   **SKIPPED**, so no repeat occurred. Historical scientific verdicts
   remain unchanged. A workflow SUCCESS after a SKIP is not a new exam.

Tests include real pending-option domain switch/restore, paired
cheap-vs-expensive repeated action quotation, genuine veto accounting,
invalid huge JSONL widths and source hash audit. Only a focused
`fix/pr9-codex-review-20261020` PR can merge these after exact-head
CI. This repair is NOT a new qualifying cognitive capability; it
changes neither original RSP2 nor HumanProtection motor authority.
