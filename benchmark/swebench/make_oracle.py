#!/usr/bin/env python3
"""Map a SWE-bench(-style) instance to a holdout oracle (printed to stdout).

The mapping is the whole point of the adapter:
  - VISIBLE cases  = FAIL_TO_PASS test ids — the tests SWE-bench's weak oracle
    checks to call a patch "solved".
  - HELD-OUT cases = PASS_TO_PASS test ids — the regression tests the headline
    metric under-weights.
  - Each case's `input` is a pytest test id; its `expected` output is "PASS".

A candidate that passes every VISIBLE test but fails a HELD-OUT one is a
behaviorally-wrong "solved" patch — the false-green PatchDiff/UTBoost report
(7.8-29.6% of SWE-bench "solved" patches). holdout flags it as heldout_score < 1
with delta_gap > 0, using the unmodified `holdout grade`.

Usage: make_oracle.py instance.json > oracle.json
"""
import sys
import json


def ids(v):
    # FAIL_TO_PASS / PASS_TO_PASS are JSON-string lists in the real dataset.
    return json.loads(v) if isinstance(v, str) else list(v)


def main():
    inst = json.load(open(sys.argv[1]))
    visible = [
        {"name": f"F2P{i}", "input": t, "expected": "PASS"}
        for i, t in enumerate(ids(inst["FAIL_TO_PASS"]))
    ]
    heldout = [
        {"name": f"P2P{i}", "input": t, "expected": "PASS"}
        for i, t in enumerate(ids(inst["PASS_TO_PASS"]))
    ]
    json.dump({"kind": "HeldoutCases", "visible": visible, "heldout": heldout}, sys.stdout)


if __name__ == "__main__":
    main()
