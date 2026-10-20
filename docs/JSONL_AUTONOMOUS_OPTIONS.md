# JSONL-интерфейс и приобретённые макродействия (20 октября 2026)

`aeterna-v1 --jsonl` — интерфейс к **доверенному виртуальному адаптеру**, а не самостоятельный доступ к роботам, ОС или интернету. Адаптер обязан исполнить выбранный AETERNA мотор и вернуть его **настоящий** POST; прогноз не является фактом.

## Пример порядка включения (до приобретения опыта)

```json
{"op":"init","width":1,"observation":[0]}
{"op":"enable_agency_aware_credit"}
{"op":"enable_self_directed_control"}
{"op":"enable_predictive_history_states"}
{"op":"enable_adaptive_history_suffix"}
{"op":"enable_temporal_options"}
{"op":"enable_autobiographical_options"}
{"op":"enable_variable_depth_options"}
{"op":"enable_complexity_need_gate"}
{"op":"enable_active_causal_probes"}
{"op":"step","goal":[1],"available":[true,true,true,true]}
```

После `step` программа печатает `{"type":"action","action":N}`. ВНЕШНИЙ МИР сначала исполняет N и отвечает, например:

```json
{"op":"post","result":{"observation":[0],"cost":1,"reward":0.0}}
```

Затем `committed` содержит реальный результат выбора и `complexity_need_decision`: `retain_simpler`, `investigate_insufficient_evidence` либо `extend_after_contradiction`. `report` возвращает данные когнитивной оценки; `checkpoint` сохраняет совместное состояние; `load` восстанавливает его в новом процессе. Прерывание, вето или изменение цели не могут выдать ложный успешный POST.

`init` сохраняет совместимые значения по умолчанию `planning_depth=2`,
`planning_work=50_000`. Теперь доступны явные поля `planning_depth` (1–32),
`planning_work`, `width` (1–32), `actions` (до 16 для самостоятельного
управления), `journal_capacity` и `max_external_cost`. Исторические результаты
3–5 действий использовали специальные Rust `Config` и остаются привязаны к
исходным версиям. Новый [режим причинной памяти](CAUSAL_CONTINUAL4.md)
показывает настройку длинного горизонта, платных моторов и уплотнения журнала.

Для воспроизведения инженерного механизма — [cold autobiography test](../tests/cold_autobiographical_option_development.rs), [variable-depth test](../tests/cold_variable_depth_option_development.rs), [necessity-gate test](../tests/complexity_necessity_development.rs). Замороженный [SELF-ORIGINATED-OPTION1](SELF_ORIGINATED_OPTION1_ONE_USE_RESULT.md) используется **только** как историческое научное доказательство; повторять его для нового seed нельзя.

[Живой статус](CURRENT_STATUS_20261020.md) · [исторические пределы](COMPLEXITY_NECESSITY1_DEVELOPMENT_RESULT.md).
