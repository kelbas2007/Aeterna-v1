"""Continue one learning organism on recorded, actually observed transitions.

The historical action is the ONLY offered action. This ingests evidence and
does not claim to have autonomously controlled the recorded physical world.
For live control, use the same canonical external_step/post JSONL protocol
with all currently available actions instead.
"""
from __future__ import annotations

import argparse
import gzip
import hashlib
import itertools
import json
import math
import subprocess
from pathlib import Path

from external5_transport import Native


def compact(value):
    return json.dumps(value, separators=(',', ':'), allow_nan=False)


def read_checkpoint(path):
    data = Path(path).read_bytes()
    return (gzip.decompress(data) if str(path).endswith('.gz') else data).decode()


def save_checkpoint(path, checkpoint):
    Path(path).write_bytes(gzip.compress(checkpoint.encode(), mtime=0))


def source_hash(path):
    with Path(path).open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def starting_checkpoint(binary, frame, config, previous=None):
    if previous is None:
        commands = [{'op': 'organism_init', 'frame': frame, 'learning_world': config}]
    else:
        stored = json.loads(previous)
        if stored['cognition'].get('learning_world') is not None:
            return previous
        commands = [{'op': 'load', 'checkpoint': previous},
                    {'op': 'organism_enable_learning_world', 'config': config, 'frame': frame}]
    commands += [{'op': 'checkpoint'}, {'op': 'quit'}]
    result = subprocess.run([str(binary), '--jsonl'],
                            input=''.join(compact(c) + '\n' for c in commands),
                            text=True, capture_output=True, timeout=60)
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    errors = [r for r in replies if r.get('type') == 'error']
    if result.returncode or errors:
        raise RuntimeError(f'Native initialization rejected: {errors or result.stderr[-1000:]}')
    return next(r['checkpoint'] for r in replies if r['type'] == 'checkpoint')


def records(path, offset, limit):
    with Path(path).open(encoding='utf-8') as source:
        for index, line in itertools.islice(enumerate(source), offset, offset + limit):
            row = json.loads(line)
            if not isinstance(row, dict) or not {'pre', 'post', 'action', 'reward'} <= row.keys():
                raise ValueError(f'Record {index} needs pre, action, post and reward')
            if type(row['action']) is not int or row['action'] not in range(16):
                raise ValueError(f'Invalid recorded action at {index}')
            if type(row['reward']) not in (int, float) or not math.isfinite(row['reward']) or not -1 <= row['reward'] <= 1:
                raise ValueError(f'Invalid reward at {index}')
            if type(row.get('cost',1)) is not int or row.get('cost', 1) != 1:
                raise ValueError('This ingestion tool quotes one unit per historical transition; use JSONL offers for other costs')
            for flag in ['terminal', 'episode_start']:
                if flag in row and type(row[flag]) is not bool:
                    raise ValueError(f'{flag} must be boolean at {index}')
            yield index, row


def ingest(binary, source, output, config, offset=0, limit=256, checkpoint=None, freeze=False,
           temporal=None, forecast_log=False):
    if offset < 0 or limit <= 0:
        raise ValueError('offset must be nonnegative and limit positive')
    binary = Path(binary).resolve()
    digest = source_hash(source)
    rows = iter(records(source, offset, limit))
    first = next(rows, None)
    if first is None:
        raise ValueError('No recorded transitions in the requested range')
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    previous = read_checkpoint(checkpoint) if checkpoint else None
    initial = starting_checkpoint(binary, first[1]['pre'], config, previous)
    save_checkpoint(output / 'starting-organism.json.gz', initial)
    stored = json.loads(initial)
    raw_current = stored['cognition']['learning_world']['context'][-1]['frame']
    agent = Native(binary, output / 'life.jsonl.gz', 'organism', checkpoint=initial)
    agent.frame = raw_current
    count = 0
    try:
        attached_at_first_pre = False
        if temporal is not None and stored['cognition']['learning_world'].get('temporal') is None:
            # Attach at the actual observation of the new recorded interface.
            # The previous life's last action may have spent its full allowance.
            agent.reset(first[1]['pre'], recorded_source_index=first[0], before_temporal_attachment=True)
            attached_at_first_pre = True
            request = {'op': 'organism_enable_temporal_learning', 'config': temporal}
            response = agent.command(request)
            agent.trace.write({'kind': 'runtime_command', 'request': request, 'expected': response})
        if freeze:
            agent.trace.write({'kind': 'runtime_command', 'request': {'op': 'freeze'}})
            agent.freeze()
        initial_report = agent.command({'op': 'organism_report'})
        ended = False
        for index, row in itertools.chain([first], rows):
            if (ended or row.get('episode_start', False)) and not (index == first[0] and attached_at_first_pre):
                agent.reset(row['pre'], recorded_source_index=index)
            elif row['pre'] != agent.frame:
                raise ValueError(f'Record {index} breaks PRE/POST continuity; declare episode_start for a real boundary')
            agent.trace.write({'kind': 'recorded_source', 'index': index,
                               'source_metadata': row.get('provenance'),
                               'historical_action_forced': True})
            if forecast_log:
                request = {'op': 'world_prediction', 'action': row['action']}
                response = agent.command(request)
                agent.trace.write({'kind': 'runtime_command', 'request': request,
                                   'expected': response, 'recorded_source_index': index})
            available = [a == row['action'] for a in range(16)]
            agent.step(available, 1, lambda action: {
                'frame': row['post'], 'reward': float(row['reward']), 'cost': 1})
            count += 1
            ended = row.get('terminal', False)
            if ended:
                agent.trace.write({'kind': 'runtime_command', 'request': {'op': 'external_episode_end'}})
                agent.finish()
        continuing = agent.checkpoint()
        if source_hash(source)!=digest:
            raise RuntimeError('Source recording changed during ingestion')
        save_checkpoint(output / 'one-organism.json.gz', continuing)
        final_report = agent.command({'op': 'organism_report'})
        report = final_report['organism']['learning_world']
        initial_world = initial_report['organism']['learning_world']
        result = {
            'mode': 'recorded_evidence_ingestion_not_autonomous_physical_control',
            'source_sha256': digest, 'binary_sha256': source_hash(binary),
            'records_ingested': count, 'offset': offset, 'next_offset': offset + count,
            'source': str(Path(source).resolve()),
            'starting_checkpoint_sha256': hashlib.sha256(initial.encode()).hexdigest(),
            'continuing_checkpoint_sha256': hashlib.sha256(continuing.encode()).hexdigest(),
            'checkpoint_bytes': len(continuing.encode()), 'whole_runtime_instances_at_once': 1,
            'original_physical_experiments_executed': 0,
            'frozen': freeze,
            'temporal_enabled': initial_world.get('temporal') is not None,
            'forecast_log': forecast_log,
            'learned_state_unchanged': initial_world['learned_fingerprint'] == report['learned_fingerprint'],
            'new_factual_ingestions': report['factual_updates'] - initial_world['factual_updates'],
            'new_online_updates': report['online_updates'] - initial_world['online_updates'],
            'new_consolidation_updates': report['consolidation_updates'] - initial_world['consolidation_updates'],
            'initial_report': initial_report, 'final_report': final_report,
            'intelligence_qualification': False,
        }
        if result['new_factual_ingestions'] != count:
            raise AssertionError('Shared model did not receive exactly the committed evidence')
        if freeze and not result['learned_state_unchanged']:
            raise AssertionError('A frozen life changed learned state')
        (output / 'summary.json').write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
        return result
    finally:
        agent.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True)
    p.add_argument('--events', required=True)
    p.add_argument('--output', required=True)
    p.add_argument('--checkpoint')
    p.add_argument('--config', help='JSON object with learning resource settings')
    p.add_argument('--offset', type=int, default=0)
    p.add_argument('--limit', type=int, default=256)
    p.add_argument('--freeze', action='store_true')
    p.add_argument('--temporal', action='store_true', help='Enable causal temporal learning in the same runtime')
    p.add_argument('--temporal-config', help='JSON configuration of rule search and its producer role')
    p.add_argument('--forecast-log', action='store_true', help='Record the exact prediction before each actual POST')
    a = p.parse_args()
    config = json.loads(Path(a.config).read_text()) if a.config else {}
    temporal = json.loads(Path(a.temporal_config).read_text()) if a.temporal_config else ({} if a.temporal else None)
    result = ingest(a.binary, a.events, a.output, config, a.offset, a.limit, a.checkpoint, a.freeze, temporal, a.forecast_log)
    print(json.dumps({k: v for k, v in result.items() if k not in ['initial_report', 'final_report']}, indent=2))


if __name__ == '__main__':
    main()
