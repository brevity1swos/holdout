#!/bin/sh
# End-to-end adapter demo on the synthetic instance: holdout catches a
# behaviorally-wrong "solved" patch that SWE-bench's weak (FAIL_TO_PASS) oracle
# would pass, via the held-out PASS_TO_PASS tests.
set -e
dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
HOLDOUT="$dir/../../target/debug/holdout"
inst="$dir/instances/calc"
oracle=$(mktemp)

python3 "$dir/make_oracle.py" "$inst/instance.json" >"$oracle"
SEAL=$("$HOLDOUT" seal --oracle "$oracle")
runner="sh $dir/test_runner.sh $inst/repo"
# Each candidate invocation does `cp + pytest` — heavyweight vs a plain function,
# so the per-run wall-clock budget is raised from the 5s default (a cold pytest
# import alone can approach it). For real Docker-backed instances, raise further.
budget="--timeout-ms 30000"

echo "=== gold candidate (correct fix) ==="
"$HOLDOUT" grade --oracle "$oracle" --candidate "$runner $inst/gold_calc.py" --seal "$SEAL" $budget || true
echo

echo "=== false-green candidate (passes FAIL_TO_PASS, breaks a PASS_TO_PASS regression) ==="
"$HOLDOUT" grade --oracle "$oracle" --candidate "$runner $inst/false_calc.py" --seal "$SEAL" $budget || true
echo
echo "(visible = FAIL_TO_PASS; heldout = PASS_TO_PASS. A false-green shows visible 100%"
echo " but heldout < 100% with gap > 0 — the wrong-but-'solved' patch SWE-bench misses.)"
rm -f "$oracle" "$oracle.seal"
