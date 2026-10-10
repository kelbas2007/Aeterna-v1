# UNSTAGED-WORLD-2 — general cognitive repetition arbitration

Date 2026-10-10. OPEN DEVELOPMENT. Distinct seed family, not scientific
source-frozen authority. The first UNSTAGED-WORLD-1
[run 38046685780](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046685780)
is preserved: train goals 0/36, frozen heldout goals 0/12
versus random 1/12, 3 acquired local category motor links,
191 real key pickups in heldout, **0 moves** and 0 doors opened.
Those counts show that locally rewarded physical manipulation can
monopolize motor control and prevent exploration.

No environment-specific navigation procedure, action labels or
map data are added. The only generic change in the native carrier
is a small transient *episode history* of factual visible
PRE→selected motor pairs. If a qualified motor was already executed
from the identical factual state in that same episode, the body
self-manipulation proposal must stop monopolizing U1. Other motors
may then be investigated through the existing factor exploration.
On a new real episode, the repetition history is cleared;
acquired category→motor phase synapses persist. The episode
history is not treated as learned knowledge: removed from
checkpoint and knowledge fingerprint. The history is updated
even under frozen learning because experiencing another action
is a factual transient observation, not a model parameter update.

The same independent Farama MiniGrid 3.1.0 DoorKey, MultiRoom, Unlock
physics; one organism across 12 interleaved training episodes
per family, then restart and frozen learning on 4 heldout
episodes/family. New train seeds **52000..52011** and heldout
**53000..53003** are not reused from the negative result. Full
worlds are truly UNSTAGED: no object placement, no carried key
insertions, no supervised motor, no spoken instruction, no
simulator map, only public partial observation and real reward.
128 U1/HP-screened external motor actions per episode.
Paired seeded random motor policy gets equal heldout budget.

The strict/open goal criterion from UNSTAGED-WORLD-1 is preserved:
>=6/12 full-task heldout terminal rewards and strictly > random,
plus >=1 actual key pickup and >=1 opened door in heldout.
We additionally report ACTUAL agent position changes
(motor movement) and observed object effects as diagnostic
outcomes, not replacements for task success. A PASS is NOT to be
declared based on movement alone. Any continued zero-goal or
stuck behavior must be preserved as FAIL.

Even if PASS: categorical MiniGrid observation and handwritten
Rust exploration are not spontaneous natural language/RGB AGI.
