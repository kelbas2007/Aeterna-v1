"""Independently replay a continuous-life development trace against its data.

Verifies real sensor values/rewards, actual MiniGrid transitions, offered motor
contracts, action budgets, unique commits, and the checkpoint resume boundary.
Does not treat integration success as evidence of behavioral superiority.
"""
import argparse,gzip,hashlib,importlib.metadata,json
from pathlib import Path
from external5_sensors import load_fashion,load_har,blank,acquire
from run_external_continual5 import motors,scene,verify_data_files

def audit(root,data):
    root=Path(root);provenance=json.loads((root/'provenance.json').read_text())
    if 'frozen_binary' in provenance:
        binary=root/provenance['frozen_binary']
        raw=binary.read_bytes() if binary.exists() else gzip.decompress(Path(str(binary)+'.gz').read_bytes())
        assert hashlib.sha256(raw).hexdigest()==provenance['binary_sha256']
    if 'dependencies' in provenance:
        assert {n:importlib.metadata.version(n) for n in provenance['dependencies']}==provenance['dependencies']
    assert verify_data_files(data)==provenance['data'],'corpus provenance mismatch'
    datasets={}
    for kind,loader in [('fashion',load_fashion),('har',load_har)]:
        start,end=provenance['training_ranges'][kind]
        datasets[kind]=loader(data,'train',start,end-start)
    import gymnasium as gym
    import minigrid  # noqa: F401 -- registration
    summaries=json.loads((root/'summary.json').read_text());proof={}
    for variant,summary in summaries.items():
        folder=root/variant;global_event=0;frames={};context=None;pending=None;env=None
        initializations=0;resumes=0;posts=0;actual_cost=0;namespaces=set()
        maps={p['result']['game']:p['result']['motor_map'] for p in summary['phases'] if 'game' in p['result']}
        try:
            for part,name in enumerate(('life-before-resume.jsonl.gz','life-after-resume.jsonl.gz')):
                frames={};local_event=0
                if part:
                    checkpoint=gzip.decompress((folder/'resume.json.gz').read_bytes())
                    assert hashlib.sha256(checkpoint).hexdigest()==summary['resume_checkpoint_sha256']
                    stored=json.loads(checkpoint)
                    assert stored['cognition']['current']['learner']['external_memory']['events']==global_event
                    assert stored['cognition']['total_external_cost']==actual_cost
                    resumes+=1
                with gzip.open(folder/name,'rt') as stream:
                    for line in stream:
                        row=json.loads(line);kind=row['kind']
                        if kind=='sensor_frame':
                            assert row['id'] not in frames
                            encoded=json.dumps(row['frame'],separators=(',',':'),allow_nan=False).encode()
                            assert hashlib.sha256(encoded).hexdigest()==row['id'],'frame content hash mismatch'
                            frames[row['id']]=row['frame']
                        elif kind=='initial':
                            initializations+=1;assert part==0 and row['variant']==variant
                        elif kind=='restart':
                            assert pending is None
                            metadata=row['metadata'];observed=frames[row['frame_ref']]
                            if metadata.get('game'):
                                if env is not None:env.close()
                                game=metadata['game'];env=gym.make(game,max_episode_steps=64)
                                obs,_=env.reset(seed=metadata['seed'])
                                context={'game':game,'namespace':observed['namespace'],'left':64,'frame':observed}
                                assert observed==scene(obs,context['namespace'])
                            else:
                                modality=metadata['phase'];index=metadata['index']
                                if not metadata.get('corrective_retry'):
                                    context={'modality':modality,'index':index,'frame':blank(modality),
                                        'measured':set(),'failed':set(),'left':11 if modality=='fashion' else 15}
                                assert context['modality']==modality and context['index']==index
                                assert observed==context['frame']
                        elif kind=='intent':
                            assert pending is None and context is not None
                            assert row['index']==local_event and row['remaining_actions']==context['left']
                            assert frames[row['pre_ref']]==context['frame']
                            action=row['action'];assert 0<=action<16
                            if 'game' in context:available=[a in maps[context['game']] for a in range(16)]
                            else:
                                query,answers=motors(context['modality']);available=[False]*16
                                for axis,a in enumerate(query):available[a]=axis not in context['measured'] and context['left']>1
                                for a in answers:available[a]=a not in context['failed'] and (context['modality']!='fashion' or bool(context['measured']))
                            assert row['offers']==[{'available':v,'max_cost':1} for v in available]
                            assert available[action]
                            pending={'action':action,'post':None}
                        elif kind=='actual_post':
                            assert pending is not None and pending['post'] is None
                            assert row['index']==local_event and row['action']==pending['action'] and row['cost']==1
                            action=row['action']
                            if 'game' in context:
                                obs,reward,_,_,_=env.step(maps[context['game']].index(action))
                                expected=scene(obs,context['namespace'])
                            else:
                                modality=context['modality'];index=context['index'];sample,target,_=datasets[modality]
                                query,answers=motors(modality)
                                if action in query:
                                    axis=query.index(action);context['measured'].add(axis)
                                    expected=acquire(context['frame'],axis,sample[index]);reward=0.0
                                else:
                                    expected=context['frame'];reward=float(answers.index(action)==int(target[index]))
                                    if not reward:context['failed'].add(action)
                            assert frames[row['frame_ref']]==expected,'invented or mismatched sensor POST'
                            assert abs(row['reward']-float(reward))<1e-12,'incorrect actual reward'
                            context['frame']=expected;context['left']-=1
                            namespaces.add((expected['kind'],expected['namespace']))
                            pending['post']=row
                        elif kind=='commit':
                            assert pending is not None and pending['post'] is not None
                            assert row['index']==local_event and row['step']['event']==global_event+1
                            assert row['step']['action']==pending['action']
                            actual_cost+=pending['post']['cost'];posts+=1;global_event+=1;local_event+=1;pending=None
                        else:raise AssertionError('unexpected trace event: '+kind)
                assert pending is None
            if 'continuing_checkpoint_sha256' in summary:
                continuing=gzip.decompress((folder/'one-organism.json.gz').read_bytes())
                assert hashlib.sha256(continuing).hexdigest()==summary['continuing_checkpoint_sha256']
                live=json.loads(continuing)
                assert not live['cognition']['freeze_controller'],'published organism cannot keep learning'
                assert live['cognition']['current']['learner']['external_memory']['events']==global_event
                assert live['cognition']['total_external_cost']==actual_cost
                checkpoint=gzip.decompress((folder/'frozen-organism.json.gz').read_bytes())
            else:checkpoint=gzip.decompress((folder/'one-organism.json.gz').read_bytes())
            assert hashlib.sha256(checkpoint).hexdigest()==summary['final_checkpoint_sha256']
            stored=json.loads(checkpoint);learner=stored['cognition']['current']['learner']
            assert learner['external_memory']['events']==global_event
            assert stored['cognition']['total_external_cost']==actual_cost
            assert summary['final_report']['organism']['total_external_cost']==actual_cost
            assert sum(p['result']['actual_actions'] for p in summary['phases'])==posts
            assert initializations==1 and resumes==1 and len(namespaces)==4
            proof[variant]={'status':'PASS','initializations':initializations,'exact_checkpoint_resumes':resumes,
                'factual_posts':posts,'actual_cost':actual_cost,'sensor_interfaces':len(namespaces),
                'corpus_values_and_rewards_replayed':True,'minigrid_transitions_replayed':True}
        finally:
            if env is not None:env.close()
    return proof

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--results',required=True);parser.add_argument('--data',required=True)
    parser.add_argument('--output',required=True);args=parser.parse_args()
    proof=audit(args.results,args.data)
    Path(args.output).write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps(proof,indent=2))
