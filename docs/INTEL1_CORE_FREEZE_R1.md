# INTEL-1 COGNITIVE FREEZE R1

Date: 2026-10-07

After the first INTEL-1 failure, exactly one preregistered cognitive repair was made and verified.

New frozen cognitive SHA:

`397e57bcceae47b59275eac9fbfa865e2d6238da`

Frozen paths:
- `src/**`
- `Cargo.toml`
- `Cargo.lock`

The only cognitive difference from the original INTEL-1 freeze is Repair-1 documented in `INTEL1_REPAIR1_PROTOCOL.md`: fallback from unavailable goal-conditioned exploration to the pre-existing generic epistemic drive.

The next INTEL run must use a new authority seed and may not reuse the first INTEL pack.
