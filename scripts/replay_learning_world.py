"""Reproduce a recorded shared learning life and compare every native commit.

No source environment is opened and no new physical experiment is performed.
"""
import argparse
import gzip
import hashlib
import json
import sqlite3
from pathlib import Path

from external5_transport import Native
from learning_world_stream import read_checkpoint


def replay(binary, source, output):
    source, output = Path(source), Path(output)
    output.mkdir(parents=True, exist_ok=False)
    initial = read_checkpoint(source / 'starting-organism.json.gz')
    expected = read_checkpoint(source / 'one-organism.json.gz')
    agent = Native(binary, output / 'replayed.jsonl.gz', 'organism', checkpoint=initial)
    database = sqlite3.connect(output / 'frames.sqlite')
    database.execute('create table frame (id text primary key, value text not null)')

    def frame(key):
        return json.loads(database.execute('select value from frame where id=?', (key,)).fetchone()[0])

    pending = None
    commits = 0
    try:
        with gzip.open(source / 'life.jsonl.gz', 'rt') as events:
            for line in events:
                event = json.loads(line)
                kind = event['kind']
                if kind == 'sensor_frame':
                    database.execute('insert or ignore into frame values (?,?)', (event['id'], json.dumps(event['frame'])))
                elif kind == 'runtime_command':
                    assert pending is None
                    response = agent.command(event['request'])
                    if 'expected' in event:
                        assert response == event['expected'], f'Runtime response mismatch after {commits} commits'
                elif kind == 'restart':
                    assert pending is None
                    agent.reset(frame(event['frame_ref']), **event['metadata'])
                elif kind == 'intent':
                    assert pending is None
                    agent.send({'op': 'external_step', 'offers': event['offers'], 'remaining_actions': event['remaining_actions']})
                    intention = agent.receive()
                    assert intention['type'] == 'action' and intention['action'] == event['action']
                    pending = 'awaiting_recorded_post'
                elif kind == 'actual_post':
                    assert pending == 'awaiting_recorded_post'
                    agent.send({'op': 'post', 'result': {'frame': frame(event['frame_ref']), 'cost': event['cost'], 'reward': event['reward']}})
                    pending = agent.receive()
                elif kind == 'commit':
                    assert isinstance(pending, dict) and pending['type'] == 'committed'
                    assert pending['step'] == event['step'], f'Commit mismatch at {commits}'
                    pending = None
                    commits += 1
        assert pending is None
        recomputed = agent.checkpoint()
        assert recomputed == expected, 'Full continuing learned state differs'
        result = {'status': 'PASS', 'native_commits_matched': commits,
                  'continuing_checkpoint_sha256': hashlib.sha256(recomputed.encode()).hexdigest(),
                  'full_state_byte_identical': True, 'new_physical_experiments': 0}
        (output / 'proof.json').write_text(json.dumps(result, indent=2) + '\n')
        return result
    finally:
        database.close()
        agent.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True)
    p.add_argument('--results', required=True)
    p.add_argument('--output', required=True)
    a = p.parse_args()
    print(json.dumps(replay(a.binary, a.results, a.output), indent=2))
