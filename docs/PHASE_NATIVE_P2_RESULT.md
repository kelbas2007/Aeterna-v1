# P2 — shared-synapse forward prediction and action/fact loop: result

Date: 2026-10-06
Status: **P2 MECHANISM_PASS and FRESH_BOUNDED_PASS for the implemented relational forward-prediction and integrated-action contract.**
This is not general intelligence, autonomous model acquisition, G10, or full migration of all cognition into the physical substrate.

## Executed source and primary evidence

- Base: `fb8b62c090a1fcbb66513ac7a6d327c151c811e8`.
- Preregistration, before implementation: `438b26a42f20a21620ebe03f279ed85406ee6d94`.
- Tested source: `f6503b234f51a11d1a8d930e61a665bae557b7fd`.
- P2 protocol blob: `815c20cc953d47e2a690e92a271e30028cedadd3`.
- GitHub Actions run / fresh authority: `37524647331`.
- Job: `112478444484`.
- Diagnostic FNV world-pack digest: `ede2aad4817f943b`.
- Primary execution record: https://github.com/kelbas2007/Aeterna-v1/actions/runs/37524647331
- Preserved fresh-output artifact: `p2-execution-evidence`, artifact ID `11442206205`, containing `p2-evidence.log`; retention configured to 30 days.
- Artifact ZIP SHA256 reported by Actions: `eeb94b6cb27db5b41f72ac1fef990f4dae01621ee01d7502f0af544cbea2fb35`.

These results were read from the completed job's logs, not inferred from source code or a green status alone. Every generated case and its digest were printed before fresh acquisition and scoring. The pack is now burned for future qualification after a design change.

Executed successfully:

```text
cargo test --release --all --all-targets -- --nocapture
cargo build --release
cargo test --release --test phase_forward_execution p2_fresh_forward_and_integrated_pack -- --ignored --nocapture
```

Tracked source was checked with `git diff --exit-code` before and after execution and was not rewritten by CI. P1's 24-case regression and the historical negative graph-execution audit remained intact. Old fresh G1/G7/G8/G9/P1 qualification bodies were not consumed in this run. Existing unused-code warnings in legacy test helpers were present; this is not a warning-free build claim.

## Preserved technical attempt

| Attempt | Source | Run | Result |
|---|---|---|---|
| CI-1 | `671148369d4d43ead8d792dd243e59ef55789e68` | `37524581849` | TECHNICAL_FAIL_WORKFLOW_YAML: workflow configuration failed before any job, compilation or scoring. |
| CI-2 | `f6503b234f51a11d1a8d930e61a665bae557b7fd` | `37524647331` | Regression, Release and one-use P2 qualification all succeeded. |

The correction changed the YAML representation of the job condition to a folded scalar. It did not change the algorithm, test thresholds, generator or protocol. No cognitive failure was relabeled as a success. No P2 runtime/test/protocol revision was made after the fresh scores were observed.

## Implemented mechanism

`src/phase_forward.rs` is included in the existing phase-native carrier module. It uses the same `EvoPhase.cells` and `EvoPhase.synapses` arrays as P1. Receptor-to-sensory decoder connections acquire amplitudes as synaptic weights and phase calibration as synaptic offsets from factual sensory observations. No separately stored future raster or vector-valued prediction table is read by the forward kernel.

One opaque action gates afferent current into the relevant acquired relay. Successor synapses propagate that current and phase into successor cells, and learned decoder synapses reconstruct a numeric sensory pattern. Subsequent imagined actions continue from the resulting membrane activity, without evaluator-provided intermediate observations. The initial observation still uses inherited HDC prototype recognition. The forward kernel itself has no access to receptor prototypes or task labels.

The preregistration described the output objective as a predicted relational phase vector. The implemented route reconstructs a numeric sensory raster through synapses and then uses the existing HDC encoder for relational comparison; it is not a newly learned direct HDC-vector decoder. Acceptance below refers to that measured relational prediction, not literal equality of the observer's raw pixel coordinates.

The activity uses a bounded synchronous phasor update rule. That update rule, action gating, and output-abstention thresholds are inherited. This is not a claim of self-invented reasoning physics, biological oscillation, or spiking dynamics.

## Ordinary integrated loop

The following methods are now directly available on `EvoPhase`:

- `observe_phase_native_forward_transition`: ingest factual PRE/action/POST/value, measure pre-update prediction error, and optionally learn.
- `imagine_phase_native_actions`: continue a supplied opaque action sequence from the initial observation and return a valid predicted prefix.
- `choose_phase_native_action_with_prediction`: select using native P1 value propagation and generate P2 prediction before physical execution.
- `observe_phase_native_action_result`: receive the actual external POST, compare it against the pre-update model, optionally revise, then advance REAL.

**P1 chooses the action; P2 predicts its sensory consequence.** P2's forward path has not independently replaced the P1 value-selection recurrence or demonstrated an additional decision-quality advantage over it.

In the tested integrated episode, the evaluator calls the organism for its action and forecast before executing the world transition. It does not choose the action for the organism. Thinking leaves the checked REAL frame, tick and learned-parameter fingerprint unchanged. The actual externally supplied POST then advances REAL. Scored episodes keep learning frozen.

## Completed results

Development/regression: all six permutations of three opaque actions at lengths 2, 3, 4 and 5, **24/24** complete causal cases. These are mechanism tests, not fresh evidence.

Fresh: **80 generated instances from 10 run-derived sub-seeds**, with a new acquired carrier per instance. Every instance uses translated observations absent from its origin-only tuition, randomized motor relabeling and factual record-order rotation.

| Measurement or intervention | Result |
|---|---:|
| Entire 2–5-step sensory forecast, correct relational prediction at every step | 80/80 |
| Ordinary native selection -> prediction -> world execution -> factual comparison -> delayed Need | 80/80 |
| Necessary internal successor conductance cut: full forecast survives | 0/80 |
| Same successor phase shifted by pi: full forecast survives | 0/80 |
| Exact restoration without learning: full forecast recovers | 80/80 |
| Decoder-only phase disruption: full forecast survives | 0/80 |
| Unrelated successor connection disrupted: full forecast retained | 80/80 |
| Factual changed-successor learning: new consequence forecast correctly | 80/80 |
| Frozen-learning matched copy: new consequence forecast correctly | 0/80 |
| Unchanged other state/action consequence retained after revision | 80/80 |
| Zero phase learning during acquisition: full forecast | 0/80 |
| Zero weight learning during acquisition: full forecast | 0/80 |
| No dormant capacity during acquisition: full forecast | 0/80 |

The necessary internal lesion occurs after the first valid predicted transition: all 80 cases retained the correct one-step prefix and stopped at the broken second link. This is stronger than merely making the entire system return no output at its input.

Decoder-only interventions alter actual acquired output synapses while recognition prototypes remain untouched. Restoration recovers the learned-state fingerprint and full output without retraining. This establishes causal dependence on the learned decoder rather than a direct copy of the successor's stored observation.

All intended prediction steps met relational cosine similarity >=0.97; terminal/nonterminal predicted outcome tolerances in the full forecast were also checked. The tests did not require raw-pixel alignment with the translated observer frame.

## Factual revision scope

After acquisition, the terminal successor of one known state/action was changed to a distinct sensory state while its reward remained 1. Revised and frozen copies each received 128 identical new factual transition presentations.

The revised copy predicted the new successor; the frozen copy retained the old forecast and unchanged learned fingerprint. Old circuit relay/synapse addresses and contradiction evidence were preserved while a new successor circuit was acquired and the contradicted afferent was attenuated. An unchanged alternative transition remained correctly predicted.

This is **local deterministic successor revision with retained old evidence**, not a claim that the old successor synapse itself was rewired in place, that a whole macro retained identity through arbitrary restructuring, or that an entire changed-world task was freshly solved. The 80/80 integrated task score belongs to the original frozen world; changed-world evidence here is a separate revised-transition prediction test.

## Statistics and cost

Combined acceptance: 80/80. Per sub-seed: `[8,8,8,8,8,8,8,8,8,8]`.
Nominal Wilson 95% interval: `[0.954182,1.000000]`.

This is a sampled-instance summary under a binomial interpretation, not a guarantee over new world families. The instances share one narrow generator and can repeat configurations; they are not 80 independent domains or unique laws.

Tuition is 160 presentations of every individual factual transition, **640–1120 presentations per acquisition**, mean **904.000** in the fresh sample. Four primary acquisitions per instance received the same factual tuples: intact, zero-phase, zero-weight and no-capacity. Revision added 128 presentations to each revised and frozen copy. No autonomous exploration or data-efficiency advantage is demonstrated.

The learner is taught the small transition set, but not a complete multi-step route script. The forecast composes acquired one-step connections. Transfer is to new spatial bindings and composed sequences of known transitions, not to completely unseen transition laws.

## Remaining intelligence gap

The implementation now contains an experimentally exercised experience -> acquired network -> forecast -> selected action -> factual result -> local revision loop. It remains a bounded component, not the requested full intelligence.

Outstanding limitations include:

- Autonomous cold-start collection of informative experience is not implemented by P2; the evaluator still supplies tuition tuples.
- Decoder reconstruction is in the tuition coordinate frame. Multiple unregistered locations assigned to one relational receptor may average incompatible raw pixels. No learned spatial registration is established.
- Output reconstructs previously acquired sensory patterns; no new object formation or unrestricted scene generation is established.
- Deterministic suppression of alternative successors is not a sufficient treatment of stochastic transitions, hidden context or competing conditional laws.
- The inherited representation machinery and other existing cognitive subsystems have not all migrated into this physical execution path.
- Novel-goal transfer beyond rehearsed small transition sets, efficient continual learning, long-horizon scalability and restart persistence remain unqualified here.
- Read-only prediction and source guards are bounded tests, not a formal proof of all provenance or ownership properties. Factual APIs still depend on the trusted shell to supply genuinely external observations.

The next capability criterion is not another renamed PASS: one cold-start organism must choose its own informative actions, acquire enough of the world from actual interaction, solve goals not provided as route scripts, retain useful old knowledge under revision, and preserve that acquired state across restart. P2 supplies a forward-model component for that task without declaring the larger objective complete.

## Exact summary lines read from the completed job

```text
P2_MECHANISM passed=24/24
P2_SEAL source=f6503b234f51a11d1a8d930e61a665bae557b7fd spec=815c20cc953d47e2a690e92a271e30028cedadd3 authority=37524647331 digest=ede2aad4817f943b N=80 seeds=10 BEFORE_ALL_TUITION_AND_SCORING
P2_FRESH accepted=80/80 wilson95=[0.954182,1.000000] per_seed=[8, 8, 8, 8, 8, 8, 8, 8, 8, 8] counts_full_integrated_lesion_phase_restored_decoder_unrelated_revised_frozen_unchanged=[80, 80, 80, 80, 80, 80, 80, 80, 80, 80] control_full_forecasts_zero_phase_zero_weight_no_capacity=[0, 0, 0]
P2_COST tuition_per_arm_mean=904.000 arms_per_case=4 revision_presentations_per_revision_and_frozen_copy=128 full_route_length=2..5
```

CI was restored to manual-only in `93ecd69339bf4c67a5dcc29e464a890e65cc316f`. Both `qualify_p1` and `qualify_p2` default to false and require explicit first-attempt fresh-run selection. There is no continuous background execution. The usage and result documents plus CI-trigger restoration do not replace the exact tested source identity above.
