#!/usr/bin/env python3
"""Audit what the native agent could know, before attributing a memory failure.

The model is unchanged. Only the evaluator can swap the initial cue and audit
its visibility. Every observation delivered to Rust is a real public MiniGrid
image. No cue coordinates, cue labels, branch truth, direction or mission are
sent to the controller. Paired tests are balanced by construction.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path
from typing import Any

WORLD = "MiniGrid-MemoryS7-v0"
TRAIN_SEEDS = list(range(160000, 160128))
TEST_SEEDS = list(range(161000, 161032))
BUDGET = 200
CONTROLLER_SOURCE = "RELATIONAL_WORKSPACE_DEVELOPMENT_OPEN_NOT_SOURCE_FROZEN"


def score_pairs(pairs: list[dict[str, Any]]) -> dict[str, Any]:
    if not pairs:
        raise ValueError("No paired episodes were scored")
    counts: Counter[str] = Counter()
    for pair in pairs:
        a, b = pair["original"], pair["cue_swapped"]
        if a["success_position"] != b["failure_position"] or \
                b["success_position"] != a["failure_position"]:
            raise ValueError("Counterfactual cue did not exchange task outcomes")
        exposed = bool(a["cue_visible_steps"] or b["cue_visible_steps"])
        identical = a["input_sha256"] == b["input_sha256"]
        same_actions = a["actions"] == b["actions"]
        if identical and not same_actions:
            raise ValueError("Identical input histories produced different outputs: uncontrolled state")
        counts["pairs"] += 1
        counts["successes"] += int(a["success"]) + int(b["success"])
        counts["both_correct"] += int(a["success"] and b["success"])
        counts["exposed_pairs"] += int(exposed)
        counts["unexposed_pairs"] += int(not exposed)
        counts["identical_input_pairs"] += int(identical)
        counts["exit_switch_pairs"] += int(a["terminal_position"] != b["terminal_position"])
        counts["same_actions_pairs"] += int(same_actions)
        counts["exposed_both_correct"] += int(exposed and a["success"] and b["success"])
        counts["no_cue_both_correct"] += int(not exposed and a["success"] and b["success"])
        if not exposed and not identical:
            raise ValueError("Unseen cue affected the input: audit/information leak")
    required = ("pairs", "successes", "both_correct", "exposed_pairs", "unexposed_pairs",
                "identical_input_pairs", "exit_switch_pairs", "same_actions_pairs",
                "exposed_both_correct", "no_cue_both_correct")
    result = {key: counts[key] for key in required}
    result["episodes"] = 2 * len(pairs)
    result["constant_exit_upper_bound"] = len(pairs)
    result["all_pairs_correct"] = counts["both_correct"] == len(pairs)
    result["beats_constant_exit"] = counts["successes"] > len(pairs)
    result["cue_use_supported"] = bool(
        counts["successes"] >= 48 and counts["both_correct"] >= 16
        and counts["exposed_both_correct"] >= 16
        and counts["no_cue_both_correct"] == 0
    )
    return result


def public_hash(obs: dict[str, Any]) -> str:
    import numpy as np
    return hashlib.sha256(np.asarray(obs["image"], dtype=np.uint8).tobytes()).hexdigest()


def replace_cue(base: Any, *, exchange_outcomes: bool) -> Any:
    """Evaluator-only intervention: alter one cue, leaving physics/layout intact."""
    from minigrid.core.world_object import Ball, Key
    x, y = 1, base.height // 2 - 1
    old = base.grid.get(x, y)
    if not isinstance(old, (Ball, Key)):
        raise ValueError("Pinned MemoryS7 layout contract changed: cue is not a key/ball")
    replacement = Ball(old.color) if isinstance(old, Key) else Key(old.color)
    replacement.init_pos = old.init_pos
    replacement.cur_pos = old.cur_pos
    base.grid.set(x, y, replacement)
    if exchange_outcomes:
        base.success_pos, base.failure_pos = base.failure_pos, base.success_pos
    return old


def cue_affects_public_input(env: Any, obs: dict[str, Any]) -> bool:
    """Exact intervention test, not merely 'same type is somewhere in the view'."""
    import numpy as np
    base = env.unwrapped
    actual = base.gen_obs()
    if not np.array_equal(actual["image"], obs["image"]):
        raise ValueError("Audited frame differs from the frame supplied by env.step/reset")
    original = replace_cue(base, exchange_outcomes=False)
    try:
        altered = base.gen_obs()
        return bool(np.any(np.asarray(actual["image"]) != np.asarray(altered["image"])))
    finally:
        base.grid.set(1, base.height // 2 - 1, original)
        if not np.array_equal(base.gen_obs()["image"], obs["image"]):
            raise ValueError("Visibility intervention did not restore the environment")


def checked_command(agent: Any, command: str, expected: str) -> dict[str, Any]:
    agent.write({"cmd": command})
    reply = agent.read()
    if reply.get("type") != expected:
        raise RuntimeError(f"{command} failed: {reply}")
    if "accepted" in reply and not reply["accepted"]:
        raise RuntimeError(f"{command} was refused")
    return reply


def create_agent(binary: Path) -> Any:
    from external_minigrid_benchmark import Agent

    class AuditedAgent(Agent):
        def write(self, message: dict[str, Any]) -> None:
            keys = {
                "init": {"cmd", "dimension", "motors"},
                "reset": {"cmd", "observation", "learning"},
                "advance": {"cmd"}, "post": {"cmd", "observation", "reward"},
                "restart": {"cmd"}, "status": {"cmd"}, "quit": {"cmd"},
                "general_policy": {"cmd"}, "developmental_memory": {"cmd"},
                "episodic_recall": {"cmd"}, "sequence_replay": {"cmd"},
                "relational_workspace": {"cmd"}, "relation_status": {"cmd"},
            }
            if set(message) != keys.get(message.get("cmd"), set()):
                raise ValueError(f"Non-public or unsupported controller input: {sorted(message)}")
            super().write(message)

    return AuditedAgent(binary)


def episode(agent: Any, seed: int, learning: bool, *, cue_swapped: bool = False,
            restart: bool = False) -> dict[str, Any]:
    import gymnasium as gym
    import minigrid  # noqa: F401
    env = gym.make(WORLD)
    try:
        obs, _ = env.reset(seed=seed)
        if cue_swapped:
            replace_cue(env.unwrapped, exchange_outcomes=True)
            obs = env.unwrapped.gen_obs()
        if restart:
            checked_command(agent, "restart", "restart_ack")
        agent.reset(obs, learning=learning)
        base = env.unwrapped
        report: dict[str, Any] = {
            "seed": seed, "cue_swapped": cue_swapped, "learning": learning,
            "initial_position": [int(v) for v in base.agent_pos],
            "success_position": [int(v) for v in base.success_pos],
            "failure_position": [int(v) for v in base.failure_pos],
            "actions": [], "input_sha256": [], "cue_visible_steps": [],
            "relation_bound_steps": [], "relation_match_steps": [],
            "relation_feature_hashes": [],
            "reward": 0.0, "stop": None,
        }
        for step in range(BUDGET):
            if cue_affects_public_input(env, obs):
                report["cue_visible_steps"].append(step)
            report["input_sha256"].append(public_hash(obs))
            if getattr(agent,"relational_mode",False):
                pair=checked_command(agent,"relation_status","relation_status")
                if pair.get("bound"):
                    report["relation_bound_steps"].append(step)
                features=pair.get("features") or [0.0]*96
                report["relation_feature_hashes"].append(
                    hashlib.sha256(json.dumps(features).encode()).hexdigest())
                if features[1]>0.0:
                    report["relation_match_steps"].append(step)
            obs, reward, terminated, truncated, response = agent.act_in(env)
            if obs is None:
                report["stop"] = response.get("reason", "unknown_stop")
                break
            report["actions"].append(response["action"])
            report["reward"] += float(reward)
            if terminated or truncated:
                break
        report["success"] = report["reward"] > 0
        report["terminal_position"] = [int(v) for v in base.agent_pos]
        return report
    finally:
        env.close()


def audit(binary: Path, memory: bool, relational:bool) -> dict[str, Any]:
    agent = create_agent(binary)
    try:
        checked_command(agent, "general_policy", "general_policy_ack")
        if memory:
            checked_command(agent, "developmental_memory", "developmental_memory_ack")
            checked_command(agent, "episodic_recall", "episodic_recall_ack")
            checked_command(agent, "sequence_replay", "sequence_replay_ack")
        if relational:
            checked_command(agent,"relational_workspace","relational_workspace_ack")
        agent.relational_mode=relational
        train = [episode(agent, seed, True) for seed in TRAIN_SEEDS]
        before = agent.status()
        pairs = []
        for seed in TEST_SEEDS:
            a = episode(agent, seed, False, restart=True)
            b = episode(agent, seed, False, cue_swapped=True, restart=True)
            pairs.append({"seed": seed, "original": a, "cue_swapped": b})
        after = agent.status()
        for key in ("general_updates", "general_rewards", "rewarded_episodic_memories",
                    "failed_episodic_experiences"):
            if before.get(key) != after.get(key):
                raise ValueError(f"Frozen learned state changed in {key}")
        summary = score_pairs(pairs)
        summary["training_successes"] = sum(e["success"] for e in train)
        summary["training_cue_exposed_episodes"] = sum(bool(e["cue_visible_steps"]) for e in train)
        summary["training_episodes"] = len(train)
        summary["training_cue_visible_at_reset"] = sum(0 in e["cue_visible_steps"] for e in train)
        summary["relational_bound_pairs"] = sum(
            bool(p["original"]["relation_bound_steps"] or
                 p["cue_swapped"]["relation_bound_steps"])
            for p in pairs)
        summary["relational_match_pairs"] = sum(
            bool(p["original"]["relation_match_steps"] or
                 p["cue_swapped"]["relation_match_steps"])
            for p in pairs)
        summary["relational_difference_pairs"] = sum(
            p["original"]["relation_feature_hashes"] !=
            p["cue_swapped"]["relation_feature_hashes"]
            for p in pairs)
        summary["relational_exposed_difference_pairs"] = sum(
            bool(p["original"]["cue_visible_steps"] or
                 p["cue_swapped"]["cue_visible_steps"])
            and p["original"]["relation_feature_hashes"] !=
                p["cue_swapped"]["relation_feature_hashes"]
            for p in pairs)
        summary["physical_heldout_actions"] = sum(len(e["actions"]) for p in pairs
                                                  for e in (p["original"], p["cue_swapped"]))
        if summary["physical_heldout_actions"] == 0:
            raise ValueError("No heldout external actions executed")
        return {"memory": memory, "relational":relational,
                "summary": summary, "train": train, "pairs": pairs,
                "carrier_before": before, "carrier_after": after}
    finally:
        agent.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = {"controller_source": CONTROLLER_SOURCE, "world": WORLD,
              "minigrid": "3.1.0", "train_seeds": TRAIN_SEEDS, "test_seeds": TEST_SEEDS,
              "budget": BUDGET, "balanced_counterfactual_pairs": True,
              "cue_metadata_sent_to_agent": False, "native_controller_modified": False,
              "arms": {}}
    for name, memory, relational in (
        ("autobiographical_no_relation",True,False),
        ("autobiographical_with_relation",True,True)):
        result["arms"][name] = audit(args.agent, memory, relational)
        print("CUE_ACCESS_AUDIT " + json.dumps({"arm": name,
              **result["arms"][name]["summary"]}, sort_keys=True), flush=True)
        args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    active = result["arms"]["autobiographical_with_relation"]["summary"]
    null = result["arms"]["autobiographical_no_relation"]["summary"]
    strong = bool(active["successes"]>32
        and active["both_correct"]>=4
        and active["exposed_both_correct"]>=4
        and active["no_cue_both_correct"]==0
        and active["both_correct"]>null["both_correct"]
        and active["relational_exposed_difference_pairs"]>=4
        and active["exit_switch_pairs"]>null["exit_switch_pairs"])
    result["verdict"] = "RELATIONAL_CAUSAL_USE_SUPPORTED" if strong else "RELATIONAL_USE_NOT_DEMONSTRATED"
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print("RELATION_PAIRS_RESULT " + result["verdict"]
          + f" relation={active['successes']}/64"
          + f" control={null['successes']}/64"
          + f" correct_pairs={active['both_correct']}/32"
          + f" changed_exits={active['exit_switch_pairs']}/32"
          + f" exposed_pairs={active['exposed_pairs']}/32"
          + f" bound_pairs={active['relational_bound_pairs']}/32"
          + f" matching_pairs={active['relational_match_pairs']}/32"
          + f" relation_difference={active['relational_difference_pairs']}/32"
          + f" exposed_relation_difference={active['relational_exposed_difference_pairs']}/32",flush=True)


if __name__ == "__main__":
    main()
