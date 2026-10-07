# INTEL-2 FAIL1 postmortem — evaluator stop plus real action-coverage bottleneck

Date: 2026-10-07

Authority run: `37687243350`
Frozen cognitive baseline: `07b44fb8e00837568bc9655e760eb031d1944dff`
Burned authority seed: `37687243350`

## Preserved record

`docs/INTEL2_RESULT_FAIL1.md` remains an immutable record of the first sealed INTEL-2 authority run.
The pack remains permanently burned and MUST NOT be reused as a new verdict.

## Correction to the first observed stop

The authority evaluator stopped in World C with:

```text
World C unified action: NoSupportedAction
```

Burned-pack diagnostic run `37688460610` reproduced the stop at:

```text
k=7 world_state=16 trials=2 error=NoSupportedAction
```

In the sealed pack World C roles were:

```text
C=[8,1,7,19,16,14]
```

so state `16` was the World C goal.

The evaluator helper `step_u` maps a legitimate runtime `StepOutcome::GoalReached`
into `RuntimeError::NoSupportedAction`. Therefore the exact earliest logged stop was
an evaluator terminal-semantics error, not evidence that EvoPhase had no supported
proposal at the junction.

No PASS is inferred from this correction.

## Burned-pack architecture diagnosis after evaluator-only terminal correction

A diagnostic-only environment reset was applied after C goal/dead terminals without:
- cognitive reset;
- world/task signal entering cognition;
- model tuition for the reset;
- cognitive source change.

The frozen source then continued without `NoSupportedAction`, but still failed to
form a context split.

Run `37689180007` / trace run `37689548713` showed:

```text
recruited_before_c = 115 / 640
recruited_after_c  = 135 / 640
junction_actions side0 = [225,0,0,0,0,0]
junction_actions side1 = [225,0,0,0,0,0]
contexts = []
u2 = []
```

Capacity exhaustion is therefore ruled out.

At the first junction visit:

```text
direct_unknown = Some(0)
general        = Some(0)
```

After motor 0 had factual support:

```text
direct_unknown = Some(1)
general        = Some(0)
```

and this pattern persisted.

The general learned drive kept selecting motor 0 because its known successor exposed
reachable frontier novelty. In this pack motor 0 led to the same dead successor under
both predecessor histories, while the useful context-sensitive motors were 1 and 3.

Consequently G21 never received the same anchor action with different predecessor /
successor pairings and could not recruit a contextual hypothesis.

## Architectural bottleneck

Unified cognition currently gives persistent U2 ecology identity only to explanatory
structure proposals. Generic goal/general action proposals use
`persistent_candidate_id=None`.

Therefore a repeatedly selected generic action can remain authoritative despite
repeated factual zero task/structure utility. U2 cannot make that state-action option
dormant, so learned reachable-frontier pressure can monopolize behavior indefinitely.

This is a general lifetime action-allocation defect, not a World C constant.

## Scientific status

The original sealed pack is burned and is not rerun as an authority verdict.

The project must:
1. repair generic lifetime state-action authority prospectively;
2. correct repeated-episode terminal handling in the next evaluator;
3. freeze a new cognitive SHA;
4. run a newly seeded INTEL-2 verdict.

The burned diagnostics are postmortem evidence only.
