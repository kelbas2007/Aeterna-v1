# P4 — LEARNED PHASE-NATIVE EXPLORATION DRIVE

Status: **PRE-REGISTERED BEFORE P4 IMPLEMENTATION**

Date: 2026-10-06

## Question

Can Aeterna-v1 learn a reusable exploration drive from the factual usefulness of its own experience, store that drive in phase-native physical synapses, transfer only that learned drive into a cold organism with no world model, and use it to acquire new longer worlds without the hand-written P3 frontier selector?

P4 targets the remaining P3 crutch: P3 defines "unknown action / reachable unknown frontier = interesting" directly in substrate code.

P4 does **not** claim spontaneous invention of curiosity from nothing. A generic P3 policy is allowed only during source-world meta-tuition as a bootstrap action generator. It is forbidden during target-world qualification.

## Physical ownership

The learned drive must be represented by cells/synapses in the same EvoPhase physical arrays.

Allowed inherited generic features:
1. **DIRECT_UNMODELLED** — whether the current receptor/action pair has no sufficiently supported factual transition.
2. **REACHABLE_FRONTIER** — generic phase-native membrane activity induced by unmodelled actions at acquired receptors and propagated backward over acquired successor synapses.

The feature definitions are inherited substrate quantities. Their **behavioral importance must not be fixed by P4 code**. P4 begins with zero drive weights.

Drive readout must evaluate candidate actions through acquired drive synapse weights and phase coherence. No Rust table may map world/state/action identities to exploration value.

## Factual learning signal

The drive is trained only from changes in EvoPhase's own model after actual physical interaction.

For a selected action:
- factual P2 learning occurs first;
- structural information gain is 1 when the interaction recruits a previously absent transition circuit or receptor, otherwise 0;
- a bounded temporal-difference target may add discounted predicted exploration value of the factual successor;
- drive weights/phases update locally from the pre-action generic feature activity and that internal target.

No evaluator route, hidden advancing action, distance-to-goal or state ID enters the drive update.

## Source-world meta-tuition

Use at least 8 independently cold source worlds.

Each source world:
- has a reset-chain length 2–3;
- has three opaque motors;
- has a hidden advancing motor that may differ by state;
- wrong motors reset to the start;
- positive factual value appears only at terminal goal.

For source tuition only:
- P3 autonomous selector chooses physical actions;
- if a learned drive is attached, the selected action's P4 features are staged before execution;
- P4 updates only after the factual result;
- after a source world, only a P4 drive checkpoint is carried forward.

World receptors, transition circuits, current REAL state and goal model must **not** transfer between source worlds.

## Drive-only checkpoint

A P4 drive checkpoint may contain:
- learned drive synaptic weights;
- learned drive phase offsets / confidence;
- drive-learning observation count and generic drive config.

It must not contain:
- world receptors;
- transition circuits;
- decoded world sensory patterns;
- current REAL observation;
- evaluator labels.

Restoring a drive checkpoint into a newly constructed cold carrier must leave:
- phase-native receptor count = 0;
- phase-native transition circuit count = 0;
- legacy graph transition count = 0.

## Target transfer

After source tuition:
- freeze P4 drive learning;
- create a newly constructed cold EvoPhase for each target world;
- restore only the learned P4 drive;
- keep ordinary P1/P2 world-model learning enabled;
- forbid P3 `choose_phase_native_autonomous_action`;
- all acquisition actions must come from P4 `choose_phase_native_learned_drive_action`.

Target family:
- at least 12 worlds;
- reset-chain length 4–5, longer than source tuition;
- hidden advancing motor independently permuted by state;
- unseen raster relations/bindings relative to source worlds where practical;
- 60-interaction acquisition budget.

The learned drive must therefore transfer as a way of valuing experience, not as memorized source-world routes.

## Controls

Matched target controls:

1. **ZERO_DRIVE**
   - same cold carrier and P1/P2 learning;
   - P4 drive weights remain zero;
   - uses the same P4 readout/tie-breaking implementation.

2. **FRONTIER_WEIGHT_LESION**
   - starts from the learned drive checkpoint;
   - the actual learned REACHABLE_FRONTIER drive synapse is set to zero after restore;
   - DIRECT_UNMODELLED drive remains.

3. **RESTORED_LESION**
   - exact frontier drive synapse is restored without retraining;
   - capability/cost must recover on the same deterministic cases.

4. **ZERO_PHASE_DRIVE_TUITION**
   - source meta-tuition has drive phase learning disabled;
   - target transfer uses the resulting checkpoint.

5. **RANDOM_ACTION**
   - seeded external diagnostic baseline; not cognition.

6. **P3_TEACHER_CEILING**
   - diagnostic only on target worlds; hand-written P3 selector may be measured as an upper/efficiency reference but cannot count as P4 success.

## Causal source guard

The production P4 target selector must not call or contain:
- `phase_native_exploration_action`;
- `choose_phase_native_autonomous_action`;
- evaluator world/state structs;
- BFS/DFS/queue/heap/frontier-node route search;
- legacy `EvoImaginationPlanner`.

It must use the physical drive synapses and same phase-native cells/successor circuits.

## Deterministic acceptance

P4 mechanism PASS requires all:

1. Drive weights start exactly zero before source tuition.
2. At least 8 independently cold source worlds contribute factual drive updates.
3. Source-to-source transfer carries only the drive checkpoint; every new source carrier begins with zero receptors/circuits.
4. After source tuition, DIRECT_UNMODELLED and REACHABLE_FRONTIER physical drive weights are both > 0.
5. At least 12 target worlds are longer than source worlds.
6. P4 target selector supplies 100% of target acquisition actions; P3 selector supplies 0%.
7. LEARNED_DRIVE reaches first factual goal in >=11/12 target worlds within 60 actions.
8. ZERO_DRIVE succeeds in <=6/12.
9. FRONTIER_WEIGHT_LESION is strictly worse than LEARNED_DRIVE in success or mean interaction cost.
10. Exact synapse restoration recovers the lesion loss without drive retraining.
11. ZERO_PHASE_DRIVE_TUITION is strictly worse than normal learned-drive transfer.
12. LEARNED_DRIVE beats RANDOM_ACTION in success or mean interaction cost.
13. Restoring drive alone into target leaves receptors=0, circuits=0, legacy graph transitions=0 before first target observation.
14. Drive learning remains frozen throughout target qualification.
15. P0/P1/P2/P3 and all prior ordinary regressions PASS.
16. Release build PASS.

## Advancement rule

A deterministic P4 PASS is a mechanism witness only. A one-use fresh statistical pack must be separately sealed before a generalization claim.

## Interpretation boundary

P4 PASS would establish learned, transferable weighting of generic epistemic features in the phase-native substrate.

It would still not establish:
- invention of the feature vocabulary itself;
- stochastic/noisy curiosity;
- general POMDP exploration;
- continual lifelong learning without interference;
- arbitrary goals/domains;
- AGI or consciousness.
