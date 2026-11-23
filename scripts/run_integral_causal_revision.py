#!/usr/bin/env python3
"""Frozen, bounded engineering trial. Capability FAIL is not a harness PASS.

The evaluator owns the world and observes decisions; production owns every
non-random action. This scorer never supplies advice or changes the actor.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
import time


WORLD_COUNT = 8
PHASE_EPISODES = {
    "acquire": 256,
    "original_held": 64,
    "adaptive": 512,
    "frozen": 512,
    "recovered_held": 64,
    "shuffled_cue_held": 64,
    "stale_held": 64,
    "random_held": 64,
}
PHASE_SCHEDULES = {
    "acquire": "acquisition", "original_held": "original_validation",
    "adaptive": "reversal", "frozen": "reversal",
    "recovered_held": "recovered_validation", "shuffled_cue_held": "recovered_validation",
    "stale_held": "recovered_validation", "random_held": "recovered_validation",
}
THRESHOLDS = {
    "aggregate_original_success": 0.80,
    "aggregate_recovered_success": 0.80,
    "per_world_original_success": 0.75,
    "per_world_recovered_success": 0.75,
    "minimum_complete_worlds": 6,
    "minimum_control_gap": 0.20,
    "contradiction_min_support": 3,
    "contradiction_min_predicted_success": 0.75,
    "revised_min_failure_probability": 0.60,
    "minimum_prediction_shift": 0.25,
    "minimum_matched_held_episodes": 3,
}


def read_json(path: Path):
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def write_json(path: Path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def probability(belief, post) -> float:
    return sum(float(p) for observation, p in belief["outcomes"] if observation == post)


def context_key(row, action):
    # The production hypotheses use PRE/action histories up to depth three.
    # Exact matching prevents unrelated observations from becoming a revision.
    return json.dumps([row["pre"], row["history"], action], separators=(",", ":"), sort_keys=True)


def location(row):
    return {k: row[k] for k in ("world", "phase", "episode", "step", "event", "action", "role", "source")}


def valid_integer(value, minimum=0):
    return type(value) is int and value >= minimum


def valid_probability(value):
    # Permit floating-point roundoff, never substantive out-of-range mass.
    return type(value) in (int, float) and math.isfinite(value) and -1e-12 <= value <= 1 + 1e-12


def valid_observation(value):
    return isinstance(value, list) and len(value) == 2 and all(type(v) is int for v in value)


MASK64 = 2**64 - 1


def mix64(value):
    value = (value + 0x9E3779B97F4A7C15) & MASK64
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & MASK64
    return value ^ (value >> 31)


def shuffled_values(values, state):
    result = list(values)
    for index in range(len(result) - 1, 0, -1):
        state = (state + 0x9E3779B97F4A7C15) & MASK64
        other = mix64(state) % (index + 1)
        result[index], result[other] = result[other], result[index]
    return result


def world_manifest(seed, world):
    """Reconstruct exogenous data only; this never runs or advises an actor."""
    base = mix64(seed ^ ((world * 0xD1342543DE82EF95) & MASK64))
    roles = dict(zip(("informative_sensor", "noise_sensor", "actuator0", "actuator1"),
                     shuffled_values(range(4), base ^ 0x6D6F746F725F6D61)))
    codes = dict(zip(("start", "sense0", "sense1", "noise0", "noise1", "success", "failure"),
                     shuffled_values((101, 211, 307, 419, 503, 617, 733), base ^ 0x6F62735F636F6465)))
    salts = {"acquisition": 0x6163717569736974, "original_validation": 0x6F726967696E616C,
             "reversal": 0x726576657273616C, "recovered_validation": 0x7265636F76657265}
    keys = {phase: mix64(base ^ salts[schedule]) for phase, schedule in PHASE_SCHEDULES.items()}
    hidden = {phase: [mix64(key ^ 0x68696464656E5F62 ^ ((ep * 0xA24BAED4963EE407) & MASK64)) & 1
                       for ep in range(PHASE_EPISODES[phase])] for phase, key in keys.items()}
    cues = {phase: list(values) for phase, values in hidden.items()}
    cues["shuffled_cue_held"] = shuffled_values(cues["shuffled_cue_held"],
                                              keys["shuffled_cue_held"] ^ 0x73687566666C6564)
    return roles, codes, keys, hidden, cues


def score_world(summary: dict, trace_path: Path) -> dict:
    phase_map = {p["phase"]: p for p in summary["phases"]}
    errors = []
    if len(summary["phases"]) != len(PHASE_EPISODES) or set(phase_map) != set(PHASE_EPISODES):
        errors.append("phase set differs from frozen protocol")
    for name, episodes in PHASE_EPISODES.items():
        if name not in phase_map or phase_map[name]["episodes"] != episodes:
            errors.append(f"wrong episode count: {name}")
    integrity = summary.get("integrity", {})
    if not integrity or any(value is not True for value in integrity.values()):
        errors.append("evaluator integrity checks not all true")

    codes = summary["observation_codes"]
    roles = summary["motor_roles"]
    manifest_roles, manifest_codes, schedule_keys, hidden_schedule, cue_schedule = world_manifest(summary["seed"], summary["world"])
    if roles != manifest_roles or codes != manifest_codes:
        errors.append("world identifiers differ from the frozen seeded generator")
    if (set(roles) != {"informative_sensor", "noise_sensor", "actuator0", "actuator1"}
            or set(codes) != {"start", "sense0", "sense1", "noise0", "noise1", "success", "failure"}
            or sorted(roles.values()) != list(range(4)) or len(set(codes.values())) != 7):
        errors.append("world identifiers are not independent unique permutations")
    success_post, failure_post = [0, codes["success"]], [0, codes["failure"]]
    expected = {name: {"actions": 0, "successes": 0, "sensor_queries": 0, "noise_queries": 0,
                       "terminal_failures": 0, "timeouts": 0} for name in PHASE_EPISODES}
    terminal_by_episode = set()
    previous_by_episode = {}
    history_by_episode = {}
    trace_episode_results = {}
    episode_action_counts = {}
    source_counts = {}
    contradictions = []
    useful_after_error = []
    matched_revisions = {}
    held_success_after_sensor = set()
    episode_sensed = set()
    before_error = None
    previous_sequence = None
    previous_permit = {}
    phase_order = {name: i for i, name in enumerate(PHASE_EPISODES)}

    for name, phase in phase_map.items():
        wanted_rule = int(name not in ("acquire", "original_held"))
        wanted_frozen = name not in ("acquire", "adaptive", "random_held")
        if (phase["rule"] != wanted_rule or phase["cue_shuffled"] is not (name == "shuffled_cue_held")
                or phase["frozen"] is not wanted_frozen or phase["schedule"] != PHASE_SCHEDULES[name]):
            errors.append(f"phase conditions differ from frozen protocol: {name}")

    with trace_path.open(encoding="utf-8") as f:
        for line_number, line in enumerate(f, 1):
            row = json.loads(line)
            phase = row["phase"]
            if phase not in PHASE_EPISODES:
                errors.append(f"unexpected trace phase at line {line_number}")
                continue
            if row["world"] != summary["world"] or row["seed"] != summary["seed"]:
                errors.append(f"trace identity mismatch at line {line_number}")
            action, role = row["action"], row["role"]
            if (type(action) is not int or action not in range(4)
                    or role not in roles or roles[role] != action or row["cost"] != 1):
                errors.append(f"invalid action/role/cost at line {line_number}")
            ep_key = (phase, row["episode"])
            if (not valid_integer(row["episode"]) or row["episode"] >= PHASE_EPISODES[phase]
                    or not valid_integer(row["step"])):
                errors.append(f"invalid episode or step index at line {line_number}")
                continue
            sequence = (phase_order[phase], row["episode"], row["step"])
            if previous_sequence is not None and sequence <= previous_sequence:
                errors.append(f"trace is not chronological at line {line_number}")
            previous_sequence = sequence
            if (row["rule"] != int(phase not in ("acquire", "original_held"))
                    or type(row["hidden"]) is not int or row["hidden"] not in (0, 1)
                    or type(row["sensor_cue"]) is not int or row["sensor_cue"] not in (0, 1)
                    or row["cue_shuffled"] is not (phase == "shuffled_cue_held")):
                errors.append(f"invalid hidden world conditions at line {line_number}")
            if phase != "shuffled_cue_held" and row["sensor_cue"] != row["hidden"]:
                errors.append(f"unshuffled sensor does not report the hidden bit at line {line_number}")
            if (row["hidden"] != hidden_schedule[phase][row["episode"]]
                    or row["sensor_cue"] != cue_schedule[phase][row["episode"]]):
                errors.append(f"latent state differs from the frozen seeded schedule at line {line_number}")
            if not valid_observation(row["pre"]) or not valid_observation(row["post"]):
                errors.append(f"invalid observation shape at line {line_number}")
            count = episode_action_counts.get(ep_key, 0)
            if row["step"] != count or count >= 4 or ep_key in terminal_by_episode:
                errors.append(f"invalid action sequence at line {line_number}")
            if row["pre"] != previous_by_episode.get(ep_key, [0, codes["start"]]):
                errors.append(f"non-factual PRE at line {line_number}")
            factual_history = history_by_episode.setdefault(ep_key, [])
            if row["history"] != factual_history[-3:]:
                errors.append(f"history is not the factual episode prefix at line {line_number}")
            factual_history.append({"pre": row["pre"], "action": action, "post": row["post"]})
            episode_action_counts[ep_key] = count + 1
            previous_by_episode[ep_key] = row["post"]
            expected[phase]["actions"] += 1
            phase_before = phase_map[phase]["before"]
            initial_event = phase_before["factual_events"] if phase_before is not None else 0
            if row["event"] != initial_event + expected[phase]["actions"]:
                errors.append(f"event count is not chronological at line {line_number}")
            if phase != "random_held":
                permit = row["permit_sequence"]
                if not valid_integer(permit) or (phase in previous_permit and permit <= previous_permit[phase]):
                    errors.append(f"reused or invalid execution permit at line {line_number}")
                previous_permit[phase] = permit
                if phase_map[phase]["frozen"] and (
                        row["learned_fingerprint_before"] != row["learned_fingerprint_after"]
                        or row["learned_fingerprint_after"] != phase_before["causal_report"]["learned_fingerprint"]):
                    errors.append(f"frozen causal knowledge changed at line {line_number}")
            source_counts.setdefault(phase, {}).setdefault(row["source"], 0)
            source_counts[phase][row["source"]] += 1

            terminal = role.startswith("actuator")
            actual_success = terminal and int(role[-1]) == (row["hidden"] ^ row["rule"])
            if row["success"] is not actual_success or row["terminal"] is not terminal:
                errors.append(f"world outcome mismatch at line {line_number}")
            if terminal:
                terminal_by_episode.add(ep_key)
                if row["post"] != (success_post if actual_success else failure_post):
                    errors.append(f"terminal POST mismatch at line {line_number}")
                expected[phase]["terminal_failures"] += int(not actual_success)
            elif role == "informative_sensor":
                expected[phase]["sensor_queries"] += 1
                episode_sensed.add(ep_key)
                if row["post"] != [0, codes[f"sense{row['sensor_cue']}"]]:
                    errors.append(f"sensor POST mismatch at line {line_number}")
            elif role == "noise_sensor":
                expected[phase]["noise_queries"] += 1
                noise_bit = mix64(schedule_keys[phase] ^ 0x6E6F6973655F6269
                                  ^ ((row["episode"] * 0x9FB21C651E98DF25) & MASK64)
                                  ^ ((row["step"] * 0xC13FA9A902A6328F) & MASK64)) & 1
                if row["post"] != [0, codes[f"noise{noise_bit}"]]:
                    errors.append(f"noise POST mismatch at line {line_number}")
            if row["reward"] != float(actual_success):
                errors.append(f"non-factual reward at line {line_number}")
            expected[phase]["successes"] += int(actual_success)

            episode_result = trace_episode_results.setdefault(ep_key, {
                "episode": row["episode"], "hidden": row["hidden"], "sensor_cue": row["sensor_cue"],
                "success": False, "terminal": False, "actions": 0, "sensor_queries": 0, "noise_queries": 0})
            if episode_result["hidden"] != row["hidden"] or episode_result["sensor_cue"] != row["sensor_cue"]:
                errors.append(f"latent state changed inside an episode at line {line_number}")
            episode_result["success"] = actual_success
            episode_result["terminal"] = terminal
            episode_result["actions"] += 1
            episode_result["sensor_queries"] += int(role == "informative_sensor")
            episode_result["noise_queries"] += int(role == "noise_sensor")

            beliefs = {b["action"]: b for b in row["pre_beliefs"]}
            if phase != "random_held" and (len(row["pre_beliefs"]) != 4 or set(beliefs) != set(range(4))):
                errors.append(f"missing prospective beliefs at line {line_number}")
            for belief in beliefs.values():
                outcomes = belief["outcomes"]
                if (not valid_integer(belief["support"]) or not valid_probability(belief["unknown_probability"])
                        or not all(valid_observation(post) and valid_probability(p) for post, p in outcomes)
                        or len({tuple(post) for post, _ in outcomes}) != len(outcomes)
                        or abs(sum(float(p) for _, p in outcomes) + belief["unknown_probability"] - 1.0) > 1e-6
                        or not math.isfinite(belief["information_gain"]) or belief["information_gain"] < 0):
                    errors.append(f"invalid prospective probability distribution at line {line_number}")
            if phase == "random_held" and row["source"] != "uniform-random":
                errors.append(f"random actor mislabeled at line {line_number}")
            if phase == "random_held" and row["pre_beliefs"]:
                errors.append(f"random actor has model forecasts at line {line_number}")
            if phase == "random_held":
                expected_random = mix64(schedule_keys[phase] ^ 0x72616E645F616374
                                        ^ ((row["episode"] * 0x91E10DA5C79E7B1D) & MASK64)
                                        ^ ((row["step"] * 0xD1B54A32D192ED03) & MASK64)) % 4
                if row["action"] != expected_random:
                    errors.append(f"random action differs from independent seeded policy at line {line_number}")

            # First actual failure of a previously supported positive forecast.
            if phase == "adaptive" and terminal and not actual_success:
                belief = beliefs[action]
                p_success = probability(belief, success_post)
                if (belief["support"] >= THRESHOLDS["contradiction_min_support"] and
                        p_success >= THRESHOLDS["contradiction_min_predicted_success"]):
                    witness = {"at": location(row), "key": context_key(row, action),
                               "support": belief["support"], "p_success_before": p_success,
                               "p_failure_before": probability(belief, failure_post),
                               "pre": row["pre"], "history": row["history"]}
                    contradictions.append(witness)
                    if before_error is None:
                        before_error = witness
            if phase == "adaptive" and role == "informative_sensor" and before_error is not None:
                useful_after_error.append(location(row))
            if phase == "recovered_held":
                if actual_success and ep_key in episode_sensed:
                    held_success_after_sensor.add(row["episode"])
                for action, belief in beliefs.items():
                    key = context_key(row, action)
                    for witness in contradictions:
                        if witness["key"] != key:
                            continue
                        p_failure = probability(belief, failure_post)
                        shift = p_failure - witness["p_failure_before"]
                        if (p_failure >= THRESHOLDS["revised_min_failure_probability"] and
                                shift >= THRESHOLDS["minimum_prediction_shift"]):
                            group = matched_revisions.setdefault(key, {"before": witness, "episodes": {}, "samples": []})
                            group["episodes"][row["episode"]] = True
                            if len(group["samples"]) < 3:
                                group["samples"].append({"at": location(row), "old_action": action,
                                                         "p_failure_after": p_failure, "shift": shift})
                            break

    for (name, episode), result in trace_episode_results.items():
        if not result["terminal"]:
            expected[name]["timeouts"] += 1
            if result["actions"] != 4:
                errors.append(f"episode stopped before terminal or action limit: {name}.{episode}")

    for name, phase in phase_map.items():
        for field, measured in expected[name].items():
            if phase[field] != measured:
                errors.append(f"summary differs from trace: {name}.{field}")
        if {ep for p, ep in episode_action_counts if p == name} != set(range(PHASE_EPISODES[name])):
            errors.append(f"not every episode has a trace: {name}")
        if phase["successes"] + phase["terminal_failures"] + phase["timeouts"] != phase["episodes"]:
            errors.append(f"episode accounting mismatch: {name}")
        if phase["source_counts"] != source_counts.get(name, {}):
            errors.append(f"source accounting mismatch: {name}")
        if phase["physical_callbacks"] != expected[name]["actions"]:
            errors.append(f"callback accounting mismatch: {name}")
        traced_episodes = [trace_episode_results[(name, ep)] for ep in range(PHASE_EPISODES[name])
                           if (name, ep) in trace_episode_results]
        if phase["episode_results"] != traced_episodes:
            errors.append(f"episode results differ from factual trace: {name}")
        if name != "random_held":
            before, after = phase["before"], phase["after"]
            for field in ("factual_events", "total_external_cost"):
                if after[field] - before[field] != expected[name]["actions"]:
                    errors.append(f"runtime accounting differs from factual trace: {name}.{field}")
            for field in ("factual_transitions", "learned_transitions"):
                wanted_delta = 0 if field == "learned_transitions" and phase["frozen"] else expected[name]["actions"]
                if after["causal_report"][field] - before["causal_report"][field] != wanted_delta:
                    errors.append(f"causal accounting differs from factual trace: {name}.{field}")
            if phase["frozen"]:
                if (before["causal_report"]["learned_fingerprint"] != after["causal_report"]["learned_fingerprint"]
                        or before["native_weights"] != after["native_weights"]
                        or before["native_meta_observations"] != after["native_meta_observations"]):
                    errors.append(f"frozen knowledge/policy changed: {name}")

    # Confirm the exogenous episode schedules are paired across independent
    # actor lineages, and cue shuffling preserves only the cue marginal.
    for names in (("adaptive", "frozen"),
                  ("recovered_held", "shuffled_cue_held", "stale_held", "random_held")):
        schedules = [[e["hidden"] for e in phase_map[name]["episode_results"]] for name in names]
        if any(values != schedules[0] for values in schedules[1:]):
            errors.append(f"paired hidden schedules differ: {','.join(names)}")
    held_hidden = [e["hidden"] for e in phase_map["recovered_held"]["episode_results"]]
    shuffled_cues = [e["sensor_cue"] for e in phase_map["shuffled_cue_held"]["episode_results"]]
    if sorted(held_hidden) != sorted(shuffled_cues):
        errors.append("shuffled cue marginal differs from the paired validation")

    rates = {name: phase["successes"] / phase["episodes"] for name, phase in phase_map.items()}
    corrections = [value for value in matched_revisions.values()
                   if len(value["episodes"]) >= THRESHOLDS["minimum_matched_held_episodes"]]
    for value in corrections:
        value["matched_episodes"] = sorted(value.pop("episodes"))
        # Link inquiry to THIS violated prediction, not to an unrelated earlier
        # error. Every corrected forecast is from the later held-out phase.
        value["information_after_this_contradiction"] = next(
            (at for at in useful_after_error if at["event"] > value["before"]["at"]["event"]), None)
        value["before"].pop("key", None)
    ordered_corrections = [value for value in corrections if value["information_after_this_contradiction"]]
    stages = {
        "acquired": rates.get("original_held", 0) >= THRESHOLDS["per_world_original_success"],
        "supported_expectation_contradicted": bool(contradictions),
        "self_selected_information_after_contradiction": bool(useful_after_error),
        "same_context_prediction_corrected": bool(corrections),
        "recovered_goal_performance": rates.get("recovered_held", 0) >= THRESHOLDS["per_world_recovered_success"],
        "information_used_in_successful_held_episodes": len(held_success_after_sensor) >= 3,
        "integrity": not errors,
    }
    windows = []
    adaptive = phase_map.get("adaptive", {}).get("episode_results", [])
    for start in range(0, len(adaptive), 64):
        block = adaptive[start:start + 64]
        windows.append({"first_episode": start, "episodes": len(block),
                        "successes": sum(bool(e["success"]) for e in block),
                        "actions": sum(e["actions"] for e in block)})
    if before_error:
        before_error.pop("key", None)
    return {
        "world": summary["world"], "seed": summary["seed"], "rates": rates,
        "phase_totals": expected, "stages": stages,
        "complete_loop": all(stages.values()) and bool(ordered_corrections),
        "chronological_information_revision_chains": len(ordered_corrections),
        "contradiction_count": len(contradictions), "first_contradiction": before_error,
        "informative_actions_after_contradiction": len(useful_after_error),
        "first_information_action_after_contradiction": useful_after_error[0] if useful_after_error else None,
        "corrected_contexts": (ordered_corrections + [c for c in corrections if c not in ordered_corrections])[:4],
        "successful_held_episodes_after_sensing": len(held_success_after_sensor),
        "adaptive_windows": windows, "errors": errors[:40],
        "source_counts": source_counts,
    }


def score_all(output: Path, seed: int, binary_hash: str | None):
    worlds, invalid = [], []
    try:
        execution = read_json(output / "execution.json")
        if execution["seed"] != seed or execution["binary_sha256"] != binary_hash:
            raise ValueError("execution manifest identity mismatch")
        processes = read_json(output / "processes.json")
        if (len(processes) != WORLD_COUNT or {p["world"] for p in processes} != set(range(WORLD_COUNT))
                or any(type(p["exit"]) is not int or p["exit"] != 0 for p in processes)):
            raise ValueError("not all eight evaluator processes exited successfully")
    except Exception as exc:
        invalid.append({"world": None, "error": f"execution integrity: {exc}"})
    for world in range(WORLD_COUNT):
        try:
            summary = read_json(output / f"world-{world}.json")
            if summary["world"] != world or summary["seed"] != seed:
                raise ValueError("summary identity mismatch")
            worlds.append(score_world(summary, output / f"world-{world}.jsonl"))
        except Exception as exc:
            invalid.append({"world": world, "error": str(exc)})

    aggregate = {}
    for name, episodes in PHASE_EPISODES.items():
        totals = {key: sum(w["phase_totals"][name][key] for w in worlds)
                  for key in ("actions", "successes", "sensor_queries", "noise_queries")}
        # Missing or malformed worlds never shrink the announced denominator.
        # The verdict is INVALID, and reported rates remain conservative.
        totals["episodes"] = WORLD_COUNT * episodes
        totals["observed_episodes"] = len(worlds) * episodes
        totals["success_rate"] = totals["successes"] / totals["episodes"] if totals["episodes"] else 0
        aggregate[name] = totals
    complete = sum(w["complete_loop"] for w in worlds)
    rate = lambda name: aggregate[name]["success_rate"]
    checks = {
        "all_eight_worlds_valid": len(worlds) == WORLD_COUNT and not invalid and all(not w["errors"] for w in worlds),
        "acquisition_at_least_80_percent": rate("original_held") >= THRESHOLDS["aggregate_original_success"],
        "recovery_at_least_80_percent": rate("recovered_held") >= THRESHOLDS["aggregate_recovered_success"],
        "complete_loop_in_at_least_six_worlds": complete >= THRESHOLDS["minimum_complete_worlds"],
        "recovery_exceeds_frozen_by_20_points": rate("recovered_held") - rate("stale_held") >= THRESHOLDS["minimum_control_gap"],
        "recovery_exceeds_shuffled_cue_by_20_points": rate("recovered_held") - rate("shuffled_cue_held") >= THRESHOLDS["minimum_control_gap"],
        "recovery_exceeds_random_by_20_points": rate("recovered_held") - rate("random_held") >= THRESHOLDS["minimum_control_gap"],
    }
    verdict = "PASS" if all(checks.values()) else ("FAIL" if checks["all_eight_worlds_valid"] else "INVALID")
    report = {
        "schema": "aeterna.integral-causal-revision.verdict.v1", "verdict": verdict,
        "scope": "Bounded synthetic engineering trial, not general scientific cognition or new causal-theory invention",
        "base_production_commit": "01a35d0c69c3dec0340c753456ada4a7b7f221da",
        "seed": seed, "binary_sha256": binary_hash, "worlds_expected": WORLD_COUNT,
        "worlds_completed": len(worlds), "complete_loop_worlds": complete,
        "thresholds": THRESHOLDS, "checks": checks, "aggregate": aggregate,
        "worlds": worlds, "invalid_worlds": invalid,
        "trace_sha256": {f"world-{w}.jsonl": sha256(output / f"world-{w}.jsonl")
                         for w in range(WORLD_COUNT) if (output / f"world-{w}.jsonl").exists()},
        "interpretation": [
            "Every non-random action belongs to the existing protected unified runtime.",
            "Source labels and useful_experiments counters alone do not establish research usefulness.",
            "Freeze controls causal/model/policy learning broadly, not one isolated mechanism.",
            "History-depth weights are not arbitrary self-invented causal hypotheses.",
            "All worlds, failed prerequisites and contradictory results remain in the denominator.",
        ],
    }
    write_json(output / "verdict.json", report)
    lines = ["# Integral causal revision trial", "", f"**Verdict: {verdict}**", "",
             f"Complete evidence chain: **{complete}/{WORLD_COUNT}** worlds.", "",
             "| Phase | Goals | Episodes | Rate | Actions | Useful sensor | Noise sensor |",
             "|---|---:|---:|---:|---:|---:|---:|"]
    for name, a in aggregate.items():
        lines.append(f"| {name} | {a['successes']} | {a['episodes']} | {a['success_rate']:.1%} | {a['actions']} | {a['sensor_queries']} | {a['noise_queries']} |")
    lines += ["", "## Frozen checks", ""]
    lines += [f"- {'PASS' if passed else 'FAIL'}: {name}" for name, passed in checks.items()]
    lines += ["", "## Per-world evidence", "",
              "| World | Original | Recovered | Contradiction | New sensing | Corrected forecast | Full chain |",
              "|---|---:|---:|---|---|---|---|"]
    for w in worlds:
        stages = w["stages"]
        lines.append(f"| {w['world']} | {w['rates']['original_held']:.1%} | {w['rates']['recovered_held']:.1%} | {stages['supported_expectation_contradicted']} | {stages['self_selected_information_after_contradiction']} | {stages['same_context_prediction_corrected']} | {w['complete_loop']} |")
    lines += ["", "This is an unchanged-production test of one synthetic family. A successful build is not a capability PASS.",
              "Full prospective action/outcome forecasts and factual trajectories are in the accompanying JSONL files."]
    (output / "RESULT.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(json.dumps({"verdict": verdict, "checks": checks, "aggregate": aggregate,
                      "complete_loop_worlds": complete, "invalid": invalid}, ensure_ascii=False), flush=True)
    return 0 if verdict == "PASS" else 1 if verdict == "FAIL" else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--seed", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=2, choices=(1, 2, 4))
    parser.add_argument("--score-only", action="store_true", help="Re-score saved evidence without re-running any world")
    args = parser.parse_args()
    if not (0 < args.seed < 2**64):
        parser.error("seed must be a positive u64")
    output = args.output.resolve()
    if args.score_only:
        try:
            binary_hash = read_json(output / "execution.json").get("binary_sha256")
        except Exception:
            binary_hash = None
        return score_all(output, args.seed, binary_hash)
    if args.binary is None:
        parser.error("--binary is required for execution")
    binary = args.binary.resolve(strict=True)
    # Refuse accidental overwrite or a concealed repeat of the same trial.
    output.mkdir(parents=True, exist_ok=False)
    binary_hash = sha256(binary)
    write_json(output / "execution.json", {"seed": args.seed, "binary_sha256": binary_hash,
                                          "workers": args.workers, "started_utc_epoch": time.time()})

    def run_world(world):
        cmd = [str(binary), "--seed", str(args.seed), "--world", str(world),
               "--trace", str(output / f"world-{world}.jsonl")]
        started = time.monotonic()
        try:
            with (output / f"world-{world}.json").open("w", encoding="utf-8") as out, (output / f"world-{world}.stderr.log").open("w", encoding="utf-8") as err:
                completed = subprocess.run(cmd, stdout=out, stderr=err, check=False)
            result = {"world": world, "exit": completed.returncode}
        except Exception as exc:
            result = {"world": world, "exit": -1, "error": str(exc)}
        print(f"INTEGRAL_WORLD world={world} exit={result['exit']} seconds={time.monotonic()-started:.2f}", flush=True)
        return result

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as executor:
        executions = list(executor.map(run_world, range(WORLD_COUNT)))
    write_json(output / "processes.json", executions)
    return score_all(output, args.seed, binary_hash)


if __name__ == "__main__":
    sys.exit(main())
