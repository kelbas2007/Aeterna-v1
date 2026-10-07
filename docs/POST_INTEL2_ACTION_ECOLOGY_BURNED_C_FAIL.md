# Post-INTEL-2 action ecology — burned C regression failure / credit-leak diagnosis

Date: 2026-10-07

Branch: `post-intel2-action-ecology`

## Preserved generic result

Run `37691250053` passed the prospective generic state-action ecology witness and all
U1/U2/U3/unified/G20-G23/Human Protection/Release regressions.

Generic witness:

```text
stale_action=0
stale_exec=6
switch_round=6
switch_action=1
dormant_weight=0.04061017
reactivated_weight=0.96942955
legacy_control_action=0
```

## Burned C diagnostic result

Run `37691590751` remained FAIL as a diagnostic regression.

Despite state-action candidate identity being present, the aliased junction still chose
motor 0 on both predecessor sides for all 450 scored training trials.

The motor-0 state-action record accumulated 900 observations and converged near:

```text
weight / authority = 0.3939394
```

rather than becoming dormant.

## Diagnosis

`ScientificRuntime::step_unified` currently computes selected proposal usefulness as:

```text
max(task_outcome, global_info_gain)
```

where `global_info_gain` is derived from a whole-organism
`PhaseUnifiedKnowledgeSnapshot`.

In the full all-refiners lifetime, representation learners can increase evidence or
structure around the same factual step even when the selected external state-action
itself makes no task progress and adds no new transition knowledge.

Thus unrelated refinement activity leaks positive credit into the selected generic
state-action U2 record. This prevents stale action dormancy.

The clean generic witness did not expose this because no refinement learners were
enabled.

## Interpretation

The state-action ecology mechanism is causally functional, but its integrated credit
assignment is too global.

This is not a failure of capacity:
the earlier C diagnostic had large remaining carrier capacity.

No INTEL verdict is changed.
