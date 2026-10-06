# G7 — WHOLE-ORGANISM FRESH OPEN-WORLD QUALIFICATION

Status: **PRE-REGISTERED BEFORE G7 IMPLEMENTATION**

Date: 2026-10-06

## Question

Can one ordinary Aeterna-v1 `EvoPhase` organism combine the mechanisms already demonstrated separately — raw relational perception, acquired rival world models, a learned exploration strategy, acquired and factually revised child macros, and acquired hierarchical reuse — on a one-use fresh family of interactive worlds without evaluator task labels entering cognition?

G7 is a qualification gate, not a new AGI claim.

## Cognitive boundary

The evaluator may know the hidden world law only to generate factual sensor transitions and factual `Need`.

The organism receives only:
- raw 12x12 numeric rasters;
- opaque motor IDs;
- factual POST rasters after executed motors;
- factual `Need`;
- its own previously acquired EvoPhase state.

The evaluator must not provide:
- hidden-law IDs;
- semantic task or skill names;
- correct probe IDs;
- correct child-macro IDs;
- correct parent sequence;
- object/orientation labels;
- a host-side mapping from a surviving hypothesis to a solution.

All adaptive task-level state must remain inside one `EvoPhase` instance.

## Required end-to-end chain

A scored episode must causally traverse this chain:

```text
raw PRE raster
  -> acquired rival predictive models
  -> learned exploration readout chooses opaque probe
  -> factual probe POST reduces rivals
  -> factual POST itself becomes the high-level cue
  -> acquired parent macro selects acquired child IDs
  -> child macros execute their own learned bodies
  -> at least one child has previously undergone same-identity factual revision
  -> final factual Need
```

A host-side planner may execute the action selected by cognition, but may not choose the probe, parent sequence, child identity, or terminal motor for the organism.

## Pre-qualification acquisition

Before fresh scoring, each sub-seed constructs one organism from generic mechanisms only.

It must acquire:
1. at least three rival world hypotheses from factual transitions;
2. exploration weights from factual rival reduction, starting from zero;
3. at least two child macros from factual successful trajectories;
4. a factual revision of one child while preserving its macro ID;
5. at least two parent macros that reference acquired child IDs.

Learning is then frozen for the scored held-out episodes.

The evaluator may enumerate generic candidate experiences during tuition, but it may not tell cognition which candidate is correct.

## Fresh authority

The qualification obeys `docs/FRESH_QUALIFICATION_PROTOCOL.md`.

- exact source SHA is frozen before the run;
- exact G7 protocol/spec SHA is frozen before the run;
- GitHub Actions run ID is the external authority seed;
- 10 deterministic sub-seeds are derived from that authority seed;
- generated parameters and a world-pack digest are logged before scoring;
- the pack is burned after first observation;
- any source/spec change requires a new authority seed.

No developer-selected held-out seed is allowed.

## Pack size

Minimum scored pack:
- 10 sub-seeds;
- 8 scored held-out episodes per sub-seed;
- N = 80 total scored worlds.

Each sub-seed uses its own independently acquired organism state so opaque-label permutations are part of acquisition rather than patched after learning.

## Opaque permutations

Per sub-seed:
- permute all four motor IDs consistently across tuition and held-out worlds;
- vary child acquisition order so acquired child IDs are not stable semantic labels;
- do not expose the permutation to cognition.

Success must therefore depend on learned relations and references, not fixed action or child IDs.

## Open-world nuisance factors

The scored pack must include factors not guaranteed by the inherited translation-cancelling phase algebra.

Across each 8-world sub-seed block:
- all worlds use unseen absolute translations;
- at least 2 worlds add one irrelevant distractor pixel;
- at least 2 worlds remove one non-critical task pixel (partial occlusion);
- at least 2 worlds apply a 90-degree rotation to a cue whose semantic world role is preserved by the evaluator;
- at least 2 worlds change geometric spacing/scale relative to tuition.

Factors may overlap. The exact assignment is derived from the fresh sub-seed before scoring and logged.

These conditions are deliberately capable of producing FAIL. Known G1 stress weaknesses are not exempted.

## Matched controls and baselines

Evaluate the exact same generated world pack under:

1. **FULL_ORGANISM**
   - learned exploration ON;
   - acquired/revised child macros ON;
   - acquired hierarchy ON.

2. **ZERO_EXPLORATION**
   - identical world models/macros/hierarchy;
   - exploration strategy reset to zero before scored episodes.

3. **NO_HIERARCHY**
   - identical child macros and factual observations;
   - parent readout disabled;
   - generic child-sequence enumeration only.

4. **UNREVISED_CHILD**
   - matched pre-revision child state retained for the child whose law changed;
   - all other acquired state identical.

5. **RANDOM_PROBE_ORDER**
   - exhaustive or deterministic random-order diagnostic baseline for probe cost where practical.

Controls may remove a mechanism but may not receive extra evaluator hints.

## Metrics

Report for every arm:
- end-to-end factual task successes / N;
- Wilson 95% confidence interval;
- mean and standard deviation of physical probe count;
- mean primitive motor actions after identification;
- parent sequence candidate evaluations where applicable;
- tuition cost separately from held-out interaction cost;
- per-sub-seed success counts.

Also report:
- source SHA;
- protocol/spec SHA;
- authority seed;
- world-pack digest;
- learned exploration weights;
- number and IDs of acquired child/parent macros before scoring;
- revised child ID before/after revision to prove identity preservation.

## Acceptance

G7 PASS requires all:

1. N >= 80 and >=10 fresh sub-seeds.
2. All generated parameters/digest are logged before scoring.
3. FULL_ORGANISM traverses the required end-to-end chain on every success.
4. Learning is frozen during scored held-out episodes.
5. Motor permutation and child-ID permutation are tolerated consistently.
6. At least one previously revised child is causally invoked in scored held-out episodes.
7. FULL_ORGANISM success rate is >= 0.80.
8. Wilson 95% lower bound for FULL_ORGANISM success is >= 0.70.
9. FULL_ORGANISM is strictly better than ZERO_EXPLORATION in success or physical probe cost.
10. FULL_ORGANISM is strictly better than NO_HIERARCHY in success or candidate/primitive interaction cost.
11. FULL_ORGANISM is strictly better than UNREVISED_CHILD on worlds requiring the revised behavior.
12. No scored nuisance category has zero FULL_ORGANISM successes.
13. Mechanical ownership audit passes.
14. G0-G6 regressions pass.
15. Release build passes.

A compile/test failure is technical, not a cognitive FAIL. A completed scored run that misses any cognitive threshold is recorded as FAIL and the pack is burned.

## Interpretation boundary

A PASS would establish bounded whole-organism composition across a fresh mixed family with explicit nuisance factors. It would still not establish:
- unrestricted object formation;
- arbitrary recursive program induction;
- general planning;
- natural-language competence;
- ARC competence;
- AGI or consciousness.

A FAIL is expected to identify the next architectural bottleneck and must be preserved in the experiment ledger.
