# HUMAN PROTECTION — NON-BYPASSABLE PHYSICAL ACTION GATE

Status: **FROZEN BEFORE IMPLEMENTATION**

Date: 2026-10-07

## Purpose

AETERNA may continue learning, planning, exploring and constructing abstractions, but no cognitive subsystem is allowed to directly authorize a physical action that can affect a person.

Human Protection is a separate host/runtime safety boundary, not a learned reward, concept, plan or preference.

## Priority invariant

For any proposed physical motor action:

```text
cognition proposes
  -> Human Protection screens
  -> only an ALLOW decision may reach a physical actuator
```

No planning score, expected reward, exploration gain, macro, concept, abstract state, drive or learned transition may override a BLOCK or EMERGENCY_STOP result.

## Frozen critical constants

Production uses hard safety constants rather than trainable parameters:

- maximum allowed predicted probability of human harm: **0.01**;
- minimum acceptable hazard-evidence confidence when a human can be affected: **0.90**.

These constants are not writable through EvoPhase learning APIs.

## Evidence contract

Before any physical action that may affect a person, the execution adapter must provide:

- `human_present`;
- `physical_effect_possible`;
- `predicted_harm_probability` in [0,1];
- `hazard_confidence` in [0,1];
- `emergency_stop`.

The evidence source is external to cognition. Human Protection does not ask AETERNA to grade its own safety.

## Frozen decisions

1. **EMERGENCY STOP**
   - if `emergency_stop=true`, block the proposed action;
   - latch the protection state;
   - subsequent physical actions remain blocked until an explicit external human reset path is called.

2. **KNOWN UNACCEPTABLE RISK**
   - if a human is present, a physical effect is possible and predicted harm probability >0.01, block.

3. **INSUFFICIENT SAFETY EVIDENCE**
   - if a human is present, a physical effect is possible and hazard confidence <0.90, block.

4. **INVALID EVIDENCE**
   - NaN/out-of-range risk or confidence blocks.

5. **ALLOW**
   - only when no rule above blocks.

## No cognition bypass

Production Human Protection code must contain no method that:

- accepts reward/utility as an override;
- accepts a planner authority token as an override;
- lets a learned weight change the harm threshold;
- lets ordinary cognition clear a latched emergency stop;
- silently substitutes another risky action after blocking.

A blocked proposal may return a configured non-physical SAFE_NOOP identifier to the host, but only the host may decide how that identifier maps to a real actuator command.

## Explicit external reset

Emergency-stop reset is an explicit external control API named as a human/operator reset.

It is deliberately separate from ordinary EvoPhase/cognitive action APIs.

Tests must prove that proposing/learning/planning actions cannot clear the latch.

## Audit record

Every screening produces a deterministic record containing:

- proposed action;
- verdict;
- reason;
- evidence snapshot;
- monotonic protection sequence number;
- whether emergency-stop is latched.

The record contains no mutable learned state.

## Mechanism PASS tests

PASS requires:

1. safe high-confidence action allowed;
2. harm risk >0.01 blocked;
3. confidence <0.90 blocked;
4. invalid/NaN evidence blocked;
5. emergency-stop blocks and latches;
6. a later apparently-safe proposal remains blocked while latched;
7. only explicit external human-reset clears the latch;
8. reward/planning/learning APIs cannot mutate protection constants or latch;
9. protection decisions are deterministic under replay;
10. audit sequence is monotonic;
11. Human Protection source guard finds no planner, reward, drive, concept or transition dependency;
12. existing G0-G16/P1-P5 regressions remain unaffected when Human Protection is not invoked;
13. Release build PASS.

## Integration boundary

This first mechanism provides a non-bypassable screening boundary. It does not itself provide:

- computer vision person detection;
- robotics collision prediction;
- medical risk estimation;
- legal/compliance policy;
- moral reasoning.

Those systems may supply hazard evidence later. The protection gate remains the final mandatory authorization boundary for physical actuation.
