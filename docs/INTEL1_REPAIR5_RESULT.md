# INTEL-1 Repair-5 verification result

Date: 2026-10-07

Verdict: **FAIL / PARTIAL — direct information-balance rule works, but lower-priority rival arbitration still reissues the stale anchor.**

Workflow: `37670324538`
Tested source included Repair-5 balanced contextual experiment selection.

Observed direct witness:

- over-sampled side -> `(false,None)`;
- under-sampled side -> `(true,Some(anchor))`.

Thus the Repair-5 local context arbitration behaves as preregistered.

However the persistent all-refiners witness still failed:

- useful later context promotion: false;
- junction score: 0/64;
- stale/new context candidates accumulated one-sided evidence such as [0,60] and [0,66], switches 0;
- all such candidates used the old anchor action 0.

Diagnosis: once context arbitration yields, the higher-priority G19 rival selector still sees the preserved parent rival successors for action 0 and independently chooses the same probe. Parent rivals must remain structurally present because the unobserved predecessor side may still validate them; therefore deleting the rival is not a valid repair.

Repair-5 remains an admitted generic improvement but does not close the INTEL bottleneck by itself. No new INTEL authority pack was consumed.
