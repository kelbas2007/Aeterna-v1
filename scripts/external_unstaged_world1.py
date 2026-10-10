#!/usr/bin/env python3
"""Unstaged cross-world MiniGrid test: no teacher, no object placement, no hints.

ONE continuous EvoPhase organism encounters independent DoorKey,
MultiRoom and Unlock worlds interleaved. The environment gives only its
public image and real reward. Evaluator may inspect simulator INTERNAL
object states solely for OUTCOME SCORING; none are sent to Rust.
"""
import argparse
import json
import random
from pathlib import Path

import gymnasium as gym
import minigrid  # noqa: F401

from external_minigrid_benchmark import Agent, random_control
from external_functional_grounding import FRONT, send

WORLDS = (
    "MiniGrid-DoorKey-5x5-v0",
    "MiniGrid-MultiRoom-N2-S4-v0",
    "MiniGrid-Unlock-v0",
)
TRAIN_SEEDS = tuple(range(50000,50012))
EVAL_SEEDS = tuple(range(51000,51004))
BUDGET = 128


def real_episode(agent, name, seed, learning, budget=BUDGET):
    env = gym.make(name)
    try:
        obs, _ = env.reset(seed=seed)
        agent.reset(obs,learning=learning)
        facts = {
            "seed":seed,"world":name,"learning":learning,
            "actions":0,"reward":0.0,"actual_key_pickups":0,
            "actual_door_openings":0,"moves":0,"stopped":None
        }
        for _ in range(budget):
            # External OUTCOME auditor; no simulator variables enter cognition.
            world=env.unwrapped
            front=world.grid.get(*world.front_pos)
            carrying_before=world.carrying
            open_before=bool(getattr(front, "is_open", False))
            position_before=tuple(world.agent_pos)
            obs, reward, terminated, truncated, response=agent.act_in(env)
            if obs is None:
                facts["stopped"]=response.get("reason")
                break
            facts["actions"]+=1
            facts["reward"]+=float(reward)
            facts["moves"]+=int(tuple(world.agent_pos)!=position_before)
            facts["actual_key_pickups"]+=int(
                front is not None and getattr(front,"type",None)=="key"
                and carrying_before is None and world.carrying is front)
            facts["actual_door_openings"]+=int(
                front is not None and getattr(front,"type",None)=="door"
                and not open_before and bool(getattr(front,"is_open",False)))
            if terminated or truncated: break
        facts["success"]=facts["reward"]>0
        facts["reward"]=round(facts["reward"],6)
        return facts
    finally:
        env.close()


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--agent",type=Path,required=True)
    parser.add_argument("--output",type=Path,default=Path("unstaged-crossworld1.json"))
    parser.add_argument("--seed-offset",type=int,default=0)
    parser.add_argument("--embodied",action="store_true")
    parser.add_argument("--general-policy",action="store_true")
    parser.add_argument("--budget",type=int,default=BUDGET)
    parser.add_argument("--include-empty",action="store_true")
    args=parser.parse_args()
    if args.general_policy and args.embodied:
        raise SystemExit("choose either general-policy or embodied baseline")
    if not 16<=args.budget<=256:
        raise SystemExit("budget must be 16..256")
    worlds=(("MiniGrid-Empty-5x5-v0",)+WORLDS
        if args.include_empty else WORLDS)
    train_seeds=[x+args.seed_offset for x in TRAIN_SEEDS]
    eval_seeds=[x+args.seed_offset for x in EVAL_SEEDS]
    agent=Agent(args.agent)
    train=[]
    heldout=[]
    try:
        reply=send(agent,{"cmd":"self_experiment","front_tile":FRONT})
        if not reply.get("accepted"): raise RuntimeError(f"embodiment refused: {reply}")
        if args.general_policy:
            decision=send(agent,{"cmd":"general_policy"})
            if decision.get("type")!="general_policy_ack" or not decision.get("accepted"):
                raise RuntimeError(f"replacement policy refused: {decision}")
        if args.embodied:
            motion=send(agent,{"cmd":"embodied_navigation"})
            if motion.get("type")!="embodied_navigation_ack" or not motion.get("accepted"):
                raise RuntimeError(f"motion exploration refused: {motion}")
        # Equal exposure to independent physics from a common persistent
        # organism, NOT a fresh model instance per family.
        for seed in train_seeds:
            for env_id in worlds:
                train.append(real_episode(agent,env_id,seed,True,args.budget))
        before_restart=agent.status()
        restart=send(agent,{"cmd":"restart"})
        if restart.get("type")!="restart_ack": raise RuntimeError("native restart failed")
        for seed in eval_seeds:
            for env_id in worlds:
                heldout.append(real_episode(agent,env_id,seed,False,args.budget))
        after_restart=agent.status()
    finally:
        agent.close()
    random_results={}
    for name in worlds:
        random_results[name]=random_control(
            name,eval_seeds,args.budget,seed_base=616161 + len(name))
    train_tasks=sum(e["success"] for e in train)
    eval_tasks=sum(e["success"] for e in heldout)
    random_tasks=sum(e["success"] for batch in random_results.values() for e in batch)
    report={
        "task":"one lifetime unstaged external task-world transfer",
        "train_seeds":train_seeds,"heldout_seeds":eval_seeds,
        "seed_offset":args.seed_offset,
        "worlds":worlds,"budget":args.budget,
        "embodied_navigation":args.embodied,
        "general_policy":args.general_policy,
        "teacher_motor_demonstrations":0,
        "teacher_words":0,
        "staged_objects":0,
        "staged_prerequisites":0,
        "train":train,"heldout":heldout,"random":random_results,
        "before_restart":before_restart,"restart":restart,
        "after_restart":after_restart,
        "train_task_successes":train_tasks,"eval_task_successes":eval_tasks,
        "heldout_successes_by_world":{
            name:sum(e["success"] for e in heldout if e["world"]==name)
            for name in worlds
        },
        "random_task_successes":random_tasks,
        "trained_effects":before_restart["self_affordances"],
        "general_training_rewards":before_restart.get("general_rewards",0),
        "general_training_updates":before_restart.get("general_updates",0),
        "heldout_effects":after_restart["self_affordances"],
        "eval_key_pickups":sum(e["actual_key_pickups"] for e in heldout),
        "eval_door_openings":sum(e["actual_door_openings"] for e in heldout),
        "eval_executed_moves":sum(e["moves"] for e in heldout),
        "native_inferred_motion_events":after_restart.get("inferred_motion",0),
        "total_physical_actions":sum(e["actions"] for e in train+heldout),
    }
    report["verdict"]=(
        "DEVELOPMENT_PASS" if eval_tasks>=6
        and eval_tasks>random_tasks
        and report["eval_key_pickups"]>0
        and report["eval_door_openings"]>0
        else "DEVELOPMENT_FAIL")
    args.output.write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
    print("UNSTAGED_WORLD1 train_goals={}/{} frozen_goals={}/{} random={}/{} "
          "learned_effects={} heldout_pickups={} heldout_door_openings={} heldout_moves={}"
          .format(train_tasks,len(train),eval_tasks,len(heldout),
                  random_tasks,len(heldout),report["trained_effects"],
                  report["eval_key_pickups"],report["eval_door_openings"],
                  report["eval_executed_moves"]),flush=True)
    print(f"UNSTAGED_EXTENDED_CURRICULUM include_empty={args.include_empty} "
          f"budget={args.budget} worlds={len(worlds)} "
          f"train={train_tasks}/{len(train)} heldout={eval_tasks}/{len(heldout)} "
          f"per_world={report['heldout_successes_by_world']}",flush=True)
    print(f"GENERAL_POLICY_MODE enabled={args.general_policy} "
          f"reward_events={report['general_training_rewards']} "
          f"updates={report['general_training_updates']}",flush=True)
    print(f"UNSTAGED_MOTION_MODE embodied={args.embodied} native_inferred_motion="
          f"{report['native_inferred_motion_events']} physical_actions="
          f"{report['total_physical_actions']}",flush=True)
    print(f"UNSTAGED_WORLD1_SUMMARY verdict={report['verdict']} "
          "claim=UNSTAGED_EXTERNAL_TASK_SUCCESS_ONLY_IF_REWARDED",
          flush=True)

if __name__=="__main__":
    main()
