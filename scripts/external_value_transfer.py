#!/usr/bin/env python3
"""Frozen generic rule transfer. Evaluator metadata never enters Rust."""
import argparse
import gzip
import hashlib
import json
import random
import subprocess
from pathlib import Path

import gymnasium as gym
import external_relational_hypothesis_pairs as paired
from external_minigrid_benchmark import Agent
from external_context_value import PublicAgent


class RuleAgent(PublicAgent):
    def write(self,message):
        if message.get('cmd') in {'value_abstraction','value_abstraction_lesion','value_predicates'}:
            if set(message)!={'cmd'}:raise ValueError('nonpublic rule command fields')
            Agent.write(self,message)
        else:super().write(message)


def configure(binary):
    agent=RuleAgent(binary)
    for cmd in ['general_policy','developmental_memory','relational_workspace',
                'context_value_learning','value_abstraction']:
        paired.checked_command(agent,cmd,cmd+'_ack')
    agent.relational_mode=True
    return agent


def navigation(agent,world,seed,baseline=None):
    env=gym.make(world,agent_start_pos=None)
    try:
        obs,_=env.reset(seed=seed);base=env.unwrapped
        initial=[int(x) for x in base.agent_pos]+[int(base.agent_dir)]
        if agent:
            paired.checked_command(agent,'restart','restart_ack');agent.reset(obs,False)
        rng=random.Random(seed+198002);actions=[];hashes=[];success=False
        for _ in range(200):
            hashes.append(paired.public_hash(obs))
            if agent:
                obs,reward,done,truncated,choice=agent.act_in(env)
                if obs is None:break
                action=choice['action']
            else:
                # Explicit authored baseline, with known sensor/motor meanings;
                # none of these meanings are supplied to the native organism.
                action=rng.randrange(7) if baseline=='random' else (
                    1 if int(obs['image'][3,5,0])==2 else 2)
                obs,reward,done,truncated,_=env.step(action)
            actions.append(action);success|=reward>0
            if done or truncated:break
        return {'seed':seed,'world':world,'initial':initial,'actions':actions,
                'input_sha256':hashes,'success':bool(success)}
    finally:env.close()


def save(path,report):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(gzip.compress((json.dumps(report,sort_keys=True)+'\n').encode(),mtime=0))


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--agent',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    if subprocess.check_output(['git','status','--porcelain','--','src','Cargo.toml','Cargo.lock'],text=True):
        raise SystemExit('Commit cognitive source before fresh evaluation')
    report={'protocol':'VALUE-RULE-TRANSFER-1','status':'open development',
        'source':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
        'binary_sha256':hashlib.sha256(args.agent.read_bytes()).hexdigest(),
        'budget':200,'train':[],'arms':{}}
    agent=configure(args.agent)
    try:
        for seed in range(194000,194512):
            report['train'].append(paired.episode(agent,seed,True))
            if len(report['train'])%32==0:
                print(json.dumps({'trained':len(report['train']),'successes':sum(e['success'] for e in report['train']),
                    'rules':agent.status()['value_abstraction']}),flush=True);save(args.output,report)
        for arm in ['rules','lesioned']:
            if arm=='lesioned':paired.checked_command(agent,'value_abstraction_lesion','value_abstraction_lesion_ack')
            before=agent.status();results={'memory':{},'navigation':[]}
            for world,start in [('MiniGrid-MemoryS7-v0',195000),('MiniGrid-MemoryS9-v0',196000)]:
                paired.WORLD=world;pairs=[]
                for seed in range(start,start+32):
                    a=paired.episode(agent,seed,False,restart=True)
                    b=paired.episode(agent,seed,False,cue_swapped=True,restart=True)
                    pairs.append({'seed':seed,'original':a,'cue_swapped':b})
                results['memory'][world]={'summary':paired.score_pairs(pairs),'pairs':pairs}
                print(json.dumps({'arm':arm,'world':world,**paired.score_pairs(pairs)}),flush=True)
            for world,start in [('MiniGrid-Empty-8x8-v0',207000),('MiniGrid-Empty-16x16-v0',207032)]:
                rows=[navigation(agent,world,s) for s in range(start,start+32)]
                results['navigation'].extend(rows)
                print(json.dumps({'arm':arm,'world':world,'successes':sum(r['success'] for r in rows),'episodes':len(rows)}),flush=True)
            after=agent.status()
            for key in ['general_updates','general_rewards','value_states','value_abstraction',
                        'rewarded_episodic_memories','failed_episodic_experiences']:
                if before[key]!=after[key]:raise ValueError('frozen knowledge changed')
            results.update(before=before,after=after);report['arms'][arm]=results;save(args.output,report)
        agent.write({'cmd':'value_predicates'});report['predicates']=agent.read()
    finally:agent.close();paired.WORLD='MiniGrid-MemoryS7-v0'
    for arm in ['cold','random','authored']:
        cold=configure(args.agent) if arm=='cold' else None
        try:
            rows=[]
            for world,start in [('MiniGrid-Empty-8x8-v0',207000),('MiniGrid-Empty-16x16-v0',207032)]:
                rows.extend(navigation(cold,world,s,arm) for s in range(start,start+32))
            report['arms'][arm]={'navigation':rows}
            print(json.dumps({'arm':arm,'navigation_successes':sum(r['success'] for r in rows),'episodes':len(rows)}),flush=True)
        finally:
            if cold:cold.close()
        save(args.output,report)
    scores={k:sum(r['success'] for r in a['navigation']) for k,a in report['arms'].items()}
    per_world={w:sum(r['success'] for r in report['arms']['rules']['navigation'] if r['world']==w)
               for w in ['MiniGrid-Empty-8x8-v0','MiniGrid-Empty-16x16-v0']}
    retention=report['arms']['rules']['memory']['MiniGrid-MemoryS7-v0']['summary']['successes']>=48
    passed=retention and scores['rules']>=48 and min(per_world.values())>=20 and all(
        scores['rules']>=scores[k]+16 for k in ['lesioned','cold','random'])
    report['navigation_verdict']='CROSS_TASK_DEVELOPMENT_PASS' if passed else 'CROSS_TASK_DEVELOPMENT_FAIL'
    s9=report['arms']['rules']['memory']['MiniGrid-MemoryS9-v0']['summary']
    control=report['arms']['lesioned']['memory']['MiniGrid-MemoryS9-v0']['summary']
    report['memory_s9_verdict']='PASS' if s9['cue_use_supported'] and s9['both_correct']>=control['both_correct']+8 else 'FAIL'
    save(args.output,report);print(json.dumps({'navigation':report['navigation_verdict'],'s9':report['memory_s9_verdict'],'scores':scores}),flush=True)


if __name__=='__main__':main()
