#!/usr/bin/env python3
"""Factual goal->probe->model->action demo through the canonical JSONL app.

Evaluator-only virtual world maps opaque actions to four effects. No role,
plan, action labels or unexecuted future result is passed into cognition.
This is a DEVELOPMENT diagnostic, NOT an independently blinded AGI benchmark.
"""
import argparse
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
ROLES=(2,0,3,1)  # Only in evaluator, never sent to AETERNA


def physical_post(world,action):
    if action==ROLES[0]:
        world[1]=1
    elif action==ROLES[1]:
        if world[1]==1:world[2]=1
    elif action==ROLES[2]:
        if world[2]==1:world[0]+=1
    elif action==ROLES[3]:
        world[2]=0
    return {"observation":list(world),"reward":0.0,"cost":1}


class Session:
    def __init__(self,binary):
        self.p=subprocess.Popen([str(binary),"--jsonl"],cwd=ROOT,
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
            text=True,bufsize=1)
    def command(self,payload):
        self.p.stdin.write(json.dumps(payload,separators=(',',':'))+'\n');self.p.stdin.flush()
        line=self.p.stdout.readline()
        if not line:raise RuntimeError('AETERNA exited before response')
        result=json.loads(line)
        if result.get('type')=='error':raise RuntimeError(str(result))
        return result
    def step(self,world,goal):
        self.p.stdin.write(json.dumps({"op":"step","goal":[goal,None,None],"available":[True]*4})+'\n');self.p.stdin.flush()
        request=json.loads(self.p.stdout.readline())
        if request.get('type')!='action':raise RuntimeError('Expected selected motor before POST: '+str(request))
        response=self.command({"op":"post","result":physical_post(world,request['action'])})
        if response.get('type')!='committed':raise RuntimeError('Expected committed factual PRE/POST: '+str(response))
        return response['step']['source']
    def close(self):
        if self.p.poll() is None:
            try:self.command({"op":"quit"})
            except (ValueError,OSError,RuntimeError):pass
            self.p.stdin.close();self.p.stdout.close()
            self.p.wait(timeout=10)
        if self.p.returncode:
            raise RuntimeError('AETERNA subprocess failed: '+self.p.stderr.read()[-1000:])
        self.p.stderr.close()


def run(binary):
    world=[0,0,0];s=Session(binary)
    try:
        assert s.command({"op":"init","width":3})['type']=='initialized'
        assert s.command({"op":"enable_self_directed_control"})['type']=='self_directed_enabled'
        sources={};n=0
        while n<112 and world[0]<3:
            source=s.step(world,3);sources[source]=sources.get(source,0)+1;n+=1
        if world[0]<3:raise AssertionError(f'cold goal failed after {n} actions: {world}')
        checkpoint=s.command({"op":"checkpoint"})['checkpoint']
    finally:s.close()
    # A completely new OS process receives ONLY the joint checkpoint.
    revived=Session(binary)
    try:
        revived.command({"op":"load","checkpoint":checkpoint})
        restored=revived.command({"op":"checkpoint"})['checkpoint']
        if restored!=checkpoint:raise AssertionError('checkpoint changed upon roundtrip')
        more=0
        while more<36 and world[0]<6:
            revived.step(world,6);more+=1
        if world[0]<6:raise AssertionError(f'learned consequences not reused: {world}')
        result={"status":"DEVELOPMENT_PASS","first_goal":3,"first_actions":n,
            "new_goal":6,"reused_actions":more,"world":world,"sources":sources,
            "learned_motor_tuition":False,"separate_process_restore":True,
            "one_scientific_runtime":True,"independent_external_benchmark":False}
        return result
    finally:revived.close()


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True)
    p.add_argument('--output',type=Path);a=p.parse_args()
    result=run(a.binary.resolve())
    print(json.dumps(result,ensure_ascii=False,indent=2))
    if a.output:
        a.output.parent.mkdir(parents=True,exist_ok=True)
        a.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
