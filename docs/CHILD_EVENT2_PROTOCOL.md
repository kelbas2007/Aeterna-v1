# CHILD-EVENT-2 — immutable-source, branch-conditional replication

Predeclared 2026-10-10 after first source run 38068337538
but BEFORE this replication. **Exact source pinned** to
\`855b58c1d3e680509ce629b02731d78f2a453eb0\`.
No architecture weights, memory matching thresholds, reward
rule, external simulator or native code changes are allowed
to affect the new experiment.

The first original experiment yielded 14/24 reward
for episode memory against 10/24 no-memory, but
the full artifact audit invalidated that cognitive claim:
memory chose \`(5,4)\` in **all** 24 episodes, exactly the
majority desired exit (14/24); memoryless chose
\`(5,2)\` in every one. See [original audited
result](CHILD_EVENT1_RESULT.md). Therefore a
raw full-task reward threshold is INSUFFICIENT
to prove using history. We upgrade the cognitive
qualification, not the source or consumed experiment.

## Clean experiment and disqualifying checks

- Farama's independently maintained original
  \`MiniGrid-MemoryS7-v0\`, categorical partial
  observation and genuine simulator step/reward.
- Training seeds \`120000..120127\` (128), frozen
  test \`121000..121023\` (24). NONE were used by
  CHILD-EVENT-1.
- Same exact source two arms, unchanged budgets and
  independent random motor baseline:
  no episodic memory vs full positive/negative
  sequence replay. U1 and Human Protection all motors.
- The evaluator alone can inspect each final
  \`agent_pos\`, \`success_pos\`, \`failure_pos\`.
  None enter the native organism, neither in training
  nor in frozen evaluation.
- Primary cognitive criterion requires **at least one
  correct win with upper and with lower desired exits**,
  and the chosen physical exit position must vary
  across heldout worlds, proving the organism did not
  always select an unconditioned constant side.
- Additionally memory full-task successes MUST exceed
  \`max(number of upper goals, number of lower goals)\`
  = the optimal fixed-side strategy for that actual seed
  set, AND exceed identical no-memory and random,
  with >=12/24 actual simulator task rewards.
- No story about having "learned history" is allowed
  based on a high success fraction alone.

This is a necessary behavioral control against constant-side
strategies. Even if passed, it would not prove the branch
choice was caused by the remembered cue rather than
uncontrolled correlates; causal cue manipulation and
counterfactual history ablation on a different domain
would be the next scientific tests.
