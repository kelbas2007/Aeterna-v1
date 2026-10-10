#!/usr/bin/env python3
"""Run independent Farama MiniGrid environments against live EvoPhase via JSONL.

Only obs["image"] (the public *partial* MiniGrid symbolic tile-view) enters
EvoPhase. Neither mission strings, hidden grid/agent positions, action labels,
object IDs by name, nor any privileged reset/transition oracle is sent.
Conversion: lossless 4-bit binary encoding of each observed tile field.
This is NOT raw RGB visual object perception.
"""
import argparse
import json
import random
import subprocess
import sys
from pathlib import Path

import gymnasium as gym
import minigrid  # noqa: F401  # registers independently maintained environments
import numpy as np

ENVIRONMENTS = (
    "MiniGrid-Empty-5x5-v0",
    "MiniGrid-DoorKey-5x5-v0",
    "MiniGrid-MultiRoom-N2-S4-v0",
)
RAW_SHAPE = (7, 7, 3)
CHANNEL_BITS = 4
DIMENSION = 7 * 7 * 3 * CHANNEL_BITS


def sensory(observation):
    tile = np.asarray(observation["image"], dtype=np.uint8)
    if tile.shape != RAW_SHAPE or np.any(tile >= 16):
        raise RuntimeError(f"unexpected MiniGrid symbolic tile view: {tile.shape}")
    # Fixed, semantics-free binary serialization; no object classifier.
    return [(int(v) >> shift) & 1 for v in tile.flat for shift in range(CHANNEL_BITS)]


class Agent:
    def __init__(self, binary):
        self.proc = subprocess.Popen(
            [str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            text=True, bufsize=1,
        )
        self.write({"cmd": "init", "dimension": DIMENSION, "motors": 7})
        ready = self.read()
        if ready.get("type") != "ready":
            raise RuntimeError(f"init: {ready}")

    def write(self, message):
        self.proc.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
        self.proc.stdin.flush()

    def read(self):
        line = self.proc.stdout.readline()
        if not line:
            raise RuntimeError(f"native agent exited: {self.proc.poll()}")
        return json.loads(line)

    def reset(self, obs, learning):
        self.write({"cmd": "reset", "observation": sensory(obs), "learning": learning})
        reply = self.read()
        if reply.get("type") != "reset_ack":
            raise RuntimeError(f"reset failed: {reply}")

    def act_in(self, env):
        self.write({"cmd": "advance"})
        choice = self.read()
        if choice.get("type") == "stopped":
            return None, 0.0, False, False, choice
        if choice.get("type") != "action":
            raise RuntimeError(f"invalid protected action response: {choice}")
        action = choice["action"]
        if not isinstance(action, int) or not 0 <= action < env.action_space.n:
            raise RuntimeError(f"invalid motor from native agent: {choice}")
        obs, reward, terminated, truncated, _ = env.step(action)
        # The only environment messages fed back into the Rust organism:
        # observation bits, and actual bounded immediate reward.
        self.write({
            "cmd": "post", "observation": sensory(obs),
            "reward": float(max(0.0, min(1.0, reward))),
        })
        confirmation = self.read()
        if confirmation.get("type") != "advance_ack" or confirmation.get("executed") != action:
            raise RuntimeError(f"unconfirmed protected action: {confirmation}")
        return obs, float(reward), bool(terminated), bool(truncated), choice

    def status(self):
        self.write({"cmd": "status"})
        reply = self.read()
        if reply.get("type") != "status":
            raise RuntimeError(f"bad status: {reply}")
        return reply

    def close(self):
        if self.proc.poll() is None:
            self.write({"cmd": "quit"})
            self.proc.wait(timeout=15)


def run_organism(env_id, executable, train_seeds, eval_seeds, budget):
    env = gym.make(env_id)
    agent = Agent(executable)
    outcomes = {"train": [], "eval": []}
    try:
        for stage, seeds in (("train", train_seeds), ("eval", eval_seeds)):
            for seed in seeds:
                obs, _ = env.reset(seed=seed)
                agent.reset(obs, learning=(stage == "train"))
                total = 0.0
                actions = 0
                stopped = None
                for _ in range(budget):
                    obs, reward, terminated, truncated, response = agent.act_in(env)
                    if obs is None:
                        stopped = response.get("reason")
                        break
                    actions += 1
                    total += reward
                    if terminated or truncated:
                        break
                outcomes[stage].append({
                    "seed": seed, "success": total > 0,
                    "reward": round(total, 6), "steps": actions,
                    "stopped": stopped,
                })
        model = agent.status()
    finally:
        agent.close()
        env.close()
    return outcomes, model


def random_control(env_id, seeds, budget, seed_base):
    env = gym.make(env_id)
    results = []
    generator = random.Random(seed_base)
    try:
        for seed in seeds:
            env.reset(seed=seed)
            reward_sum = 0.0
            steps = 0
            for _ in range(budget):
                _, reward, terminated, truncated, _ = env.step(generator.randrange(7))
                reward_sum += float(reward)
                steps += 1
                if terminated or truncated:
                    break
            results.append({
                "seed": seed, "success": reward_sum > 0,
                "reward": round(reward_sum, 6), "steps": steps
            })
    finally:
        env.close()
    return results


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=Path("external-world1.json"))
    parser.add_argument("--budget", type=int, default=64)
    args = parser.parse_args()
    if not args.agent.exists():
        raise SystemExit(f"missing native agent executable: {args.agent}")
    if args.budget <= 0 or args.budget > 256:
        raise SystemExit("invalid action budget")
    version = getattr(minigrid, "__version__", "unknown")
    report = {
        "benchmark": "external-minigrid-1",
        "library": f"minigrid-{version}",
        "observation": "public partial 7x7x3 categorical tiles, four binary bits/field",
        "mission_visible_to_agent": False,
        "training_seeds": list(range(100, 108)),
        "heldout_seeds": list(range(900, 904)),
        "steps_budget": args.budget,
        "results": {},
    }
    for env_id in ENVIRONMENTS:
        organism, model = run_organism(
            env_id, args.agent, report["training_seeds"],
            report["heldout_seeds"], args.budget,
        )
        random_eval = random_control(
            env_id, report["heldout_seeds"], args.budget, 0xAE7E + len(env_id))
        trained = sum(r["success"] for r in organism["train"])
        held = sum(r["success"] for r in organism["eval"])
        baseline = sum(r["success"] for r in random_eval)
        report["results"][env_id] = {
            "training": organism["train"],
            "heldout": organism["eval"],
            "random_heldout": random_eval,
            "train_successes": trained,
            "heldout_successes": held,
            "random_heldout_successes": baseline,
            "carrier": model,
        }
        print(
            f"EXTERNAL_WORLD env={env_id} trained={trained}/8 "
            f"frozen_heldout={held}/4 random={baseline}/4 "
            f"rules={model['rules']} positive_rewards={model['rewarded_examples']} "
            f"contradictions={model['contradictions']}", flush=True
        )
    report["all_heldout_successes"] = sum(
        r["heldout_successes"] for r in report["results"].values())
    report["random_heldout_successes"] = sum(
        r["random_heldout_successes"] for r in report["results"].values())
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(
        "EXTERNAL_WORLD_SUMMARY frozen_heldout="
        f"{report['all_heldout_successes']}/12 "
        f"random={report['random_heldout_successes']}/12 "
        "verdict=MEASURED_NO_SCIENTIFIC_PASS",
        flush=True
    )


if __name__ == "__main__":
    main()
