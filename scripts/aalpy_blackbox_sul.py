#!/usr/bin/env python3
"""An external REAL SUL driver: no synthetic outcomes inside AETERNA.

Upstream AALpy MIT, EXACT git commit:
335c50a1b4b5a830d788f86b7c30a8cb6c73ecc0
get_faulty_coffee_machine_SMM is third-party authored.
"""
import json
import random
import sys
from aalpy.utils.BenchmarkSULs import get_faulty_coffee_machine_SMM
from aalpy.SULs import AutomatonSUL

sul = AutomatonSUL(get_faulty_coffee_machine_SMM())
map_motors = ["coin", "but"]

def write(record):
    sys.stdout.write(json.dumps(record, separators=(",", ":")) + "\n")
    sys.stdout.flush()

for line in sys.stdin:
    try:
        q = json.loads(line)
        op = q.get("op")
        if op == "reset":
            seed = q["seed"]
            assert isinstance(seed, int) and 0 <= seed < (1 << 64)
            order = q["map"]
            assert order in ([0, 1], [1, 0])
            map_motors = [["coin", "but"][i] for i in order]
            random.seed(seed)
            sul.pre()
            write({"ok": True, "observation": [0, 0], "output": "init"})
        elif op == "step":
            motor = q["motor"]
            assert isinstance(motor, int) and 0 <= motor < 2
            output = sul.step(map_motors[motor])
            assert output in ("init", "beep", "coffee"), repr(output)
            observed = {"init": [0, 0], "beep": [0, 1],
                        "coffee": [1, 0]}[output]
            write({"ok": True, "observation": observed, "output": output,
                   "actual_step": map_motors[motor], "cost": 1})
        else:
            raise ValueError("Unsupported SUL operation")
    except Exception as exc:
        write({"ok": False, "error": type(exc).__name__ + ": " + str(exc)})
        sys.exit(1)
