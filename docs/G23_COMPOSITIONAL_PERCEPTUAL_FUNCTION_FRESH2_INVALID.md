# FRESH-G23-2 — burned evaluator-invalid qualification

Date: 2026-10-07

## Verdict

**INVALID AFTER SEAL — evaluator-generated weak marker overlapped an inherited motif pixel. Pack permanently burned.**

Run: `37655207521`  
Job: `112908461170`  
Source: `4ddc07016201fdd43394a525a6eda22db08557a6`.

The run passed the G23 mechanism preflight, G22, G21, G20, Human Protection, full ordinary regressions and Release build before exposing the new authority pack.

After `FRESH_G23_2_SEAL` and the complete pack dump, sub-seeds 0..4 executed successfully and each produced FULL 8/8 with promoted authority-selected target compositions. During later pack execution, the evaluator called the frozen scene helper with an authority-selected weak-marker index that was already occupied by an inherited >=0.5 motif. The helper correctly rejected this invalid construction at:

```text
assertion failed: index < s.len() && s[index] < 0.5
```

This is an evaluator/world-generation invalidity, not a cognitive threshold result: the raw weak marker could not be instantiated under the protocol's requirement that it remain outside the inherited active motif.

Because the pack had already been sealed and printed, it is permanently burned and cannot be repaired/replayed or counted PASS.

## Required prospective repair

A new independently seeded protocol may sample marker positions only from a precomputed set that is guaranteed free for **every layout used by that sample** before the pack is sealed. The free-position algorithm must depend only on the inherited raw scene geometry/layout and not on target operator, action, outcome, promotion or model behavior.

Production G23 cognition, grammar and evidence thresholds must remain unchanged.
