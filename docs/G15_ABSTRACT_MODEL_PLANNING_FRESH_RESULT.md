# FRESH-G15 — abstract phase-native model-based planning qualification

Date: 2026-10-07

## Verdict

**PASS — one-use fresh statistical qualification of bounded abstract model-based planning.**

Workflow: `37609743908`  
Source: `bebd479389bad814093a459c59f60df691442181`  
Spec blob SHA: `89589201d28b078bb326c41be7f53c7d08ab59a1`  
Authority seed: `37609743908`  
Burned pack digest: `84dffac8bcb36171`

The authority-derived pack was printed before transition tuition/scoring. Run attempt was 1.

## Fresh result

10 independent sub-seeds x 8 held-out start bindings = N=80.

Observed:

- FULL_ABSTRACT_PLANNING: **80/80**;
- Wilson 95% CI: **[0.954182,1.000000]**;
- every sub-seed: **8/8**;
- selected-depth violations: **0**;
- DEPTH1 delayed branch: **0/80**;
- NO_ABSTRACT_MODEL delayed branch: **0/80**;
- BROKEN_ABSTRACT_STATE: **0/20**;
- BROKEN_TRANSITION: **0/20**;
- PI_PHASE_TRANSITION: **0/20**;
- exact RESTORE without retraining: **20/20**;
- IRRELEVANT_LESION preserved target: **10/10**;
- structure violations: **0**;
- abstract transition endpoint violations: **0**;
- REAL mutations during planning: **0**;
- learned-fingerprint mutations during planning: **0**;
- legacy graph-table violations: **0**;
- motor-role mask: **0b111111**;
- geometry rejection count: **0**;
- full optimized regressions PASS;
- Release build PASS.

Fresh route lengths included 2, 3 and 4 transitions after the first delayed action. Immediate distractor values ranged from 0.508 to 0.683. Every fresh organism learned exactly 12 physical abstract transition circuits, two opaque planning-action outcomes for each of six acquired abstract states.

## Architectural meaning

The fresh run statistically qualifies the bounded causal chain:

```text
raw held-out observation
 -> physically active acquired abstraction
 -> learned physical abstract transition
 -> future acquired abstraction
 -> further learned transition(s)
 -> delayed factual value
 -> local backward phase-native value propagation
 -> opaque motor choice
```

The transition model was learned only on a separate tuition binding and was not relearned on held-out raw bindings.

Depth-1 never selected the delayed branch, while full physical propagation selected it on all 80 held-out starts. Destroying either the physical abstract state path or the necessary abstract transition eliminated the targeted delayed choice; exact synapse restoration restored it without relearning.

## Boundary

This establishes bounded abstract model-based planning across fresh delayed chains.

It does not establish:
- autonomous acquisition of the missing abstract transition model;
- autonomous goal invention;
- unrestricted stochastic planning;
- arbitrary program search;
- natural-language reasoning;
- AGI or consciousness.

The next architectural gap is **goal-directed autonomous abstract model acquisition**: the organism must discover which missing abstract transitions need physical interaction, gather them itself, and then plan without being handed a complete transition curriculum.
