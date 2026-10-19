# U1 + factual Learner: one unscheduled goal / probe / action loop

Date: 2026-10-19. **DEVELOPMENT_PASS only**, not AGI or a blind independent benchmark.

## What changed

The existing Learner's selection of information-gathering actions from
*factual* PRE/action/POST events was available only in an older protected
construction runtime. The canonical ScientificRuntime instead arbitrated U1
proposals from prediction, planning, and episodic recall. These did not meet.

Now an explicitly enabled factual proposal joins the **same existing U1**
decision competition with the same five numeric fields as other candidate
sources. No new independent controller or action authority is installed.
The original HumanProtection single-use permit executes the chosen action.
Real external POST is the only teacher of transition consequences. No world
role labels, motor demonstrations or hypothetical POSTs reach the Learner.
The new factual-experience proposal is optional; old runtime behavior remains
unchanged without enablement. Learning survives the same joint checkpoint.
Already-satisfied masked goals suppress unnecessary external action.

## Actual development measurements

An evaluator-only virtual world has three scalar sensors and four opaque
motors: power, enable a gate if powered, advance if enabled, and reset the gate.
Only the evaluator knows action roles. The organism sees a goal coordinate
and post-action observations. It is never given a list of required actions.

- All **24 of 24** permutations of opaque motor roles reached the first goal
  in a bounded 96-action development run. 422 virtual actions total;
  269 final U1 choices originated from the factual-experience proposal.
- A fixed 32-action paired ablation reached **22/24** with factual-proposal
  enablement and **19/24** with unchanged U1, using identical world roles.
  These are small exposed development cases, not statistical proof of a
  transferable advantage.
- A representative run first reached the numeric goal 3 in **28 actions**.
  A second OS process restored the actual joint checkpoint and reached a new
  goal 6 in **3 more actions**. First run's U1 source choices: factual-experience
  12, predict 4, recall 9, plan 3. The second process received no action script.
- A deliberately unsafe hazard blocked the physical action callback; no
  external event was executed. A satisfied goal also issued no action.

## Reproduce

```text
cargo test --locked --test unified_self_directed_lifetime -- --nocapture
cargo build --locked --bin aeterna-v1
python scripts/unified_factual_autonomy_demo.py --binary target/debug/aeterna-v1
python scripts/project_audit.py
```

The virtual-world evaluator is a **harness**, not a hidden reasoning module.
This experiment contains no fresh unseen external task family, no learning of
operator primitives by SNN, no proof of self-created goals without a setpoint,
and no RSP2 producer-level self-improvement. It does not establish the ability
to solve arbitrary natural-language or open-world tasks.
