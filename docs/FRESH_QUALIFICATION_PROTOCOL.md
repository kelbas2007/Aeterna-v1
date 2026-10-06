# Fresh qualification authority protocol

Version: FRESH-1
Date: 2026-10-06

## Purpose

Prevent adaptive reuse of held-out worlds after a failure or design revision.

## Seal

A qualification run is valid only for one exact source SHA and one exact qualification-spec SHA.

Before a run:
- source is committed;
- protocol/spec files are committed;
- no world seed is selected by the developer.

At run time:
- the external CI run identifier becomes the fresh authority seed;
- the runner derives 10 sub-seeds;
- each sub-seed generates an independent block of held-out worlds;
- generated parameters and their digest are written to the log before scoring.

After a run:
- PASS/FAIL is appended to the experiment ledger;
- if code or protocol changes, the old pack is permanently burned for qualification purposes;
- a new CI run produces a new authority seed and new world pack.

## Minimum pack

For transfer/generalization claims:
- >=64 total held-out worlds;
- >=10 sub-seeds;
- randomized absolute translations where applicable;
- randomized opaque motor relabeling where applicable;
- matched arms evaluated on exactly the same pack.

## Reporting

Always report:
- source SHA;
- protocol SHA;
- authority seed;
- world-pack digest;
- N worlds;
- successes / failures;
- 95% interval;
- mean and distribution of physical interaction cost;
- matched baseline/control values.

## Prohibition

A world pack observed before a design change may be used for debugging and regression only. It may never again be called fresh held-out evidence for that revised design.
