# AETERNA-v1 COGNITIVE CORE FREEZE — INTEL-1

Date: 2026-10-07

The G-series stops after qualified FRESH-G23-3.

## Frozen cognitive baseline

Exact qualified source SHA:

`308aba7c06fa89613276ef90da725be963f2d25f`

INTEL-1 must compare all cognition-bearing files against that SHA.

Frozen paths:

- `src/**`
- `Cargo.toml`
- `Cargo.lock`

No modification to those paths is allowed between this freeze and an INTEL-1 verdict.

Evaluator-only additions are allowed under:

- `tests/intel1_*.rs`
- `docs/INTEL1_*.md`
- `.github/workflows/intel1*.yml`

Existing historical tests/docs may be read, but must not be changed to make INTEL-1 pass.

## Rule after failure

If INTEL-1 fails, do not add a numbered G-mechanism.

Record the earliest causal cognitive bottleneck. A later repair may change the cognitive freeze only to address that explicit bottleneck, followed by a new INTEL qualification with a new frozen baseline.

## Interpretation

This freeze is procedural, not a claim that the source is intelligent. INTEL-1 is the experiment intended to answer that question.
