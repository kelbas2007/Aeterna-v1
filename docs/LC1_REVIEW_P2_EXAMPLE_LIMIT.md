# LC1 PR #8: reject synthesis tasks beyond the fragment-induction limit

Review finding: https://github.com/kelbas2007/Aeterna-v1/pull/8#discussion_r4308498699

## Compatibility contract

The general `SynthesisTask::validate` accepts up to 256 support examples. The existing generic fragment learner `induction::infer` accepts **4 through 128** examples. LC1 delegates missing-right-expression induction to that unchanged backend. Therefore LC1 now checks `validate_completion_task` in both `RequirementMemory::complete/discover` and the two public `ScientificRuntime` methods **before reserving computation work**.

- 4–128 rows: permitted, subject to the existing task and work limits.
- 129–256 rows: `Error::Capacity`, without searching or charging the computation budget.
- Fewer than 4 rows: pre-existing `Error::InvalidData` contract is preserved.
- The general `SynthesisTask`, RSP2 search engine and producer/retention gates remain unchanged, so this LC1-specific restriction is not imposed on unrelated capabilities.

This is a conservative limit, **not a new ability to induce expressions from 256 rows**. Supporting a larger corpus in one LC1 call requires extending and independently qualifying the fragment-induction backend.

## Completed local checks (Rust 1.99, offline)

`lc1_induction_support_limit_rejects_before_work_reservation` checks the accepted 128-row boundary and both formerly accepted-but-unsupported values 129/256. It exercises both `RequirementMemory` APIs and both public runtime paths with a previously learned inverse rule. Refused calls leave the joint checkpoint and cumulative computation ledger byte-identical. A subsequent supported 22-example task still produces a VM-forward-verified expression.

Five related test binaries: 54 of 54 test cases passed (`learned_requirements` 11, `recursive_search_repair` 21, `experiential` 7, `unified_runtime` 10, `human_protection` 5). `cargo check --all-targets --locked --offline`: PASS. `scripts/project_audit.py`: PASS with original RSP2 policy hash unchanged. `scripts/check_docs.py`: 317 local Markdown documents checked, no missing local links.

These are engineering and regression checks, not proof of general intelligence. Final exact-revision Linux/Windows GitHub CI remains the merge requirement.
