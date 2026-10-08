# Exploration edge-cost development result

Date: 2026-10-08
Run: 37768602332
Candidate tested source: 01cd01bda8ccad07a8a5c81b90bfd7af3b2d167e
Production-change commit: d6510c05f80cf073d65cff8da84f4eead7e20e22
Baseline tested source: 6f754b4e97fc0a65f09321bb4eb170b3363e8eef (cognition identical to frozen 07b44fb8e00837568bc9655e760eb031d1944dff)
Candidate job: 113282261230
Baseline job: 113282260927
Candidate artifact: 11546933126; SHA256 89c2249e9bd6c855bdc95990b8a6f225f237d79b03de0c5a5a56f0027c5a8d15

## Verdict

PARTIAL DEVELOPMENT IMPROVEMENT; complete World-C diagnostic FAIL.

The new generic starvation witness fails on the exact baseline and passes on the corrected source. All six familiar-motor permutations pass after correction. All six remote-frontier travel controls also pass; exploration has not been replaced by direct-only action sampling.

## Unchanged burned World-C episodic diagnostic

Before this change, diagnostic run 37723874139 made 450 junction trials, always motor 0, with no context candidate or promotion.

With the edge-cost correction:
- all six opaque motors were sampled in the first six junction trials;
- training reached 72 trials and 71 environment resets;
- one useful contextual hypothesis promoted;
- promotion evidence: 32 future-only observations, 31 context switches, log evidence 18.24776870998912;
- no unsupported-action stop;
- recruited relay count: 115 before C, 149 after C, capacity 640;
- held-out score: 16/32;
- side scores: 16/16 and 0/16;
- memoryless comparator: 16/32.

The diagnostic FAILED the unchanged c_correct >= 28 threshold. It must not be described as World-C PASS.

This separates two problems: initial exploration starvation is repaired in the tested configurations, but the acquired contextual distinction is not yet successfully used on both sides by the unified action pipeline.

## Regressions

Library tests and every non-INTEL integration target selected by the committed workflow completed successfully, including G16-G23, U1/U2/U3, unified_runtime_integration, phase-native mechanisms and Human Protection. Historical ignored fresh qualifications were not executed and are NOT new fresh passes.

Release build passed. The final source-integrity diff was empty. The workflow's final gate failed correctly because the World-C test's actual outcome was failure.

Important CI interpretation: the World-C step used continue-on-error. A success conclusion in the steps API was not a test PASS; its outcome and full log show failure.

## Source scope

Only src/phase_drive.rs differs in production from the frozen baseline. The behavioral change charges the current-to-successor edge using the existing discount. The commit also contains a blank line and an equivalent index-variable substitution in checkpoint restoration; the latter creates an unused-variable warning and should be cleaned up before integration.

No other branch was written. main, unified-cognition and Codex AETERNA were not changed by this work. No historical INTEL verdict was overwritten and no fresh authority pack was consumed.

## Next diagnostic, not a new INTEL claim

Inspect direct contextual readout and refined-state action coverage against the coarse-parent fields used by unified competition. Determine whether useful contextual exploration or exploitation is being evaluated as if the acquired refinement did not exist. Preserve this 16/32 failure before any additional production change.

Other live repository branches observed during this work include developmental-agenda and unified-operation-competition. Do not silently merge or replace them; compare their current heads before any integration.
