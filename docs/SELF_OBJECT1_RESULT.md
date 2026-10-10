# SELF-OBJECT-1 — first autonomously selected opaque external object action

Date 2026-10-10. OPEN DEVELOPMENT PASS for isolated self-discovery
of a nearby object motor, NOT for complete autonomous task solving.
[Original Actions run 38046369141](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046369141);
fixed source commit \`16ce7a42cb6c02e3ca9b52f673411681badd27e0\`.

The same EvoPhase organism learned how to manipulate two visual
objects from the **actual PRE/action/POST of its own protected U1
motor choices**, not from an examiner-supplied motor demonstration:
key from DoorKey-5x5 and door from MultiRoom-N2-S4. The examiner
staged the item immediately in front, but did NOT teach object
type labels/words, motor ID or effect meaning to EvoPhase.
Only the body-relative front tile coordinate (26) was specified
as generic sensor/embodiment geometry.

The original raw log:
\`SELF_AFFORDANCE1 train_actions=10 motor_tuition=0 learned=2 frozen_interactions=24/24 blocked=0 goal_rewards=0\`.
\`SELF_AFFORDANCE1_SUMMARY verdict=DEVELOPMENT_PASS\`.
Inspection of the uploaded JSON explicitly found 4 cold probes
for the key and 6 for the door. After probe actions 0,1,2,
the key was learned on motor **3**; after 0..4, the door was
learned on motor **5**. Heldout actions on new MiniGrid-Unlock
and DoorKey were exactly **3 in 12/12 key trials** and
**5 in 12/12 door trials**.
Two self-acquired causal motor synapses were preserved through
native checkpoint/restart and exploited under frozen learning.
A native 2/2 causal test verified first cold skill, physical
synapse lesion/restore, stable category recognition and
U1/Human Protection execution.

**First external grader limitation**: \`24/24\` originally used
visible front-cell change, which can occur due to a camera turn
even without successful object manipulation. Do NOT use this
metric alone for claims about physical causal success.
A second differently seeded \`SELF-OBJECT-2\` test uses
independent simulator actual key carrying / door opening
ground-truth and addresses this confound. Preserve this
original run and its complete first results.

This is not autonomous navigation or full goal completion.
The examiner externally positioned each object and provided
an already carried matching key at the locked door in heldout.
No complete task terminal reward was observed. MiniGrid supplies
symbolic categorical tile types, not raw RGB pixels, and the
motor-probing logic is generic handwritten Rust.
