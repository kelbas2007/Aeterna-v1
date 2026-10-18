# RSP2: explicit search semantics and dual-role admission

Base: 81adeb621bc511e6f65b65d628bc78ab4b6ad2a4 (RSP1 implementation 7c3aaa0b).
This is a preregistration of the next bounded implementation and evaluation,
not a claim that stable recursive improvement or general intelligence exists.

## Observed development defects and changes

1. Non-policy expressions passed the generic integer type check and consumed
   nested evaluation slots. Product-shape validity must be tested before an
   objective call; syntax-generation work is still charged.
2. An evolvable function previously saw raw RNG draws. A change could remap a
   draw without learning a useful search preference. New mode scores actual
   operator/site alternatives; random sampling belongs to the host. A fixed
   positive rank-based distribution and the old exploration reserve prevent
   permanent suppression of alternatives. All scoring overhead is metered.
3. Unchanged offspring can look like a better producer when the control's
   offspring are worse than their initial state. A producer is tested against
   the SAME initial offspring and the unchanged offspring as a third control.
4. A single aggregate solver score does not establish productive improvement.
   Production fitness includes worker and maker roles. Admission requires two
   disjoint fresh cohorts, comparison to parent and epoch baseline, positive
   offspring contribution, and nondecreasing verified solve counts in both
   roles. The tests are finite development gates, not a universal guarantee.

An explicit semantic epoch preserves RSP1 history but starts its new descriptor
interface from a uniform variation/site preference. Old noise-input policies
are never silently interpreted as new descriptor policies. The candidate VM
remains bounded and cannot read a target program, held-out reference, external
permission or acceptance mutator. Typed edits, features, budgets and the
admission algorithm remain authored; no claim of an invented learning kernel.

## Development already performed (not confirmation)

Six local exploratory variants used seeds 21 and 22 and the eight RSP1 teaching
families: raw-RNG joint objective; descriptor hard choice; worker/producer
counterfactual; canonical preferences; soft positive weights. A manual scan of
45 simple legacy policies was also performed. All are development. They may
inform implementation but their apparent gains are not held-out evidence.
The old 512 related/512 transfer and public SyGuS 2017 cases are now EXPOSED
regressions. Do not relabel them independent after this development.

## Confirmation contract

- Train eight independent new epochs, seeds 301..308, at most 24 attempted
  replacements each. Host provides a mixed authored teaching distribution;
  do not claim autonomous curriculum creation.
- Per attempt: 8 worker tasks and 1 producer case for proposal fitness; two
  cohorts each with 64 worker tasks and 8 producer cases. Each producer case
  learns from 4 tasks, then its fixed offspring is checked on 8 other tasks.
- Object limits: 50000 metered work, 256 evaluations, 11 nodes, frontier 512,
  24 edits/expansion. Child creation: 5000000 work, 16 evaluations, 23 nodes,
  frontier 256, 8 edits. Outer creation: 100000000 work, 100 evaluations,
  23 nodes, frontier 256, 12 edits. Count training, rejected production,
  validation, control production and actual policy evaluation cost.
- Admission: on BOTH cohorts at least 2 percent aggregate cost improvement
  against parent AND epoch baseline; no fewer verified worker or offspring
  solves; positive net offspring utility relative to unchanged offspring,
  and at least one changed beneficial offspring. Failure keeps the parent.
- Freeze all eight epochs before confirmation. Compare frozen worker and
  producer roles to the same-kernel epoch baseline and an authored loss+size
  control, with identical tasks, initial knowledge and resource ceilings.
  Do not pick a winning seed; retain all results and all failed attempts.
- Reserve 128 fresh worker tasks and 16 producer cases per epoch from the
  mixed teaching family distribution, and 128 tasks from different authored
  expression families. Expose labels only to evaluators, not candidate policies.
- Public SyGuS 2017 source from RSP1 is an exposed external regression. A new
  source cohort may be reserved by syntax/file metadata before content fetch;
  otherwise make no claim of a new independent external result.
- Report solve counts, paired changes, full metered costs, verified expression
  results, unchanged-state controls and per-epoch outcomes. Generations and
  green implementation tests are not intelligence scores. Production gains
  must not be inferred solely from child-solver gains.

Required integration: same Learner/ScientificRuntime, explicit epoch API,
checkpoint, no second external commander, no main or frozen-INTEL changes.
Negative results remain in evidence. A safe refusal to install a bad policy
is a stability mechanism, not successful recursive self-improvement.
