#!/usr/bin/env python3
"""A separate Python world for the real JSONL executable. Standard library only.
The host supplies no solution, learning target, constructor or action schedule.
A new session directory is required: old experience is never silently erased.
"""
from __future__ import annotations
import argparse,json,os,subprocess,sys
from pathlib import Path
from typing import Any
class Host:
    def __init__(self,binary:Path,folder:Path)->None:
        self.process=subprocess.Popen([str(binary.resolve()),str(folder.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True,encoding='utf-8',bufsize=1)
        self.ready=self.read()
    def read(self)->dict[str,Any]:
        assert self.process.stdout is not None
        line=self.process.stdout.readline()
        if not line:raise RuntimeError(f'runtime exited: {self.process.poll()}')
        value=json.loads(line)
        if not isinstance(value,dict):raise RuntimeError('invalid runtime message')
        return value
    def send(self,command:dict[str,Any])->None:
        assert self.process.stdin is not None
        self.process.stdin.write(json.dumps(command,separators=(',',':'))+'\n');self.process.stdin.flush()
    def request(self,command:dict[str,Any])->dict[str,Any]:
        self.send(command);response=self.read()
        if response.get('event')=='error':raise RuntimeError(response['error'])
        return response
    def close(self)->None:
        if self.process.poll() is None:self.send({'op':'close'});self.process.wait(timeout=10)
        if self.process.returncode!=0:raise RuntimeError(f'runtime exit code {self.process.returncode}')
    def abort(self)->None:
        if self.process.poll() is None:self.process.kill()
        self.process.wait(timeout=10)
def main()->None:
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--session',type=Path,required=True);args=parser.parse_args()
    if not args.binary.is_file():parser.error('binary does not exist; build aeterna-construction first')
    if args.session.exists():parser.error('use a new directory; existing experience is not overwritten')
    args.session.mkdir(parents=True)
    search=dict(work=200_000,max_expr_nodes=1,max_frontier=4096,max_updates=16,max_state=0,deterministic=True)
    config=dict(width=1,actions=2,search=search,minimum_training=8,search_every=4,shadow_steps=4,journal_capacity=256,planning_depth=3,planning_work=20000)
    ceiling=dict(work=2_000_000,max_expr_nodes=7,max_frontier=32768,max_updates=24,max_state=2,deterministic=True)
    policy=dict(ceiling=ceiling,failures_per_expansion=1,max_external_cost=256,evidence=None)
    safe=dict(human_present=False,physical_effect_possible=False,predicted_harm_probability=0.0,hazard_confidence=1.0,emergency_stop=False)
    offers=[dict(available=True,max_cost=1) for _ in range(2)]
    host=Host(args.binary,args.session);world={'hidden':0,'observation':[0],'events':0};resumed=False
    try:
        state=host.request(dict(op='start',config=config,policy=policy,observation=[0]))
        for i in range(160):
            if i==128:state=host.request({'op':'freeze'})
            host.send(dict(op='step',offers=offers,hazards=[safe,safe]));action=host.read()
            if action.get('event')!='act' or action.get('id')!=i:raise RuntimeError(f'unexpected action message: {action}')
            post=[world['hidden']];world={'hidden':action['action'],'observation':post,'events':i+1}
            state=host.request(dict(op='post',event=i,action=action['action'],observation=post,cost=1))
            if i==63:
                # This is a completed action. The host must preserve its own world,
                # not claim hidden-state continuity just from a matching sensor.
                save=args.session/'demo-world.json'
                with save.open('w',encoding='utf-8') as stream:
                    json.dump(dict(world=world,continuity=state['continuity']),stream);stream.flush();os.fsync(stream.fileno())
                host.close();saved=json.loads(save.read_text(encoding='utf-8'));world=saved['world'];host=Host(args.binary,args.session)
                if host.ready['continuity']!=saved['continuity']:raise RuntimeError('world/runtime continuity mismatch')
                state=host.request(dict(op='resume',continuity=saved['continuity']));resumed=True
        host.close()
        success=resumed and state['learner']['state_cells']>0 and state['learner']['promotions']>0 and state['autonomy']['external_cost']==160 and state['autonomy']['frozen_at']==128
        print(json.dumps(dict(kind='external-process-adapter-development',independent_qualification=False,commands_selected_by_runtime=160,process_restart=resumed,injected_solutions=0,injected_constructors=0,success=success,state=state),ensure_ascii=False,indent=2))
        if not success:raise RuntimeError('adapter criterion failed; retain session for inspection')
    finally:host.abort()
if __name__=='__main__':
    try:main()
    except (OSError,ValueError,RuntimeError,subprocess.SubprocessError) as exc:print(str(exc),file=sys.stderr);sys.exit(1)
