#!/usr/bin/env python3
"""Attach autonomous inquiry to an existing organism without executing actions.

The original checkpoint is read only. A separate output is created exclusively
after preservation checks and an exact checkpoint reload in a fresh process.
This verifies state preservation, not behavioral ability or general intelligence.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


MAX_CHECKPOINT_BYTES = 32 * 1024 * 1024
MAX_PROTOCOL_BYTES = 72 * 1024 * 1024
PROCESS_TIMEOUT_SECONDS = 60


def require(condition, message):
    if not condition:
        raise ValueError(message)


def reject_constant(value):
    raise ValueError(f"Non-finite JSON constant is not a valid checkpoint: {value}")


def decode_json(text):
    return json.loads(text, parse_constant=reject_constant)


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def read_checkpoint(path):
    require(path.is_file(), "Input checkpoint must be a regular file")
    with path.open("rb") as stream:
        data = stream.read(MAX_CHECKPOINT_BYTES + 1)
    require(data and len(data) <= MAX_CHECKPOINT_BYTES,
            "Input checkpoint is empty or exceeds the 32 MiB runtime limit")
    text = data.decode("utf-8")
    value = decode_json(text)
    require(isinstance(value, dict) and isinstance(value.get("cognition"), dict)
            and "carrier" in value, "Input must be a complete constructing-runtime checkpoint")
    return data, text, value


def run_protocol(binary, commands, expected_types):
    """Run only a finite, predetermined protocol; never supply step or POST."""
    allowed = {"load", "causal_report", "inquiry_report", "report",
               "enable_autonomous_inquiry", "checkpoint", "quit"}
    require(all(message.get("op") in allowed for message in commands),
            "Action-producing commands are forbidden in this upgrade utility")
    payload = b"".join(
        json.dumps(message, ensure_ascii=False, allow_nan=False,
                   separators=(",", ":")).encode("utf-8") + b"\n"
        for message in commands
    )
    # Temporary files prevent a malfunctioning binary's diagnostics from
    # growing an unbounded in-memory capture. subprocess.run kills and waits
    # for the child on timeout; no shell or platform-specific pipe selector.
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        try:
            process = subprocess.run(
                [str(binary), "--jsonl"], input=payload, stdout=stdout,
                stderr=stderr, timeout=PROCESS_TIMEOUT_SECONDS, check=False,
            )
        except subprocess.TimeoutExpired as exc:
            raise TimeoutError(
                f"Runtime did not complete the read/enable/checkpoint protocol within "
                f"{PROCESS_TIMEOUT_SECONDS} seconds; no output checkpoint was created"
            ) from exc
        require(stdout.tell() <= MAX_PROTOCOL_BYTES,
                "Runtime protocol output exceeded the bounded capture limit")
        stdout.seek(0)
        raw = stdout.read(MAX_PROTOCOL_BYTES + 1)
        stderr.seek(0)
        error_excerpt = stderr.read(2048).decode("utf-8", errors="replace").strip()
    replies = [decode_json(line) for line in raw.decode("utf-8").splitlines() if line.strip()]
    require(all(isinstance(reply, dict) for reply in replies), "Runtime returned a non-object reply")
    require(not any(reply.get("type") == "action" for reply in replies),
            "Unexpected actuator handshake; upgrade aborted without supplying POST")
    error = next((reply for reply in replies if reply.get("type") == "error"), None)
    if error or process.returncode != 0:
        detail = str(error.get("message", "runtime error")) if error else error_excerpt
        prior = next((reply.get("report") for reply in replies
                      if reply.get("type") == "inquiry_report"), None)
        if prior is not None:
            detail = "checkpoint already contains inquiry; its live inquiry state will not be replaced"
        raise ValueError(f"Runtime rejected the upgrade (exit {process.returncode}): {detail[:2048]}")
    actual_types = [reply.get("type") for reply in replies]
    require(actual_types == expected_types,
            f"Unexpected runtime protocol replies: {actual_types}")
    return replies


def without_extension(checkpoint, ignore_new_router):
    """A shallow structural view preserves every field outside the extension."""
    value = dict(checkpoint)
    value.pop("hash", None)  # Necessarily changes when new state is added.
    cognition = dict(value["cognition"])
    value["cognition"] = cognition
    current = dict(cognition["current"])
    cognition["current"] = current
    learner = dict(current["learner"])
    current["learner"] = learner
    memory = dict(learner["causal_memory"])
    learner["causal_memory"] = memory
    memory.pop("inquiry", None)
    memory.pop("bytes", None)  # Checked separately against the shared ceiling.
    if ignore_new_router:
        memory.pop("regime", None)
    return value


def write_exclusive(path, data):
    """Never replace an existing path; remove only our own partial write."""
    created_identity = None
    try:
        with path.open("xb") as stream:
            stat = os.fstat(stream.fileno())
            created_identity = (stat.st_dev, stat.st_ino)
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
    except BaseException:
        if created_identity is not None:
            try:
                stat = path.lstat()
                if (stat.st_dev, stat.st_ino) == created_identity:
                    path.unlink()
            except OSError:
                pass
        raise


def upgrade(args):
    started = time.monotonic()
    source = args.input.expanduser().resolve(strict=True)
    binary = args.binary.expanduser().resolve(strict=True)
    destination = args.output.expanduser().absolute()
    require(binary.is_file(), "--binary must name the existing aeterna-v1 executable")
    require(destination.parent.is_dir(), "Output parent directory must already exist")
    require(not destination.exists() and not destination.is_symlink(),
            "Output already exists; choose a new path (nothing will be overwritten)")
    destination = destination.parent.resolve(strict=True) / destination.name
    require(destination != source, "Input and output must be different files")
    original_bytes, original_text, original = read_checkpoint(source)
    input_hash = sha256_bytes(original_bytes)
    binary_hash = sha256_file(binary)
    current = original["cognition"].get("current")
    require(isinstance(current, dict) and isinstance(current.get("learner"), dict),
            "Input checkpoint lacks its current organism learner")
    old_memory = current["learner"].get("causal_memory")
    require(isinstance(old_memory, dict), "Checkpoint has no acquired causal memory")
    require(old_memory.get("inquiry") is None,
            "Checkpoint already contains inquiry; its live inquiry state will not be replaced")
    require(old_memory.get("config", {}).get("history_depth") == 3,
            "Inquiry requires existing causal-memory history_depth=3; this utility does not rewrite memory configuration")

    replies = run_protocol(binary, [
        {"op": "load", "checkpoint": original_text},
        {"op": "causal_report"}, {"op": "inquiry_report"}, {"op": "report"},
        {"op": "enable_autonomous_inquiry", "mode": args.mode},
        {"op": "causal_report"}, {"op": "inquiry_report"}, {"op": "report"},
        {"op": "checkpoint"}, {"op": "quit"},
    ], ["loaded", "causal_report", "inquiry_report", "report", "autonomous_inquiry_enabled",
        "causal_report", "inquiry_report", "report", "checkpoint", "bye"])
    before_causal, before_inquiry, before_runtime = (replies[i]["report"] for i in (1, 2, 3))
    after_causal, after_inquiry, after_runtime = (replies[i]["report"] for i in (5, 6, 7))
    require(before_inquiry is None, "Loaded runtime already has inquiry")
    require(isinstance(before_causal, dict) and isinstance(after_causal, dict),
            "Runtime lacks the required causal preservation reports")
    require(isinstance(before_runtime, dict) and isinstance(after_runtime, dict),
            "Runtime lacks the required native-controller reports")
    require(isinstance(after_inquiry, dict) and after_inquiry.get("mode") == args.mode,
            "Requested inquiry mode was not enabled")
    for field in ("observations", "questions_opened", "questions_resolved", "questions_expired",
                  "selected_experiments", "factual_experiments", "extra_inference_work"):
        require(after_inquiry.get(field) == 0, f"Enabling inquiry invented {field}")
    require(after_inquiry.get("active") is None and after_inquiry.get("last_resolution") is None,
            "Enabling inquiry invented a question or resolution")
    preserved_causal = {key: value for key, value in before_causal.items() if key != "model_bytes"}
    require(preserved_causal == {key: value for key, value in after_causal.items() if key != "model_bytes"},
            "Enabling inquiry changed acquired causal evidence, scores, or factual counters")
    required_causal = ("learned_fingerprint", "learned_transitions", "factual_transitions")
    require(all(key in before_causal for key in required_causal), "Required causal counters are missing")
    require("native_weights" in before_runtime, "Required native U1 weights are missing")
    require({k: v for k, v in before_runtime.items() if k != "inquiry"}
            == {k: v for k, v in after_runtime.items() if k != "inquiry"},
            "Enabling inquiry changed native control, costs, operations, or another organism report")

    checkpoint_text = replies[8].get("checkpoint")
    require(isinstance(checkpoint_text, str), "Runtime did not return a full checkpoint string")
    upgraded_bytes = checkpoint_text.encode("utf-8")
    require(len(upgraded_bytes) <= MAX_CHECKPOINT_BYTES, "Upgraded checkpoint exceeds runtime limit")
    upgraded = decode_json(checkpoint_text)
    no_old_router = old_memory.get("regime") is None
    require(without_extension(original, no_old_router) == without_extension(upgraded, no_old_router),
            "A field outside the new inquiry/router reservation changed in the full organism")
    new_memory = upgraded["cognition"]["current"]["learner"]["causal_memory"]
    added_bytes = new_memory["bytes"] - old_memory["bytes"]
    require(added_bytes >= after_inquiry["state_bytes"]
            and new_memory["bytes"] <= new_memory["config"]["model_bytes"],
            "Inquiry reservation exceeds the existing memory ceiling or is not accounted")
    require(isinstance(new_memory.get("regime"), dict)
            and new_memory["regime"].get("mode") == "ordinary",
            "Upgraded organism lacks the required ordinary applicability inference")

    verified = run_protocol(binary, [
        {"op": "load", "checkpoint": checkpoint_text}, {"op": "causal_report"},
        {"op": "inquiry_report"}, {"op": "report"}, {"op": "checkpoint"}, {"op": "quit"},
    ], ["loaded", "causal_report", "inquiry_report", "report", "checkpoint", "bye"])
    require(verified[4].get("checkpoint") == checkpoint_text,
            "Fresh-process checkpoint roundtrip was not byte exact")
    require([verified[i]["report"] for i in (1, 2, 3)] == [after_causal, after_inquiry, after_runtime],
            "Fresh-process reload changed evidence or native/inquiry state")
    require(sha256_file(source) == input_hash, "Original checkpoint changed during verification")
    require(sha256_file(binary) == binary_hash, "Runtime binary changed during verification")
    write_exclusive(destination, upgraded_bytes)
    return {
        "status": "upgraded", "mode": args.mode,
        "input": str(source), "output": str(destination), "binary": str(binary),
        "input_sha256": input_hash, "output_sha256": sha256_bytes(upgraded_bytes),
        "binary_sha256": binary_hash,
        "preservation": {
            "input_unchanged": True, "full_state_outside_extension_unchanged": True,
            "causal_evidence_unchanged": True, "native_controller_unchanged": True,
            "fresh_process_roundtrip_byte_exact": True,
            **{key: {"before": before_causal[key], "after": after_causal[key]}
               for key in required_causal},
            "native_weights": {"before": before_runtime["native_weights"], "after": after_runtime["native_weights"]},
            "total_external_cost": {"before": before_runtime["total_external_cost"],
                                    "after": after_runtime["total_external_cost"]},
        },
        "inquiry": after_inquiry,
        "added_model_reservation_bytes": added_bytes,
        "actuator_handshakes": 0, "factual_actions_executed": 0,
        "process_timeout_seconds": PROCESS_TIMEOUT_SECONDS,
        "elapsed_seconds": round(time.monotonic() - started, 6),
        "qualification": "state_preservation_only_no_behavioral_trial",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True, help="Existing complete saved organism checkpoint (read only)")
    parser.add_argument("--output", type=Path, required=True, help="New checkpoint path; existing paths are never overwritten")
    parser.add_argument("--binary", type=Path, required=True, help="Built aeterna-v1 or aeterna-v1.exe supporting autonomous inquiry")
    parser.add_argument("--mode", choices=("active", "observe_only"), default="active")
    args = parser.parse_args()
    try:
        result = upgrade(args)
    except KeyboardInterrupt:
        print(json.dumps({"status": "error", "error": "Upgrade interrupted; the original checkpoint was never opened for writing"}))
        return 130
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(json.dumps({"status": "error", "error": str(exc),
                          "original_opened_for_writing": False}, allow_nan=False))
        return 2
    # ASCII JSON is portable even when a Windows console's active code page
    # cannot display every character in the user's checkpoint path.
    print(json.dumps(result, allow_nan=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
