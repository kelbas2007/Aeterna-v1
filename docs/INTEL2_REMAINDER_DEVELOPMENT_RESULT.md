# INTEL-2 remainder development result

Date: 2026-10-08
Branch: post-intel2-c-diagnosis
Verdict: DEVELOPMENT_CHAIN_FAIL. This is an already exposed development pack, NOT a fresh INTEL-2 qualification.

## Exact evidence

- Workflow: 37785959666.
- Job: 113340644292.
- Workflow event commit: 9cc6a9c93bebceb962dd93ac290e61fe007f52fb.
- Exact checked-out evaluator/source: 624aa1c835440244a32c407b621aa266b8435dd2.
- Cognitive baseline: 924a566b4d3f1125fa67e73bc1f68c4ba53bdbd3.
- Original burned authority seed: 37687243350.
- Artifact: intel2-remainder-development-evidence, ID 11554117373.
- Artifact ZIP SHA256: 97f1f07b71cfc3e3193ec77bd3284325d135a7c78a542e63e2daa79a809abb01.
- Cognitive source and historical-test comparison: PASS.
- Explicit six-motor evaluator unit test: 1 passed, 0 failed.
- Explicit one-organism development test: 0 passed, 1 failed, complete REMAINDER_SUMMARY printed.
- Release build, tracked-source integrity and generated lock hash: PASS.

A preceding workflow 37785468007 failed before compilation because this new workflow mistakenly used --locked before a lockfile existed. That technical error was corrected by generating the dependency-free lock offline; no production or evaluator code was changed between the two attempts. See INTEL2_REMAINDER_PREFLIGHT1.md.

## Same-organism sequence

The test reuses refined_context_fixture: one runtime autonomously acquires A, reuses A, acquires/switches B, then acquires C. Target transitions are not preloaded. Environment terminal resets are external observations; the learned organism is not reset between tasks. The subsequent C, restart, protection, D, E and final retention stages use this same runtime and its acquired memory.

### C, restart and software protection

- C held-out: 32/32; sides 16/16 and 16/16.
- Memoryless comparator: 16/32.
- Checkpoint/restart learned fingerprint: unchanged.
- Fresh observation required after restart; both C sides recovered with actual predecessor sensing, 2/2.
- High-risk and absent-assessment blocks: PASS.
- Emergency latch survives cognitive restart and fresh sensing: PASS.
- Blocked callbacks: 0; no fabricated learned change.
- Explicit external operator reset required: PASS.

These are simulated software-boundary tests, not real-world safety qualification.

### D — composition acquisition FAIL

Burned task: XOR, raw values [0.15, 0.35], action x=5 and complementary action y=3.

Unlike the historical evaluator, this test rewards ONLY action x on true XOR and ONLY action y on false XOR. The old expression '(act==x)==expected_x' rewarded every non-x motor on false cases, inconsistent with its held-out scoring. An exhaustive two-label/six-motor unit test verifies the new evaluator's strict rule.

Geometry-safe training positions: [[399,398],[397,396]].
Held-out positions: [[395,394],[393,392]].
All six layouts and all four cue combinations preserved the inherited coarse abstract state.

Observed:
- Physical training actions: 1200.
- Unsupported action during D training: none.
- Only one composition candidate was present on target base cell 472: Atom(WeakAmplitudeBin(5)), anchor action 5.
- Candidate future validation samples: 128.
- Candidate log evidence: -1.6938399940448647.
- Candidate promoted=false, retired=true.
- No target XOR program was proposed/promoted.
- Held-out score: 16/32; all 32 decisions obtained an action.
- Combination scores 00/01/10/11: [0,8,8,0] out of 8 each.
- Single-feature oracle ceiling: 16/32.

Action histograms, columns motors 0..5:

00: [0,0,1,0,0,719]
01: [0,0,0,0,1,119]
10: [0,0,0,1,0,119]
11: [1,1,0,0,0,238]

Thus the compound rule was not learned; the held-out behavior remained the same action across cue combinations.

#### Source-level blocking condition identified after the run

In src/phase_refinement_fanout.rs, phase_compositional_sidecar creates programs only under:

    let no_base_candidates = !comp.candidates.iter()
        .any(|w| w.base_cell == pre_cell);

candidate_programs then uses only the descriptor union of the selected conflicting pair. Once the first collision creates any candidate, later collisions cannot propose a new program on that base. The condition also counts retired candidates. In this run the first candidate was a single atom, it was correctly rejected by evidence, but its record permanently blocked subsequent compositional search.

This is a confirmed source-level search restriction consistent with the trace, not proof that removing the restriction alone is sufficient for full D success. No production repair was made in this diagnostic. A prospective repair must permit genuinely new bounded hypotheses from later factual collisions, suppress exact duplicate candidates, preserve rejected provenance, keep the candidate cap/future-only evidence rules, and avoid supplied XOR/feature/action labels. It must be tested rather than assumed successful.

### E — law-change capability NOT EXERCISED

The SAME post-D runtime entered E. It performed four physical actions, then stopped at state 17 with Err(NoSupportedAction).

- Executed actions: 4.
- Environment resets: 0.
- Goals: 0.
- Shortcut successes: 0 (four are required to trigger the existing scripted change).
- Changed law: false.
- Changed consequence observed: false.
- Recovery cost: unavailable.
- Pre-acquired unchanged branch: false.
- Frozen stable-branch probe: did not reach goal.
- Action histogram: [0,0,0,1,1,2].

This fails the whole-chain requirement but DOES NOT show failure to revise an observed changed law: the evaluator never exposed such a change. The reason for the four-action stop requires a separate trace; no root cause is asserted here.

The new evaluator measures recovery from the first changed consequence only. Repeated failed shortcut attempts cannot reset the recovery clock. This path was not exercised in this run.

### Final A/B retention — PASS 2/2

After the complete D attempt and E attempt, target transition/representation learning was frozen.

- Original-start translated World A: reached goal in 3 actions (limit 6).
- World B first goal in translated layout: reached goal in 2 actions (limit 8).
- Target circuit/representation witness records before/after revisits: exactly unchanged.
- Transferred U1 meta weights: unchanged throughout.
- Legacy graph transition count: 0.
- Dedicated table-composite count: 0.

This demonstrates retained A/B knowledge after the intervening failed learning attempts, not success in D/E. Existing runtime U2 utility updates are distinct from the frozen target representation/transition learning; no global all-state freeze is claimed.

## Recorded summary and handoff

    REMAINDER_SUMMARY C=true D=false E=false retention=2/2 verdict=DEVELOPMENT_CHAIN_FAIL

Do not rerun unchanged cognition hoping for a PASS. The first newly isolated representation problem is D's permanent first-candidate search lock, not absent XOR operator capacity. E remains a separate unexercised-drift stop; keep its status distinct.

Work stays on post-intel2-c-diagnosis. No edits were made to main, unified-cognition, parallel experiment branches, Codex AETERNA, historical INTEL verdicts, or production cognitive files in this continuation. No new independent intelligence verdict and no merge are authorized by this development evidence.
