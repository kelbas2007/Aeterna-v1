#!/usr/bin/env python3
"""Run and independently score the preregistered endogenous-context comparison.

One legacy integrity replay precedes every new actor. Each of 16 fresh and 8
retained worlds has five paired arms, with switching and stationary continuations.
--score-only reads persisted evidence and never launches a runtime.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import copy
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
import sys
import time
import zipfile


ROOT = Path(__file__).resolve().parents[1]
BASE_COMMIT = "b714f40c83bb6ba2a9c383d67f3687a67b748ab9"
PRODUCTION_COMMIT = "01a35d0c69c3dec0340c753456ada4a7b7f221da"
ORIGINAL_SEED = 45231338992
ARMS = ("baseline", "ordinary", "phase", "phase_scrambled", "phase_uncoupled")
CONDITIONS = {"switching": 2048, "stationary": 512}
SUITES = {"fresh": 16, "retained": 8}
ACTION_LIMIT = 4
STATE_BYTES = 2048
NLL_FLOOR = 1e-12
TIMEOUT_SECONDS = 2400
ORIGINAL_ARCHIVE = ROOT / "docs/experiments/evidence/integral_causal_revision_20261123/integral-causal-evidence-45231338992.zip"
ORIGINAL_ARCHIVE_SHA256 = "73c4a3f6d1bb27d62c0fdf963479dbe45b489493ebf87ec5022d63fb3796c21e"
RETAINED_ARCHIVE = ROOT / "docs/experiments/evidence/integral_retained_knowledge_20261123/integral-retained-knowledge-45235768424.zip"
RETAINED_ARCHIVE_SHA256 = "bf231f8834d9f4b954bf1bcb064931836763dac76d51bbc8d8c73de48cf3c571"
FROZEN_FILES = {
    "examples/integral_causal_revision_eval.rs": "f79dc005680a01d4fb011b70f0ac1b127c0d8107b63d8b48f34d0b90d4759c4f",
    "scripts/run_integral_causal_revision.py": "7b2081b82c9c9bd529a1c57e128192d36e88cec95ea5cf4cf40de2fd6f0da514",
    "docs/experiments/INTEGRAL_CAUSAL_REVISION_20261123_PROTOCOL.md": "a699624f9748742e25bc051c5fe32f44a3ddb4185f4aa58e516e837e8f77ffb7",
    "examples/integral_retained_knowledge_eval.rs": "39a1dd56eb55ef17f2c855441dde65f444bf45916f19c0cc8c066c046b1de507",
    "scripts/run_integral_retained_knowledge.py": "d37467f9f1ea15c63d35d0bc15bf391edadcfa77c092ce5bdc955c3df31fcd5b",
    "docs/experiments/INTEGRAL_RETAINED_KNOWLEDGE_20261123_PROTOCOL.md": "a773cf5bbdc6ae757d0e0a7ee292ce40bfb0fdcfb15da1fc67f317ba7e6a4fc4",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write_json(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def digest_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def fingerprint(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return value


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def provenance():
    tracked = subprocess.run(["git", "ls-files", "-z", "--", "src", "Cargo.toml", "Cargo.lock",
                              "rust-toolchain.toml", "rust-toolchain"], cwd=ROOT, check=True,
                             capture_output=True).stdout.decode().split("\0")
    files = set(filter(None, tracked)) | set(FROZEN_FILES) | {
        "examples/integral_phase_context_eval.rs", "scripts/run_integral_phase_context.py"}
    for pattern in ("docs/experiments/*PHASE_CONTEXT*PROTOCOL.md", ".github/workflows/*phase-context*.yml"):
        files.update(str(p.relative_to(ROOT)) for p in ROOT.glob(pattern))
    require(any("PHASE_CONTEXT" in p for p in files), "frozen phase-context protocol missing")
    require(any(".github/workflows/" in p for p in files), "frozen execution workflow missing")
    hashes = {name: sha256(ROOT / name) for name in sorted(files)}
    for name, expected in FROZEN_FILES.items():
        require(hashes[name] == expected, f"earlier frozen experiment changed: {name}")
    git("ls-files", "--error-unmatch", "--", *sorted(files))
    require(not git("diff", "--name-only", "HEAD", "--", *sorted(files)),
            "execution source differs from the committed version")
    return {"source_commit": git("rev-parse", "HEAD"), "source_sha256": hashes,
            "experiment_parent_commit": BASE_COMMIT, "unchanged_main_commit": PRODUCTION_COMMIT}


def load_original():
    require(sha256(ORIGINAL_ARCHIVE) == ORIGINAL_ARCHIVE_SHA256, "original archive hash mismatch")
    with zipfile.ZipFile(ORIGINAL_ARCHIVE) as archive:
        trace = archive.read("integral-evidence/world-0.jsonl")
        summary = json.loads(archive.read("integral-evidence/world-0.json"))
        verdict = json.loads(archive.read("integral-evidence/verdict.json"))
    require(verdict["verdict"] == "FAIL" and verdict["complete_loop_worlds"] == 4,
            "original historical verdict identity mismatch")
    return {"trace_sha256": digest_bytes(trace), "summary": summary}


def retained_inputs(output, extract):
    require(sha256(RETAINED_ARCHIVE) == RETAINED_ARCHIVE_SHA256, "retained archive hash mismatch")
    directory = output / "retained-input"
    if extract:
        directory.mkdir()
    records = []
    with zipfile.ZipFile(RETAINED_ARCHIVE) as archive:
        for world in range(SUITES["retained"]):
            member = f"retained-evidence/repeat/checkpoints/world-{world}/continuation-unfrozen.checkpoint.json"
            data = archive.read(member)
            path = directory / f"world-{world}.checkpoint.json"
            if extract:
                path.write_bytes(data)
            require(path.read_bytes() == data, f"retained input differs from archived state: world {world}")
            parsed = json.loads(data)
            learner = parsed["cognition"]["current"]["learner"]
            causal = learner["causal_memory"]
            require(not parsed["cognition"]["freeze_controller"], "retained input is frozen")
            require(causal["learned_events"] > 0, "retained input has no learned experience")
            require(causal.get("regime") is None, "historical input unexpectedly contains new context routing")
            records.append({"world": world, "archive_member": member, "file": str(path.relative_to(output)),
                            "sha256": digest_bytes(data), "bytes": len(data),
                            "factual_events": causal["events"], "learned_events": causal["learned_events"]})
    manifest = {"archive_sha256": RETAINED_ARCHIVE_SHA256, "world_seed": ORIGINAL_SEED, "checkpoints": records}
    if extract:
        write_json(directory / "manifest.json", manifest)
    else:
        require(read_json(directory / "manifest.json") == manifest, "retained input manifest differs")
    return manifest


def legacy_replay(output, binary):
    directory = output / "legacy-baseline"
    directory.mkdir()
    command = [str(binary), "--seed", str(ORIGINAL_SEED), "--world", "0", "--trace", str(directory / "world-0.jsonl")]
    started = time.monotonic()
    record = {"command": command, "world": 0, "seed": ORIGINAL_SEED, "timeout_seconds": TIMEOUT_SECONDS}
    try:
        with (directory / "world-0.json").open("w") as out, (directory / "stderr.log").open("w") as err:
            result = subprocess.run(command, stdout=out, stderr=err, timeout=TIMEOUT_SECONDS, check=False)
        record.update(exit=result.returncode, timed_out=False)
    except subprocess.TimeoutExpired:
        record.update(exit=124, timed_out=True)
    except Exception as exc:
        record.update(exit=-1, timed_out=False, error=str(exc))
    record["seconds"] = time.monotonic() - started
    write_json(directory / "process.json", record)
    gate = verify_legacy(output)
    write_json(output / "legacy-gate.json", gate)
    print(f"PHASE_LEGACY_BARRIER valid={gate['valid']} seconds={record['seconds']:.2f}", flush=True)
    return gate


def verify_legacy(output):
    errors = []
    details = {}
    try:
        original = load_original()
        directory = output / "legacy-baseline"
        process = read_json(directory / "process.json")
        require(process["exit"] == 0 and not process["timed_out"], "legacy evaluator did not finish normally")
        actual = sha256(directory / "world-0.jsonl")
        same_summary = read_json(directory / "world-0.json") == original["summary"]
        details = {"expected_trace_sha256": original["trace_sha256"], "actual_trace_sha256": actual,
                   "parsed_summary_equal": same_summary, "process": process}
        require(actual == original["trace_sha256"] and same_summary,
                "disabled context extension changed original baseline behavior or reports")
    except Exception as exc:
        errors.append(str(exc))
    return {"valid": not errors, "errors": errors, "details": details,
            "counts_as_new_ability_evidence": False}


def run_worlds(output, binary, seed, workers):
    jobs = [(suite, world) for suite, count in SUITES.items() for world in range(count)]

    def one(job):
        suite, world = job
        directory = output / suite / f"world-{world}"
        directory.parent.mkdir(exist_ok=True)
        # Evaluator creates its output directory. Logs stay alongside it so an
        # early process error still has a durable record.
        command = [str(binary), "--seed", str(seed), "--suite", suite,
                   "--world", str(world), "--out-dir", str(directory)]
        if suite == "retained":
            command += ["--checkpoint", str(output / "retained-input" / f"world-{world}.checkpoint.json")]
        record = {"suite": suite, "world": world, "command": command, "timeout_seconds": TIMEOUT_SECONDS}
        started = time.monotonic()
        try:
            with (directory.parent / f"world-{world}.stdout.log").open("w") as out, \
                    (directory.parent / f"world-{world}.stderr.log").open("w") as err:
                result = subprocess.run(command, stdout=out, stderr=err, timeout=TIMEOUT_SECONDS, check=False)
            record.update(exit=result.returncode, timed_out=False)
        except subprocess.TimeoutExpired:
            record.update(exit=124, timed_out=True)
        except Exception as exc:
            record.update(exit=-1, timed_out=False, error=str(exc))
        record["seconds"] = time.monotonic() - started
        write_json(directory.parent / f"world-{world}.process.json", record)
        print(f"PHASE_WORLD suite={suite} world={world} exit={record['exit']} seconds={record['seconds']:.2f}", flush=True)
        return record

    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
        records = list(executor.map(one, jobs))
    write_json(output / "processes.json", records)


def safe_file(directory, name):
    path = Path(name)
    require(not path.is_absolute() and ".." not in path.parts, f"unsafe evidence path: {name}")
    result = directory / path
    require(result.is_file(), f"missing evidence file: {name}")
    return result


def close(actual, expected, label):
    require(isinstance(actual, (int, float)) and not isinstance(actual, bool) and math.isfinite(actual),
            f"invalid numeric {label}")
    require(math.isclose(actual, expected, rel_tol=2e-9, abs_tol=2e-8),
            f"{label} differs: observed={actual}, independently scored={expected}")


def causal_checkpoint(parsed):
    return parsed["cognition"]["current"]["learner"]["causal_memory"]


def normalized_start(parsed):
    value = copy.deepcopy(parsed)
    value.pop("hash", None)
    causal = causal_checkpoint(value)
    regime = causal.pop("regime", None)
    if regime is not None:
        causal["bytes"] -= STATE_BYTES
    return value


def inspect_checkpoints(directory, summary, suite, world, output):
    integrity = read_json(directory / "integrity.json")
    for name in ("common_source_exact", "baseline_clone_exact", "arm_start_model_equivalent", "final_roundtrips_exact"):
        require(integrity.get(name) is True, f"evaluator checkpoint check failed: {name}")
    entries = integrity["checkpoints"]
    require(len({r["file"] for r in entries}) == len(entries), "duplicate checkpoint records")
    expected = {"common.checkpoint.json"} | {f"starts/{arm}.checkpoint.json" for arm in ARMS}
    expected |= {f"finals/{condition}-{arm}{suffix}.checkpoint.json"
                 for condition in CONDITIONS for arm in ARMS for suffix in ("", ".frozen")}
    require({r["file"] for r in entries} == expected, "checkpoint set does not match protocol")
    files, parsed = {}, {}
    for record in entries:
        path = safe_file(directory, record["file"])
        data = path.read_bytes()
        require(record["roundtrip_exact"] is True, f"checkpoint did not roundtrip: {record['file']}")
        require(record["bytes"] == len(data) and record["fingerprint"] == fingerprint(data),
                f"checkpoint bytes/fingerprint mismatch: {record['file']}")
        value = json.loads(data)
        memory = causal_checkpoint(value)
        learner = value["cognition"]["current"]["learner"]
        for key, expected_value in {"width": 2, "actions": 4, "planning_depth": 4, "planning_work": 120000,
                                    "minimum_training": 128, "search_every": 1000000, "journal_capacity": 4096}.items():
            require(learner["config"][key] == expected_value, f"persisted runtime {key} differs from fixed cap")
        require(learner["autonomy"]["max_external_cost"] == 100000
                and value["cognition"]["max_external_cost"] == 100000, "persisted lifetime cost ceiling differs")
        require(0 <= memory["learned_events"] <= memory["events"], "invalid persisted learning counters")
        require(len(memory["counts"]) <= memory["config"]["model_entries"], "checkpoint exceeds entry cap")
        require(memory["bytes"] <= memory["config"]["model_bytes"], "checkpoint exceeds byte cap")
        require(value["cognition"]["freeze_controller"] == (".frozen." in record["file"]),
                f"unexpected frozen checkpoint state: {record['file']}")
        files[record["file"]] = {"sha256": digest_bytes(data), "bytes": len(data), "fingerprint": record["fingerprint"],
                                "factual_transitions": memory["events"], "learned_transitions": memory["learned_events"],
                                "model_bytes": memory["bytes"], "model_entries": len(memory["counts"])}
        if record["file"] == "common.checkpoint.json" or record["file"].startswith("starts/"):
            parsed[record["file"]] = value
    common = parsed["common.checkpoint.json"]
    if suite == "retained":
        path = output / "retained-input" / f"world-{world}.checkpoint.json"
        require((directory / "common.checkpoint.json").read_bytes() == path.read_bytes(),
                "common start differs from actual retained checkpoint bytes")
    normalized = normalized_start(common)
    for arm in ARMS:
        start = parsed[f"starts/{arm}.checkpoint.json"]
        require(normalized_start(start) == normalized, f"{arm} altered inherited state beyond bounded context inference")
        has_regime = causal_checkpoint(start).get("regime") is not None
        require(has_regime == (arm != "baseline"), f"{arm} has unexpected context enablement")
        if has_regime:
            require(causal_checkpoint(start)["regime"]["mode"] == arm, "context start mode does not match named arm")
    require(files["starts/baseline.checkpoint.json"]["sha256"] == files["common.checkpoint.json"]["sha256"],
            "baseline start is not the exact common checkpoint")
    return {"files": files, "common_learned_transitions": causal_checkpoint(common)["learned_events"],
            "common_factual_transitions": causal_checkpoint(common)["events"],
            "common_protection_sequence": common["protection_sequence"],
            "causal_config": causal_checkpoint(common)["config"]}


def inspect_schedule(schedule, suite, world, seed):
    require(schedule["seed"] == seed and schedule["suite"] == suite and schedule["world"] == world,
            "schedule identity mismatch")
    require(sorted(schedule["motors"]) == [0, 1, 2, 3], "motor mapping is not a permutation")
    require(sorted(schedule["codes"]) == [101, 211, 307, 419, 503, 617, 733], "observation mapping is not a permutation")
    if suite == "retained":
        require(schedule["mapping_seed"] == ORIGINAL_SEED, "historical mapping seed changed")
    segments = schedule["segments"]
    require(len(segments) == 8, "switching schedule must contain eight segments")
    cursor = 0
    for index, segment in enumerate(segments):
        require(segment["index"] == index and segment["start_episode"] == cursor, "segments not contiguous")
        require(segment["length"] >= 128 and segment["end_episode"] == cursor + segment["length"],
                "invalid segment duration")
        require(segment["rule"] == ((suite == "retained") ^ bool(index % 2)), "incorrect hidden rule sequence")
        cursor = segment["end_episode"]
    require(cursor == CONDITIONS["switching"], "switching total differs from 2048")
    expected = dict(CONDITIONS, acquisition=384 if suite == "fresh" else 0)
    require(set(schedule["conditions"]) == set(expected), "unexpected schedule condition set")
    for condition, count in expected.items():
        rows = schedule["conditions"][condition]
        require(len(rows) == count, f"incorrect schedule episode count: {condition}")
        for episode, row in enumerate(rows):
            require(row["episode"] == episode and row["hidden"] in (0, 1), "invalid schedule episode")
            require(len(row["noise"]) == ACTION_LIMIT and all(x in (0, 1) for x in row["noise"]), "invalid noise schedule")
            if condition == "switching":
                segment = next(s for s in segments if s["start_episode"] <= episode < s["end_episode"])
                require(row["segment"] == segment["index"] and row["rule"] == segment["rule"],
                        "episode does not match hidden segment")
            else:
                require(row["segment"] == 0 and row["rule"] == int(suite == "retained"),
                        "stationary/acquisition schedule changes its hidden rule")
    regenerated = expected_schedule(seed, suite, world)
    for key in ("mapping_seed", "motors", "codes", "segments", "conditions"):
        require(schedule[key] == regenerated[key], f"{key} differs from independently regenerated private schedule")
    require(schedule["actor_receives_schedule"] is False, "schedule metadata indicates privileged actor access")


def expected_schedule(seed, suite, world):
    """Independent port of the frozen physical generator; never executes an actor."""
    mask = (1 << 64) - 1

    def mix(x):
        x = (x + 0x9e3779b97f4a7c15) & mask
        x = ((x ^ (x >> 30)) * 0xbf58476d1ce4e5b9) & mask
        x = ((x ^ (x >> 27)) * 0x94d049bb133111eb) & mask
        return x ^ (x >> 31)

    class Rng:
        def __init__(self, value):
            self.value = value

        def next(self):
            self.value = (self.value + 0x9e3779b97f4a7c15) & mask
            return mix(self.value)

        def shuffle(self, values):
            for index in range(len(values) - 1, 0, -1):
                other = self.next() % (index + 1)
                values[index], values[other] = values[other], values[index]

    mapping_seed = ORIGINAL_SEED if suite == "retained" else seed
    mapping_base = mix(mapping_seed ^ ((world * 0xd1342543de82ef95) & mask))
    motors, codes = list(range(4)), [101, 211, 307, 419, 503, 617, 733]
    Rng(mapping_base ^ 0x6d6f746f725f6d61).shuffle(motors)
    Rng(mapping_base ^ 0x6f62735f636f6465).shuffle(codes)
    tag = 0x66726573685f7068 if suite == "fresh" else 0x72657461696e7068
    base = mix(seed ^ tag ^ ((world * 0xd1342543de82ef95) & mask))
    rng = Rng(base ^ 0x6477656c6c5f746d)
    weights = [1 + rng.next() % 1024 for _ in range(8)]
    lengths = [128 + 1024 * weight // sum(weights) for weight in weights]
    order = list(range(8))
    rng.shuffle(order)
    for index in order[:2048 - sum(lengths)]:
        lengths[index] += 1
    segments, cursor = [], 0
    for index, length in enumerate(lengths):
        segments.append({"index": index, "start_episode": cursor, "end_episode": cursor + length,
                         "length": length, "rule": int(suite == "retained") ^ (index & 1)})
        cursor += length
    stationary = [{"index": 0, "start_episode": 0, "end_episode": 512,
                   "length": 512, "rule": int(suite == "retained")}]
    acquisition = [{"index": 0, "start_episode": 0, "end_episode": 384, "length": 384, "rule": 0}] if suite == "fresh" else []
    conditions = {}
    for name, parts, salt in (("switching", segments, 0x7377697463687068),
                              ("stationary", stationary, 0x73746174696f7068),
                              ("acquisition", acquisition, 0x6163717569737068)):
        private = mix(base ^ salt)
        rows = []
        for part in parts:
            for episode in range(part["start_episode"], part["end_episode"]):
                hidden = mix(private ^ 0x68696464656e5f62 ^ ((episode * 0xa24baed4963ee407) & mask)) & 1
                noise = [mix(private ^ 0x6e6f6973655f6269 ^ ((episode * 0x9fb21c651e98df25) & mask)
                             ^ ((step * 0xc13fa9a902a6328f) & mask)) & 1 for step in range(ACTION_LIMIT)]
                rows.append({"episode": episode, "segment": part["index"], "rule": part["rule"],
                             "hidden": hidden, "noise": noise})
        conditions[name] = rows
    return {"mapping_seed": mapping_seed, "motors": motors, "codes": codes,
            "segments": segments, "conditions": conditions}


def snapshot_checks(before, after, actions, arm, config):
    for snapshot in (before, after):
        require(snapshot["controller_frozen"] is False, "primary continuation unexpectedly froze learning")
        causal = snapshot["causal_report"]
        require(causal["retained_entries"] <= config["model_entries"], "reported model entries exceed cap")
        require(causal["model_bytes"] <= config["model_bytes"], "reported model bytes exceed cap")
        agency = snapshot["agency_credit"]
        require(agency["externally_forced_actions"] == 0 and agency.get("option_continuation_actions", 0) == 0,
                "actions were externally forced or delegated to an option")
        require((snapshot.get("regime_report") is not None) == (arm != "baseline"), "context arm/report mismatch")
        if arm != "baseline":
            require(snapshot["regime_report"]["state_bytes"] == STATE_BYTES, "context state reservation differs")
            require(snapshot["regime_report"]["mode"] == arm, "context snapshot mode does not match named arm")
    for key in ("factual_events", "total_external_cost"):
        require(after[key] - before[key] == actions, f"{key} increment is not actual callbacks")
    for key in ("factual_transitions", "learned_transitions"):
        require(after["causal_report"][key] - before["causal_report"][key] == actions,
                f"causal {key} increment is not factual actions")
    require(after["agency_credit"]["self_selected_actions"] - before["agency_credit"]["self_selected_actions"] == actions,
            "self-selected action credit differs from executed actions")


def probability_from_beliefs(beliefs, action, post):
    require(len(beliefs) == 4 and sorted(b["action"] for b in beliefs) == [0, 1, 2, 3],
            "prospective beliefs missing an equally available motor")
    for belief in beliefs:
        require(type(belief["support"]) is int and belief["support"] >= 0, "invalid prospective support")
        mass = 0.0
        seen = set()
        for observation, probability in belief["outcomes"]:
            require(isinstance(probability, (int, float)) and math.isfinite(probability)
                    and 0 <= probability <= 1.000000001, "invalid prospective probability")
            require(tuple(observation) not in seen, "duplicate prospective outcome")
            seen.add(tuple(observation))
            mass += probability
        require(mass <= 1.00000001, "prospective probability mass exceeds one")
    chosen = next(b for b in beliefs if b["action"] == action)
    return sum(p for observation, p in chosen["outcomes"] if observation == post)


def window_metric(episodes):
    return {"episodes": len(episodes), "successes": sum(e["success"] for e in episodes),
            "success_rate": sum(e["success"] for e in episodes) / len(episodes) if episodes else None,
            "actions": sum(e["actions"] for e in episodes),
            "sensor_queries": sum(e["sensor_queries"] for e in episodes),
            "noise_queries": sum(e["noise_queries"] for e in episodes)}


def inspect_trace(directory, result, schedule, context):
    condition, arm = result["condition"], result["arm"]
    require(condition in (*CONDITIONS, "acquisition") and arm in ARMS, "unknown trace arm/condition")
    rows = schedule["conditions"][condition]
    expected_count = len(rows)
    trace = safe_file(directory, result["trace_file"])
    before, after = result["before"], result["after"]
    base_event = before["factual_events"]
    base_protection = context["common_protection_sequence"] if condition != "acquisition" else 0
    motors, codes = schedule["motors"], schedule["codes"]
    roles = ("informative_sensor", "noise_sensor", "actuator0", "actuator1")
    total_actions = 0
    source_counts = {}
    episodes = []
    row_count = 0
    current_episode = None
    expected_step = 0
    history = []
    pre = [0, codes[0]]
    episode_record = None
    previous_terminal = False
    operation_counts = {}
    belief_work = computation_work = 0
    switches = []
    nll_total = 0.0
    last_compute = before["computation_work"]
    last_fingerprint = before["causal_report"]["learned_fingerprint"]
    peak_entries = before["causal_report"]["retained_entries"]
    peak_bytes = before["causal_report"]["model_bytes"]

    def finish_episode():
        if episode_record is None:
            return
        require(previous_terminal or expected_step == ACTION_LIMIT,
                f"episode {current_episode} ended without terminal outcome or exhausting its budget")
        episode_record["timeout"] = not previous_terminal
        episodes.append(episode_record.copy())

    with trace.open(encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, 1):
            row = json.loads(line)
            row_count += 1
            episode = row["episode"]
            require(type(episode) is int and 0 <= episode < expected_count, "trace episode outside schedule")
            if episode != current_episode:
                finish_episode()
                require(episode == len(episodes), "episodes not contiguous and ordered")
                current_episode = episode
                expected_step = 0
                history = []
                pre = [0, codes[0]]
                previous_terminal = False
                episode_record = {"episode": episode, "segment": rows[episode]["segment"],
                                  "rule": rows[episode]["rule"], "success": False, "terminal": False,
                                  "actions": 0, "sensor_queries": 0, "noise_queries": 0,
                                  "nll_sum": 0.0, "nll_count": 0}
            require(not previous_terminal and row["step"] == expected_step < ACTION_LIMIT,
                    f"invalid or post-terminal step at line {line_number}")
            for key in ("seed", "suite", "world"):
                require(row[key] == schedule[key], f"trace {key} identity mismatch")
            require(row["condition"] == condition and row["arm"] == arm, "mixed trace arm/condition")
            physical = rows[episode]
            require(row["segment"] == physical["segment"] and row["rule"] == physical["rule"]
                    and row["hidden"] == physical["hidden"] and row["sensor_cue"] == physical["hidden"],
                    "trace physical schedule differs across arms or from declared environment")
            require(row["pre"] == pre and row["history"] == history, "trace PRE/history continuity failed")
            action = row["action"]
            require(type(action) is int and action in motors, "invalid selected motor")
            role = motors.index(action)
            terminal = role >= 2
            success = terminal and role - 2 == (physical["hidden"] ^ physical["rule"])
            state = (1 + physical["hidden"] if role == 0 else
                     3 + physical["noise"][expected_step] if role == 1 else 5 if success else 6)
            post = [0, codes[state]]
            require(row["role"] == roles[role] and row["post"] == post
                    and row["terminal"] is terminal and row["success"] is success,
                    "observed transition differs from independently calculated physics")
            require(row["cost"] == 1 and row["reward"] == int(success), "reward/cost differs from actual outcome")
            total_actions += 1
            require(row["event"] == base_event + total_actions, "factual event identity not contiguous")
            require(row["callbacks_total"] == total_actions, "callback/action count differs")
            require(row["permit_sequence"] == base_protection + total_actions, "permit sequence/action identity differs")
            for key in ("factual_transitions", "learned_transitions"):
                initial = before["causal_report"][key]
                require(row[key + "_before"] == initial + total_actions - 1
                        and row[key + "_after"] == initial + total_actions, "per-step causal counter differs")
            require(row["learned_fingerprint_before"] == last_fingerprint, "prospective model fingerprint chain broken")
            last_fingerprint = row["learned_fingerprint_after"]
            require(row["causal_retained_entries"] <= context["causal_config"]["model_entries"]
                    and row["causal_model_bytes"] <= context["causal_config"]["model_bytes"], "per-step model exceeds capacity")
            peak_entries = max(peak_entries, row["causal_retained_entries"])
            peak_bytes = max(peak_bytes, row["causal_model_bytes"])
            require(isinstance(row["source"], str) and row["source"], "missing runtime-owned choice provenance")
            require(row["source"] != "uniform-random", "comparison arm replaced by random action selector")
            source_counts[row["source"]] = source_counts.get(row["source"], 0) + 1
            p = probability_from_beliefs(row["pre_beliefs"], action, post)
            close(row["prequential_probability"], p, "prospective factual-outcome probability")
            nll = -math.log(max(p, NLL_FLOOR))
            close(row["prequential_nll"], nll, "step NLL")
            nll_total += nll
            episode_record["nll_sum"] += nll
            episode_record["nll_count"] += 1
            require(row["computation_work_before"] >= last_compute, "cumulative computation work went backwards")
            delta = row["computation_work"] - row["computation_work_before"]
            require(delta >= 0 and delta == row["computation_work_delta"], "invalid computation work delta")
            computation_work += delta
            last_compute = row["computation_work"]
            require(isinstance(row["operations"], list) and all(isinstance(x, str) for x in row["operations"]),
                    "invalid operation provenance")
            for operation in row["operations"]:
                operation_counts[operation] = operation_counts.get(operation, 0) + 1
            require(type(row["readonly_belief_work"]) is int and 0 <= row["readonly_belief_work"] <= 120000,
                    "diagnostic belief work exceeds fixed cap")
            belief_work += row["readonly_belief_work"]
            before_regime, after_regime = row.get("regime_before"), row.get("regime_after")
            require((before_regime is None) == (arm == "baseline")
                    and (after_regime is None) == (arm == "baseline"), "per-step context arm mismatch")
            if arm != "baseline":
                require(before_regime["mode"] == arm and after_regime["mode"] == arm,
                        "context trace mode does not match named arm")
                require(after_regime["observations"] == before_regime["observations"] + 1,
                        "context inference did not observe exactly one factual POST")
                if after_regime["bank"] != before_regime["bank"]:
                    switches.append({"episode": episode, "step": expected_step,
                                     "from": before_regime["bank"], "to": after_regime["bank"]})
            episode_record["actions"] += 1
            episode_record["sensor_queries"] += int(role == 0)
            episode_record["noise_queries"] += int(role == 1)
            episode_record["success"] = success
            episode_record["terminal"] = terminal
            history.append({"pre": pre, "action": action, "post": post})
            history = history[-3:]
            pre = post
            previous_terminal = terminal
            expected_step += 1
    finish_episode()
    require(len(episodes) == expected_count, "trace is incomplete")
    summary_episodes = result["episode_results"]
    require(len(summary_episodes) == expected_count, "summary episode count differs")
    for actual, observed in zip(summary_episodes, episodes):
        for key in ("episode", "segment", "rule", "success", "terminal", "actions", "sensor_queries", "noise_queries"):
            require(actual[key] == observed[key], f"summary episode {observed['episode']} {key} differs from trace")
        close(actual["prequential_nll_sum"], observed["nll_sum"], "episode NLL")
        require(actual["prequential_nll_count"] == observed["nll_count"], "episode NLL denominator differs")
    metrics = window_metric(episodes)
    metrics.update(timeouts=sum(e["timeout"] for e in episodes),
                   terminal_failures=sum(e["terminal"] and not e["success"] for e in episodes),
                   terminal_actions=sum(e["terminal"] for e in episodes),
                   prequential_nll_sum=nll_total, prequential_nll_count=row_count,
                   prequential_nll_mean=nll_total / row_count, readonly_belief_work=belief_work,
                   computation_work=computation_work, source_counts=source_counts, operation_counts=operation_counts,
                   inferred_context_switches=len(switches), switch_events=switches)
    for key in ("episodes", "successes", "actions", "sensor_queries", "noise_queries", "terminal_actions", "prequential_nll_count",
                "readonly_belief_work", "computation_work", "terminal_failures", "timeouts"):
        require(result[key] == metrics[key], f"result {key} differs from independently scored trace")
    require(result["source_counts"] == source_counts and result["operation_counts"] == operation_counts,
            "source/operation summary differs from trace")
    require(result["physical_callbacks"] == total_actions, "physical callback total differs from trace")
    require(result["peak_retained_entries"] == peak_entries and result["peak_model_bytes"] == peak_bytes,
            "reported resource peak differs from trace")
    close(result["prequential_nll_sum"], nll_total, "result NLL")
    require(result["wall_seconds"] >= 0 and math.isfinite(result["wall_seconds"]), "invalid measured runtime")
    snapshot_checks(before, after, total_actions, arm, context["causal_config"])
    require(after["causal_report"]["learned_fingerprint"] == last_fingerprint,
            "final learned-model fingerprint differs from trace")
    require(after["computation_work"] - before["computation_work"] == computation_work,
            "runtime computation snapshot differs from trace deltas")
    first_windows, last_windows, per_segment = [], [], []
    segment_ids = sorted({e["segment"] for e in episodes})
    for index in segment_ids:
        part = [e for e in episodes if e["segment"] == index]
        item = {"segment": index, "rule": part[0]["rule"], **window_metric(part),
                "first32": window_metric(part[:32]), "last32": window_metric(part[-32:])}
        per_segment.append(item)
        if condition == "switching" and index > 0:
            first_windows.extend(part[:32])
        last_windows.extend(part[-32:])
    metrics.update(first32_after_reversals=window_metric(first_windows), last32_of_segments=window_metric(last_windows),
                   segments=per_segment, wall_seconds=result["wall_seconds"],
                   model_bytes=after["causal_report"]["model_bytes"],
                   model_entries=after["causal_report"]["retained_entries"],
                   peak_model_bytes=peak_bytes, peak_model_entries=peak_entries,
                   evicted_entries=after["causal_report"]["evicted_entries"] - before["causal_report"]["evicted_entries"],
                   trace_sha256=sha256(trace), trace_file=result["trace_file"])
    if arm == "baseline":
        metrics["inference_work_charged_upper_bound"] = 0
    else:
        r0, r1 = before["regime_report"], after["regime_report"]
        metrics["inference_work_charged_upper_bound"] = r1["extra_inference_work"] - r0["extra_inference_work"]
        require(metrics["inference_work_charged_upper_bound"] == 8192 * total_actions,
                "inference charged work differs from fixed per-POST allowance")
        require(r1["switches"] - r0["switches"] == len(switches), "context switch counter differs from trace")
    require(result["extra_inference_work"] == metrics["inference_work_charged_upper_bound"],
            "inference-work summary differs from actual inference counters")
    boundaries = [s["start_episode"] for s in schedule["segments"][1:]] if condition == "switching" else []
    detected = [any(b <= s["episode"] < b + 32 for s in switches) for b in boundaries]
    metrics["switch_diagnostics"] = {
        "true_reversals": len(boundaries), "reversals_with_inferred_switch_within32": sum(detected),
        "reversals_without_inferred_switch_within32": len(boundaries) - sum(detected),
        "inferred_switches_outside_first32": sum(not any(b <= s["episode"] < b + 32 for b in boundaries) for s in switches),
        "qualification": "Descriptive timing overlap only; bank labels are not true-rule labels and an overlap does not establish correct inference."}
    return metrics


def inspect_world(output, suite, world, seed):
    directory = output / suite / f"world-{world}"
    process = read_json(directory.parent / f"world-{world}.process.json")
    require(process["suite"] == suite and process["world"] == world, "process identity mismatch")
    require(process["exit"] == 0 and not process["timed_out"], "world process incomplete, failed or timed out")
    summary = read_json(directory / "summary.json")
    require(summary["schema"] == "aeterna.integral-phase-context.world.v1", "unexpected world schema")
    require(summary["seed"] == seed and summary["suite"] == suite and summary["world"] == world,
            "world summary identity mismatch")
    schedule = read_json(directory / "schedule.json")
    inspect_schedule(schedule, suite, world, seed)
    require(summary["segments"] == schedule["segments"], "summary segments differ from physical schedule")
    context = inspect_checkpoints(directory, summary, suite, world, output)
    config = summary["config"]
    expected_config = {"width": 2, "actions": 4, "minimum_training": 128, "search_every": 1000000,
                       "planning_depth": 4, "planning_work": 120000, "history_depth": 3,
                       "model_entries": 2048, "model_bytes": 2 * 1024 * 1024, "outcomes_per_entry": 16,
                       "planning_states": 192, "journal_capacity": 4096, "max_external_cost": 100000,
                       "episode_action_limit": 4, "acquisition_episodes": 384, "switching_episodes": 2048,
                       "stationary_episodes": 512, "readonly_belief_work_per_call": 120000, "nll_floor": NLL_FLOOR,
                       "readonly_diagnostics_outside_actor_budget": True, "all_behavioral_episodes_learning_enabled": True,
                       "phase_parameter_selection_on_test_results": False}
    for key, value in expected_config.items():
        require(config[key] == value, f"configured {key} differs from frozen resource/protocol setting")
    for key, value in context["causal_config"].items():
        require(config[key] == value, f"checkpoint causal config differs: {key}")
    common = summary["common_before"]
    require(common["causal_report"]["learned_transitions"] == context["common_learned_transitions"]
            and common["causal_report"]["factual_transitions"] == context["common_factual_transitions"],
            "common snapshot differs from persisted state")
    start_rows = summary["arm_starts"]
    require(len(start_rows) == len(ARMS) and {r["arm"] for r in start_rows} == set(ARMS), "missing/duplicate arm starts")
    starts = {r["arm"]: r for r in start_rows}
    for arm, item in starts.items():
        require(item["common_source_exact"] is True and item["model_equivalent"] is True, "unequal ancestor state")
        require(item["checkpoint"]["file"] == f"starts/{arm}.checkpoint.json", "incorrect arm start path")
    results = summary["results"]
    require(len(results) == len(ARMS) * len(CONDITIONS)
            and {(r["condition"], r["arm"]) for r in results} == {(c, a) for c in CONDITIONS for a in ARMS},
            "missing or duplicate arm/condition result")
    scored = {condition: {} for condition in CONDITIONS}
    for result in results:
        arm, condition = result["arm"], result["condition"]
        require(result["before"] == starts[arm]["snapshot"], "condition did not start from its unchanged common clone")
        require(result["checkpoint_file"] == f"finals/{condition}-{arm}.checkpoint.json"
                and result["frozen_checkpoint_file"] == f"finals/{condition}-{arm}.frozen.checkpoint.json",
                "final checkpoint is linked to wrong arm/condition")
        score = inspect_trace(directory, result, schedule, context)
        final = context["files"][result["checkpoint_file"]]
        causal = result["after"]["causal_report"]
        for key in ("factual_transitions", "learned_transitions", "model_bytes"):
            require(final[key] == causal[key], "final checkpoint differs from final runtime counters")
        require(final["model_entries"] == causal["retained_entries"], "final checkpoint entry count differs")
        scored[condition][arm] = score
    acquisition = None
    if suite == "fresh":
        require(summary["acquisition"] is not None, "fresh common acquisition missing")
        require(summary["acquisition"]["after"] == common, "common state differs from acquisition endpoint")
        acquisition = inspect_trace(directory, summary["acquisition"], schedule, context)
    else:
        require(summary["acquisition"] is None, "historical continuation unexpectedly acquired fresh experience before cloning")
    return {"suite": suite, "world": world, "valid": True, "errors": [], "process": process,
            "schedule_sha256": sha256(directory / "schedule.json"), "summary_sha256": sha256(directory / "summary.json"),
            "checkpoint_integrity": context, "acquisition": acquisition, "conditions": scored}


def aggregate_worlds(worlds):
    aggregate = {}
    for condition in CONDITIONS:
        aggregate[condition] = {}
        for arm in ARMS:
            values = [w["conditions"][condition][arm] for w in worlds if w["valid"]]
            if not values:
                aggregate[condition][arm] = None
                continue
            result = {"worlds_observed": len(values)}
            for key in ("episodes", "successes", "actions", "sensor_queries", "noise_queries", "terminal_actions",
                        "terminal_failures", "timeouts", "prequential_nll_sum", "prequential_nll_count",
                        "computation_work", "readonly_belief_work", "inference_work_charged_upper_bound",
                        "inferred_context_switches", "evicted_entries", "wall_seconds"):
                result[key] = sum(v[key] for v in values)
            result["success_rate"] = result["successes"] / result["episodes"]
            result["mean_per_world_success_rate"] = statistics.mean(v["success_rate"] for v in values)
            result["prequential_nll_mean"] = result["prequential_nll_sum"] / result["prequential_nll_count"]
            result["model_bytes_total"] = sum(v["model_bytes"] for v in values)
            result["model_entries_total"] = sum(v["model_entries"] for v in values)
            result["peak_model_bytes_any_world"] = max(v["peak_model_bytes"] for v in values)
            result["peak_model_entries_any_world"] = max(v["peak_model_entries"] for v in values)
            for window in ("first32_after_reversals", "last32_of_segments"):
                result[window] = {key: sum(v[window][key] for v in values)
                                  for key in ("episodes", "successes", "actions", "sensor_queries", "noise_queries")}
                item = result[window]
                item["success_rate"] = item["successes"] / item["episodes"] if item["episodes"] else None
            aggregate[condition][arm] = result
    return aggregate


def comparison(worlds, first, second, condition="switching", qualification=True):
    pairs = []
    for world in worlds:
        if not world["valid"]:
            continue
        a, b = (world["conditions"][condition][arm] for arm in (first, second))
        pairs.append({"world": world["world"], "first_success_rate": a["success_rate"],
                      "second_success_rate": b["success_rate"], "difference": a["success_rate"] - b["success_rate"],
                      "first_actions": a["actions"], "second_actions": b["actions"]})
    wins = sum(p["difference"] > 0 for p in pairs)
    losses = sum(p["difference"] < 0 for p in pairs)
    ties = len(pairs) - wins - losses
    nonzero = wins + losses
    one_sided = sum(math.comb(nonzero, k) for k in range(wins, nonzero + 1)) / 2 ** nonzero if nonzero else 1.0
    lower = sum(math.comb(nonzero, k) for k in range(wins + 1)) / 2 ** nonzero if nonzero else 1.0
    differences = [p["difference"] for p in pairs]
    mean = statistics.mean(differences) if differences else None
    complete = len(pairs) == SUITES["fresh"]
    passed = complete and mean >= .05 and wins >= 13 if qualification else None
    return {"first": first, "second": second, "condition": condition, "worlds_observed": len(pairs),
            "mean_paired_gain": mean, "wins": wins, "losses": losses, "ties": ties,
            "paired_difference_min": min(differences) if differences else None,
            "paired_difference_max": max(differences) if differences else None,
            "paired_difference_standard_error": statistics.stdev(differences) / math.sqrt(len(differences)) if len(differences) > 1 else None,
            "exact_sign_test_one_sided_p": one_sided, "exact_sign_test_two_sided_p": min(1.0, 2 * min(one_sided, lower)),
            "sign_test_non_tied_worlds": nonzero, "sign_test_is_descriptive": True,
            "gate": {"enabled": qualification, "complete_16_worlds": complete if qualification else None,
                     "minimum_mean_gain": .05 if qualification else None, "minimum_strict_world_wins": 13 if qualification else None,
                     "passed": passed}, "pairs": pairs}


def comparison_set(worlds, qualification):
    pairs = (("ordinary_vs_baseline", "ordinary", "baseline"), ("phase_vs_baseline", "phase", "baseline"),
             ("phase_vs_ordinary", "phase", "ordinary"), ("phase_vs_scrambled", "phase", "phase_scrambled"),
             ("phase_vs_uncoupled", "phase", "phase_uncoupled"))
    values = {name: comparison(worlds, a, b, qualification=qualification) for name, a, b in pairs}
    values["stationary_phase_vs_ordinary"] = comparison(worlds, "phase", "ordinary", "stationary", qualification=False)
    return values


def conclusions(comparisons, aggregate, valid):
    ordinary = comparisons["ordinary_vs_baseline"]["gate"]["passed"] is True
    phase_comparisons = all(comparisons[key]["gate"]["passed"] is True
                            for key in ("phase_vs_baseline", "phase_vs_ordinary", "phase_vs_scrambled"))
    stationary_gain = comparisons["stationary_phase_vs_ordinary"]["mean_paired_gain"]
    stationary_ok = stationary_gain is not None and stationary_gain >= -.02
    a, b = aggregate["switching"]["phase"], aggregate["switching"]["ordinary"]
    action_ratio = a["actions"] / b["actions"] if a and b and b["actions"] else None
    action_ok = action_ratio is not None and action_ratio <= 1.10
    return {"ordinary_context_benefit": {"supported": valid and ordinary, "improvement_gate": ordinary},
            "phase_benefit": {"supported": valid and phase_comparisons and stationary_ok and action_ok,
                              "three_comparison_gates": phase_comparisons,
                              "stationary_mean_gain": stationary_gain, "stationary_gain_minimum": -.02,
                              "stationary_guard_passed": stationary_ok, "switching_action_ratio": action_ratio,
                              "maximum_action_ratio": 1.10, "action_guard_passed": action_ok},
            "coupling_contribution": {"supported": valid and comparisons["phase_vs_uncoupled"]["gate"]["passed"] is True,
                                      "improvement_gate": comparisons["phase_vs_uncoupled"]["gate"]["passed"]}}


def write_report(output, report):
    write_json(output / "verdict.json", report)
    lines = ["# Endogenous context and phase comparison", "", f"**Overall phase-benefit verdict: {report['verdict']}**", "",
             "The overall PASS requires the preregistered practical phase-benefit gate. Ordinary context and coupling are separate conclusions.", "",
             "Earlier results remain unchanged: original integral FAIL (4/8), retained repeat FAIL (0/8 new complete chains).", "",
             "## Separate conclusions", "", "| Claim | Supported in this study |", "|---|---|"]
    for key, value in report["conclusions"].items():
        lines.append(f"| {key} | {value['supported']} |")
    for suite, data in report["suites"].items():
        lines += ["", f"## {suite}: {data['valid_worlds']}/{SUITES[suite]} valid worlds", "",
                  "Fresh worlds qualify only within one binary sensor/actuator task family. Retained worlds are descriptive continued life.", "",
                  "| Arm | Switching successes | Whole success % | First32 after reversal % | Stationary % | Actions | Inferred switches |",
                  "|---|---:|---:|---:|---:|---:|---:|"]
        for arm in ARMS:
            switching = data["aggregate"]["switching"][arm]
            stationary = data["aggregate"]["stationary"][arm]
            if switching is None:
                lines.append(f"| {arm} | missing | — | — | — | — | — |")
                continue
            early = switching["first32_after_reversals"]["success_rate"]
            lines.append(f"| {arm} | {switching['successes']}/{switching['episodes']} | {100*switching['success_rate']:.2f} | "
                         f"{100*early:.2f} | {100*stationary['success_rate']:.2f} | {switching['actions']} | {switching['inferred_context_switches']} |")
        lines += ["", "| Paired switching comparison | Mean gain, pp | Wins/losses/ties | Exact one-sided sign p | Fixed gate |",
                  "|---|---:|---:|---:|---|"]
        for name, item in data["comparisons"].items():
            if item["condition"] != "switching":
                continue
            gain = f"{100*item['mean_paired_gain']:+.2f}" if item["mean_paired_gain"] is not None else "missing"
            lines.append(f"| {name} | {gain} | {item['wins']}/{item['losses']}/{item['ties']} | "
                         f"{item['exact_sign_test_one_sided_p']:.6g} | {item['gate']['passed'] if suite == 'fresh' else 'descriptive only'} |")
        lines += ["", "### Individual switching success rates", "",
                  "| World | Valid | Baseline | Ordinary | Phase | Scrambled | Uncoupled |", "|---|---|---:|---:|---:|---:|---:|"]
        for world in data["worlds"]:
            rates = [f"{100*world['conditions']['switching'][arm]['success_rate']:.2f}%" for arm in ARMS] if world["valid"] else ["INVALID"] * 5
            lines.append(f"| {world['world']} | {world['valid']} | " + " | ".join(rates) + " |")
    lines += ["", "## Interpretation and limits", "",
              "- Source and parameters were frozen before behavioral execution; all seeds, arms and negative results remain included.",
              "- Both context routers retain two banks inside one shared memory ceiling. No hidden change label or investigation schedule reaches the actor.",
              "- All continuation and stationary episodes allow learning. These are adaptive behavior comparisons, not new frozen held-out qualification.",
              "- Phase disruption alone does not prove superiority; the ordinary comparator, stationary guard and action-cost guard are required.",
              "- The unchanged baseline already uses phase-native planning. This study isolates the added context inference path.",
              "- `inference_work_charged_upper_bound` is a conservative fixed accounting allowance, not measured arithmetic or CPU time.",
              "- Actual elapsed seconds are recorded separately. Equal resource limits do not establish equal computation or an energy advantage.",
              "- Predictive log loss is prequential on each policy's selected actions; it is not a common-action held-out forecast test.",
              "- Exact sign-test probabilities use paired worlds, excluding ties, and are descriptive; episodes are not independent replicates.",
              "- Missing/malformed/process-timeout evidence is INVALID and can never produce a PASS.",
              "- Checkpoint roundtrips were executed by the evaluator; score-only independently checks their saved fingerprints, ancestor equivalence and state/counter bounds."]
    if report["errors"]:
        lines += ["", "## Integrity errors", ""] + [f"- {error}" for error in report["errors"]]
    for suite, data in report["suites"].items():
        for world in data["worlds"]:
            if world["errors"]:
                lines += [f"- {suite} world {world['world']}: {'; '.join(world['errors'])}"]
    lines += ["", "`verdict.json` contains full paired metrics, diagnostics, per-world state and trace hashes, integrity details and all comparison gates."]
    (output / "RESULT.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def score_all(output, current_provenance, manifest):
    execution = read_json(output / "execution.json")
    errors = []
    if execution["source_sha256"] != current_provenance["source_sha256"]:
        errors.append("current source hashes differ from frozen execution")
    legacy = verify_legacy(output)
    if not legacy["valid"] or legacy != read_json(output / "legacy-gate.json"):
        errors.append("legacy baseline gate is invalid or differs from its sealed record")
    try:
        require(retained_inputs(output, extract=False) == manifest, "retained input manifest changed")
        processes = read_json(output / "processes.json")
        require(len(processes) == sum(SUITES.values())
                and {(r["suite"], r["world"]) for r in processes} == {(s, w) for s, n in SUITES.items() for w in range(n)},
                "process manifest missing or duplicate worlds")
        for record in processes:
            require(record == read_json(output / record["suite"] / f"world-{record['world']}.process.json"),
                    "process manifest differs from saved process record")
    except Exception as exc:
        errors.append(str(exc))
    suites = {}
    for suite, count in SUITES.items():
        worlds = []
        for world in range(count):
            try:
                item = inspect_world(output, suite, world, execution["seed"])
            except Exception as exc:
                item = {"suite": suite, "world": world, "valid": False, "errors": [str(exc)]}
            worlds.append(item)
            print(f"PHASE_SCORE suite={suite} world={world} valid={item['valid']}" +
                  (f" errors={item['errors']}" if item["errors"] else ""), flush=True)
        suites[suite] = {"worlds": worlds, "valid_worlds": sum(w["valid"] for w in worlds),
                         "aggregate": aggregate_worlds(worlds), "comparisons": comparison_set(worlds, suite == "fresh")}
    valid = not errors and all(data["valid_worlds"] == SUITES[suite] for suite, data in suites.items())
    conclusions_value = conclusions(suites["fresh"]["comparisons"], suites["fresh"]["aggregate"], valid)
    verdict = "INVALID" if not valid else "PASS" if conclusions_value["phase_benefit"]["supported"] else "VALID_NEGATIVE"
    report = {"schema": "aeterna.integral-phase-context.verdict.v1", "verdict": verdict,
              "overall_verdict_means": "preregistered practical benefit of the phase context variant",
              "valid": valid, "execution": execution, "legacy_gate": legacy,
              "retained_input_manifest": manifest, "suites": suites, "conclusions": conclusions_value,
              "thresholds": {"primary_worlds": 16, "minimum_mean_gain": .05, "minimum_strict_wins": 13,
                             "stationary_minimum_gain": -.02, "maximum_phase_to_ordinary_action_ratio": 1.10},
              "historical_verdicts_unchanged": {"original": "FAIL, 4/8", "retained_repeat": "FAIL, 0/8"}, "errors": errors}
    write_report(output, report)
    print(json.dumps({"verdict": verdict, "valid": valid, "conclusions": conclusions_value,
                      "valid_worlds": {s: d["valid_worlds"] for s, d in suites.items()}, "errors": errors}), flush=True)
    return 0 if verdict == "PASS" else 1 if verdict == "VALID_NEGATIVE" else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--legacy-binary", type=Path)
    parser.add_argument("--run-id", help="Frozen new world and continuation seed; normally the GitHub run ID")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=2, choices=(1, 2))
    parser.add_argument("--score-only", action="store_true", help="Re-read and score saved evidence without executing any actor")
    args = parser.parse_args()
    if not args.score_only and (args.binary is None or args.legacy_binary is None or args.run_id is None):
        parser.error("execution requires --binary, --legacy-binary and --run-id")
    output = args.output.resolve()
    current_provenance = provenance()
    if args.score_only:
        execution = read_json(output / "execution.json")
        for path, key in ((args.binary, "binary_sha256"), (args.legacy_binary, "legacy_binary_sha256")):
            if path is not None:
                require(sha256(path) == execution[key], f"provided {key} differs from recorded executable")
        manifest = retained_inputs(output, extract=False)
    else:
        seed = int(args.run_id)
        require(0 < seed < 2**64, "run ID must be a positive u64")
        binary, legacy_binary = args.binary.resolve(strict=True), args.legacy_binary.resolve(strict=True)
        output.mkdir(parents=True, exist_ok=False)
        execution = {**current_provenance, "run_id": args.run_id, "seed": seed,
                     "binary_sha256": sha256(binary), "legacy_binary_sha256": sha256(legacy_binary),
                     "workers": args.workers, "timeout_seconds": TIMEOUT_SECONDS,
                     "retained_archive_sha256": RETAINED_ARCHIVE_SHA256,
                     "original_archive_sha256": ORIGINAL_ARCHIVE_SHA256,
                     "evidence_origin": str(output), "started_utc_epoch": time.time()}
        write_json(output / "execution.json", execution)
        manifest = retained_inputs(output, extract=True)
        gate = legacy_replay(output, legacy_binary)
        if gate["valid"]:
            run_worlds(output, binary, seed, args.workers)
        else:
            # No new actor, not even a fresh acquisition, runs after a failed gate.
            write_json(output / "processes.json", [])
        require(sha256(binary) == execution["binary_sha256"]
                and sha256(legacy_binary) == execution["legacy_binary_sha256"], "executable changed during execution")
    return score_all(output, provenance(), manifest)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        # Infrastructure/source failures never turn into a capability failure
        # or a successful qualification. The durable evaluator/process logs
        # remain available even if malformed evidence prevents report creation.
        print(json.dumps({"verdict": "INVALID", "valid": False, "error": str(exc)}), file=sys.stderr, flush=True)
        sys.exit(2)
