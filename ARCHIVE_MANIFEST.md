# AETERNA-v1 — Git historical archive (2026-10-08)

This branch anchors the full branch-head history before qualified INTEL-4 promotion to `main`. Every branch head listed below is a real parent (directly or through ancestry) of the archive merge commit, not merely a SHA in documentation. Do **not** cherry-pick the archival octopus merge into product `main`; it intentionally includes negative and abandoned scientific tracks as *history only*.

## Immutable qualified system evidence

- INTEL-4 PASS commit: `8f70de4bf391b605be99c4476512de538a4644e2`
- Frozen cognitive source: `c7b5455ba006b297288fa8d16ef6300c8a19ceca`
- Actions run: https://github.com/kelbas2007/Aeterna-v1/actions/runs/37821849040
- Result: `docs/INTEL4_RESULT_PASS.md` at this commit.

## All branch heads captured at handoff

| Branch before cleanup | Full HEAD SHA | Disposition |
|---|---|---|
| `developmental-agenda` | `1f71b7285e084316a503c8d70c50390576959043` | Scientific history or superseded diagnostic; preserve failed evidence |
| `developmental-candidate-succession` | `16dd12c99d892fd7d5a6be4d1badad446d72b720` | Scientific history or superseded diagnostic; preserve failed evidence |
| `developmental-law-revision` | `ed9f69d39beef8aaffc53ff11c7523c1a22dca32` | Scientific history or superseded diagnostic; preserve failed evidence |
| `g1/raw-raster-distinction` | `e5b8608005f0f14e3f32a183206f58221ba1402c` | Open PR #3; do not close without review |
| `genesis/full-evophase` | `f50c97d97fcea8b65f1727edb70f33a0b307a250` | Scientific history or superseded diagnostic; preserve failed evidence |
| `intel2-r1-frozen` | `94347b62e1074d25aac056cb12452a7162a41f03` | Scientific history or superseded diagnostic; preserve failed evidence |
| `intel2-r1-postmortem` | `1429ce98ddf0a7851a074eedef984071497635b2` | Scientific history or superseded diagnostic; preserve failed evidence |
| `intel4-frozen-unified` | `8f70de4bf391b605be99c4476512de538a4644e2` | Qualified PASS snapshot |
| `main` | `d13585da3af57ca71a4d863636e0b14cb7d83471` | Old default; promote qualified INTEL-4 after separate merge verification |
| `post-intel2-action-ecology` | `4d132e9164b2e673a8ea70e8b4f2b56fef3252d4` | Scientific history or superseded diagnostic; preserve failed evidence |
| `post-intel2-c-diagnosis` | `f50bbc161c29833549e07de1a1db3551fa5d18dd` | Scientific history or superseded diagnostic; preserve failed evidence |
| `unified-cognition` | `903500f2f384e3a3c10329e44d27f620137b3d1c` | Scientific history or superseded diagnostic; preserve failed evidence |
| `unified-operation-competition` | `6abc7dbb70bfdb2917a033628b70df4027d45259` | Scientific history or superseded diagnostic; preserve failed evidence |

## Safety and workflow

1. One default branch `main` containing the verified INTEL-4 cognitive tree and its own previous commit ancestry.
2. One active research branch `research/beyond-intel4` for tests on structurally new worlds; no score transfer or patching frozen INTEL-4.
3. This `archive/evidence-20261008` head keeps every source branch tip reachable even if a stale branch is later deleted.
4. No automatic merge of other divergences; Codex AETERNA is a separate repository and is not touched.
5. Historical pre-seal/negative records are immutable; older tests and workflows may be placed under archived manifests but not erased from history.

If the GitHub connection does not support deleting branch refs, branch deletion must be finished in GitHub's branches UI **after** confirming this archive merge and the successful `main` integration. Do not delete `g1/raw-raster-distinction` while PR #3 remains open.
