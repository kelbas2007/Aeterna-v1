import argparse,gzip,json
from pathlib import Path
parser=argparse.ArgumentParser(description='Read-only check of actual choices against the common agency ledger.')
parser.add_argument('--results',required=True);parser.add_argument('--output',required=True)
args=parser.parse_args();root=Path(args.results)/'lives'
proof={}
for variant in ('organism_consolidated','organism_fifo'):
    path=root/variant
    start=json.loads(gzip.decompress((path/'starting-organism.json.gz').read_bytes()))
    end=json.loads(gzip.decompress((path/'one-organism.json.gz').read_bytes()))
    counts={'self_selected_actions':0,'externally_forced_actions':0}
    for name in ('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz'):
        for line in gzip.open(path/name,'rt'):
            row=json.loads(line)
            if row['kind']=='intent':
                choices=sum(o['available'] for o in row['offers'])
                assert choices>0
                key='self_selected_actions' if choices>=2 else 'externally_forced_actions'
                counts[key]+=1
    a=start['cognition']['agency_credit'];b=end['cognition']['agency_credit']
    assert counts=={k:b[k]-a[k] for k in counts}
    proof[variant]={'status':'PASS','actual_actions':sum(counts.values()),**counts,
        'counted_from_real_motor_offers':True,'common_checkpoint_credit_ledger_matches':True,
        'one_available_motor_is_not_an_autonomous_choice':True}
out=Path(args.output)
out.write_text(json.dumps(proof,indent=2)+'\n')
print(out.read_text())
