# G1 — RAW RASTER AUTONOMOUS RELATIONAL DISTINCTION

Status: **PRE-REGISTERED BEFORE G1 RUNTIME IMPLEMENTATION**

Date: 2026-10-06

## Scientific question

Can the EvoPhase carrier acquire a translation-tolerant relational distinction directly from a raw raster, then use that carrier-owned distinction to change a held-out motor choice, without the host supplying object labels, coordinates, orientation names, relevant pixels, a map, or the correct action?

This is the next gate after G0. It is deliberately narrower than general object discovery and does not claim program invention, planning, ARC competence, or AGI.

## Environment contract

Input to cognition:
- one lossless 12x12 scalar raster, flattened only for transport;
- two opaque motor tokens;
- continuation/terminal state;
- protected factual Need outcome.

Private evaluator only:
- constructs two classes of three-pixel patterns at varying translations;
- one private class rewards motor 0;
- the other private class rewards motor 1;
- class names, orientation names and generating coordinates never enter cognition.

The training set contains several translations of each private class.
Held-out evaluation uses translations not present in acquisition.

## Allowed inherited structure

The substrate may know retinotopic adjacency: neighboring sensory cells occupy neighboring positions in the input sheet. This is analogous to fixed local connectivity in a sensory surface and is not a task label.

The substrate may use generic phase/HDC operations, local residuals, structural growth and generic exploration pressure.

It may not contain a rule equivalent to "horizontal -> motor 0" or "vertical -> motor 1".

## Candidate carrier mechanism

G1 tests two carrier-owned structures.

### 1. Context-local exploration memory

The carrier binds the active sensory population into an HDC/FHRR scene trace. Motor novelty is local to similar carrier context rather than only to global motor usage. This exists to keep unsupported motor hypotheses testable in a novel scene.

### 2. Relational motif

From active sensory cells the carrier may acquire a sparse relative-offset signature. A motif stores:
- relative offsets, not absolute coordinates;
- motor binding;
- factual outcome support;
- confidence/utility;
- support/revision counters.

A motif is MODEL authority. It is not a factual object label.

The same learned relative-offset structure may match a held-out translation because its parameters are relative.

## Matched acquisition

To avoid giving GENUINE a different factual curriculum merely because its new representation already affects policy during training:

- GENUINE: motif formation ON, motif readout OFF during acquisition.
- NO_FORMATION: motif formation OFF, motif readout OFF.
- all other carrier mechanisms, raw observations, actions, outcomes, budgets and initial state are matched.

The acquisition PRE/action/Need sequence must remain identical between these two arms.

After acquisition:
- G_READOUT is a clone of GENUINE with motif readout enabled and learning frozen;
- G_NO_READOUT is the same acquired state with motif readout disabled;
- NO_FORMATION contains no acquired motifs.

Thus formation and readout are distinguishable.

## Acceptance

G1 PASS requires all of the following:

1. Rust tests and release build pass.
2. GENUINE and NO_FORMATION have identical acquisition factual tuples.
3. GENUINE acquires at least one relational motif; NO_FORMATION acquires none.
4. At least one acquired motif is translation-relative rather than tied to an absolute raster address.
5. On both held-out translations, G_READOUT selects the evaluator-correct opaque motor before receiving the held-out outcome.
6. G_NO_READOUT and/or NO_FORMATION loses the held-out advantage.
7. REAL/MODEL/IMAGINED authority separation remains intact.
8. No task label, orientation name, hidden coordinates, correct action or host-side classifier reaches cognition.
9. Resource bounds remain fixed across matched arms.

## Failure interpretation

A failure is not permission to insert the missing class as a hand-written feature.

Classify it as one of:
- exploration did not obtain discriminating factual outcomes;
- relational motif formation did not occur;
- motif formed but did not generalize across translation;
- motif generalized but downstream motor readout ignored it;
- host/observer integrity failed.

Every failed physical/CI qualification remains in the ledger.

## Next gate after PASS

Only after G1 passes do we move to G2: carrier-owned selection of informative experiments in an unknown interactive world. G2 must use the same no-hidden-solver ownership boundary.
