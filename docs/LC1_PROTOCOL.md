# LC1 — learned requirements for constructing missing computations

Base: main/research 6bb3bf83cb9eeb977dea0dec49b9e24ed4ff9f6e.
Date: 2026-10-18. This protocol is written after disclosed development, not
before the entire research effort. No RSP1/RSP2 or historical external result
is replaced by this work.

## Hypothesis and status before the confirmation run

An initial attempted edit-credit predictor was evaluated in four development
variants. It did not establish a consistent cost-effective advantage; it is
not promoted. Immediate loss reduction can undervalue a useful intermediate
computation. The next mechanism learns inverse computations from observed
executions, uses them to derive requirements for an unknown subexpression,
and checks every proposed completion in the original forward VM.

The implementation at this stage is experimental. Development on 96 numeric
problems gave 93 held-out-checked solutions versus 62 from existing induction.
These 96 are exposed development cases, not independent confirmation. Before
confirmation the unlinked control also gets the same conditional expression
grammar. No improvement or intelligence result is promised by this protocol.

## Information and trust boundaries

The inverse learner receives numerical execution records, not an inverse
formula. Conditional experiments execute both alternatives on matched inputs;
contradictory inverse labels are explicitly recognized as non-identifiable.
The acquired rules are hints: incorrect, partial or inapplicable inverses may
not prune the ordinary grammar or turn a forward mismatch into a solution.
All support rows are checked before a result is returned. Probe rows are only
available to the evaluator after the solver has returned its program.

Learning, variable binding, constraint execution, enumeration and verification
are software computations; this is not a claim that SNN acquired these laws.
The generator, primitives and search envelope are authored. Acquiring inverse
rules and improving object-level search are not automatically proof of a
stronger producer or sustained recursive self-improvement.

## Frozen confirmation

After implementation and source hashes are frozen, use eight independent
internal-study seeds 6301..6308. Each receives executed arithmetic and selector
observations and disjoint validation observations. No external test case is a
training observation. Each frozen book is tested on the same 64 fresh instances
of the existing 12-family numeric suite, plus 32 new four-input compositions.
Keep every task in the denominator, including unsupported or failed searches.

Compare linked acquired rules, the same acquired state with links disabled,
existing systematic induction, and existing loss-plus-size RSP2. Where an old
API cannot express a task, report that limitation rather than substitute a
silently easier cohort. Charge both acquisition and inference; don't turn a
metered-work score into wall-clock or energy claims. Test outcomes are not
feedback for selecting or updating the frozen book.

External examination is a replay of all 21 original public SyGuS tasks from
the RSP1 external artifact. This cohort is already exposed and is therefore
an external regression, not a fresh independent qualification or an official
SyGuS result. Preserve the original examples and constraints. Check returned
programs against those constraints with the original bounded-integer checker,
including overflow/division validity. Unsupported cases remain in the total.

## Release boundary

Add a bounded computation tool to the existing Learner/ScientificRuntime only
through an explicit opt-in interface. Preserve ordinary RSP2 admission,
retention, external permissions and existing checkpoint behavior. A successful
new program or faster search is not permission to replace a global policy
without its corresponding qualification. Preserve negative development data,
source bindings, exact acquired state, all per-case results and a runnable
reproduction. Do not label this experiment human-level intelligence.
