# Real SWE-bench Docker eval — run log

**Date:** 2026-06-20 · macOS arm64 (aarch64, 8 CPU / 8 GB) · Docker Desktop ·
`swebench` harness, `--namespace none` (native arm64 image build).

This is a **real** run: the SWE-bench harness built a Docker image and ran pytest
inside it; holdout graded the genuine in-Docker results (not simulated).

## Instance: `psf__requests-1142` (gold prediction)

```
swebench: built base + env image; ran in Docker in 62.9s; Instances resolved: 1
real tests_status: FAIL_TO_PASS 1 pass / 0 fail ; PASS_TO_PASS 5 pass / 0 fail

holdout grade (on the REAL in-Docker results):
  heldout 100%  visible 100%  gap 0.00  reward 1.00   exit 0
```

The gold (reference) patch resolves the instance, so every FAIL_TO_PASS and
PASS_TO_PASS test passes → holdout reports a clean solve. **This validates the
entire real pipeline end to end:** swebench Docker eval → `report.json` →
`report_to_results.py` → `holdout grade` via `replay.py`, against an oracle built
from the real `FAIL_TO_PASS`/`PASS_TO_PASS` split.

## What this proves, and what it doesn't

- **Proven:** holdout grades real, in-Docker SWE-bench test results, with the
  FAIL_TO_PASS = visible / PASS_TO_PASS = held-out mapping, on a real instance.
- **Not shown here:** a real *false-green* on Docker. Gold patches resolve every
  instance by construction, so they cannot produce one. Catching a real
  false-green needs a **model's prediction set** — run instances the model marked
  "solved" (passes FAIL_TO_PASS) and look for ones where a held-out PASS_TO_PASS
  test fails (`visible 100% / heldout < 100% / gap > 0`). The mechanism is already
  demonstrated on simulated-real results for this exact instance (a broken
  `test_basic_building` → heldout 80%, gap 0.20; see `run_docker_eval.sh` + the
  synthetic `instances/calc`). To hunt one in the wild:

```sh
# get a model's predictions for SWE-bench_Verified, then:
sh run_docker_eval.sh --predictions model_preds.jsonl <instance_ids the model resolved...>
```

## Cost note (arm64)

One light, pure-Python instance (`requests`) built + ran in ~1–3 min via local
arm64 build. Heavier repos (numpy/scientific stacks) build much slower; a full
500-instance Verified sweep is a serious compute job and is not attempted here.
