# INTEL-2 preflight-1 — technical shallow-clone failure

Run: `37686689632`
Date: 2026-10-07

The workflow failed before `INTEL2_SEAL` at the frozen-source comparison command with exit code 128.

Cause: `actions/checkout@v4` used the default shallow checkout, so frozen cognitive baseline commit `07b44fb8e00837568bc9655e760eb031d1944dff` was not present in local Git history.

No cognitive source was scored or changed. No INTEL-2 authority pack was exposed or consumed.

Prospective workflow-only repair: use `fetch-depth: 0` so the frozen baseline can be compared. Frozen cognitive source remains unchanged.
