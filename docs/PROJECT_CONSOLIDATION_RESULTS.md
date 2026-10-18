# AETERNA-v1: completed project consolidation — 2026-10-18

## Published state

PR #6 is merged into `main` with ordinary merge commit
`33a58d664ab7c24e78f764293c14686f43bf530a`. Complete research/RSP2 ancestry is
preserved; no squash or forced history replacement was used.

Verified implementation head: `0017dbca01aa6e9b35082fdc378fd1f10f4b364c`.
The final PR check used synthetic merge `2289c60e095b11de25e2f1e5deb0334dbafbfe1e`.
Its tree and the actual merge tree both equal
`27ce05320493e565660af6eacf1d578b9d3c60d9`.
This report is documentation only; it does not change the tested implementation.

`main` is the canonical integrated project; `research/beyond-intel4` is the
continuation branch and is synchronized by fast-forward after consolidation.
`integration/project-20261018` is the completed PR branch, not a competing
current implementation. Historical snapshots are intentionally preserved:

- `intel4-frozen-unified`: `8f70de4bf391b605be99c4476512de538a4644e2`.
- `archive/evidence-20261008`: `46c8e64562de7b9e300cb65ddcacc89b4e652d1f`.

The user's local `aeterna_genome_run` checkout/running experiments were not
modified. Issues #1 and #4 remain research work; consolidation is not their
scientific completion.

## What was consolidated

The canonical application is now `aeterna-v1` (also Cargo's default target).
`aeterna-unified` reuses the exact same implementation as a compatibility name.
The former motor-only main is retained as `examples/legacy_motor_demo.rs`.

The existing ScientificRuntime exposes virtual interaction, EvoPhase/U1,
directed construction, prospective evidence EP1, school memory/composition SG1,
quantity relations REL1 and recursive search RSP2 through one application.
These are available capabilities, not proof that every optional neural lane
runs together or that an unloaded application has acquired knowledge.

`--help`, `--status`, `--inspect FILE`, explicit school/relation modes and JSONL
are available. `--status` is an implementation inventory. `--inspect` validates
an actual saved state without changing it or executing an external action.
Independent saved lifetimes are not silently fused or relabelled.

README, PROJECT_OVERVIEW and PROJECT_STATE identify the current entry point,
contracts, previous results and remaining limitations. Historical documentation
is retained with navigation to the current overview. Before this final report,
314 Markdown documents passed the local-target link checker.

One main Linux/Windows project workflow verifies the application. Fourteen
specialized workflows are now manual. Current DC-2/R1 manual jobs verify the
installed code rather than rerunning already-consumed installation patches.
Temporary consolidation installers are absent from the final tree. Frozen
one-shot scientific qualifications were not reopened.

## Review correction and unchanged cognitive source

Eighty cognitive source files are byte-for-byte identical to the audited
research baseline `02b4175f34bcc68b8c80197cca6dd79f38b85892`. One existing file,
ScientificRuntime, has three explicitly recorded boundary guards: generic
native-only restart, native restore and online restore reject constructing
mode before mutation. This prevents partial restoration of a joint Learner/U1
state. The proper combined checkpoint API remains usable.

Four regressions fail on the former implementation and pass after the fix.
They cover atomic rejection, matching subsequent continuation and persistence
of the emergency latch. The source-binding audit verifies the exact guards and
reconstructs the original 81-file aggregate when removing only these guards.

RSP2's algorithm, residual features, admission and producer criteria are unchanged.
`src/construction/search_policy.rs` SHA256:
`aafe66f572983b9172e733a0abb7eccee2ba740af4eeec5ef36529782409bcb0`.

All five PR review threads were answered with evidence and resolved. Partial
restoration and two obsolete installer invocations were corrected. The claim
that Rust's Windows file rename cannot replace an existing file was not
reproduced; all eleven Windows persistent-session tests passed. No unsafe
remove-then-rename workaround was introduced. The claimed missing RSP2 ancestry
was contradicted by the actual remote commit graph; ancestry was not fabricated.

## Completed execution, not scheduled checks

Final GitHub Actions run:
https://github.com/kelbas2007/Aeterna-v1/actions/runs/39120412905

- Linux job `117654757945`: completed success.
- Windows job `117654758156`: completed success.
- All compilation targets checked on both platforms.
- 313 distinct selected Rust tests passed on Linux and the same 313 on Windows.
  This is not 626 different tests. The set includes 21 RSP2 repair tests,
  seven new canonical-entry tests and four new restoration tests.
- The additional broad Linux causal regression job and separate-process demos
  passed. Its log contains 278 passing test instances; they may overlap the
  selected set and are not added as 278 independent tests or capabilities.
- 242 Rust source files in the downloaded Linux and Windows archives match
  byte-for-byte. Human-readable JSON summaries differ only in LF/CRLF endings;
  their decoded contents agree. The acquired checkpoint bytes agree exactly.

The initial installer run `39118018444` passed compilation/tests but its final
push was rejected because the Actions token could not modify workflow files.
The exact tested source was published through the authorized GitHub connection.
That failed publication is preserved, not called a successful workflow.
Intermediate review run `39120270593` passed before final cross-platform checks.
The merge's skip-ci marker avoids only a duplicate run of the already verified
identical tree; no failed required check was bypassed.

## One actual joint-state example

One process performed sixteen real callbacks into a virtual environment,
received thirty-two numerical teacher examples, retained U1 and initialized
RSP2 in the same checkpoint. After freezing learning, the new input `137 + 58`
produced `195` by executing an acquired function. A separate process restored
and inspected that state. Linux, Windows and the local test agree on checkpoint:

`9ea86f90fd30dac0cfe672d2e5f5a9a6c58955c039612c7929b04b56c0fe60b3`.

RSP2 generation is zero in this example: initializing the mechanism is not
learning a better search policy. This is an integration example, not a new
external intelligence result.

Separately, four actual earlier checkpoints — Unified, SG1, REL1 and legacy
RSP1 — were loaded/inspected locally with the final guarded implementation.
Their input files remained unchanged and no external action was performed.
Legacy RSP1 remains inspectable/solvable, not silently converted into a state
that passed the stronger RSP2 admission contract.

## Evidence and Windows package

Final artifact IDs and SHA256 digests:

- Windows program `12133269722`, 940055 bytes:
  `6fedffdf3ad70fc423744699b2cb4b91671e90008627deb356fa54794c1bded7`.
- Windows evidence `12133849122`:
  `0b5dfe6f65f7dcbe782417586c31023bb4c762a9d6e14524db507194255671b8`.
- Linux evidence `12134685825`:
  `5deb4c435c139b5aa84405d35eddbd964478d2c41411667b4c1032d42f822f0e`.

The package contains `aeterna-v1.exe`, its explicit small example checkpoint,
result, launchers and file checksums. It does NOT replace every historical
trained state with one invented combined lifetime. `START-AETERNA.cmd` opens
help; `START-EXAMPLE.cmd` opens the example's limited school interface. Existing
acquired states are selected explicitly with `--school FILE` or `--relations FILE`.
Rust/Python/internet are not required to run this packaged executable.

**Result: one current main project, one canonical application and verified
state compatibility. General intelligence and stable recursive improvement
are not newly established by this consolidation.**
