#!/bin/sh
# The "candidate command" holdout grades. holdout writes ONE pytest test id to
# stdin per invocation; this prints PASS or FAIL for that test against the
# candidate-patched repo.
#
#   args: <repo_dir> <candidate_file>
#   stdin: a pytest test id (e.g. "test_calc.py::test_upper")
#
# For this synthetic instance the "patch" is a full replacement of calc.py
# (robust + Docker-free). For a REAL SWE-bench instance, replace the cp line with
# the official swebench harness step (apply the unified diff inside the instance's
# Docker image, run pytest there) — that is the only Docker-bound part; the
# holdout mapping above it is identical. See README.md.
repo="$1"
candidate="$2"
read tid
work=$(mktemp -d)
cp -R "$repo"/. "$work"/
cp "$candidate" "$work/calc.py"
if (cd "$work" && python3 -m pytest -q -p no:cacheprovider "$tid" >/dev/null 2>&1); then
  echo PASS
else
  echo FAIL
fi
rm -rf "$work"
