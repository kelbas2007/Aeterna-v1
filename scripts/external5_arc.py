"""Official ARC wrapper; no game implementation or solution is inspected."""
from __future__ import annotations
import hashlib,json,logging,os
from collections import Counter,deque
from pathlib import Path
from external5_transport import Native,new_agent

def targets(pixels):
    """At most ten generic component points, each inside its visible component."""
    h=len(pixels);w=len(pixels[0]);background=Counter(v for row in pixels for v in row).most_common(1)[0][0]
    seen=set();components=[]
    for y in range(h):
        for x in range(w):
            if (y,x) in seen or pixels[y][x]==background:continue
            token=pixels[y][x];queue=deque([(y,x)]);seen.add((y,x));points=[]
            while queue:
                cy,cx=queue.popleft();points.append((cy,cx))
                for ny,nx in [(cy-1,cx),(cy+1,cx),(cy,cx-1),(cy,cx+1)]:
                    if 0<=ny<h and 0<=nx<w and (ny,nx) not in seen and pixels[ny][nx]==token:
                        seen.add((ny,nx));queue.append((ny,nx))
            ay=sum(p[0] for p in points)/len(points);ax=sum(p[1] for p in points)/len(points)
            point=min(points,key=lambda p:((p[0]-ay)**2+(p[1]-ax)**2,p))
            components.append((len(points),token,point[0],point[1]))
    components.sort(key=lambda p:(-p[0],p[1],p[2],p[3]))
    return [(p[3],p[2]) for p in components[:10]] or [(w//2,h//2)]

def sensory(frame,namespace):
    raster=frame.frame[-1]
    if hasattr(raster,'tolist'):raster=raster.tolist()
    return {'kind':'scene','namespace':namespace,'pixels':raster}

def episodes(arc,agent,game,seeds,namespace,training):
    from arcengine import GameAction,GameState
    ordinary=[GameAction.ACTION1,GameAction.ACTION2,GameAction.ACTION3,GameAction.ACTION4,GameAction.ACTION5,GameAction.ACTION7]
    rows=[];version=None
    for seed in seeds:
        env=arc.make(game,seed=int(seed),save_recording=False)
        if env is None:raise RuntimeError('official game unavailable: '+game)
        version=env.environment_info.game_id;frame=env.observation_space
        if frame is None:frame=env.reset()
        if frame is None:raise RuntimeError('no official initial observation')
        observed=sensory(frame,namespace);agent.reset(observed,game=version,seed=int(seed),training=training)
        initial=frame.levels_completed;steps=0;reason='action_budget'
        for left in range(256,0,-1):
            if frame.state in (GameState.WIN,GameState.GAME_OVER):reason=frame.state.name;break
            points=targets(observed['pixels']);space=env.action_space
            available=[action in space for action in ordinary]+[GameAction.ACTION6 in space and i<len(points) for i in range(10)]
            def execute(action):
                nonlocal frame,observed
                motor=ordinary[action] if action<6 else GameAction.ACTION6
                data=None if action<6 else {'x':points[action-6][0],'y':points[action-6][1]}
                agent.trace.write({'kind':'arc_motor_intent','game':version,'action':motor.name,'data':data},flush=True)
                previous=frame.levels_completed;following=env.step(motor,data=data,reasoning={'source':'aeterna-native-u1-external5'})
                if following is None:raise RuntimeError('unknown official POST; no retry')
                frame=following;observed=sensory(frame,namespace)
                reward=float(frame.levels_completed>previous or frame.state==GameState.WIN)
                agent.trace.write({'kind':'official_public_outcome','game':version,'state':frame.state.name,
                    'levels_completed':frame.levels_completed,'win_levels':frame.win_levels})
                return {'frame':observed,'cost':1,'reward':reward}
            agent.step(available,left,execute);steps+=1
        agent.finish();row={'seed':int(seed),'actions':steps,'completed_levels':int(frame.levels_completed-initial),
            'win':frame.state==GameState.WIN,'state':frame.state.name,'stop_reason':reason}
        rows.append(row);agent.trace.write({'kind':'episode_result','training':training,**row})
        print(json.dumps({'phase':'arc','variant':agent.variant,'game':version,'training':training,**row}),flush=True)
    return {'game_version':version,'episodes':len(rows),'actions':sum(r['actions'] for r in rows),
            'completed_levels':sum(r['completed_levels'] for r in rows),'lives':rows}

def run_arc(args,out):
    import arc_agi
    from run_external_continual5 import VARIANTS,save,variants
    logger=logging.getLogger('aeterna.external5.arc');logger.setLevel(logging.ERROR)
    envdir=Path(args.arc_environments)
    arc=arc_agi.Arcade(operation_mode=arc_agi.OperationMode.OFFLINE,environments_dir=str(envdir),logger=logger)
    games=['ft09','vc33'] if args.score else ['ls20']
    available={e.game_id.split('-')[0] for e in arc.get_environments()}
    missing=set(games)-available
    if missing and not os.environ.get('ARC_API_KEY'):
        result={'status':'BLOCKED','unrun_games':sorted(missing),'levels_completed':None,
            'reason':'Official SDK needs ARC_API_KEY to download absent games; current environment has no key.',
            'required_environment_setting':'ARC_API_KEY','required_network_domain':'three.arcprize.org',
            'source_inspected':False,'held_seeds_consumed':False}
        save(out/'summary.json',result);return result
    if missing:
        downloader=arc_agi.Arcade(environments_dir=str(envdir),logger=logger)
        for game in sorted(missing):
            if downloader.make(game,seed=810000,save_recording=False) is None:raise RuntimeError('official game download failed: '+game)
        arc=arc_agi.Arcade(operation_mode=arc_agi.OperationMode.OFFLINE,environments_dir=str(envdir),logger=logger)
    # Bind downloaded files without reading their implementation into context.
    files={str(p.relative_to(envdir)):hashlib.sha256(p.read_bytes()).hexdigest()
           for p in envdir.rglob('*') if p.is_file() and p.suffix in ('.py','.json')}
    save(out/'official-environment-hashes.json',files)
    result={}
    for i,game in enumerate(games):
        folder=out/game;folder.mkdir();result[game]={}
        for variant in variants(args):
            current=folder/variant;current.mkdir()
            env=arc.make(game,seed=840100 if args.score else 810000,save_recording=False)
            if env is None:raise RuntimeError('official game unavailable')
            frame=env.observation_space
            if frame is None:frame=env.reset()
            initial=sensory(frame,30+i);agent=new_agent(args.binary,current/'training.jsonl.gz',variant,initial)
            entry={}
            try:
                entry['training']=episodes(arc,agent,game,range(840100,840108) if args.score else range(810000,810002),30+i,True)
                if isinstance(agent,Native):
                    agent.freeze();checkpoint=agent.checkpoint();agent.close()
                    agent=new_agent(args.binary,current/'held.jsonl.gz',variant,initial,checkpoint)
                before=agent.report()
                entry['evaluation']=episodes(arc,agent,entry['training']['game_version'],range(840000,840008) if args.score else range(810002,810004),30+i,False)
                after=agent.report()
                if isinstance(agent,Native) and before['learned_fingerprint']!=after['learned_fingerprint']:raise AssertionError('ARC evaluation changed frozen knowledge')
                entry['memory']=after
            finally:agent.close();save(current/'summary.json',entry)
            result[game][variant]=entry
    scores={v:sum(result[g][v]['evaluation']['completed_levels'] for g in games) for v in variants(args)}
    answer={'status':'COMPLETED','results':result,'completed_levels':scores,
        'target_met':scores['full']>=1 and scores['full']>scores['random'] if {'full','random'}<=scores.keys() else None,
        'click_interface':'six ordinary actions plus at most ten visible component points'}
    save(out/'summary.json',answer);return answer
