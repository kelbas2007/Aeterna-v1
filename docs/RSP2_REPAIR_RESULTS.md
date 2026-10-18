# RSP2 local repair results — 2026-10-18

Scope: the three requested RSP1 repairs in the isolated local branch
`codex/rsp1-search-repair`, based on
`81adeb621bc511e6f65b65d628bc78ab4b6ad2a4`. No changes were made to the active
`aeterna_genome_run` checkout or its running experiments. The results below are
from local verification; GitHub Actions status must be checked separately.
Contracts and compatibility changes are described in
[RSP2_REPAIR.md](RSP2_REPAIR.md).

## Verification

Local Windows / Rust 1.99.0, locked dependencies, offline execution after setup:

- `cargo check --locked --offline --all-targets`: PASS, exit 0.
- 32 existing recursive-search tests and 21 repair tests: PASS.
- 187 other integration tests and 62 construction library tests: PASS.
- Total: 302 distinct passing tests in these selected suites; not a claim that
  every test in the repository was executed.
- `git diff --check`: PASS.

The selected integration suites cover directed construction, education,
quantity relations, word problems, semantic grounding, unified runtime, human
protection and U1 metacontrol. Nine session tests initially received access
denied from the sandbox's default temporary directory. All 11 tests in that
suite passed after TEMP/TMP was directed to this checkout's `.test-temp`;
production session code was not altered to bypass the failure.

The additional retention regression constructs an actually accepted policy
through the public runtime API, checkpoints and restores it exactly, verifies
all 10 retained cases, rejects a checkpoint with one retained case removed,
then executes a new admission that automatically evaluates those 10 cases.
Its accepted-case fixture comes from run-02; it is an engineering regression,
not an independent generalization trial.

## Final engineering experiment: run-02

The protocol was written before execution. Four lifetimes, seeds 201–204,
eight attempts each; all 32 attempts and all four exact runtime checkpoints are
retained under `rsp2-results/run-02`. Object search: 100,000 work units,
384 evaluations; meta search: 20,000,000 work units, 32 evaluations. Same budgets,
lessons, gates and seeds as run-01. Policies receive examples, not family IDs or
generating target programs.

One of 32 candidates was accepted: seed 203, attempt 7. Its acquired code is:

```text
priority = Input(0)
variation = Input(8)
location = Input(7)
```

At admission it preserved every incumbent verified solution and changed the
verified totals from 4 to 5 on eight fresh cases and from 0 to 5 on eight
transfer cases. Ten cases entered retention. Its producer comparisons gave
children with 1 -> 2 and 2 -> 2 verified solutions, respectively, out of eight
cases per comparison. The second comparison returned the same child more
cheaply; it is not evidence of a new child strategy. Combined creation plus
child cost score was 14,069,729 -> 9,656,684. This score includes failure
penalties; it is not CPU time or a 31% runtime speedup.

Seed 204, attempt 1, passed direct qualification but failed producer admission:
one child comparison fell from 3 to 1 verified solutions. The incumbent was
kept. This is an executed example of the third repair blocking a solver-only
improvement.

The accepted policy reads only the original features. Therefore this experiment
does **not** demonstrate autonomous acquisition of a residual-driven strategy.
The new residual-to-edit path is demonstrated by targeted engineering tests;
their deliberately authored policies must not be described as acquired ones.

## Separate post-training exam

Each row contains 128 tasks (32 per lifetime). A solution must fit the support
examples and predict all withheld probe examples correctly. All arms use the
same object-search budgets and seeds. The last four structures were not used in
the admission curriculum.

| Exam group | Acquired | Cold policy | Authored loss + size |
| --- | ---: | ---: | ---: |
| New tasks from lesson structures 0–3 | 61 | 59 | 71 |
| New tasks from admission-transfer structures 4–7 | 33 | 34 | 35 |
| Reserved structures 8–11 | 26 | 28 | 31 |
| Total / 384 | 120 | 121 | 137 |

There is no demonstrated overall advantage over either control. The accepted
policy gained two related solves but lost one transfer solve and two reserved-
structure solves relative to its matched cold policy. This remains visible;
the admission sample is not being relabelled as a general solution guarantee.
Four lifetimes and one adoption are also insufficient to establish a stable
effect across repeated self-modifications.

Total metered acquisition/admission work: 363,698,491 units, including failed
attempts, residual diagnostics, descendant construction and admission probes.
Post-training exam work is separate in the per-seed files. Whole run wall time
was 56.78 seconds on this local machine; no matched wall-clock speedup was
measured. Summary exam `work` fields are search work only; raw probe work is
also present in the per-seed reports and must be added for a total exam cost.

## Preserved earlier attempt

Run-01 had 0/32 accepted candidates. Its direct finalist was rejected for the
same 3 -> 1 child regression. Before run-02 a missing malformed-row guard was
added to residual observation, together with a charged row-shape scan and its
regression test. That accounting change affects bounded search trajectories.
The admission rules, seeds and resources were not weakened. Both runs remain;
run-02 is an engineering repeat after that correction, not an independent fresh
scientific qualification. The runs must not be pooled as 64 independent trials.

## Reproducibility references

The current search-policy source is byte-for-byte equal to the source embedded
in the run-02 protocol. SHA256:

```text
src/construction/search_policy.rs
aafe66f572983b9172e733a0abb7eccee2ba740af4eeec5ef36529782409bcb0
rsp2-results/run-02/summary.json
718d1c3d287aeef8989bbbbcee1fde97f218603bcbba10b2bb1bc64f40df9e9e
tests/recursive_search_repair.rs
709667d34af65b10693c832939c7f26b7f72e56b07063f80e9e53ef456ae6380
```

The two local result directories contain 20 files, 12,062,613 bytes total. They
are gitignored, not claimed to have been published. Historical RSP1 source and
results have not been rewritten. The changes repair admission and available
feedback; autonomous residual-driven learning, conditional specialization and
sustained producer/generalization gains remain separate, unachieved outcomes.
