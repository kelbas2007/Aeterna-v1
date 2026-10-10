# EXTERNAL-WORLD-2 — generic exploration under real partial observations

Date: 2026-10-10. OPEN DEVELOPMENT diagnostic; no frozen authority.

First external benchmark source run [38033397140](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38033397140)
was validly executed (protected motors 12/12 episodes, nonzero actual steps)
but trained 0/8 on each and heldout 0/12 versus random 2/12.
All environments had 32 factor rules and zero positive rewards.
The preceding run [38033213634](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38033213634)
was INVALID, because an absent U2 caused NoSupportedAction, zero motor executions;
a unit test now prevents that transport mistake.

Development adjustment: when there is no learned reward goal,
replace original optimistic BFS tie that systematically preferred a
repeated opaque motor with a bounded carrier-observation-driven score
using local state/action coverage, whole-lifetime action coverage,
observed non-noop rate, imagined novel successor and deterministic
tick/sensory tie breaking. No action ID semantics or map properties.
U1 and Human Protection are unchanged. This is still a handcrafted
generic Rust exploration policy, NOT learning an intrinsic neural search.

The next run uses separately reserved training seeds 1100–1107
and frozen evaluation seeds 1900–1903 on each of three independent
Farama environments, 64 steps/episode, paired seeded random baseline.
Goal success is actual positive external terminal reward. Do not label
the result PASS unless independently qualified with a different
fixed preregistered source/seed/control design; record even a zero.
