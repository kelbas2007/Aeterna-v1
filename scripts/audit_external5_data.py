"""Read-only post-hoc provenance audit of an already consumed EXTERNAL5 exam.

Never starts an agent, changes a checkpoint, or creates a fresh evaluation.
"""
from __future__ import annotations
import argparse, gzip, hashlib, json
from datetime import datetime, timezone
from pathlib import Path
from external5_sensors import load_fashion, load_har, blank, acquire
from run_external_continual5 import verify_data_files


def audit(data, evidence):
    verified = verify_data_files(data)
    frozen = json.loads((evidence / 'source-freeze.json').read_text())
    commitment = json.loads((evidence / 'sensor-study/commitment.json').read_text())
    manifest_bytes = (data / 'manifest.json').read_bytes()
    if hashlib.sha256(manifest_bytes).hexdigest() != commitment['data_manifest_sha256']:
        raise ValueError('current dataset manifest differs from the evaluated commitment')
    if json.loads(manifest_bytes) != json.loads((evidence / 'data-manifest.json').read_text()):
        raise ValueError('published dataset manifest differs from the evaluated data')
    datasets = {}
    for kind, loader in [('fashion', load_fashion), ('har', load_har)]:
        for training, part, count in [(True, 'train', 3000), (False, 'test', 1000)]:
            datasets[kind, training] = loader(data, part, 0, count)

    reports = []
    traces = evidence / 'traces/sensor-study/sensors'
    expected = {traces / variant / (name + '.jsonl.gz')
                for variant in commitment['variants']
                for name in ['continuous-training', 'fashion-before-har', 'fashion-after-har', 'har-after-har']}
    if set(traces.rglob('*.jsonl.gz')) != expected:
        raise ValueError('missing or unexpected sensory trace')
    for path in sorted(expected):
        frames = {}; context = None; current = None; pending = None
        measured = set(); failed = set(); prediction = None; actions = windows = 0
        with gzip.open(path, 'rt') as stream:
            for line in stream:
                row = json.loads(line); event = row['kind']
                if event == 'sensor_frame':
                    frames[row['id']] = row['frame']
                elif event == 'restart' and row['metadata'].get('dataset'):
                    meta = row['metadata']
                    if meta.get('corrective_retry'):
                        if context is None or (meta['dataset'], meta['index']) != context[:2]:
                            raise ValueError('corrective retry changed the sample')
                        if frames[row['frame_ref']] != current:
                            raise ValueError('corrective retry granted unmeasured data')
                    else:
                        context = (meta['dataset'], meta['index'], meta['training'])
                        current = blank(meta['dataset']); measured = set(); failed = set()
                        prediction = None
                        if frames[row['frame_ref']] != current:
                            raise ValueError('new sample contains free measurements')
                elif event == 'intent':
                    if context is None or pending is not None or frames[row['pre_ref']] != current:
                        raise ValueError('intention has wrong sensory context')
                    kind, index, training = context
                    queries, answers = commitment['motor_maps'][kind]
                    allowed = [False] * 16
                    for axis, motor in enumerate(queries):
                        allowed[motor] = axis not in measured and row['remaining_actions'] > 1
                    for motor in answers:
                        allowed[motor] = motor not in failed and (kind != 'fashion' or bool(measured))
                    if [offer['available'] for offer in row['offers']] != allowed:
                        raise ValueError('offered motors disagree with acquired measurements')
                    pending = row
                elif event == 'actual_post':
                    if pending is None or row['action'] != pending['action']:
                        raise ValueError('unmatched sensory action')
                    kind, index, training = context
                    sample = datasets[kind, training][0][index]
                    target = int(datasets[kind, training][1][index])
                    queries, answers = commitment['motor_maps'][kind]; action = row['action']
                    if action in queries:
                        axis = queries.index(action)
                        current = acquire(current, axis, sample); measured.add(axis); reward = 0.0
                    else:
                        prediction = answers.index(action); reward = float(prediction == target)
                        if training and not reward: failed.add(action)
                    if frames[row['frame_ref']] != current or row['reward'] != reward or row['cost'] != 1:
                        raise ValueError('actual raw sample, reward, or cost differs from the verified corpus')
                    pending = None; actions += 1
                elif event == 'graded_episode':
                    kind, index, training = context
                    if (row['dataset'], row['index'], row['training']) != context:
                        raise ValueError('graded sample context mismatch')
                    if row['target_after_response'] != int(datasets[kind, training][1][index]):
                        raise ValueError('graded feedback differs from the verified corpus')
                    if row['prediction'] != prediction or row['measurements'] != len(measured):
                        raise ValueError('graded outcome differs from the actual actions')
                    windows += 1
        if pending is not None:
            raise ValueError('unfinished sensory action')
        expected_windows = 6000 if path.name == 'continuous-training.jsonl.gz' else 1000
        if windows != expected_windows:
            raise ValueError('incomplete sensory trace')
        reports.append({'path': str(path.relative_to(evidence)), 'status': 'PASS',
                        'actual_actions': actions, 'graded_windows': windows})
    if verify_data_files(data) != verified:
        raise ValueError('dataset changed during audit')
    return {'status': 'PASS', 'audit_kind': 'post_hoc_read_only_provenance',
            'utc_after_completed_evaluation': datetime.now(timezone.utc).isoformat(),
            'evaluated_source_commit': frozen['source_commit'],
            'verified_data_files': verified, 'trace_files': len(reports),
            'actual_actions': sum(r['actual_actions'] for r in reports),
            'graded_windows': sum(r['graded_windows'] for r in reports), 'traces': reports,
            'scope': 'All logged sensory observations, acquired masks, motor offers and factual reward labels match the verified original corpus. No agent or new exam was run.'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--data', required=True, type=Path)
    parser.add_argument('--evidence', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    result = audit(args.data, args.evidence)
    with args.output.open('x') as stream:
        json.dump(result, stream, indent=2); stream.write('\n')
    print(json.dumps({k: result[k] for k in ['status', 'trace_files', 'actual_actions', 'graded_windows']}))
