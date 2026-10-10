# VALUE-RULE-TRANSFER-1 — preregistered open development

2026-10-10. This protocol precedes the new evaluation. It is not an AGI or
EvoPhase ownership qualification. Historical CONTEXT-VALUE-1 and failed S9
transfer remain unchanged. Development seeds 191000/192000 are consumed.

## Mechanism

Opt-in CART predicate induction distils the best *visited* actions of positive
MODEL values acquired from factual PRE/action/POST/reward. It never receives
correct actions or map labels. Features are the current public binary frame,
first factual frame and the existing workspace's factual encounter contents.
No fixed coordinates or motor roles are selected by the learner's constructor.
Rules have at most 255 nodes/depth 12; uncertain leaves abstain. Exact known
states keep their original decisions. Only frozen unknown states use rules.
Training decisions therefore do not gain a hand-written exploration schedule.
The access relay is physical; CART, values and predicates remain authored
software computations, not full EvoPhase ownership or primitive invention.

## Frozen cross-task transfer

One cold life trains only on Farama MemoryS7, seeds 194000..194511, maximum
200 actions/episode. Freeze the source and binary before evaluating:

- MemoryS7 retention: 32 original/swapped pairs, seeds 195000..195031.
- MemoryS9: 32 original/swapped pairs, seeds 196000..196031, no retraining.
- **Different task:** Empty8 seeds 207000..207031 and Empty16 seeds
  207032..207063, using Farama's `agent_start_pos=None` to randomize real
  initial position/direction. Goal navigation has no remembered cue. No map,
  mission, goal coordinates, category meanings or motor semantics enter Rust.

Restart before each evaluation. Compare acquired rules with the same learned
life after physically cutting only rule access, a cold untrained instance,
random actions, and an explicit simple authored turn-at-wall controller. All
receive the same starts and 200-action caps; report actual expenditures.

Primary cross-task DEVELOPMENT PASS requires >=48/64 navigation successes,
>=20/32 at **each** size, and >=16 additional successes over *each* of the
lesioned/cold/random controls. Retention requires >=48/64 MemoryS7 successes.
Report the authored controller without implying it is beaten. All persistent
metadata must remain frozen. S9 is separately judged by >=48/64 rewards,
>=16/32 both-correct exposed pairs, zero unexposed both-correct, and >=8 more
both-correct pairs than the lesioned control. An S9 failure must remain FAIL
even if cross-task navigation succeeds. Report raw-frame novelty relative to
training, unique initial states and counterfactual visibility audits.

## Recorded measurements, separate acquisition

The **same Rust constructor** also learns from real recorded 8x8 UCI/sklearn
digits and UCR GunPoint trajectories. These are two separate cold lives;
do not call separate training image-to-signal knowledge transfer. Existing
fixtures/test splits are public and consumed; this is open recorded-data
validation, not independent qualification or real camera/robot testing.

Digits: the existing per-class every-fifth-record test split (364 images);
shuffle training with seed 198000 and select 32 images/class (320 total).
Lossless five-bit serialization of each measured 0..16 pixel, 320 sensors.
Twelve passes, at most 20 self-chosen classification attempts per image/pass.
GunPoint: the official 50 training/150 test recordings; retain all 150
coordinates, fixed clipped [-4,4] six-bit measurement serialization, 900
sensors. Sixteen passes, eight attempts per record/pass. Class-to-motor roles
are shuffled with seed 198001 and remain evaluator-only. A trial receives
only actual success reward. Native acquisition is unchanged between corpora.

Frozen one-action classification compares rules with a physical rule-access
lesion and 3-NN using only records/actions identified by actual rewarded
training attempts. Report actual training actions. Recorded-data gates:
digits >=75%, GunPoint >=80%, and >=20 percentage points over each corpus's
lesioned control. Report 3-NN without claiming superiority unless measured.
Save acquired predicate programs, source/binary/fixture hashes and raw actions.

Do not tune or change the frozen cognition after these test outcomes and
rescore the same sample as fresh evidence. CI exercises native mechanisms;
external evaluations are manual and preserve negative results.
