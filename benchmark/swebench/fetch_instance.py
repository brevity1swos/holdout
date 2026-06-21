#!/usr/bin/env python3
"""Fetch a REAL SWE-bench_Verified instance's metadata (no Docker, no deps beyond
stdlib) and write it as an instance.json this adapter's make_oracle.py accepts.

Only the FAIL_TO_PASS / PASS_TO_PASS lists + patch are needed to BUILD the holdout
oracle. Actually RUNNING the tests for a real instance needs the repo's
environment — use the official `swebench` Docker harness for test_runner (see
README). This script exists so the adapter is exercised against real field shapes.

Usage: fetch_instance.py <offset> > instance.json     # e.g. 0
"""
import sys
import json
import urllib.request

OFFSET = int(sys.argv[1]) if len(sys.argv) > 1 else 0
URL = (
    "https://datasets-server.huggingface.co/rows"
    "?dataset=princeton-nlp/SWE-bench_Verified&config=default&split=test"
    f"&offset={OFFSET}&length=1"
)


def main():
    with urllib.request.urlopen(URL, timeout=40) as resp:
        row = json.load(resp)["rows"][0]["row"]
    out = {
        "instance_id": row["instance_id"],
        "repo": row["repo"],
        "base_commit": row["base_commit"],
        "FAIL_TO_PASS": row["FAIL_TO_PASS"],
        "PASS_TO_PASS": row["PASS_TO_PASS"],
    }
    json.dump(out, sys.stdout, indent=2)


if __name__ == "__main__":
    main()
