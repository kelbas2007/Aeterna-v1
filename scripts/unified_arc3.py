"""Opaque adapter: no rules, solution scripts or LLM calls. Frozen ls20, 256 actions.
Original raster frames are preserved alongside the lossy 32-feature interface.
"""
from __future__ import annotations
import argparse, hashlib, json, logging, os, subprocess
from pathlib import Path
from collections import Counter
from enum import Enum

def encode(raster) -> list[int]:
    if hasattr(raster,'tolist'):raster=raster.tolist()
    if not raster or not raster[0] or len(raster)>256 or len(raster[0])>256:raise ValueError('invalid raster')
    height,width=len(raster),len(raster[0])
    if height<4 or width<4 or any(len(row)!=width for row in raster):raise ValueError('invalid raster shape')
    flat=[int(v) for row in raster for v in row]
    if any(v<0 or v>15 for v in flat):raise ValueError('palette outside contract')
    counts=Counter(flat);result=[counts.get(c,0) for c in range(16)]
    for by in range(4):
        for bx in range(4):
            pixels=[int(raster[y][x]) for y in range(by*height//4,(by+1)*height//4) for x in range(bx*width//4,(bx+1)*width//4)]
            result.append(sum(pixels)*16//len(pixels))
    return result

def dump(value):
    if hasattr(value,'model_dump'):
        result=dump(value.model_dump(mode='python'))
        # The official FrameDataRaw excludes pixels from model_dump by default.
        if hasattr(value,'frame'):result['frame']=dump(value.frame)
        return result
    if hasattr(value,'tolist'):return dump(value.tolist())
    if isinstance(value,Enum):return value.name
    if isinstance(value,dict):return {str(k):dump(v) for k,v in value.items()}
    if isinstance(value,(tuple,list)):return [dump(v) for v in value]
    return value

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',required=True);ap.add_argument('--trained',required=True)
    ap.add_argument('--game',default='ls20');ap.add_argument('--max-actions',type=int,default=256);ap.add_argument('--output',required=True)
    args=ap.parse_args()
    if args.game!='ls20' or not 1<=args.max_actions<=256:raise ValueError('frozen public development scope')
    import arc_agi
    from arcengine import GameAction,GameState
    out=Path(args.output);out.mkdir(parents=True,exist_ok=True)
    if (out/'events.jsonl').exists():raise FileExistsError('refusing to overwrite an external attempt')
    logger=logging.getLogger('aeterna.arc');logger.setLevel(logging.ERROR)
    arc=arc_agi.Arcade(operation_mode=arc_agi.OperationMode.OFFLINE,logger=logger)
    env=arc.make(args.game,seed=0,save_recording=False)
    if env is None:raise RuntimeError('preloaded official game unavailable')
    frame=env.observation_space
    if frame is None:frame=env.reset()
    if frame is None:raise RuntimeError('no initial observation')
    action_map=[GameAction.ACTION1,GameAction.ACTION2,GameAction.ACTION3,GameAction.ACTION4]
    binary=Path(args.binary).resolve();trained=Path(args.trained).read_text()
    with (out/'agent-stderr.log').open('w') as stderr, (out/'events.jsonl').open('x') as events:
        child=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
        def send(v):child.stdin.write(json.dumps(v,separators=(',',':'))+'\n');child.stdin.flush()
        def receive():
            line=child.stdout.readline(34*1024*1024)
            if not line or not line.endswith('\n'):raise RuntimeError('agent transport closed/oversized')
            value=json.loads(line)
            if value.get('type')=='error':raise RuntimeError('agent: '+value.get('message','unknown'))
            return value
        def command(v):send(v);return receive()
        def record(v):events.write(json.dumps(v,separators=(',',':'),allow_nan=False)+'\n');events.flush();os.fsync(events.fileno())
        actions=0;seen=set();reason='action_budget';status='RUNNING';report=None
        initial_levels=frame.levels_completed
        try:
            loaded=command({'op':'load','checkpoint':trained})
            initial=encode(frame.frame[-1])
            entered=command({'op':'enter','name':'arc3-ls20-raster-v1','observation':initial})
            record({'kind':'initial','raw':dump(frame),'encoded':initial,'loaded':loaded,'entered':entered})
            for index in range(args.max_actions):
                if frame.state in (GameState.WIN,GameState.GAME_OVER):reason=frame.state.name;break
                available=[a in env.action_space for a in action_map]
                send({'op':'step','available':available});request=receive()
                if request.get('type')!='action':raise RuntimeError('agent did not request action')
                a=request['action']
                if not isinstance(a,int) or a not in range(4) or not available[a]:raise RuntimeError('invalid chosen action')
                record({'kind':'intent','index':index,'action_index':a,'action':action_map[a].name})
                old_levels=frame.levels_completed
                following=env.step(action_map[a],reasoning={'source':'aeterna-unified','index':index});actions+=1
                if following is None:raise RuntimeError('external outcome unknown; stop without retry')
                frame=following
                encoded=encode(frame.frame[-1]);reward=float(frame.levels_completed>old_levels or frame.state==GameState.WIN)
                send({'op':'post','result':{'observation':encoded,'cost':1,'reward':reward}});commit=receive()
                if commit.get('type')!='committed':raise RuntimeError('missing factual commit')
                record({'kind':'post','index':index,'raw':dump(frame),'encoded':encoded,'commit':commit})
                seen.add(tuple(encoded))
            report=command({'op':'report'})['report']
            saved=command({'op':'checkpoint'})['checkpoint'];(out/'after-external.json').write_text(saved)
            command({'op':'quit'});child.wait(timeout=10);status='COMPLETED'
        except Exception as error:
            reason=f'{type(error).__name__}: {error}';status='ERROR';raise
        finally:
            if child.poll() is None:child.kill();child.wait()
            summary={'kind':'official-public-ARC-AGI-3-development-attempt','status':status,'game_id':env.info.game_id,
                'seed':0,'action_limit':args.max_actions,'actions_executed':actions,'stop_reason':reason,
                'game_state':frame.state.name,'initial_levels':initial_levels,'levels_completed':frame.levels_completed,
                'win_levels':getattr(frame,'win_levels',None),'win':frame.state==GameState.WIN,'distinct_encoded_frames':len(seen),
                'pretrained_checkpoint_sha256':hashlib.sha256(trained.encode()).hexdigest(),
                'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'representation':'16 palette counts + 16 coarse tile means',
                'agent_game_source_access':False,'solutions_injected':False,'llm_policy_calls':0,
                'official_leaderboard_submission':False,'policy_actions':'all chosen by Rust ScientificRuntime; Python only executes',
                'transfer':'U1 physical weights retained; numerical learner archived; raster model starts untrained',
                'final_report':report,'scorecard':dump(arc.get_scorecard())}
            (out/'summary.json').write_text(json.dumps(summary,indent=2))
    print(json.dumps(summary,indent=2))
if __name__=='__main__':main()
