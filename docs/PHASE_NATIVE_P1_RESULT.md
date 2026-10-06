# P1 — shared-cell phase-coupled execution: result

Date: 2026-10-06
Status: **P1 MECHANISM_PASS and FRESH_BOUNDED_PASS under the preregistered P1 contract.**
This is not full physical cognitive ownership, forward generative imagination, G10, or AGI.

## Executed checkpoint and evidence

- Base before this work: `6723d835107588bc7f4466c8811a3d97db5d8fce`.
- Preregistration commit, before implementation: `8f57aa47113338731821ac58d4c48714e02e66ca`.
- Tested source: `f33028a5d2809fd7500d46a2021b35f230b1e704`.
- Protocol blob: `99b7eab4b9bdd31b4d3ae1a3fac79f4f4443a4da`.
- GitHub Actions run / fresh authority: `37522194429`.
- Job: `112470148845`.
- World-pack diagnostic FNV digest: `70672a58c33ca2ff`.
- All generated world parameters and the digest were printed before all tuition and scoring.
- Regression command: `cargo test --all --all-targets -- --nocapture` — success.
- Release command: `cargo build --release` — success.
- Fresh command: `cargo test --release --test phase_native_execution p1_fresh_bounded_shared_cell_pack -- --ignored --nocapture` — success.
- Tracked source was checked before and after execution with `git diff --exit-code`. It was not automatically reformatted or rewritten in CI.

Primary execution evidence is the completed [workflow](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37522194429) and its job logs. The metrics below were read from those logs, not inferred from test code alone. This report is a post-run record, not a replacement preregistration.

The pack is now **burned for future fresh qualification after any design change**. Historical G1/G7/G8/G9 fresh packs were not newly qualified in this run. Their ordinary regression/preflight tests ran; their guarded fresh bodies did not.

## What changed

`src/phase_native.rs` is a child module of the existing carrier. It recruits receptors and conjunctive relays in the existing `EvoPhase.cells` array and stores learned conductances, outcome weights, phase offsets and eligibility in the existing `EvoPhase.synapses` array. Circuit metadata holds structural addresses and evidence counts, not a second executable transition/reward table.

Factual PRE/action/POST/outcome observations establish afferent, successor, outcome and motor connections by generic local learning rules. Ordinary `observe_planning_transition`, `plan_imagined`, depth-limited and belief-planning APIs route to this circuit when native mode is explicitly enabled. Native mode has no fallback to the legacy planner. The legacy transition table and graph rollout are absent in this mode.

Execution is bounded synchronous value propagation through phase-coherent learned conductances. IMAGINED membrane charges are stored in a mode-isolated buffer indexed by the same physical cell IDs; the acquired topology, weights and phase offsets are shared. REAL observations, factual tick and acquired parameter fingerprints remain unchanged during the tested thought/readout operations.

The standalone graph planner remains an explicit reference backend, and the original negative architectural audit remains unchanged as a regression of that legacy path. Old G8/G9 scores have not been relabeled as phase-native results.

## Verified results

Regular P1 tests passed across 24 deterministic cases: all six permutations of three motor labels, each at chain lengths 2, 3, 4 and 5. These are development/regression witnesses, not fresh statistical evidence.

Fresh qualification sampled 80 test instances from 10 run-derived sub-seeds, eight instances per sub-seed. Each instance used its own newly trained carrier, randomized opaque motor labels, factual-record order rotation and an absolute raster translation absent from its tuition.

| Arm or intervention | Completed delayed-reward tasks |
|---|---:|
| Intact acquired native circuit | 80/80 |
| Independent graph reference, identical factual tuition | 80/80 |
| Necessary internal successor connection: conductance set to zero | 0/80 |
| Same connection: phase offset shifted by pi, topology and weights retained | 0/80 |
| Exact restoration of the original connection/phase, no relearning | 80/80 |
| Phase learning rate zero during acquisition | 0/80 |
| Weight learning rate zero during acquisition | 0/80 |
| No dormant capacity | 0/80 |
| Structural growth disabled | 0/80 |

In all 80 intact-success instances, the intervention suite additionally verified:

- an unrelated connection lesion retained the correct decision;
- disconnecting all native motor-output synapses caused abstention, not a graph fallback;
- one-step readout preferred the immediate reward, while the acquired multi-step circuit preferred the delayed reward;
- reversal of the terminal outcome from 1 to 0 changed the initial decision after 32 factual devaluation observations;
- that revision retained the same relay and outcome-synapse IDs and retained contradiction evidence;
- the frozen-learning matched control retained its obsolete delayed-route decision and unchanged learned fingerprint;
- restoring the exact lesioned parameters recovered the learned-state fingerprint;
- complete intact evaluation executed 2–5 opaque actions and reached evaluator-confirmed terminal Need.

The 80/80 revision figure refers to **same-identity outcome devaluation and the corresponding changed initial decision**, not a separately scored full changed-world episode or a demonstrated change to the transition topology.

## Statistics and scope

Native count: 80/80. Per sub-seed: `[8,8,8,8,8,8,8,8,8,8]`.
Nominal two-sided Wilson 95% interval: `[0.954182, 1.000000]`.

This interval summarizes the sampled-instance success proportion under a binomial interpretation; it is not a guarantee for unknown task families. Instances come from a narrow shared generator, and sampling permits repeated configurations. They must not be advertised as 80 unique independent task families. The block-level descriptive t interval printed as `[1,1]` because every block scored identically; this degeneracy is not evidence of zero uncertainty.

Every instance was taught its small transition set before held-out execution. The result qualifies acquired execution and held-out spatial binding under those conditions, **not zero-shot discovery of an unseen world law or transfer to an untrained topology**. The graph reference also solved 80/80: no competitive accuracy, sample-efficiency or speed advantage is established.

## Costs

The fixed tuition rule presents every factual transition 160 times. For chain lengths 2–5, this is 640–1120 generated factual transition presentations per acquisition, mean **854.000**, sample standard deviation **179.913**. These are evaluator-supplied factual tuples; P1 does not demonstrate autonomous exploration or autonomous choice of a tuition curriculum.

The same primary tuition was supplied independently to the native, reference and four acquisition-control arms. The intervention suite reacquired an additional matched native checkpoint; that extra audit acquisition is not included in the per-arm primary tuition mean. It must not be hidden when estimating total experiment work.

Held-out native execution used mean **3.337** physical simulator actions as printed in the log, sample standard deviation **1.124**. Revision added **32** factual terminal-outcome observations to each of the revision and frozen-control copies. No efficiency claim is based on these values.

## Architectural interpretation — what remains unresolved

The causal result is narrower than the project-wide goal: this implemented decision path depends on acquired shared synapses and their learned phase calibration. Selective connection or phase disruption changes its result, and restoration recovers it without retraining.

However, the local value recurrence and winner-take-all competition are inherited rules written by the developer. Their mathematical content is a phase-gated, finite-horizon value-propagation computation; successful lesions do not prove spontaneous invention of a reasoning algorithm. Phase offsets are learned during acquisition, but phases are not evolving oscillator trajectories during the P1 settling loop. This is a CPU simulation of phase-dependent connectivity, not a hardware or spiking-neural result.

Further limits:
- HDC prototype matching remains inherited representation machinery.
- Native state recruitment currently occurs on unmatched factual traces/transitions; P1 does not establish a general residual-gated developmental policy.
- The tested thought propagates outcome value; it does not generate future sensory observations.
- Native mode is explicit opt-in. Other existing cognitive subsystems have not all been migrated to the shared physical execution path.
- Belief API routing was implemented, but the P1 fresh suite qualifies raw-raster chains, not a new phase-native G9 claim.
- Cycles, stochastic transitions, signed outcomes, topology revision, long-horizon scalability, nuisance robustness and restart persistence remain unqualified by P1.
- The source guard checks named forbidden execution paths and actual shared-array usage; it is not a formal proof of whole-repository ownership.

The next architectural requirement is forward, phase-dependent prediction and continuation of acquired trajectories in the same carrier, with interventions on that execution and ordinary integrated-agent validation. Full ownership remains a requirement, not a completed status.

## Usage boundary

The opt-in configuration is exported at `aeterna_v1::carrier::PhaseNativeConfig`. On an appropriately configured EvoPhase with an attached raw-raster field:

```rust
evo.enable_phase_native_planning(PhaseNativeConfig::default());
// Acquire through actual factual observations; native mode does not import a graph table.
evo.observe_planning_transition(&pre, opaque_action, &post, factual_value);
// Freeze only after sufficient factual acquisition, then use the ordinary API.
evo.set_planning_learning_enabled(false);
let decision = evo.plan_imagined(&observation); // May abstain: Option<PlanDecision>.
```

In native mode `PlanDecision.expanded_nodes` is a compatibility field counting local relay updates, NOT graph frontier expansions. Use `phase_native_local_updates()` for explicitly named diagnostics. `planning_transition_count()` and `imagined_rollout_nodes()` describe the legacy backend and are zero in native mode.

## Exact summary lines from the completed job

```text
P1_DEBUG cases=24 lesions=causal phase=causal restoration=exact revision=same_identity graph_table=absent
P1_SEAL source=f33028a5d2809fd7500d46a2021b35f230b1e704 spec=99b7eab4b9bdd31b4d3ae1a3fac79f4f4443a4da authority=37522194429 pack_digest=70672a58c33ca2ff N=80 seeds=10 BEFORE_ALL_TUITION_AND_SCORING
P1_FRESH full=80/80 wilson95=[0.954182,1.000000] per_seed=[8, 8, 8, 8, 8, 8, 8, 8, 8, 8] reference=80/80 controls_zero_phase_zero_weight_no_capacity_no_growth=[0, 0, 0, 0] lesion=0/80 phase=0/80 restored=80/80 revised_same_identity=80/80
P1_COST matched_tuition_mean=854.000 sd=179.913 physical_actions_mean=3.337 sd=1.124 revision_observations=32 block_t95_descriptive=[1.000000,1.000000] block_interval_degenerates_if_all_blocks_equal=true
```

The run passed on its first executed P1 checkpoint. Existing warnings in legacy evaluator helpers were present but did not fail compilation. There was no P1 result-driven runtime or protocol change after the fresh pack was seen. Subsequent report/README and CI-trigger changes do not replace the tested source identity above.

CI was returned to manual-only in `e0bc458653bfcb5fa16adccbd46b4bff4707a68f`; ordinary manual runs do not consume fresh packs. A new P1 fresh run requires explicit `qualify_p1=true` and the first attempt of a new workflow run. No continuous background execution is configured.
