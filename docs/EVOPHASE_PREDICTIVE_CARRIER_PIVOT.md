# EvoPhase fundamental research decision after GENERAL-POLICY-1/2

Date: 2026-10-10. **ARCHITECTURE PIVOT — do not merge the current learned
replacement into main as a demonstrated general intelligence.**

## Decisive controlled evidence

The architecture-wide controller replacement \`PhaseGeneralPolicy\`
bypassed every previous handwritten object, navigation, goal-rule
and factor-action selector and used only distributed active sensory
features, action-conditioned prediction error, real reward and
temporal credit. Physical motor links, U1 and Human Protection
remained binding. Its native logic/physical checkpoint, frozen
weights and synapse lesion controls PASSED.

Original real Farama MiniGrid heldout:
[run 38053197738](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38053197738)
→ general learned **9/16**, manual authored 8/16, random 2/16.
3/4 FULL DoorKey solves, 4/4 Empty, 2/4 Unlock, 0/4 MultiRoom.

**Independent immutable-source replication:**
[run 38053677397](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38053677397)
built the **exact same source commit
\`58d204089eff50a36793397bad3407f30b249e56\`**,
using fresh train/heldout seeds. General learned **2/16**,
authored 4/16, random 2/16, with 2/4 FULL MultiRoom
but 0/4 DoorKey/Empty/Unlock.
The original superiority did NOT replicate.
[Both detailed reports](GENERAL_POLICY1_RESULT.md) and
[second result](GENERAL_POLICY2_RESULT.md).

These are actual independent externally executed complete tasks,
not synthetic in-repo goal transitions. The simulations nevertheless
share a pre-categorized MiniGrid observation ontology and
the architecture uses authored Rust plasticity/hashing mechanisms.
**No AGI or robust learned common intelligence is demonstrated.**

## A fundamental, falsifiable diagnosis

Current general policy maps \`raw visible frame → signed 96D feature
hash → motor scores\`. Eligibility traces distribute delayed
rewards BACKWARD but do NOT encode belief for later forward
decisions. Episodic \`recent[(frame_hash,motor)]\` only suppresses
repeated motors for identical frames; it cannot bind remembered
identity, inventory, intended object, location or previously
seen cue to a latent, persistent history-dependent world state.

Two distinct histories can lead to the same camera image but demand
different future actions. The replacement has no expressive
learned state representation for this distinction. This is an
architectural restriction, not a missing MiniGrid game rule or
a deficiency that an extra hardcoded "door" mechanism should repair.

## Discard incremental game-specific patches

Do NOT grow another set of locally optimized object/door/navigation
rules or tune source to consumed seed sets 62000/63000, 66000/67000.
The strongest next candidate is to change what "thinking state"
means, replacing frame-reactive decision making with ONE
history-dependent predictive carrier:
- \`z_{t+1} = F_theta(z_t, o_{t+1}, a_t, prediction_error_t)\`:
  SNN/phase or HDC-supported persistent latent causal state,
  **not just the current observation**, with write/read/forget
  controlled by factual predictive evidence.
- Predict the next observation and reward from \`z_t, a_t\`,
  train from **actual environment POST and reward**. Updating
  recurrent state must be useful even with zero terminal reward.
- Preserve an object or prerequisite through disappearance,
  rotation, delay, room transitions and distractors, while
  distinguishing context changes with identical visible images.
- Learn **temporal transition fragments / macro-operations**
  from repeated successful experience rather than shipping
  module-specific enumerated programs. Support credit through
  uncertainty, contradiction, and counterfactual prediction.
- Choose external motors through the same single U1/HP boundary;
  no privileged simulator map/carrying/agent coordinates and no
  hidden task scripts.

The geometric/actor/value learning rules themselves are not to
be considered emergent just because their parameters adapt.
A claim of native SNN cognition needs lesion/ablation proof that
the recurrent physical carrier IS causally necessary.

## New independent gate before touching benchmark heuristics

1. **History ambiguity:** two externally generated episodes have
   identical current visible observation yet DIFFERENT correct
   action because of an earlier cue. Successful frozen decisions
   must vary correctly by history. Matched no-memory and
   shuffled-history ablations must fail. The examiner supplies
   neither correct actions during learning nor latent state labels.
2. **Cross-environment transfer:** same frozen recurrent mechanism
   must solve unfamiliar observation/action compositions in
   at least TWO independently authored environment families,
   ideally including a non-MiniGrid domain. Do not count shared
   category IDs as raw visual abstraction.
3. **Whole-task completion:** source-sealed fresh external
   DoorKey and MultiRoom goal rewards MUST exceed both
   authored and random matched controls, repeated on at least
   two independent frozen seed families with no in-between
   source changes.
4. **Representation evidence:** timestamped prediction-error
   decrease on heldout sequences, correct belief update after
   intervention and contradiction, and necessary physical
   recurrence ablation.
5. **Budget/implementation:** CPU-first implementation appropriate
   for a 16GB laptop; avoid trying to reproduce a 317M-parameter
   model just to add perceived generality.

Relevant public research showing why this is a credible direction:
- [DreamerV3, Nature 2025](https://www.nature.com/articles/s41586-025-08744-2):
  recurrent learned world dynamics and imagined outcomes across
  heterogeneous tasks, not a fixed per-task route planner.
- [TD-MPC2, ICLR 2024](https://proceedings.iclr.cc/paper_files/paper/2024/hash/cf73d57b6dcda32b293df7c2d5341f49-Abstract-Conference.html):
  learned implicit latent model for planning across domains.
- [Compositional Planning with Jumpy World Models, ICML 2026](https://proceedings.mlr.press/v306/farebrother26a.html):
  predictive multi-timescale temporal abstraction and policy
  composition rather than hardwired specific object routines.

These publications are architectural precedents, NOT evidence
AETERNA has implemented their capabilities.
