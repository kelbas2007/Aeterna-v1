#!/usr/bin/env python3
"""External FACTUAL action-affordance demonstration and independent heldout.

All task physics are the real Farama MiniGrid simulator. A demonstrator is
ALLOWED in *training* to position the virtual agent and perform one known
action; the Rust organism receives only (pointed word, raw PRE, motor ID, raw
POST) of that performed action. This is explicitly supervised motor learning,
NOT autonomous discovery. In heldout, evaluator may position an object one
tile ahead but NEVER sends a motor; native U1 + Human Protection must choose.
"""
import argparse
import json
from pathlib import Path
import numpy as np
import gymnasium as gym
import minigrid  # noqa: F401
from minigrid.core.constants import OBJECT_TO_IDX
from external_minigrid_benchmark import Agent, sensory

FRONT = 3 * 7 + 5
DIRECTION_VECTORS = ((1,0),(0,1),(-1,0),(0,-1))
TRAIN = {"key": ("MiniGrid-DoorKey-5x5-v0", 3),
         "door": ("MiniGrid-MultiRoom-N2-S4-v0", 5)}
HELDOUT = (
    ("key","MiniGrid-DoorKey-5x5-v0",3),
    ("door","MiniGrid-MultiRoom-N2-S4-v0",5),
)

def staged_observation(env, object_name):
    # Evaluator-only setup of an interaction opportunity. NO privileged
    # object position/direction enters the model; only the public obs image.
    base=env.unwrapped
    for y in range(1, base.height-1):
        for x in range(1, base.width-1):
            obj=base.grid.get(x,y)
            if obj is None or obj.type != object_name: continue
            for d,(dx,dy) in enumerate(DIRECTION_VECTORS):
                px,py=x-dx,y-dy
                if not (0<=px<base.width and 0<=py<base.height): continue
                stand=base.grid.get(px,py)
                if stand is not None and not stand.can_overlap(): continue
                base.agent_pos=np.array((px,py))
                base.agent_dir=d
                base.carrying=None
                obs=base.gen_obs()
                tiles=np.asarray(obs["image"]).reshape(-1,3)
                if int(tiles[FRONT,0])==OBJECT_TO_IDX[object_name]:
                    return obs
    raise RuntimeError(f"Unable to place an actual {object_name} ahead")

def send(agent,cmd):
    agent.write(cmd)
    return agent.read()

def teach_word(agent,word):
    result=send(agent,{"cmd":"teach_word","word":word,"tile":FRONT})
    if result.get("type")!="lesson_ack" or not result.get("learned"):
        raise RuntimeError(f"deictic lesson not accepted: {result}")

def train_affordance(agent,word,env_id,action,seed):
    env=gym.make(env_id)
    try:
        env.reset(seed=seed)
        pre=staged_observation(env,word)
        agent.reset(pre,learning=True)
        teach_word(agent,word)
        post,reward,terminated,truncated,_=env.step(action)
        if terminated or truncated:
            raise RuntimeError("training effect unexpectedly terminal")
        # Exact factual consequence of one externally demonstrated action.
        result=send(agent,{
            "cmd":"demonstrate","word":word,"tile":FRONT,
            "action":action,"before":sensory(pre),
            "after":sensory(post)
        })
        if result.get("type")!="demonstration_ack":
            raise RuntimeError(f"no demonstration receipt: {result}")
        return {
            "word":word,"staged":True,"action_supervised":action,
            "acquired":bool(result["acquired"]),
            "affordances":result["learned_affordances"],
            "public_target_changed": bool(np.any(
                np.asarray(pre["image"])[3,5,:] !=
                np.asarray(post["image"])[3,5,:])),
        }
    finally:
        env.close()

def holdout(agent,word,env_id,known_action,seed):
    env=gym.make(env_id)
    try:
        env.reset(seed=seed)
        obs=staged_observation(env,word)
        agent.reset(obs,learning=False)
        setting=send(agent,{"cmd":"word_intent","word":word})
        if setting.get("type")!="word_intent_ack" or not setting.get("accepted"):
            raise RuntimeError(f"learned word intent not accepted: {setting}")
        post,reward,terminated,truncated,response=agent.act_in(env)
        observed_motor=response.get("action") if post is not None else None
        changed=(post is not None and bool(np.any(
            np.asarray(obs["image"])[3,5,:] !=
            np.asarray(post["image"])[3,5,:])))
        # The evaluator is allowed to know what task effect should occur.
        # Native U1 never reads this correct action ID.
        return {
            "word":word, "seed":seed, "world":env_id,
            "native_action":observed_motor, "evaluator_action":known_action,
            "correct_action":observed_motor==known_action,
            "factual_object_change":changed, "reward":float(reward),
            "success":bool(changed and observed_motor==known_action),
            "stopped":response.get("reason"),
        }
    finally:
        env.close()

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--agent",type=Path,required=True)
    parser.add_argument("--output",type=Path,default=Path("external-functional1.json"))
    args=parser.parse_args()
    agent=Agent(args.agent)
    try:
        results=[]
        training=[
            train_affordance(agent,word,env_id,motor,20000+i)
            for i,(word,(env_id,motor)) in enumerate(TRAIN.items())
        ]
        before_restart=agent.status()
        restart=send(agent,{"cmd":"restart"})
        if restart.get("type")!="restart_ack":
            raise RuntimeError(f"checkpoint failed: {restart}")
        for word,env_id,control_motor in HELDOUT:
            for seed in range(22000,22012):
                results.append(holdout(
                    agent,word,env_id,control_motor,seed))
        after=agent.status()
        teacher_actions=2
        passed=sum(x["success"] for x in results)
        score={
            "training":training,
            "heldout":results,
            "teacher_motor_demonstrations":teacher_actions,
            "train_worlds":list(x[0] for x in TRAIN.values()),
            "heldout_seeds":list(range(22000,22012)),
            "same_lifetime":True,
            "carrier_before_restart":before_restart,
            "carrier_after":after,
            "checkpoint":restart,
            "total":len(results),
            "successful_object_interactions":passed,
            "task_goal_rewards":sum(int(x["reward"]>0) for x in results),
            "verdict":"DEVELOPMENT_PASS" if
                all(t["acquired"] for t in training)
                and before_restart["learned_affordances"]==2
                and after["learned_affordances"]==2
                and passed==len(results) else "DEVELOPMENT_FAIL",
        }
        args.output.write_text(json.dumps(score,indent=2)+"\n",encoding="utf-8")
        print(f"FUNCTIONAL_EXTERNAL1 teacher={teacher_actions} "
              f"witnesses={[x['acquired'] for x in training]} "
              f"affordances={after['learned_affordances']} "
              f"object_interactions={passed}/{len(results)} "
              f"goal_rewards={score['task_goal_rewards']}",flush=True)
        print(f"FUNCTIONAL_EXTERNAL1_SUMMARY verdict={score['verdict']} "
              "claim=SUPERVISED_MOTOR_AFFORDANCE_NOT_AUTONOMOUS_TASK_SOLVING",
              flush=True)
    finally:
        agent.close()

if __name__=="__main__":
    main()
