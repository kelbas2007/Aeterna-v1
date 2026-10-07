# Human Protection v1 — physical action authorization boundary

Date: 2026-10-07

## Verdict

**MECHANISM PASS — fail-closed non-learned Human Protection gate is integrated into EvoPhase.**

Workflow: `37614779191`  
Scored source: `2f4bf80edf230778bac4c36080db66757e674a4d`  
Protocol: `docs/HUMAN_PROTECTION_PROTOCOL.md`

## Frozen safety constants

- maximum allowed predicted probability of human harm: **0.01**;
- minimum hazard-evidence confidence when a human can be physically affected: **0.90**.

These are production constants, not trainable weights.

## Passing tests

The Human Protection test suite passed **3/3**:

1. safe high-confidence proposal is allowed and receives a sealed permit;
2. risk above 0.01 is blocked;
3. confidence below 0.90 is blocked;
4. NaN/invalid evidence is blocked;
5. emergency-stop blocks and latches;
6. later apparently-safe proposals remain blocked while latched;
7. only explicit `external_human_emergency_reset()` clears the latch;
8. protection replay is deterministic;
9. audit sequence is monotonic;
10. restoring a cognitive/phase-native checkpoint in the same protected runtime does **not** clear the protection latch;
11. source guard finds no planner, drive, concept, reward override or threshold setter dependency.

Full optimized regressions and Release build passed on the same source.

## Runtime contract

Cognition may propose a motor action, but a physical execution adapter must obtain a `HumanProtectionPermit` from:

`EvoPhase::screen_physical_action(...)`

before physical actuation.

A permit cannot be constructed directly by ordinary external Rust callers because its sealing field is private.

Protection state is intentionally not stored in the cognitive phase-native checkpoint, so cognitive restore cannot overwrite a latched emergency stop in the same runtime.

## Important boundary

Human Protection v1 is the **authorization gate**, not the hazard sensor.

It does not yet provide person detection, collision prediction, medical-risk estimation or authenticated hardware emergency-reset infrastructure. Those systems must provide hazard evidence externally.

A real actuator integration must also enforce two deployment rules outside cognition:

- actuator APIs accept only a valid protection permit;
- `external_human_emergency_reset()` is authenticated as a real human/operator path.

The current repository contains the safety boundary and proof tests, not a physical robot/actuator driver.
