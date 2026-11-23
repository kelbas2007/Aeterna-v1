# Integral test repeated with retained knowledge — frozen protocol

## Question

What happens when the same AETERNA organism repeats the complete original
integral scenario while retaining its full state from the first passage?

This is a requested developmental repetition of already exposed worlds, not a
new independent qualification. The original run `45231338992`, source
`4b391bf81f1f8203fb3db24e6e1a8fd48f24fc13`, remains **FAIL: 4/8 complete chains**.
Neither its results nor its thresholds are revised. Its full evidence was
preserved at `c889d72c55aa1be586fa01e387f56ddca882b4ff`.

Production remains the unchanged source from main
`01a35d0c69c3dec0340c753456ada4a7b7f221da`. The original evaluator, scorer,
protocol and evidence archive stay unchanged. New work is confined to the
retained-state evaluator, runner, this protocol, workflow and result artifacts
on `experiment/integral-retained-knowledge-20261123`.

## Why reconstruction is required

The first artifact saved complete prospective/factual traces and summaries,
but not the checkpoint bytes themselves. Its checkpoints existed inside the
first process and were discarded when that process exited. We therefore cannot
claim to load an archived historical checkpoint that does not exist.

The new evaluator reconstructs that passage through actual autonomous runtime
decisions using the pinned old generator, master seed `45231338992`, world
indices 0–7, configurations, phase order, offers, observations and action limits.
It does not teach the actor from archived actions, force a recorded motor, or
feed the actor a trace. The archived data are only an independent verifier of
the completed reconstruction.

All eight reconstructed full JSONL files must have exactly the original SHA256,
and all original-format JSON summaries must match. **No actor may begin the
second passage until all eight reconstructions pass this global barrier.** A
mismatch makes continuation INVALID and prevents warm execution. Reconstruction
is not counted as additional evidence of generalization or improvement.

The archive SHA256 is
`73c4a3f6d1bb27d62c0fdf963479dbe45b489493ebf87ec5022d63fb3796c21e`.

## State boundary and persistence

During reconstruction, save the complete live checkpoint immediately after
the original 512 adaptive episodes and immediately before the original call
to `freeze_constructing`. That checkpoint retains learning capability, causal
counts and hypothesis weights, native learned weights, VM/organism state,
decision progression, journal, protection/cost position and cumulative events.
No part of that state is reset or selectively copied into a new cold organism.

The checkpoint API is read-only. Original validation and controls still execute
in the original order, and their observations do not train the continuing state.
For each checkpoint, require exact serialization/restoration equality. On
disposable copies, require that freezing the saved live state yields the same
frozen checkpoint as the reconstructed original path.

An ordinary episode restart finalizes pending learning credit. Verify its
boundary handling by comparing, on disposable copies of the saved live state:

1. freeze, then ordinary restart at the start observation;
2. ordinary restart at that same observation, then freeze.

The resulting checkpoint bytes must be identical. The real second passage is
restored from the original live checkpoint and receives only its ordinary first
episode restart; it does not receive these verification clone operations.

Persist initial, acquired, live continuation, revised frozen and stale frozen
checkpoints for both reconstructed and repeated passages, alongside metadata
and file hashes. Later continuations can load these actual persisted files.
The missing historical checkpoint digest remains a stated provenance limitation:
complete historical trace/summary agreement verifies reconstruction, not a
comparison against unavailable historical checkpoint bytes.

## Exact repeated scenario

Repeat the entire original 0→1 protocol with the original world identities and
the same hidden/noise schedules. Master seed is still `45231338992`. The new
GitHub Actions run ID is execution provenance only, not a new world seed.

| Phase | Episodes per world | Rule | Learning |
|---|---:|---:|---|
| `acquire` (now reacquisition) | 256 | 0 | Enabled from retained live state |
| `original_held` | 64 | 0 | Frozen clone |
| `adaptive` | 512 | 1 | Enabled |
| `frozen` | 512 | 1 | Frozen from the same reacquired checkpoint |
| `recovered_held` | 64 | 1 | Frozen revised checkpoint |
| `shuffled_cue_held` | 64 | 1 | Same checkpoint, shuffled useful cue |
| `stale_held` | 64 | 1 | Frozen continuation checkpoint |
| `random_held` | 64 | 1 | Original independent random policy |

The first passage ends with rule 1. The repeated scenario starts with rule 0,
so its beginning is another unannounced inversion. The first 256 episodes are
reacquisition after that inversion, not simply more exposure to the final rule.
Early worse performance could reveal interference from retained experience.
We do not presume that prior knowledge must improve every phase.

All four opaque actions remain equally available at unit cost. Each episode
still has at most four actions, and all non-random choices remain runtime-owned.
The same single goal, ordinary episode restarts, physical callback, planning
budget, history depth and memory capacity apply. There is no research-action
schedule, teacher, newly installed solver, expanded memory or threshold tuning.

## Evaluation

Reuse the original frozen scorer and all its capability thresholds on the
second passage, adding reconstruction/checkpoint integrity as a prerequisite.
In particular: aggregate original/recovered held accuracy each at least 80%;
at least six complete chains; original contradiction probability at least .75
with support at least 3; corrected old-action failure probability at least .60
and an increase of at least .25 in at least three held episodes; and recovered
accuracy at least 20 percentage points above each matched held control.

The second passage has a separate PASS, FAIL or INVALID verdict. A PASS cannot
be assembled by pooling favorable worlds from both passages. A reconstruction
or checkpoint failure gives INVALID, never a capability PASS. Every world and
every failed prerequisite stays in its original denominator.

Report first/second results side by side for every phase, together with action
costs, informative/noise queries, timeouts, per-world failed gates and exact
forecast-correction witnesses. Report fixed first/last 64-episode windows
within both learning phases. Changes are descriptive quantities; no improvised
improvement threshold is introduced after observing the second passage.

Also report a descriptive cross-passage comparison: take the originally
supported contradicted forecasts from the first passage and look up those exact
PRE/history/old-action contexts in the second passage's recovered held traces.
Report the new probabilities and counts, including whether the same .60/.25/
three-episode correction condition is met. This uses no new physical experience
and does not replace the separate strict verdict of either passage or pool
worlds into a qualification.

A lower number of confident-error witnesses can reflect less overconfident
predictions, so a failed chain must be explained by its actual missing stage.
Do not equate every strict FAIL with worse goal behavior or absence of learning.

## Scope and execution

This compares complete retained agent state and prior experience, not causal
memory in isolation. The schedules have been seen before, and all eight worlds
share one binary sensor/actuator task structure. There is no claim of fresh-world
qualification, broad scientific discovery, or a newly tested wave/helix mechanism.
The user's separate DNA analogy is not inserted into this runtime experiment.

Freeze source/protocol before any behavioral execution. Use Rust 1.99.0 and
locked dependencies, preserve first-attempt evidence and do not tune settings
or rerun for a preferred verdict.

```sh
cargo build --locked --release --example integral_retained_knowledge_eval
python3 -u scripts/run_integral_retained_knowledge.py \
  --binary target/release/examples/integral_retained_knowledge_eval \
  --run-id "$GITHUB_RUN_ID" --workers 2 --output retained-evidence
```

Re-scoring persisted trajectories is permitted; it must not execute an actor.
All checkpoints, traces, failures, integrity diagnostics and result summaries
are retained in the output archive. No production merge or repair is included.
