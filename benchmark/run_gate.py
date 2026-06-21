#!/usr/bin/env python3
"""QuixBugs mechanism gate for holdout.

For each QuixBugs program that has a buggy version:
  1. write its test inputs (one JSON arg-array per line),
  2. `holdout record` the CORRECT version's behavior (first V cases visible,
     the rest held out), sealed,
  3. `holdout grade` the BUGGY version against that sealed oracle.

We then classify each program by what holdout reported:
  - caught                 heldout_score < 1.0  (holdout flagged the bug)
  - weak-oracle false-green  visible_score == 1.0 AND heldout_score < 1.0
                           (an agent seeing only the visible cases would have
                            shipped the bug; holdout caught it on held-out inputs)
  - missed                 passed (no test input triggers the defect)
  - incompatible           the correct reference can't run via the JSON
                           interface (graph/object inputs) — reported, not hidden

This validates the core thesis on REAL bugs using holdout's own record+grade.
"""
import sys
import json
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "quixbugs-data")
CORRECT = os.path.join(DATA, "correct_python_programs")
BUGGY = os.path.join(DATA, "python_programs")
TESTS = os.path.join(DATA, "json_testcases")
RUNNER = os.path.join(HERE, "runner.py")
HOLDOUT = os.path.join(HERE, "..", "target", "debug", "holdout")
WORK = os.path.join(HERE, ".work")
VISIBLE = int(os.environ.get("VISIBLE", "2"))  # cases an "agent" sees; rest held out
# Bound only the TRUSTED reference during record (a few correct QuixBugs programs
# are pathologically slow, e.g. naive knapsack). The buggy CANDIDATE is bounded
# by holdout itself (grade --timeout-ms) — that is what the gate validates.
REF_ENV = {**os.environ, "RUNNER_MAX_SECONDS": "10"}

os.makedirs(WORK, exist_ok=True)


def buggy_programs():
    names = []
    for fn in sorted(os.listdir(BUGGY)):
        if not fn.endswith(".py") or fn.endswith("_test.py"):
            continue
        name = fn[:-3]
        if (
            os.path.exists(os.path.join(CORRECT, fn))
            and os.path.exists(os.path.join(TESTS, name + ".json"))
        ):
            names.append(name)
    return names


def write_inputs(name):
    path = os.path.join(WORK, name + ".inputs")
    n = 0
    with open(os.path.join(TESTS, name + ".json")) as src, open(path, "w") as out:
        for line in src:
            line = line.strip()
            if not line:
                continue
            tc = json.loads(line)
            out.write(json.dumps(tc[0]) + "\n")  # tc = [args, expected]
            n += 1
    return path, n


def reference_runs(name):
    """Sanity check: does the CORRECT version run on its first input without error?"""
    with open(os.path.join(TESTS, name + ".json")) as f:
        first = json.loads(f.readline())
    out = subprocess.run(
        [sys.executable, RUNNER, CORRECT, name],
        input=json.dumps(first[0]),
        capture_output=True,
        text=True,
        timeout=20,
        env=REF_ENV,
    ).stdout.strip()
    return "__error__" not in out


def gate_one(name):
    inputs_path, n = write_inputs(name)
    if n < 2:
        return {"name": name, "class": "incompatible", "reason": "too few testcases"}
    if not reference_runs(name):
        return {"name": name, "class": "incompatible", "reason": "reference needs non-JSON inputs"}

    visible = min(VISIBLE, n - 1)
    oracle = os.path.join(WORK, name + ".oracle.json")
    rec = subprocess.run(
        [HOLDOUT, "record",
         "--reference", f"{sys.executable} {RUNNER} {CORRECT} {name}",
         "--inputs", inputs_path, "--visible", str(visible), "--out", oracle],
        capture_output=True, text=True, timeout=200, env=REF_ENV,
    )
    if rec.returncode != 0:
        return {"name": name, "class": "incompatible", "reason": "record failed: " + rec.stderr.strip()[:60]}
    # If the trusted reference timed out/errored on any input, there is no
    # reliable ground truth — exclude rather than score against a bad oracle.
    with open(oracle) as f:
        spec = json.load(f)
    if any("__error__" in c["expected"] for c in spec["visible"] + spec["heldout"]):
        return {"name": name, "class": "incompatible", "reason": "reference unstable (too slow / errors)"}

    grd = subprocess.run(
        [HOLDOUT, "grade", "--oracle", oracle,
         "--candidate", f"{sys.executable} {RUNNER} {BUGGY} {name}",
         "--timeout-ms", "2000", "--json"],  # holdout bounds the buggy candidate
        capture_output=True, text=True, timeout=300,
    )
    try:
        r = json.loads(grd.stdout.strip().splitlines()[-1])
    except (json.JSONDecodeError, IndexError):
        return {"name": name, "class": "incompatible", "reason": "grade no json: " + grd.stderr.strip()[:60]}

    vis, held, gap = r["visible_score"], r["heldout_score"], r["delta_gap"]
    if held >= 1.0:
        cls = "missed"
    elif vis >= 1.0:
        cls = "false-green"  # visible passes, held-out catches → the thesis case
    else:
        cls = "caught"
    return {"name": name, "class": cls, "visible": vis, "heldout": held, "gap": gap, "cases": n}


def main():
    names = buggy_programs()
    results = [gate_one(name) for name in names]

    runnable = [r for r in results if r["class"] in ("caught", "false-green", "missed")]
    caught = [r for r in runnable if r["class"] in ("caught", "false-green")]
    false_green = [r for r in runnable if r["class"] == "false-green"]
    incompatible = [r for r in results if r["class"] == "incompatible"]

    print(f"QuixBugs mechanism gate — VISIBLE={VISIBLE}\n")
    print(f"{'program':<26} {'class':<12} {'visible':>7} {'heldout':>7} {'gap':>5}")
    print("-" * 62)
    for r in sorted(results, key=lambda x: (x["class"], x["name"])):
        if r["class"] == "incompatible":
            print(f"{r['name']:<26} {'incompatible':<12} {r['reason']}")
        else:
            print(f"{r['name']:<26} {r['class']:<12} {r['visible']:>7.2f} {r['heldout']:>7.2f} {r['gap']:>5.2f}")

    n_buggy = len([f for f in os.listdir(BUGGY) if f.endswith(".py") and not f.endswith("_test.py")]) - 1
    print("\n=== SUMMARY ===")
    print(f"buggy programs (excl. node helper) : {n_buggy}")
    print(f"  excluded pre-attempt (graph/object, no JSON testcases): {n_buggy - len(results)}")
    print(f"  attempted via JSON iface         : {len(results)}")
    print(f"    reference unstable/slow (excluded): {len(incompatible)}")
    print(f"    runnable                        : {len(runnable)}")
    if runnable:
        print(f"  caught by holdout      : {len(caught)}/{len(runnable)} = {100*len(caught)/len(runnable):.0f}%")
        print(f"  WEAK-ORACLE FALSE-GREEN: {len(false_green)}/{len(runnable)} = {100*len(false_green)/len(runnable):.0f}%")
        print("    (visible examples pass, held-out inputs catch the bug — the thesis)")
        print(f"  missed (no test triggers): {len(runnable)-len(caught)}")


if __name__ == "__main__":
    main()
