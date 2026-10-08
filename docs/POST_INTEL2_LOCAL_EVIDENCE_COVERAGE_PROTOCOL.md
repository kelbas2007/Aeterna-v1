# POST-INTEL-2 — LOCAL EVIDENCE COVERAGE REDESIGN

Status: **PRE-REGISTERED BEFORE COGNITIVE SOURCE CHANGE**

Date: 2026-10-08
Branch: `unified-operation-competition`

## Preserved verdict

INTEL-2 run `37687243350` remains a burned **FAIL**.
Its frozen cognitive baseline remains `07b44fb8e00837568bc9655e760eb031d1944dff`.

No later diagnostic or redesign may relabel that run.

## Burned-pack diagnosis

Read-only diagnostics on branch `post-intel2-c-diagnosis` reproduced the World C failure without changing the frozen cognitive source.

Key observations:

- after World A/B, World C had ample unused carrier capacity;
- corrected environment terminal resets removed the earlier episode-loop artifact;
- after 450 history trials, no contextual hypothesis had been born:
  - `contexts=[]`;
  - `u2=[]`;
- both predecessor sides at the aliased junction executed only motor 0:
  - `junction_actions=[[225,0,0,0,0,0],[225,0,0,0,0,0]]`;
- the direct local-unknown selector changed from motor 0 to motor 1 after the first local fact;
- the abstract learned-drive selector continued returning motor 0;
- learned drive weights were effectively equal:
  - DIRECT_UNMODELLED = 0.99999994;
  - REACHABLE_FRONTIER = 1.0.

Therefore the starvation is caused by the evidence-order tie-break in
`choose_phase_native_abstract_learned_drive_action`:

when epistemic scores are equal, current code prefers the motor with more
**global** supported transitions over the lifetime. A motor made common in
earlier worlds can therefore dominate a locally under-sampled motor in a new
state indefinitely.

This prevents the factual action coverage required for a context collision to
be observed. U1/U2 never receive a useful context hypothesis because one is
never born.

## Architectural diagnosis

Global transfer evidence is useful, but it must not override missing local
evidence when two actions have equal learned epistemic value.

The missing invariant is:

> equal epistemic value -> prefer the action with less factual support at the
> current physical state before using lifetime-global support.

This is not a World C rule. It applies to any new state where a historically
popular action competes with a locally under-explored action.

## Allowed source change

Only the generic learned-drive evidence ordering may change.

Add a task-agnostic carrier-local support measure for (physical state, opaque
motor action).

For equal learned drive score within the existing epsilon:

1. lower local supported evidence wins;
2. if local support is equal, preserve the existing global-support preference
   in the abstract selector;
3. if both are equal, preserve deterministic opaque motor order.

Apply the same local-evidence-first tie rule to the base learned-drive selector
so the generic mechanism is consistent across native and abstract carriers.

Do NOT change:

- learned drive features or weights;
- U1 meta-control weights/scorer;
- U2 ecology;
- contextual promotion thresholds;
- world/task IDs or evaluator hints;
- action semantics;
- Human Protection;
- INTEL-2 evaluator or verdict.

## Required causal witness

Construct a generic carrier state in which:

- two opaque actions have equal learned epistemic score;
- action A has larger lifetime-global support but is already locally supported;
- action B has lower/zero local support;
- no task label or correct-action signal is supplied.

Required:

- FULL chooses B;
- matched old global-first ordering chooses A;
- after B receives factual local support, coverage advances rather than
  permanently repeating A;
- phase/weight lesion controls for the learned drive remain causal;
- G16 and P4 learned-drive regressions remain PASS.

## Burned-pack diagnostic check

The old INTEL-2 seed may be used only as a non-verdict diagnostic.

Required diagnostic evidence before a fresh system verdict:

- World C junction executes more than one opaque motor;
- a factual contextual collision is born;
- useful context can promote under the corrected episodic diagnostic.

This does not change the INTEL-2 FAIL.

## Next system verdict

If the generic redesign and regressions pass, freeze a new cognitive SHA and
open a **new independently seeded system verdict**. Do not rerun the burned
INTEL-2 pack as a new qualification.
