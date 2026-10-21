"""Replay the whole native organism against recorded POSTs, without a world.

Every action and complete commit must match, then both the continuing and
frozen checkpoints must match byte for byte. This is software replay of an
already recorded life, never another experiment or new measurement.
"""
import argparse,gzip,hashlib,json,tempfile
from pathlib import Path
from external5_transport import Native


def replay(root,variant,output,binary_override=None):
    root=Path(root);output=Path(output);output.mkdir(parents=True,exist_ok=False)
    provenance=json.loads((root/'provenance.json').read_text())
    summary=json.loads((root/'summary.json').read_text())[variant]
    start=provenance.get('starting_checkpoints',{}).get(variant,{})
    path=root/start.get('file','starting-organism.json.gz')
    saved=gzip.decompress(path.read_bytes()).decode()
    assert hashlib.sha256(saved.encode()).hexdigest()==start.get('sha256',provenance['starting_checkpoint_sha256'])
    binary=Path(binary_override).resolve() if binary_override else root/provenance['frozen_binary']
    with tempfile.TemporaryDirectory(prefix='aeterna-factual-replay-') as temp:
        if not binary.exists():
            raw=gzip.decompress(Path(str(binary)+'.gz').read_bytes())
            binary=Path(temp)/'aeterna-v1';binary.write_bytes(raw);binary.chmod(0o700)
        binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest()
        if binary_override is None:assert binary_sha==provenance['binary_sha256']
        actions=0;commits=0;resumes=0
        agent=Native(binary,output/'before-resume.jsonl.gz',variant,checkpoint=saved)
        try:
            mode=provenance.get('readout_modes',{}).get(variant)
            if mode is not None:agent.command({'op':'organism_enable_first_readouts','fixed_readout_control':mode})
            report=agent.command({'op':'organism_report'})
            if provenance.get('contextual_u1_requested') and variant!='organism_previous' and report['organism'].get('contextual_namespaces') is None:
                agent.command({'op':'organism_enable_contextual_control'})
            if variant=='organism' and report['reward_memory'].get('acquired_rules') is None:
                agent.command({'op':'organism_enable_rules'})
            for part,name in enumerate(('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz')):
                if part:
                    recomputed=agent.checkpoint()
                    expected=gzip.decompress((root/variant/'resume.json.gz').read_bytes()).decode()
                    assert recomputed==expected,'intermediate whole-organism state differs'
                    agent.close()
                    agent=Native(binary,output/'after-resume.jsonl.gz',variant,checkpoint=recomputed)
                    resumes+=1
                with gzip.open(root/variant/name,'rt') as stream:rows=[json.loads(line) for line in stream]
                frames={r['id']:r['frame'] for r in rows if r['kind']=='sensor_frame'}
                posts={r['index']:r for r in rows if r['kind']=='actual_post'}
                pending=None
                for row in rows:
                    kind=row['kind']
                    if kind=='restart':
                        # The registered adapter finishes an episode before
                        # every reset. Empty finish is idempotent; reports and
                        # checkpoint exports between these operations are read-only.
                        agent.finish()
                        meta=row['metadata']
                        if (meta.get('phase')=='mnist' and meta.get('index')==0 and not meta.get('corrective_retry')
                            and variant!=provenance.get('frozen_before_mnist')):
                            assert hashlib.sha256(agent.checkpoint().encode()).hexdigest()==summary['mnist_entry_checkpoint_sha256']
                        agent.reset(frames[row['frame_ref']],**row['metadata'])
                    elif kind=='task_end':
                        agent.end_task()
                    elif kind=='learning_frozen':
                        assert variant==provenance.get('frozen_before_mnist') and row['before_phase']=='mnist'
                        # The adapter closes its last HAR reward episode before
                        # exporting the matched prefix and freezing learning.
                        agent.finish()
                        assert hashlib.sha256(agent.checkpoint().encode()).hexdigest()==summary['mnist_entry_checkpoint_sha256']
                        agent.freeze()
                    elif kind=='intent':
                        assert pending is None
                        agent.send({'op':'external_step','offers':row['offers'],'remaining_actions':row['remaining_actions']})
                        intention=agent.receive()
                        assert intention['type']=='action' and intention['action']==row['action'],f'native action differs at {actions}'
                        post=posts[row['index']]
                        agent.send({'op':'post','result':{'frame':frames[post['frame_ref']],'cost':post['cost'],'reward':post['reward']}})
                        pending=agent.receive();actions+=1
                        assert pending['type']=='committed'
                    elif kind=='commit':
                        assert pending and pending['step']==row['step'],f'whole native commit differs at {commits}'
                        pending=None;commits+=1
                assert pending is None
                agent.finish()
            continuing=agent.checkpoint()
            expected=gzip.decompress((root/variant/'one-organism.json.gz').read_bytes()).decode()
            assert continuing==expected,'final continuing whole-organism state differs'
            agent.freeze();frozen=agent.checkpoint()
            expected=gzip.decompress((root/variant/'frozen-organism.json.gz').read_bytes()).decode()
            assert frozen==expected,'final frozen whole-organism state differs'
            result={'status':'PASS','variant':variant,'recorded_actions_replayed':actions,'native_commits_matched':commits,
                'exact_checkpoint_resumes':resumes,'continuing_checkpoint_sha256':hashlib.sha256(continuing.encode()).hexdigest(),
                'frozen_checkpoint_sha256':hashlib.sha256(frozen.encode()).hexdigest(),
                'study_binary_sha256':provenance['binary_sha256'],'replay_binary_sha256':binary_sha,'all_native_choices_matched':True,
                'full_learned_state_byte_identical':True,'new_external_world_actions':0,
                'purpose':'deterministic software replay of the complete recorded organism; not a fresh performance experiment'}
            assert result['continuing_checkpoint_sha256']==summary['continuing_checkpoint_sha256']
            (output/'proof.json').write_text(json.dumps(result,indent=2)+'\n')
            return result
        finally:agent.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--results',required=True);parser.add_argument('--variant',default='organism')
    parser.add_argument('--binary',help='explicitly verify a current executable against the frozen complete life')
    parser.add_argument('--output',required=True);args=parser.parse_args()
    print(json.dumps(replay(args.results,args.variant,args.output,args.binary),indent=2))
