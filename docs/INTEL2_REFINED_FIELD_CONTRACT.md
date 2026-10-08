# Unified fields must preserve acquired operational representation

Date: 2026-10-08
Branch: post-intel2-c-diagnosis
Pre-change production: 01cd01bda8ccad07a8a5c81b90bfd7af3b2d167e (edge-cost candidate)
Diagnostic source: feaa9e7063acb2f6267262f2d043e200a4f9820e
Diagnostic run: 37770131496; job 113287393345
Artifact 11547044553; SHA256 b6056b045c0ab710e8454bd40bb62294043518fb86f79b6a5b6f94556a024b2b

## Observed cause, not a new intelligence verdict

The edge-cost correction allows context formation, but the unchanged episodic C diagnostic scores 16/32. The new read-only diagnostic reproduced the same acquired context and inspected its physical transition inventory:

- refined cell 541 has only action 1 -> successor 482, support 33;
- refined cell 542 has only action 1 -> successor 480, support 32;
- no other action has been learned from either refined cell.

For the successful predecessor side, frozen direct contextual readout chooses action 1. For the other side it returns applicable=true, action=None, because the useful alternative has never been learned from that refined state.

With learning enabled on diagnostic clones, direct contextual exploration requests action 0 on BOTH refined sides. Unified competition instead selects action 1 on both. Its action-1 fields are identical [0.95, 0.95, 1.0, 0.9726027, 1.0] across opposite histories; proposed context exploration is evaluated using the same coarse parent rather than its acquired refined state.

Source inspection confirms unified_action_fields re-resolves every action against phase_native_abstract_state(sensory), losing the contextual state used by its proposal source. The problem is both continued refined-state acquisition and applicability of coarse-parent predictions, not a missing context cell.

## Prospective bounded correction

Resolve a unique currently operational acquired state before calculating unified action fields.

1. Use factual raw base and factual predecessor/current raw descriptors only.
2. Collect physically coherent, promoted, nonretired refined states from the EXISTING contextual, perceptual and compositional structures. Do not rank by source class or choose by world identity.
3. When exactly one refined state is applicable, use that physical cell for ALL competing actions' goal, epistemic and disagreement fields. Ordinary inherited-base fields remain unchanged when no refinement is applicable.
4. A required missing history/feature, damaged required input link, or multiple different applicable refined states fails closed; do not fall back to a contradictory coarse-parent plan.
5. Use the existing phase-native goal recurrence and frontier equations, including the already tested one-edge discount. No route, action map, threshold change, new operator or host cognitive priority.
6. Keep U1 weights, U2 update rules, source proposal enumeration and existing global motor-confidence/economy fields unchanged to isolate representation loss.

Conservative multi-refinement abstention is an explicit limitation, not a solution to arbitrary joint representation synthesis.

## Required development checks

- Run the existing burned C terminal-reset diagnostic UNCHANGED, including promotion and >=28/32 threshold.
- On a continued A/B/C lifetime, verify correct action on both acquired history sides after learning freeze.
- Necessary context input weight lesion and pi shift must prevent coarse-parent fallback; exact restore recovers without training.
- Missing factual predecessor after restart must not produce a guessed contextual decision.
- Reversed proposal enumeration leaves the action unchanged.
- Edge-cost local and remote-frontier controls remain PASS.
- G20-G23, U1/U2/U3, unified runtime integration, Human Protection and Release remain PASS; preserve all failures rather than changing thresholds.

This is development on an already exposed pack. No historical INTEL verdict is relabeled, no new independent system qualification is claimed, and no other repository branch is modified.
