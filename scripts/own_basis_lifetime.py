"""Teach one initially untrained organism across real images and raw signals.

Explicit supervised lessons are NOT autonomous physical knowledge acquisition.
No test labels enter teaching. The same protected U1 performs every answer.
Growth/fixed arms receive the same lessons and per-lesson work ceiling.
"""
from __future__ import annotations
import argparse, gzip, hashlib, json, shutil, subprocess
from pathlib import Path
import numpy as np
from external5_sensors import load_fashion, load_har, metrics
from external5_transport import Native
from learning_world_stream import compact, save_checkpoint, source_hash


def frame(kind, x):
    if kind == 'har':
        return {'kind': 'signals', 'namespace': 2, 'channels': x.tolist()}
    return {'kind': 'image', 'namespace': 1 if kind == 'fashion' else 3, 'pixels': x.tolist()}


def load(root, kind, part, offset, count):
    if kind == 'har': return load_har(root, part, offset, count)
    return load_fashion(root / 'mnist' if kind == 'mnist' else root, part, offset, count)


def initial(binary, sample, grow):
    requests = [{'op': 'organism_init', 'frame': sample},
                {'op': 'organism_enable_own_basis', 'frame': sample,
                 'config': {'grow': grow}}, {'op': 'checkpoint'}, {'op': 'quit'}]
    process = subprocess.run([str(binary), '--jsonl'], input=''.join(compact(r)+'\n' for r in requests),
                             capture_output=True, text=True, timeout=120, check=True)
    responses = [json.loads(line) for line in process.stdout.splitlines()]
    if any(r['type'] == 'error' for r in responses): raise RuntimeError(responses)
    return next(r['checkpoint'] for r in responses if r['type'] == 'checkpoint')


def command(agent, request):
    response = agent.command(request)
    agent.trace.write({'kind': 'runtime_command', 'request': request, 'expected': response})
    return response


def predictions(agent, skill, xs):
    result = []
    for x in xs:
        p = command(agent, {'op': 'basis_predict', 'skill': skill, 'frame': frame(skill, x)})['prediction']
        result.append(p['probabilities'])
    return result


def score(agent, skill, xs, ys):
    classes = 6 if skill == 'har' else 10
    command(agent, {'op': 'basis_goal', 'goal': {'skill': skill,
        'answers': [{'motor': i, 'concept': str(i)} for i in range(classes)]}})
    readouts, actions, disagreements = [], [], 0
    for index, (x, target) in enumerate(zip(xs, ys)):
        observation = frame(skill, x)
        p = command(agent, {'op': 'basis_predict', 'skill': skill, 'frame': observation})['prediction']['probabilities']
        readout = int(max(p, key=lambda k: (p[k], -int(k))))
        agent.reset(observation, skill=skill, evaluation_index=index, teaching=False)
        action, _ = agent.step([i < classes for i in range(16)], 1,
            lambda a: {'frame': observation, 'cost': 1, 'reward': 0.0})
        readouts.append(readout);actions.append(action);disagreements += action != readout
    return {'readout': metrics(ys, readouts, classes), 'native_u1': metrics(ys, actions, classes),
            'readout_u1_disagreements': disagreements, 'targets': ys.tolist(),
            'readout_predictions': readouts, 'native_predictions': actions}


def run(a):
    binary, root, output = Path(a.binary).resolve(), Path(a.data).resolve(), Path(a.output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    repo = Path(__file__).resolve().parents[1]
    if not a.develop:
        status = subprocess.check_output(['git', 'status', '--porcelain'], cwd=repo, text=True)
        if status.strip(): raise RuntimeError('Evaluation requires a committed, clean source tree')
    tasks = a.tasks.split(',')
    assert set(tasks) <= {'fashion', 'har', 'mnist'} and len(set(tasks)) == len(tasks)
    train_count = a.train
    # Development probes are TRAIN examples, disjoint from these teaching ranges.
    # Evaluation probes are fixed ends of official TEST partitions. Earlier
    # project work has used these corpora; this is no blind new-corpus claim.
    probes = {k: load(root, k, 'train' if a.develop else 'test',
        6000 if a.develop else (2947-a.probe if k == 'har' else 10000-a.probe), a.probe)[:2] for k in tasks}
    teaching = {k: load(root, k, 'train', 0, train_count)[:2] for k in tasks}
    manifest = {'schema': 'aeterna.own-basis.real-teaching.v1', 'develop': a.develop,
                'pretrained_weights': False, 'feedback': 'explicit_supervised_labels',
                'tasks': tasks, 'train_count_each': train_count, 'probe_count_each': a.probe,
                'teaching_permutation_seed': 20261022, 'work_ceiling_per_teaching_event': 8_000_000,
                'binary_sha256': source_hash(binary), 'git_commit': subprocess.check_output(
                    ['git', 'rev-parse', 'HEAD'], cwd=Path(__file__).resolve().parents[1], text=True).strip(),
                'dataset_files': {str(p.relative_to(root)): source_hash(p) for p in sorted(root.rglob('*'))
                    if p.is_file() and (p.suffix == '.gz' or p.name == 'uci-har.zip')}}
    (output/'protocol.json').write_text(json.dumps(manifest, indent=2)+'\n')
    first_frame = frame(tasks[0], teaching[tasks[0]][0][0])
    results = {}
    for arm, grow in [('growing', True), ('fixed', False)]:
        dest = output/arm;dest.mkdir()
        start = initial(binary, first_frame, grow)
        save_checkpoint(dest/'starting-organism.json.gz', start)
        agent = Native(binary, dest/'life.jsonl.gz', 'organism', checkpoint=start)
        arm_result = {'stages': []}; retained = {}
        try:
            for skill in tasks:
                xs, ys = teaching[skill];order = np.random.default_rng(20261022).permutation(len(ys))
                for index, j in enumerate(order):
                    command(agent, {'op': 'basis_teach', 'skill': skill, 'frame': frame(skill, xs[j]),
                        'label': str(int(ys[j])), 'source_id': f'{skill}:train:{j}'})
                    if (index+1)%512 == 0:
                        report = command(agent, {'op': 'basis_report'})['report']
                        print(json.dumps({'arm': arm, 'skill': skill, 'taught': index+1,
                            'features': report['features'], 'admitted': report['admitted_growth'],
                            'last_growth': report['last_growth']}), flush=True)
                before_eval = command(agent, {'op': 'basis_report'})['report']
                retention = {}
                for old, saved in retained.items():
                    now = predictions(agent, old, probes[old][0])
                    retention[old] = {'probabilities_identical': now == saved,
                        'changed_examples': sum(p != q for p, q in zip(now, saved))}
                    assert now == saved, 'Previously taught goal changed during another goal'
                result = score(agent, skill, *probes[skill])
                # Evaluation feedback is withheld from the entire runtime:
                # factual POSTs have neutral reward. Labels exist only in the
                # external scoring harness. Do not freeze the life between goals.
                retained[skill] = predictions(agent, skill, probes[skill][0])
                checkpoint = agent.checkpoint()
                save_checkpoint(dest/f'after-{skill}.json.gz', checkpoint)
                agent.close()
                agent = Native(binary, dest/f'resume-{skill}.jsonl.gz', 'organism', checkpoint=checkpoint)
                assert agent.checkpoint() == checkpoint
                arm_result['stages'].append({'skill': skill, 'scores': result,
                    'old_skill_retention': retention, 'basis': before_eval, 'checkpoint_resume_identical': True})
                print(json.dumps({'arm': arm, 'stage': skill, 'scores': result,
                    'retention': retention, 'features': before_eval['features']}), flush=True)
            # Return to all skills in one body, then establish immutable final
            # knowledge for subsequent deployment/evaluation.
            arm_result['return_scores'] = {k: score(agent, k, *probes[k]) for k in tasks}
            command(agent, {'op': 'freeze'})
            arm_result['final_basis'] = command(agent, {'op': 'basis_report'})['report']
            arm_result['physical_commits'] = sum(s['scores']['native_u1']['samples'] for s in arm_result['stages']) + len(tasks)*a.probe
            final = agent.checkpoint();save_checkpoint(dest/'one-organism.json.gz', final)
            arm_result['checkpoint_sha256'] = hashlib.sha256(final.encode()).hexdigest()
            basis_state = json.loads(final)['cognition']['own_basis']
            arm_result['growth_receipts'] = basis_state['receipts']
            arm_result['program_library'] = [{
                'coordinates': len(library['coordinates']),
                'programs': [{k: v for k, v in program.items() if k != 'weights'} |
                             {'parameters': len(program['weights'])} for program in library['programs']]}
                for library in basis_state['libraries'].values()]
            # Join ordered transport segments into one replayable life. A
            # save/load round trip between segments changes no learned state.
            agent.close()
            first_segment = dest/'segment-start.jsonl.gz'
            (dest/'life.jsonl.gz').rename(first_segment)
            segments = [first_segment] + [dest/f'resume-{k}.jsonl.gz' for k in tasks]
            with gzip.open(dest/'life.jsonl.gz', 'wb') as combined:
                for segment in segments:
                    with gzip.open(segment, 'rb') as source: shutil.copyfileobj(source, combined)
            results[arm] = arm_result
            (output/'results.json').write_text(json.dumps(results, indent=2)+'\n')
        finally: agent.close()
    results['comparison'] = {k: {
        'growing_u1_accuracy': results['growing']['return_scores'][k]['native_u1']['accuracy'],
        'fixed_u1_accuracy': results['fixed']['return_scores'][k]['native_u1']['accuracy']}
        for k in tasks}
    (output/'results.json').write_text(json.dumps(results, indent=2)+'\n')
    return results['comparison']


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True);p.add_argument('--data', required=True);p.add_argument('--output', required=True)
    p.add_argument('--tasks', default='fashion,har,mnist');p.add_argument('--train', type=int, default=3072)
    p.add_argument('--probe', type=int, default=256);p.add_argument('--develop', action='store_true')
    args = p.parse_args()
    if not 512 <= args.train <= 4096 or not 32 <= args.probe <= 512: p.error('unsupported fixed ranges')
    print(json.dumps(run(args), indent=2))
