# Research note: evidence-gated operational state distinctions

Date: 2026-10-07
Associated protocol: G21_EVIDENCE_GATED_STATE_REFINEMENT_PROTOCOL.md
This note explains the proposed mechanism. It does not substitute for an executed result.

## The representational question

A recurring contradiction has several possible explanations: a changed transition law, stochastic outcomes, a bad observation, or two histories collapsed into one representation. The G19 last-fact suppression mechanism handles a deterministic change but cannot by itself establish which explanation applies.

G21 tests a deliberately narrower alternative: can ONE previous acquired observation make the next-state distribution predictable where the current acquired observation alone cannot? The distinction is learned in the ongoing protected runtime, not installed through a context-label API.

The meaning of a new contextual state is operational: base observation AND an acquired preceding observation jointly predict consequences of an action. This is not a named semantic concept, a new sensory primitive, or proof of causal identification in arbitrary environments.

## Why the old representation has a ceiling in the fixture

At the junction, the current raster and requested goal are identical under two equally represented hidden contexts. Correct actions are different. For a frozen policy using only that raster and goal, let q0 and q1 be the probabilities assigned to those two actions. Its expected correctness is (q0+q1)/2 <=1/2. Extra irrelevant actions cannot improve this bound.

An observation-history policy is not subject to that bound because the preceding visible cue differs. A fixed RNN, an always-contextual model or U-Tree could exploit such information too. Consequently, beating the memoryless comparator does NOT demonstrate superiority to those established approaches. It isolates whether the newly acquired distinction is useful in AETERNA.

The code's memoryless comparator is the SAME acquired physical model queried through the unrefined goal readout, on the same scored observation. It is not an independently interacting learning baseline. Lesions and exact restoration ask the separate question of whether the contextual input links are necessary for the new decision.

## Future-only evidence construction

A collision first selects a base cell, anchor action, two predecessor cells and two candidate successor cells from the past. Let T denote that stopping time. Data through T choose the hypothesis but are excluded from its validation.

For subsequent eligible anchor-action observations, C_i is the recognized predecessor category and Y_i is the binary successor. Counts n[c,o] are derived from native physical transition circuits recruited AFTER T.

The alternative sequential probability uses a Beta(1/2,1/2) predictive distribution independently per context:

q_i(Y_i=o | C_i=c, past) = (n[c,o]+1/2)/(n[c,0]+n[c,1]+1).

The probability of the observed ordered future string is

Q_n = product_c B(n[c,0]+1/2,n[c,1]+1/2)/B(1/2,1/2).

No binomial coefficient belongs in this expression. The implementation has a test comparing this batch expression to a product of sequential predictions.

The denominator is the best stationary context-independent binary likelihood:

L0max_n = (N0/N)^N0 (N1/N)^N1,

with 0^0=1. The monitored statistic is E_n=Q_n/L0max_n.

### Restricted null and optional-stopping argument

The guarantee assumes that, conditional on the discovery history through T and on the whole eligible past, the next eligible Y is Bernoulli(p) for one fixed p, regardless of the predictable context selection. It does not assume merely that the two groups have equal marginal means.

For any such fixed p in (0,1), M_n=Q_n/L_n(p) is a nonnegative likelihood-ratio martingale with initial value 1. Because L0max_n >= L_n(p), E_n <= M_n pathwise for every n. Ville's inequality therefore bounds the probability of EVER observing E_n >=1/alpha_i by alpha_i. The degenerate p=0 or p=1 cases do not create a two-outcome contrast under the null.

Up to 16 candidates are allowed during a lifetime, each assigned alpha_i=0.01/16. Conditional validity at each candidate's discovery stopping time followed by the union bound yields at most 0.01 probability of any false promotion among true stationary-null candidates, provided the assumptions and the future-only data contract hold. Retired candidate slots are never silently reused. Retirement on a third successor makes no claim about a more general categorical alternative.

Minimum sample counts, minimum context switches and a minimum effect size further restrict promotion; they cannot increase the crossing probability relative to the evidence threshold. They do not make the procedure robust to an arbitrarily drifting or confounded environment.

### What this argument does not guarantee

It does not provide calibrated probabilities of human harm; the statistical alpha is unrelated to Human Protection. It does not establish immunity to dependent observation noise, nonstationarity coupled to the context schedule, policy-induced confounding outside the null, unbounded adaptive feature search, or a bad abstraction encoder. The 512 null simulations are arithmetic/regression diagnostics, not a substitute for those assumptions or a universal empirical guarantee.

## Native implementation and remaining authored structure

The lifetime candidate/discovery metadata belongs to PhaseNativeState and its native checkpoint. The two operational states are recruited carrier cells. Each requires current-base and predecessor input conductance. Its action consequences are ordinary native transition circuits, not a second host answer table. The same phase-coupled goal recurrence performs readout from the contextual state.

The last observed base-state address is transient factual memory. It is NOT a latent world label. It is cleared on cognitive checkpoint and external resensing; a lone junction image after restart must not fabricate the absent history. Acquired witness metadata, physical cells and transition knowledge remain persistent.

The candidate search vocabulary (one previous observation, two contexts), evidence equation, validation threshold, diagnostic repetition policy and planning recurrence are authored generic rules. A successful lesion test shows dependence on specific modeled synapses, not that all computation has become an emergent physical oscillator process. 'Physical' here means the shared simulated carrier substrate, not a fabricated neuromorphic device.

## Prior art and novelty boundary

- Andrew McCallum, U-Tree/selective perception and hidden-state work: author overview at https://www.cs.cmu.edu/~mccallum/ . Memory distinctions driven by task demand are not new.
- Zhang et al., Learning Causal State Representations of Partially Observable Environments: https://arxiv.org/abs/1906.10437 . History-based predictive state representations are established.
- Hansen-Estruch et al., Bisimulation Makes Analogies in Goal-Conditioned Reinforcement Learning: https://arxiv.org/abs/2204.13060 . Functional goal-oriented state abstractions are established.
- Li et al., Learning Causal States Under Partial Observability and Perturbation: https://arxiv.org/abs/2512.00357 . Noise and partial observability require stronger handling than this binary fixture.
- Xu et al., Integrating Causal DAGs in Deep RL: Activating Minimal Markovian States with Multi-Order Exposure: https://arxiv.org/abs/2605.07057 . This 2026 work assumes a given longitudinal causal graph, unlike G21's narrow collision-selected predecessor test; that contrast does not prove superiority.
- Gruenwald, de Heide and Koolen, Safe Testing: https://arxiv.org/abs/1906.07801 . Anytime-valid evidence is established statistical theory.
- Ramdas, Ruf, Larsson and Koolen, Testing exchangeability: Fork-convexity, supermartingales and e-processes: https://doi.org/10.1016/j.ijar.2021.06.017 . A mixture alternative divided by null maximum likelihood and martingale domination are prior ideas.

The project-level hypothesis is that these constraints can coexist with ongoing native representation repair and mandatory protected actuation. A world-first or general-intelligence claim would require much broader independent replication, strong baselines, matched compute/experience budgets, diverse tasks and a comprehensive novelty review. None is inferred from passing this mechanism fixture.
