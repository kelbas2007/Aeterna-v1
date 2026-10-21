"""Development integration run: one checkpoint through sensors and navigation.

This never opens a held-out partition and is not a new blind qualification.
The environment adapter supplies actual frames/rewards, never candidate programs,
lessons for RSP2, success verdicts, policy IDs, or a motor action schedule.
"""
from __future__ import annotations
import argparse, gzip, hashlib, importlib.metadata, json, random, shutil, subprocess
from pathlib import Path
from external5_transport import Native
from external5_sensors import load_fashion, load_har, blank, acquire, metrics
from run_external_continual5 import motors, scene, verify_data_files

ROOT=Path(__file__).resolve().parents[1]
VARIANTS=('organism','organism_fixed','full','organism_rules_fixed','organism_previous')

def save(path,value):
    Path(path).write_text(json.dumps(value,indent=2,allow_nan=False)+'\n')

def digest(data):return hashlib.sha256(data).hexdigest()

def sensory_phase(agent,kind,data,training=True,source_offset=None):
    x,y,_=data;queries,answers=motors(kind)
    lookup={a:i for i,a in enumerate(queries)};classes={a:i for i,a in enumerate(answers)}
    targets=[];first_predictions=[];solved=0;actions=0;measurements=0
    for index,(sample,target) in enumerate(zip(x,y)):
        metadata={} if source_offset is None else {'source_offset':source_offset}
        frame=blank(kind);agent.reset(frame,phase=kind,index=index,development=True,**metadata)
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
                if left>1:agent.reset(frame,phase=kind,index=index,corrective_retry=True,development=True,**metadata)
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
    # Cargo may rebuild its normal output while regression tests run. Every
    # lifetime and control must execute the same immutable copied binary.
    original=Path(args.binary).resolve()
    frozen=out/('frozen-aeterna-v1'+('.exe' if original.suffix=='.exe' else ''))
    shutil.copy2(original,frozen);args.binary=str(frozen.resolve())
    # Authenticate the training data before decoding, including canonical HAR
    # membership and original Fashion checksums. No test partition is decoded.
    data_proof=verify_data_files(args.data)
    datasets={
        'fashion':load_fashion(args.data,'train',args.fashion_offset,args.samples),
        'har':load_har(args.data,'train',args.har_offset,args.samples),
    }
    starting=None;starting_cost=0;starting_events=0
    if args.checkpoint:
        source=Path(args.checkpoint)
        starting=(gzip.decompress(source.read_bytes()).decode() if source.suffix=='.gz' else source.read_text())
        stored=json.loads(starting);cognition=stored['cognition']
        if cognition['freeze_controller'] or cognition.get('development') is None:raise ValueError('need a living common organism')
        memory=cognition['current']['learner']['external_memory']
        if memory.get('acquired_rules') is not None:raise ValueError('starting controls must share the same state before rule acquisition')
        if any(v not in ('organism','organism_rules_fixed','organism_previous') for v in args.variants):raise ValueError('continuation requires matched common-organism variants')
        starting_cost=cognition['total_external_cost'];starting_events=memory['events']
        gzip.open(out/'starting-organism.json.gz','wt').write(starting)
    starts={};start_proofs={}
    for variant in args.variants:
        raw=starting
        if args.resume_lives:
            if variant not in ('organism','organism_rules_fixed','organism_previous'):raise ValueError('resume complete common-organism lives only')
            path=Path(args.previous_life) if variant=='organism_previous' and args.previous_life else Path(args.resume_lives)/variant/'one-organism.json.gz'
            raw=gzip.decompress(path.read_bytes()).decode()
        if raw is not None:
            state=json.loads(raw)['cognition'];memory=state['current']['learner']['external_memory']
            if state['freeze_controller'] or state.get('development') is None:raise ValueError('need a living common organism')
            if variant in ('organism_rules_fixed','organism_previous') and memory.get('acquired_rules') is not None:raise ValueError('control acquired rule programs')
            starts[variant]=raw
            start_proofs[variant]={'sha256':digest(raw.encode()),'events':memory['events'],'cost':state['total_external_cost'],
                'file':variant+'/starting-organism.json.gz'}
    binding={p.relative_to(ROOT).as_posix():digest(p.read_bytes())
        for folder,extension in (('src','*.rs'),('scripts','*.py'))
        for p in sorted((ROOT/folder).rglob(extension))}
    proof={'purpose':'development integration; no independent intelligence qualification',
        'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        'dirty':bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip()),
        'source_sha256':binding,'binary_sha256':digest(Path(args.binary).read_bytes()),
        'frozen_binary':frozen.name,
        'dependencies':{name:importlib.metadata.version(name) for name in ('numpy','gymnasium','minigrid')},
        'data':data_proof,'training_ranges':{'fashion':[args.fashion_offset,args.fashion_offset+args.samples], 'har':[args.har_offset,args.har_offset+args.samples]},
        'variants':args.variants,'max_actions_per_sensor':{'fashion':11,'har':15},'grid_action_budget':64}
    proof['declared_build_profile']=args.build_profile
    proof['contextual_u1_requested']=args.contextual_u1
    proof['starting_checkpoint_sha256']=None if starting is None else digest(starting.encode())
    proof['starting_cost']=starting_cost;proof['starting_events']=starting_events
    proof['starting_checkpoints']=start_proofs
    proof['sensory_sources']={'fashion':{str(args.fashion_offset):args.samples},'har':{str(args.har_offset):args.samples}}
    if args.retention_offset is not None:
        proof['sensory_sources']['fashion'][str(args.retention_offset)]=args.retention_samples
    proof['purpose']=args.purpose
    save(out/'provenance.json',proof)
    results={}
    for variant in args.variants:
        folder=out/variant;folder.mkdir()
        raw=starts.get(variant)
        if raw is not None:gzip.open(folder/'starting-organism.json.gz','wt').write(raw)
        agent=Native(args.binary,folder/'life-before-resume.jsonl.gz',variant,blank('fashion'),raw)
        if args.contextual_u1 and variant!='organism_previous' and agent.command({'op':'organism_report'})['organism'].get('contextual_namespaces') is None:
            agent.command({'op':'organism_enable_contextual_control'})
        if raw is not None and variant=='organism' and json.loads(raw)['cognition']['current']['learner']['external_memory'].get('acquired_rules') is None:
            agent.command({'op':'organism_enable_rules'})
        phases=[];previous_cost=start_proofs.get(variant,{}).get('cost',0);previous_events=start_proofs.get(variant,{}).get('events',0)
        def record(name,result):
            nonlocal previous_cost,previous_events
            if digest(Path(args.binary).read_bytes())!=proof['binary_sha256']:
                raise AssertionError('frozen executable changed during the lifetime')
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
            record('fashion',sensory_phase(agent,'fashion',datasets['fashion'],source_offset=args.fashion_offset))
            saved=agent.checkpoint();resume_sha=digest(saved.encode())
            gzip.open(folder/'resume.json.gz','wt').write(saved)
            agent.close()
            agent=Native(args.binary,folder/'life-after-resume.jsonl.gz',variant,blank('fashion'),saved)
            if agent.checkpoint()!=saved:raise AssertionError('resume changed the cognitive checkpoint')
            record('har',sensory_phase(agent,'har',datasets['har'],source_offset=args.har_offset))
            for index,game in enumerate(('MiniGrid-Empty-5x5-v0','MiniGrid-DoorKey-5x5-v0')):
                record(game,navigation_phase(agent,game,20+index,range(850000,850000+args.grid_episodes)))
            # Acquired, previously experienced images. Report as a revisit with
            # continuing feedback, never as a new blind retention exam.
            if args.retention_offset is None:
                revisit=(datasets['fashion'][0][:64],datasets['fashion'][1][:64],None)
                record('fashion_revisit_with_feedback',sensory_phase(agent,'fashion',revisit,source_offset=args.fashion_offset))
            else:
                retention=load_fashion(args.data,'train',args.retention_offset,args.retention_samples)
                record('fashion_new_after_other_worlds',sensory_phase(agent,'fashion',retention,source_offset=args.retention_offset))
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
    parser.add_argument('--variants',nargs='+',choices=VARIANTS,default=['organism','organism_fixed','full'])
    parser.add_argument('--build-profile',default='unspecified; executable SHA256 recorded')
    resume=parser.add_mutually_exclusive_group()
    resume.add_argument('--checkpoint');resume.add_argument('--resume-lives')
    parser.add_argument('--previous-life',help='matched previous-U1 control export with the same completed development schedule')
    parser.add_argument('--fashion-offset',type=int,default=42000)
    parser.add_argument('--har-offset',type=int,default=4000)
    parser.add_argument('--retention-offset',type=int);parser.add_argument('--retention-samples',type=int,default=160)
    parser.add_argument('--purpose',default='development integration; no independent intelligence qualification')
    parser.add_argument('--contextual-u1',action='store_true',help='enable context-conditioned common U1 in loaded historical states, for both arms')
    args=parser.parse_args()
    if not 64<=args.samples<=1000 or not 1<=args.grid_episodes<=16 or not 1<=args.retention_samples<=500:parser.error('bounded development budget exceeded')
    run(args)
