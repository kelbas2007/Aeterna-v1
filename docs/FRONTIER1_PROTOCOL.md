# FRONTIER-1 — noisy partial-observation temporal integration

Status: PREREGISTERED, BEFORE NEW EVALUATOR OR TARGET INTERACTION
Date: 2026-10-08
Branch: `research/beyond-intel4`
Cognitive baseline: frozen INTEL-4 production SHA `c7b5455ba006b297288fa8d16ef6300c8a19ceca`.

## Why this is not another A–E permutation

INTEL-4 demonstrated one persistent EvoPhase resolving deterministic transport, ordered skills, immediate predecessor context, one bounded AND/XOR perceptual program and a factual law change.

FRONTIER-1 tests a **qualitatively different requirement**: a hidden binary cause produces noisy contradictory sensory readings. The last visible frame is informative but insufficient. An organism can pay for repeated active observations before committing to one of two actions. This requires temporal evidence integration and control of the *value of additional observations* under uncertainty, not simply one-step predecessor identity or a known transition graph.

## Prior

Allowed before seal: generic 24 abstract raw visual state classes and generic learned exploration-drive and meta-control checkpoints. No latent side/action mapping, cue reliability table or world transitions are preloaded.

## New environment (hidden from cognition)

- Authority run seed = fresh GitHub `run_id`, checked after all pre-seal gates, printed at `FRONTIER1_SEAL`.
- Independently randomize distinct sensory raw class roles for two noisy cue frames, goal and terminal failure, plus three opaque motors: `sample`, `commit-left`, `commit-right`. Other motors give ordinary nonterminal factual observations.
- At each episode start choose a balanced hidden side (40/40 per evaluation block); one raw cue is emitted. Every cue independently matches the hidden side with probability 0.7. The episode can offer up to 7 additional independent samples, with no label identifying the side.
- `sample` produces the next noisy raw factual frame and has no task reward; other noncommit motors do not reveal extra information.
- `commit-left/right` terminates with factual goal/dead solely according to hidden side; success reward is the actual task outcome, never a teacher action.
- External episodic reset emits a new genuine raw frame and new hidden cause. EvoPhase/its checkpoint are *not* reset between episodes; no host-inserted terminal transition.
- Max 10 motor actions per episode. Unknown state/action mapping is not supplied to EvoPhase. Human Protection screens every physical action through the normal protected runtime.

## Qualification

- Cold one-organism acquisition on 320 balanced hidden episodes, no target transition pretraining, at most 10 actions per episode.
- Freeze target model/representation learning; heldout 80 balanced episodes with unseen actual cue-noise sequences, at most 10 actions each.
- Primary criterion: at least **64/80** correct committed outcomes (noncommit / unsupported = incorrect).
- Baseline on the same episodes: best single **last** noisy cue mapping (computed by a fully informed external scorer) must have no more than **60/80** correct. A constant policy is exactly 40/80 under balance.
- Record fraction of episodes with at least 2 active samples, mean total actions, absolute success and a full-history 8-cue majority oracle. No source-code/API introspection by cognition.
- A positive result also requires a measurable **relative advantage over last-cue oracle** and no knowledge leak, continued Human Protection, no legacy graph or answer table.
- Every failure, early missing action and expensive sample loop is a scored FAIL. No evaluator exception may be silently converted to success.

Any run after seal is burned. Report PASS/FAIL/INVALID separately with full raw metric output and exact frozen cognition integrity. Even PASS proves only noisy binary latent evidence integration in a bounded environment, not unrestricted AGI.

## Scientific boundaries

The evaluator knows hidden causes; EvoPhase receives only sensory observations, raw goal image, actual task outcome and safety evidence. Exact self-chosen action history may affect cognition through its own substrate. The hidden state, correct motor, cue trustworthiness and random seed must never be injected into a model call.

No subsequent run with altered thresholds or repaired target-specific code may qualify against the same burned authority pack.
