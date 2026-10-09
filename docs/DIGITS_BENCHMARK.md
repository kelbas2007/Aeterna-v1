# Recorded handwritten digits — open development result

The [protocol](VECTOR_PROTOCOL.md), defaults and 80% per-variant capability
criterion were written before scoring. This is open development on a familiar
public corpus, not a sealed scientific qualification.

## Data and factual interaction

The [fixture](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/tests/data/README.md)
contains 1797 recorded 8×8 handwritten digit images. The fixed within-label
index-modulo-five split gives **1433 training and 364 held-out images**. The
held-out records never update acquired models. Writer identities are unavailable;
no writer-independent or general photographic-vision claim follows.

One persistent cold organism learns from full training frames. It chooses an
opaque motor and receives its actual external outcome: 1 for a correct response,
0 otherwise. Each record permits at most eleven actions until success. Thus
this is corrective supervised feedback, although EvoPhase never receives the
digit annotation or a supplied correct action. The application/evaluator owns
the annotations and motor semantics.

Frozen testing starts with all 64 features missing. An acquired measurement
action reveals the image; at most one terminal response follows. The total
budget is two actions per image. A factual positive outcome alone satisfies the
goal. Unsupported predictions abstain. The learned fingerprint stays unchanged.

Two software controls receive the same actual acquired tuples: class centroids
and Euclidean 3-nearest-neighbor voting with 32 retained positive examples per
response. They get the same measurement/response budget. Experience is matched;
this does not compare independent acquisition policies or prove an advantage
over optimized image classifiers.

## Measured results

| Motor/pixel permutation | Actual training actions | EvoPhase | Centroid | Bounded 3-NN | Phase abstentions | Measurement first | Phase test actions | Checkpoint bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 2294 | 343/364 | 328/364 | 335/364 | 7 | 364/364 | 721 | 2623049 |
| 1 | 2330 | 343/364 | 328/364 | 335/364 | 7 | 364/364 | 721 | 2623059 |
| 2 | 2318 | 343/364 | 328/364 | 335/364 | 7 | 364/364 | 721 | 2623055 |

EvoPhase accuracy is **94.23%**, centroid 90.11%, bounded 3-NN 92.03%. Phase
errors per variant are fourteen wrong commitments and seven abstentions.
Centroid used 728 actions; 3-NN used 714. All variants satisfy the fixed 80%
criterion and two-action budget. Every model occupies 320 of 352 allocated
prototype slots; the topology does not grow during tuition.

The three variants reuse the **same 364 held-out images** with different motor
and pixel mappings. They are 1092 episodes, not 1092 independent test images.
No held-out-driven default tuning was performed. After the first score, an
engineering correction made exact-distance ties prefer the stronger physical
response; the factual-correction test required this. The repeated benchmark
retained every first-pass score, action count and checkpoint size above.

## Physical and persistence evidence

Learned centers are actual feature-to-prototype phase offsets; learned response
strengths are actual prototype-to-motor weights. There is no separate numeric
prototype backup. Algorithmic distance, quantization and voting remain inherited
software; this does not qualify full phase ownership of cognition.

The small mechanism control removes the actual feature and response links and
loses predictions; exact restoration returns the original prediction. Changing
feature-link phases by π changes predictions, and restoration recovers them.
These are engineering controls on a small acquired fixture, not corpus-wide
ablations. A changed factual annotation weakens the old response and learns the
new one. An additional 512 feedback records retain fixed cells, links and bounded
provenance. Broad continual retention on real image streams is still untested.

The benchmark serializes and restores before frozen testing. The CLI integration
test separately trains, loads actual PNG/JPEG, handles light-background inversion,
corrects with external feedback into a new checkpoint, and rejects overwrite
and malformed input. See [the runnable guide](VECTOR_PERCEPTION.md).

```bash
cargo test --locked --release --test real_digits -- --nocapture
cargo run --locked --release --example real_digits
```

The example also prints the complete 10×11 confusion matrix for each variant;
column ten represents abstention. Existing FRONTIER-1, TE1 and TE4 scientific
FAIL records remain unchanged.
