#!/usr/bin/env python3
"""EXTERNAL FUNCTIONAL CROSS-WORLD: identical live agent, distinct Farama task families.

Tutor trains only DoorKey key-pickup and MultiRoom door-toggle. New heldout
family Unlock tests key pickup; DoorKey locked door with a key available
tests door-toggle conditioned on experimentally staged inventory. External
evaluator stages proximity/precondition for a *skill* test. No motor is
transmitted during heldout; U1 and Human Protection own execution.
This is NOT autonomous navigation nor inference of unlocking prerequisites.
"""
import argparse
import json
from pathlib import Path
import gymnasium as gym
import numpy as np
import minigrid  # noqa
from minigrid.core.world_object import Key
from external_functional_grounding import (
    train_affordance, staged_observation, FRONT, send
)
from external_minigrid_benchmark import Agent

def evaluate(agent,word,world,seed):
    env=gym.make(world)
    try:
        env.reset(seed=seed)
        pre=staged_observation(env,word)
        prepared_inventory=False
        if word=="door":
            # Evaluator-only precondition preparation. Locked DoorKey
            # otherwise cannot open by toggle alone. This actual simulator
            # inventory placement is NOT told to the organism as a label.
            env.unwrapped.carrying=Key("yellow")
            pre=env.unwrapped.gen_obs()
            prepared_inventory=True
        agent.reset(pre,learning=False)
        response=send(agent,{"cmd":"word_intent","word":word})
        if response.get("type")!="word_intent_ack" or not response.get("accepted"):
            raise RuntimeError(f"word intent not grounded: {response}")
        agent.write({"cmd":"advance"})
        selected=agent.read()
        if selected.get("type")!="action":
            return {"world":world,"word":word,"seed":seed,"success":False,
                    "reason":selected}
        action=selected["action"]
        post,reward,terminated,truncated,_=env.step(action)
        agent.write({"cmd":"post","observation":__import__("external_minigrid_benchmark").sensory(post),"reward":float(reward)})
        ack=agent.read()
        if ack.get("type")!="advance_ack" or ack.get("executed")!=action:
            raise RuntimeError(f"unacknowledged physical action: {ack}")
        changed=bool(np.any(
            np.asarray(pre["image"])[3,5,:] !=
            np.asarray(post["image"])[3,5,:]
        ))
        # Only evaluation sees its known exact motor identity: MiniGrid's
        # externally maintained definitions are not imported into EvoPhase.
        expected=3 if word=="key" else 5
        return {
            "world":world,"word":word,"seed":seed,
            "native_action":action,"expected_evaluator_only":expected,
            "effect_observed":changed,"success":action==expected and changed,
            "prepared_inventory":prepared_inventory,
            "reward":float(reward)
        }
    finally:
        env.close()

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--agent",type=Path,required=True)
    p.add_argument("--output",type=Path,default=Path("functional-crossworld2.json"))
    args=p.parse_args()
    native=Agent(args.agent)
    try:
        training=[
            train_affordance(native,"key","MiniGrid-DoorKey-5x5-v0",3,31001),
            train_affordance(native,"door","MiniGrid-MultiRoom-N2-S4-v0",5,31002)
        ]
        before=native.status()
        restart=send(native,{"cmd":"restart"})
        if restart.get("type")!="restart_ack": raise RuntimeError("restart failed")
        test=[]
        for word,world in (
            ("key","MiniGrid-Unlock-v0"),
            ("door","MiniGrid-DoorKey-5x5-v0"),
        ):
            for seed in range(33000,33012):
                test.append(evaluate(native,word,world,seed))
        after=native.status()
        total=sum(int(x["success"]) for x in test)
        verdict=("DEVELOPMENT_PASS" if total==len(test)
                 and all(x["acquired"] for x in training)
                 and before["learned_affordances"]==2
                 and after["learned_affordances"]==2 else "DEVELOPMENT_FAIL")
        report={
            "source":"Farama MiniGrid 3.1.0 independent simulator",
            "training_supervision":"two real staged motor demonstrations",
            "testing_supervision":"only word and externally staged visible object/precondition",
            "training":training,"heldout":test,"restart":restart,
            "heldout_seeds":list(range(33000,33012)),
            "heldout_successes":total,"heldout_total":len(test),
            "terminal_task_rewards":sum(x["reward"]>0 for x in test),
            "verdict":verdict
        }
        args.output.write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
        print(f"FUNCTIONAL_CROSSWORLD2_SUMMARY successful={total}/{len(test)} "
              f"rewards={report['terminal_task_rewards']} verdict={verdict} "
              "claim=STAGED_SUPERVISED_CROSSWORLD_SKILLS",flush=True)
    finally:
        native.close()

if __name__=="__main__":
    main()
