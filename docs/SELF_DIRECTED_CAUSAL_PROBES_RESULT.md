# SELF-PROBE exposed cold acquisition — engineering PASS, policy benefit NEGATIVE

2026-10-19. Written **after first disclosed development trial**. Not an independent blind scientific trial.

## Evidence and source
- Development protocol before implementation: `86cccf73d0b8096510be359698ed6113a5b94758`.
- Core U1/Learner source commit: `7deca4c6b960603d3313e3b0ad0a36e0ed82b86c`.
- Test frozen before run: `ad0e631cfa18d7132d3c2f5677da797413602846`.
- Exact diagnostic run: [39313546099](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39313546099); test verdict PASS **on engineering controls**, NOT on improvement to the cold task.
- All four training arms began from a fully cold source. During each arm's **192 consecutive real protected actions**, all four primitive motors were available at every step, and native U1 decided every motor. No scripted action ID, no hidden-ready sensor, no mocked PRE/POST or externally inserted learner rows.
- Two opaque role orders were tested. Each source checkpoint restored; each holdout goal required *one additional actual progress unit* from its own last physical state, with 24 permitted actions.

## Diagnostic outcomes

| Opaque roles | Pair-probe opt-in | Self-selected probes in cold lifetime | Information probes offered | Real progress-increasing steps in acquisition | Goal solved after freezing? | Holdout actions | History goal plan offered / selected |
| --- | --- | ---: | ---: | ---: | --- | ---: | --- |
| 0,1,2,3 | OFF | 0 | 0 | 8 | YES | 2 | 2 / 1 |
| 0,1,2,3 | ON | **27** | 108 | 14 | **NO** | 24 | **24 / 0** |
| 2,0,3,1 | OFF | 0 | 0 | 6 | YES | 11 | 0 / 0 |
| 2,0,3,1 | ON | **30** | 65 | 9 | YES | 15 | 0 / 0 |
| **Total** | ON | **57** | **173** | **23** | **1/2** | **39** | **24 / 0** |
| **Total** | OFF | 0 | 0 | 14 | **2/2** | **13** | 2 / 1 |

Across both ON arms, **57 physical causal pair-probe actions were selected by native U1**, not merely offered. The environment callback returned every actual post. The agency-aware counter verified zero forced training motors. The single-use HumanProtection veto and insufficient-budget compatibility tests also passed.

## Interpretation

- **Successful engineering ability**: U1 can autonomously choose information-gathering actions during a cold unscheduled lifetime; the world model records them as genuine factual experience.
- **FAILED behavioral qualification**: the new probing option did not improve goal execution; it underperformed its cold control. In one world, U1 continuously declined an offered history-conditioned plan despite having the factual evidence. More observations and higher immediate progress during training were **not** sufficient for useful downstream control.
- This was only two disclosed role maps, with different post-acquisition underlying physical progress between arms and different learned U1 weights; it is **not** a controlled same-checkpoint causal comparison of long-term policy advantage.
- Mechanistic hypothesis to test separately: U1's five shared immediate-evidence weights cannot correctly arbitrate 'epistemic usefulness during research' against 'instrumental usefulness during goal pursuit.' We need **temporal credit and mode/phase-dependent evaluation**; do not directly tune history-proposal score constants against these now-exposed trajectories.

## Next disjoint protocol

Design an opt-in self-calibrating U1 evaluation of **completed information-acquisition → goal-solving episodes**, with explicit control over whether a probe changed an independently validated causal belief. Keep epistemic credit separate from external goal credit, using actual terminal post outcomes and a prospective gate. Require non-regression in a new, separately sourced world family. No replay of these two opaque mappings may be called 'fresh'. Maintain RSP2/HumanProtection unchanged and PR9 DRAFT.
