#!/usr/bin/env python3
"""Independent Farama MemoryS7: true historical cue must survive a corridor.

No env mission, start cue object type, correct exit, hidden map, global pose,
action names or optimal route enters native cognition. The evaluator
uses hidden state ONLY for retrospective scoring, with genuine reward from
Farama. Identical source is evaluated with and without history memory.
"""
import argparse
import json
from pathlib import Path

import gymnasium as gym
import minigrid  # noqa: F401, registers external environments
from external_minigrid_benchmark import Agent
from external_functional_grounding import send

WORLD = "MiniGrid-MemoryS7-v0"
TRAIN_SEEDS = range(80000, 80128)
HELDOUT_SEEDS = range(81000, 81024)
BUDGET=200

def run_episode(agent, seed, learning):
    env=gym.make(WORLD)
    try:
        observation,_=env.reset(seed=seed)
        agent.reset(observation,learning=learning)
        total=0.0
        actions=0
        error=None
        for _ in range(BUDGET):
            obs,reward,terminated,truncated,decision=agent.act_in(env)
            if obs is None:
                error=decision.get("reason","agent_stop")
                break
            total+=float(reward)
            actions+=1
            if terminated or truncated:break
        # These hidden-side attributes are for evaluation/debug only.
        # They are NEVER transmitted to Rust.
        actual=env.unwrapped
        return {
            "seed":seed, "reward":round(total,6),"success":total>0,
            "actions":actions,"stop":error,
            "terminal_position":list(map(int,actual.agent_pos)),
            "success_position":list(map(int,actual.success_pos)),
            "failure_position":list(map(int,actual.failure_pos))
        }
    finally:env.close()

def random_control(seed, rng):
    env=gym.make(WORLD)
    try:
        env.reset(seed=seed)
        reward_total=0.0
        for i in range(BUDGET):
            _,reward,terminated,truncated,_=env.step(rng.randrange(7))
            reward_total+=float(reward)
            if terminated or truncated:break
        return {"seed":seed,"success":reward_total>0,"reward":round(reward_total,6)}
    finally:env.close()

def main():
    import random
    p=argparse.ArgumentParser()
    p.add_argument("--agent",type=Path,required=True)
    p.add_argument("--memory",action="store_true")
    p.add_argument("--episodic-recall",action="store_true")
    p.add_argument("--seed-offset",type=int,default=0)
    p.add_argument("--output",type=Path,required=True)
    args=p.parse_args()
    if args.episodic_recall and not args.memory:
        raise SystemExit("episodic recall requires developmental memory")
    train_seeds=[i+args.seed_offset for i in TRAIN_SEEDS]
    test_seeds=[i+args.seed_offset for i in HELDOUT_SEEDS]
    native=Agent(args.agent)
    try:
        result=send(native,{"cmd":"general_policy"})
        if result.get("type")!="general_policy_ack" or not result.get("accepted"):
            raise RuntimeError(f"general policy refused: {result}")
        if args.memory:
            result=send(native,{"cmd":"developmental_memory"})
            if result.get("type")!="developmental_memory_ack" or not result.get("accepted"):
                raise RuntimeError(f"memory refused: {result}")
        if args.episodic_recall:
            result=send(native,{"cmd":"episodic_recall"})
            if result.get("type")!="episodic_recall_ack" or not result.get("accepted"):
                raise RuntimeError(f"episodic recall refused: {result}")
        training=[run_episode(native,seed,True) for seed in train_seeds]
        learned=native.status()
        resumed=send(native,{"cmd":"restart"})
        if resumed.get("type")!="restart_ack":
            raise RuntimeError(f"native restart failed: {resumed}")
        heldout=[run_episode(native,seed,False) for seed in test_seeds]
        after=native.status()
    finally:native.close()
    randomizer=random.Random(0xABCEFF)
    random_eval=[random_control(seed,randomizer) for seed in test_seeds]
    report={
        "library":"Farama MiniGrid 3.1.0",
        "world":WORLD,"training_seeds":train_seeds,
        "heldout_seeds":test_seeds,"budget":BUDGET,
        "memory":args.memory,"episodic_recall":args.episodic_recall,
        "mission_sent":False,
        "hidden_state_sent":False,"correct_exit_sent":False,
        "motor_demonstrations":0,"episode_staging":False,
        "train":training,"heldout":heldout,
        "random_heldout":random_eval,"carrier_train":learned,
        "carrier_frozen":after,"restart":resumed,
        "train_successes":sum(int(x["success"]) for x in training),
        "heldout_successes":sum(int(x["success"]) for x in heldout),
        "random_successes":sum(int(x["success"]) for x in random_eval),
        "frozen_executions":sum(x["actions"] for x in heldout)
    }
    if report["frozen_executions"]==0:
        raise RuntimeError("INVALID_MEMORY_RUN: zero factual heldout actions")
    args.output.write_text(json.dumps(report,indent=2)+"\n",encoding="utf8")
    print(f"FARAMA_MEMORY_HISTORY memory={args.memory} "
          f"train={report['train_successes']}/{len(training)} "
          f"heldout={report['heldout_successes']}/{len(heldout)} "
          f"random={report['random_successes']}/{len(heldout)} "
          f"frozen_steps={report['frozen_executions']} "
          f"learned_rewards={learned.get('general_rewards',0)} "
          f"learned_updates={learned.get('general_updates',0)} "
          f"learned_events={learned.get('rewarded_episodic_memories',0)}",flush=True)
if __name__=="__main__":main()
