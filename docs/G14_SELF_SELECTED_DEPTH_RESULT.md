# G14 — self-selected abstraction depth

Date: 2026-10-07

## Verdict

**MECHANISM PASS — representational depth is selected from factual sufficiency rather than supplied as task depth.**

Workflow: `37604831999`  
Source: `29f37f5b9d196533bbae205f8df91385a26d302d`

Protocol: `docs/G14_SELF_SELECTED_DEPTH_PROTOCOL.md`

## Result

The ordinary open-depth API receives no task-depth argument. Its generic architectural safety ceiling is **16**.

Using the same open-depth engine:

### SIMPLE task

- held-out: **4/4**;
- highest promoted level: **2**;
- promoted L2 nodes: 4;
- L3 candidates after extra confirming facts: **0**.

### DEEP task

- held-out across both motor mappings: **8/8**;
- highest promoted level: **3**;
- L3 nodes: 2;
- L4 candidates after extra confirming facts: **0**.

Controls/interventions:

- CAP_LEVEL2: **0/8** on DEEP;
- NO_HIGHER_ENGINE: **0/8**;
- necessary L2->L3 lesion: **4/8**;
- pi phase shift: **4/8**;
- exact restore without retraining: **8/8**;
- lower L1->L2 lesion left dependent target correct: **0/2**;
- open-depth source guard PASS;
- full optimized regressions PASS;
- Release build PASS.

## Architectural meaning

The available physical substrate permits much deeper abstraction than either development task needs, but the organism does not automatically fill it.

The same production API:
- stops at L2 when L2 is sufficient;
- grows to L3 when L2 is supported but insufficient;
- stops again once L3 becomes sufficient.

This removes the evaluator/task-specific `max_level=3` choice from ordinary cognition. A hard safety ceiling remains as a resource boundary, not as the answer to the task.

## Boundary

This is a bounded self-selected-depth mechanism witness across tasks requiring levels 2 and 3. It does not establish unbounded recursion, open-world goal discovery, abstract planning, language, AGI or consciousness.
