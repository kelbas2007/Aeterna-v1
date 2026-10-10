# VALUE-RULE-TRANSFER-1 — FAIL preserved

Source `dcd3fa5a0598a5464b5bb1d92a40c02e78990200`, frozen executable SHA-256
`e41d505edba8a1e3e84fe0297077a2b4fcdb2ab0844156936f268b2a1a60c192`.
[Protocol](VALUE_RULE_TRANSFER1_PROTOCOL.md). This negative first evaluation
is retained before further development; later results cannot replace it.

| Frozen navigation from a MemoryS7-trained life | Successes /64 |
|---|---:|
| Acquired predicate rules | **8** |
| Same knowledge, rule-access lesion | 20 |
| Cold native controller | 20 |
| Random | 14 |
| Explicit authored turn-at-wall controller | 64 |

Rules scored 8/32 at Empty8 and 0/32 at Empty16. Initial positions/directions
were randomized by Farama. The preregistered >=48/64 and per-size/delta gates
failed. MemoryS7 retained 64/64; MemoryS9 scored 26/64 and **0/32** both-correct
pairs. Longer-memory transfer remains FAIL. Counters stayed frozen.

| Recorded data, separately trained lives | Rules | Lesion | Matched 3-NN |
|---|---:|---:|---:|
| Real digit images /364 | **200** | 48 | 353 |
| Real GunPoint signals /150 | **104** | 72 | 128 |

Both recorded-data accuracy gates failed (75% digits, 80% signals).
The nearest-neighbor reference uses exactly the measurements received by the
native learner and only labels identified by rewarded training attempts.
These are consumed public fixtures, not independent qualification.

Raw records: [navigation/memory](evidence/value-rule-transfer1.json.gz),
[digits](evidence/value-rule-digits1.json.gz),
[signals](evidence/value-rule-gunpoint1.json.gz),
[checksums](evidence/value-rule1-summary.json).

Diagnosis motivating the next iteration: exact addresses overfit; sparse
predicate leaves also lose useful measured similarity. Reward-only decisions
failed to reuse the physical consequences of motors. This diagnosis is not
itself proof that a replacement mechanism succeeds.
