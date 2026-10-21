"""Development integration run: one checkpoint through sensors and navigation.

This never opens a held-out partition and is not a new blind qualification.
The environment adapter supplies actual frames/rewards, never candidate programs,
lessons for RSP2, success verdicts, policy IDs, or a motor action schedule.
"""
from __future__ import annotations
import argparse, gzip, hashlib, json, random, subprocess
from pathlib import Path
from external5_transport import Native
from external5_sensors import load_fashion, load_har, blank, acquire, metrics
from run_external_continual5 import motors, scene, verify_data_files

ROOT=Path(__file__).resolve().parents[1]
VARIANTS=('organism','organism_fixed','full')

def save(path,value):
    Path(path).write_text(json.dumps(value,indent=2,allow_nan=False)+'\n')

def digest(data):return hashlib.sha256(data).hexdigest()

def sensory_phase(agent,kind,data,training=True):
    x,y,_=data;queries,answers=motors(kind)
    lookup={a:i for i,a in enumerate(queries)};classes={a:i for i,a in enumerate(answers)}
    targets=[];first_predictions=[];solved=0;actions=0;measurements=0
    for index,(sample,target) in enumerate(zip(x,y)):
        frame=blank(kind);agent.reset(frame,phase=kind,index=index,development=True)
        measured=set();failed=set();first=None;success=False
        for left in range(11 if kind=='fashion' else 15,0,-1):
            available=[False]*16
            for a,axis in lookup.items():available[a]=axis not in measured and left>1
            for a in answers:available[a]=a not in failed and (kind!='fashion' or bool(measured))
            def execute(action):
                nonlocal frame,first,success,measurements
                if action in lookup:
                    axis=lookup[action];measured.add(axis);measurements+=1
                    frame=acquire(frame,axis,sample);reward=0.0
                else:
                    prediction=classes[action]
                    if first is None:first=prediction
                    reward=float(prediction==int(target));success=reward>0
                    if not success:failed.add(action)
                return {'frame':frame,'cost':1,'reward':reward}
            action,_=agent.step(available,left,execute);actions+=1
            if action in classes:
                agent.finish()
                if success:break
                if left>1:agent.reset(frame,phase=kind,index=index,corrective_retry=True,development=True)
        agent.finish()
        if first is None:raise AssertionError('no response before action budget expired')
        targets.append(int(target));first_predictions.append(first);solved+=success
    return {'first_answer_prequential':metrics(targets,first_predictions,len(answers)),
        'eventually_correct_with_feedback':solved,'actual_actions':actions,'measured_channels':measurements}

def navigation_phase(agent,game,namespace,seeds):
    import gymnasium as gym
    import minigrid  # noqa: F401 -- environment registration
    env=gym.make(game,max_episode_steps=64)
    mapping=random.Random(2026102100+namespace).sample(range(16),7)
    available=[a in mapping for a in range(16)];episodes=[]
    try:
        for seed in seeds:
            obs,_=env.reset(seed=seed);frame=scene(obs,namespace)
            agent.reset(frame,game=game,seed=seed,development=True)
            total=0.0;done=False;actions=0
            for left in range(64,0,-1):
                def execute(action):
                    nonlocal frame,total,done
                    obs,reward,terminated,truncated,_=env.step(mapping.index(action))
                    frame=scene(obs,namespace);total+=float(reward);done=bool(terminated or truncated)
                    return {'frame':frame,'cost':1,'reward':float(reward)}
                agent.step(available,left,execute);actions+=1
                if done:break
            agent.finish();episodes.append({'seed':seed,'actions':actions,'reward':total,'success':total>0})
    finally:env.close()
    return {'game':game,'motor_map':mapping,'episodes':episodes,
        'successes':sum(e['success'] for e in episodes),'actual_actions':sum(e['actions'] for e in episodes)}

def run(args):
    out=Path(args.output);out.mkdir(parents=True,exist_ok=False)
    # Authenticate the training data before decoding, including canonical HAR
    # membership and original Fashion checksums. No test partition is decoded.
    data_proof=verify_data_files(args.data)
    datasets={
        'fashion':load_fashion(args.data,'train',42000,args.samples),
        'har':load_har(args.data,'train',4000,args.samples),
    }
    binding={p.relative_to(ROOT).as_posix():digest(p.read_bytes())
        for folder,extension in (('src','*.rs'),('scripts','*.py'))
        for p in sorted((ROOT/folder).rglob(extension))}
    proof={'purpose':'development integration; no independent intelligence qualification',
        'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        'dirty':bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip()),
        'source_sha256':binding,'binary_sha256':digest(Path(args.binary).read_bytes()),
        'data':data_proof,'training_ranges':{'fashion':[42000,42000+args.samples], 'har':[4000,4000+args.samples]},
        'variants':args.variants,'max_actions_per_sensor':{'fashion':11,'har':15},'grid_action_budget':64}
    proof['declared_build_profile']=args.build_profile
    save(out/'provenance.json',proof)
    results={}
    for variant in args.variants:
        folder=out/variant;folder.mkdir()
        agent=Native(args.binary,folder/'life-before-resume.jsonl.gz',variant,blank('fashion'))
        phases=[];previous_cost=0;previous_events=0
        def record(name,result):
            nonlocal previous_cost,previous_events
            report=agent.command({'op':'organism_report'})
            current=report['organism'];memory=report['reward_memory']
            if current['total_external_cost']!=previous_cost+result['actual_actions']:
                raise AssertionError('shared lifetime lost or duplicated actual action cost')
            if memory['factual_events']!=previous_events+result['actual_actions']:
                raise AssertionError('shared sensory memory was replaced or invented a fact')
            previous_cost=current['total_external_cost'];previous_events=memory['factual_events']
            phases.append({'phase':name,'result':result,'report':report})
            save(folder/'phases.json',phases)
            print(json.dumps({'variant':variant,'phase':name,'actions':previous_cost,
                'development':report['development']},allow_nan=False),flush=True)
        try:
            record('fashion',sensory_phase(agent,'fashion',datasets['fashion']))
            saved=agent.checkpoint();resume_sha=digest(saved.encode())
            gzip.open(folder/'resume.json.gz','wt').write(saved)
            agent.close()
            agent=Native(args.binary,folder/'life-after-resume.jsonl.gz',variant,blank('fashion'),saved)
            if agent.checkpoint()!=saved:raise AssertionError('resume changed the cognitive checkpoint')
            record('har',sensory_phase(agent,'har',datasets['har']))
            for index,game in enumerate(('MiniGrid-Empty-5x5-v0','MiniGrid-DoorKey-5x5-v0')):
                record(game,navigation_phase(agent,game,20+index,range(850000,850000+args.grid_episodes)))
            # Acquired, previously experienced images. Report as a revisit with
            # continuing feedback, never as a new blind retention exam.
            revisit=(datasets['fashion'][0][:64],datasets['fashion'][1][:64],None)
            record('fashion_revisit_with_feedback',sensory_phase(agent,'fashion',revisit))
            continuing=agent.checkpoint()
            gzip.open(folder/'one-organism.json.gz','wt').write(continuing)
            agent.freeze();final=agent.checkpoint()
            gzip.open(folder/'frozen-organism.json.gz','wt').write(final)
            report=agent.command({'op':'organism_report'})
            if variant.startswith('organism'):
                dev=report['development']
                if len(dev['namespaces'])!=4:raise AssertionError('not all four interfaces share development')
                if report['reward_memory']['recursive_generations']!=0:raise AssertionError('parallel EXTERNAL5 mutator still active')
                if variant=='organism' and not dev['recursive_attempts']:raise AssertionError('RSP2 never received factual experience')
                if not dev['evaluator_searches']:raise AssertionError('shared RSP2 policy never drove evaluator search')
                if dev['abandoned_episodes']:raise AssertionError('completed adapter episodes were lost')
            results[variant]={'resume_checkpoint_sha256':resume_sha,'final_checkpoint_sha256':digest(final.encode()),
                'continuing_checkpoint_sha256':digest(continuing.encode()),
                'phases':phases,'final_report':report}
            save(folder/'summary.json',results[variant]);save(out/'summary.json',results)
        finally:agent.close()
    return results

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True);parser.add_argument('--data',required=True)
    parser.add_argument('--output',required=True);parser.add_argument('--samples',type=int,default=240)
    parser.add_argument('--grid-episodes',type=int,default=4)
    parser.add_argument('--variants',nargs='+',choices=VARIANTS,default=list(VARIANTS))
    parser.add_argument('--build-profile',default='unspecified; executable SHA256 recorded')
    args=parser.parse_args()
    if not 64<=args.samples<=500 or not 1<=args.grid_episodes<=16:parser.error('bounded development budget exceeded')
    run(args)
