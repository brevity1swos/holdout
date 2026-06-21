#!/usr/bin/env python3
"""Extract per-test PASS/FAIL from a swebench run_evaluation report for one instance.

swebench writes, per instance, a report with
  tests_status: { FAIL_TO_PASS: {success:[...], failure:[...]},
                  PASS_TO_PASS: {success:[...], failure:[...]}, ... }
We flatten that to {test_id: "PASS"|"FAIL"} (the candidate's real in-Docker
behavior) and write it to <results_out>, which replay.py serves to holdout grade.

Usage: report_to_results.py <report.json> <instance_id> <results_out.json>
"""
import sys
import json


def main():
    report = json.load(open(sys.argv[1]))
    iid = sys.argv[2]
    out = sys.argv[3]
    entry = report[iid] if iid in report else next(iter(report.values()))
    status = entry["tests_status"]
    results = {}
    for grp in ("FAIL_TO_PASS", "PASS_TO_PASS"):
        for t in status[grp]["success"]:
            results[t] = "PASS"
        for t in status[grp]["failure"]:
            results[t] = "FAIL"
    json.dump(results, open(out, "w"))
    print(f"wrote {len(results)} test outcomes to {out} (resolved={entry.get('resolved')})")


if __name__ == "__main__":
    main()
