#!/usr/bin/env python3
"""Retain the entire frozen CAUSAL4 engineering comparison, including failures."""
import argparse
import concurrent.futures
import datetime
import gzip
import hashlib
import json
import platform
import queue
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--workers', type=int, default=4)
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = args.output.resolve()
    assert 1 <= args.workers <= 8
    if output.exists():
        raise SystemExit('Refusing to overwrite existing evaluation evidence')
    if subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT).strip():
        raise SystemExit('Commit the implementation and protocol before held scoring')
    output.mkdir(parents=True)
    identity = {
        'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'binary_sha256': digest(binary),
        'platform': platform.platform(),
        'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'seeds': [240000, 240031],
        'workers': args.workers,
        'process_isolation': 'one seed per process, fresh runtime per arm/family',
        'independent_external_qualification': False,
    }
    # Persist commitment BEFORE the executable sees the held worlds.
    (output / 'commitment.json').write_text(json.dumps(identity, indent=2) + '\n')
    groups = {}
    seen = set()
    trace_path = output / 'actual_traces.jsonl.gz'
    messages = queue.Queue(maxsize=32)

    def worker(seed):
        try:
            with (output / f'progress-{seed}.log').open('w') as log:
                with subprocess.Popen([str(binary), '--first', str(seed), '--count', '1'],
                    cwd=ROOT, stdout=subprocess.PIPE, stderr=log, text=True, bufsize=1) as process:
                    for line in process.stdout:
                        messages.put(('row', line))
                    messages.put(('exit', process.wait()))
        except Exception as error:
            messages.put(('error', repr(error)))

    exits = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool, gzip.open(trace_path, 'wt') as traces:
        futures = [pool.submit(worker, seed) for seed in range(240000, 240032)]
        while len(exits) < 32:
            kind, payload = messages.get()
            if kind == 'error':
                exits.append(payload)
            elif kind == 'exit':
                exits.append(payload)
            else:
                line = payload
                row = json.loads(line)
                key = (row['seed'], row['family'], row['arm'])
                assert key not in seen, key
                seen.add(key)
                assert all(event[5] == 1 for event in row['trace'])
                assert sum(not event[0] for event in row['trace']) == row['held_actions']
                assert sum(not event[0] and event[6] for event in row['trace']) == row['held_goals']
                group = groups.setdefault((row['family'], row['arm']),
                    dict(worlds=0, goals=0, episodes=0, actions=0, queries=0, training_actions=0))
                for field, value in [('worlds', 1), ('goals', row['held_goals']),
                    ('episodes', row['held_episodes']), ('actions', row['held_actions']),
                    ('queries', row['held_queries']),
                    ('training_actions', len(row['trace']) - row['held_actions'])]:
                    group[field] += value
                traces.write(line)
                if len(seen) % 12 == 0:
                    traces.flush()
                    print(f'{len(seen)}/384 complete', flush=True)
        for future in futures:
            future.result()
    assert exits == [0] * 32, f'evaluator exits {exits}; partial traces retained'
    expected = {(seed, family, arm) for seed in range(240000, 240032)
        for family in ['ten_step', 'noisy_sensor', 'uninformative_sensor', 'direct_goal']
        for arm in ['legacy', 'causal', 'horizon_one']}
    assert seen == expected and all(v['worlds'] == 32 for v in groups.values())

    def score(family, arm):
        row = groups[family, arm]
        return row['goals'] / row['episodes']

    gates = {
        'ten_step_at_least_24_of_32': groups['ten_step', 'causal']['goals'] >= 24,
        'ten_step_better_than_horizon_one': score('ten_step', 'causal') > score('ten_step', 'horizon_one'),
        'noisy_sensor_at_least_75_percent': score('noisy_sensor', 'causal') >= .75,
        'noisy_sensor_at_least_10_points_over_legacy': score('noisy_sensor', 'causal') >= score('noisy_sensor', 'legacy') + .10,
        'direct_goal_nonregression_2_points': score('direct_goal', 'causal') >= score('direct_goal', 'legacy') - .02,
    }
    result = {
        **identity, 'completed_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'verdict': 'ENGINEERING_PASS' if all(gates.values()) else 'ENGINEERING_FAIL',
        'gates': gates,
        'groups': [{**dict(family=f, arm=a), **v} for (f, a), v in sorted(groups.items())],
        'traces_sha256': digest(trace_path),
    }
    (output / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    if not all(gates.values()):
        raise SystemExit(1)


if __name__ == '__main__':
    main()
