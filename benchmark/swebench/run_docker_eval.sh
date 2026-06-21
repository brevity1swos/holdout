#!/bin/sh
# REAL SWE-bench Docker eval on a subset, graded through holdout.
# Requires: Docker running + `pip install swebench datasets`.
#
#   sh run_docker_eval.sh [--predictions PATH] <instance_id> [<instance_id> ...]
#
# Default predictions = `gold` (the reference patch — resolves every instance, so
# it validates the pipeline: visible 100% / heldout 100%). Pass --predictions with
# a model's predictions file to look for REAL false-greens (a patch that passes
# FAIL_TO_PASS but breaks PASS_TO_PASS → holdout flags visible 100% / heldout <100%).
set -e
dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
HOLDOUT="$dir/../../target/debug/holdout"
DATASET=princeton-nlp/SWE-bench_Verified
PREDS=gold
case "$1" in --predictions) PREDS="$2"; shift 2 ;; esac
work="$dir/.docker_work"; mkdir -p "$work"

for iid in "$@"; do
  echo "================= $iid (predictions=$PREDS) ================="
  run_id="holdout_$(printf '%s' "$iid" | tr -c 'A-Za-z0-9' _)"

  # 1. REAL eval in Docker (builds/pulls the instance image, runs pytest inside).
  python3 -m swebench.harness.run_evaluation \
    --dataset_name "$DATASET" --predictions_path "$PREDS" \
    --instance_ids "$iid" --run_id "$run_id" --max_workers 1 --cache_level instance

  # 2. Locate the per-instance report and flatten to real PASS/FAIL outcomes.
  report=$(find "$dir/.." "$dir" ./logs . -path "*${run_id}*/${iid}/report.json" 2>/dev/null | head -1)
  [ -z "$report" ] && report=$(find . -name report.json -path "*${iid}*" 2>/dev/null | head -1)
  echo "report: $report"
  python3 "$dir/report_to_results.py" "$report" "$iid" "$work/$iid.results.json"

  # 3. Build the holdout oracle (visible=FAIL_TO_PASS, heldout=PASS_TO_PASS) + seal.
  python3 "$dir/fetch_by_id.py" "$iid" > "$work/$iid.instance.json"
  python3 "$dir/make_oracle.py" "$work/$iid.instance.json" > "$work/$iid.oracle.json"
  SEAL=$("$HOLDOUT" seal --oracle "$work/$iid.oracle.json")

  # 4. holdout grades the REAL in-Docker test results.
  echo "--- holdout grade (real Docker results) ---"
  "$HOLDOUT" grade --oracle "$work/$iid.oracle.json" \
    --candidate "python3 $dir/replay.py $work/$iid.results.json" --seal "$SEAL" || true
  echo
done
