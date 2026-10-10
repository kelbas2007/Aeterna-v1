#!/usr/bin/env python3
"""Zero motor demonstrations: acquire external MiniGrid object manipulation.

The world/examiner stages an *opportunity* (object in front) but never says a
word or supplies the correct motor in training. All motor experiments go
through Rust U1 -> Human Protection -> actual external Farama step. The
evaluator alone knows object names/type to arrange a controlled opportunity.
This is NOT unaided navigation or language acquisition.
"""
import argparse
import json
from pathlib import Path
import numpy as np
import gymnasium as gym
import minigrid  # noqa: F401
from minigrid.core.world_object import Key
from external_functional_grounding import staged_observation, FRONT, send
from external_minigrid_benchmark import Agent

TRAIN = (("key", "MiniGrid-DoorKey-5x5-v0"),
         ("door", "MiniGrid-MultiRoom-N2-S4-v0"))
TEST = (("key", "MiniGrid-Unlock-v0"),
        ("door", "MiniGrid-DoorKey-5x5-v0"))
TRAIN_SEEDS = list(range(42000,42010))
TEST_SEEDS = list(range(43000,43012))

def episode(agent, object_name, world_id, seed, learning):
    env = gym.make(world_id)
    try:
        env.reset(seed=seed)
        original = staged_observation(env,object_name)
        preloaded = object_name == "door" and world_id == "MiniGrid-DoorKey-5x5-v0"
        if preloaded:
            # Only independent evaluator supplies key as a precondition.
            # Native agent is given no carrying label/condition flag.
            env.unwrapped.carrying = Key("yellow")
            original = env.unwrapped.gen_obs()
        agent.reset(original,learning=learning)
        previous_category = int(np.asarray(original["image"])[3,5,0])
        if previous_category == 0:
            raise RuntimeError("no observable object in front")
        # Evaluator-only physical witness. Never transmitted to the agent.
        target = env.unwrapped.grid.get(*env.unwrapped.front_pos)
        was_open = getattr(target, "is_open", False)
        prior_carrying = env.unwrapped.carrying
        observed, reward, done, truncated, response = agent.act_in(env)
        action = response.get("action") if observed is not None else None
        changed = observed is not None and bool(np.any(
            np.asarray(original["image"])[3,5,:] !=
            np.asarray(observed["image"])[3,5,:]
        ))
        # A changed tile is an actual simulator measurement. The agent does
        # not learn any action semantics from evaluator scoring.
        truly_picked_up = (object_name == "key"
            and prior_carrying is None and target is not None
            and env.unwrapped.carrying is target)
        truly_opened = (object_name == "door" and target is not None
            and not was_open and getattr(target, "is_open", False))
        physical_effect = bool(truly_picked_up or truly_opened)
        status=agent.status()
        return {
            "seed":seed, "world":world_id, "staged":True,
            "correct_label_hidden_from_agent":True,
            "action":action, "changed":changed,
            "physical_effect":physical_effect,
            "reward":float(reward), "native_effects":status["self_affordances"],
            "native_trials":status["self_experiments"], "preloaded_key":preloaded,
            "stopped":response.get("reason"),
        }
    finally:
        env.close()

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--agent",type=Path,required=True)
    parser.add_argument("--output",type=Path,default=Path("external-self-affordance1.json"))
    parser.add_argument("--seed-offset",type=int,default=0)
    parser.add_argument("--strict-physical",action="store_true")
    args=parser.parse_args()
    train_seeds=[seed+args.seed_offset for seed in TRAIN_SEEDS]
    test_seeds=[seed+args.seed_offset for seed in TEST_SEEDS]
    agent=Agent(args.agent)
    try:
        setup=send(agent,{"cmd":"self_experiment","front_tile":FRONT})
        if setup.get("type")!="self_experiment_ack" or not setup["accepted"]:
            raise RuntimeError(f"self experiment interface denied: {setup}")
        training={}
        for object_name,env_id in TRAIN:
            attempts=[]
            for seed in train_seeds:
                result=episode(agent,object_name,env_id,seed,learning=True)
                attempts.append(result)
                if result["physical_effect"] and result["native_effects"] >= len(training)+1:
                    break
            training[object_name]=attempts
        snapshot=agent.status()
        checkpoint=send(agent,{"cmd":"restart"})
        if checkpoint.get("type")!="restart_ack":
            raise RuntimeError(f"carrier restart failed: {checkpoint}")
        heldout={}
        for object_name,env_id in TEST:
            heldout[object_name]=[
                episode(agent,object_name,env_id,seed,learning=False)
                for seed in test_seeds
            ]
        frozen=agent.status()
        successes=sum(int(entry["physical_effect"] if args.strict_physical
                             else entry["changed"])
                      for batch in heldout.values() for entry in batch)
        blocked=sum(entry["stopped"] is not None
                    for batch in heldout.values() for entry in batch)
        report={
            "test":"autonomous opaque motor experiments on real outside MiniGrid",
            "training_motor_tuition":0,
            "training_object_staging":True,
            "heldout_object_staging":True,
            "heldout_door_key_preloaded_by_examiner":True,
            "training_seeds":train_seeds,
            "heldout_seeds":test_seeds,
            "strict_physical":args.strict_physical,
            "training":training,"heldout":heldout,
            "before_restart":snapshot,"after_restart":checkpoint,"frozen":frozen,
            "heldout_interaction_successes":successes,
            "physical_heldout_successes":sum(int(e["physical_effect"]) for v in heldout.values() for e in v),
            "heldout_interaction_total":24,
            "blocked_or_unsupported":blocked,
            "terminal_goal_rewards":sum(entry["reward"]>0
                for batch in heldout.values() for entry in batch),
        }
        success=(snapshot["self_affordances"]>=2
            and frozen["self_affordances"]==snapshot["self_affordances"]
            and successes==24 and blocked==0
            and all(any(t["changed"] for t in trials)
                    for trials in training.values()))
        report["verdict"]="DEVELOPMENT_PASS" if success else "DEVELOPMENT_FAIL"
        args.output.write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
        print(f"SELF_AFFORDANCE1 train_actions={sum(len(x) for x in training.values())} "
              f"motor_tuition=0 learned={snapshot['self_affordances']} "
              f"frozen_interactions={successes}/24 "
              f"blocked={blocked} goal_rewards={report['terminal_goal_rewards']}",
              flush=True)
        print(f"SELF_AFFORDANCE1_SUMMARY verdict={report['verdict']} "
              "claim=AUTONOMOUS_ACTION_DISCOVERY_WITH_STAGED_OBJECTS_NOT_AUTONOMOUS_NAVIGATION",
              flush=True)
    finally:
        agent.close()

if __name__=="__main__":
    main()
