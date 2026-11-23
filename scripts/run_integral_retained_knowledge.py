#!/usr/bin/env python3
"""Repeat the known integral scenario from an exactly reconstructed live state.

Historical replay is reconstruction, not fresh qualification. No repeat actor is
launched until all eight original traces and parsed summaries match the archive.
The original scorer and all of its capability thresholds remain unchanged.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import time
import zipfile


ROOT = Path(__file__).resolve().parents[1]
WORLD_COUNT = 8
ORIGINAL_SEED = 45231338992
BASE_COMMIT = "01a35d0c69c3dec0340c753456ada4a7b7f221da"
ORIGINAL_COMMIT = "4b391bf81f1f8203fb3db24e6e1a8fd48f24fc13"
ARCHIVE = ROOT / "docs/experiments/evidence/integral_causal_revision_20261123/integral-causal-evidence-45231338992.zip"
ARCHIVE_SHA256 = "73c4a3f6d1bb27d62c0fdf963479dbe45b489493ebf87ec5022d63fb3796c21e"
FROZEN_FILES = {
    "examples/integral_causal_revision_eval.rs": "f79dc005680a01d4fb011b70f0ac1b127c0d8107b63d8b48f34d0b90d4759c4f",
    "scripts/run_integral_causal_revision.py": "7b2081b82c9c9bd529a1c57e128192d36e88cec95ea5cf4cf40de2fd6f0da514",
    "docs/experiments/INTEGRAL_CAUSAL_REVISION_20261123_PROTOCOL.md": "a699624f9748742e25bc051c5fe32f44a3ddb4185f4aa58e516e837e8f77ffb7",
}
CHECKPOINT_FILES = {
    "initial": "initial.checkpoint.json",
    "acquired": "acquired.checkpoint.json",
    "continuation_unfrozen": "continuation-unfrozen.checkpoint.json",
    "revised_frozen": "revised-frozen.checkpoint.json",
    "stale_frozen": "stale-frozen.checkpoint.json",
}
METADATA_CHECKS = (
    "checkpoint_roundtrips_exact", "continuation_freezes_to_revised_exactly",
    "continuation_boundary_equivalent", "initial_unfrozen",
)


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write_json(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def stream_hash(stream):
    digest = hashlib.sha256()
    for block in iter(lambda: stream.read(1024 * 1024), b""):
        digest.update(block)
    return digest.hexdigest()


def sha256(path):
    with Path(path).open("rb") as stream:
        return stream_hash(stream)


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, check=True,
                          capture_output=True, text=True).stdout.strip()


def provenance():
    sources = set(FROZEN_FILES) | {
        "examples/integral_retained_knowledge_eval.rs",
        "scripts/run_integral_retained_knowledge.py",
    }
    for pattern in ("docs/experiments/*RETAINED*PROTOCOL.md", ".github/workflows/*retained*.yml"):
        sources.update(str(p.relative_to(ROOT)) for p in ROOT.glob(pattern))
    hashes = {name: sha256(ROOT / name) for name in sorted(sources)}
    for name, expected in FROZEN_FILES.items():
        if hashes[name] != expected:
            raise ValueError(f"frozen original source changed: {name}")
    protected = ("src", "tests", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rust-toolchain")
    changed = git("diff", "--name-only", BASE_COMMIT, "--", *protected)
    untracked = git("ls-files", "--others", "--exclude-standard", "--", *protected)
    if changed or untracked:
        raise ValueError(f"production/test/dependency/toolchain tree differs from base: {changed or untracked}")
    # Execution is allowed only from committed, unchanged experiment sources.
    git("ls-files", "--error-unmatch", "--", *sorted(sources))
    if git("diff", "--name-only", "HEAD", "--", *sorted(sources)):
        raise ValueError("experiment sources differ from the committed execution version")
    return {"source_commit": git("rev-parse", "HEAD"), "base_production_commit": BASE_COMMIT,
            "source_sha256": hashes, "protected_tree_matches_base": True}


def load_archive():
    if sha256(ARCHIVE) != ARCHIVE_SHA256:
        raise ValueError("historical archive SHA256 differs from the preserved original")
    with zipfile.ZipFile(ARCHIVE) as archive:
        verdict_bytes = archive.read("integral-evidence/verdict.json")
        verdict = json.loads(verdict_bytes)
        summaries, traces = {}, {}
        for world in range(WORLD_COUNT):
            summaries[world] = json.loads(archive.read(f"integral-evidence/world-{world}.json"))
            with archive.open(f"integral-evidence/world-{world}.jsonl") as stream:
                traces[world] = stream_hash(stream)
            if traces[world] != verdict["trace_sha256"][f"world-{world}.jsonl"]:
                raise ValueError(f"historical internal trace hash mismatch: world {world}")
    if (verdict["seed"] != ORIGINAL_SEED or verdict["verdict"] != "FAIL"
            or verdict["worlds_completed"] != WORLD_COUNT or verdict["complete_loop_worlds"] != 4):
        raise ValueError("archive is not the preserved original eight-world FAIL")
    return {"verdict": verdict, "verdict_bytes": verdict_bytes, "summaries": summaries,
            "trace_sha256": traces}


def load_scorer():
    path = ROOT / "scripts/run_integral_causal_revision.py"
    if sha256(path) != FROZEN_FILES[str(path.relative_to(ROOT))]:
        raise ValueError("refusing to load a modified original scorer")
    spec = importlib.util.spec_from_file_location("frozen_integral_scorer", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def checkpoint_dir(output, mode, world):
    return output / mode / "checkpoints" / f"world-{world}"


def run_stage(output, mode, binary, binary_hash, workers, run_id):
    stage = output / mode
    stage.mkdir(exist_ok=False)
    (stage / "checkpoints").mkdir()
    write_json(stage / "execution.json", {
        "seed": ORIGINAL_SEED, "binary_sha256": binary_hash, "workers": workers,
        "mode": mode, "run_id": run_id, "started_utc_epoch": time.time(),
    })

    def run_world(world):
        directory = checkpoint_dir(output, mode, world)
        command = [str(binary), "--mode", mode, "--seed", str(ORIGINAL_SEED),
                   "--world", str(world), "--trace", str(stage / f"world-{world}.jsonl"),
                   "--checkpoints", str(directory)]
        if mode == "repeat":
            command += ["--input-checkpoint", str(checkpoint_dir(output, "reconstruct", world)
                                                  / CHECKPOINT_FILES["continuation_unfrozen"])]
        started = time.monotonic()
        try:
            with (stage / f"world-{world}.json").open("w", encoding="utf-8") as out, \
                    (stage / f"world-{world}.stderr.log").open("w", encoding="utf-8") as err:
                result = subprocess.run(command, stdout=out, stderr=err, check=False)
            record = {"world": world, "exit": result.returncode}
        except Exception as exc:
            record = {"world": world, "exit": -1, "error": str(exc)}
        print(f"RETAINED_WORLD mode={mode} world={world} exit={record['exit']} "
              f"seconds={time.monotonic()-started:.2f}", flush=True)
        return record

    # Capability or process failure in one world never removes another world.
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
        records = list(executor.map(run_world, range(WORLD_COUNT)))
    write_json(stage / "processes.json", records)


def successful_processes(stage):
    processes = read_json(stage / "processes.json")
    if (len(processes) != WORLD_COUNT or {p["world"] for p in processes} != set(range(WORLD_COUNT))
            or any(type(p["exit"]) is not int or p["exit"] != 0 for p in processes)):
        raise ValueError("not all eight evaluator processes exited successfully")
    return processes


def inspect_checkpoints(output, mode, world, summary):
    directory = checkpoint_dir(output, mode, world)
    # Metadata records the evaluator's original canonical paths. Downloaded
    # archives may move; validate those identities against their recorded root,
    # while reading and hashing the relocated local files below.
    origin = Path(read_json(output / "execution.json")["evidence_origin"])
    if not origin.is_absolute():
        raise ValueError("recorded evidence origin is not absolute")
    original_directory = checkpoint_dir(origin, mode, world)
    metadata = read_json(directory / "metadata.json")
    if (metadata["schema"] != "aeterna.integral-retained-knowledge.metadata.v1"
            or metadata["mode"] != mode or metadata["world"] != world
            or metadata["seed"] != ORIGINAL_SEED):
        raise ValueError("checkpoint metadata identity mismatch")
    if any(metadata["integrity"].get(key) is not True for key in METADATA_CHECKS):
        raise ValueError("checkpoint integrity assertions did not all pass")
    phases = {p["phase"]: p for p in summary["phases"]}
    before = metadata["before_reacquisition"]
    if before != phases["acquire"]["before"] or before["controller_frozen"] is not False:
        raise ValueError("initial live snapshot differs from the acquire boundary")
    if set(metadata["checkpoints"]) != set(CHECKPOINT_FILES):
        raise ValueError("checkpoint set differs from the continuation protocol")
    files = {}
    for key, filename in CHECKPOINT_FILES.items():
        path = directory / filename
        item = metadata["checkpoints"][key]
        if (item["path"] != str(original_directory / filename)
                or item["bytes"] != path.stat().st_size):
            raise ValueError(f"checkpoint path or byte count mismatch: {key}")
        if type(item["fingerprint"]) is not int or not 0 <= item["fingerprint"] < 2**64:
            raise ValueError(f"invalid checkpoint fingerprint: {key}")
        files[key] = {"path": str(path.relative_to(output)), "bytes": item["bytes"],
                      "fingerprint": item["fingerprint"], "sha256": sha256(path)}
    if mode == "reconstruct":
        if metadata["input_checkpoint"] is not None or metadata["input_boundary_equivalent"] is not None:
            raise ValueError("historical reconstruction unexpectedly used a warm input")
    elif metadata["input_boundary_equivalent"] is not True:
        raise ValueError("repeat input did not preserve the prior episode boundary")
    return {"metadata_sha256": sha256(directory / "metadata.json"), "checkpoints": files,
            "before_reacquisition": before, "input_checkpoint": metadata["input_checkpoint"]}


def verify_reconstruction(output, archive):
    errors, worlds = [], []
    try:
        successful_processes(output / "reconstruct")
    except Exception as exc:
        errors.append(f"reconstruction process integrity: {exc}")
    for world in range(WORLD_COUNT):
        record = {"world": world, "valid": False, "errors": []}
        try:
            stage = output / "reconstruct"
            summary = read_json(stage / f"world-{world}.json")
            actual_hash = sha256(stage / f"world-{world}.jsonl")
            record.update({"original_trace_sha256": archive["trace_sha256"][world],
                           "reconstructed_trace_sha256": actual_hash,
                           "trace_matches": actual_hash == archive["trace_sha256"][world],
                           "parsed_summary_matches": summary == archive["summaries"][world]})
            if not record["trace_matches"] or not record["parsed_summary_matches"]:
                raise ValueError("historical trace or parsed summary does not match exactly")
            record.update(inspect_checkpoints(output, "reconstruct", world, summary))
            record["valid"] = True
        except Exception as exc:
            record["errors"].append(str(exc))
        worlds.append(record)
    return {"valid": not errors and all(w["valid"] for w in worlds), "errors": errors,
            "worlds": worlds, "historical_reconstruction_only": True,
            "counts_as_fresh_qualification": False}


def verify_continuity(output, archive, replay):
    errors, worlds = [], []
    try:
        successful_processes(output / "repeat")
    except Exception as exc:
        errors.append(f"repeat process integrity: {exc}")
    for world in range(WORLD_COUNT):
        record = {"world": world, "valid": False, "errors": []}
        try:
            summary = read_json(output / "repeat" / f"world-{world}.json")
            info = inspect_checkpoints(output, "repeat", world, summary)
            record.update(info)
            prior = replay["worlds"][world]["checkpoints"]["continuation_unfrozen"]
            initial = info["checkpoints"]["initial"]
            source_path = checkpoint_dir(output, "reconstruct", world) / CHECKPOINT_FILES["continuation_unfrozen"]
            origin = Path(read_json(output / "execution.json")["evidence_origin"])
            original_source = checkpoint_dir(origin, "reconstruct", world) / CHECKPOINT_FILES["continuation_unfrozen"]
            source = info["input_checkpoint"]
            if (source["path"] != str(original_source)
                    or source["bytes"] != prior["bytes"] or source["fingerprint"] != prior["fingerprint"]
                    or sha256(source_path) != prior["sha256"] or initial["sha256"] != prior["sha256"]):
                raise ValueError("repeat initial checkpoint is not the exact preserved live state")
            previous = next(p for p in archive["summaries"][world]["phases"] if p["phase"] == "adaptive")["after"]
            if info["before_reacquisition"] != previous:
                raise ValueError("retained starting snapshot differs from the original adaptive endpoint")
            if previous["factual_events"] <= 0 or previous["causal_report"]["learned_transitions"] <= 0:
                raise ValueError("repeat did not start with learned experience")
            record["retained_factual_events"] = previous["factual_events"]
            record["retained_learned_transitions"] = previous["causal_report"]["learned_transitions"]
            record["input_checkpoint_sha256"] = prior["sha256"]
            record["valid"] = True
        except Exception as exc:
            record["errors"].append(str(exc))
        worlds.append(record)
    return {"valid": not errors and all(w["valid"] for w in worlds), "errors": errors, "worlds": worlds}


def score_repeat(output, binary_hash):
    stream = io.StringIO()
    try:
        scorer = load_scorer()
        with contextlib.redirect_stdout(stream):
            code = scorer.score_all(output / "repeat", ORIGINAL_SEED, binary_hash)
        verdict = read_json(output / "repeat" / "verdict.json")
        return verdict, code, None
    except Exception as exc:
        return None, 2, f"original scorer could not complete: {exc}"
    finally:
        (output / "repeat" / "scorer.log").write_text(stream.getvalue(), encoding="utf-8")


def comparable(cold, warm):
    aggregate, worlds = {}, []
    if warm is None:
        return {"aggregate": aggregate, "worlds": worlds}
    for phase, c in cold["aggregate"].items():
        w = warm["aggregate"][phase]
        aggregate[phase] = {"cold": c, "repeat": w,
                            "repeat_minus_cold": {key: w[key] - c[key] for key in
                                ("successes", "actions", "sensor_queries", "noise_queries", "success_rate")}}
    repeat_worlds = {w["world"]: w for w in warm["worlds"]}
    for c in cold["worlds"]:
        w = repeat_worlds.get(c["world"])
        item = {"world": c["world"], "cold_complete_loop": c["complete_loop"],
                "cold_stages": c["stages"], "cold_rates": c["rates"],
                "cold_phase_totals": c["phase_totals"], "cold_adaptive_windows": c["adaptive_windows"],
                "repeat_present": w is not None}
        if w is not None:
            item.update({"repeat_complete_loop": w["complete_loop"], "repeat_stages": w["stages"],
                         "repeat_rates": w["rates"], "repeat_phase_totals": w["phase_totals"],
                         "repeat_adaptive_windows": w["adaptive_windows"],
                         "repeat_minus_cold_rates": {p: w["rates"][p] - c["rates"][p] for p in c["rates"]},
                         "repeat_failed_stages": [key for key, value in w["stages"].items() if not value],
                         "repeat_errors": w["errors"]})
        worlds.append(item)
    return {"aggregate": aggregate, "worlds": worlds}


def costs(verdict):
    if verdict is None:
        return None
    phases = verdict["aggregate"]
    return {"episodes": sum(p["episodes"] for p in phases.values()),
            "observed_episodes": sum(p["observed_episodes"] for p in phases.values()),
            "all_actions": sum(p["actions"] for p in phases.values()),
            "runtime_actions": sum(p["actions"] for name, p in phases.items() if name != "random_held"),
            "random_actions": phases["random_held"]["actions"],
            "learning_actions": sum(phases[name]["actions"] for name in ("acquire", "adaptive"))}


def save_report(output, execution, archive, replay, continuity, warm, errors):
    cold = archive["verdict"]
    valid = (not errors and replay["valid"] and continuity is not None and continuity["valid"]
             and warm is not None and warm["verdict"] != "INVALID")
    verdict = warm["verdict"] if valid else "INVALID"
    comparisons = comparable(cold, warm)
    report = {
        "schema": "aeterna.integral-retained-knowledge.continuation.v1", "verdict": verdict,
        "strict_repeat_verdict": warm["verdict"] if warm is not None else "NOT_RUN",
        "scope": "Developmental repetition of the known 0-to-1 scenario with full retained agent state; no fresh qualification",
        "world_seed": ORIGINAL_SEED, "run_id_is_provenance_only": True, "execution": execution,
        "historical": {"verdict": cold["verdict"], "complete_loop_worlds": cold["complete_loop_worlds"],
                       "source_commit": ORIGINAL_COMMIT, "archive_sha256": ARCHIVE_SHA256,
                       "binary_sha256": cold["binary_sha256"], "verdict_unchanged": True},
        "replay_integrity": replay, "retained_state_integrity": continuity,
        "repeat": None if warm is None else {"verdict": warm["verdict"],
                    "complete_loop_worlds": warm["complete_loop_worlds"], "checks": warm["checks"],
                    "thresholds": warm["thresholds"], "invalid_worlds": warm["invalid_worlds"]},
        "comparison": comparisons,
        "costs": {"historical_first_pass": costs(cold),
                  "reconstruction_this_run": costs(cold) if replay["valid"] else None,
                  "repeat_this_run": costs(warm)},
        "errors": errors,
        "interpretation": [
            "The first FAIL and its 4/8 complete chains remain unchanged.",
            "All eight historical traces and parsed summaries must match before any repeat actor starts.",
            "Reconstruction restores a historical state; it is not a new successful trial.",
            "The repeat preserves motor/symbol meanings, the original episode schedule and all capability thresholds.",
            "The retained endpoint knows rule 1; returning to rule 0 is itself an unannounced change.",
            "Retained state includes RNG/counters/model/policy; this does not isolate causal memory alone.",
            "No fresh schedule or unrelated-task transfer is measured.",
            "Goal improvement and stricter forecast correction are reported separately.",
        ],
    }
    write_json(output / "continuation.json", report)
    lines = ["# Integral retained-knowledge continuation", "", f"**Continuation verdict: {verdict}**", "",
             "The original result remains **FAIL, 4/8 complete chains**. Historical replay is state reconstruction, not fresh qualification.", "",
             f"Original world seed: `{ORIGINAL_SEED}`. Current run ID `{execution.get('run_id')}` is provenance only.",
             f"Global replay barrier: **{'PASS' if replay['valid'] else 'INVALID'}**.",
             f"Retained state continuity: **{'PASS' if continuity and continuity['valid'] else 'NOT VERIFIED'}**.", ""]
    if warm is not None:
        lines += [f"Strict repeat verdict under unchanged thresholds: **{warm['verdict']}**, "
                  f"complete chains **{warm['complete_loop_worlds']}/{WORLD_COUNT}**, required **6/{WORLD_COUNT}**.", "",
                  "## Cold and retained-state results", "",
                  "| Phase | Cold goals | Repeat goals | Change, percentage points | Cold actions | Repeat actions |",
                  "|---|---:|---:|---:|---:|---:|"]
        for name, pair in comparisons["aggregate"].items():
            c, w, d = pair["cold"], pair["repeat"], pair["repeat_minus_cold"]
            lines.append(f"| {name} | {c['successes']}/{c['episodes']} | {w['successes']}/{w['episodes']} | "
                         f"{100*d['success_rate']:+.2f} | {c['actions']} | {w['actions']} |")
        lines += ["", "## Per-world gates", "", "| World | Cold full chain | Repeat full chain | Failed repeat stages |",
                  "|---|---|---|---|"]
        for item in comparisons["worlds"]:
            failures = ", ".join(item.get("repeat_failed_stages", [])) if item["repeat_present"] else "missing world"
            lines.append(f"| {item['world']} | {item['cold_complete_loop']} | "
                         f"{item.get('repeat_complete_loop', 'INVALID')} | {failures or 'none'} |")
    lines += ["", "## Retained starting state", "",
              "| World | Factual events retained | Learned transitions retained | Input checkpoint SHA256 |",
              "|---|---:|---:|---|"]
    if continuity is not None:
        for item in continuity["worlds"]:
            lines.append(f"| {item['world']} | {item.get('retained_factual_events', 'INVALID')} | "
                         f"{item.get('retained_learned_transitions', 'INVALID')} | "
                         f"{item.get('input_checkpoint_sha256', 'unverified')} |")
    lines += ["", "## Interpretation", ""] + [f"- {item}" for item in report["interpretation"]]
    if errors or replay["errors"] or (continuity and continuity["errors"]):
        lines += ["", "## Integrity errors", ""]
        lines += [f"- {item}" for item in errors + replay["errors"] + (continuity["errors"] if continuity else [])]
    lines += ["", "`continuation.json` contains complete comparisons, costs, source/binary/checkpoint hashes and per-world integrity details.",
              "`repeat/verdict.json` is produced by the unchanged original scorer. All traces, checkpoints, failures and timeouts are retained."]
    (output / "RESULT.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(json.dumps({"verdict": verdict, "strict_repeat_verdict": report["strict_repeat_verdict"],
                      "replay_valid": replay["valid"], "continuity_valid": bool(continuity and continuity["valid"]),
                      "cold_complete_worlds": cold["complete_loop_worlds"],
                      "repeat_complete_worlds": warm["complete_loop_worlds"] if warm else None,
                      "errors": errors}), flush=True)
    return 0 if verdict == "PASS" else 1 if verdict == "FAIL" else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--run-id", help="Current workflow run ID, used only as provenance")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=2, choices=(1, 2, 4))
    parser.add_argument("--score-only", action="store_true", help="Verify and score saved evidence without any evaluator execution")
    args = parser.parse_args()
    if not args.score_only and (args.binary is None or not args.run_id):
        parser.error("execution requires --binary and --run-id")
    output = args.output.resolve()
    archive = load_archive()
    current_provenance = provenance()
    if args.score_only:
        execution = read_json(output / "execution.json")
        if execution["world_seed"] != ORIGINAL_SEED or execution["source_sha256"] != current_provenance["source_sha256"]:
            raise ValueError("saved execution differs from the frozen seed or source files")
        binary_hash = execution["binary_sha256"]
        if args.binary is not None and sha256(args.binary) != binary_hash:
            raise ValueError("provided binary differs from the recorded execution binary")
        sealed_replay = read_json(output / "replay-gate.json")
    else:
        binary = args.binary.resolve(strict=True)
        binary_hash = sha256(binary)
        output.mkdir(parents=True, exist_ok=False)
        execution = {**current_provenance, "world_seed": ORIGINAL_SEED, "run_id": args.run_id,
                     "binary_sha256": binary_hash, "workers": args.workers,
                     "archive_sha256": ARCHIVE_SHA256, "evidence_origin": str(output),
                     "started_utc_epoch": time.time()}
        write_json(output / "execution.json", execution)
        (output / "historical-verdict.json").write_bytes(archive["verdict_bytes"])
        run_stage(output, "reconstruct", binary, binary_hash, args.workers, args.run_id)
        sealed_replay = verify_reconstruction(output, archive)
        write_json(output / "replay-gate.json", sealed_replay)
        print(f"RETAINED_GLOBAL_REPLAY_BARRIER valid={sealed_replay['valid']}", flush=True)
        # No repeat directory, process or actor exists before this global gate.
        if not sealed_replay["valid"]:
            return save_report(output, execution, archive, sealed_replay, None, None, [])
        run_stage(output, "repeat", binary, binary_hash, args.workers, args.run_id)

    errors = []
    replay = verify_reconstruction(output, archive)
    if not sealed_replay["valid"] or replay != sealed_replay:
        errors.append("historical evidence/checkpoints differ from the sealed global replay gate")
    if sha256(output / "historical-verdict.json") != hashlib.sha256(archive["verdict_bytes"]).hexdigest():
        errors.append("preserved historical verdict differs from the original archive")
    if current_provenance["source_sha256"] != provenance()["source_sha256"]:
        errors.append("experiment source files changed during execution")
    if not args.score_only and sha256(binary) != binary_hash:
        errors.append("evaluator binary changed during execution")
    continuity, warm = None, None
    if (output / "repeat").exists():
        continuity = verify_continuity(output, archive, sealed_replay)
        warm, _, error = score_repeat(output, binary_hash)
        if error is not None:
            errors.append(error)
    else:
        errors.append("repeat was not executed because reconstruction did not qualify")
    return save_report(output, execution, archive, replay, continuity, warm, errors)


if __name__ == "__main__":
    sys.exit(main())
