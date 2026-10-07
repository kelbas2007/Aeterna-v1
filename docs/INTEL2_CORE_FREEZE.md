# INTEL-2 cognitive core freeze

Date: 2026-10-07
Branch: `unified-cognition`

## Frozen cognitive baseline

`07b44fb8e00837568bc9655e760eb031d1944dff`

Frozen paths:
- `src/**`
- `Cargo.toml`
- `Cargo.lock`

This source passed:
- U1 carrier-owned meta-control;
- U2 carrier-owned hypothesis ecology;
- U3 cross-mechanism residual allocation;
- unified protected runtime integration;
- G20-G23 regressions;
- Human Protection;
- Release build.

## INTEL-2 rule

From this freeze until the INTEL-2 verdict:

**NO cognitive source changes.**

Allowed additions/changes:
- `tests/intel2_*.rs`;
- `docs/INTEL2_*.md`;
- `.github/workflows/intel2-*.yml`;
- evidence/status documentation that does not alter cognitive source.

The workflow must enforce:

```sh
git diff --exit-code 07b44fb8e00837568bc9655e760eb031d1944dff -- src Cargo.toml Cargo.lock
```

Any cognitive source difference invalidates the INTEL-2 verdict.

INTEL-2 must use `ScientificRuntime::step_unified`, not legacy `step()` / fixed-priority `propose()`.

If INTEL-2 fails a frozen criterion, record FAIL before any redesign. No immediate Repair-N patch is allowed.
