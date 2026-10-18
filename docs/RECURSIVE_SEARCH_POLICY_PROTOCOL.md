# RSP1 — executable search policies, protocol before reserved evaluation

Base branch: research/beyond-intel4 at 2b1f521c2b08d72eff9b16d167020fd48c68276f.
Development pilots, including failed transfer, have been inspected. They are not
qualification results. The following reserved evaluations have not been run.

## Implemented object and boundaries

SearchPolicy contains three executable expressions in the existing pure VM:
frontier priority, choice of primitive syntax edit, and edit location. The same
search implementation executes these expressions when searching task programs
and when searching replacement SearchPolicy code. Adopted policy P_t is the
actual producer and initial target for P_(t+1); changes are not confined to holes.
Every proposal records the producer fingerprint and the replayable edit chain.

Syntax operators, interpreter, ten numerical search-state features, validation,
1/8 FIFO and 1/8 random-variation reserves, and admission remain authored.
The expressions do not possess filesystem, network, external-action or judge
capabilities. This is bounded metacircular search, not a claim that all possible
learning algorithms are representable or autonomously invented.

The state is held in the existing Learner Registry, accessed by ScientificRuntime,
serialized in its existing checkpoint, and available to ordinary construction
without bypassing prospective model acceptance or HumanProtection. Prior modes
without the optional policy retain their existing behavior. No claim is made
that the existing SNN performs VM expression evaluation.

## Development and single-use admission

Eight independent complete lifetimes, seeds 101..108; 24 attempts each. Do not
select the most successful seed to represent the result. The shipped example
state is seed 101 regardless of its performance. Each starts from the same
cold policy and empty acquired function library. Original prior checkpoints
and branches are not overwritten or presented as having received this training.

Each lifetime gets eight authored numerical example tasks. Four families:
a+b; a*b+c; (a-b)*c; a+k, with rotated input positions. Task solutions are not
passed to the searcher; it receives twelve numerical input/output examples.
After each proposed change, 24 new input-disjoint tasks are used once for
admission. Reusing their inputs, relabelling them, or moving them into later
training is rejected. The proposed policy must not solve fewer admission tasks
and must lower the cost-sensitive score by at least 5%. This finite gate is
NOT a proof of arbitrary retention or a calibrated statistical confidence level.

Object search: work=100000, evaluations=384, max_nodes=11, frontier=512,
edits_per_expansion=24. Meta search: work=100000000, evaluations=120,
max_nodes=23, frontier=256, edits_per_expansion=12. Lifetime work cap=5000000000.
All unsuccessful proposals, nested searches, comparisons and used gate inputs
remain accounted for. Rejected candidates stay in evidence, not in active control.
Neutral intermediate syntax is retained inside each bounded search frontier.

The objective is solved-case measured work; an unsolved task receives the fixed
2*work_budget plus capped residual loss. This combines success and cost; report
raw solved counts, work and regressions alongside it. Work counts VM evaluation,
policy execution, edits/scans and nested calls; it is not a complete CPU instruction
count and does not by itself establish wall-clock speedup.

## Frozen outcome tests

Write exact source hashes and all eight trained checkpoints before evaluation.
No further learned weights, code or hyperparameter changes based on these results.

For each lifetime compare learned, cold loss-first, FIFO, and authored
loss-plus-size policies. Same initial expression, example data, library, seeds
and resource caps in every arm. 64 fresh tasks from the training families plus
64 tasks from new families: a*b-c; a*(b+c); if a<b then a else b; a XOR b.
For every inferred program also evaluate 32 unseen numerical inputs. Do not
count fitting the supplied examples as proof of a general rule. Report failures,
false fits, sample-generalizing solves, costs and each seed separately.

Producer-swap test: for each lifetime eight independent comparisons. Hold the
TARGET policy cold and identical in all arms; swap only the PRODUCER (learned,
cold, authored loss-plus-size). Same eight construction-training tasks and meta
budget per pair. Evaluate each resulting child on 32 new tasks not used during
its creation. Report wins/ties/losses, child performance and total creation cost.
Winning as a solver alone is not a positive producer-swap result. This test
measures one bounded further improvement step, not unlimited acceleration.

Report acquisition cost separately from savings on frozen solves. Do not assert
amortization if the measured deployment workload has not repaid acquisition.
No AGI or human-age claim follows from accepted generation counts.

## Additional external source

Reserved source: SyGuS-Org/benchmarks, comp/2017/CLIA_Track tree
b7d6bffdda8b0a00ea841a1c7d0171b4cfcbe465 (directory metadata seen; contents not yet read).
Select by syntax only: single Int-returning function with one to three Int
arguments. Include all selected files in the attempted denominator. Unsupported
constraints/grammar are reported as such, not removed after failures.

The adapter may derive numerical teaching examples from the PUBLIC formal
constraints using an independent SMT engine, then pass ONLY examples to AETERNA.
It may not use a reference solution, task name, SMT solving state, or hidden test
outcome as solver features. Independently check candidates against the original
constraints, and distinguish example fits from verified solutions. This is a
PBE adaptation of public external synthesis tasks, not an official SyGuS score
or ARC-AGI-3. Seed 101 is the frozen external learned policy; the other seeds
cannot replace it after the result. External caps fixed at work=2000000,
evaluations=8192, max_nodes=15, frontier=2048, edits_per_expansion=24.

## Evidence and attribution

Preserve development negatives, source, raw admissions, frozen checkpoints,
all test arms, producer lineages, CPU timings, and portable executable. Check
existing protection/continuity tests and separate-process replay. Research
precedents include Michael A. Lones OptoPsh (2021) and Push autoconstructive
programs; this implementation must not claim those ideas as a new discovery.
Strong recursive self-improvement is established only to the extent supported
by the independent producer comparisons, not by relabelling accepted revisions.
