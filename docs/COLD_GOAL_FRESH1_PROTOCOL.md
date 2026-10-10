# COLD-GOAL-FRESH1 — frozen-source first-attempt untaught causal model transfer

Date: 2026-10-10. Research branch `research/beyond-intel4`.
State: **PREREGISTERED BEFORE FIRST TARGET RUN**. Strictly no edits to frozen cognitive `src/` tree or evaluator after outcomes.

## Previous evidence (open, not scientific qualification)
The original [COLD-GOAL-1 open experiment](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38025025890) passed 4/4 opaque assignments after source changes; six acquired transitions per organism, frozen four-step goal and changed-law recovery. Earlier warm GOAL-REPLAN-1 taught individual causal edges; this test eliminates ALL such edge tuition.

## Frozen cognitive source
`1405d2f2af9b5ba3a1eb76481488f0f9d96c5173:src`.
Check actual Git tree identity before first run, not merely a branch name.
New separate unused seed `0xC01D_2026_FE57_1010` identifies **12 fresh opaque motor and visual state assignments**, not previously used to repair this source.

## First-attempt task
- Each of 12 arms is one fresh-to-world EvoPhase with the identical previously acquired generic visual/abstraction foundation and pretrained U1 policy, **but zero PRE/action/POST knowledge for the target six-state world**. No host-taught transitions and no externally imposed motor schedule.
- Six opaque motors, six recognized physical state classes: one start, one junction, two successive route states, one detour and one goal; exactly six actual changing-state motor edges; all other motor invocations produce true factual no-op POSTs.
- 192 independent starts (episodes) per arm, each at most 16 protected real actions, a continuous EvoPhase lifetime. `ScientificRuntime::set_goal` receives only the raw target visual observation, never a hidden correct motor/path.
- During acquisition self-select all unknown motor experiments, learn conducted transition links from factual PRE/action/POST, and use task_outcome=1 only on actually reaching goal.
- Checkpoint/restart. Freeze all model learning for intact-goal check (limit 12 protected actions). No further training during intact check.
- Same organism resumes learning; the external world silently blocks a previously working interior edge. Observe failed step, physically invalidate contradicted link and reach the goal through a detour assembled from acquired transitions (limit 12 actions).
- Independently lesion and restore the first source→motor synapse of the acquired goal route; verify path availability truly depends on physical causal evidence. Checkpoint and U1 weights survive restart.

## Predeclared ALL-ARMS PASS
All 12 arms independently:
1. >=6 physically acquired factual changing-state transitions, at least one experienced successful goal during training.
2. Frozen, unsupported/blocked-free goal reach using a 4-action plan, with actual protected `step_unified` execution.
3. The same organism receives a genuine no-op on the formerly valid interior transition; it changes its plan, traverses BOTH detour edges and reaches the target within 12 actions.
4. Lesion first necessary physical link removes the route; restoring it restores the route. Checkpoint retains learned links and frozen U1 metaparameters.
5. No supplied hidden law-change flag, motor IDs, preprogrammed route or edge tuition.

A single failure implies **COLD-GOAL-FRESH1 DEVELOPMENT_FAIL**, permanently recorded for this seed/source. Do not rerun after modifying code, change the seed or threshold, or translate compilation SUCCESS into cognition PASS.

## Interpretation
A PASS confirms bounded self-directed causal model acquisition and goal re-planning transferred to unseen role assignments **within one fixed deterministic synthetic task family**. It is not wholly untrained-from-zero perception, not structurally novel environments, not proof that synaptic dynamics invent the Rust BFS search or exploration threshold, not arbitrary long-horizon general planning, real-world safety or AGI. Every arm is a separate continuous organism.
