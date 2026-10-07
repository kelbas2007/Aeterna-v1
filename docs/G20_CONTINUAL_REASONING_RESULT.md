# G20 — protected continual reasoning: integration result

Date: 2026-10-07
Verdict: **MECHANISM / INTEGRATION PASS**. This is not a fresh statistical qualification.

## Executed evidence

Workflow: 37633962959 (`g20-lifetime`)
Job: 112835171970
Exact tested source: `30dfd5a66b7dc8e4f68555b0bf72429c6e300dc9`
Completed: 2026-10-07T14:11:37Z
Protocol: `docs/G20_CONTINUAL_REASONING_PROTOCOL.md`, committed before implementation.
Evidence artifact: `g20-lifetime-evidence`, ID 11486659950, retention 30 days.
Artifact archive SHA256: `760d80f3b1130a784625f992791358a5f743019dc4e4b317575a368e1a6bb944`.

The job passed all three new G20 tests, all five Human Protection tests, the full ordinary regression suite, the Release build, and the checks that tracked source was unchanged. Existing unused-code warnings remain; the build is not claimed warning-free.

No fresh qualification was requested or consumed. Historical fresh tests remained ignored or explicitly skipped. The G20 fixture imports the existing G19 test file; the extra historical tests executed in the full regression suite are not independent G20 trials.

## What was integrated

`src/scientific_runtime.rs` provides `ScientificRuntime`, a non-Clone runtime holding one evolving native EvoPhase, a separate Human Protection gate, the current raw goal, and a bounded operational audit.

Its action path is:

raw current state and goal -> native rival-discrimination proposal, otherwise native goal-directed active action/planning -> external hazard assessment -> screening -> one-use permit consumption -> actual environment callback -> factual raw POST -> native model revision -> next proposal.

The native learned model continues across task boundaries. No task-specific route solver, evaluator goal ID or answer mapping is added to the runtime. The arbitration between existing native mechanisms is programmer-defined orchestration; this result does not prove that every cognitive operation is emergent oscillator computation.

## Lifetime result

There were two deterministic lifetime variants covering two opaque motor permutations. Within each FULL lifetime the same acquired organism completed eight alternating goal tasks. The environment moved only through actual action callbacks; there was no teleport or reset to the start between tasks.

Both FULL lifetimes completed **8/8**, for **16/16 actual goal arrivals** in total.

Both had the same per-task executed-action counts:

`[2, 3, 3, 3, 6, 3, 4, 3]`

This sums to 27 executed actions per FULL lifetime. The count excludes initial representation/model tuition and blocked safety proposals. It is not an end-to-end cold-learning sample-efficiency figure.

After the first four goals, the evaluator changed the previously useful A-shortcut consequence to a dead state. It supplied no change flag or repaired model to cognition. On the next A task the FULL organism encountered that consequence, revised the acquired transition and reached A through its longer known fallback in **6 actions**, including travel and the failed shortcut attempt.

The unchanged B task then still completed in **3 actions**. Following the ordinary checkpoint restart after six goals, the next A task completed in **4 actions** and B again in **3**. The repaired knowledge and the useful unchanged knowledge remained operational.

The matched FROZEN_MODEL versions completed their first four tasks, but each failed the first changed-A task after the full **16-action** budget. Thus the changed-task success was **2/2 FULL versus 0/2 FROZEN**, not an evaluator-provided alternative plan.

All six opaque motor IDs were exercised. Physical transition addresses remained stable; the contradicted A-successor link remained structurally present with revision/counterexample provenance and zero executable weight. The transferred drive weights were unchanged, dedicated table composites remained absent, and legacy graph transition count remained zero.

## Exact witness lines from the completed log

```text
G20_LIFETIME swap=false full=LifetimeResult { completed: 8, costs: [2, 3, 3, 3, 6, 3, 4, 3], changed_revision: true, motor_mask: 63 } frozen=LifetimeResult { completed: 4, costs: [2, 3, 3, 3, 16], changed_revision: false, motor_mask: 63 }
G20_LIFETIME swap=true full=LifetimeResult { completed: 8, costs: [2, 3, 3, 3, 6, 3, 4, 3], changed_revision: true, motor_mask: 63 } frozen=LifetimeResult { completed: 4, costs: [2, 3, 3, 3, 16], changed_revision: false, motor_mask: 63 }
G20_RESULT actual_goals=16/16 motor_mask=0b111111 safety=PASS restart=PASS frozen_changed=0/2
```

These values appeared in the targeted integration run and again in the ordinary regression run; the repetition is not counted as additional independent evidence.

## Human Protection in the operational loop

The integration tests verified:

- high-risk evidence blocks execution;
- missing hazard evidence blocks execution;
- emergency stop blocks and latches;
- each blocked attempt invokes the action callback zero times and leaves the learned fingerprint and factual sensory/Need/tick unchanged;
- cognitive checkpoint restart preserves the outer safety latch, goal and monotonically increasing lifetime event sequence;
- fresh sensing after that restart does not clear the emergency latch;
- only an explicit external operator reset permits resumption;
- every allowed callback is preceded by screening and consumption of its single-use permit;
- an execution callback returning an I/O error latches protection, records no fabricated factual result, and prevents a blind retry;
- invalid goals and non-finite observations are rejected without model learning.

The runtime exposes read-only organism diagnostics, not a mutable carrier/protection handle. It does not provide cognition with a permit.

## What is NOT established

The initial representation and prior alternative transitions still come from inherited factual tuition. The sixteen arrivals are repeated tasks involving two goals in one small nine-state family, not sixteen independent domains. This stage did not induce new abstractions or invent new hypotheses during the lifetime.

Cognitive restart uses an in-memory native checkpoint. It is not durable disk persistence, power-loss recovery or process-level crash recovery. The audit ring is bounded to 256 operational events and is not a durable security log.

The gate protects the callback owned by this runtime; it is not a sandbox against a privileged host with separate device access. Hazard numbers are supplied externally, not measured by a person detector or calibrated risk estimator. Existing numeric thresholds are fixture settings, not approved safety limits. Hardware isolation, authenticated operator access, emergency relays, deadlines and panic/abort containment remain deployment work.

The next evidence gap is broader persistent integration: new representational distinctions and hypotheses must be acquired within the same lifetime, with independently sampled tasks, incomplete observations and appropriately tested uncertainty handling. Those capabilities are not claimed by this result.

## Reproduce the ordinary integration check

```sh
cargo test --release --test g20_continual_reasoning g20_ -- --nocapture
cargo test --release --test human_protection -- --nocapture
```

See `docs/G20_RUNTIME_USAGE.md` for the runtime API and execution-boundary limitations.
