# PR #8 P2: disabled LC1 must not charge shared work

Codex finding: https://github.com/kelbas2007/Aeterna-v1/pull/8#discussion_r4308609187

**Root cause:** Both `ScientificRuntime::complete_missing_computation` and
`discover_missing_computation` reserved `limits.work` from the shared
computation ledger even if no LC1 `RequirementMemory` had been enabled.
The existing learner then returned `NoEvidence`, and the refund branch was
not executed for `Err`. The shared checkpoint was mutated by a request that
performed no computational search. At most 20,000,000 work units could be
erroneously reserved on a single valid-but-unenabled request.

**Repair:** After existing task/sketch/limit preflight and **before** work
reservation, both runtime entrypoints verify that the *same constructing
Learner* has an initialized requirements book. If it does not, return
`NoEvidence` without state mutation. No separate subsystem or privilege
is created. Existing LC1 training, RSP3 candidacy, RSP2 admission, producer
checks and other search budgets are untouched.

**Reproduced failure first:** The new test
`lc1_disabled_requests_cannot_consume_organism_budget` called both
entrypoints twice with valid tasks, a 20,000,000-unit budget, and no
requirements book. On the old code it failed because the joint checkpoint
changed despite `NoEvidence`. The identical test passed after the repair.
It also tests an unchanged restored checkpoint, subsequent legitimate LC1
opt-in and learning from 35 executed arithmetic observations, and eventual
successful forward-verified completion.

**Local Rust 1.99.0 evidence after repair (offline, same revision):**
- 57/57 test cases PASS across five selected integration suites:
  `learned_requirements` (14), `recursive_search_repair` (21),
  `experiential` (5), `unified_runtime` (10),
  `human_protection` (7).
- `cargo check --locked --offline --all-targets`: PASS.
- `python scripts/project_audit.py`: PASS, original 81-file
  cognitive source aggregate reconstructed, 77 untouched base files.
- `python scripts/check_docs.py`: PASS, 319 Markdown files.
- Historical RSP2 `src/construction/search_policy.rs` remains
  `aafe66f572983b9172e733a0abb7eccee2ba740af4eeec5ef36529782409bcb0`
  (SHA256). No new general-intelligence outcome is claimed.

The current PR must pass **new exact-head Linux and Windows GitHub Actions
checks before merging**. This is a bounded resource-accounting fix, not
evidence of durable recursive self-improvement.
