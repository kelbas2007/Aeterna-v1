# STRUCTURE-1 — two-stage active sensing, cold causal transfer

Status: **PREREGISTERED BEFORE FIRST EXPERIMENT EXECUTION**
Date: 2026-10-09
Branch: `research/beyond-intel4`

## Question

Can the *same frozen EvoPhase cognitive architecture* discover and execute a **two-motor information-gathering dependency**, form a belief from noisy evidence obtained only after that dependency, and use the belief to commit an appropriate outcome action without motor tuition? This differs causally from TE4–Fresh2, in which a single opaque sensor motor directly emits an independent cue.

## Frozen cognition and target-world mechanics

- Freeze physical cognitive `src/` tree at the qualified research-branch parent of this protocol, `13e2c4677e5bfee68ea706a19ac2b9d74a05dfbb:src`. No changes to source, old test generators, pretrained visual foundation, or U1 control until the first verdict.
- New independent local seed `0x57A7_2026_5E71_0001` (unused by TE4/Fresh1/Fresh2).
- Four opaque physical role assignments over six external motors: **arm**, **read**, two **terminal** outcomes and two irrelevant motors; 24 visual classes permuted for both noisy cue identities, visible ready indicator, success and failure. Neither role indices nor class identities enter EvoPhase.
- One hidden balanced binary cause per episode. The external first observation is a noisy 70%-accurate cue, exactly as in earlier controls. Subsequent **independent** cues are available **only after a two-action causal chain**: perform the *arm* motor (which produces an actually visible new READY state), then perform the distinct *read* motor from READY (returns a new noisy cue). Read without arm does not produce new evidence. Arm alone provides **no new information about the hidden cause**. READY is a physical observed state and not provided as a symbolic role label.
- Each successful read consumes the READY condition; gaining a second new cue requires another arm/read pair. Terminal motors commit to hidden-side-conditioned success; other motors produce factual no-ops. No executor picks motors for cognition.
- Reward contingency switches A→B→A by **external outcome changes** at episode 64 and 128, without a mode tag delivered to cognition.
- Exactly 192 self-directed training episodes per arm (64+64+64), one lifetime and one EvoPhase object per arm. Then cognitive checkpoint/restart and 80 balanced frozen heldout episodes per arm, each maximum 12 protected executed actions.
- Reuse the generic pretrained raw visual substrate and U1 policy from Fresh2 only; these are explicitly **not** a from-zero organism. Keep `ScientificRuntime::step_unified`, factual POST and Human Protection; no adapter route/planner priorities and no hidden-label reward leakage.

## Predeclared pass/fail metrics

A *STRUCTURE-1 DEVELOPMENT PASS* requires **ALL four arms individually**:

1. At least 64/80 correct terminal commits, and strictly more than the realized first-cue oracle.
2. At least 32/80 heldout episodes with two **actual** extra independent cues produced by a self-selected arm→read chain.
3. Causal chain learned from action consequences, with zero unsupported actions or protection blocks; no PRE/POST fabrication.
4. Frozen U1 weights unchanged on holdout; positive physical causal dependence and checkpoint retention must be separately validated before any scientific promotion.

A single failure is `STRUCTURE1_DEVELOPMENT_FAIL`, regardless of whether the test or CI runner exits successfully. Report first-attempt outcomes per arm, attempts to arm and read, successful chains, committed/missing actions, sampling, reward, first-cue and three-cue oracles, physical sensor models, and checkpoint status.

**Scientific limits:** the synthetic generator and visual primitives remain bounded. If this fails, preserve this consumed evidence and diagnose why; do not retune/reseed the same first-attempt test to turn the FAIL into PASS. Research and repair on separate open mechanism controls, then preregister a genuinely new challenge. Do not touch qualified INTEL-4 main.
