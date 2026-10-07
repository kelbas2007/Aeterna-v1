# INTEL-1 Repair-4 result

Date: 2026-10-07

Verdict: **PASS — bounded competing contextual hypotheses no longer block later continual context learning.**

Workflow: `37667787912`
Exact tested source: `b811b59a5777b59b4ab640ef4ac2ebf75a53c45d`.

## Repair witness

One persistent organism first acquired an unrelated, nonpromoted contextual candidate on the same physical base state. Without clearing memory, later factual experience created a second structurally distinct contextual hypothesis.

Observed target witness:
- future validation observations: **32**;
- context switches: **31**;
- log evidence: **18.24776871**;
- promoted: **true**.

Both hypotheses remained structurally present on the same base state. The old unrelated candidate remained nonpromoted and was not deleted.

Held-out target contextual score:
- FULL: **64/64**;
- old-candidate-only control: **0/64**.

Repair-3 fanout regression also passed:
- one external fact -> shared parent support exactly **1**;
- all-refiners history witness promoted and scored **64/64**.

Repair-1, Repair-2, G21, G22, G23, G20, Human Protection and Release build all passed.

## Scope

Repair-4 only generalizes G21 hypothesis management:
- multiple bounded contextual hypotheses may coexist on one base cell;
- identity is factual structure, not world/task ID;
- readout ignores hypotheses incompatible with the current predecessor;
- competing promoted hypotheses fail closed rather than being resolved by host answer.

No G24/G25 mechanism family was added.

Earlier INTEL-1/R1/R2/R3 verdicts remain failed and their authority packs remain burned.
