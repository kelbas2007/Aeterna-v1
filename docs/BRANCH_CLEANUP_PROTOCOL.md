# AETERNA-v1 — one-use safe branch-list cleanup

Date: 2026-10-08
Status: READY FOR PREREQUISITE CHECKS. Deleting a GitHub branch **ref** is not deleting its historical commits; all specified old refs have a retained ancestry path to the protected archive / qualified main.

## Protected refs — NEVER DELETE
- `main` — current qualified INTEL-4 working tree, which preserves former main ancestry in a two-parent merge.
- `intel4-frozen-unified` — exact original frozen verdict and result.
- `archive/evidence-20261008` — one 13-parent Git archive head, pinned at `46c8e64562de7b9e300cb65ddcacc89b4e652d1f`.
- `research/beyond-intel4` — current new-world experiment branch, frozen source unchanged.

## 12 stale refs eligible only at these exact old HEADs

| Branch | Pinned head SHA | Where old history remains reachable |
|---|---|---|
| `developmental-agenda` | `1f71b7285e084316a503c8d70c50390576959043` | Archive octopus ancestry |
| `developmental-candidate-succession` | `16dd12c99d892fd7d5a6be4d1badad446d72b720` | Archive octopus ancestry |
| `developmental-law-revision` | `ed9f69d39beef8aaffc53ff11c7523c1a22dca32` | Archive octopus ancestry |
| `g1/raw-raster-distinction` | `e5b8608005f0f14e3f32a183206f58221ba1402c` | Archive octopus ancestry |
| `genesis/full-evophase` | `f50c97d97fcea8b65f1727edb70f33a0b307a250` | Archive octopus ancestry |
| `intel2-r1-frozen` | `94347b62e1074d25aac056cb12452a7162a41f03` | Archive octopus ancestry |
| `intel2-r1-postmortem` | `1429ce98ddf0a7851a074eedef984071497635b2` | Archive octopus ancestry |
| `post-intel2-action-ecology` | `4d132e9164b2e673a8ea70e8b4f2b56fef3252d4` | Archive octopus ancestry |
| `post-intel2-c-diagnosis` | `f50bbc161c29833549e07de1a1db3551fa5d18dd` | Archive octopus ancestry |
| `unified-cognition` | `903500f2f384e3a3c10329e44d27f620137b3d1c` | Archive octopus ancestry |
| `unified-operation-competition` | `6abc7dbb70bfdb2917a033628b70df4027d45259` | Archive octopus ancestry |
| `integration/intel4-main` | `59c0e299f8927cd9c03c148db07d60ed04ab0e8b` | Current main ancestry |

## Mandatory guards before any deletion

1. `archive/evidence-20261008` is present with the pinned SHA above; each non-integration stale HEAD is provably its ancestor.
2. `integration/intel4-main` is fully reachable from qualified `main` after its CI PASS `37831043806`.
3. Current GitHub remote SHA of each candidate **exactly equals** its pinned historical SHA. Changed/missing branches are skipped, not force-deleted.
4. PR #3 for `g1/raw-raster-distinction` has been explicitly closed as superseded, **not merged**; original SHA preserved in archive.
5. Any newly open PR using a stale candidate head must block that deletion.
6. Execution only inside a one-use GitHub Actions workflow, with `contents: write` permission and checkout of the default `main`; no user secrets or paid browser automation.
7. Deleting the twelve refs must not affect the frozen INTEL-4 source, evidence ledger, or the separate mature Codex-AETERNA.

The action prints the SHA of each deleted branch, skips mismatches or unsafe refs, and explicitly verifies `main`, INTEL-4, archive, and research remain afterward. On insufficient repository token permissions, the action fails without silently claiming cleanup.

## Recovery
All pre-cleanup SHA and full 13-parent history: `archive/evidence-20261008/ARCHIVE_MANIFEST.md`.
To recover an old head: GitHub → compare the archive/old SHA; create a temporary branch at that exact SHA if needed.

A GitHub branch cleanup is administrative only. It **does not count as scientific evidence** and does not imply that all old tests passed.
