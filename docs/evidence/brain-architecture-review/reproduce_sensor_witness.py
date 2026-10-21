#!/usr/bin/env python3
"""Reproduce encoder collisions through one existing runtime, with no actions.

Usage: python reproduce_sensor_witness.py /path/to/aeterna-v1

The input frames are not a performance benchmark. The frozen original
binary/source identities are recorded in sensor-information-witness.json.
A later implementation can legitimately remove these collisions.
"""

import hashlib
import json
import subprocess
import sys
from pathlib import Path


def main():
    binary = Path(sys.argv[1]).resolve()
    original = json.loads(
        Path(__file__).with_name("sensor-information-witness.json").read_text()
    )
    frames = [row["frame"] for row in original["records"]]
    requests = [{"op": "organism_init", "frame": frames[0]}]
    requests.append({"op": "checkpoint"})
    for frame in frames[1:]:
        requests.extend(
            [{"op": "external_reset", "frame": frame}, {"op": "checkpoint"}]
        )
    requests.append({"op": "quit"})
    completed = subprocess.run(
        [str(binary), "--jsonl"],
        input="".join(json.dumps(value) + "\n" for value in requests),
        text=True,
        capture_output=True,
        timeout=30,
        check=True,
    )
    replies = [json.loads(line) for line in completed.stdout.splitlines()]
    if any(reply.get("type") == "error" for reply in replies):
        raise RuntimeError("Native runtime rejected a request")
    checkpoints = [
        json.loads(reply["checkpoint"])
        for reply in replies
        if reply.get("type") == "checkpoint"
    ]
    if len(checkpoints) != 4:
        raise RuntimeError("Expected exactly four native checkpoints")
    learners = [state["cognition"]["current"]["learner"] for state in checkpoints]
    if any(learner["next_id"] != 0 for learner in learners):
        raise RuntimeError("A factual action unexpectedly occurred")
    vectors = [learner["current"] for learner in learners]
    expected = [row["encoded"] for row in original["records"]]
    result = {
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "original_frozen_binary": hashlib.sha256(binary.read_bytes()).hexdigest()
        == original["binary_sha256"],
        "external_actions": 0,
        "one_runtime": True,
        "exact_original_vectors": vectors == expected,
        "image_collision": vectors[0] == vectors[1],
        "signal_reversal_collision": vectors[2] == vectors[3],
        "vectors": vectors,
    }
    print(json.dumps(result, indent=2))
    if not result["image_collision"] or not result["signal_reversal_collision"]:
        raise SystemExit("This binary no longer exhibits the documented collisions")
    if result["original_frozen_binary"] and not result["exact_original_vectors"]:
        raise SystemExit("Frozen binary did not reproduce its original vectors")


if __name__ == "__main__":
    main()
