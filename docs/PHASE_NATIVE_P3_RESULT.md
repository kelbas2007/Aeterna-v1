# P3 deterministic preflight result — autonomous acquisition

Date: 2026-10-06

Status: **MECHANISM PASS / FRESH-P3 PASS**

Protocol was committed before P3 implementation as `docs/PHASE_NATIVE_P3_PROTOCOL.md` at commit `47d325790068e4b43a9f242a71c57b26ef807b91`.

Verified workflow:
- run: `37528040155`
- tested source: `b2dade26db1a7d83a5c16f05fe2c70f39d27a33c`
- full optimized regression suite: PASS
- Release build: PASS
- one-use P1/P2 qualification packs: not consumed

## Mechanism outcome

Across 12 deterministic hidden reset-chain worlds:
- FULL cold autonomous acquisition: **12/12**
- DIRECT_ONLY exploration: **0/12**
- seeded RANDOM_ACTION baseline: **4/12**
- mean physical interactions to first reward: **24.667**
- mean additional changed-law repair interactions: **9.083**

The evaluator supplied no transition tuple and no correct action to FULL. Every acquisition transition entered P2 only after the opaque motor selected by `choose_phase_native_autonomous_action` was physically executed by the test-side world.

The production selector:
- assigns intrinsic value to actions not yet modelled at the current acquired receptor;
- removes that local novelty after factual transition acquisition;
- propagates deeper receptor novelty backward over acquired phase-native successor synapses;
- reuses known transitions to return to deeper unexplored frontiers;
- falls back to ordinary P1 reward propagation only when no reachable frontier remains.

The reset-chain family makes this causal: wrong actions return to the start. A selector restricted to current-state novelty cannot deliberately return to the deeper frontier after start actions become known; its matched result was 0/12.

## Goal use

After autonomous first reward:
- learning was frozen;
- the organism solved from an unseen absolute raster translation;
- ordinary P1/P2 action/prediction APIs were used;
- legacy graph transition count remained zero.

All 12 acquired worlds passed this frozen exploitation check.

## Restart persistence

P3 adds an opaque `PhaseNativeCheckpoint`:
- learned cells/phases and synapses are retained;
- phase-native receptors/circuits are retained;
- transient REAL charge and current observation are not restored.

A newly constructed EvoPhase restored each checkpoint and solved the held-out translated task with learning frozen. This is not a plain `EvoPhase::clone()` witness.

## Factual changed-law repair

After initial acquisition, one formerly valid advancing transition was changed to lead into a previously unseen detour state.

The acquired organism:
- first carried its old model into the changed environment;
- received only factual POST/value;
- acquired the changed successor;
- explored the unseen detour autonomously;
- found the reconnecting opaque action;
- regained the factual goal within the preregistered 40-interaction budget.

All 12 deterministic worlds passed. Mean additional interactions: 9.083.

A frozen stale copy could not acquire the detour solution. An unaffected old transition remained a valid factual successor after local revision. Revised capability also survived checkpoint restore.

## Controls

Per the deterministic preflight, NO_LEARNING, NO_STRUCTURAL_GROWTH, ZERO_PHASE_LEARNING and ZERO_WEIGHT_LEARNING were required not to match FULL on each tested world. The completed run satisfied those assertions.

The production source guard also passed: the autonomous selector contains no `EvoImaginationPlanner`, learned transition table, frontier node, binary heap, queue-pop route search, or equivalent host graph search. It directly uses `self.cells`, `self.synapses`, successor synapses, conductance and mode-isolated intrinsic membrane activity.

## Preserved technical failures

Two earlier runs remain part of the evidence history:

1. `37527093837` at `9f2ab1b6c98544d6003ea4c959c372cbd0d04739` — TECHNICAL_FAIL_TRANSIENT_FINGERPRINT. The checkpoint test exposed that the old learned fingerprint accidentally included transient REAL charge. The fingerprint was corrected to exclude transient charge; the autonomy mechanism and thresholds were unchanged.

2. `37527565396` at `bc955c4db07fd2ba6cb1fbd7cfe2b52648d8d933` — TECHNICAL_FAIL_OVERSTRICT_RETENTION_OBSERVER. Autonomous acquisition, restart and changed-law repair had already executed; the final observer demanded bit-level decoder identity (<1e-6) and saw mean drift 3.4711426e-5. The preregistered contract requires retained factual capability, so the observer was changed to verify the correct factual successor. Production cognition, budgets and PASS thresholds were unchanged.

Neither failure is rewritten as a PASS.

## What this establishes

P3 closes one concrete gap left by P2: transition experience no longer has to be supplied as a prepared curriculum in the tested family.

The organism can now execute this bounded cycle:

```text
cold raw observation
 -> self-selected opaque physical action
 -> factual POST/value
 -> acquired phase-native transition
 -> intrinsic frontier propagation through acquired synapses
 -> deliberate return to deeper unknown state
 -> first factual goal
 -> frozen phase-native exploitation
 -> checkpoint/restart
 -> factual surprise after world change
 -> autonomous detour acquisition
 -> repaired goal behavior
```

## What remains open

This is still a small deterministic family with a hand-specified intrinsic novelty rule. P3 does not yet show:
- learned intrinsic motivation;
- stochastic or partially observed world modelling;
- autonomous invention of new representational dimensions;
- long-horizon continual learning without interference;
- arbitrary goals or domains;
- AGI.

## FRESH-P3 one-use statistical qualification

The first authority pack was opened only after source and fresh specification freeze.

- workflow run: `37528857872`;
- source SHA: `52bbbc2654894bbaf8501e6834757dff25f1e242`;
- fresh specification blob SHA: `b561a3af878c89531ed8df27b338959fd4d868f5`;
- authority seed: `37528857872`;
- world-pack digest: `c9a3d25d6f64c483`;
- N = 80 worlds from 10 independent authority-derived sub-seeds;
- world descriptions and the digest were logged before any acquisition or scoring;
- full optimized regression suite: PASS;
- Release build: PASS;
- P1/P2 one-use qualification packs were not rerun.

Frozen-threshold results:

- FULL cold autonomous acquisition: **80/80**;
- Wilson 95% CI: **[0.954182, 1.000000]**;
- per-sub-seed acquisition: **[8,8,8,8,8,8,8,8,8,8]**;
- frozen held-out translated exploitation: **80/80**;
- new-EvoPhase checkpoint restore + frozen exploitation: **80/80**;
- DIRECT_ONLY exploration: **9/80**;
- seeded RANDOM_ACTION baseline: **35/80**;
- NO_LEARNING / NO_STRUCTURAL_GROWTH / ZERO_PHASE_LEARNING / ZERO_WEIGHT_LEARNING: **[0,0,9,2] / 80**;
- mean acquisition cost: **20.075** physical interactions, SD **9.957**;
- changed-law autonomous detour repair: **78/80**;
- repair per sub-seed: **[8,7,8,8,8,7,8,8,8,8]**;
- frozen post-repair exploitation: **78/78**;
- revised checkpoint restore + exploitation: **78/78**;
- frozen stale changed-law copies: **0/80**;
- mean repair cost: **16.462** additional interactions, SD **8.847**;
- legacy graph transition table non-zero cases: **0**.

Two changed-law worlds exhausted the frozen 40-interaction repair budget, so the repair result is 78/80 rather than 80/80. This was accepted by the preregistered >=72/80 threshold; no post-observation tuning was performed.

All frozen FRESH-P3 thresholds passed. The pack is now **permanently burned**. It may be used for diagnosis/regression only and must never be called fresh evidence for a design revised after observing this run.

## Current interpretation

P3 establishes bounded autonomous acquisition in a deterministic reset-chain family without a supplied transition curriculum: the organism selects its own opaque physical actions, builds a phase-native forward/value model from the consequences, propagates frontier novelty through acquired synapses, exploits the learned model, restores it into a new carrier, and repairs an unseen detour after factual surprise.

The result does **not** establish general intelligence. The intrinsic novelty rule is still hand-specified, the worlds are fully observed and deterministic, and the tested family is structurally narrow. The next architectural target should remove one of those remaining crutches rather than merely enlarge this same benchmark.
