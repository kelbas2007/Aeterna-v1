#!/usr/bin/env python3
"""Post-exam oracle diagnostic. Never import this module into the solver.

Uses gold equations AFTER the frozen examination to distinguish missing
expressive capacity from selection failure. Counts are not solved-task scores.
Usage: python scripts/word_problem_coverage_audit.py <extracted Linux evidence>
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
from typing import Any
from word_problem_verify import equal_program, gold_tree, quantities, run


def load_checked(path: Path, expected: str) -> Any:
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError(f"Evidence checksum mismatch: {path}")
    return json.loads(raw)


def audit(root: Path) -> dict[str, Any]:
    external = root / "external"
    summary = json.loads((external / "external-summary.json").read_text(encoding="utf-8"))
    freeze = json.loads((external / "freeze.json").read_text(encoding="utf-8"))
    rows = load_checked(external / "SVAMP-original.json", summary["test_sha256"])
    model = load_checked(root / "learned/role-model.json", freeze["learned_models"]["role"])
    raw = (external / "full-predictions.jsonl").read_bytes()
    if hashlib.sha256(raw).hexdigest() != summary["traces"]["full"]["predictions_sha256"]:
        raise ValueError("Prediction checksum mismatch")
    predictions = [json.loads(line) for line in raw.splitlines()]
    if len(rows) != len(predictions) or len(rows) != summary["test_records"]:
        raise ValueError("Incomplete evidence")
    counts = dict(symbolically_bindable=0, correct_program_available=0,
                  selected_equivalent=0, correct_program_in_top3=0,
                  wrong_selection_despite_available=0, unavailable_reference=0,
                  ambiguous_reference=0)
    cases = []
    for index, (row, entry) in enumerate(zip(rows, predictions)):
        question = row["Body"].strip() + " " + row["Question"].strip()
        if entry["index"] != index or entry["question"] != question:
            raise ValueError(f"Misaligned evidence at {index}")
        values = quantities(question)
        try:
            gold = gold_tree(row["Equation"], values)
        except (ValueError, KeyError, SyntaxError):
            counts["ambiguous_reference"] += 1
            cases.append({"index": index, "status": "unbound_reference"})
            continue
        counts["symbolically_bindable"] += 1
        eligible = []
        for program in model["programs"]:
            expr = program["expression"]
            try:
                run(expr, values)
            except (IndexError, ZeroDivisionError):
                continue
            eligible.append(expr)
        interpretations = entry["prediction"]["interpretations"]
        available = any(equal_program(e, gold, len(values)) for e in eligible)
        selected = bool(interpretations) and equal_program(interpretations[0]["program"], gold, len(values))
        top3 = any(equal_program(h["program"], gold, len(values)) for h in interpretations[:3])
        counts["correct_program_available"] += available
        counts["selected_equivalent"] += selected
        counts["correct_program_in_top3"] += top3
        counts["wrong_selection_despite_available"] += available and not selected
        counts["unavailable_reference"] += not available
        cases.append({"index": index, "available": available, "selected": selected, "in_top3": top3})
    return {"stage": "POST_EVALUATION_DIAGNOSTIC_NOT_SOLVER_INPUT", **counts,
            "caveat": "Gold equations are used only here. Repeated ambiguous numeric references are excluded from this diagnostic, not from the 1000-task primary score. Symbolic equality is not proof of language understanding.",
            "cases": cases}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    args = parser.parse_args()
    result = audit(args.folder)
    (args.folder / "posthoc-program-coverage.json").write_text(
        json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({k: v for k, v in result.items() if k != "cases"}, indent=2))


if __name__ == "__main__":
    main()
