"""Score pre-POST native forecasts and causal references on recorded streams.

Dataset names are used for reporting only. The native input has one common
physical interface and receives none of these names or anomaly labels.
"""
import argparse
import gzip
import json
from pathlib import Path

import numpy as np


def norm(value):
    return value / (1 + abs(value))


def references(rows):
    history = []
    weights = np.zeros(8)
    covariance = np.eye(8) * 10
    predictions = []
    for row in rows:
        before = row['pre']['channels'][0][0]
        actual = row['post']['channels'][0][0]
        if row.get('episode_start') or not history:
            history = [before]
            weights = np.zeros(8)
            weights[0] = 1
            covariance = np.eye(8) * 10
        last = norm(before)
        mean = norm(float(np.mean(history[-8:])))
        ar = last
        if len(history) >= 8:
            x = np.array([norm(v) for v in history[-8:][::-1]])
            ar = float(np.clip(x @ weights, -1, 1))
            projected = covariance @ x
            gain = projected / (0.999 + x @ projected)
            weights += gain * (norm(actual) - x @ weights)
            covariance = (covariance - np.outer(gain, projected)) / 0.999
        predictions.append({'last': last, 'mean8': mean, 'ar8': ar})
        history.append(actual)
        if len(history) > 256:
            history.pop(0)
    return predictions


def native_predictions(run):
    summary = json.loads((run / 'summary.json').read_text())
    predictions = {}
    operations = set()
    with gzip.open(run / 'life.jsonl.gz', 'rt') as events:
        for line in events:
            event = json.loads(line)
            if event['kind'] == 'runtime_command' and event['request']['op'] in ('world_prediction', 'temporal_prediction'):
                operations.add(event['request']['op'])
                index = event['recorded_source_index']
                assert index not in predictions, 'Duplicate forecast for a factual transition'
                predictions[index] = event['expected']['prediction']['values']['s:0:1:0']
    assert len(predictions) == summary['records_ingested']
    assert operations == {'world_prediction'}, 'Qualification requires the common world forecast in every arm'
    return summary, predictions


def analyze(events, runs, seed=20261022):
    rows = [json.loads(line) for line in Path(events).open()]
    basic = references(rows)
    forecasts = {}
    summaries = {}
    for name, directory in runs.items():
        summaries[name], forecasts[name] = native_predictions(Path(directory))
    indices = sorted(set.intersection(*(set(p) for p in forecasts.values())))
    assert indices and all(set(f) == set(indices) for f in forecasts.values())
    errors = {}
    for index in indices:
        row = rows[index]
        group = row['provenance']['path']
        if group not in errors:
            errors[group] = {name: [] for name in ['last', 'mean8', 'ar8', *forecasts]}
        actual = norm(row['post']['channels'][0][0])
        for name, value in basic[index].items():
            errors[group][name].append((value - actual) ** 2)
        for name, values in forecasts.items():
            errors[group][name].append((values[index] - actual) ** 2)
    sources = {source: {'observations': len(values['last']),
                        'mse': {name: float(np.mean(v)) for name, v in values.items()}}
               for source, values in errors.items()}
    relative = {name: float(np.mean([s['mse'][name] / max(s['mse']['last'], 1e-30)
                                    for s in sources.values()]))
                for name in ['last', 'mean8', 'ar8', *forecasts]}
    best = min(['last', 'mean8', 'ar8'], key=relative.get)
    work = {}
    for name, summary in summaries.items():
        work[name] = (summary['final_report']['organism']['computation_work']
                      - summary['initial_report']['organism']['computation_work'])
    result = {'scope': 'recorded real-data forecasting; no physical control or general-intelligence qualification',
              'forecast_operation': 'world_prediction',
              'sources': sources, 'mean_source_relative_mse': relative,
              'best_single_reference': best, 'whole_organism_work': work,
              'rule_reports': {name: summary['final_report']['organism']['learning_world']['temporal']
                               for name, summary in summaries.items()},
              'pre_post_forecasts': len(indices)}
    if 'full' in forecasts and 'fixed' in forecasts:
        result['criteria'] = {
            'beats_best_reference_by_five_percent': relative['full'] <= 0.95 * relative[best],
            'no_source_ten_percent_regression_vs_last': all(s['mse']['full'] <= 1.10 * s['mse']['last'] for s in sources.values()),
            'improved_acquisition_by_five_percent': relative['full'] <= 0.95 * relative['fixed'],
            'work_within_one_point_five': work['full'] <= 1.5 * work['fixed'],
            'recursive_producer_admitted': result['rule_reports']['full']['search']['policy_generations'] > 0,
            'acquired_learning_program_executed': result['rule_reports']['full']['rule_admissions'] > 0 and result['rule_reports']['full'].get('accepted_rule_uses', 0) > 0,
        }
        result['qualified'] = len(sources) == 5 and all(result['criteria'].values())
        # Resample contiguous blocks within each source, keeping algorithms
        # paired; this interval describes these streams, not all real worlds.
        rng = np.random.default_rng(seed)
        draws = []
        block = 64
        for _ in range(2000):
            differences = []
            for values in errors.values():
                n = len(values['last'])
                starts = rng.integers(0, n, size=(n + block - 1) // block)
                sampled = ((starts[:, None] + np.arange(block)) % n).ravel()[:n]
                last = max(float(np.mean(np.array(values['last'])[sampled])), 1e-30)
                differences.append((float(np.mean(np.array(values['fixed'])[sampled]))
                                    - float(np.mean(np.array(values['full'])[sampled]))) / last)
            draws.append(float(np.mean(differences)))
        result['paired_time_block_interval'] = {'block': block, 'resamples': 2000,
                                               'fixed_minus_full_relative_mse_95': np.quantile(draws, [0.025, 0.975]).tolist(),
                                               'independent_generalization_claim': False}
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--events', required=True)
    parser.add_argument('--full', required=True)
    parser.add_argument('--fixed', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    result = analyze(args.events, {'full': args.full, 'fixed': args.fixed})
    Path(args.output).write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'rule_reports'}, indent=2))
