# EXTERNAL-BLACKBOX-SMM0 — third-party stochastic motor world

Date: 2026-10-20. **Preimplementation protocol; no outcome inspected.**
The source parent is PR12's review-clean experimental commit
`a6cbd44f9a8f11b771790879a5cf0f916815f5b8`.
This protocol is isolated in its OWN new branch and must NOT change
PR12's exact-head CI, its first-use science or the present `main`.

## Independently authored black-box environment

Use the REAL Python SUL/automaton implementation from the small
open-source [DES-Lab/AALpy](https://github.com/DES-Lab/AALpy)
(MIT licence), **exact upstream source commit**
`335c50a1b4b5a830d788f86b7c30a8cb6c73ecc0`.
The source under learning is AALpy's
`aalpy.utils.BenchmarkSULs.get_faulty_coffee_machine_SMM()`, executed
through `aalpy.SULs.AutomatonSUL`, NOT a manually re-created Rust
transition table and NOT a scripted answer sequence.

It is an external **stochastic** Mealy machine with an observable
`init` → `beep` → `coffee` goal pattern and a real 10% alternative
outcome for one motor/context (per AALpy authored model).
Agent never reads hidden Python automaton states, transitions,
probabilities or raw motor-role names. The adapter maps the opaque
external motor ID to a SUL input; role map is randomized/frozen.

This is meaningful third-party source provenance, NOT a physically
external appliance, an unforeseeable secret benchmark or AGI proof.

## Prior-specific failure mode to test

The current observable empirical graph deliberately rejects a
visible `PRE/motor` edge if two actually executed outcomes differ.
That is correct for deterministic claims but stochastic worlds may
then lose every useful graph route. We must compare *with and without*
the new graph against the SAME real stochastic black-box without
changing the oracle or tuning policy to exposed outcomes.

## Frozen evaluation contract BEFORE driver/evaluator source
- Two original motors `coin` and `but` are opaque to U1; both
  genuinely available at all decision times. Explicitly match both
  permutations of input IDs. No other useful actions.
- Numeric actual observed outcome vector has width two:
  `coffee` -> [1,0], `beep` -> [0,1], `init` -> [0,0].
  **Only** [Some(1), None] is the goal; no hidden state coordinate,
  no future/privileged transition probabilities, no imagined POST.
  This conversion applies identically to both conditions.
- Two cold Rust `ScientificRuntime` arms, identical basic config,
  native U1, factual journal and protected physical actions:
  A) observable empirical graph OFF;
  B) same graph ON. Other enabled developmental options fixed
  identically, with explicit `planning_depth=4`, `planning_work=120000`.
- Each arm starts with zero PRE/action/POST records and U1 weights.
  The Python AALpy SUL starts from its own `pre()` reset and
  all actual `step(motor)` calls are externally executed ONLY after
  an independent HumanProtection permit.
- The SUL is reset at a genuine episode boundary by `pre()`;
  a trusted actual start observation is conveyed as external
  observation, without fabricating an action row. Preserve one
  learning lifetime over 24 episodes, up to 24 physical actions/episode.
- Exactly EIGHT disclosed evaluation seeds (not selected post hoc)
  derived from the frozen source commit and fixed index
  `[0xA110,0xB221,0xC332,0xD443,0xE554,0xF665,0x1776,0x2887]`
  for EACH of the two opaque action permutations; total 16 paired
  worlds × 24 episodes = 384 per arm. Reset the Python SUL random
  seed by (world_seed, episode) before each episode. Every real
  random transition outcome must be archived.
- Both programs execute genuine third-party SUL code from the
  immutable pinned GitHub SHA, as checked by a Git revision digest
  in the CI artifact. Do not pin to floating `master` or use an
  invented rewritten hidden state model.
- Strict finite run limits, process exit validation, source audit,
  source rev proof, complete real trace, actual action cost,
  checkpoint roundtrip and 0 externally forced motor actions.

## Precommitted PASS/NEGATIVE decision
A robust graph readout deserves a **narrow external-world positive**
only if ALL hold:
1. Graph ON achieves at least **5% more** observed goal-completing
   episodes than graph OFF (relative to OFF, integer ceil), or
   same goals with at least **10% fewer** real physical actions;
2. No opaque action mapping drops more than 5 successful episodes
   relative to its own paired OFF arm;
3. Total real external actions no more than 10% higher with graph;
4. No false goal, invented model transition, unprotected action or
   non-roundtrippable checkpoint, and no silent process failure.
5. The binary test asserts the verdict **after** archiving all actual
   positive and negative evidence; any failed test exits nonzero.

If criteria fail, record
`NEGATIVE_EXTERNAL_STOCHASTIC_GRAPH_TRANSFER`, no tuning on this
cohort and no relabeling as scientific one-use success.
A positive disclosed environment result does NOT demonstrate AGI,
general POMDP solution or autonomously acquired sensor apparatus.

## Active perception next, only with an external inspection action

If AALpy stochastic state transfer is qualified, follow with
third-party POMDP **Tiger/listen** and **RockSample/check** tasks
(e.g. namoshizun/PyPOMDP), in which agent must CHOOSE when a real,
costly observation is worth buying. That is a distinct protocol, not
retroactive reinterpretation of this SMM trial.

This project's user wants **actual autonomous use of learned
knowledge**, not uncontrolled growth in helper modules. The next
change must remain in the SAME protected U1/Learner runtime; no
LLM motor script, separate controller or privileged world topology.
