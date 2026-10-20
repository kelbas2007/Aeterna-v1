# PR12 original U1 compatibility repair (20 October 2026)

## STOP-MERGE finding at the exact Codex-reviewed head

The full project checks on `d6c207c8c434fd174485fe2fecad88b9ae005484`
[GitHub Actions 39422865687](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39422865687)
were **FAIL on Linux and Windows**, although the 13 specialist
workflows PASSED and Codex issued a no-major-findings review.

The unmodified existing
`tests/unified_self_directed_lifetime.rs::physical_roles_remain_opaque_under_joint_arbitration`
is the counterexample: 24 opaque motor permutations; native U1 with
unused unverified factual readout and native U1 control BOTH solved 23/24,
but the physical action totals **646 vs 671**. The contract requires
IDENTICAL choices because no qualifying factual motor hypothesis
was ever selected, and the old source has zero such choices.

Cause: late per-step budget introduced to protect the optional new
observable graph was also applied when the graph option was DISABLED.
The legacy plan helper used to have its own `cfg.planning_work`
allowance, and HDC recall traversed up to 128 rows; the PR12 helper
changed these into globally restricted remaining budgets, silently
perturbing the old policy.

## Fix architecture and frozen verification

- Make strict global helper budgeting conditional on
  `observable_graph_enabled` in the existing TemporalOptionLane.
- When no new graph opt-in exists, preserve the old native plan
  search allowance and original 128-record HDC recall; no new
  physical option authority or role labels.
- When the graph IS enabled, all helpers still share remaining
  metered `planning_work` and the pre-permit reject barrier.
- DO NOT weaken the old 24-role parity test, or rewrite historical
  numeric results to make this failure disappear.
- Add cross-platform targeted release-mode parity workflow, original
  graph scan-budget tests and full project audit.
- Re-run the exact graph source-disclosed 36-world cohort on the
  corrected code. The previous 457/864 result cannot automatically
  be claimed for a new source revision.
- No merge until BOTH Linux and Windows full project CI as well as
  targeted graph, parity and Codex review are PASS.

Scientific boundary: all previously consumed one-use scores and
negative results are unchanged. No new AGI claim.
