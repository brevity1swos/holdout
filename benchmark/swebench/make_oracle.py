#!/usr/bin/env python3
"""Map a SWE-bench(-style) instance to a holdout oracle (printed to stdout).

The mapping: each case's `input` is a pytest test id, `expected` is "PASS"; a
candidate passing every VISIBLE test but failing a HELD-OUT one is a false-green
(heldout_score < 1, delta_gap > 0), via the unmodified `holdout grade`.

Which tests go where depends on the mode (see README "which weak oracle?"):
  - Mode A (FAIL_TO_PASS-only weak oracle): visible = FAIL_TO_PASS,
    held-out = PASS_TO_PASS. Catches regressions a "did my new test pass?" check
    misses. NOT what SWE-bench misses (its "resolved" metric checks P2P too).
  - Mode B (faithful): run against the UTBoost dataset so the augmented tests are
    folded into FAIL_TO_PASS/PASS_TO_PASS; a patch SWE-bench marks *resolved* that
    fails an augmented test is the real PatchDiff/UTBoost false-green.

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
