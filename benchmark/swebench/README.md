# SWE-bench adapter (Phase 2)

Maps SWE-bench's repo-patch + pytest model onto holdout's existing `grade`, with
**no changes to holdout**: the "candidate" is `test_runner.sh` (a pytest test id on
stdin → `PASS`/`FAIL`), each case's `expected` is `"PASS"`, and a **false-green**
is `visible 100%` but `heldout < 100%` (`gap > 0`).

> [!IMPORTANT] **Honest framing — which weak oracle?** SWE-bench's own "resolved"
> metric checks **both** `FAIL_TO_PASS` **and** `PASS_TO_PASS`. So the split below
> does **not** reproduce the PatchDiff/UTBoost finding by itself — those are
> patches SWE-bench marks *resolved* (F2P+P2P both pass) that are *still* wrong,
> caught only by **augmented** tests beyond both sets. This adapter has two modes:

**Mode A — naive-oracle demo (default).** visible = `FAIL_TO_PASS`,
held-out = `PASS_TO_PASS`. This models a **`FAIL_TO_PASS`-only** weak oracle — a
"did my new test pass?" check (a real agent/CI failure mode) — and shows holdout's
held-out catches the regressions it misses. It is **not** "what SWE-bench misses"
(SWE-bench checks P2P too).

**Mode B — faithful literature finding.** visible = original `FAIL_TO_PASS` +
`PASS_TO_PASS` (= SWE-bench "resolved"), held-out = **UTBoost augmented tests**.
Run `run_docker_eval.sh` against `--dataset uiuc-kang-lab/SWE-bench-Verified-UTBoost`
with a model's *resolved* prediction: a patch that passes the original oracle but
fails an augmented held-out test is the real **7.8–29.6%** false-green
(PatchDiff/UTBoost, arXiv 2503.15223 / 2506.09289) that SWE-bench's full oracle
missed. See `DOCKER_RUN.md` for the run.

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
