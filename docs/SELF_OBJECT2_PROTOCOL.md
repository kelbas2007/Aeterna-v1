# SELF-OBJECT-2 — stricter external causal skill scorer, different seeds

2026-10-10, OPEN DEVELOPMENT preregistration before new strict scored test.

First SELF-OBJECT-1 [run 38046369141](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046369141)
had a real GitHub CI SUCCESS and reported 2 independently acquired object
affordances after 10 protected motor trials with NO correct-motor tuition;
frozen heldout 24/24 visually changed front tiles. Inspection of its raw
artifact shows that the heldout actions were precisely 12 x motor 3 for
key, and 12 x motor 5 for door, matching the actual Farama definitions.
However its scorer used just a VISUAL front tile change; rotating in place
may also change that tile, and a weak grader could count it erroneously.
This result remains unchanged and is NOT retroactively reclassified.

The second test strengthens the target from 'observed front tile changed'
to an **independent physical fact** inside the simulator:
- key: the actual specific key instance previously in the front cell is
  now actually being carried by the agent, where no object was carried before;
- door: the actual specific door instance has transitioned to is_open=true
  from is_open=false.

Only the evaluator can inspect those object identities and states, and
none are passed to the native Rust process. The native agent still
receives only the public partial 588-bit observation and real reward.

A single continuing organism explores real staged object opportunities
without demonstration of the motor or explicit language teaching;
train seeds: 44000..44009; frozen heldout: 45000..45011 for
MiniGrid-Unlock key and DoorKey locked-door conditions. Locked-door
heldout contains a matching key supplied by the examiner; its presence
is NOT taught as a prerequisite. There are at most ten protected train
experiments per category. U1 and Human Protection own the motor in
training and heldout. Native physical checkpoint and lesion controls
from SELF-OBJECT-1 remain mandatory.

OPEN DEVELOPMENT PASS requires TWO distinct learned affordances,
both grounded by actual training effect, 24/24 independently scored
physical object interactions, 0 blocked/unsupported actions,
frozen checkpoint retention, and recorded action IDs consistent
with measured real effects.

Limitations identical: **the evaluator stages object proximity and the
prerequisite held key**, visual ontology is category-coded MiniGrid,
and search uses a generic Rust least-tried motor policy.
This does NOT establish autonomous navigation, prerequisite discovery,
RGB perception or full-task solving. There is no scientific authority
PASS without a separate frozen/independently seeded gate.
