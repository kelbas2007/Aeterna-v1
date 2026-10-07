# INTEL-2 preflight-2 — technical evaluator compile failure

Run: `37686879024`
Date: 2026-10-07

All frozen-core pre-gates passed:
- cognitive source freeze;
- unified runtime assembly;
- U3;
- U2;
- U1;
- Human Protection;
- Release build.

The INTEL-2 evaluator then failed to compile because the nested test-only `foundation` module used `BTreeSet` without importing it into that module scope (Rust E0433).

`INTEL2_SEAL` was never printed. No target pack was exposed or consumed.

Only the evaluator import is repaired. Frozen cognitive source remains unchanged.
