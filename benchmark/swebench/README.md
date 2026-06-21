# SWE-bench adapter (Phase 2)

Maps SWE-bench's repo-patch + pytest model onto holdout's existing `grade`, with
**no changes to holdout**. The thesis it tests: SWE-bench's headline oracle checks
only the `FAIL_TO_PASS` tests, so a patch can be marked **"solved"** while breaking
other behavior — PatchDiff/UTBoost (arXiv 2503.15223 / 2506.09289) put this at
**7.8–29.6%** of "solved" patches. holdout catches exactly those.

## The mapping (this is the whole adapter)

| SWE-bench | holdout |
|---|---|
| a candidate patch applied to the repo | the **candidate command** (`test_runner.sh`): given a pytest test id on stdin, prints `PASS`/`FAIL` |
| `FAIL_TO_PASS` tests (the weak oracle) | **visible** cases, each `expected: "PASS"` |
| `PASS_TO_PASS` tests (regressions it under-weights) | **held-out** cases, each `expected: "PASS"` |
| a behaviorally-wrong "solved" patch | a **false-green**: `visible 100%` but `heldout < 100%`, `gap > 0` |

`make_oracle.py instance.json` builds the oracle; `holdout grade` does the rest.

## Run the demo (no Docker — pure pytest)

```sh
sh run_adapter.sh
```

`instances/calc/` is a synthetic instance with the *same* FAIL_TO_PASS /
PASS_TO_PASS structure as a real one, small enough to run here. Result:

```
gold candidate (correct fix):           heldout 100%  visible 100%  gap 0.00   exit 0
false-green candidate:                  heldout  50%  visible 100%  gap 0.50   exit 1
  first divergence @ c1: input "test_calc.py::test_lower" expected "PASS" got "FAIL"
```

The false-green passes the targeted `test_upper` (FAIL_TO_PASS) — SWE-bench would
call it solved — but breaks the `test_lower` regression (PASS_TO_PASS), which the
held-out half catches. That is the adapter working.

> **Budget note:** each candidate invocation does `cp + pytest`, so `run_adapter.sh`
> raises holdout's per-run wall-clock budget to 30s (the 5s default is for fast
> commands; a cold pytest import alone can approach it). holdout's budget correctly
> kills a too-slow run as a divergence — size it to the workload.

## Running REAL instances

The mapping above is identical for real instances. Two pieces differ:

1. **Metadata** — `fetch_instance.py <offset> > instance.json` pulls a real
   SWE-bench_Verified instance's `FAIL_TO_PASS`/`PASS_TO_PASS` via the HuggingFace
   datasets-server REST API (no deps, no Docker). `make_oracle.py` consumes it
   unchanged (verified: builds a 2-visible / 13-held-out oracle for
   `astropy__astropy-12907`).

2. **Test execution** — real repos need their own environment (numpy/django/etc.
   at pinned versions). This is the **only Docker-bound part**: replace
   `test_runner.sh`'s `cp` line with the official
   [`swebench`](https://github.com/princeton-nlp/SWE-bench) harness step — apply
   the candidate's unified diff inside the instance's Docker image and run the
   given test id there, printing `PASS`/`FAIL`. The holdout grade call above it is
   unchanged.

```sh
pip install swebench datasets        # for a full Docker-backed run
```

## Honest scope

This delivers the **adapter** — the mapping, `make_oracle`, the candidate-command
contract, real-metadata ingestion — and proves it end-to-end on a runnable
instance. It is **not** a 500-instance eval run: that is a heavy Docker compute
exercise (GBs of images, minutes–hours per instance) and is intentionally left as
the operator's step. What's proven here is that holdout, unmodified, flags the
SWE-bench false-green via the FAIL_TO_PASS-vs-PASS_TO_PASS split. Scaling it to
real numbers is turning the crank with `swebench` + Docker, not new design.
