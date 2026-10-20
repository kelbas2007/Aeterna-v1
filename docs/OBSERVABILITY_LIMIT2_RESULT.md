# OBSERVABILITY-LIMIT2 — old certified chain planner does not rescue actual-state sensor

**2026-10-20 SOURCE-DISCLOSED NEGATIVE**:
`NO_CERTIFIED_CHAIN_SENSOR_RESCUE`.
[Protocol frozen before testing](OBSERVABILITY_LIMIT2_PROTOCOL.md) at
[`36d50f3c`](https://github.com/kelbas2007/Aeterna-v1/commit/36d50f3c2d3cf48b67a34c9761506e7e2d53b29c).
[Source `3182bc5c`](https://github.com/kelbas2007/Aeterna-v1/commit/3182bc5c173d02a0aa32e570ec3c33289d5e5254).
[Linux and Windows exact workflow 39413911268](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39413911268):
**2/2 PASS each OS**, original source binding audit PASS; exact
metrics identical across both. Engineering execution PASS but
the preregistered SENSOR-RESCUE GATE **FAILED**.

| Source-disclosed 36-world arm | Real goals /864 | Real actions | Native selected certified factual chains |
|---|---:|---:|---:|
| Blind progress / no certified chains | 487 | 8,625 | 0 |
| True intermediate sensor / no certified chains | 194 | 12,875 | 0 |
| Blind / certified chains ENABLED | 487 | 8,625 | 0 |
| True intermediate sensor / certified chains ENABLED | **188** | **12,915** | **0** |

In all arms the `certified_chain_proposed` counter is ZERO.
Turning on the existing certified future-transition planner is not
a behavioral intervention here: neither its proposer nor native U1
sees any qualifying chain despite real sensor data. The true-stage
goal successes further regress from 194 to 188. Exact 36-world
sub-families:
- Four-action: rich OFF 18/288; rich ON **15/288**.
- Five-action: rich OFF 1/288; rich ON **1/288**.
- Three-action: rich OFF 31/144; rich ON **28/144**.
- Direct-action: rich OFF=ON **144/144**.

The source graph is developer-authored. Calibration requires a
real older-prefix and three newer factual protected POSTs to validate
each individual transition. Zero qualified routes may reflect this
admission evidence threshold, incomplete graph connectivity, or
candidate U1 competition; the present test establishes ZERO proposals,
not uniquely which subsystem is the single cause.

## Consequence for architecture

The current `carrier_autobiographical_option_metered` and
variable-history routes assume preparatory steps leave the full
observation unchanged. Visible real causal stages violate this
assumption; the alternative certified graph remains unusable under
the observed finite cold action budget.

The next narrow corrective experiment must **reuse actual PRE/action/POST
transitions as an empirical observable-state graph**, without
requirement of sensor-invisible preparation, and treat graph plans
as tentative native-U1 options only after actual observed support.
Each subsequent action must still be protected and its REAL POST must
match the predicted successor, revoking otherwise. Check the
original blind and direct-action controls so the new graph cannot
fake hidden conditions or silently regress earlier skills.

This is not a new one-use scientific claim, nor evidence that AETERNA
can invent a physical sensor, model arbitrary continuous dynamics,
run real actuators or achieve AGI.
