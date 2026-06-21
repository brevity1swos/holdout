#!/usr/bin/env python3
"""Write one instance's FAIL_TO_PASS / PASS_TO_PASS to stdout as make_oracle input.

Uses the local `datasets` cache (no Docker). Usage: fetch_by_id.py <instance_id>
"""
import sys
import json
from datasets import load_dataset

iid = sys.argv[1]
dataset = sys.argv[2] if len(sys.argv) > 2 else "princeton-nlp/SWE-bench_Verified"
ds = load_dataset(dataset, split="test")
row = next(r for r in ds if r["instance_id"] == iid)
json.dump(
    {
        "instance_id": row["instance_id"],
        "repo": row["repo"],
        "FAIL_TO_PASS": row["FAIL_TO_PASS"],
        "PASS_TO_PASS": row["PASS_TO_PASS"],
    },
    sys.stdout,
)
