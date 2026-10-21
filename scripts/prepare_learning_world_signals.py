"""Make a sensor-playback stream from real UCI HAR training recordings.

No activity labels or subject IDs are decoded or sent to the organism. Each
record reveals the second half of a real 128-sample window after the first.
Action 0 is a playback/read operation, NOT an invented physical robot action.
"""
import argparse
import hashlib
import json
import zipfile
from pathlib import Path

import numpy as np
from external5_sensors import SENSOR_NAMES
from prepare_external5_data import verify_har_archive


def build(archive, output, offset=0, count=64):
    if offset < 0 or count < 1:
        raise ValueError('invalid source range')
    archive, output = Path(archive), Path(output)
    verify_har_archive(archive)
    source_hash = hashlib.sha256(archive.read_bytes()).hexdigest()
    with zipfile.ZipFile(archive) as source:
        channels = []
        for sensor in SENSOR_NAMES:
            with source.open(f'UCI HAR Dataset/train/Inertial Signals/{sensor}_train.txt') as member:
                channels.append(np.loadtxt(member, max_rows=offset + count, ndmin=2)[offset:offset + count])
    values = np.stack(channels, axis=1)
    if values.shape != (count, 9, 128) or not np.isfinite(values).all():
        raise ValueError('Invalid real raw signal window range')

    def frame(window):
        return {'kind': 'signals', 'namespace': 70, 'channels': window.tolist()}

    with output.open('x', encoding='utf-8') as target:
        for index, window in enumerate(values):
            row = {'pre': frame(window[:, :64]), 'post': frame(window[:, 64:]),
                   'action': 0, 'reward': 0.0, 'cost': 1, 'episode_start': True,
                   'provenance': {'source': 'UCI HAR original raw training recording',
                                  'archive_sha256': source_hash, 'window': offset + index,
                                  'pre_samples': [0, 64], 'post_samples': [64, 128],
                                  'action_semantics': 'advance recorded sensor playback; no original actuator labels',
                                  'label_access': False, 'independent_qualification': False}}
            target.write(json.dumps(row, separators=(',', ':'), allow_nan=False) + '\n')
    return {'records': count, 'source_sha256': source_hash, 'labels_decoded': False,
            'stream_sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
            'recorded_physical_actions_available': False}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--archive', required=True)
    p.add_argument('--output', required=True)
    p.add_argument('--offset', type=int, default=0)
    p.add_argument('--count', type=int, default=64)
    a = p.parse_args()
    print(json.dumps(build(a.archive, a.output, a.offset, a.count), indent=2))
