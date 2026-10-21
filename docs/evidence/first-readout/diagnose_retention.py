"""Retrospective memory diagnostics on already consumed Fashion rows.

No controller, learning, new world action, or fresh performance qualification.
Counts approximated failure penalties for inputs absent from retained seen keys.
"""
import argparse,gzip,hashlib,json,sys
from collections import Counter
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'scripts'))
from external5_sensors import features,load_fashion
from run_external_continual5 import motors

MASK=(1<<64)-1


def fnv(data,seed=0xcbf29ce484222325):
    for value in data:seed=((seed^value)*0x100000001b3)&MASK
    return seed


def estimate(memory,key):
    table=memory.get('retired_failures')
    if not table:return 0.0
    return min(table['buckets'][row*2048+fnv(key,0xcbf29ce484222325^((row*0x9e3779b97f4a7c15)&MASK))%2048] for row in range(4))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--before',required=True);p.add_argument('--after',required=True)
    p.add_argument('--data',required=True);p.add_argument('--output',required=True)
    args=p.parse_args();raws=[gzip.decompress(Path(path).read_bytes()) for path in (args.before,args.after)]
    models=[json.loads(raw)['cognition']['current']['learner']['external_memory'] for raw in raws]
    samples,_,_=load_fashion(args.data,'train',57000,192);answers=motors('fashion')[1]
    seen=[set(m['acquired_rules']['domains']['4']['seen']) for m in models]
    stats=[]
    for memory,known in zip(models,seen,strict=True):
        penalties=[];novel_inputs=0;exact_rates=0;changed_by_motor=0
        for image in samples:
            namespace,mask,x=features({'kind':'image','namespace':1,'pixels':image.tolist()})
            state=[namespace,mask]+[int(v) for v in x]
            key_hash=fnv(json.dumps(state,separators=(',',':')).encode())
            if key_hash in known:continue
            novel_inputs+=1;current=[]
            for motor in answers:
                key=json.dumps([state,motor],separators=(',',':'))
                exact_rates+=int(key in memory['reward_rates'])
                value=estimate(memory,key.encode());penalties.append(value);current.append(value)
            changed_by_motor+=int(max(current)>min(current))
        buckets=memory['retired_failures']['buckets']
        positive=[r for r in memory['rows'] if r['pre'][0]==4 and r['reward']>0]
        stats.append({'retained_successful_fashion_cases':len(positive),
            'positive_motor_counts':dict(Counter(str(r['action']) for r in positive)),
            'retained_known_fashion_input_hashes':len(known),'new_inputs_not_in_known_hashes':novel_inputs,
            'answer_keys_with_exact_reward_rate':exact_rates,'new_answer_keys_examined':len(penalties),
            'new_answer_keys_with_nonzero_approximate_failure_penalty':sum(v>0 for v in penalties),
            'inputs_with_different_penalties_across_answer_motors':changed_by_motor,
            'approximate_failure_mean':sum(penalties)/max(len(penalties),1),
            'nonzero_summary_buckets':sum(v>0 for v in buckets),'summary_bucket_count':len(buckets)})
    before,after=models
    result={'purpose':'retrospective diagnosis on consumed data, not fresh behavioral evidence',
        'checkpoint_sha256':{'before':hashlib.sha256(raws[0]).hexdigest(),'after':hashlib.sha256(raws[1]).hexdigest()},
        'fashion_normalization_unchanged':before['metrics']['4']==after['metrics']['4'],
        'conditional_program_unchanged':before['acquired_rules']['domains']['4']['active']==after['acquired_rules']['domains']['4']['active'],
        'retained_fashion_after_is_subset_of_before':{r['id'] for r in after['rows'] if r['pre'][0]==4 and r['reward']>0}.issubset({r['id'] for r in before['rows'] if r['pre'][0]==4 and r['reward']>0}),
        'before_new_worlds':stats[0],'after_new_worlds_before_fashion_return':stats[1],
        'interpretation':'FIFO within namespace/reward strata removes old successful examples. Shared count-min failure estimates also penalize keys without exact rates and absent from retained known inputs; collisions may change relative action values. This does not isolate how many of the 21 lost actual first answers come from each mechanism.'}
    Path(args.output).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))


if __name__=='__main__':main()
