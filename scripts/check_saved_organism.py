"""Load the published real sensory organism through the canonical executable.

Runs on Linux/Windows CI without downloading datasets or executing an action.
"""
import argparse,gzip,hashlib,json,subprocess,tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]

def check(binary):
    folder=ROOT/'docs/evidence/organism-lifetime'
    summary=json.loads((folder/'summary.json').read_text())['organism']
    raw=gzip.decompress((folder/'organism/one-organism.json.gz').read_bytes())
    assert hashlib.sha256(raw).hexdigest()==summary['continuing_checkpoint_sha256']
    with tempfile.TemporaryDirectory(prefix='aeterna-real-organism-') as temp:
        state=Path(temp)/'organism.json';state.write_bytes(raw)
        report=json.loads(subprocess.check_output([str(Path(binary).resolve()),'--inspect',str(state)],text=True))
    assert report['type']=='checkpoint_inspection'
    assert report['read_only'] and report['external_actions_executed']==0
    organism=report['organism'];development=report['development']
    assert organism['total_external_cost']==2200 and not organism['controller_frozen']
    assert organism['native_meta_observations']>0
    assert development['episodes']==1050 and development['namespaces']==[4,9,82,86]
    assert development['recursive_attempts']==5 and development['evaluator_searches']==7
    assert report['recursive_search']['attempts']==5 and not report['recursive_search']['frozen']
    return {'status':'PASS','checkpoint_sha256':hashlib.sha256(raw).hexdigest(),
        'factual_action_cost':2200,'interfaces':4,'rsp2_attempts':5,'evaluator_searches':7,
        'learning_remains_enabled':True,'external_actions_executed':0}

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',required=True)
    args=parser.parse_args();print(json.dumps(check(args.binary),indent=2))
