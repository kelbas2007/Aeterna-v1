"""Inspect and round-trip the published continuing organism; execute no action."""
import argparse,gzip,hashlib,json,subprocess,tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]


def check(binary,folder):
    folder=Path(folder);summary=json.loads((folder/'summary.json').read_text())['organism']
    raw=gzip.decompress((folder/'organism/one-organism.json.gz').read_bytes())
    assert hashlib.sha256(raw).hexdigest()==summary['continuing_checkpoint_sha256']
    binary=str(Path(binary).resolve())
    with tempfile.TemporaryDirectory(prefix='aeterna-acquired-organism-') as temp:
        path=Path(temp)/'organism.json';path.write_bytes(raw)
        inspection=json.loads(subprocess.check_output([binary,'--inspect',str(path)],text=True))
    assert inspection['read_only'] and inspection['external_actions_executed']==0
    current=inspection['organism'];memory=inspection['reward_memory'];expected=summary['final_report']
    assert not current['controller_frozen']
    assert current['contextual_namespaces']==[4,9,82,86]
    assert current['total_external_cost']==expected['organism']['total_external_cost']
    assert inspection['development']['episodes']==expected['development']['episodes']
    assert memory==expected['reward_memory']
    requests=[{'op':'load','checkpoint':raw.decode()},{'op':'checkpoint'},{'op':'quit'}]
    replies=subprocess.check_output([binary,'--jsonl'],input=''.join(json.dumps(c)+'\n' for c in requests),text=True)
    replies=[json.loads(line) for line in replies.splitlines()]
    assert not any(r['type']=='error' for r in replies)
    saved=next(r['checkpoint'] for r in replies if r['type']=='checkpoint')
    assert saved.encode()==raw,'restoration changed acquired knowledge or pending evidence'
    return {'status':'PASS','checkpoint_sha256':hashlib.sha256(raw).hexdigest(),
        'factual_actions':memory['factual_events'],'completed_episodes':inspection['development']['episodes'],
        'acquired_rules':memory['acquired_rules'],'joint_checkpoint_bytes':len(raw),
        'reward_model_bytes':memory['model_bytes'],'learning_enabled':True,'external_actions_executed':0}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True)
    parser.add_argument('--results',default=str(ROOT/'docs/evidence/acquired-rules/confirmation'))
    args=parser.parse_args();print(json.dumps(check(args.binary,args.results),indent=2))
