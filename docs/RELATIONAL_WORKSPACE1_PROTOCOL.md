# RELATIONAL-WORKSPACE-1 — identity, absence, relation and causal cue-use

2026-10-10 OPEN development experiment. NOT frozen-source
independent scientific validation; predeclared here before
real outside evaluator is executed.

## Exact scientific motivation
[Previously audited independent Farama MemoryS7](CUE_ACCESS1_RESULT.md)
showed an information-acquisition barrier and a memory-use barrier:
in 22/32 counterfactual cue pairs the native agent never observed the
starting cue; in the remaining 10/32 cue-visible pairs, changing
that object DID NOT change any selected motor or physical exit.
A language-like "same object" and causal memory cannot be proven
merely from 14/24 rewards, when an agent chooses the same
exit always.

## Fundamental intervention vs another scripted motor
One opt-in relational representation is injected as ADDITIONAL
FEATURES of the SAME native general learned action policy;
it cannot itself select any motor, path, turn, action index or
task title. It stores the first unambiguous uncommon
foreground appearance from the FIRST FACTUAL partial visual view
as an episode-local subject. The candidate foreground rule is
based on visual field rarity (an appearance occurring in <=2
visible cells), not a dictionary or list of MiniGrid object
codes. The body-relative center is reserved as an embodied
self sensor location; no key/ball/goal codes, object names,
goal coordinates or correct exit are used.

On subsequent REAL partial images, the subject survives
occlusion and each visible rare candidate is compared for
"same candidate appearance"/"different candidate appearance".
Those relational comparisons with relative sensory position
enter a 96-dimensional distributed relation feature field
ALONGSIDE the controller's raw sensory features and
temporally accumulated earlier state. Learnable action
preferences then condition on these relation features
using existing factual PRE/action/POST plus external reward.
No externally correct action label or matcher route.
An ACTUAL phase synapse powers the relational readout;
physical zero-weight lesion MUST abolish relational content.

This is not a solution to absent cue information: if the initial
frame has no unambiguous salient subject, report UNKNOWN
and DO NOT fake an identity. Exploration/looking backwards
remains open. MiniGrid's public sensor categories are ALREADY
coded, and exact type equality does not constitute raw RGB
object segmentation, human semantics, individual object
identity when two identical objects coexist, or AGI.
The foreground heuristic is explicitly programmed Rust and may
misidentify background features or fail with multiple rare
objects; test these limitations rather than hide them.

## Tests

Native structural qualification:
- two different already witnessed unique foreground cues;
- then 10 indistinguishable empty frames representing occlusion;
- identical current frame with two candidate object appearances;
- relation representation MUST distinguish which candidate
  matches previous cue WITHOUT a word or an object type name;
- absent matching candidate MUST produce an UNKNOWN/absent
  signal instead of inventing a matching object;
- relational phase link lesion MUST erase comparison output;
  restoration must recover;
- native checkpoint clears unwitnessed episode contents but
  preserves physical learned circuitry; no fictional cue.
Old native general-policy and causal episode tests run unchanged.

Externally independent exact Farama MiniGrid 3.1.0
\`MiniGrid-MemoryS7-v0\`, balanced 32 new layouts/paired
cue swaps, train seeds 150000..150127, heldout
151000..151031. Each layout gets one original episode
and one counterfactual where only the original physical cue
changes key↔ball and correct goal exit changes accordingly.
No task/position/correct-label data or experimental audit
enters the actual Rust process. The evaluator alone audits
physical cue visibility by a reversible temporary physical
swap, then RESTORES it before action. Input packet allowlist
rejects hidden metadata. Native cognition is restarted
before each frozen heldout episode to remove counterfactual
carryover and permit fair pair comparisons.

Compare source-identical:
A. generic autobiographical learner without new relation features;
B. same learner with opt-in relational comparison context.
Actual task completion, cue-exposed pairs, changed exit behavior,
pairs solved in BOTH original/swap worlds, and genuine relation
readouts are separately counted. The strongest development
claim requires >=48/64 actual rewarded episodes, >=16/32
correct BOTH cue-swapped instances, at least 16 exposed
and correct pairs, changed exits versus control, and ZERO
suspicious "both correct" in cue-unobserved pairs. Otherwise
report RELATIONAL_USE_NOT_DEMONSTRATED, even if the native
representation controls pass. A result where cue exposure
is insufficient is a valid informative FAIL; no source edits
to this consumed seed family may count as independent
qualification.

It remains possible that even a true relation representation
will NOT be useful because the organism must first actively
SEEK the missing initial information rather than travel
blind. Need independent evidence on both information
acquisition and relation-conditioned choice, not just storing
an episode or achieving majority-exit rewards.
