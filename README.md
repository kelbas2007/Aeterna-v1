# AETERNA v1

A clean no-LLM intelligence research line targeting **full EvoPhase cognitive ownership**.

## Current architectural qualification

**Full physical phase-native execution is NOT yet established.** The recorded task results in `STATUS.md` remain evidence, but they must not be confused with proof that the phase-cell/synapse network itself executes every cognitive operation.

The 2026-10-06 [phase-execution audit](docs/PHASE_EXECUTION_AUDIT_RESULT.md) reproduced a specific gap: the legacy G8 planning path gave identical decisions in 18/18 deterministic matched cases with phase plasticity and structural recruitment disabled and no dormant cells. Its learned transition table remained present; removing that table removed the plan. A standalone version of the same production planner also worked without constructing an EvoPhase instance.

This is a negative architectural witness, not an all-learning ablation and not an invalidation of every earlier result. The original audit and its tests are preserved. The strong ownership requirement below remains unchanged. G10 is preregistered; neither the audit, P1 nor P2 implements or qualifies it.

### P1: acquired shared-synapse execution now tested

A new explicit native mode routes ordinary planning APIs through learned connections in the **same EvoPhase cell and synapse arrays**, with no legacy graph fallback. Tested source: `f33028a5d2809fd7500d46a2021b35f230b1e704`; workflow `37522194429`.

The preregistered P1 run passed 24 deterministic intervention cases and 80/80 freshly sampled bounded task instances. Cutting a necessary internal successor connection gave 0/80; shifting its learned phase offset by pi also gave 0/80; exact restoration recovered 80/80 without relearning. Same-relay/same-synapse factual outcome devaluation changed the decision in 80/80 cases. Separate zero-phase-learning, zero-weight-learning, no-capacity and no-growth acquisition controls each completed 0/80 tasks. The independent graph reference also completed 80/80.

This is **phase-dependent local value propagation**, not forward generated sensory imagination or a complete neural migration of all cognition. The recurrence and winner-take-all competition are inherited; P1 inference uses learned phase calibration rather than evolving oscillator trajectories. Primary tuition averaged 854 factual transition presentations per arm and instance. No data-efficiency, competitive-performance or general-intelligence claim follows.

See the [P1 protocol](docs/PHASE_NATIVE_P1_PROTOCOL.md) and [verified P1 result, costs, evidence and limits](docs/PHASE_NATIVE_P1_RESULT.md). The tested pack is burned. Old G8/G9 results are not relabeled as phase-native results.

### P2: acquired forward sensory model and ordinary action/fact loop

P2 adds shared-synapse forward continuation and learned sensory decoding in `src/phase_forward.rs`. Tested source: `f6503b234f51a11d1a8d930e61a665bae557b7fd`; workflow `37524647331`. The full regression suite and Release build passed; an initial workflow YAML failure is preserved separately in the result record.

All 24 deterministic causal cases and 80/80 one-use generated instances passed the implemented bounded contract. The carrier forecasts 2–5-step sequences by continuing internal membrane activity, without being given intermediate future observations. In the ordinary interaction test, native P1 chooses an action, P2 predicts BEFORE the external world executes it, and factual POST is compared with the prior forecast. All 80 instances reached delayed factual Need with correct relational forecasts.

Necessary internal connection lesions, pi phase shifts and decoder-only phase perturbations each removed the full forecast in 80/80 cases. Exact restoration recovered it, and unrelated lesions preserved it. Separate zero-phase-learning, zero-weight-learning and no-capacity controls each produced 0/80 full forecasts. Factual changed-successor learning corrected the new transition forecast in 80/80 while frozen copies retained obsolete forecasts and unchanged alternative transitions stayed correct.

The result is **a learned associative forward model with physical causal dependence**, not full intelligence. P1 still chooses actions; P2 has not independently demonstrated better decision selection. The decoder reconstructs learned sensory patterns in their tuition coordinate frame, evaluated through relational HDC comparison, not observer-aligned novel image generation. The synchronous phasor update and recognition machinery are inherited. All small transition sets are taught: mean 904 tuple presentations per arm and instance, with 128 additional factual presentations for the revision test. There is no autonomous exploration or data-efficiency claim. The fresh instances share one narrow generator, not 80 independent domains.

See the [P2 protocol](docs/PHASE_NATIVE_P2_PROTOCOL.md), [executed P2 result and limitations](docs/PHASE_NATIVE_P2_RESULT.md), and [ordinary API usage and coordinate-frame limits](docs/PHASE_NATIVE_P2_USAGE.md). P2's observed pack is burned for subsequent design revisions.

### P3: autonomous acquisition without a supplied transition curriculum

P3 adds a phase-native acquisition selector that gives intrinsic value to unmodelled actions and propagates deeper frontier novelty backward through **the same acquired successor synapses**. In the reset-chain test family, wrong actions reset the environment to the start, so the organism must reuse already learned transitions to deliberately return to deeper unknown states.

The first one-use FRESH-P3 authority run `37528857872` at source `52bbbc2654894bbaf8501e6834757dff25f1e242` passed its frozen contract:

- cold FULL acquisition **80/80**, Wilson95 [0.954182,1.000000];
- held-out frozen exploitation **80/80**;
- restore into a newly constructed EvoPhase and solve again **80/80**;
- DIRECT_ONLY **9/80**, seeded random **35/80**;
- no-learning / no-growth / zero-phase / zero-weight controls **[0,0,9,2]/80**;
- changed-law detour repair **78/80**, while frozen stale copies scored **0/80**;
- revised solve + checkpoint restore **78/78**;
- mean initial acquisition cost **20.075** physical interactions;
- mean changed-law repair cost **16.462**;
- legacy graph transition table remained absent.

This is the first bounded result in this line where the tested transition experience is **not handed in as a prepared curriculum**. The organism chooses the physical acquisition actions itself and learns only from their factual consequences.

It is still not general intelligence: the intrinsic frontier-novelty rule is hand-specified, worlds are deterministic and fully observed, and the tested family is narrow. See [P3 protocol](docs/PHASE_NATIVE_P3_PROTOCOL.md), [fresh protocol](docs/PHASE_NATIVE_P3_FRESH_PROTOCOL.md), and [P3 result](docs/PHASE_NATIVE_P3_RESULT.md). The authority pack `c9a3d25d6f64c483` is permanently burned.

## Non-negotiable rule

EvoPhase is the cognitive substrate, not a planner plugin.

Rust may implement substrate physics and the trusted shell (I/O, persistence, serialization, provenance, safety and exact verification), but Rust must not contain hidden task answers or perform task-level cognition on behalf of the organism.

The following adaptive state must be EvoPhase-owned:

- learned sensory distinctions and representations;
- concepts and relations;
- predictive hypotheses and rival models;
- reusable skills / programs and their composition;
- experiment selection;
- imagined rollouts and planning state;
- action policy;
- revision after factual counterexamples;
- structural growth, retirement and reuse.

No LLM is used by the runtime.

## Acceptance chain

A capability is accepted only when the same ordinary organism demonstrates:

```text
raw experience
  -> EvoPhase-owned state change
  -> new prediction / program / experiment / plan
  -> changed physical action
  -> factual external result
  -> revision of the same EvoPhase-owned structure
```

Matched controls must preserve raw observations, primitive substrate, factual outcomes and resource limits while disabling only the mechanism under test.

## Current branch

Development is on **`main`**, following the owner's merge of `genesis/full-evophase`.

Historical capability measurements and failed packs are retained in [STATUS.md](STATUS.md) and [EXPERIMENT_LEDGER.md](EXPERIMENT_LEDGER.md). The [phase-execution audit result](docs/PHASE_EXECUTION_AUDIT_RESULT.md) qualifies their architectural interpretation; it does not erase them. The separate P1, P2 and P3 result records document bounded phase-native checkpoints without rewriting that history.

P3 closes the prepared-transition-curriculum gap for one deterministic family. The next architectural bottleneck is no longer “can it collect its own transitions?” but whether exploration itself can become **learned and history-dependent** rather than a hand-written unknown-action novelty rule, while retaining old knowledge across multiple changing worlds. G10 composite-concept construction remains a separate preregistered capability target.

Status: **research implementation; no AGI claim or production promotion**. CI is manual-only. Any future fresh qualification requires a newly frozen source/spec and a new first-attempt authority run.
