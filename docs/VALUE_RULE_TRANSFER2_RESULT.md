# VALUE-RULE-TRANSFER-2 — navigation FAIL, recorded-data improvement

Frozen source `f934442ffeae897fa1d8914a8c262506235949ff`, executable SHA-256
`1b4df205b67dfae6ba6ac399c5b6d2722f5e35c92386d6cb4daf967288a67800`.
[Protocol](VALUE_RULE_TRANSFER2_PROTOCOL.md). The same executable ran all tasks.

| Frozen navigation from a MemoryS7-trained life | Successes /64 |
|---|---:|
| Acquired representations and effect models | **0** |
| Same knowledge, physical rule-access lesion | 15 |
| Cold native controller | 15 |
| Random | 9 |
| Explicit authored turn-at-wall controller | 64 |

MemoryS7 retained 64/64, 32/32 both-correct pairs. MemoryS9 scored 32/64,
**0/32 both-correct**; its lesion scored 25/64, 5/32 both-correct. Both navigation
and longer-memory transfer remain FAIL. All persistent counters stayed frozen.
The training sample produced 360/512 successful lives.

| Recorded data, separately trained lives | Rules | Lesion | Matched 3-NN |
|---|---:|---:|---:|
| Real digit images /364 | **353** | 48 | 353 |
| Real GunPoint signals /150 | **127** | 72 | 128 |

Both declared recorded-data gates pass. These same public held records were
already used in the first failed iteration: this is iterative validation,
not independent qualification. Images match the simple reference; signals
remain one example below it. Image and signal training use separate lives.
The learner chose width 5 for digits (320 cases, internal 61/64) and width 7
for signals (50 cases, 9/10), despite the latter's true serialization width 6.
Actual training costs were 16,489 and 1,108 motor actions respectively.
This demonstrates useful selection, not invention of new primitive operations.

Replaying a consumed navigation trajectory exposed the next defect: the agent
reached an unfamiliar goal appearance and turned away. A sparse effect rule
assigned certainty to a joint sensor tuple absent from its learning evidence.
That motivates a separate amendment and fresh navigation sample; this result
must not be relabeled after that amendment.

Raw records: [navigation/memory](evidence/value-rule-transfer2.json.gz),
[digits](evidence/value-rule-digits2.json.gz),
[signals](evidence/value-rule-gunpoint2.json.gz),
[checksums](evidence/value-rule2-summary.json).
