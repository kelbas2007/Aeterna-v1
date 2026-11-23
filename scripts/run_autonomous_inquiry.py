#!/usr/bin/env python3
"""Run and independently score the frozen autonomous inquiry experiment.

Development runs cannot qualify. --score-only never executes a runtime.
Scientific negative results exit 1; invalid evidence exits 2.
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

ROOT = Path(__file__).resolve().parents[1]
ARMS = ("ordinary", "active", "observe_only")
CONDITIONS = {"switching": 1024, "stationary": 256}
ROLES = ("cue_sensor", "diagnostic_sensor", "noise_sensor", "actuator0", "actuator1")
BASE_COMMIT = "48197411f2dc581b3eeba1bec2f47a11ec2ee8aa"
FILES = ("examples/autonomous_inquiry_eval.rs", "scripts/run_autonomous_inquiry.py",
         "docs/experiments/AUTONOMOUS_INQUIRY_20261123_PROTOCOL.md")
GATES = {"worlds": 16, "mean_gain_minimum": .02, "positive_worlds_minimum": 12,
         "stationary_gain_minimum": -.02, "maximum_action_ratio": 1.15,
         "chain_worlds_minimum": 12, "goal_followup_episodes": 8}


def require(value, message):
    if not value:
        raise ValueError(message)


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write_json(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def fingerprint(data):
    result = 0xcbf29ce484222325
    for byte in data:
        result = ((result ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return result


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()


def close(actual, expected, label):
    require(isinstance(actual, (int, float)) and not isinstance(actual, bool) and math.isfinite(actual), f"nonfinite {label}")
    require(math.isclose(actual, expected, rel_tol=1e-8, abs_tol=2e-7), f"{label} differs: {actual} != {expected}")


def provenance(binary, development):
    files = sorted({str(p.relative_to(ROOT)) for p in (ROOT / "src").rglob("*.rs")} | set(FILES) |
                   {"Cargo.toml", "Cargo.lock"} |
                   {p for p in ("rust-toolchain.toml", "rust-toolchain", "tests/autonomous_inquiry_contract.rs") if (ROOT / p).is_file()} |
                   {str(p.relative_to(ROOT)) for p in (ROOT / ".github/workflows").glob("*autonomous-inquiry*.yml")})
    if not development:
        git("ls-files", "--error-unmatch", "--", *files)
        require(not git("diff", "--name-only", "HEAD", "--", *files), "qualifying source is not committed")
    return {"source_commit": git("rev-parse", "HEAD"), "experiment_base": BASE_COMMIT,
            "source_sha256": {p: sha256(ROOT / p) for p in files},
            "binary_sha256": sha256(binary), "binary": str(binary), "development": development,
            "seed_used_after_source_manifest": True}


def execute(output, binary, seed, worlds, workers):
    def one(world):
        directory = output / f"world-{world}"
        command = [str(binary), "--seed", str(seed), "--world", str(world), "--out-dir", str(directory)]
        record = {"world": world, "command": command, "timeout_seconds": 1800}
        started = time.monotonic()
        try:
            with (output / f"world-{world}.stdout.log").open("w") as stdout, (output / f"world-{world}.stderr.log").open("w") as stderr:
                result = subprocess.run(command, stdout=stdout, stderr=stderr, timeout=1800, check=False)
            record.update(exit=result.returncode, timed_out=False)
        except subprocess.TimeoutExpired:
            record.update(exit=124, timed_out=True)
        except Exception as exc:
            record.update(exit=-1, timed_out=False, error=str(exc))
        record["seconds"] = time.monotonic() - started
        write_json(output / f"world-{world}.process.json", record)
        print(f"INQUIRY_WORLD world={world} exit={record['exit']} seconds={record['seconds']:.2f}", flush=True)
        return record
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        records = list(pool.map(one, range(worlds)))
    write_json(output / "processes.json", records)


def safe_file(directory, name):
    path = Path(name)
    require(not path.is_absolute() and ".." not in path.parts, f"unsafe path {name}")
    result = directory / path
    require(result.is_file(), f"missing evidence file {name}")
    return result


def memory(parsed):
    return parsed["cognition"]["current"]["learner"]["causal_memory"]


def check_checkpoints(directory, summary):
    records = [summary["common_checkpoint"]] + [s["checkpoint"] for s in summary["starts"]]
    for result in [summary["acquisition"]] + summary["results"]:
        records.extend(result[k] for k in ("midpoint_checkpoint", "final_checkpoint", "frozen_checkpoint"))
    require(len({r["file"] for r in records}) == len(records), "duplicate checkpoint file")
    parsed = {}
    for record in records:
        path = safe_file(directory, record["file"]); data = path.read_bytes()
        require(record["roundtrip_exact"] is True, "checkpoint exact roundtrip not checked")
        require(len(data) == record["bytes"] and fingerprint(data) == record["fingerprint"], "checkpoint identity differs")
        value = json.loads(data); causal = memory(value)
        require(causal["config"]["model_entries"] == 2048 and causal["config"]["model_bytes"] == 2*1024*1024, "capacity differs")
        require(causal["bytes"] <= 2*1024*1024 and len(causal["counts"]) <= 2048, "checkpoint exceeds capacity")
        require(value["cognition"]["freeze_controller"] == (".frozen." in record["file"]), "freeze identity differs")
        config = value["cognition"]["current"]["learner"]["config"]
        require(config["width"] == 2 and config["actions"] == 5 and config["planning_work"] == 120000 and config["planning_depth"] == 5,
                "runtime limits differ")
        parsed[record["file"]] = value
    common_name = summary["common_checkpoint"]["file"]
    common = parsed[common_name]
    starts = {s["arm"]: s for s in summary["starts"]}
    require(set(starts) == set(ARMS), "wrong arm starts")
    require((directory / starts["ordinary"]["checkpoint"]["file"]).read_bytes() == (directory / common_name).read_bytes(), "ordinary not exact common clone")
    def normalized(value):
        result = copy.deepcopy(value); result.pop("hash", None)
        causal = memory(result); causal.pop("inquiry", None)
        causal["bytes"] = memory(common)["bytes"]
        return result
    for arm in ARMS:
        require(normalized(parsed[starts[arm]["checkpoint"]["file"]]) == normalized(common), f"{arm} changes existing warm state")
        require(starts[arm]["snapshot"]["causal"]["learned_fingerprint"] == summary["common_snapshot"]["causal"]["learned_fingerprint"], "learned model changed at enable")
    return {"count": len(records), "all_fingerprints_match": True, "warm_state_preserved": True}


def entropy(values):
    return -sum(p * math.log(p) for p in values if p > 0)


def check_decision(row):
    decision = row.get("inquiry_decision")
    if row["source"] != "causal-belief-inquiry":
        require(decision is None, "unselected inquiry witness attached to action")
        return None
    require(isinstance(decision, dict), "selected inquiry lacks prospective witness")
    require(row["arm"] == "active", "directed inquiry selected in an ablation")
    require(decision["event"] == row["factual_before"] and decision["action"] == row["action"], "witness event/action is not executed action")
    before = row["inquiry_before"]
    require(before and before["active"] and decision["question_id"] == before["active"]["id"], "witness not from current pending question")
    weights = decision["bank_probability"]
    require(len(weights) == 2 and all(math.isfinite(x) and 0 <= x <= 1 for x in weights), "invalid bank prior")
    close(sum(weights), 1, "bank prior mass")
    predictions = decision["predictions"]
    require(len(predictions) == 2 and {p["bank"] for p in predictions} == {0, 1}, "two rival predictions missing")
    mixture = {}; conditional = 0.0
    for prediction in predictions:
        probability = {}
        for post, mass in prediction["outcomes"]:
            require(math.isfinite(mass) and 0 <= mass <= 1, "invalid predicted outcome mass")
            key = tuple(post); require(key not in probability, "duplicate predicted outcome")
            probability[key] = mass
        unknown = prediction["unknown_probability"]
        require(math.isfinite(unknown) and 0 <= unknown <= 1, "invalid unknown mass")
        probability[None] = unknown
        close(sum(probability.values()), 1, "rival distribution mass")
        w = weights[prediction["bank"]]
        conditional += w * entropy(probability.values())
        for key, value in probability.items(): mixture[key] = mixture.get(key, 0) + w * value
    immediate = max(0.0, entropy(mixture.values()) - conditional)
    recorded_information = decision["immediate_information_gain"]
    require(math.isfinite(recorded_information) and 0 <= recorded_information <= immediate + 2e-7, "information exceeds reducible rival disagreement")
    # The actor may gate unsupported or goal-irrelevant immediate contrasts to
    # zero while planning a supported diagnostic opportunity further ahead.
    if recorded_information > 1e-12:
        close(recorded_information, immediate, "rival information excludes common outcome noise")
    expected = decision["expected_information_gain"]
    require(math.isfinite(expected) and expected > 0, "selected experiment has no expected information")
    require(math.isfinite(decision["expected_goal_probability"]) and 0 <= decision["expected_goal_probability"] <= 1.00000001, "invalid inquiry goal probability")
    return decision


def score_trace(directory, result, schedule, world, arm, condition, motors, codes):
    path = safe_file(directory, result["trace_file"])
    rows_by_episode = [[] for _ in schedule]
    physical_hash = hashlib.sha256()
    questions = {}; resolutions = {}; diagnostics = {}; inquiry_actions = 0
    totals = {k: 0 for k in ("episodes", "successes", "failures", "timeouts", "actions", "cue_queries", "diagnostic_queries", "noise_queries",
                            "terminal_actions", "inquiry_source_actions", "readonly_work")}
    totals["predictive_nll"] = 0.0
    factual = result["before"]["events"]; learned = result["before"]["causal"]["learned_transitions"]
    permit = None; previous_episode = -1; previous_step = -1; current = None
    prior_inquiry = result["before"]["inquiry"]
    with path.open(encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, 1):
            row = json.loads(line)
            require(row["world"] == world and row["arm"] == arm and row["condition"] == condition, "trace identity mismatch")
            ep = row["episode"]; step = row["step"]
            require(0 <= ep < len(schedule) and 0 <= step < 5, "episode/step beyond fixed limits")
            if ep != previous_episode:
                require(ep == previous_episode + 1 and step == 0, "episode order mismatch")
                current = [0, codes[0]]; previous_step = -1
            require(step == previous_step + 1 and row["pre"] == current, "step PRE continuity mismatch")
            require(not rows_by_episode[ep] or not rows_by_episode[ep][-1]["terminal"], "action after terminal POST")
            expected = schedule[ep]
            require(all(row[k] == expected[k] for k in ("segment", "rule", "hidden")), "private schedule mismatch")
            action = row["action"]; require(isinstance(action, int) and 0 <= action < 5, "invalid motor")
            role = motors.index(action); terminal = role >= 3
            success = terminal and role-3 == (expected["hidden"] ^ expected["rule"])
            state = 1+expected["hidden"] if role == 0 else 3+expected["rule"] if role == 1 else 5+expected["noise"][step] if role == 2 else 7 if success else 8
            post = [0, codes[state]]
            require(row["role"] == ROLES[role] and row["post"] == post and row["terminal"] == terminal and row["success"] == success,
                    f"physical execution mismatch at {condition}/{arm}/{ep}/{step}")
            require(row["reward"] == int(success) and row["cost"] == 1, "physical reward/cost differs")
            require(row["factual_before"] == factual and row["factual_after"] == factual+1 and row["event"] == factual+1, "factual transition conservation")
            require(row["learned_before"] == learned and row["learned_after"] == learned+1, "learning transition conservation")
            require(permit is None or row["permit_sequence"] > permit, "permit reused")
            permit = row["permit_sequence"]; factual += 1; learned += 1
            require(row["model_bytes"] <= 2*1024*1024 and row["retained_entries"] <= 2048, "trace exceeds shared capacity")
            beliefs = row["pre_beliefs"]
            require({b["action"] for b in beliefs} == set(range(5)), "prospective offers differ")
            belief = next(b for b in beliefs if b["action"] == action)
            probability = sum(p for target,p in belief["outcomes"] if target == post)
            close(row["prequential_probability"], probability, "prospective outcome probability")
            close(row["nll"], -math.log(max(probability, 1e-12)), "prospective logarithmic loss")
            decision = check_decision(row)
            if decision:
                inquiry_actions += 1
                if role == 1 and decision["immediate_information_gain"] > 1e-9:
                    diagnostics.setdefault(decision["question_id"], []).append({"event":row["factual_before"],"episode":ep,
                        "rule":expected["rule"],"segment":expected["segment"],"expected_information_gain":decision["expected_information_gain"],
                        "immediate_information_gain":decision["immediate_information_gain"],"source":row["source"]})
            inquiry = row["inquiry_after"]
            require(row["inquiry_before"] == prior_inquiry, "inquiry changed without a factual action or was reset between episodes")
            prior_inquiry = inquiry
            if arm == "ordinary": require(inquiry is None and row["inquiry_before"] is None, "ordinary contains inquiry")
            else:
                require(inquiry and inquiry["mode"] == arm, "inquiry mode differs")
                before_inquiry = row["inquiry_before"]
                require(inquiry["observations"] == before_inquiry["observations"] + 1, "inquiry factual count differs")
                require(inquiry["extra_inference_work"] == before_inquiry["extra_inference_work"] + 8192, "inquiry work conservation")
                require(inquiry["selected_experiments"] == before_inquiry["selected_experiments"] + int(decision is not None), "selected experiment ledger differs")
                require(inquiry["factual_experiments"] == before_inquiry["factual_experiments"] + int(decision is not None), "factual experiment ledger differs")
                require(inquiry["questions_opened"] == inquiry["questions_resolved"] + inquiry["questions_expired"] + int(inquiry["active"] is not None), "question ledger conservation")
                q = inquiry["active"]
                if q and q["id"] not in questions:
                    require(q["opened_event"] == row["factual_before"], "question not opened on recorded factual event")
                    require(q["triggering_action"] == action and q["triggering_pre"] == row["pre"] and q["actual_post"] == post,
                            "question origin differs from actual physical transition")
                    require(q["support"] >= 8 and q["prediction_probability"] >= .75 and q["observed_probability"] < .25,
                            "question lacks supported surprising contradiction")
                    require(q["predicted_post"] != post, "question says its predicted outcome occurred")
                    questions[q["id"]] = {"id":q["id"],"opened_event":q["opened_event"],"episode":ep,"rule":expected["rule"],"segment":expected["segment"],
                        "support":q["support"],"prediction_probability":q["prediction_probability"],"observed_probability":q["observed_probability"]}
                resolution = inquiry["last_resolution"]
                if resolution and resolution["question_id"] not in resolutions:
                    require(resolution["ended_event"] == row["factual_before"], "resolution is not from the factual event")
                    require(resolution["question_id"] in questions, "resolution lacks originating question")
                    require(resolution["opened_event"] == questions[resolution["question_id"]]["opened_event"], "resolution changes question origin")
                    require(resolution["status"] in ("resolved", "expired"), "unknown resolution status")
                    if resolution["status"] == "resolved":
                        require(resolution["bank"] in (0, 1) and resolution["evidence_events"] >= 2 and abs(resolution["log_bayes_factor"]) >= math.log(9), "resolution lacks factual repeated likelihood evidence")
                        require((resolution["bank"] == 0 and resolution["log_bayes_factor"] > 0) or
                                (resolution["bank"] == 1 and resolution["log_bayes_factor"] < 0), "resolved bank contradicts signed factual evidence")
                    else:
                        require(resolution["bank"] is None and resolution["ended_event"] - resolution["opened_event"] >= 192, "expiry falsely resolves a bank")
                    resolutions[resolution["question_id"]] = dict(resolution, episode=ep, rule=expected["rule"], segment=expected["segment"])
                require(inquiry["questions_opened"] == len(questions), "opened-question counter differs from trace")
                require(inquiry["questions_resolved"] == sum(r["status"] == "resolved" for r in resolutions.values()), "resolved-question counter differs from trace")
                require(inquiry["questions_expired"] == sum(r["status"] == "expired" for r in resolutions.values()), "expired-question counter differs from trace")
            totals["actions"] += 1; totals["cue_queries"] += role == 0; totals["diagnostic_queries"] += role == 1
            totals["noise_queries"] += role == 2; totals["terminal_actions"] += terminal
            totals["inquiry_source_actions"] += decision is not None; totals["readonly_work"] += row["readonly_work"]
            totals["predictive_nll"] += row["nll"]
            physical_hash.update(json.dumps({k:row[k] for k in ("episode","step","action","post","success","source")},sort_keys=True,separators=(",",":")).encode())
            rows_by_episode[ep].append({k:row[k] for k in ("episode","rule","segment","success","terminal","action","event","step")})
            current = post; previous_episode = ep; previous_step = step
    require(previous_episode == len(schedule)-1, "incomplete trace")
    require(len(result["episodes"]) == len(schedule), "episode summary length differs")
    for ep, rows in enumerate(rows_by_episode):
        require(rows, "empty episode")
        last = rows[-1]
        totals["episodes"] += 1; totals["successes"] += last["success"]
        totals["failures"] += last["terminal"] and not last["success"]
        totals["timeouts"] += not last["terminal"]
        require(last["terminal"] or len(rows) == 5, "premature nonterminal episode stop")
        saved = result["episodes"][ep]
        require(saved["episode"] == ep and saved["success"] == last["success"] and saved["terminal"] == last["terminal"], "episode outcome differs")
        require(saved["totals"]["actions"] == len(rows), "episode action total differs")
    for key, expected in totals.items(): close(result["totals"][key], expected, f"trace total {key}")
    require(result["after"]["events"] == factual and result["after"]["causal"]["learned_transitions"] == learned, "final event count differs")
    require(result["after"]["cost"] - result["before"]["cost"] == totals["actions"], "runtime cost total differs")
    require(result["after"]["agency"]["self_selected_actions"]-result["before"]["agency"]["self_selected_actions"] == totals["actions"], "self-selection ledger differs")
    close(result["totals"]["computation_work"], result["after"]["work"]-result["before"]["work"], "computation work delta")
    chains = []
    for qid, q in questions.items():
        resolution = resolutions.get(qid)
        if not resolution or resolution["status"] != "resolved": continue
        probes = [p for p in diagnostics.get(qid, []) if q["opened_event"] < p["event"] <= resolution["ended_event"] and p["rule"] == q["rule"] == resolution["rule"] and p["segment"] == q["segment"] == resolution["segment"]]
        if not probes: continue
        probe = probes[0]; success_event = None
        for ep in range(resolution["episode"], min(len(schedule), resolution["episode"]+GATES["goal_followup_episodes"]+1)):
            if schedule[ep]["segment"] != q["segment"]: break
            successes = [r for r in rows_by_episode[ep] if r["success"] and r["event"]-1 > resolution["ended_event"]]
            if successes: success_event = successes[0]; break
        if success_event:
            chains.append({"question":q,"diagnostic":probe,"resolution":resolution,"successful_goal":success_event})
    return {"valid":True,"totals":totals,"questions":len(questions),"resolved":sum(r["status"]=="resolved" for r in resolutions.values()),
        "expired":sum(r["status"]=="expired" for r in resolutions.values()),"selected_inquiries":inquiry_actions,
        "diagnostic_inquiries":sum(len(p) for p in diagnostics.values()),"complete_chains":len(chains),"chains":chains,
        "physical_behavior_sha256":physical_hash.hexdigest(),"trace_sha256":sha256(path),
        "unresolved_questions":len(questions)-len(resolutions)}


def score_world(output, world, seed):
    directory = output/f"world-{world}"; process = read_json(output/f"world-{world}.process.json")
    require(process["exit"] == 0 and not process["timed_out"], f"world process unsuccessful: {process}")
    summary = read_json(directory/"summary.json"); schedule = read_json(directory/"schedule.json")
    require(summary["world"] == world and summary["seed"] == seed == schedule["seed"], "world/seed identity mismatch")
    require(schedule["actor_receives_schedule"] is False and all(summary["integrity"].values()), "evaluator integrity assertion missing")
    checkpoint_report = check_checkpoints(directory, summary)
    require(len(summary["results"]) == 6, "missing arm/condition")
    results = {}
    acquisition = score_trace(directory, summary["acquisition"], schedule["episodes"]["acquisition"], world, "ordinary", "acquisition", schedule["motors"], schedule["codes"])
    for result in summary["results"]:
        arm, condition = result["arm"], result["condition"]
        require(arm in ARMS and condition in CONDITIONS and (condition,arm) not in results, "duplicate or unknown arm")
        require(len(schedule["episodes"][condition]) == CONDITIONS[condition], "schedule size differs")
        results[condition,arm] = score_trace(directory,result,schedule["episodes"][condition],world,arm,condition,schedule["motors"],schedule["codes"])
    observed_equal = {condition: results[condition,"ordinary"]["physical_behavior_sha256"] == results[condition,"observe_only"]["physical_behavior_sha256"] for condition in CONDITIONS}
    return {"world":world,"valid":True,"checkpoints":checkpoint_report,"acquisition":acquisition,
        "ordinary_observe_only_behavior_equal":observed_equal,
        "results":{condition:{arm:results[condition,arm] for arm in ARMS} for condition in CONDITIONS}}


def rate(record):
    t = record["totals"]; return t["successes"]/t["episodes"]


def summarize(output):
    manifest = read_json(output/"manifest.json"); worlds=[]; errors=[]
    for world in range(manifest["worlds"]):
        try:
            row = score_world(output,world,manifest["seed"]); worlds.append(row)
            print(f"INQUIRY_AUDIT world={world} valid=True chains={row['results']['switching']['active']['complete_chains']}",flush=True)
        except Exception as exc:
            errors.append({"world":world,"error":str(exc)})
            print(f"INQUIRY_AUDIT world={world} valid=False error={exc}",flush=True)
    valid = not errors and len(worlds)==manifest["worlds"]
    comparisons={}; aggregate={}; chain_worlds=0; active_chains=0
    if valid:
        for condition in CONDITIONS:
            aggregate[condition]={}
            for arm in ARMS:
                records=[w["results"][condition][arm] for w in worlds]
                totals={k:sum(r["totals"][k] for r in records) for k in records[0]["totals"]}
                aggregate[condition][arm]={"totals":totals,"success_rate":totals["successes"]/totals["episodes"],
                    "questions":sum(r["questions"] for r in records),"resolved":sum(r["resolved"] for r in records),
                    "expired":sum(r["expired"] for r in records),"selected_inquiries":sum(r["selected_inquiries"] for r in records),
                    "diagnostic_inquiries":sum(r["diagnostic_inquiries"] for r in records),"complete_chains":sum(r["complete_chains"] for r in records)}
        for comparison in (("active","observe_only"),("active","ordinary"),("observe_only","ordinary")):
            first,second=comparison
            gains=[rate(w["results"]["switching"][first])-rate(w["results"]["switching"][second]) for w in worlds]
            stationary=[rate(w["results"]["stationary"][first])-rate(w["results"]["stationary"][second]) for w in worlds]
            action_ratio=aggregate["switching"][first]["totals"]["actions"]/aggregate["switching"][second]["totals"]["actions"]
            comparisons[f"{first}_vs_{second}"]={"paired_gains":gains,"mean_gain":statistics.mean(gains),"positive_worlds":sum(x>0 for x in gains),
                "stationary_mean_gain":statistics.mean(stationary),"switching_action_ratio":action_ratio,
                "benefit_gate":statistics.mean(gains)>=.02 and sum(x>0 for x in gains)>=12,
                "stationary_guard":statistics.mean(stationary)>=-.02,"action_guard":action_ratio<=1.15}
        chain_worlds=sum(w["results"]["switching"]["active"]["complete_chains"]>0 for w in worlds)
        active_chains=sum(w["results"]["switching"]["active"]["complete_chains"] for w in worlds)
    primary=comparisons.get("active_vs_observe_only",{})
    benefit=valid and bool(primary.get("benefit_gate")) and bool(primary.get("stationary_guard")) and bool(primary.get("action_guard"))
    chain=valid and chain_worlds>=12
    qualified=not manifest["development"] and manifest["worlds"]==16
    verdict="INVALID" if not valid else "DEVELOPMENT" if not qualified else "SUPPORTED" if benefit and chain else "VALID_NEGATIVE"
    result={"schema":"aeterna.autonomous-inquiry.verdict.v1","verdict":verdict,"valid":valid,"qualifying":qualified,
        "seed":manifest["seed"],"provenance":manifest["provenance"],"fixed_gates":GATES,"valid_worlds":len(worlds),"errors":errors,
        "conclusions":{"directed_inquiry_benefit":{"supported":qualified and benefit},
            "integrated_factual_chain":{"supported":qualified and chain,"worlds":chain_worlds,"chains":active_chains}},
        "comparisons":comparisons,"aggregate":aggregate,"worlds":worlds}
    write_json(output/"verdict.json",result)
    lines=["# Autonomous inquiry evaluation result","",f"**Verdict: {verdict}.** Valid worlds: {len(worlds)}/{manifest['worlds']}.","",
           f"Seed: `{manifest['seed']}`. Source commit: `{manifest['provenance']['source_commit']}`.","",
           "This evaluates the integrated bounded inquiry mechanism in an opaque two-rule sensor/actuator family. It does not establish general intelligence.",""]
    if valid:
        lines += ["## Observed behavior","","| Condition | Arm | Goals | Success | Actions | Diagnostic reads | Noise reads | Selected inquiries |", "|---|---|---:|---:|---:|---:|---:|---:|"]
        for condition in CONDITIONS:
            for arm in ARMS:
                r=aggregate[condition][arm]; t=r["totals"]
                lines.append(f"| {condition} | {arm} | {t['successes']}/{t['episodes']} | {r['success_rate']:.2%} | {t['actions']} | {t['diagnostic_queries']} | {t['noise_queries']} | {r['selected_inquiries']} |")
        lines += ["","## Fixed-gate conclusions","",f"Directed-inquiry benefit: **{qualified and benefit}**. Integrated factual chain: **{qualified and chain}** ({chain_worlds}/{manifest['worlds']} worlds, {active_chains} chains).","",
            f"Active versus observe-only mean switching gain: {primary['mean_gain']*100:.3f} percentage points; positive worlds: {primary['positive_worlds']}/{manifest['worlds']}; stationary gain: {primary['stationary_mean_gain']*100:.3f} points; action ratio: {primary['switching_action_ratio']:.4f}.","",
            "A valid negative result means one or more preregistered scientific gates failed. It does not turn evidence-integrity checks into an ability claim.","",
            "## Per-world comparison","","| World | Ordinary | Active | Observe only | Active gain (pp) | Complete chains | Passive behavior identical |","|---|---:|---:|---:|---:|---:|---|"]
        for w in worlds:
            r=w["results"]["switching"]
            lines.append(f"| {w['world']} | {rate(r['ordinary']):.2%} | {rate(r['active']):.2%} | {rate(r['observe_only']):.2%} | {(rate(r['active'])-rate(r['observe_only']))*100:.3f} | {r['active']['complete_chains']} | {w['ordinary_observe_only_behavior_equal']['switching']} |")
        lines += ["","Every arm starts from the same acquired organism. Ordinary/observe-only equality is checked on physical choices and outcomes, excluding inquiry diagnostics and computational accounting. If it differs, equal memory ceilings and the added inquiry reservation may change eviction pressure; the exact traces retain the evidence.",""]
    else:
        lines += ["## Integrity failures",""]+[f"- World {r['world']}: {r['error']}" for r in errors]+[""]
    lines += ["## Reproduction","","`python3 scripts/run_autonomous_inquiry.py --score-only --output <evidence-directory>`",""]
    (output/"RESULT.md").write_text("\n".join(lines),encoding="utf-8")
    print(json.dumps({k:result[k] for k in ("verdict","valid","valid_worlds","conclusions","errors")},indent=2),flush=True)
    return 2 if not valid else 0 if verdict in ("SUPPORTED","DEVELOPMENT") else 1


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary",type=Path,default=ROOT/"target/release/examples/autonomous_inquiry_eval")
    parser.add_argument("--seed",type=int)
    parser.add_argument("--workers",type=int,default=2)
    parser.add_argument("--worlds",type=int,default=16)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--development",action="store_true")
    parser.add_argument("--score-only",action="store_true")
    args=parser.parse_args(); output=args.output.resolve()
    if args.score_only: return summarize(output)
    require(args.seed is not None and 0 <= args.seed < 1 << 64,"--seed must be an unsigned 64-bit value")
    require(1 <= args.worlds <= 16 and 1 <= args.workers <= 16,"world/worker count invalid")
    require(args.development or args.worlds==16,"qualification requires all 16 worlds")
    binary=args.binary.resolve(); require(binary.is_file(),"runtime executable missing")
    prov=provenance(binary,args.development)
    output.mkdir(parents=True,exist_ok=False)
    write_json(output/"manifest.json",{"schema":"aeterna.autonomous-inquiry.manifest.v1","seed":args.seed,"worlds":args.worlds,
        "development":args.development,"workers":args.workers,"provenance":prov,"fixed_gates":GATES})
    execute(output,binary,args.seed,args.worlds,args.workers)
    return summarize(output)


if __name__ == "__main__":
    try: sys.exit(main())
    except Exception as exc:
        print(f"INQUIRY_EXECUTION_ERROR: {exc}",file=sys.stderr); sys.exit(2)
