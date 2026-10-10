# CUE-ACCESS-1 — audit information acquisition before diagnosing relational memory

2026-10-10. OPEN diagnostic, not a new intelligence capability claim.
Controller is byte-identical to commit `9e923f410b87214faa4f99ff237ef745248ac3b5`; no src/, dependency-lock or actuator-bridge changes are permitted by this workflow. Previous CHILD-EVENT 14/24 and 7/24 are preserved, including their fixed-side failure verdicts.

## Why the existing diagnosis is incomplete

Farama MiniGrid v3.1.0 MemoryEnv._gen_grid places the cue at (1, height//2-1), but randomly starts the agent anywhere from x=1 through hallway_end, looking east. A cue that was never observed cannot be remembered. The prior evaluator counted rewards and chosen exits but did not establish whether the initial cue ever reached the public image consumed by the controller. Failure to use a cue does not alone distinguish failure to seek information, failure to encode it, or failure to compare it later.

## Unmodified external controller and paired intervention

Two native arms: current generic actor, and the same actor with developmental memory + episodic recall + sequence replay. One lifetime per arm, 128 unmodified training environments (seeds 130000..130127), maximum 200 real U1/HP actions per episode. Only standard public categorical image bits and actual reward enter the agent. No cue label, cue position, mission, target branch, map or camera orientation is sent.

Evaluation: 32 NEW seeds 131000..131031. Every map is evaluated twice. The second copy differs only in the initial key/ball cue; correct and incorrect terminal positions exchange accordingly, keeping their relation to that cue consistent with original Farama task mechanics. Nothing about this intervention is told to the controller. This is an evaluator-created causal contrast using external task physics, not an untouched extra environment family.

Before EACH evaluation episode, restore native cognition from its current learned checkpoint, then freeze learning. This resets carrier clock/transient history so pair order cannot masquerade as cue sensitivity. Learned counters must remain unchanged throughout evaluation. Each pair contains exactly one upper-target and one lower-target episode; a constant-side strategy is bounded at 32/64 by construction.

## Does the cue reach perception?

Before each real decision, the EVALUATOR briefly changes the physical cue identity and regenerates the public observation, checks whether image bytes differ, and restores the original object. Both replacement and restoration are verified; no counterfactual image is sent to Rust. This directly tests whether that specific starting cue contributes information to the actual sensory input, rather than mistakenly recognizing the same object class at a branch. The real observation hash, selected opaque action and audit are stored.

An allowlist rejects any controller packet containing hidden-state or task-specific metadata. Identical complete decision-input histories must produce identical action histories after native restarts; otherwise mark INVALID due to uncontrolled state. Invisible-cue pairs with different sensory histories are also INVALID, not evidence of memory.

## Predeclared outcomes

Report separately training cue coverage, test cue-exposed pairs, identical input-history pairs, changed exit choices, correct BOTH members of a pair, actual full task rewards and constant-side bound. Minimum evidence of cue-use: 48/64 actual rewards, at least 16 pairs correctly solved in both cue variants with observed cue information, and no successes on both sides of a pair that never exposed the cue. This is stricter than merely beating random or majority prevalence.

If a pair has identical input sequences because neither trajectory observes the cue, no deterministic controller receiving only those inputs can solve both members. Diagnose missing information-seeking in those cases, not memory loss. Observed-cue pairs that still use the same exit remain evidence of inadequate encoding/retrieval/relational decision, which this audit alone does not separate.

Six independent scoring tests check constant-side rejection, paired correct choices, absent-cue classification and invalid packet-history/target contrasts. They are TEST-HARNESS checks, not cognitive achievements. Results remain open development on one new seed family; categorical MiniGrid sensing is not RGB object discovery or human child intelligence.
