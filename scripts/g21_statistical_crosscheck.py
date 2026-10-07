#!/usr/bin/env python3
"""Independent arithmetic diagnostic, not the AETERNA cognition implementation.

Uses lgamma rather than Rust's explicit log products to check the registered
binary evidence statistic and the same 512 fixed null streams. It consumes no
fresh authority pack and supplies no runtime state or answers to EvoPhase.
"""
from __future__ import annotations
import json
import math

MASK = (1 << 64) - 1
THRESHOLD = math.log(16 / 0.01)


def log_kt(a: int, b: int) -> float:
    if min(a, b) < 0:
        raise ValueError("Counts must be nonnegative")
    return (math.lgamma(a + 0.5) + math.lgamma(b + 0.5)
            - math.lgamma(a + b + 1) - 2 * math.lgamma(0.5))


def log_evidence(counts: list[list[int]]) -> float:
    if len(counts) != 2 or any(len(row) != 2 for row in counts):
        raise ValueError("Expected a 2-by-2 count matrix")
    a, b = (sum(row[i] for row in counts) for i in range(2))
    n = a + b
    if n == 0:
        return 0.0
    null_mle = sum(x * math.log(x / n) for x in (a, b) if x)
    return sum(log_kt(*row) for row in counts) - null_mle


def gate(counts: list[list[int]], switches: int) -> bool:
    n0, n1 = map(sum, counts)
    if n0 + n1 < 32 or n0 < 8 or n1 < 8 or switches < 4:
        return False
    effect = abs(counts[0][1] / n0 - counts[1][1] / n1)
    return effect >= 0.60 and log_evidence(counts) >= THRESHOLD


def next_rng(x: int) -> int:
    x ^= x >> 12
    x ^= (x << 25) & MASK
    x ^= x >> 27
    return x & MASK


def main() -> None:
    assert not gate([[1, 0], [0, 1]], 1)
    assert not gate([[16, 0], [0, 16]], 1)
    assert gate([[16, 0], [0, 16]], 10)
    assert not gate([[16, 16], [16, 16]], 30)
    crossings: list[tuple[int, int]] = []
    for stream in range(512):
        rng = ((stream + 1) * 0x9E3779B97F4A7C15) & MASK
        counts = [[0, 0], [0, 0]]
        switches = 0
        previous = None
        for sample in range(128):
            rng = next_rng(rng)
            context = ((((rng * 0x2545F4914F6CDD1D) & MASK) >> 60) & 1)
            rng = next_rng(rng)
            cutoff = (1, 2, 3)[stream % 3]
            outcome = int((((rng * 0x2545F4914F6CDD1D) & MASK) >> 62) < cutoff)
            switches += int(previous is not None and previous != context)
            previous = context
            counts[context][outcome] += 1
            if gate(counts, switches):
                crossings.append((stream, sample + 1))
                break
    print(json.dumps({
        "streams": 512,
        "false_promotions": len(crossings),
        "crossings": crossings,
        "deterministic_16_per_context_log_e": log_evidence([[16, 0], [0, 16]]),
        "threshold_log_e": THRESHOLD,
        "scope": "fixed arithmetic/null diagnostic; not an independent task qualification",
    }, indent=2))
    assert len(crossings) <= 15


if __name__ == "__main__":
    main()
