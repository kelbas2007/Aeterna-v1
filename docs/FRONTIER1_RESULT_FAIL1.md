# FRONTIER-1 — frozen stochastic latent-evidence result (2026-10-08)

**Verdict: FAIL** in a genuinely new noisy partial-observation family. This is an independently seeded single-use run, not a re-scoring of INTEL-4 A–E and not a reason to relabel qualified INTEL-4 evidence.

- Workflow: [37831945655](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37831945655), conclusion **failure at scored target**.
- Frozen EvoPhase cognition: `c7b5455ba006b297288fa8d16ef6300c8a19ceca` (unchanged before/after).
- Authority source commit: `80547ed1df55b40f8a765c7bbb88bf15260d9f77`.
- Preregistered protocol SHA at seal: `04d0131136ec55e3a75d8a4ae2dceca99e5bbb70`.
- Authority seed: `37831945655` (GitHub run ID).
- `FRONTIER1_SEAL` printed before target interaction: noisy cues `[2,12]`, terminal goal `1`, dead `10`, sample action `0`, commit actions `[3,5]`.
- Source integrity, preseal evaluator guard, runtime U1/U3 + protection, Release: **PASS**.
- Evidence uploaded by workflow `frontier1-evidence`.

## Facts observed

| Measurement | Result |
|---|---:|
| Training outcome | 162/320 factual correct commits |
| Training commits | 320/320 episodes |
| Frozen heldout correct commits | **40/80** |
| Frozen heldout commits | 80/80 |
| Best one noisy-cue oracle on these episodes | **56/80** |
| Eight-reading majority oracle on the same hidden episodes | **69/80** |
| Episodes with at least 2 active sample actions | **0/80** |
| Mean actions per heldout episode | **1.0** |
| Mean sampling actions | **0.0** |
| Legacy planner graph and answer tables | **0/0** |

The environment balanced hidden modes 40/40; a constant commit choice scores exactly 40/80. The organism selected a commit action immediately, without ever taking the additional information-gathering motor during the scored episodes. It therefore failed the preregistered ≥64/80 score and multi-sample requirement.

## Interpretation

The previously qualified developmental mechanisms can learn deterministic cause/context/composition/revision in the A–E family, but this new controlled run **did not demonstrate** gathering and integrating multiple noisy observations about a hidden latent cause. The immediate failure is **active-sensing policy choice**, before any claim about deeper Bayesian or multi-step working-memory inference can be evaluated. The run does not prove that EvoPhase is intrinsically incapable of such inference under *all* designs; it identifies a clean missing demonstrated competence for this frozen architecture.

The current INTEL-4 PASS remains valid within its predeclared bounded deterministic family and must never be promoted to unrestricted AGI. FRONTIER-1 is independently qualified FAIL in a different stochastic environment.

## Next architecture hypothesis (not implemented)

A durable carrier-owned evidence debt / active-sensing research project should compete for action authority based on the value of another observation under uncertainty. Evidence from independent samples should update a persistent belief over latent rival causes, with stopping/commit driven by posterior confidence vs sensing cost. Prove this on a generator unrelated to FRONTIER-1 first; after architectural causal controls, freeze and test on a second independent stochastic family. Do not patch the burned FRONTIER-1 world or rerun the same seed as PASS.
