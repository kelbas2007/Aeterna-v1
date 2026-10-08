# POST-INTEL-2 — REPRESENTATION OWNERSHIP IN UNIFIED COGNITION

Status: **PRE-REGISTERED BEFORE SOURCE CHANGE**

Date: 2026-10-08
Branch: `unified-operation-competition`

## Preserved evidence

- INTEL-2 run `37687243350` remains burned FAIL.
- Local-evidence coverage redesign result remains
  `MECHANISM PASS / SYSTEM INSUFFICIENT`.
- No old pack may become a new project verdict.

## Diagnosed architectural defect

Contextual, perceptual and compositional selectors already implement the
physical invariant that an applicable acquired refinement makes the inherited
parent representation insufficient.

Examples in production cognition:

- contextual refinement returns `(true, None)` when required context is
  present but no supported refined action exists;
- perceptual refinement returns `(true, None)` when a promoted raw-feature
  distinction applies but its physical path is unavailable;
- compositional refinement returns `(true, None)` when promoted applicable
  structures do not yield one coherent refined state.

The legacy runtime respected this by not falling through to the ambiguous
parent.

The unified collector currently loses that invariant: it adds a refinement only
when `(true, Some(action))`, but if the selector returns `(true, None)` it
continues collecting parent-level rival/goal/general proposals.

Burned diagnostic evidence after context promotion showed exactly this:

- side 0: context `(true, Some(1))`, correct;
- side 1: context `(true, None)`;
- parent goal still proposed action 1;
- unified selected action 1;
- held-out side 1 scored 0/16.

## Architectural invariant

A physically applicable refinement owns the current representation scope.

Therefore:

> if any enabled contextual/perceptual/compositional mechanism reports
> `applicable=true`, the ambiguous inherited parent state must not emit
> external-action proposals in parallel.

This is an applicability rule, not a source-priority rule.

Several simultaneously applicable refinements may still emit proposals and
compete through unchanged U1/U2 scoring/ecology. No authored ordering among
context/perceptual/compositional is allowed.

## Allowed source change

Only unified proposal collection may change.

1. Evaluate contextual, perceptual and compositional applicability.
2. Add every actionable refinement proposal exactly as before.
3. If no refinement claims applicability, collect parent-level:
   - rival discrimination;
   - goal-active;
   - general epistemic.
4. If one or more refinements claim applicability, do not emit those coarse
   parent proposals.
5. If refinements claim applicability but none yields an action, return no
   supported action (fail closed) rather than bypass the acquired
   representation.
6. U1 scorer, U2 ecology, learned weights, representation learning/promotion
   thresholds and Human Protection remain unchanged.

No world ID, task ID, correct action, motor semantics or evaluator state may
enter cognition.

## Required witness

Generic witness:

- acquire a promoted refinement over an aliased parent;
- make the refined physical state initially lack a supported action;
- verify parent goal/general/rival cannot bypass it;
- while learning is enabled, refined-state local exploration must acquire its
  own action support;
- after factual support, refined representation supplies the action;
- when refinement is not applicable, parent reasoning remains available.

Regressions:

- U1/U2/U3;
- P4/G16;
- G20-G23;
- Human Protection;
- Release.

## Burned diagnostic

The old INTEL-2 C pack may be used only diagnostically.

Required before a fresh verdict:

- context still promotes;
- both histories acquire usable refined actions;
- held-out C >=28/32, each side >=13/16.

Passing this diagnostic does not alter INTEL-2 FAIL.

## Next verdict

Only after generic regressions and the burned diagnostic pass may a new frozen
core and independently seeded system verdict be opened.
