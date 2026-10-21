"""Load real published memory-study states and restore exact bytes; no actions."""
import argparse,gzip,hashlib,json,subprocess,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def check(binary,folder):
    root=Path(folder);summaries=json.loads((root/'lives/summary.json').read_text());result={}
    for variant in ('organism_causal','organism_consolidated','organism_fifo'):
        raw=gzip.decompress((root/'lives'/variant/'one-organism.json.gz').read_bytes())
        assert hashlib.sha256(raw).hexdigest()==summaries[variant]['continuing_checkpoint_sha256']
        state=json.loads(raw)['cognition'];assert not state['freeze_controller'] and not state['archived']
        memory=state['current']['learner']['external_memory'];assert len(memory['rows'])<=2048
        assert ('consolidated_memory' in memory)==(variant!='organism_fifo')
        with tempfile.TemporaryDirectory(prefix='aeterna-coverage-state-') as temp:
            path=Path(temp)/'state.json';path.write_bytes(raw)
            current=json.loads(subprocess.check_output([str(binary),'--inspect',str(path)],text=True))
        assert current['read_only'] and current['external_actions_executed']==0
        assert current['reward_memory']==summaries[variant]['continuing_report']['reward_memory']
        requests=[{'op':'load','checkpoint':raw.decode()},{'op':'checkpoint'},{'op':'quit'}]
        replies=subprocess.check_output([str(binary),'--jsonl'],input=''.join(json.dumps(r)+'\n' for r in requests),text=True)
        replies=[json.loads(line) for line in replies.splitlines()]
        assert not any(r['type']=='error' for r in replies)
        assert next(r['checkpoint'] for r in replies if r['type']=='checkpoint').encode()==raw
        result[variant]={'status':'PASS','factual_actions':memory['events'],'retained_cases':len(memory['rows']),
            'one_live_learner':True,'checkpoint_sha256':hashlib.sha256(raw).hexdigest(),'external_actions_executed':0}
    return result

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',required=True)
    p.add_argument('--results',default=str(ROOT/'docs/evidence/causal-readout'))
    args=p.parse_args();print(json.dumps(check(Path(args.binary).resolve(),args.results),indent=2))
