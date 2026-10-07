# HUMAN PROTECTION v1.1 — SINGLE-USE ACTUATION PERMIT

Status: **FROZEN BEFORE v1.1 IMPLEMENTATION**

Date: 2026-10-07

## Purpose

Human Protection v1 qualified screening and an emergency latch.

v1.1 strengthens the runtime authorization boundary so an ALLOW decision cannot
be cached and replayed as a reusable physical-actuation credential.

## Frozen requirements

### Single-use permit

- `HumanProtectionPermit` is opaque and not `Clone` or `Copy`.
- A permit is bound to exactly:
  - one proposed action;
  - one protection sequence number.
- The actuator/runtime consumes the permit through the protection gate.
- A consumed permit cannot be consumed again through safe Rust ownership.
- Any later screening event advances the protection sequence and invalidates all older unconsumed permits.
- An emergency-stop latch invalidates every previously issued permit.

### Fail-closed uncertainty

For any proposal with `physical_effect_possible=true`:

- if hazard evidence confidence < **0.90**, block regardless of the Boolean human-presence estimate.

Interpretation:

- `human_present=false` is usable as “human absent” only when the external hazard system is itself sufficiently confident;
- an uncertain person detector cannot turn “unknown” into “safe”.

The existing >0.01 harm-risk block remains mandatory when a human is reported present.

### No cognitive override

The permit-consumption path may not depend on:

- planner value;
- reward;
- exploration drive;
- concepts;
- transition model;
- task authority.

Cognitive checkpoint restore still may not clear the emergency latch or mint a permit.

## v1.1 PASS requirements

1. allowed proposal yields one opaque permit;
2. permit is valid for exactly the screened action and current sequence;
3. consuming it succeeds exactly once;
4. a later screening invalidates an older unconsumed permit;
5. emergency-stop invalidates a previously issued permit;
6. low-confidence physical-effect evidence blocks even when `human_present=false`;
7. high-confidence human-absent evidence may pass if other checks pass;
8. cognitive checkpoint restore cannot clear emergency latch or create a valid permit;
9. source guard confirms no cognitive override dependencies;
10. Human Protection v1 tests remain satisfied;
11. G0-G16/P1-P5 regressions PASS;
12. Release build PASS.

## Deployment boundary

The repository still cannot stop an unrelated external process from directly
driving hardware. Real deployment must make the hardware adapter accept actions
only through the permit-consumption interface, with OS/process/hardware access
controls preventing bypass.
