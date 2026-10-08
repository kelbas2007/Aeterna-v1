# Representation-preserving unified fields: development result

Date: 2026-10-08
Branch: post-intel2-c-diagnosis
Verdict: DEVELOPMENT CHECK PASS — the unchanged burned-pack episodic World-C diagnostic now succeeds.

## Exact evidence

- Workflow run: 37770945107
- Job: 113290077847
- Workflow event SHA: 0661e0d2296a58401e50470f5d095d4dd3437c15
- Exact checked-out and tested source: 924a566b4d3f1125fa67e73bc1f68c4ba53bdbd3
- Artifact: refined-field-development-evidence, ID 11548441144
- Artifact ZIP SHA256: 50d72ae8610eb0328db7559bc964b490d3930927c4219695f8507f1c7f7c564e
- The final check, artifact upload and source-integrity check completed successfully.
- Test failures were not masked with continue-on-error in this workflow.

## Three preserved stages

1. Original episodic diagnostic, run 37723874139: 450 trials; the same motor on every junction visit; no context candidate or promotion. No held-out contextual score was reached.
2. Edge-cost correction only, run 37768602332: exploration covers all six motors; a context promotes; held-out score 16/32, sides 16/16 and 0/16. This remains a failed complete diagnostic and is recorded in INTEL2_EXPLORATION_EDGE_COST_RESULT.md.
3. Edge-cost plus representation-preserving fields, run 37770945107: the SAME episodic diagnostic, with unchanged thresholds, now scores 32/32, sides 16/16 and 16/16.

## Successful World-C diagnostic

- Pre-C navigation acquisition/reuse and tool-world acquisition/goal-switch assertions passed in the same persistent runtime.
- Training junction trials: 72.
- Environment terminal resets: 71; resets occur on both success and failure and preserve the organism.
- Useful context promoted: true.
- Future-only promotion observations: 32.
- Context switches at promotion: 31.
- Log evidence: 18.24776870998912.
- Unsupported-action stop: none.
- Held-out contextual decisions: 32/32.
- Predecessor-side scores: 16/16 and 16/16.
- Memoryless comparator: 16/32.
- Recruited relays: 115 before C, 166 after C, capacity 640.

The second predecessor now acquires the previously missing useful action from its own refined physical state. It is no longer evaluated using the contradictory coarse-parent prediction.

## Causal and persistence controls

A separate development control rebuilt the same A/B/C lifetime and verified:

- held-out unified readout: 32/32;
- reversing proposal enumeration preserves each chosen action;
- read-only proposal probes preserve the learned fingerprint;
- for EACH predecessor side, zeroing its required context input synapse prevents unified readout rather than exposing coarse-parent fallback;
- pi phase shift on the same link also prevents readout;
- exact restoration recovers the correct action and original fingerprint without retraining;
- missing actual predecessor history prevents a guessed contextual decision;
- cognitive restart preserves learned knowledge, requires fresh predecessor sensing, and then restores both correct decisions;
- transferred U1 meta weights remain unchanged.

## Other verification

The six-motor local-starvation controls and six remote-frontier travel controls both passed. Library tests and the selected historical non-INTEL integration targets passed, including G16-G23, U1/U2/U3, unified_runtime_integration and Human Protection. Release build passed. Historical ignored fresh tests remained ignored; no fresh pack was consumed.

Final workflow summary:

```text
EDGE_COST_CONTROLS=PASS
UNCHANGED_BURNED_C=PASS
REFINED_CAUSAL_CONTROLS=PASS
HISTORICAL_REGRESSIONS=PASS
RELEASE=PASS
```

## Production scope

Relative to frozen INTEL-2 cognition, the branch changes only:

- src/phase_drive.rs: include the current-to-successor discount in reachable epistemic value;
- src/phase_unified_cognition.rs: evaluate competing actions in the currently applicable acquired representation;
- src/phase_unified_representation.rs: collect existing coherent promoted refinements without source-class priority; abstain on required missing evidence, broken links or ambiguous distinct operational states.

No target world ID, motor answer, route, new operator, rewritten promotion threshold or new U1 weight was added. Human Protection code was not modified.

The earlier phase_drive.rs commit contains a harmless equivalent checkpoint index expression producing an unused-variable warning; it remains documented and should be cleaned before eventual integration, not silently represented as a one-line textual diff.

## Claim boundary

This is a regression/development success on an already exposed World-C pack with the previously introduced episodic terminal-reset harness. It is NOT a new independent INTEL-2 verdict and does not relabel INTEL-2 or INTEL-2R1 failures.

World D, World E and final cross-world retention were not newly qualified here. Conservative abstention on multiple distinct promoted refinements remains a limitation. Generic raw perceptual and compositional resolution was included for consistency but does not acquire new standalone qualification from this C-only development result.

## Current work handoff

Continue from branch post-intel2-c-diagnosis, not main. The exact tested cognitive snapshot is 924a566b4d3f1125fa67e73bc1f68c4ba53bdbd3; later workflow/document commits do not change cognition. The key production commits are d6510c05f80cf073d65cff8da84f4eead7e20e22, b8beb2de0669efe003fa33d8a892f02bd9817879 and 4dfee8052ebece02a96ab246c1bd8ee41b0d0f62.

main, unified-cognition, developmental-agenda and other parallel branches were not written by this work. Codex AETERNA was not modified. Compare current branch heads before any future integration; no merge or new independent system run was performed. The diagnostic/check workflows were returned to manual-only to avoid unrequested background runs.
