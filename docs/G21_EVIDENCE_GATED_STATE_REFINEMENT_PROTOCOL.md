# G21 — evidence-gated contextual state refinement

Status: PREREGISTERED BEFORE IMPLEMENTATION AND OUTCOME OBSERVATION
Date: 2026-10-07
Base source: 818280f34447dc16dae48d9bcc7aa3388c431f5f

## Scientific question

Can one continuing AETERNA stop destructively alternating between incompatible predictions, discover that an existing perceptual state is insufficient, and recruit two operational state distinctions using an actually observed predecessor — without being supplied context labels, a split command, or an answer table?

This is a proposed mechanism and integration experiment, not a claimed unique invention, AGI benchmark, or fresh statistical qualification. The inherited raw concept vocabulary is unchanged. What can be new during the lifetime is a distinction between two histories ending in the SAME raw observation.

## Prior work and proposed contribution

Selective memory/state splitting is established: McCallum's U-Tree work (author description: https://www.cs.cmu.edu/~mccallum/) and Learning Causal State Representations of Partially Observable Environments (https://arxiv.org/abs/1906.10437). Recent partial-observability work includes CaDiff (https://arxiv.org/abs/2512.00357) and MOSE (https://arxiv.org/abs/2605.07057). The inference principle uses established anytime-valid likelihood/e-process ideas, not an invented statistical guarantee: Safe Testing (https://arxiv.org/abs/1906.07801) and Testing exchangeability: Fork-convexity, supermartingales and e-processes (https://doi.org/10.1016/j.ijar.2021.06.017).

The candidate project contribution is the integration of post-selection evidence-gated state recruitment, actual phase-sensitive contextual cells, physical learned transition readout, and the persistent protected G20 runtime. No superiority or worldwide novelty is claimed from combining familiar ingredients.

## Representation failure and hypotheses

The old memoryless state can conflate two causal contexts. A single contradicted observation cannot establish that the world changed, that the observation was noisy, or that the representation omitted context.

For a fixed acquired base state and opaque motor, compare:
- H0: a stationary binary next-state distribution independent of the observed predecessor;
- H1: a separate binary next-state distribution for each of two observed predecessors.

The two predecessors, two possible successor cells and anchor motor are selected using a bounded factual discovery record. Selection records NEVER contribute to the subsequent evidence score.

Context is limited here to ONE preceding acquired raw state. A failed gate is not proof that no useful longer history exists.

## Mechanism

1. Native EvoPhase retains a bounded discovery record of factual (predecessor, current base state, action, next base state), with no goal/correct-action/world labels.
2. After an observed collision across different predecessors, allocate at most one candidate per base state. A lifetime has at most 16 candidates; retired slots are not reused.
3. Each candidate owns two recruited physical context cells, each receiving one acquired base-state synapse and one acquired predecessor synapse.
4. Future factual transitions teach ordinary native transition circuits FROM those context cells. The first discovery collision is excluded from this evidence.
5. Counts are read from those physical transition circuits' support metadata. No separate context-to-answer classifier/table is used for action readout.
6. Until validation, the old conflicting predictions are retained as unresolved alternatives. The acquired anchor action may be repeated as a diagnostic experiment; it is not supplied by the evaluator.
7. Promotion uses the frozen evidence gate below. Promoted contextual readout requires both learned contextual input synapses to conduct coherently, and uses the existing native goal recurrence on acquired transition circuits.
8. Unknown outgoing actions at a newly promoted contextual state may be physically explored. They are not filled in with evaluator answers or copied from a known winning route.
9. The previous observed state is factual transient memory, never a latent context label. Checkpoint restore clears that transient memory but preserves acquired distinctions; acting at an aliased state without renewed context must abstain rather than invent the missing history.

Formation/arbitration equations are programmer-defined generic rules. Phase-lesion dependence alone is not proof of full emergent oscillator computation. All adaptive representation addresses and transition knowledge for this mechanism belong to native EvoPhase, not to a host-side task solver.

## Frozen evidence gate

For two contexts c and binary outcomes o, use future-only counts n[c,o].

Q = product_c Beta(n[c,0]+1/2,n[c,1]+1/2)/Beta(1/2,1/2).
L0max = product_o (N[o]/N)^N[o], with 0^0=1.
E = Q / L0max.

Promote only when ALL hold:
- at least 32 future anchor observations;
- at least 8 observations for each context;
- at least 4 switches between observed contexts;
- absolute difference in context-conditional empirical outcome rates >=0.60;
- log(E) >= log(16/0.01);
- no third successor was observed for this binary witness.

Retire an unpromoted candidate after 128 anchor observations, or on a third successor. Unsupported third contexts are not silently mapped to an acquired context.

Why this controls optional stopping under the stated null: for every fixed stationary null parameter p, Q/L(p) is a likelihood-ratio martingale for the selected future subsequence when its context/action selection is predictable before the outcome. Since L0max >= L(p), E is pathwise bounded by that martingale. Ville's bound gives P(sup E >=1/alpha_i)<=alpha_i. Spending alpha_i=0.01/16 over at most 16 post-selection candidates gives a union bound <=0.01. This argument is conditional on the stated stationary binary null and correctly excluded selection data; it does not cover arbitrary nonstationary confounding, dependent sensor faults, malicious evidence, or causal identification from observational correlation alone.

Human Protection's safety thresholds are unrelated to this statistical alpha.

## Deterministic integrated witness

Two opaque motor permutations. Each variant uses one acquired organism and a continuous environment stream. The raw junction observation is bit-for-bit identical under two contexts. An earlier visible cue identifies the context to a system with usable history, but no context ID enters cognition.

The two relevant junction actions exchange useful/dead consequences according to the preceding cue. The other four motors do not reveal the answer. Context order is seeded and balanced in scored decisions, not inferred from alternating time/parity. Representation discovery and learning use only authorized actual interaction and actual POST observations, not a supplied training tuple list for the new distinction.

Budgets: at most 1600 warm-up physical actions per lifetime; 64 scored junction decisions per variant after learning is frozen. Raw absolute layouts at scoring differ from learning layouts. No cognition reset between learning and scoring.

Matched decision control: the SAME acquired model's unrefined/memoryless goal readout on the same current observation. This is a decision comparator, not an independently interacting agent. Always-contextual memory is a conceptual upper-bound comparator, not a claimed beaten baseline.

## Mechanism acceptance

- Two useful contextual cells are absent initially and acquired during the lifetime in each FULL variant.
- No promotion occurs before the frozen future-only evidence gate.
- FULL correct junction choices >=60/64 in each variant; memoryless comparator <=40/64.
- Both contextual consequences remain usable without alternately erasing the other context's transition.
- Necessary context-input weight lesion and pi phase shift each remove the affected context's correct decision; exact restoration recovers it without training.
- Unrelated-link intervention preserves the unaffected target decision.
- Native cognitive restart retains learned evidence/structure and requires renewed factual history. It does not clear the external protection latch.
- No learned fingerprint change occurs merely from proposing, blocking, or frozen scoring (transient observation memory excluded from learned fingerprint).
- No host context/answer table, evaluator cue label, task counter or hidden context enters production.
- 512 preregistered stationary Bernoulli null streams of up to 128 observations are checked with the SAME evidence gate at every prefix; <=3% false promotions is a finite diagnostic ceiling, not an estimate proving a universal guarantee.
- Known-context deterministic separation passes the evidence gate; uninformative contexts and single-observation contradiction do not.
- G20/Human Protection and all ordinary prior regressions plus Release build pass.

All failures are preserved; no threshold is relaxed after seeing an outcome. This stage consumes no authority-derived fresh pack. A future study must vary history lengths, changing laws, distracting histories and stochastic consequences, and compare sample/compute costs against strong established state-representation methods.
