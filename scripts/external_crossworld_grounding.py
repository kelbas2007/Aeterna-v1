#!/usr/bin/env python3
"""One *persistent EvoPhase organism*, two independent Farama world families.

Grounding is an EXPLICIT DEICTIC TEACHER signal (visible tile index + word).
Teacher uses public observed MiniGrid categorical type only to choose what
to point at. This metadata is NOT transmitted as type IDs or action hints to
the agent, only a word and a tile index. Evaluation has no word lessons.
A correct result establishes narrow visual-symbol reference transfer, not
self-discovered natural language and not autonomous task completion.
"""
import argparse
import json
from pathlib import Path

import gymnasium as gym
import minigrid  # noqa
from minigrid.core.constants import OBJECT_TO_IDX
import numpy as np

from external_minigrid_benchmark import Agent

WORDS = ("key", "door")
TRAIN_SEEDS = list(range(4100, 4112))
TEST_SEEDS = list(range(5100, 5108))
BUDGET = 64

def targets(obs, word):
    view = np.asarray(obs["image"], dtype=np.uint8)
    return [i for i, t in enumerate(view.reshape(-1, 3))
            if int(t[0]) == OBJECT_TO_IDX[word]]

def teach(agent, word, index):
    agent.write({"cmd": "teach_word", "word": word, "tile": index})
    r = agent.read()
    if r.get("type") != "lesson_ack":
        raise RuntimeError(f"teaching handshake failed: {r}")
    return bool(r["learned"])

def locate(agent, word):
    agent.write({"cmd": "locate_word", "word": word})
    r = agent.read()
    if r.get("type") != "referents":
        raise RuntimeError(f"word readout failed: {r}")
    return list(map(int, r["tiles"]))

def restart(agent):
    agent.write({"cmd": "restart"})
    r = agent.read()
    if r.get("type") != "restart_ack":
        raise RuntimeError(f"cognitive restart failed: {r}")
    return r

def score_current(agent, env_name, obs, counter):
    for word in WORDS:
        truth = set(targets(obs, word))
        predicted = set(locate(agent, word))
        counter["queries"] += 1
        counter["visible"] += len(truth)
        counter["correct"] += len(truth.intersection(predicted))
        counter["false_positive"] += len(predicted.difference(truth))
        if predicted and not truth:
            counter["hallucinated_frames"] += 1
        if env_name.startswith("MiniGrid-MultiRoom") and word == "door":
            counter["crossworld_visible"] += len(truth)
            counter["crossworld_correct"] += len(truth.intersection(predicted))

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--agent", type=Path, required=True)
    p.add_argument("--output", type=Path, default=Path("external-object-lexicon.json"))
    args = p.parse_args()
    agent = Agent(args.agent)
    teaching = {w: 0 for w in WORDS}
    actions = 0
    training_successes = 0
    measurements = {
        "queries": 0, "visible": 0, "correct": 0,
        "false_positive": 0, "hallucinated_frames": 0,
        "crossworld_visible": 0, "crossworld_correct": 0,
    }
    first_status = agent.status()
    try:
        # World A: visually witnessed key and door, named by a tutor.
        # Every movement is an unmodified protected native proposal.
        env = gym.make("MiniGrid-DoorKey-5x5-v0")
        try:
            for seed in TRAIN_SEEDS:
                obs, _ = env.reset(seed=seed)
                agent.reset(obs, learning=True)
                for _ in range(BUDGET):
                    for word in WORDS:
                        for tile in targets(obs, word)[:1]:
                            teaching[word] += int(teach(agent, word, tile))
                    post, reward, terminated, truncated, event = agent.act_in(env)
                    if post is None:
                        raise RuntimeError(f"native action unavailable: {event}")
                    actions += 1
                    obs = post
                    if reward > 0:
                        training_successes += 1
                    if terminated or truncated:
                        break
        finally:
            env.close()
        after_training = agent.status()
        replay = restart(agent)
        if replay["word_count"] != after_training["word_count"]:
            raise RuntimeError("checkpoint erased grounded words")

        # World B: never point/names. Heldout MiniGrid DoorKey with new
        # starts and an independently defined MultiRoom environment.
        # One and the same Rust process, same memory and motor history.
        worlds = (
            "MiniGrid-DoorKey-5x5-v0",
            "MiniGrid-MultiRoom-N2-S4-v0",
            "MiniGrid-Empty-5x5-v0",
        )
        task_scores = {}
        for name in worlds:
            env = gym.make(name)
            executed = 0
            success = 0
            try:
                for seed in TEST_SEEDS:
                    obs, _ = env.reset(seed=seed)
                    agent.reset(obs, learning=False)
                    score_current(agent, name, obs, measurements)
                    for _ in range(BUDGET):
                        post, reward, terminated, truncated, event = agent.act_in(env)
                        if post is None:
                            raise RuntimeError(f"frozen motor unavailable: {event}")
                        executed += 1
                        actions += 1
                        obs = post
                        score_current(agent, name, obs, measurements)
                        success += int(reward > 0)
                        if terminated or truncated:
                            break
            finally:
                env.close()
            task_scores[name] = {"actions": executed, "positive_reward_steps": success}
        after = agent.status()
    finally:
        agent.close()
    report = {
        "task": "cross-world externally supervised object-word binding",
        "teacher": "deictic word+visible tile; category ID used by teacher ONLY",
        "world_names_sent_to_agent": False,
        "mission_sent_to_agent": False,
        "training_seeds": TRAIN_SEEDS,
        "holdout_seeds": TEST_SEEDS,
        "teacher_acceptances": teaching,
        "teaching_world": "MiniGrid-DoorKey-5x5-v0",
        "first_status": first_status,
        "after_training": after_training,
        "after_restart": replay,
        "after": after,
        "train_positive_rewards": training_successes,
        "protected_actions": actions,
        "holdout": task_scores,
        "word_reference": measurements,
    }
    verdict = (
        all(n > 0 for n in teaching.values())
        and after["word_count"] == 2
        and measurements["crossworld_visible"] > 0
        and measurements["crossworld_correct"] > 0
        and measurements["false_positive"] == 0
        and measurements["correct"] == measurements["visible"]
    )
    report["grounding_transfer_diagnostic"] = "DEVELOPMENT_PASS" if verdict else "DEVELOPMENT_FAIL"
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"CROSSWORLD_OBJECT teacher={teaching} words={after['word_count']} "
          f"visual_classes={after['visual_categories']} frames={after['visual_frames']} "
          f"actions={actions} visible={measurements['visible']} "
          f"correct={measurements['correct']} false_positive={measurements['false_positive']} "
          f"crossworld_door={measurements['crossworld_correct']}/{measurements['crossworld_visible']}",
          flush=True)
    print(f"CROSSWORLD_OBJECT_SUMMARY verdict={report['grounding_transfer_diagnostic']} "
          f"task_rewards={sum(v['positive_reward_steps'] for v in task_scores.values())} "
          "claim=SUPERVISED_CATEGORICAL_REFERENCE_NOT_AGI", flush=True)

if __name__ == "__main__":
    main()
