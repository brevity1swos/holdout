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
- **Not shown here:** a real *false-green*. Gold patches resolve every instance by
  construction, so they cannot produce one. Two honest caveats on what a
  false-green even means (see README "which weak oracle?"):
  - A patch that passes `FAIL_TO_PASS` but breaks `PASS_TO_PASS` is **not** a
    SWE-bench false-green — SWE-bench's "resolved" metric checks P2P, so it catches
    that. It only fools a naive `FAIL_TO_PASS`-only check (Mode A).
  - The real PatchDiff/UTBoost false-green is a patch SWE-bench marks **resolved**
    (F2P+P2P pass) that an **augmented** test exposes. To catch one, run a model's
    *resolved* prediction against the UTBoost dataset (Mode B):

```sh
sh run_docker_eval.sh --dataset uiuc-kang-lab/SWE-bench-Verified-UTBoost \
   --predictions model_preds.jsonl <instance_ids the model resolved...>
# false-green => visible (original tests) 100% / heldout (augmented) < 100% / gap > 0
```

## Cost note (arm64)

One light, pure-Python instance (`requests`) built + ran in ~1–3 min via local
arm64 build. Heavier repos (numpy/scientific stacks) build much slower; a full
500-instance Verified sweep is a serious compute job and is not attempted here.
