#!/usr/bin/env python3
"""New open development evaluation; never replays a historical authority pack.

Only public partial images, actual rewards and an episode boundary enter Rust.
The evaluator alone owns counterfactual cue swaps and ground-truth scoring.
"""
import argparse
import gzip
import hashlib
import json
import subprocess
from pathlib import Path

import external_relational_hypothesis_pairs as paired
from external_minigrid_benchmark import Agent


class PublicAgent(Agent):
    def write(self, message):
        command = message.get("cmd")
        allowed = {
            "init": {"cmd", "dimension", "motors"},
            "reset": {"cmd", "observation", "learning"},
            "post": {"cmd", "observation", "reward"},
        }
        commands = {"advance", "restart", "status", "quit", "general_policy",
                    "developmental_memory", "relational_workspace", "relation_status",
                    "episodic_recall", "sequence_replay", "context_value_learning"}
        expected = allowed.get(command, {"cmd"} if command in commands else set())
        if set(message) != expected:
            raise ValueError(f"Unexpected controller fields: {sorted(message)}")
        super().write(message)


def save(path, report):
    data = (json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n").encode()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(gzip.compress(data, mtime=0) if path.suffix == ".gz" else data)


def arm(binary, name, train_seeds, test_seeds, transfer_seeds):
    agent = PublicAgent(binary)
    try:
        commands = ["general_policy"]
        if name != "value_no_memory":
            commands.extend(["developmental_memory", "relational_workspace"])
        if name == "legacy_memory":
            commands.extend(["episodic_recall", "sequence_replay"])
        else:
            commands.append("context_value_learning")
        for command in commands:
            paired.checked_command(agent, command, command + "_ack")
        agent.relational_mode = name != "value_no_memory"
        train = []
        for seed in train_seeds:
            train.append(paired.episode(agent, seed, True))
            if len(train) % 64 == 0:
                print(json.dumps({"arm": name, "trained": len(train),
                                  "successes": sum(e["success"] for e in train)}), flush=True)
        before = agent.status()
        evaluations = {}
        for world, seeds in (("MiniGrid-MemoryS7-v0", test_seeds),
                             ("MiniGrid-MemoryS9-v0", transfer_seeds)):
            paired.WORLD = world
            pairs = []
            for seed in seeds:
                original = paired.episode(agent, seed, False, restart=True)
                swapped = paired.episode(agent, seed, False, cue_swapped=True, restart=True)
                pairs.append({"seed": seed, "original": original, "cue_swapped": swapped})
            evaluations[world] = {"summary": paired.score_pairs(pairs), "pairs": pairs}
            print(json.dumps({"arm": name, "world": world,
                              **evaluations[world]["summary"]}), flush=True)
        after = agent.status()
        for field in ("general_updates", "general_rewards", "value_states",
                      "rewarded_episodic_memories", "failed_episodic_experiences"):
            if before[field] != after[field]:
                raise ValueError(f"Heldout evaluation modified learned {field}")
        return {"commands": commands, "train": train, "evaluations": evaluations,
                "before": before, "after": after}
    finally:
        paired.WORLD = "MiniGrid-MemoryS7-v0"
        agent.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--arm", choices=("value_memory", "value_no_memory", "legacy_memory"))
    args = parser.parse_args()
    dirty = subprocess.check_output(
        ["git", "status", "--porcelain", "--", "src", "Cargo.toml", "Cargo.lock"], text=True)
    if dirty:
        raise SystemExit("Commit the cognitive source before running the declared heldout evaluation")
    report = {
        "protocol": "CONTEXT-VALUE-1", "status": "open development, not independent qualification",
        "source": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "binary_sha256": hashlib.sha256(args.agent.read_bytes()).hexdigest(),
        "train_seeds": list(range(181000, 181512)),
        "test_seeds": list(range(182000, 182032)),
        "transfer_seeds": list(range(183000, 183016)),
        "budget": 200, "cue_metadata_sent_to_agent": False, "arms": {},
    }
    names = [args.arm] if args.arm else ["value_memory", "value_no_memory", "legacy_memory"]
    for name in names:
        report["arms"][name] = arm(args.agent, name, report["train_seeds"],
                                  report["test_seeds"], report["transfer_seeds"])
        save(args.output, report)
    if len(report["arms"]) == 3:
        scores = {name: data["evaluations"]["MiniGrid-MemoryS7-v0"]["summary"]
                  for name, data in report["arms"].items()}
        learned = scores["value_memory"]
        best = max(scores[n]["both_correct"] for n in ("value_no_memory", "legacy_memory"))
        report["verdict"] = ("CAUSAL_MEMORY_DEVELOPMENT_PASS"
                             if learned["cue_use_supported"] and learned["both_correct"] >= best + 8
                             else "CAUSAL_MEMORY_DEVELOPMENT_FAIL")
        save(args.output, report)
        print(report["verdict"], flush=True)


if __name__ == "__main__":
    main()
