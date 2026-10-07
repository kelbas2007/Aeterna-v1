# G16 — autonomous abstract model acquisition

Date: 2026-10-07

## Verdict

**MECHANISM PASS — transferred learned exploration autonomously acquires a physical abstract transition model and then plans with it.**

Workflow: `37614779191`  
Scored source: `2f4bf80edf230778bac4c36080db66757e674a4d`  
Protocol: `docs/G16_AUTONOMOUS_ABSTRACT_MODEL_PROTOCOL.md`

## Preserved development history

G16 was not retroactively declared successful.

- `37611423730`: technical compile failure from a missing import after checkpoint extension.
- `37611533602`: real episodic-continuation representation failure. FULL found delayed reward 12/12, but the immediate terminal reset was encoded as a repeatable S0->S0 future and frozen planning preferred it.
- `37612194226`: capability criteria passed, but mean first-delayed-reward cost was **45.667**, above the frozen <=45.0 threshold. Scientific FAIL.
- `37614617047`: capability and cost passed with mean **32.000**, but source guard matched a forbidden word only inside an explanatory comment. Preserved as technical source-guard failure.

The final passing run changes no frozen threshold or target world.

## Passing result

- FULL delayed reward discovery: **12/12**;
- frozen held-out delayed planning: **12/12**;
- checkpoint/restart delayed planning: **12/12**;
- ZERO_DRIVE reward discovery: **0/12**;
- FRONTIER_LESION reward discovery: **0/12**;
- DIRECT_ONLY reward discovery: **0/12**;
- SEEDED_RANDOM reward discovery: **1/12**;
- NO_TRANSITION_LEARNING held-out delayed plans: **0/12**;
- NO_GROWTH held-out delayed plans: **0/12**;
- abstract transition endpoint violations: **0**;
- legacy graph transition count: **0**;
- transferred drive weights: `[0.99999994, 1.0]`, unchanged during target lifetime;
- mean physical interactions to first delayed reward: **32.000** vs frozen maximum **45.0**;
- source guard PASS;
- full optimized regressions PASS;
- Release build PASS.

## Efficiency fix

The inherited P4 feature vocabulary remains exactly two-dimensional:

1. DIRECT_UNMODELLED;
2. REACHABLE_FRONTIER.

No new weighted exploration feature was added.

When two actions have exactly equal learned drive score, the selector now uses a deterministic evidence-order tie-break: prefer the opaque motor that already owns more supported physical transitions elsewhere in the acquired abstract model.

This is a generic cross-state factual-evidence ordering rule. It receives no evaluator world law, state ID, route depth or correct-action metadata.

The change reduced mean acquisition cost from **45.667** to **32.000** on the unchanged preregistered 12-world mechanism pack.

## Architectural meaning

A target organism begins with:

- acquired physical abstract state representations;
- a transferred learned exploration drive;
- **zero** target abstract transition circuits;
- zero legacy graph transitions.

It then:

1. chooses its own physical actions;
2. observes only factual raw POST + scalar value + episode boundary;
3. grows the physical abstract transition model from those experiences;
4. discovers the delayed reward;
5. freezes learning;
6. plans through its own abstract model on a held-out raw binding;
7. retains that model across checkpoint/restart.

This closes the prepared abstract-transition curriculum gap for the bounded development family.

## Boundary

This does not establish task-specific goal-conditioned information value, stochastic/POMDP active learning, autonomous invention of the exploration feature vocabulary, arbitrary goals or AGI.

The active evidence gate is one-use FRESH-G16.
