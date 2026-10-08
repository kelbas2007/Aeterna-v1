# POST-INTEL-2 — PROMOTED REPRESENTATION OWNERSHIP V2

Status: **PRE-REGISTERED BEFORE V2 SOURCE CHANGE**

Date: 2026-10-08
Branch: `unified-operation-competition`

## Preserved evidence

All prior INTEL verdicts and post-INTEL diagnostics remain unchanged.

## Corrected invariant

An unpromoted candidate is a hypothesis/experiment request. It may compete for
an action through U1/U2, but it does not invalidate the inherited parent state.

A promoted, physically applicable refinement is an acquired representation.
Only that structure may claim representation ownership and suppress coarse
parent reasoning.

## Ownership conditions

Context:
- promoted non-retired candidate on current base;
- if predecessor is known, it must contain that predecessor;
- if predecessor is unavailable while the current base has a promoted context
  split, ownership remains fail-closed.

Perceptual:
- promoted non-retired candidate on current base owns the ambiguous parent;
- inability to resolve a coherent side remains fail-closed rather than parent
  fallback.

Compositional:
- one or more promoted non-retired candidates on current base own the parent;
- disagreement or damaged physical path remains fail-closed.

Unpromoted candidates may emit experiment proposals but never set the ownership
mask.

## Unified collection

1. collect experiment/action proposals from all refinement mechanisms;
2. compute promoted ownership independently of experiment applicability;
3. if no promoted refinement owns the current representation, also collect
   parent rival/goal/general proposals;
4. if promoted ownership exists, parent proposals are suppressed;
5. U1/U2 still arbitrate among multiple actionable refinement proposals.

No source-class priority ordering among refinements is introduced.

## Required diagnostic

On the burned C diagnostic:

- unpromoted context must continue gathering evidence;
- context must promote;
- after promotion, refined-state exploration must not be bypassed by the
  ambiguous parent;
- held-out >=28/32 and both sides >=13/16.

P4/G16 and U1-U3/G20-G23/HP regressions remain required before a fresh verdict.
