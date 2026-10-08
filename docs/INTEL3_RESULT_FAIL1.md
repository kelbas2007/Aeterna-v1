# INTEL-3 — frozen unified-cognition V2 verdict

Date: 2026-10-08

## Verdict

**FAIL — frozen unified-cognition V2 did not pass INTEL-3.**

Workflow: `37726178364`  
Job source: `394467ece76329f48638e8f544e3a0b6981d49d3`  
Frozen cognitive baseline: `2c71da210357cb9452d5bd2071faab2f38d2f352`  
Protocol blob printed at seal: `e4e0d5527aeda12d04395ac7b75d542c69f313a0`  
Authority seed / burned pack: `37726178364`  
Evidence artifact: `intel3-evidence`, ID `11527614449`, SHA256 `dc02eb7194ecaf318380a8114ed6d68be08bf81382a3284d7781feaa1c6430c7`.

`INTEL3_SEAL` was printed. The pack is permanently burned.

The cognitive source matched the frozen baseline before and after the run.

## Pre-verdict gate

PASS:

- frozen cognitive source;
- evaluator source guard;
- P4 learned drive;
- G16 abstract model acquisition;
- U1;
- U1 runtime integration;
- U2;
- U3;
- Human Protection;
- Release build.

## Sealed pack

Authority run selected:

- A states/motors: `[21,8,14,3,12]/[4,3,0,5,1]`;
- B: `[18,13,20,2,23]/[4,0,3,5,1]`;
- C: `[0,16,15,1,9,4]/[5,0]`;
- D: `[17,6,5]/[0,2]`, operator **XOR**, bins `[0.16,0.36]`;
- E: `[7,10,19,11,22]/[0,1,4,5]`.

## World A — PASS

- first goal: true;
- cost: **10** <=56;
- translated frozen reuse: true;
- reuse cost: **2** <=7.

## World B — PASS

- first terminal: true;
- cost: **10** <=72;
- switched goal: true;
- switch cost: **4** <=16.

## World C — PASS

Fresh authority mapping, not the burned INTEL-2 mapping.

- context promoted: true;
- acquisition trials: 96;
- unsupported action: none;
- FULL held-out: **40/40**;
- side 0: **20/20**;
- side 1: **20/20**;
- memoryless comparator: **20/40** <=24.

This independently confirms that the post-INTEL-2 local-evidence and promoted-ownership redesign closed the history-dependent failure that stopped INTEL-2.

## Earliest failed criterion — World D

Fresh compositional cue selected XOR.

Observed:

- compositional promotion: **false**;
- held-out FULL: **0/40**;
- best single-descriptor comparator: **20/40**.

Therefore World D fails before a usable compositional representation is established.

Under the preregistered all-or-nothing rule, INTEL-3 verdict is **FAIL**.

## Later observations are not repair authority

The evaluator continued and printed:

- E fast successes: 1;
- law change not reached;
- final frozen A/B retention: 2/2.

These observations do not supersede the earliest failed criterion. The only post-verdict architecture-analysis target is World D.

## System evidence retained

Across the sealed run:

- A PASS;
- B PASS;
- C PASS 40/40;
- final A/B retention observed 2/2;
- Human Protection PASS;
- U1 meta weights unchanged;
- proposal count: 605;
- max U2 candidates: 10;
- dormancy observed: true;
- zero-applicability observed: true;
- legacy planner transitions: 0;
- composite legacy table: 0.

## Scientific boundary

Allowed statement:

> The redesigned unified EvoPhase system independently passes new transport, multi-stage causal goal-switch and history-dependent representation worlds in one persistent frozen lifetime, but still fails the fresh compositional-representation stage of INTEL-3.

Do not relabel this run after later changes.
