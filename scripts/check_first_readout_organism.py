"""Read and exactly restore published whole-organism states, without actions."""
import argparse,gzip,hashlib,json,subprocess,tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]


def check(binary,folder):
    folder=Path(folder);summaries=json.loads((folder/'summary.json').read_text())
    result={}
    for variant,frozen in [('organism_first_readout',False),('organism_no_new_learning',True)]:
        expected=summaries[variant]
        raw=gzip.decompress((folder/variant/'one-organism.json.gz').read_bytes())
        assert hashlib.sha256(raw).hexdigest()==expected['continuing_checkpoint_sha256']
        stored=json.loads(raw)['cognition']
        assert stored['freeze_controller']==frozen and not stored['archived']
        with tempfile.TemporaryDirectory(prefix='aeterna-first-readout-state-') as temp:
            path=Path(temp)/'organism.json';path.write_bytes(raw)
            current=json.loads(subprocess.check_output([str(binary),'--inspect',str(path)],text=True))
        assert current['read_only'] and current['external_actions_executed']==0
        assert current['organism']['controller_frozen']==frozen
        assert current['reward_memory']==expected['final_report']['reward_memory']
        assert current['development']==expected['final_report']['development']
        assert current['organism']['total_external_cost']==expected['final_report']['organism']['total_external_cost']
        rules=current['reward_memory']['acquired_rules']['first_readouts']
        assert rules and not rules['fixed_readout_control'] and rules['enabled_at']==11174
        requests=[{'op':'load','checkpoint':raw.decode()},{'op':'checkpoint'},{'op':'quit'}]
        replies=subprocess.check_output([str(binary),'--jsonl'],input=''.join(json.dumps(x)+'\n' for x in requests),text=True)
        replies=[json.loads(line) for line in replies.splitlines()]
        assert not any(r['type']=='error' for r in replies)
        assert next(r['checkpoint'] for r in replies if r['type']=='checkpoint').encode()==raw
        result[variant]={'status':'PASS','learning_enabled':not frozen,'one_learner':True,
            'checkpoint_sha256':hashlib.sha256(raw).hexdigest(),'factual_actions':current['reward_memory']['factual_events'],
            'completed_episodes':current['development']['episodes'],'raw_checkpoint_bytes':len(raw),
            'immutable_forecasts':rules['captured'],'external_actions_executed':0}
    assert summaries['organism_first_readout']['mnist_entry_checkpoint_sha256']==summaries['organism_no_new_learning']['mnist_entry_checkpoint_sha256']
    return result


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',required=True)
    p.add_argument('--results',default=str(ROOT/'docs/evidence/first-readout/study'))
    args=p.parse_args();print(json.dumps(check(Path(args.binary).resolve(),args.results),indent=2))
