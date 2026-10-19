# AGENCY-CREDIT1 — policy credit for self-selected actions only

Date: 2026-10-19. Pre-implementation opt-in protocol. **No independent intelligence claim.**

## Trigger and diagnosis
The exposed diagnostic on PR9 commit `73004cf8` compared training with externally forced motor offers versus no policy-weight updates during those forced experiences. The organism acquired the same REAL PRE/action/POST experience, but its U1 goal-value/policy-credit synapses were otherwise changed by actions it had no control over. This conflates identification and agency.

The descriptive 24 motor permutations yielded (solved/tasks, actions):
- forced policy credit, no history: 18/24, 351
- forced policy credit, history enabled: 18/24, 339
- manually suppress forced policy credit, no history: 17/24, 346
- manually suppress forced policy credit, history: 20/24, 232

This cohort is EXPOSED and cannot be reused as a blind test of improvement.

## Research intervention
Enable one **opt-in** policy-credit rule before collecting any constructing observations. The organism itself reads the trusted effective allowed-action mask in each actual step. When fewer than two feasible primitive actions were genuinely available, there was no action choice, so the native U1 meta-utility projection must **not** be updated by the executed outcome. No separate runtime, score program or safety bypass is introduced.

Preserve WITHOUT modification: physical execution via the original HumanProtection one-use permit; actual world model PRE/action/POST journal; native observation readout; consequence learning; checkpoint/restoration; safety denial/fault. When at least two feasible actions were offered, the same U1 selected-action reward and helper-credit updates apply as before.

A forced training transition MUST NOT be silently reclassified as a 'self-directed choice' simply because the U1 cognitive helper or motor received a score. Meta credit is based on externally feasible motor alternatives, not a heuristic reward threshold or invented world roles.

## Falsifiable engineering checks
1. With policy-credit opt-in and only ONE allowed motor, repeated actual successful protected transitions must preserve initial native U1 meta weights and meta observation count while changing the factual journal, and exact joint checkpoint must restore.
2. Without this opt-in, the same forced experience must continue to update native U1 meta-weights; this is a compatibility control.
3. With at least TWO feasible offers, an actually selected protected action must again increase native meta observations and update the weights.
4. HumanProtection veto never performs the callback or advances factual/meta counts.
5. The opt-in is a one-way choice before experience; a restored preexisting checkpoint cannot silently enable it later, and a frozen mode cannot rewrite the credit policy.

## Scientific boundary
The exposed 24-permutation demonstration is useful mechanistic diagnosis, NOT evidence of out-of-distribution policy superiority. An independent, disjoint causal-world test must be preregistered and run once after source freeze. No source tuning on exposed worlds can become a generalization claim. Actual multi-step autonomy and new-world transfer must be measured separately from mere native-score changes.

PR9 stays DRAFT. main, existing RSP2 admission and HumanProtection stay unchanged.
