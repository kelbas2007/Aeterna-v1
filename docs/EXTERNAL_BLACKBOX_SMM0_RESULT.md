# EXTERNAL-BLACKBOX-SMM0 — independently AUTHORED stochastic SUL, NEGATIVE graph transfer

**Developer-disclosed research. Actual frozen verdict:
`NEGATIVE_EXTERNAL_STOCHASTIC_GRAPH_TRANSFER`.**
The exact protocol [EXTERNAL_BLACKBOX_SMM0_PROTOCOL.md](EXTERNAL_BLACKBOX_SMM0_PROTOCOL.md)
was committed `a91db90a38257d6fd8a9c7e7150fc4a10147c63d`
**before** the evaluator/adapter source
`d8441048bf1be8d7d9e9fb278a17cfb6cd64f96c`.

[First actual pinned GitHub SUL execution **39431269294**](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39431269294).
The Linux runner installed and EXECUTED the genuine MIT-licensed
[DES-Lab/AALpy](https://github.com/DES-Lab/AALpy) source at
`335c50a1b4b5a830d788f86b7c30a8cb6c73ecc0`,
`get_faulty_coffee_machine_SMM()` wrapped by `AutomatonSUL`.
The system under learning was a real live subprocess; no hardcoded
Rust state transitions or training motor schedule. It has stochastic
transitions, including an unpredictable 10% failed button outcome.

**The software ran and produced genuine results.** Rust compilation,
Python upstream checkout/import and 16×24 episode execution succeeded;
the test exited nonzero **at the deliberately preregistered
post-result acceptance assertion**. That CI FAILURE is scientific
negative evidence, NOT a technical setup failure.
[Full real physical-action and external output traces](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39431269294/artifacts/12268861160)
archived before the assertion.

| Same pinned black-box goal episodes | Native U1 without graph | Same U1 with empirical graph |
|---|---:|---:|
| Actual goals reached | **383/384** | **384/384** |
| Actually executed external actions | **1,689** | **1,590** |
| Completed goals when motor roles as first map | 191/192 | **192/192** |
| Completed goals when roles swapped | 192/192 | 192/192 |
| Native chosen graph actions in candidate | — | **132** |
| First-six goals across 16 cold lifetimes | 96/96 | 96/96 |
| Last-six goals across 16 cold lifetimes | 95/96 | 96/96 |

The candidate has 1 more actual success and saves 99 actions
(5.86%) but **FAILS the precommitted threshold**: at least 5%
relative goal improvement, OR exact equal goal successes with
10% action savings, plus mapping retention/cost guards.
Per-map nonregression and ≤10% extra cost both passed.

## Interpretation: saturation of this external task

The *baseline* already solved **99.74%** of episodes, leaving only
one unsolved goal. Thus the present source-disclosed "faulty coffee
machine" is TOO EASY to provide evidence of robust external
generalization with these policies and action horizons. The 10%
transition noise does not imply that the organism learned a stochastic
belief model: the observable output `beep` practically reveals
its preparation. Under these conditions repeated ordinary attempts
suffice.

Do NOT:
- call this POSITIVE (it failed preregistered gates);
- alter the eight seeds, horizon, world, criteria or partial outcome
  data post hoc to obtain a favorable score;
- claim this is a genuine physical appliance, a secret AGI exam or
  independent learned model of stochastic dynamics;
- merge an intentionally failing scientific-criterion test into
  normal `main` regression without proper ignored/labeled handling.

**Next falsifiable third-party task:** an external **Tiger/listen**
or **RockSample/check** POMDP where a costly sensory query must be
selected to distinguish genuinely ambiguous latent states. It has
meaningful uncertainty and action costs, unlike the almost solved
AALpy coffee task. Preregister a NEW, disjoint externally authored
world and seeds before implementing any adapter; do not reuse this
cohort to pick thresholds. Keep the same native U1, factual journal
and HumanProtection permits on every external step.
