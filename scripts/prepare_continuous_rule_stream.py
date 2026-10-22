"""Turn pinned NAB recordings into causal, label-free physical observations.

The development path parses only the declared prefix. The evaluation path
must be used after freezing the evaluated native source and executable.
Source names and timestamps remain transport provenance, outside SensorFrame.
"""
import argparse
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path


def frame(value):
    return {'kind': 'signals', 'namespace': 80,
            'channels': [[value], *([None] * 8)]}


def prepare(manifest_path, data_root, split, output):
    manifest = json.loads(Path(manifest_path).read_text())
    count = 0
    with Path(output).open('x') as target:
        for entry in manifest['files']:
            path = Path(data_root) / (entry['group'] + '.csv')
            with path.open('rb') as data:
                digest = hashlib.file_digest(data, 'sha256').hexdigest()
            if digest != entry['sha256']:
                raise ValueError(f'Pinned source digest differs: {path}')
            boundary = math.floor(entry['rows'] * entry['development_fraction'])
            start, end = ((0, boundary) if split == 'development'
                          else (boundary - 1, entry['rows']))
            with path.open(newline='') as data:
                selected = itertools.islice(enumerate(csv.DictReader(data)), start, end)
                previous = None
                first = True
                used = 0
                for index, row in selected:
                    value = float(row['value'])
                    if not math.isfinite(value) or abs(value) > 1_000_000:
                        raise ValueError('Observation is outside the native signal contract')
                    used += 1
                    if previous is not None:
                        event = {
                            'pre': frame(previous), 'post': frame(value),
                            'action': 0, 'reward': 0.0, 'cost': 1,
                            'episode_start': first,
                            'provenance': {
                                'repository': manifest['repository'], 'path': entry['path'],
                                'revision': entry['ref'], 'row': index,
                                'timestamp': row['timestamp'],
                                'development': split == 'development',
                                'labels_accessed': False,
                            },
                        }
                        target.write(json.dumps(event, separators=(',', ':'), allow_nan=False) + '\n')
                        count += 1
                        first = False
                    previous = value
                if used != end - start:
                    raise ValueError(f'Truncated pinned source: {path}')
    with Path(output).open('rb') as data:
        digest = hashlib.file_digest(data, 'sha256').hexdigest()
    return {'split': split, 'transitions': count, 'sha256': digest,
            'historical_physical_actions_controlled': 0, 'labels_accessed': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', required=True)
    parser.add_argument('--data-root', required=True)
    parser.add_argument('--split', choices=['development', 'evaluation'], required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    print(json.dumps(prepare(args.manifest, args.data_root, args.split, args.output), indent=2))
