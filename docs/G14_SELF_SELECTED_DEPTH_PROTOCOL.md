# G14 — SELF-SELECTED ABSTRACTION DEPTH

Status: **PRE-REGISTERED BEFORE G14 IMPLEMENTATION**

Date: 2026-10-07

## Why G14 exists

G13 statistically qualified one generic physical higher-abstraction mechanism through level 3, but the engine was still enabled with an externally supplied `max_level=3`.

G14 removes the task-specific depth ceiling from the ordinary API.

The organism receives only a generic resource/safety ceiling that is intentionally above every depth required by the development tasks. It must decide from factual explanatory sufficiency whether to:

- stay at the current abstraction level;
- grow one level higher;
- stop growing once the higher representation is sufficient.

## Production API requirement

Add an ordinary open-depth enable path with **no task-depth argument**.

The hard ceiling is a substrate safety/resource bound only and must be >= 8 in the development witness. It must not encode the correct task depth.

The generic higher engine remains the G13 engine:
- one node type;
- one factual learning rule;
- one bottom-up readout rule;
- no L2/L3/L4 task branch.

## Development worlds

Use the frozen 20x20 G13 substrate and the same raw relation vocabulary.

Build the same physical L1 foundation.

Then evaluate two matched organisms using the **same open-depth API**.

### SIMPLE world

Facts require L1 -> L2 abstraction but no L3.

Requirements:

1. before pair evidence: 0 higher candidates;
2. after supported-but-weak L1 pair evidence: exactly 4 promoted L2 nodes;
3. after additional confirming L2 facts: **0 L3 candidates**;
4. frozen held-out L2 action succeeds on unseen spatial bindings.

### DEEP world

Continue from the same L1/L2 foundation with facts that make pairs of L2 nodes individually insufficient but jointly predictive.

Requirements:

1. before residual L3 pair evidence: 0 L3 candidates;
2. after supported-but-weak L2 pair evidence: exactly 2 promoted L3 nodes;
3. after additional confirming L3 facts: **0 L4 candidates**;
4. frozen held-out L3 decisions succeed.

The production API receives no indication that SIMPLE should stop at 2 or DEEP should grow to 3.

## Safety ceiling

The open-depth path uses a fixed architectural safety ceiling >= 8, independent of task and evaluator-required depth.

The test must verify:
- configured safety ceiling > 3;
- SIMPLE ends with highest promoted level 2;
- DEEP ends with highest promoted level 3.

## Controls

1. **OPEN_DEPTH**
   - no task depth supplied.

2. **CAP_LEVEL2**
   - diagnostic control using the old explicit max_level=2 API;
   - must fail the DEEP held-out task.

3. **NO_HIGHER_ENGINE**
   - L1 foundation only;
   - must fail higher held-out tasks.

4. **ALWAYS_ESCALATE diagnostic**
   - may allocate higher candidates before explanatory insufficiency;
   - used only as a structural-economy comparison, never as FULL cognition.

## Causal checks

For DEEP frozen OPEN_DEPTH:

- lesion one necessary L2->L3 physical synapse;
- pi-shift the same synapse;
- restore exact saved synapse without retraining;
- lesion a lower L1->L2 synapse under the target hierarchy.

## Mechanism PASS thresholds

PASS requires all:

1. Open-depth safety ceiling >= 8.
2. SIMPLE held-out score = **4/4**.
3. SIMPLE promoted L2 count = 4.
4. SIMPLE L3 candidate count = **0** after extra confirming facts.
5. SIMPLE highest promoted level = 2.
6. DEEP held-out score = **8/8**.
7. DEEP promoted L2 count = 4.
8. DEEP promoted L3 count = 2.
9. DEEP L4 candidate count = **0** after extra confirming facts.
10. DEEP highest promoted level = 3.
11. CAP_LEVEL2 <= **4/8** on DEEP.
12. NO_HIGHER_ENGINE <= **4/8** on DEEP.
13. Necessary L2->L3 lesion loses >=2/8.
14. Pi phase shift loses >=2/8.
15. Exact restoration returns 8/8.
16. Lower L1->L2 lesion removes dependent DEEP target in both motor permutations.
17. Source guard confirms open-depth API contains no evaluator/task depth constant 2 or 3.
18. G0-G13 / P1-P5 regressions PASS.
19. Release build PASS.

## Interpretation boundary

PASS establishes bounded evidence-driven **self-selection of representational depth** between tasks requiring depth 2 and depth 3 while the available safety ceiling is higher.

It does not establish:
- unbounded recursion;
- arbitrary open-world task discovery;
- autonomous primitive feature invention;
- abstract planning over long horizons;
- language;
- AGI or consciousness.

A later fresh qualification must randomize hierarchy structure, opaque motors and which worlds require depth 2 versus depth 3.
