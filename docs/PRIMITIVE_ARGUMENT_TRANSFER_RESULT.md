# Existing operation definitions with acquired argument bindings

Status: OPEN DEVELOPMENT MECHANISM AND RUNTIME TEST PASS.
Evidence run: https://github.com/kelbas2007/Aeterna-v1/actions/runs/37967510317
Starting source commit: `12bd5a0cfd195eef34e945bc1c5f52208624e61c` (guarded integration applied in this run, then tested).

## What was measured

The existing acquired-operation learner first acquired a two-input relation from factual environment outcomes. No target function is added to the new engine code. Its original definition and physical synapses then remain unchanged while argument calls are transferred to different sensor channels.

Explicit argument-call mechanism control: 96/96 correct across four new ordered argument pairs, eight Boolean cases and three new numeric magnitudes. The original unbound call scores 48/96 on those same target cases. This is a control of argument mapping, NOT a superiority claim over another fully retrained learner.

Runtime control: one continuing training organism encounters two externally scheduled new input-pair tasks. It receives only the ordinary selected action and factual outcome, never the correct pair or a binding. The binding selector uses the existing bounded per-action factual buffers. It selects on an earlier prefix and requires at least eight subsequent validating facts; a failing suffix does not select an alternative binding. On 48 heldout numeric cases, the existing protected runtime performs correct goal actions using original source-operation roots. Checkpoint/restart preserves the operation and factual evidence used to reconstruct the bindings. Direct prediction and binding inspection do not update any retained state.

The feature is opt-in (`set_phase_primitive_argument_transfer`). The normal induction prediction path uses validated calls before ordinary program prediction. Factual observation checks reuse before rebuilding a definition. Existing acquired-operation and real-program pipeline regressions ran in the same job. Their previously published real-dataset performance FAIL verdicts are not altered by this test.

## Exact emitted measurements

```text
PRIMITIVE_ARGUMENT_MECHANISM correct=96/96 old_unbound=48/96 same_definition=true no_query_learning=true lesion=true restore=true
PRIMITIVE_ARGUMENT_BINDING_DIAGNOSTIC task=0 calls=[(0, 70, [1, 2], 107, 8), (2, 55, [1, 2], 60, 8)]
PRIMITIVE_ARGUMENT_TASK task=0 correct=24/24 factual_training_actions=159 original_calls=[(0, 70, [1, 2], 107, 8), (2, 55, [1, 2], 60, 8)]
PRIMITIVE_ARGUMENT_BINDING_DIAGNOSTIC task=1 calls=[(0, 70, [0, 2], 120, 8), (2, 55, [0, 2], 59, 8)]
PRIMITIVE_ARGUMENT_TASK task=1 correct=24/24 factual_training_actions=149 original_calls=[(0, 70, [0, 2], 120, 8), (2, 55, [0, 2], 59, 8)]
PRIMITIVE_ARGUMENT_RUNTIME correct=48/48 protected_actions=48 one_training_lifetime=true arguments_not_supplied=true original_definitions_unchanged=true checkpoint=true
```

## Boundaries — essential to interpreting this result

This does NOT establish spontaneous invention of a new phase-native constructor or general intelligence. The source relation and the two later tasks are externally scheduled synthetic curricula. Generic perception, branch interpreter, and bounded argument-search algorithm remain inherited software. Binding views are reconstructed from factual buffers; they are not independently learned physical argument-binding synapses. The original computation is read through its physical definition, whose lesion makes the parameterized call unavailable.

Current limit: at most eight channels and three structurally used raw arguments, with injective binding candidates. Explicit calls retain original support guards and require a fully observed frame of the existing width. Search is capped at 32768 definition evaluations per action. This does not yet establish transfer to arbitrary input width, unrestricted objects, new computational node types, speed advantage or cold open-ended primitive invention.

The concrete new capability is reuse of an unchanged, acquired computational definition on new evidence-derived argument mappings inside the existing predictor. No MAIN or frozen INTEL-4 source is modified by this workflow.
