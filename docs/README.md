# Documentation index — 22 October 2026

[Непрерывное обучение: отрицательная проверка на реальных рядах](CONTINUOUS_RULE_LEARNING_RESULT.md) ·
[Реализация и единое сохраняемое состояние](CONTINUOUS_RULE_LEARNING.md) ·
[Заранее заданные критерии](CONTINUOUS_RULE_LEARNING_PROTOCOL.md).
Прогнозы полного и фиксированного вариантов совпали; рекурсивного улучшения нет.

[Реализованный общий контур обучения v2 и запуск на файлах](LEARNING_ORGANISM.md) ·
[Проверки реализации](LEARNING_ORGANISM_RESULT.md).

[Разбор исследований мозга и ограничений общего организма](BRAIN_ARCHITECTURE_REVIEW_20261021.md) ·
[Выбранное направление архитектуры: общий контур обучения](decisions/0001-learning-organism.md).
Обзор фиксирует исходную архитектуру; первый полный контур этапов 1–2 теперь реализован.

[Аудит архитектуры](ARCHITECTURE_AUDIT_20261021.md) ·
[Протокол первых прогнозов целого организма](FIRST_READOUT_PROTOCOL.md) ·
[Сравнение памяти: удержание PASS, преимущество нового оценщика FAIL](CONSOLIDATED_MEMORY_RESULT.md) ·
[Протокол консолидации](CONSOLIDATED_MEMORY_PROTOCOL.md) ·
[Предыдущая неудача политики покрытия](COVERAGE_MEMORY_RESULT.md) ·
[Основной живой организм PR18](FIRST_READOUT_RESULT.md) ·
[Проверка при одинаковой заморозке](FROZEN_PROBE_PROTOCOL.md) ·
[Предыдущая опубликованная жизнь](ACQUIRED_RULES_RESULT.md).

Прежняя формальная внешняя оценка: [EXTERNAL-CONTINUAL-5](EXTERNAL_CONTINUAL5_RESULT.md) — изображения 75.1% → 72.3%, сигналы 50.5% balanced accuracy, MiniGrid 3/48, ARC BLOCKED. [Общий сенсорный ученик и адаптация](EXTERNAL_CONTINUAL5.md) · [протокол](EXTERNAL_CONTINUAL5_PROTOCOL.md) · [ревью и происхождение данных](EXTERNAL_CONTINUAL5_REVIEW.md).

[CAUSAL-CONTINUAL-4C](CAUSAL_CONTINUAL4C_RESULT.md): **ENGINEERING_PASS**, все 384 жизни на 32 новых перестановках.
Десятишаговые цели **32/32** против **0/32**,
полезный шумный датчик **1877/2048** против **1140/2048**, простая цель **2048/2048**.
Все планировщики соблюдают оставшийся срок; полный расход и бесполезные измерения опубликованы.
[Прежнее B с оговоркой о сроке](CAUSAL_CONTINUAL4B_RESULT.md) сохранено отдельно.
[PR #14](https://github.com/kelbas2007/Aeterna-v1/pull/14) объединён в `main`; [Linux/Windows CI](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39488724394) прошёл.

Текущая разработка: [причинные гипотезы, ценность измерения, отложенная
награда и непрерывная память](CAUSAL_CONTINUAL4.md). Новый режим включается явно;
JSONL принимает горизонт до 32, до 32 датчиков и 16 моторов.
Приведённые ниже старые оценки сохраняют исходные версии и границы.

**Start with current truth, not older dated "latest" passages:**
[Current status (RU)](CURRENT_STATUS_20261020.md) ·
[Architecture](PROJECT_OVERVIEW.md) ·
[JSON state](PROJECT_STATE.json) ·
[Integration / provenance](MAIN_INTEGRATION_20261020.md) ·
[Actual JSONL API and limits](JSONL_AUTONOMOUS_OPTIONS.md) ·
[Root README](../README.md).

## Observability and directly observed causal skills (research 2026-10-20)

- [True-sensor ablation: NEGATIVE 487→194/864](OBSERVABILITY_LIMIT1_RESULT.md) ·
  [frozen development protocol](OBSERVABILITY_LIMIT1_PROTOCOL.md).
- [Older strict certified chain does not rescue sensory change:
  NEGATIVE 194→188/864](OBSERVABILITY_LIMIT2_RESULT.md) ·
  [frozen protocol](OBSERVABILITY_LIMIT2_PROTOCOL.md).
- [NEW observable empirical graph: ENGINEERING PASS 194→457/864,
  296 real self-selected completed routes](OBSERVABLE_GRAPH1_RESULT.md) ·
  [precommitted protocol](OBSERVABLE_GRAPH1_PROTOCOL.md) ·
  [original first source](https://github.com/kelbas2007/Aeterna-v1/commit/e03b75cc40408e14c624c8b5693177c94039bb00).
  This is NOT new independent science or learned sensor invention.

## Source-frozen scientific results vs ordinary engineering

| Line | Protocol or design | Actual result and evidence |
|---|---|---|
| Own self-selected causal sequences | [frozen one-use protocol](SELF_ORIGINATED_OPTION1_ONE_USE_PROTOCOL.md) | [658/768 vs 366/768 QUALIFIED_SYNTHETIC](SELF_ORIGINATED_OPTION1_ONE_USE_RESULT.md), first [run 39370868718](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39370868718) |
| Cold causal history negative control | [COLD-FRESH protocol](COLD_EPISODES_FRESH1_PROTOCOL.md) | [NEGATIVE 407→432/768 with two regressed families](COLD_EPISODES_FRESH1_RESULT.md) |
| Acquired trajectory evaluator | [design](TRAJECTORY_SELF_REVISION_TRANSFER_PROTOCOL.md) | [T-SR1 NEGATIVE](TRAJECTORY_SELF_REVISION_TRANSFER_RESULTS.md); [T-SR2 NEGATIVE](T_SR2_BEHAVIORAL_GATE_RESULTS.md) |
| Experience-derived options | [development protocol](SELF_ORIGINATED_OPTION0_DEVELOPMENT_PROTOCOL.md) | [Open cold development positive](SELF_ORIGINATED_OPTION0_DEVELOPMENT_RESULT.md) |
| Physical option execution and terminal credit | [mechanism](TEMPORAL_OPTION_OUTCOME_CREDIT_DEVELOPMENT.md) | [Causal option real POST regression](CAUSAL_OPTION_EXECUTION_RESULT.md) |
| Variable option depth (2–5) | [development protocol](VARIABLE_LENGTH_CAUSAL_OPTIONS_DEVELOPMENT.md) | 467/864 vs fixed 307/864; five-action reliability not established |
| Necessity before extra complexity | [preimplementation protocol](COMPLEXITY_NECESSITY1_DEVELOPMENT_PROTOCOL.md) | [Development 472 vs 467 vs 307 /864](COMPLEXITY_NECESSITY1_DEVELOPMENT_RESULT.md), Linux/Windows PASS |
| LC1 / RSP2 / RSP3 | [LC1](LC1_INTEGRATION.md), [RSP2](RSP2_REPAIR_RESULTS.md), [RSP3](RSP3_EXPERIENCE_RESULTS.md) | Limited software learning, no proven recursively self-improving general policy |

Later successful *SKIPPED* one-use workflow invocations are **not new
scientific results**. Preserve original PASS/NEGATIVE protocols and seeds.
See [experiment ledger](../EXPERIMENT_LEDGER.md).

---

## Historical documentation catalog — original October 2026 material

## Earlier consolidation — 2026-10-18

Read [the current project overview](PROJECT_OVERVIEW.md) first. The canonical executable is
`aeterna-v1`; `aeterna-unified` shares the same application code. RSP2 from PR #5
is included. Run `python scripts/check_project.py` for the current regression
set. Historical results below remain tied to their original code; statements
calling older results “latest” apply only to their original date. The unified
CI does not repeat external exams or consumed authority packs.


Start with [the README](../README.md), [project map](../PROJECT_MAP.md) and
[status](../STATUS.md). Current source and examples are integrated into `main`;
`research/beyond-intel4` continues development from the integration point.

## Current development modes

The [acquired representation guide](ACQUIRED_VALUE_REPRESENTATIONS.md) and
[frozen transfer result](VALUE_RULE_TRANSFER3_RESULT.md) cover learned
predicates, measurement grouping and factual motor-effect models. MemoryS7
training transferred to random-start navigation at 60/64 versus lesion/cold
12/64 and random 14/64; the authored controller scored 64/64. Separate learned
lives scored real images 353/364 and signals 127/150 on reused public fixtures.
The [first](VALUE_RULE_TRANSFER1_RESULT.md) and
[second](VALUE_RULE_TRANSFER2_RESULT.md) failures remain preserved.
MemoryS9 cue-use transfer is still FAIL; generic algorithms remain authored.

The [context-value guide](CONTEXT_VALUE_LEARNING.md) and
[new result](CONTEXT_VALUE1_RESULT.md) add measured causal cue use:
MemoryS7 64/64, with MemoryS9 transfer FAIL 10/32. They do not qualify general
intelligence or individual object identity. Earlier memory failures remain intact.

The [2026-10-10 review](GITHUB_REVIEW_20261010.md) covers integrated external
MiniGrid interaction, argument transfer, episodic and relational memory,
their failed external cue-use evaluations, and the consolidated CI.

| Guide | What it covers | Example |
|---|---|---|
| [Online learning](ONLINE_LEARNING.md) | Cold raw sensors, bounded receptors and disk checkpoints | `online_learning` |
| [Learned rules](LEARNED_RULES.md) | Circular copy/permutation rules and informative experiments | `learned_rules` |
| [Expanded rules](EXPANDED_RULES.md) | Constants, integer gains, signed sums and differences | `expanded_rules` |
| [Partial observation](PARTIAL_OBSERVATION.md) | Explicit missing channels, conservative alternatives and factual goals | `partial_observation` |
| [Inverse inference](INVERSE_INFERENCE.md) | Hidden operands, modular roots and separating measurements | `inverse_inference` |
| [Adaptive rules](ADAPTIVE_RULES.md) | Observed threshold conditions, bounded noise and retained old models | `adaptive_learning` |
| [Uncertain learning](UNCERTAIN_LEARNING.md) | Combined noisy partial inference, joint equations and acquired sensing | `uncertain_learning` |
| [Multivariate perception](VECTOR_PERCEPTION.md) | Bounded numeric patterns, PNG/JPEG files and factual correction | `perception` |
| [Induced programs](INDUCED_PROGRAMS.md), [protocol](INDUCTION_PROTOCOL.md) | Experience-built predicates/topology, subsequent validation and bounded physical programs | `induced_programs` |
| [Acquired operations](ACQUIRED_OPERATIONS.md), [protocol](PRIMITIVE_PROTOCOL.md) | Callable acquired definitions, factual constructor preferences, recorded images/signals; both usefulness gates FAIL | `acquired_primitives`, `real_programs` |
| [Recorded digit benchmark](DIGITS_BENCHMARK.md), [protocol](VECTOR_PROTOCOL.md) | Frozen real 64-pixel images and same-facts centroid/3-NN controls | `real_digits` |
| [Recorded Iris benchmark](IRIS_BENCHMARK.md) | External measurements, hidden features and matched decision stump | `iris_grounded` |

The seventh opt-in mode combines partial inverse inference with adaptive noisy
models. Numeric tuition still requires actual full PRE/POST; inferred centers
cannot teach. The checkpoint writer is v9 and accepts v1–v8. The Iris benchmark
extends open development validation beyond synthetic generators and preserves
the simple comparator's higher aggregate score. The eighth mode learns bounded
multivariate patterns in shared phase synapses, with inherited distance and
voting algorithms. Its digit benchmark and actual file utility add external
image use; integration with the separate symbolic rule modes remains future work. The ninth mode augments vector
learning with data-driven conditional programs; generic branch search remains
inherited software, and the initial three-input failure is retained.

## Scientific evidence

| Line | Read first | Interpretation |
|---|---|---|
| INTEL-4 | [Result](INTEL4_RESULT_PASS.md), [protocol](INTEL4_PROTOCOL.md), [core freeze](INTEL4_CORE_FREEZE.md) | Qualified bounded deterministic five-world system |
| FRONTIER-1 | [Result](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/FRONTIER1_RESULT_FAIL1.md), [protocol](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/FRONTIER1_PROTOCOL.md) | Independent noisy-world FAIL, 40/80 |
| TE1–TE3 | [TE1](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE1_RESULT.md), [TE2](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE2_RESULT.md), [TE3](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/docs/TE3_RESULT.md) | Physical evidence/sensing components; TE1 strict noisy holdout FAIL |
| TE4–TE5 | [TE4](TE4_RESULT_FAIL.md), [TE5](TE5_RESULT.md) | Prepared mechanisms succeed; historical cold failures preserved |
| Fresh-2 autonomous cycle | [Result](AUTONOMOUS_CYCLE_FRESH2_RESULT.md), [protocol](AUTONOMOUS_CYCLE_FRESH2_PROTOCOL.md) | Bounded same-family development PASS, 273/320; earlier first-attempt FAIL preserved |
| Two-stage / variable-depth sensing | [Two-stage protocol](STRUCTURE1_PROTOCOL.md), [variable-depth protocol](MULTISTEP_CAUSAL_SENSING_PROTOCOL.md), [result](MULTISTEP_CAUSAL_SENSING_RESULT.md) | Mechanism 12/12; open end-to-end transfer 2/4, overall criterion FAIL |
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
- [Current branch integration](BRANCH_INTEGRATION_20261009.md): main/research synchronization and pinned archived baseline.
- [Local project ideas](LOCAL_PROJECT_IDEAS.md): provenance of the earlier design comparison and implemented follow-up.
- [Historical README](HISTORICAL_README.md) and [historical development ladder](ROADMAP.md): earlier narrative and milestones. Their “current” labels belong to those historical stages; consult the project map for today's next steps.
