"""Read-only post-hoc provenance audit of an already consumed EXTERNAL5 exam.

Never starts an agent, changes a checkpoint, or creates a fresh evaluation.
"""
from __future__ import annotations
import argparse, gzip, hashlib, json
from datetime import datetime, timezone
from pathlib import Path
from external5_sensors import load_fashion, load_har, blank, acquire
from run_external_continual5 import verify_data_files


def audit_trace(path, motor_maps, datasets, expected_contexts, variant):
    frames = {}; context = None; current = None; pending = None
    measured = set(); failed = set(); prediction = None; actions = windows = 0
    left = 0; finished = retry_required = awaiting_commit = False
    with gzip.open(path, 'rt') as stream:
        for line in stream:
            row = json.loads(line); event = row['kind']
            if awaiting_commit and event not in ['sensor_frame', 'commit']:
                raise ValueError('missing native factual commit')
            if event == 'sensor_frame':
                raw = json.dumps(row['frame'], separators=(',', ':'), allow_nan=False).encode()
                if hashlib.sha256(raw).hexdigest() != row['id'] or row['id'] in frames:
                    raise ValueError('invalid or duplicate sensory frame hash')
                frames[row['id']] = row['frame']
            elif event == 'initial' or (event == 'restart' and not row['metadata'].get('dataset')):
                if context is not None or windows or actions or frames[row['frame_ref']] != blank(expected_contexts[0][0]):
                    raise ValueError('unexpected initial sensory state')
                if event == 'initial' and row['variant'] != variant:
                    raise ValueError('initial controller differs from trace variant')
            elif event == 'restart':
                meta = row['metadata']
                if meta.get('corrective_retry'):
                    if not retry_required or context is None or (meta['dataset'], meta['index']) != context[:2]:
                        raise ValueError('invalid corrective retry')
                    if frames[row['frame_ref']] != current:
                        raise ValueError('corrective retry granted unmeasured data')
                    retry_required = False
                else:
                    next_context = (meta['dataset'], meta['index'], meta['training'])
                    if context is not None or windows >= len(expected_contexts) or next_context != expected_contexts[windows]:
                        raise ValueError('sample sequence has a duplicate, gap, or missing phase')
                    context = next_context; current = blank(meta['dataset'])
                    measured = set(); failed = set(); prediction = None; finished = False
                    kind, _, training = context
                    left = (11 if kind == 'fashion' else 15) if training else (2 if kind == 'fashion' else 5)
                    if frames[row['frame_ref']] != current:
                        raise ValueError('new sample contains free measurements')
            elif event == 'intent':
                if context is None or pending is not None or finished or retry_required or frames[row['pre_ref']] != current:
                    raise ValueError('intention has wrong sensory context')
                if row['remaining_actions'] != left or left < 1 or row['index'] != actions:
                    raise ValueError('intention deadline or action index mismatch')
                kind, index, training = context; queries, answers = motor_maps[kind]
                allowed = [False] * 16
                for axis, motor in enumerate(queries):
                    allowed[motor] = axis not in measured and left > 1
                for motor in answers:
                    allowed[motor] = motor not in failed and (kind != 'fashion' or bool(measured))
                if [offer['available'] for offer in row['offers']] != allowed or any(offer['max_cost'] != 1 for offer in row['offers']):
                    raise ValueError('offered motors disagree with acquired measurements')
                action = row['action']
                if type(action) is not int or action not in range(16) or not allowed[action]:
                    raise ValueError('selected sensory action is unavailable')
                pending = row
            elif event == 'actual_post':
                if pending is None or row['action'] != pending['action'] or row['index'] != actions:
                    raise ValueError('unmatched sensory action')
                kind, index, training = context; sample, labels, _ = datasets[kind, training]
                queries, answers = motor_maps[kind]; action = row['action']; target = int(labels[index])
                if action in queries:
                    axis = queries.index(action); current = acquire(current, axis, sample[index])
                    measured.add(axis); reward = 0.0
                else:
                    prediction = answers.index(action); reward = float(prediction == target)
                    if training and not reward: failed.add(action)
                if frames[row['frame_ref']] != current or row['reward'] != reward or row['cost'] != 1:
                    raise ValueError('actual raw sample, reward, or cost differs from the verified corpus')
                pending = None; actions += 1; left -= 1; awaiting_commit = variant != 'random'
                finished = action in answers and (not training or reward > 0 or left == 0)
                retry_required = action in answers and training and not reward and left > 0
            elif event == 'commit':
                if not awaiting_commit or row['index'] != actions - 1 or row['step']['action'] != action:
                    raise ValueError('unexpected native factual commit')
                awaiting_commit = False
            elif event == 'graded_episode':
                if context is None or not finished or prediction is None or pending is not None:
                    raise ValueError('grade without one completed active sample')
                kind, index, training = context
                if (row['dataset'], row['index'], row['training']) != context:
                    raise ValueError('graded sample context mismatch')
                if row['target_after_response'] != int(datasets[kind, training][1][index]):
                    raise ValueError('graded feedback differs from the verified corpus')
                if row['prediction'] != prediction or row['measurements'] != len(measured):
                    raise ValueError('graded outcome differs from the actual actions')
                windows += 1; context = None
            else:
                raise ValueError('unexpected sensory trace event: ' + event)
    if pending is not None or context is not None or awaiting_commit or windows != len(expected_contexts):
        raise ValueError('incomplete sensory trace or sample coverage')
    return {'status': 'PASS', 'actual_actions': actions, 'graded_windows': windows}


def audit(data, evidence):
    frozen = json.loads((evidence / 'source-freeze.json').read_text())
    commitment = json.loads((evidence / 'sensor-study/commitment.json').read_text())
    if any(frozen[field] != commitment[field] for field in ['source_commit', 'binary_sha256']):
        raise ValueError('source freeze and evaluated commitment identity mismatch')
    manifest = json.loads((evidence / 'manifest.json').read_text())
    records = [r for r in manifest['files'] if 'path' in r]
    pins = {r['path']: r for r in records}
    if len(pins) != len(records):
        raise ValueError('duplicate published evidence entry')
    inputs = ['source-freeze.json', 'sensor-study/commitment.json', 'data-manifest.json', 'har-mirror-manifest.json']
    inputs += [str(p.relative_to(evidence)) for p in (evidence / 'traces/sensor-study/sensors').rglob('*.jsonl.gz')]
    for name in inputs:
        raw = (evidence / name).read_bytes(); pin = pins[name]
        if len(raw) != pin['bytes'] or hashlib.sha256(raw).hexdigest() != pin['sha256']:
            raise ValueError('published evidence bytes differ from manifest: ' + name)
    verified = verify_data_files(data)
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
        sequence = ([(kind, i, True) for kind in ['fashion', 'har'] for i in range(3000)]
                    if path.name == 'continuous-training.jsonl.gz'
                    else [('har' if path.name == 'har-after-har.jsonl.gz' else 'fashion', i, False) for i in range(1000)])
        reports.append({'path': str(path.relative_to(evidence)),
                        **audit_trace(path, commitment['motor_maps'], datasets, sequence, path.parent.name)})
    if verify_data_files(data) != verified:
        raise ValueError('dataset changed during audit')
    return {'status': 'PASS', 'audit_kind': 'post_hoc_read_only_provenance',
            'utc_after_completed_evaluation': datetime.now(timezone.utc).isoformat(),
            'evaluated_source_commit': frozen['source_commit'],
            'evaluated_binary_sha256': frozen['binary_sha256'],
            'verified_input_files': len(inputs),
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
