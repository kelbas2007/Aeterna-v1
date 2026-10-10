# CUE-ACCESS-1 — distinguish missing evidence from failure to use remembered evidence

2026-10-10. Completed OPEN diagnostic. **CUE_USE_NOT_DEMONSTRATED.**

[Completed GitHub run 38069741752](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38069741752), evaluator commit `5df0405820b13185dd0d52c55d01eb5107021cd1`. CI verified that `src/`, Cargo.toml, Cargo.lock and the external actuator bridge are unchanged from controller source `9e923f410b87214faa4f99ff237ef745248ac3b5`. This experiment changes measurement, NOT the agent architecture.

[Protocol](CUE_ACCESS1_PROTOCOL.md) was committed before this run. [Evaluator](../scripts/external_cue_access_audit.py) and [six scorer tests](../tests/test_cue_access_audit.py) are executable. Five existing native tests also passed (history alias 2, sequence replay 1, general policy 2). These are validity/component checks, not new cognitive success.

## Experimental control

Two existing native modes were tested: generic reactive policy, and the same general policy with developmental history, episodic recall and whole-sequence replay. Each had one continuing lifetime over 128 unmodified Farama MiniGrid v3.1.0 MemoryS7 training seeds, 130000..130127.

Evaluation used 32 new layouts/seeds, 131000..131031, with TWO cue variants for each layout. Only the initial cue object was exchanged (key/ball, same color), and the task's correct/incorrect terminal positions exchanged consistently. No route, position, initial-cue label, target, mission or audit field was given to Rust. Both phases continued to use only public categorical observation bits, actual motor decisions through U1/Human Protection, and actual simulator rewards.

The evaluator restored native cognition before every evaluation episode, froze learning, and checked that learned counters did not change. It enforced an input-packet field allowlist. Counterfactual cue changes used for visibility measurement were immediately restored, and those counterfactual images were NOT sent to the agent.

The public image was tested before EVERY decision: change that specific physical starting cue and regenerate the observation; if its bytes do not change, the cue is not informationally available at that moment. This avoids confusing a similar-looking key or ball at the fork with the actual initial cue.

Each pair has one upper-target and one lower-target instance. A constant-exit strategy therefore cannot exceed 32/64; target-side imbalance can no longer produce the old misleading 14/24 result.

## Actual measured results

| Measured quantity | Reactive policy | Episode-memory policy |
|---|---:|---:|
| Training episodes with cue in any decision input /128 | 41 | 42 |
| Training episodes with cue in initial input /128 | 40 | 40 |
| Initially unexposed episodes in which subsequent action acquired cue /88 | 1 | 2 |
| Actual training task successes /128 | 53 | 73 |
| Paired layouts in which cue ever appeared /32 | 10 | 10 |
| Paired layouts with no cue exposure at any decision /32 | 22 | 22 |
| Pairs with exactly identical input-image hash sequences /32 | 22 | 22 |
| Pairs with identical motor sequences under cue exchange /32 | 32 | 32 |
| Pairs in which selected terminal exit changed /32 | 0 | 0 |
| Pairs solved correctly in BOTH cue variants /32 | 0 | 0 |
| Actual rewarded heldout episodes /64 | 32 | 32 |
| Actual heldout motor executions | 356 | 356 |

Independent local recomputation of `score_pairs` from the downloaded JSON agreed with every summary field.

The reactive controller selected lower `(5,4)` in all 64 episodes. The episode-memory controller selected upper `(5,2)` in all 64 episodes. In the 10 exposed pairs, the cue appeared only in the initial decision frame; cue exchange did change that frame, but not any chosen action. In the other 22 pairs, the complete sequence of observation hashes was identical across the cue variants.

Example pair, seed 131000, episode-memory mode: initial position `(1,3)`, cue visible at step 0. Both initial-cue identities led to actions `[2,2,2,2,2,0,2]`. Example pair 131001: initial position `(3,3)`, cue never visible; both variants gave `[2,2,2,0,2]`. These coordinates and semantic motor interpretations are evaluator-only diagnostics, not controller inputs.

Frozen learned counters remained unchanged: reactive updates 889, rewards 53; memory updates 1230, rewards 73, rewarded memories 58, unsuccessful experience associations 367. Matching these counters does not by itself prove every internal tensor is immutable; existing native freeze tests provide additional, limited coverage.

## Corrected diagnosis

The previous claim that the behavior failure is solely a memory/relational-representation defect was too strong. This audit establishes TWO separate failures on this sample:

1. **Information acquisition:** in 22/32 pairs the executed policy never made the distinguishing initial cue available to its sensors. Along those identical sensory trajectories, no controller restricted to those inputs can infer which of the opposite exits is correct. This does NOT make the task unsolvable: a different information-seeking policy could turn or return and inspect the cue.
2. **Information use:** in 10/32 pairs, the cue did reach the real input, but replacing it did not change the action sequence. Here missing observation alone cannot explain failure. This audit does NOT yet localize whether the remaining defect is encoding, retention, relevance learning, retrieval or relational comparison.

Higher training reward (73 vs 53) did not establish cue use or heldout superiority. Both modes remained at the balanced constant-side bound, with zero pairs correctly solved on both cue identities. No new intelligence capability or child-like understanding is claimed.

## Requirement for the next architectural change

The single cognitive process must jointly represent what has been observed, what is inferred, and what is unknown; select an information-seeking action when task-relevant alternatives remain unresolved; retain the acquired distinction; and condition the later choice on the relevant relation. Implementing only a larger episode store or a hardcoded matching/key/turn rule does not meet this requirement.

Future evidence must separate these stages. Keep this balanced cue intervention as a non-negotiable control, add observed-cue retention/erasure interventions, and test the learned relation with unseen object encodings and reversed candidate locations. Do not claim a generic biological capacity from scalar labels such as 'continuity' or 'speech readiness'. No such new architecture has been implemented by this diagnostic-only commit.

## Reproducible evidence

Artifact: `cue-access1-paired-information-evidence`, GitHub artifact ID `11675894667`, containing `cue-access1.json` and `cue-access1.log`.

ZIP SHA-256: `86158ad106daf201ff24f2fa4606942c447180db1cab1fda06b936a45ebdedd3`.

Pinned runtime observed in this run: Python 3.11.17, Rust 1.99.0, minigrid 3.1.0, gymnasium 1.4.0, numpy 2.4.6. The workflow pins MiniGrid and Rust; its Python package transitive dependencies were not fully locked, so future cross-date reproduction should capture or pin these versions. No claim of broad statistical generality or independent raw-RGB object discovery follows from this one related-gridworld audit.
