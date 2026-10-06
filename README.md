# AETERNA v1

A clean no-LLM intelligence research line targeting **full EvoPhase cognitive ownership**.

## Current architectural qualification

**Full physical phase-native execution is NOT yet established.** The recorded task results in `STATUS.md` remain evidence, but they must not be confused with proof that the phase-cell/synapse network itself executes every cognitive operation.

The 2026-10-06 [phase-execution audit](docs/PHASE_EXECUTION_AUDIT_RESULT.md) reproduced a specific gap: the legacy G8 planning path gave identical decisions in 18/18 deterministic matched cases with phase plasticity and structural recruitment disabled and no dormant cells. Its learned transition table remained present; removing that table removed the plan. A standalone version of the same production planner also worked without constructing an EvoPhase instance.

This is a negative architectural witness, not an all-learning ablation and not an invalidation of every earlier result. The original audit and its tests are preserved. The strong ownership requirement below remains unchanged. G10 is preregistered; neither the audit nor P1 implements or qualifies it.

### P1: acquired shared-synapse execution now tested

A new explicit native mode routes ordinary planning APIs through learned connections in the **same EvoPhase cell and synapse arrays**, with no legacy graph fallback. Tested source: `f33028a5d2809fd7500d46a2021b35f230b1e704`; workflow `37522194429`.

The preregistered P1 run passed 24 deterministic intervention cases and 80/80 freshly sampled bounded task instances. Cutting a necessary internal successor connection gave 0/80; shifting its learned phase offset by pi also gave 0/80; exact restoration recovered 80/80 without relearning. Same-relay/same-synapse factual outcome devaluation changed the decision in 80/80 cases. Separate zero-phase-learning, zero-weight-learning, no-capacity and no-growth acquisition controls each completed 0/80 tasks. The independent graph reference also completed 80/80.

This is **phase-dependent local value propagation**, not forward generated sensory imagination or a complete neural migration of all cognition. The recurrence and winner-take-all competition are inherited; P1 inference uses learned phase calibration rather than evolving oscillator trajectories. Primary tuition averaged 854 factual transition presentations per arm and instance. No data-efficiency, competitive-performance or general-intelligence claim follows.

See the [P1 protocol](docs/PHASE_NATIVE_P1_PROTOCOL.md) and [verified P1 result, costs, evidence and limits](docs/PHASE_NATIVE_P1_RESULT.md). The tested pack is burned. Old G8/G9 results are not relabeled as phase-native results.

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

Historical capability measurements and failed packs are retained in [STATUS.md](STATUS.md) and [EXPERIMENT_LEDGER.md](EXPERIMENT_LEDGER.md). The [phase-execution audit result](docs/PHASE_EXECUTION_AUDIT_RESULT.md) qualifies their architectural interpretation; it does not erase them. The separate [P1 result record](docs/PHASE_NATIVE_P1_RESULT.md) records the subsequent shared-synapse execution checkpoint without rewriting that history.

Next architectural requirement: forward, phase-dependent prediction and continuation through the acquired carrier, followed by ordinary whole-organism validation. P1 establishes a bounded necessary physical connection/phase path for value-based decisions, not completion of this requirement. The existing graph planner remains a reference implementation.

Status: **research implementation; no AGI claim or production promotion**. CI is manual-only. Fresh P1 qualification is disabled by default and requires explicit selection of a new first-attempt workflow run.
