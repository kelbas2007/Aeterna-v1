# Documentation index

Start with [the README](../README.md), [project map](../PROJECT_MAP.md) and
[status](../STATUS.md). Source code for the new development modes lives on
`research/beyond-intel4`; switch to that branch before running their examples.

## Current development modes

| Guide | What it covers | Example on the research branch |
|---|---|---|
| [Online learning](ONLINE_LEARNING.md) | Cold raw sensors, bounded receptors and disk checkpoints | `online_learning` |
| [Learned rules](LEARNED_RULES.md) | Circular copy/permutation rules and informative experiments | `learned_rules` |
| [Expanded rules](EXPANDED_RULES.md) | Constants, integer gains, signed sums and differences | `expanded_rules` |
| [Partial observation](PARTIAL_OBSERVATION.md) | Explicit missing channels, conservative alternatives and factual goals | `partial_observation` |
| [Inverse inference](INVERSE_INFERENCE.md) | Hidden operands, modular roots and separating measurements | `inverse_inference` |
| [Adaptive rules](ADAPTIVE_RULES.md) | Observed threshold conditions, bounded noise and retained old models | `adaptive_learning` |

These are separate opt-in development modes. Inverse inference extends partial
observation; adaptive rules use full observations. They are not one combined
noise/hidden-context system. The checkpoint writer is v5 and accepts v1–v4.

## Scientific evidence

| Line | Read first | Interpretation |
|---|---|---|
| INTEL-4 | [Result](INTEL4_RESULT_PASS.md), [protocol](INTEL4_PROTOCOL.md), [core freeze](INTEL4_CORE_FREEZE.md) | Qualified bounded deterministic five-world system |
| FRONTIER-1 | [Result](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/FRONTIER1_RESULT_FAIL1.md), [protocol](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/FRONTIER1_PROTOCOL.md) | Independent noisy-world FAIL, 40/80 |
| TE1–TE3 | [TE1](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE1_RESULT.md), [TE2](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE2_RESULT.md), [TE3](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE3_RESULT.md) | Physical evidence/sensing components; TE1 strict noisy holdout FAIL |
| TE4–TE5 | [TE4](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE4_RESULT_FAIL.md), [TE5](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE5_RESULT.md) | TE5 prepared mechanism succeeds; cold stochastic discovery remains failed |
| P1–P5 | [P1](PHASE_NATIVE_P1_RESULT.md), [P2](PHASE_NATIVE_P2_RESULT.md), [P3](PHASE_NATIVE_P3_RESULT.md), [P4](PHASE_NATIVE_P4_RESULT.md), [P5 fresh](PHASE_NATIVE_P5_FRESH_RESULT.md) | Phase-native execution, acquisition and retention evidence |
| G20–G23 | [G20](G20_CONTINUAL_REASONING_RESULT.md), [G21](G21_EVIDENCE_GATED_STATE_REFINEMENT_RESULT.md), [G22](G22_EVIDENCE_GATED_PERCEPTUAL_VARIABLE_RESULT.md), [G23 fresh](G23_COMPOSITIONAL_PERCEPTUAL_FUNCTION_FRESH3_RESULT.md) | Bounded reasoning, state refinement and perceptual composition |
| U1–U3 | [U1](U1_META_CONTROL_OWNERSHIP_RESULT.md), [U2](U2_HYPOTHESIS_ECOLOGY_RESULT.md), [U3](U3_CROSS_MECHANISM_RESIDUAL_ALLOCATION_RESULT.md), [runtime](UNIFIED_RUNTIME_INTEGRATION_RESULT.md) | Unified mechanism competition and allocation |

Historical results retain their original limits. Ordinary regression counts do
not represent independent task families or a new intelligence qualification.
The full dated record is in [EXPERIMENT_LEDGER.md](../EXPERIMENT_LEDGER.md).

## Architecture, operations and history

- [Architecture in Russian](ARCHITECTURE_RU.md) and [ownership contract](OWNERSHIP_CONTRACT.md): research objectives and boundaries.
- [G20 runtime usage](G20_RUNTIME_USAGE.md) and [P2 usage](PHASE_NATIVE_P2_USAGE.md): existing library interaction paths.
- [Human Protection](HUMAN_PROTECTION_RESULT.md) and [actuator permit](HUMAN_PROTECTION_ACTUATOR_RESULT.md): execution boundary and tested limits.
- [Contributing](../CONTRIBUTING.md): branch roles, ordinary checks and evidence rules.
- [Development integration validation](DEVELOPMENT_VALIDATION.md): current regression/demo outcomes and the remote verification boundary.
- [Branch cleanup proof](BRANCH_CLEANUP_RESULT.md): completed administrative cleanup and archive recovery.
- [Local project ideas](LOCAL_PROJECT_IDEAS.md): provenance of the earlier design comparison and implemented follow-up.
- [Historical README](HISTORICAL_README.md) and [historical development ladder](ROADMAP.md): earlier narrative and milestones. Their “current” labels belong to those historical stages; consult the project map for today's next steps.
