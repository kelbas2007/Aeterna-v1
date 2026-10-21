"""Preregistered external/continual study. Development NEVER opens test files."""
from __future__ import annotations
import argparse, gzip, hashlib, importlib.metadata, json, os, random, subprocess, sys
from pathlib import Path
import numpy as np
from external5_transport import Native,RandomReference,new_agent
from external5_sensors import load_fashion,load_har,blank,run_windows,MatchedKNN
from prepare_external5_data import FASHION,verify_har_archive

VARIANTS=['legacy','random','one_step','fixed','full']
GAMES=['MiniGrid-DoorKey-8x8-v0','MiniGrid-MultiRoom-N2-S4-v0','MiniGrid-Unlock-v0']
ROOT=Path(__file__).resolve().parents[1]
PINNED_VERSIONS={'minigrid':'3.1.0','gymnasium':'1.2.0','arc-agi':'0.9.9','arcengine':'0.9.3','numpy':'2.3.5'}
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def save(path,value):Path(path).write_text(json.dumps(value,indent=2,allow_nan=False)+'\n')
def variants(args):return VARIANTS if args.score else args.dev_variants

def verify_data_files(root,expected_names=None):
    root=Path(root);manifest=json.loads((root/'manifest.json').read_text())
    expected=set(FASHION)|{'uci-har.zip'} if expected_names is None else set(expected_names)
    records=manifest['files'];names=[r['name'] for r in records]
    if len(names)!=len(set(names)) or set(names)!=expected:raise ValueError('dataset manifest membership mismatch')
    verified={}
    for row in records:
        name=row['name']
        if Path(name).name!=name:raise ValueError('dataset name must be a local basename')
        path=root/name
        if not path.is_file() or path.stat().st_size!=row['bytes']:raise ValueError('dataset size mismatch: '+name)
        digest=hashlib.sha256();md5=hashlib.md5()
        with path.open('rb') as f:
            while block:=f.read(1024*1024):digest.update(block);md5.update(block)
        if digest.hexdigest()!=row['sha256']:raise ValueError('dataset SHA256 mismatch: '+name)
        if name in FASHION and md5.hexdigest()!=FASHION[name]:raise ValueError('official dataset checksum mismatch: '+name)
        if name=='uci-har.zip':verify_har_archive(path)
        verified[name]={'bytes':path.stat().st_size,'sha256':digest.hexdigest()}
    return verified

def dependency_versions(scoring):
    actual={name:importlib.metadata.version(name) for name in PINNED_VERSIONS}
    if scoring and actual!=PINNED_VERSIONS:
        raise RuntimeError('scoring dependency versions differ from pinned study environment: '+json.dumps(actual,sort_keys=True))
    return actual

def motors(kind):
    rng=random.Random({'fashion':202610201,'har':202610202,'mnist':202610213}[kind]);mapping=rng.sample(range(16),16)
    n=9 if kind=='har' else 1;classes=6 if kind=='har' else 10
    return mapping[:n],mapping[n:n+classes]

def commitment(args,out):
    dirty=subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True)
    if args.score and dirty:raise RuntimeError('scoring requires a clean, committed source tree')
    dependencies=dependency_versions(args.score)
    verified=verify_data_files(args.data)
    state={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        'development':not args.score,'working_tree_dirty':bool(dirty),'binary_sha256':sha(args.binary),
        'protocol_sha256':sha(ROOT/'docs/EXTERNAL_CONTINUAL5_PROTOCOL.md'),
        'data_manifest_sha256':sha(Path(args.data)/'manifest.json'),
        'verified_data_files':verified,
        'dependencies':dependencies,
        'variants':variants(args),'suites':args.suites,'motor_maps':{kind:motors(kind) for kind in ['fashion','har']}}
    save(out/'commitment.json',state);return state

def evaluate_sensor(parent,args,out,name,kind,data,paired):
    checkpoint=parent.checkpoint() if isinstance(parent,Native) else None
    child=new_agent(args.binary,out/(name+'.jsonl.gz'),parent.variant,blank(kind),checkpoint)
    try:
        if isinstance(child,Native):child.freeze()
        before=child.report()
        if isinstance(child,Native):
            saved=child.checkpoint();gzip.open(out/(name+'.checkpoint.json.gz'),'wt').write(saved)
        result=run_windows(child,kind,*data,motors(kind),False,paired)
        after=child.report()
        if isinstance(child,Native) and before['learned_fingerprint']!=after['learned_fingerprint']:
            raise AssertionError('frozen sensory evaluation changed learned knowledge')
        result['memory_before']=before;result['memory_after']=after
        return result
    finally:child.close()

def sensors(args,out):
    train_count=3000 if args.score else args.dev_train
    count=1000 if args.score else args.dev_validation
    part='test' if args.score else 'train'
    offset_fashion=0 if args.score else 50000
    offset_har=0 if args.score else 6000
    # These are the only held-partition decoding calls in the sensor study.
    datasets={
        'fashion_train':load_fashion(args.data,'train',0,train_count),
        'har_train':load_har(args.data,'train',0,train_count),
        'fashion_eval':load_fashion(args.data,part,offset_fashion,count),
        'har_eval':load_har(args.data,part,offset_har,count),
    }
    if args.score and set(datasets['har_train'][2])&set(datasets['har_eval'][2]):
        raise AssertionError('HAR subject leakage')
    results={}
    for variant in variants(args):
        folder=out/variant;folder.mkdir()
        agent=new_agent(args.binary,folder/'continuous-training.jsonl.gz',variant,blank('fashion'))
        paired=MatchedKNN() if variant=='full' else None
        result={}
        try:
            result['fashion_training']=run_windows(agent,'fashion',*datasets['fashion_train'],motors('fashion'),True,paired)
            result['fashion_before_har']=evaluate_sensor(agent,args,folder,'fashion-before-har','fashion',datasets['fashion_eval'],paired)
            result['har_training']=run_windows(agent,'har',*datasets['har_train'],motors('har'),True,paired)
            result['fashion_after_har']=evaluate_sensor(agent,args,folder,'fashion-after-har','fashion',datasets['fashion_eval'],paired)
            result['har_after_har']=evaluate_sensor(agent,args,folder,'har-after-har','har',datasets['har_eval'],paired)
            result['training_memory']=agent.report()
            if isinstance(agent,Native):
                checkpoint=agent.checkpoint();gzip.open(folder/'one-continuing-learner.json.gz','wt').write(checkpoint)
                result['checkpoint_bytes']=len(checkpoint.encode());result['checkpoint_sha256']=hashlib.sha256(checkpoint.encode()).hexdigest()
                result['unified_organism']=agent.command({'op':'report'})['report']
            if paired:result['matched_3nn_training']=paired.report()
        finally:
            agent.close();save(folder/'summary.json',result)
        results[variant]=result;save(out/'summary.json',results)
    full=results.get('full')
    gates={'fashion_accuracy':None,'har_balanced_accuracy':None,'fashion_retention':None}
    if full is not None:
        before=full['fashion_before_har']['accuracy'];after=full['fashion_after_har']['accuracy']
        gates={'fashion_accuracy':before>=0.65,'har_balanced_accuracy':full['har_after_har']['balanced_accuracy']>=0.65,
               'fashion_retention':after>=before*0.9}
    return {'results':results,'gates':gates,'all_targets_met':None if full is None else all(gates.values()),'evaluation_partition':part,
            'note':'Matched 3NN receives the same actually acquired measurements and successful training-answer facts.'}

def scene(observation,namespace):
    # Only the public image tuple; never mission, hidden map or absolute pose.
    image=np.asarray(observation['image'],dtype=np.int64)
    pixels=image[:,:,0]*100+image[:,:,1]*10+image[:,:,2]
    return {'kind':'scene','namespace':namespace,'pixels':pixels.tolist()}

def run_grid_episodes(agent,game,seeds,mapping,namespace,training):
    import gymnasium as gym
    import minigrid  # registers official environments
    env=gym.make(game,max_episode_steps=256);results=[]
    try:
        for seed in seeds:
            observation,_=env.reset(seed=int(seed));frame=scene(observation,namespace)
            agent.reset(frame,game=game,seed=int(seed),training=training)
            total=0.0;done=False;steps=0;seen={json.dumps(frame,separators=(',',':'))}
            available=[a in mapping for a in range(16)]
            for left in range(256,0,-1):
                def execute(action):
                    nonlocal frame,total,done
                    observation,reward,terminated,truncated,_=env.step(mapping.index(action))
                    frame=scene(observation,namespace);done=bool(terminated or truncated);total+=float(reward)
                    return {'frame':frame,'cost':1,'reward':float(reward)}
                agent.step(available,left,execute);steps+=1;seen.add(json.dumps(frame,separators=(',',':')))
                if done:break
            agent.finish();row={'seed':int(seed),'actions':steps,'reward':total,'success':total>0,'distinct_public_views':len(seen)}
            results.append(row);agent.trace.write({'kind':'episode_result','training':training,**row})
            print(json.dumps({'phase':'minigrid','variant':agent.variant,'game':game,'training':training,**row}),flush=True)
    finally:env.close()
    return {'episodes':len(results),'successes':sum(r['success'] for r in results),
            'actions':sum(r['actions'] for r in results),'lives':results}

def grids(args,out):
    import gymnasium as gym
    import minigrid
    games=GAMES if args.score else ['MiniGrid-Empty-5x5-v0','MiniGrid-DoorKey-5x5-v0']
    result={}
    for family,game in enumerate(games):
        folder=out/game;folder.mkdir();result[game]={}
        mapping=random.Random(20261020+family).sample(range(16),7);namespace=20+family
        env=gym.make(game);obs,_=env.reset(seed=830000 if args.score else 810000);initial=scene(obs,namespace);env.close()
        train_seeds=range(830000,830032) if args.score else range(810000,810004)
        eval_seeds=range(820000,820016) if args.score else range(810004,810008)
        for variant in variants(args):
            current=folder/variant;current.mkdir();agent=new_agent(args.binary,current/'training.jsonl.gz',variant,initial)
            entry={'motor_map':mapping}
            try:
                entry['training']=run_grid_episodes(agent,game,train_seeds,mapping,namespace,True)
                if isinstance(agent,Native):
                    agent.freeze();checkpoint=agent.checkpoint();gzip.open(current/'trained.json.gz','wt').write(checkpoint)
                    agent.close();agent=new_agent(args.binary,current/'held.jsonl.gz',variant,checkpoint=checkpoint,frame=initial)
                before=agent.report();entry['evaluation']=run_grid_episodes(agent,game,eval_seeds,mapping,namespace,False)
                after=agent.report()
                if isinstance(agent,Native) and before['learned_fingerprint']!=after['learned_fingerprint']:
                    raise AssertionError('frozen MiniGrid evaluation changed knowledge')
                entry['memory']=after
            finally:agent.close();save(current/'summary.json',entry)
            result[game][variant]=entry;save(out/'summary.json',result)
    scores={variant:sum(result[g][variant]['evaluation']['successes'] for g in games) for variant in variants(args)}
    total=len(eval_seeds)*len(games)
    return {'results':result,'scores':scores,'episodes_per_variant':total,
            'target_met':scores['full']/total>=0.25 and scores['full']>max(scores['legacy'],scores['random']) if {'full','legacy','random'}<=scores.keys() else None}

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--data',required=True)
    ap.add_argument('--output',required=True);ap.add_argument('--score',action='store_true')
    ap.add_argument('--suites',nargs='+',choices=['sensors','minigrid','arc'],default=['sensors','minigrid','arc'])
    ap.add_argument('--dev-train',type=int,default=256);ap.add_argument('--dev-validation',type=int,default=64)
    ap.add_argument('--dev-variants',nargs='+',choices=VARIANTS,default=VARIANTS)
    ap.add_argument('--arc-environments',default='/workspace/.aeterna-env/arc-environments');args=ap.parse_args()
    out=Path(args.output)
    if out.exists():raise FileExistsError('refusing to reuse an external attempt directory')
    out.mkdir(parents=True);state=commitment(args,out);result={'commitment':state,'status':'RUNNING'}
    try:
        for suite in args.suites:
            folder=out/suite;folder.mkdir()
            if suite=='sensors':result[suite]=sensors(args,folder)
            elif suite=='minigrid':result[suite]=grids(args,folder)
            else:
                from external5_arc import run_arc
                result[suite]=run_arc(args,folder)
            save(out/'summary.json',result)
        result['status']='PARTIAL_ARC_BLOCKED' if result.get('arc',{}).get('status')=='BLOCKED' else 'COMPLETED'
    except Exception as error:
        result['status']='ERROR';result['error']=f'{type(error).__name__}: {error}';raise
    finally:save(out/'summary.json',result)
    print(json.dumps({'status':result['status'],'output':str(out)},allow_nan=False),flush=True)

if __name__=='__main__':main()
