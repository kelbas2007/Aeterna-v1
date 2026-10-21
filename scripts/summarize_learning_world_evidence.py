"""Verify and summarize the open engineering readout of the shared learning life.

The signal baselines use PRE only, have no trainable parameters, and use the
same normalized target samples as the native prequential sensor error. This
readout is not an independent intelligence or equal-compute qualification.
"""
import argparse
import gzip
import hashlib
import json
import math
from pathlib import Path


def summarize(root):
    root = Path(root)

    def read(relative):
        return json.loads((root / relative).read_text())

    def checkpoint(relative):
        return hashlib.sha256(gzip.decompress((root / relative).read_bytes())).hexdigest()

    names = ('real-first', 'real-resumed', 'real-frozen', 'physics', 'physics-frozen')
    runs = {name: read(f'{name}/summary.json') for name in names}
    binary = read('identity.json')['binary_sha256']
    matched = 0
    for name, run in runs.items():
        assert run['binary_sha256'] == binary, f'{name}: binary changed'
        for filename, key in (('starting-organism', 'starting_checkpoint_sha256'),
                              ('one-organism', 'continuing_checkpoint_sha256')):
            assert checkpoint(f'{name}/{filename}.json.gz') == run[key], name
        proof = read(f'{name}/replay-proof.json')
        assert proof['status'] == 'PASS' and proof['full_state_byte_identical'], name
        assert proof['continuing_checkpoint_sha256'] == run['continuing_checkpoint_sha256'], name
        expected = run.get('records_ingested', sum(e['actions'] for e in run.get('episodes', [])))
        assert proof['native_commits_matched'] == expected, name
        matched += expected

    first, resumed, frozen = (runs[n] for n in names[:3])
    physics, physics_frozen = (runs[n] for n in names[3:])
    assert first['continuing_checkpoint_sha256'] == resumed['starting_checkpoint_sha256']
    assert first['starting_checkpoint_sha256'] == frozen['starting_checkpoint_sha256']
    assert resumed['continuing_checkpoint_sha256'] == physics['starting_checkpoint_sha256']
    assert resumed['continuing_checkpoint_sha256'] == physics_frozen['starting_checkpoint_sha256']
    assert frozen['learned_state_unchanged']

    def organism(run, which='final_report'):
        return run[which]['organism']

    def world(run):
        return organism(run)['learning_world']

    assert world(physics_frozen)['learned_fingerprint'] == world(resumed)['learned_fingerprint']
    assert organism(physics)['learner']['events'] == 17798 + 64 + 128
    assert world(physics)['factual_updates'] == 192
    assert first['offset'] == 0 and resumed['offset'] == first['next_offset'] == 32
    assert resumed['next_offset'] == frozen['next_offset'] == 64

    raw = gzip.decompress((root / 'real-signals.jsonl.gz').read_bytes())
    digest = hashlib.sha256(raw).hexdigest()
    assert all(runs[n]['source_sha256'] == digest for n in names[:3])
    errors = {k: [] for k in ('zero', 'repeat_first_half', 'last_sample', 'mean_pre')}
    records = [json.loads(line) for line in raw.decode().splitlines()]
    for record in records:
        for pre, post in zip(record['pre']['channels'], record['post']['channels']):
            pre = [x / (1 + abs(x)) for x in pre]
            post = [x / (1 + abs(x)) for x in post]
            assert len(pre) == len(post) == 64
            predictions = {
                'zero': [0.0] * len(post), 'repeat_first_half': pre,
                'last_sample': [pre[-1]] * len(post),
                'mean_pre': [math.fsum(pre) / len(pre)] * len(post),
            }
            for name, predicted in predictions.items():
                errors[name].extend((x - y) ** 2 for x, y in zip(predicted, post))
    mse = {name: math.fsum(values) / len(values) for name, values in errors.items()}
    count = len(errors['zero'])
    assert len(records) == 64 and world(resumed)['prequential_sensor_count'] == count
    assert world(frozen)['prequential_sensor_count'] == count
    learned_mse = world(resumed)['prequential_sensor_mse']
    frozen_mse = world(frozen)['prequential_sensor_mse']
    parent_work = organism(first, 'initial_report')['computation_work']
    learned_work = organism(resumed)['computation_work'] - parent_work
    frozen_work = organism(frozen)['computation_work'] - parent_work
    physics_work = organism(physics)['computation_work'] - organism(resumed)['computation_work']
    physics_frozen_work = organism(physics_frozen)['computation_work'] - organism(resumed)['computation_work']
    return {
        'status': 'ENGINEERING_LOOP_IMPLEMENTED_BEHAVIORAL_ADVANTAGE_NOT_ESTABLISHED',
        'independent_intelligence_qualification': False,
        'source_commit': read('identity.json')['source_commit'],
        'binary_sha256': binary,
        'primary_life': {
            'parent_events': 17798, 'after_real_first': organism(first)['learner']['events'],
            'after_real_resumed': organism(resumed)['learner']['events'],
            'after_simulator': organism(physics)['learner']['events'],
            'original_physical_experiments': 0,
            'recorded_sensor_ingestions': 64, 'self_selected_simulator_actions': 128,
            'online_updates': world(physics)['online_updates'],
            'consolidation_updates': world(physics)['consolidation_updates'],
            'retained_experiences': world(physics)['retained_experiences'],
            'reserved_memory_bytes': world(physics)['reserved_memory_bytes'],
            'budget_skipped_updates': world(physics)['budget_skipped_updates'],
            'recursive_generations': physics['final_report']['development']['recursive_generations'],
        },
        'real_signal_forecast': {
            'records': len(records), 'normalized_target_samples': count,
            'normalization': 'x / (1 + abs(x)); error measured before each POST update',
            'learned_mse': learned_mse, 'frozen_mse': frozen_mse,
            'relative_reduction_vs_frozen': 1 - learned_mse / frozen_mse,
            'simple_pre_only_baseline_mse': mse,
            'learned_to_last_sample_error_ratio': learned_mse / mse['last_sample'],
            'new_whole_organism_work': learned_work, 'frozen_new_whole_organism_work': frozen_work,
            'work_ratio': learned_work / frozen_work,
            'work_unit': 'native algorithm ledger, not CPU instructions or elapsed time',
            'advantage_over_simple_forecasting': False,
            'independent_samples_or_statistical_significance_claimed': False,
        },
        'external_simulator_control': {
            'environment': 'Gymnasium Pendulum-v1; 32x32 rendered camera and three public signals',
            'seeds': [physics['seed'] + i for i in range(len(physics['episodes']))],
            'learned_original_returns': [e['original_environment_return'] for e in physics['episodes']],
            'frozen_original_returns': [e['original_environment_return'] for e in physics_frozen['episodes']],
            'new_whole_organism_work': physics_work,
            'frozen_new_whole_organism_work': physics_frozen_work,
            'work_ratio': physics_work / physics_frozen_work,
            'freeze_ablation': 'all organism learning frozen, not an isolated ablation of the new model',
            'comparison_scope': 'post-hoc open engineering reference; two episodes; no equal-compute or generalization claim',
            'cross_modality_sensor_mse_comparable': False,
            'sensor_metric_note': 'frozen model does not register new camera coordinates; no comparison over different coordinate sets',
        },
        'replay': {'native_commits_matched': matched, 'full_states_byte_identical': True,
                   'new_environment_or_physical_experiments': 0},
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    result = summarize(args.evidence)
    Path(args.output).write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    print(json.dumps(result, indent=2))
