#!/usr/bin/env python3
"""Regression: untrusted JSONL width must fail BEFORE default allocation.

This is ordinary engineering evidence, NOT a one-use scientific experiment.
"""
import argparse
import json
import subprocess
from pathlib import Path

def exchange(binary, requests):
    p=subprocess.run([str(binary), "--jsonl"],
        input="".join(json.dumps(q,separators=(",",":"))+"\n" for q in requests),
        text=True,encoding="utf-8",capture_output=True,timeout=12,check=False)
    replies=[]
    for line in p.stdout.splitlines():
        replies.append(json.loads(line))
    return p,replies

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--binary",type=Path,required=True)
    args=ap.parse_args()
    b=args.binary.resolve()
    for bad in (0,33,4294967296,18446744073709551615):
        p,replies=exchange(b,[{"op":"init","width":bad}])
        assert p.returncode != 0,(bad,p.returncode)
        assert len(replies)==1 and replies[0]["type"]=="error",(bad,replies,p.stderr)
        assert "capacity" in replies[0]["message"].lower(),(bad,replies)
    p,replies=exchange(b,[{"op":"init","width":1},{"op":"quit"}])
    assert p.returncode==0,(p.returncode,p.stderr)
    assert [r["type"] for r in replies]==["initialized","bye"],replies
    print(json.dumps({"status":"PASS","bad_widths_rejected_before_allocation":4,
        "valid_small_default_observation_roundtrip":True}))

if __name__=="__main__":
    main()
