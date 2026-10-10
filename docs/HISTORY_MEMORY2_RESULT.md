# HISTORY-MEMORY-2 — competitive temporal learning did not solve history alias

2026-10-10, VALID **NATIVE DEVELOPMENT FAIL**.
[Original GitHub Actions run 38067087802](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38067087802).

The experimental source changed general (task-agnostic) eligibility
traces to reward selected motors and inhibit alternatives at the
same factual earlier context. No object/motor/world-specific
selection rules were introduced.

The exact native causal gate still failed:
two different prior observed cues, the SAME currently visible
binary frame, and two distinct factually rewarded final
training actions led to motor 2 in BOTH frozen history conditions
(`left: 2; right: 2`). Therefore the key test of history-sensitive
decision competence failed and the Farama external MemoryS7 stages
were NEVER RUN by this workflow, being skipped on native failure.

Do not interpret the green GitHub environment-install steps as
a cognitive PASS. The signed temporal plasticity update cannot
by itself produce a reliable usable recalled event or prevent a
frequently rewarded preparatory motor from dominating final
choice. The follow-up fundamentally introduces episodic
factually successful event-context binding rather than another
MiniGrid navigation/door heuristic.

Reproducible first source failure remains permanent. External
source-implemented context recall must be demonstrated separately.
