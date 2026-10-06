# G8 — CARRIER-OWNED INTERNAL IMAGINATION AND MULTI-STEP PLANNING

Status: **PRE-REGISTERED BEFORE G8 IMPLEMENTATION**

Date: 2026-10-06

## Question

Can one EvoPhase organism choose a multi-step delayed-reward plan by internally simulating alternative acquired transitions/macros under IMAGINED authority before executing the first physical action?

G8 tests planning ownership. It does not require natural language, arbitrary recursion or AGI.

## Starting state

G8 begins from the qualified organism substrate:
- robust relational perception;
- acquired predictive hypotheses;
- learned exploration strategy;
- acquired and revised child macros;
- acquired hierarchy;
- REAL / MODEL / IMAGINED authority firewall.

## World family

Each episode begins from a raw relational scene with at least three opaque first-step choices.

Properties:
- one action gives an immediate small factual Need reward but blocks the final goal;
- one action looks neutral but enables a two- or three-step delayed high-Need route;
- at least one action leads to a dead end;
- the correct route varies under opaque motor permutation;
- held-out absolute translations and nuisance factors remain possible.

The evaluator generates factual transitions but never supplies path labels, route length, goal distance or the correct first action.

## Acquired transition substrate

Before scored planning:
- factual tuition exposes local action-conditioned transitions;
- predictive post-traces remain MODEL authority;
- repeated successful sub-trajectories may already exist as acquired macros.

The planner may use only acquired EvoPhase-owned transition/macro state.

## IMAGINED rollout

A rollout state must live in carrier-owned IMAGINED state and contain:
- current imagined relational trace;
- candidate opaque action or acquired macro reference;
- predicted successor trace;
- accumulated predicted Need/value;
- depth/budget;
- provenance back to acquired MODEL structures.

No imagined state may write REAL.

## Search ownership

Allowed substrate:
- generic bounded expansion of carrier-owned imagined states;
- comparison of predicted cumulative Need;
- cycle/dead-end suppression;
- fixed maximum rollout depth and node budget.

Forbidden:
- host-side BFS/A* returning the best action;
- evaluator goal coordinates;
- task-specific route tables;
- hidden-state distance heuristics;
- host ranking of candidate plans.

The first physical motor must be decoded from the carrier's selected imagined trajectory.

## Controls

1. FULL_IMAGINATION
   - IMAGINED rollouts enabled.

2. NO_IMAGINATION
   - identical acquired models/macros;
   - must choose from immediate MODEL value only.

3. DEPTH1
   - imagination restricted to one transition.

4. SHUFFLED_MODEL
   - same model inventory but successor bindings consistently permuted, testing causal dependence on learned transition structure.

## Mechanism acceptance

PASS requires:

1. At least two alternative imagined trajectories coexist before physical action.
2. Selected first action is produced from an IMAGINED trajectory owned by EvoPhase.
3. At least one held-out task requires rejecting a larger immediate reward for a larger delayed factual Need.
4. FULL_IMAGINATION succeeds on >=80% of the deterministic development witness.
5. FULL_IMAGINATION strictly exceeds NO_IMAGINATION and DEPTH1.
6. SHUFFLED_MODEL materially degrades performance.
7. REAL state is byte-equivalent before and after pure imagination.
8. No host API returns a best action/route.
9. G0-G7 regressions and ownership audit remain PASS.
10. Release build PASS.
11. Rollout node/depth cost is reported.

## Fresh qualification after mechanism PASS

After source/spec freeze:
- >=80 one-use fresh worlds / >=10 sub-seeds;
- randomized opaque motor labels;
- mixed route depths 2-4;
- nuisance factors;
- report Wilson 95% CI;
- matched NO_IMAGINATION / DEPTH1 controls;
- pack burned after first observation.

## Interpretation boundary

A PASS would establish bounded carrier-owned model-based planning with delayed reward. It would not establish unrestricted theorem proving, arbitrary planning horizon, language reasoning, ARC competence or AGI.
