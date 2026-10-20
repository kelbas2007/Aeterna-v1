#!/usr/bin/env python3
"""Retain the entire frozen CAUSAL4 engineering comparison, including failures."""
import argparse
import datetime
import gzip
import hashlib
import json
import platform
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = args.output.resolve()
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
        'seeds': [230000, 230031],
        'independent_external_qualification': False,
    }
    # Persist commitment BEFORE the executable sees the held worlds.
    (output / 'commitment.json').write_text(json.dumps(identity, indent=2) + '\n')
    groups = {}
    seen = set()
    trace_path = output / 'actual_traces.jsonl.gz'
    with (output / 'progress.log').open('w') as log, gzip.open(trace_path, 'wt') as traces:
        with subprocess.Popen([str(binary)], cwd=ROOT, stdout=subprocess.PIPE,
                              stderr=log, text=True, bufsize=1) as process:
            for line in process.stdout:
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
            code = process.wait()
    assert code == 0, f'evaluator exited {code}; partial traces retained'
    assert len(seen) == 384 and all(v['worlds'] == 32 for v in groups.values())

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
