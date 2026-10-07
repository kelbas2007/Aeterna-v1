# INTEL-1 REPAIR-1 — UNKNOWN-GOAL BOOTSTRAP FALLBACK

Status: **PREREGISTERED AFTER INTEL-1 FAIL, BEFORE COGNITIVE REPAIR**

Date: 2026-10-07

## Single permitted change

In `ScientificRuntime::propose`, preserve all current representation/rival/goal-conditioned priorities.

Only when:
- no representation-specific action applies;
- no rival probe applies;
- the goal-conditioned active selector returns no action,

allow the already-qualified G16 generic learned-drive selector
`choose_phase_native_abstract_learned_drive_action()`
to propose a general epistemic action.

No new feature, weight, world metadata, reward, transition table or planner is introduced.

## Rationale

A disconnected goal cannot propagate relevance through transitions that have not yet been learned. General exploration is therefore the generic bootstrap phase; as soon as the model establishes goal-relevant structure, the existing goal-conditioned selector regains priority.

## Acceptance before new INTEL freeze

- new unit/integration witness: cold abstract target with no known goal path returns a generic epistemic action;
- once a goal-relevant known path exists, goal-conditioned action remains preferred;
- G16, G17, G18, G19, G20, G21, G22, G23 and Human Protection regressions PASS;
- Release PASS;
- source guard shows no evaluator/world constants.

No other cognitive change is allowed in Repair-1.
