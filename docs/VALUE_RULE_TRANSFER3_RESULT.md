# VALUE-RULE-TRANSFER-3 — acquired motor skill transfers; longer memory FAIL

**CROSS_TASK_DEVELOPMENT_PASS**, first evaluation of this frozen amendment on
the new held navigation seeds. [Preregistered protocol](VALUE_RULE_TRANSFER3_PROTOCOL.md).
Source `5a929189253662b9299aba504c4ff93697c50dde`, executable SHA-256
`eabc479fd1e355c3df9c4c7d42f4eb67d3f0611ddd54398e7c9135f6a1301fcf`.
One source and executable ran navigation, images and signals.

## Frozen transfer from another task

A cold life learned only from its own actions in 512 MemoryS7 episodes. All
512 factual training trajectories exactly match iteration 2; the improvement
comes from the amended frozen-use mechanism, not different training facts.
No target navigation demonstrations, maps, goal labels or motor meanings were
supplied. Acquired parameters were frozen; actual failed probes could revise
only bounded episode-local beliefs. Navigation had random starts and a
200-action cap in each arm.

| Navigation arm | Successes /64 |
|---|---:|
| Acquired effect models with support-aware uncertainty | **60** |
| Same learned life, physical rule-access lesion | 12 |
| Cold native controller | 12 |
| Random actions | 14 |
| Explicit authored turn-at-wall controller | 64 |

Rules scored **30/32 at Empty8 and 30/32 at Empty16**. The original >=48/64,
per-size >=20 and >=16 control-delta gates all pass. All 2,400 observations
of these rule-arm navigation episodes were absent from its training input
hashes; these included 79 distinct views. There were 59 distinct
world/start/direction combinations in 64 runs;
random starts need not be unique. Mean cost was 37.5 actions, including four
failures at the action cap. This is evidence of acquired local motor-law reuse
under new layouts, not new physics, general reasoning or superiority to the
authored controller.

S7 retention remained **64/64**, 32/32 both-correct original/swapped pairs;
its rule-access lesion also scored 64/64 because exact learned states retain
their original policy. The abstraction access is causal for new-state transfer.

**MemoryS9 remains FAIL: 32/64, 0/32 both-correct pairs**. Only 5/32 pairs
exposed the cue; all pairs chose identical action sequences. The lesion arm
scored 21/64 and 2/32 both-correct. This does not establish transferred memory
use or cue-conditioned reasoning in a longer layout. No numerical navigation
success can replace that separate failed criterion.

## Repeated real recorded data

| Separately trained lives | Acquired representation | Rule-access lesion | Matched 3-NN |
|---|---:|---:|---:|
| Real digit images /364 | **353 (96.98%)** | 48 | 353 |
| Real GunPoint signals /150 | **127 (84.67%)** | 72 | 128 |

Both predeclared recorded-data gates pass. Every training and held action
record matches iteration 2, as expected from the unchanged numerical inference.
These public test records were already scored in two iterations; they are
**iterative validation, not independent qualification**. The same constructor
trains separately; it does not transfer image knowledge to signals. The 3-NN
reference receives the same quantized measurements and only training labels
identified by the agent's actual rewarded attempts, with one held action per
record for both. There is no measured advantage over that simple reference.

Digits acquired width 5, 320 cases, internal 61/64; signals width 7, 50 cases,
9/10, despite true signal serialization width 6. The learner chooses a useful
representation, not necessarily the source format. Training consumed 16,489
and 1,108 factual motor actions respectively. Labels remain in the evaluator;
the Rust controller receives measurements, selects its own motor, then receives
the actual success reward.

## Provenance and limits

All checked persistent counters and acquired model metadata stayed unchanged
during held episodes, including after native restart. Native causal tests also
check unchanged learned fingerprints, opaque motor permutations, memory and
access lesions, unsupported tuple probing and transient-only revision.

The constructor, CART, case comparison, candidate grouping, action-effect
statistics and uncertainty procedure remain authored Rust software. Physical
access dependence does not prove full EvoPhase ownership or self-invented
primitives. The new head has bounded storage and in-process native checkpoints;
Online v9 disk persistence does not yet serialize it. Navigation is simulated
categorical input, while recorded pixels/signals come from real measurements.
Open-world intelligence and live robot adaptation are not established.

The raw diagnostic exports the acquired policy tree and operator/model counters,
not the case table, effect-tree parameters or joint support table. Their
reproduction requires repeating the frozen factual training. Raw action
provenance is therefore not a complete serialized learned-model artifact.

Prior navigation failures are retained: [iteration 1](VALUE_RULE_TRANSFER1_RESULT.md),
[iteration 2](VALUE_RULE_TRANSFER2_RESULT.md). The amendment was developed from
consumed earlier trajectories, then frozen before new held navigation scoring.

Raw evidence: [navigation/memory](evidence/value-rule-transfer3.json.gz),
[images](evidence/value-rule-digits3.json.gz),
[signals](evidence/value-rule-gunpoint3.json.gz),
[checksums and summaries](evidence/value-rule3-summary.json).
[Usage and capacities](ACQUIRED_VALUE_REPRESENTATIONS.md).
