# Conclusive: holdout catches a real SWE-bench false-green (real Docker)

**Date:** 2026-06-21 · macOS arm64 · real `swebench` Docker harness + UTBoost.

This is the faithful result the whole SWE-bench thread was after: holdout catches a
behaviorally-wrong patch that **SWE-bench's own official oracle marked "resolved"** —
the PatchDiff/UTBoost false-green (7.8–29.6% of "solved" patches), on real data.

## The catch

- **Instance:** `django__django-16485` (the `floatformat` template filter — the
  original tests don't cover zero-case handling).
- **Patch:** the submission from **SWE-agent + Claude 3.5 Sonnet**, which SWE-bench
  Verified officially marks **resolved** for this instance.
- **Real Docker eval** (`--namespace none`, native arm64 build) against the
  **UTBoost augmented** dataset → real in-Docker pytest results:
  - **10 original SWE-bench tests: all PASS** → SWE-bench's oracle says "solved".
  - **1 UTBoost augmented test: FAIL** —
    `test_additional_floatformat_zero_cases` exposes the patch as wrong.

## holdout's verdict (faithful Mode B — unmodified `holdout grade`)

visible = the 10 original tests (what SWE-bench checks); held-out = the augmented test.

```
heldout 0%   visible 100%   gap 1.00   reward -1.00   exit 1
first divergence: test_additional_floatformat_zero_cases  expected "PASS" got "FAIL"
```

A patch that is **100% green on SWE-bench's full oracle** is flagged by holdout as
behaviorally wrong, via a held-out test the official metric never ran. That is the
verification-bottleneck thesis demonstrated on real, in-the-wild model output.

## How it was found (and the rigor that matters)

UTBoost augments only **26 of 500** Verified instances (the ones with weak
coverage). Intersecting those with the instances this model **resolved** gave 10
false-green *candidates*. Running them against the UTBoost dataset in Docker and
applying a strict test:

> a CLEAN false-green requires **every original test to PASS** (the patch really
> does reproduce "resolved") **and** an **augmented** test to FAIL.

- **`django-16485`** → original 10/10 pass, augmented 1/1 fail → **clean false-green.**
- `django-11133`, `sympy-16450/20154/21847` → original + augmented all pass →
  **genuinely correct patches** (correctly *not* flagged).
- An earlier candidate (`pylint-6528`) was **rejected**: it showed 136/171
  original tests failing — a pure **arm64 reproduction artifact** (and UTBoost had
  not even augmented it; identical `test_patch`), not a false-green. Naive
  "resolved-on-original XOR resolved-on-UTBoost" would have falsely flagged it;
  the original-tests-must-pass check rejects it.

## Extended sweep (honest tally)

Ran all 10 candidates (3 django, 3 sympy, 2 xarray, 2 sklearn — `pylint` was in an
earlier batch) against UTBoost in real Docker. Outcome:

- **1 clean false-green:** `django-16485` (above).
- **Correct patches, correctly NOT flagged:** `sympy-16450/20154/21847`,
  `django-11133`, `xarray-3305` (original + augmented both pass).
- **Reproduction artifacts, correctly REJECTED** by the all-original-must-pass
  rule: `pylint-6528`, `xarray-4687` (original tests failed → arm64 env, not a
  behavioral catch).
- **Build/run failed on arm64:** both `scikit-learn` instances, `django-14915`,
  `pylint-6903` (scientific-stack / parallel-build issues under native arm64).

So this is **one confirmed false-green, not a measured rate** — the full augmented
sweep (26 instances × many models) would need x86 hardware to run reliably. The
existence proof on real Docker is what's done.

## Reproduce

```sh
# model patches for resolved instances come from the SWE-bench experiments S3 bucket;
# build a predictions file, then:
sh run_docker_eval.sh --dataset uiuc-kang-lab/SWE-bench-Verified-UTBoost \
   --predictions <model_preds.json> django__django-16485
# then split visible=original / heldout=augmented and `holdout grade` (see this dir's scripts)
```
