# RSP1 — recursive construction works; stable recursive improvement does not pass

Verified implementation: `7c3aaa0b5a910f03c168daba53b2274dcf214934`.
Branch: `research/beyond-intel4`; main and intel4-frozen-unified unchanged.
GitHub Actions: https://github.com/kelbas2007/Aeterna-v1/actions/runs/39106129069
Linux job 117612232442 and Windows job 117613416359 completed successfully.
The successful build is not a successful intelligence qualification.

## Implemented change

The existing pure VM now executes a SearchPolicy with three acquired expressions:
frontier priority, choice of a primitive syntax edit, and edit location. The SAME
search procedure is used to construct a task solution and a replacement policy.
The accepted policy is both the actual PRODUCER and starting TARGET of its next
candidate. Recorded edit chains reconstruct every candidate from its producer.
Changes can replace, grow, remove, or recombine subtrees; they are not limited to
filling the holes of a first-generation template.

The state lives in the existing Learner Registry and ScientificRuntime checkpoint.
It can influence ordinary construction through synthesis_lane. Prospective model
acceptance and external HumanProtection are not bypassed. This is optional;
legacy modes without recursive search preserve their behavior. It is a bounded
programmatic search policy, NOT evidence that SNN executes these expressions.

The interpreter, primitive edits, ten search-state features, resource limits,
exploration reserves, lessons and admission criterion remain authored. Training
was driven by an explicit experimental loop, not autonomous lifelong selection
of arbitrary external tasks. The old constructor generation counter is a separate
metric; the new policy lineage must not be presented as old constructor growth.

## Completed acquisition, all seeds retained

Eight separate lifetimes, 24 attempts each, initial empty function libraries.
Seeds 101..108 reached policy generations: **3, 9, 7, 7, 2, 3, 5, 9**.
192 executed attempts; **45 accepted policy changes**. There is no setter that
assigns an artificial generation number. Acceptance required changed search
behavior, no fewer fits on 24 fresh admission tasks, and at least 5% improvement
in the registered cost-sensitive objective. Each admission batch was single-use.
This finite rule is not a calibrated guarantee of future retention.

All eight checkpoints matched the SHA256 values committed before the reserved
examination. They were frozen before test outcomes. The shipped checkpoint is
seed 101, selected in advance, NOT the best seed. These are new research lives,
not an update of every previously acquired school fact or of the entire organism.

## Frozen solver test: no aggregate improvement

A solve below requires a fit on supplied examples AND correct predictions on
32 unseen numerical inputs. This is sample generalization, not formal universal
correctness. Each entry pools 64 tasks across eight lifetimes; the common family
generators mean these are not hundreds of independent research replications.

| Policy | Related families, 512 | New families, 512 |
|---|---:|---:|
| Learned self-constructed | 257 | 89 |
| Cold loss-first | 261 | 122 |
| Authored loss-plus-size | 269 | 148 |
| FIFO control | 161 | 31 |

Related-family false fits (learned/cold/strong): 11/10/10; new-family false fits:
15/27/27. They are not counted as generalizing solves. Learned search consumed
18,048,701 and 26,620,137 instrumented work units; cold consumed 15,770,274 and
21,557,815. The strong authored control consumed 13,886,309 and 18,297,150.
More accepted generations did not translate into stronger frozen search.

## Producer-swap test: the required ignition result is absent

Hold the starting TARGET policy, examples, library and resource limits fixed;
change only the PRODUCER. Test the resulting child on 32 further tasks not used
for creating it. Eight pairs per lifetime, **64 matched comparisons** in total.

| Producer | Child sample-generalizing solves / 2048 | Creation work |
|---|---:|---:|
| Learned | 1010 | 1,262,099,883 |
| Cold | 1022 | 1,606,807,998 |
| Authored loss-plus-size | 1026 | 1,605,281,346 |

Learned versus cold child solve counts: **20 wins, 13 ties, 31 losses**.
Creation used about 21.45% less instrumented work, but did not preserve aggregate
child quality. Acquisition itself cost **4,163,170,939** units over eight lives.
The measured creation savings did not repay it. Work is an explicit software
counter, not all CPU instructions or an established wall-clock speedup.

Consequently RSP1 demonstrates executable recursive construction, not stable
recursive enhancement. Calling its nine-generation lineage a breakthrough in
intelligence would contradict its own frozen controls.

## Additional real external tasks

Source: SyGuS-Org/benchmarks, comp/2017/CLIA_Track, tree
`b7d6bffdda8b0a00ea841a1c7d0171b4cfcbe465`. Selection was fixed by syntax:
one Int-returning function, one to three Int arguments. All 21 selected files
remain in the denominator; one is unsupported by the adapter.

The teacher derives 16 numeric input/output examples from public constraints
using an SMT engine. AETERNA receives only these examples, not the formal target,
reference solution, task identifier, checker state, or counterexamples. The SMT
engine is then used separately to verify the produced candidate against the
original constraints for every input in [-1,000,000, 1,000,000], including integer
overflow and division validity. This is a bounded PBE adaptation, NOT the official
SyGuS grammar score, an unbounded-integer theorem, or ARC-AGI-3.

**Learned policy: 2/21 verified. Cold policy: 3/21 verified.** Both fitted examples
on four tasks; fitting examples alone was not counted as success.

The learned policy constructed x+y for fg_mpg_plane1.sl and an expression equal
to 3*x+3*y+3 for fg_mpg_plane2.sl, correctly not using the third argument.
The cold policy solved those two and additionally fg_max2.sl. Thus there are
real externally checked programs, but no external advantage of self-improvement.
The previously reported ARC-AGI-3 result is unchanged; it was not rerun here.

## Verification and independent recount

Rust/Cargo 1.99.0: cargo check --locked --all-targets PASS on Linux and Windows.
Linux: **281/281** selected Rust tests. Windows: **149/149**, a repeated subset.
New RSP1 Rust tests: **32**. Independent SMT adapter tests: **12/12**.
Windows loaded the same checkpoint bit-exactly and reproduced the complete search
results of the 20 supported external cases. That is replay, not a second exam.

Post-run Python audit reexecuted **29,736 saved pure-program runs** comprising
561,792 scalar predictions, reconstructed all policy edit chains, recomputed
admissions and compared the raw probe/producer results with summaries. These
large replay counts include errors and repeated tests, not solved-task totals.
A deliberately corrupted probe record was rejected by this audit. All 15 frozen
source hashes, eight checkpoint hashes, artifact hashes and Windows package
file hashes were checked. The Python recount counts CI SMT verdicts; it does not
claim to rerun the SMT proofs. Its script and raw output accompany the evidence
package as audit/rsp1_recount.py and audit/recount.json.

First integration stopped at a whitespace-only git diff --check failure; frozen
source bytes were preserved. The successful rerun checked their original hashes.
Neither solver nor policy was tuned after these reserved outcomes. The publication
commit changes only result documentation and routine read-only CI, not the tested
algorithm or acquired states. No previous failed research result is overwritten.

## Evidence and use

Linux artifact 12128926416 (SHA256
1ba9531d7764c3ab7ff9cc28ace8a85610ae5b761825d93e37f00810861ecd36).
Windows evidence 12128836596 (SHA256
8029795df6e87b4c767d94a54d44fda47ff377c9d319b344ebdbf324910cdf5d).
Windows binary package 12128796633 (SHA256
78c9967434fbeb6d836e27b5768057ca1dd0553a9f0e4c4cd0b4dd2eda3043f1).
Pretest freeze artifact 12128413377 was preserved before external examination.

Raw files: lifetimes/development-*.json, frozen-*.json, producer-swap-*.json,
exam-summary.json, trained-*.json; external/cases.json, external/summary.json;
source.tar.gz, source-check.json and compilation/test logs.
Machine-readable consolidated results: docs/evidence/RECURSIVE_SEARCH_POLICY_RESULTS.json.

Windows package includes aeterna-unified.exe, recursive_search.exe and the
preselected trained-101.json. To reproduce the authored experiment without Rust,
Python, internet or LLM: `recursive_search.exe train new-session`, then
`recursive_search.exe examine new-session`. A new directory is mandatory.
The general binary is JSONL machine I/O, not a human-language assistant.
External formal checking separately needs Python/Z3.

Status: implementation and registered experiment completed. The requested
stable self-amplifying intelligence is NOT established. The observed bottleneck
is not a first-generation-only representation any more: the current finite
admission criterion selects locally favorable changes which do not survive
aggregate transfer and producer controls. Mechanism availability, local
acceptance and global improvement must remain three separate claims.
