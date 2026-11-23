# Свидетельства автономного исследования

Проверенный исходный код: `df9257ac2c15be9a843409ad28b25adc5ab3ab86`.
Проверочный запуск: [45259082705, попытка 1](https://github.com/kelbas2007/Aeterna-v1/actions/runs/45259082705).
Итог: **SUPPORTED**, 16/16 корректных миров и 434 полные цепочки.

Полное объяснение результата, стоимости и ограничений находится в
[AUTONOMOUS_INQUIRY_20261123_RESULT.md](../../AUTONOMOUS_INQUIRY_20261123_RESULT.md).

| Материал | Содержание |
|---|---|
| [CI_RESULT.md](CI_RESULT.md) | Неизменённый отчёт оценщика из CI; все 16 миров |
| [summary.json](summary.json) | Фиксированные критерии, выводы и общие численные показатели |
| [independent-analysis.json](independent-analysis.json) | Повторное чтение итогов, ресурсы и открытые вопросы при восстановлении |
| [representative-chain-world9.json](representative-chain-world9.json) | Полные записи открытия вопроса 2382, диагностики и следующего успешного действия |
| [independent-integrity-audit.json](independent-integrity-audit.json) | Побайтное воспроизведение оценщиком и сверка 108 исходных файлов |
| [independent-retention-audit.json](independent-retention-audit.json) | Независимая проверка сохранения восьми исторических жизней |
| [retention-preservation.json](retention-preservation.json) | Квитанции обновления и повторной загрузки восьми полных состояний в CI |
| [CI_integrity.log](CI_integrity.log) | 32 целевые инженерные проверки и проверка обновления |
| [CI_run.log](CI_run.log) | Запуск 16 проверочных миров и первичная оценка |
| [provenance.txt](provenance.txt) | Исходный коммит и хеши исходных файлов и бинарных сборок |
| [archive/](archive/README.md) | Полные исходные траектории и контрольные точки; семь частей одного tar.xz |
| [local_development/](local_development/manifest.json) | Отдельно обозначенные разработочные и совместимые прогоны; не проверочная выборка |
| [EVIDENCE.json](EVIDENCE.json) | Происхождение и SHA256 этих копий |

Исходный архив из CI сохранён целиком. Повторный запуск оценщика не исполнял
новые действия организмов и воспроизвёл исходные результаты побайтно.
Реализация, оценщик и критерии после проверочного запуска не менялись.
