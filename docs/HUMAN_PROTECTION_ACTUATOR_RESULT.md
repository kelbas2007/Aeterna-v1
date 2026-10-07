# Human Protection v1.1 — single-use actuator authorization

Date: 2026-10-07

## Verdict

**MECHANISM PASS — Human Protection now issues single-use current-sequence permits and fails closed on uncertain human absence.**

Workflow: `37616386983`  
Scored source: `435c1816c91c40acf53ffc8ea616862cf4d6352b`  
Protocol: `docs/HUMAN_PROTECTION_ACTUATOR_PROTOCOL.md`

## Result

Human Protection tests: **5/5 PASS**.

Qualified behavior:

- safe high-confidence physical proposal may receive an opaque permit;
- `HumanProtectionPermit` is neither Clone nor Copy;
- permit is bound to one action and one protection sequence;
- actuator-side `consume_permit` validates and consumes it;
- any later screening makes an older unconsumed permit stale;
- emergency-stop invalidates a previously issued permit;
- emergency-stop remains latched until explicit external human reset;
- low-confidence hazard evidence blocks a physical-effect proposal even when the external estimate says `human_present=false`;
- confident human-absence evidence may permit an otherwise safe proposal;
- invalid/NaN evidence blocks;
- cognitive checkpoint restore cannot clear the protection latch;
- source guard finds no planner/reward/drive/concept override dependency;
- full optimized regressions PASS;
- Release build PASS.

## Runtime meaning

A physical adapter should follow:

```text
cognition proposes motor
 -> external hazard evidence
 -> screen_physical_action
 -> opaque non-cloneable permit
 -> consume_physical_action_permit
 -> actuator command
```

The permit is not a reusable capability token.

## Deployment boundary

This repository still cannot prevent a separate privileged process from bypassing the API and driving hardware directly. Real deployment must:

- give actuator access only to the protected adapter;
- prevent cognition/untrusted processes from opening raw actuator devices;
- authenticate the external emergency-reset path;
- provide independent person/hazard sensing.

Human Protection v1.1 is the software authorization boundary, not the complete hardware safety case.
