# HISTORY-MEMORY-1 — first ambiguity qualification failed

Date 2026-10-10. **NATIVE HISTORY-ALIAS DEVELOPMENT FAIL**.
[GitHub Actions run 38066838257](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38066838257).

The controlled native exercise presented different factual cue histories
and the SAME current binary observation. A policy with opt-in leaky
experience memory was repeatedly taught a two-step sensorimotor sequence,
with factual terminal reward. After freezing the acquired policy,
the final action was **motor 2 in BOTH histories**, failing the
pre-registered requirement that different past events yield different
decisions with identical present observation.

The first CI failure was neither a Rust compiler failure nor an
external transport issue: the assertion explicitly caught the
insufficient cognitive memory/temporal action discrimination:
`assertion left != right failed; left: 2; right: 2`.

The workflow never reached the independent Farama
`MiniGrid-MemoryS7-v0` simulation, so **there is no external
memory-game success rate from run 38066838257**. Do not record
it as 0/24 or as a tested independent environment—it is
`NOT_RUN_DUE_TO_NATIVE_FAIL`.

Architectural cause hypothesis: positive-only eligibility
traces credited frequently repeated preparatory motor 2
from later successes, rather than forming competition among
final action alternatives in different recalled contexts.
A second registered run [HISTORY-MEMORY-2](HISTORY_MEMORY2_PROTOCOL.md)
tests generic competitive signed traces on DIFFERENT external
seeds 90000/91000, without any object/task-specific action rule.
The first FAIL is preserved regardless of subsequent outcomes.

This first native test uses deliberately scripted training
examples to isolate action-history identifiability; even if
it passed, it would not alone prove autonomous developmental
world understanding. The decisive evidence must come from
full external Farama rewards under self-selected U1/HP action.
