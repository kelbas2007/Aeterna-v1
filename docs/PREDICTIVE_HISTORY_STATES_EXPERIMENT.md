# Predictive short-history states from actual action consequences

2026-10-19 · AETERNA PR #9 · opt-in developmental research only.

## Problem and lineage

T-SR2's single progress sensor aliased distinct external conditions: a
preparatory motor might change a gate, resource or stage without changing
visible progress. The previous newly synthesized goal-value VM failed to
solve this representation problem. The 32 T-SR2 cases are EXPOSED and
are not reused as held-out scientific evidence here.

This research takes the **predictive state splitting** idea from small
automata-learning / computational-mechanics projects (DB-RPNI, transCSSR,
AALpy) and independently implements a much smaller finite-history
case in the SAME existing Rust Learner. No copied source code, separate
action governor or externally authored motor labels.

## Implemented, bounded opt-in scope

- `enable_constructing_self_directed_control` then
  `enable_constructing_predictive_history_states` (planning_work >= 32,000).
- Each learned context is an opaque one-action predecessor suffix.
  Only adjacent REAL PRE/action/POST transitions, with no event gap or
  external restart boundary, may supply predecessor evidence.
- For a given target motor, four earlier cases in each of the target
  and a different predecessor group plus THREE matched-state,
  actually differing outcome contrasts are required.
- A rule must also predict three later real target-context transitions
  using the earlier prefix and remain identical after the holdouts;
  all work is charged to the same finite planning budget.
- State for planning is (visible sensor vector, previous opaque action).
  A stable, independently verified no-op may be retained as an
  intermediate preparation but not credited as a one-step goal.
  A 2+ step path with a certified context-dependent consequence may be
  offered to the existing U1 native arbitration as `predictive-history`.
  The first action always needs its usual HumanProtection permit; only
  its real POST is recorded.
- Shared Learner/U1 checkpoint stores the single opt-in flag and
  actual journal; no hidden-world state is provided by the evaluator.
  Experimental source/offer telemetry distinguishes a proposed path
  from a selected and executed action. Post-freeze real POSTs cannot
  become new training/qualification examples.

## What the exposed development tests establish

A small protected virtual world has one invisible charge flag. One
opaque action primes it with no visible progress; another makes
progress iff charged; a distractor yields the same visible observation
without charging. Executed actions vary predecessor while holding
observed PRE and tested motor constant. The engine must propose
a learned two-step plan if and only if the history-conditioned
effect is identifiable, stable under chronological holdouts and
preparation is available. A control with no history effect must not
produce the new kind of proposal. Underfunded planners abstain.

**These development worlds are exposed and synthetic. A U1 proposal
is not a selected action, and a successful engineering test does not
prove transfer to disjoint POMDPs.**

## Scientific limitations and necessary next trials

History length is ONE; no arbitrary finite-state automaton, probabilistic
CSSR reconstruction, HDC-native symbol invention, stochastic world
inference, persistent unobserved battery capacity or multi-key
composition is established. Matching PRE observations reduces some
confounding, not all. Further development must support action histories
of variable length and active, budgeted discriminating interventions.
For real transfer claims, preregister a disjoint state-family test
before authoring the evaluator and compare native U1 with/without the
same opt-in under identical permitted external actions and budgets.
T-SR1/T-SR2 are permanently exposed and may not be silently reused.
