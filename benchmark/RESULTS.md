# QuixBugs mechanism gate — results

**Date:** 2026-06-20 · **holdout:** local `target/debug` · **VISIBLE swept 1–5**

Validates holdout's core claim on **real, published bugs**: a weak oracle that
only checks the examples an agent sees ("visible") passes behaviorally-wrong
code; holdout catches it on **held-out** inputs. Each [QuixBugs](https://github.com/jkoppel/QuixBugs)
program's correct version is the sealed reference (via `holdout record`), and
its one-line-bug version is graded against it (via `holdout grade`).

## Headline numbers

| Metric | Value |
|---|---|
| Buggy programs (excl. `node.py` helper) | 40 |
| Excluded — graph/object inputs (no JSON interface) | 9 |
| Excluded — reference unstable/slow (naive `knapsack`/`levenshtein`, O(2ⁿ)) | 2 |
| **Runnable** | **29** |
| **Caught by holdout** | **28 / 29 = 97%** |
| **Weak-oracle false-greens** (visible passes, held-out catches) | **VISIBLE-dependent: 62%→10% (curve below)** |
| Missed (no provided test triggers the defect) | 1 (`quicksort`) |

## The false-green rate is a curve, not a number

A **false-green** is a buggy program that passes every *visible* case but a
held-out one catches — exactly what holdout exists to flag. But the rate depends
entirely on **how many cases the agent is assumed to see**: with fewer visible
cases, more bugs hide in the held-out region. So a single headline number would
be cherry-picked. The honest report is the sensitivity curve:

| VISIBLE (cases the agent sees) | caught | **false-greens** |
|---|---|---|
| 1 | 29/29 = 100% | **18/29 = 62%** |
| 2 | 28/29 = 97% | **6/29 = 21%** |
| 3 | 28/29 = 97% | **5/29 = 17%** |
| 5 | 27/29 = 93% | **3/29 = 10%** |

The trend is the real finding: **the more an agent already knows (more examples),
the less holdout's held-out catching adds** — which matches the dogfood result
(on well-known tasks an agent effectively has "many examples" and one-shots, so
holdout validates rather than catches). holdout earns its keep where the agent is
under-specified relative to the true behavior.

> **On the literature comparison (honesty note):** the false-green rate overlaps
> the 7.8–29.6% band reported for behaviorally-wrong patches that pass weak
> SWE-bench oracles (PatchDiff/UTBoost) — but this is **suggestive, not
> equivalent**. Those numbers measure PR-modified tests missing regressions in
> *other* functionality; ours measures a held-out split of a toy algorithm's own
> tests. Same ballpark, genuinely different phenomena. A real head-to-head needs
> the deferred SWE-bench adapter. N=29 is small; treat these as a mechanism
> demonstration, not a benchmark result.

## A real holdout finding the benchmark surfaced — now fixed and validated

The buggy `bitcount` (`n ^= n - 1` vs `n &= n - 1`) **infinite-loops**, and the
first gate run exposed that `holdout grade`/`verify` had **no wall-clock budget
on candidate execution** — a non-terminating candidate hung the grader forever
(timed out at 120 s). An ungameable grader for *untrusted* candidates must bound
execution.

**Fixed in holdout core** (`Candidate::with_timeout`, `grade/verify --timeout-ms`,
default 5000): each candidate run is killed at the budget and recorded as a
divergence. The gate now runs **with no harness-side timeout on candidates** —
holdout's own budget handles them:

```
$ holdout grade --oracle bitcount.oracle.json \
    --candidate 'python runner.py python_programs bitcount' --timeout-ms 2000 --json
exit 1 | wall 18.1s | {"heldout_score":0.0, ...,
  "first_divergence":{"input":"[13]","expected":"3","actual":"<timed out>"}}
```

18.1 s = ~9 infinite-loop cases × 2 s budget — **bounded, not hung**, and
correctly reported as a divergence. (The harness retains an *env-gated* net only
for the trusted *reference* during `record`, because a few correct QuixBugs
programs are pathologically slow; holdout does not time trusted references.)

## Honest caveats

- **29 of 40 programs** ran; 9 graph/linked-list/object-based programs are
  excluded because they take `Node`/`WeightedEdge` objects, not JSON (a limit of
  this harness's stdin interface, not of holdout), and 2 (`knapsack`,
  `levenshtein`) are excluded because their naive O(2ⁿ) *correct* reference is
  too slow to establish ground truth — the harness flags these rather than
  scoring against a polluted oracle.
- `quicksort` is "missed" because **QuixBugs' own provided testcases don't
  trigger its defect** — a known limitation of the QuixBugs test sets, not a
  holdout miss. With fresh generated inputs (`holdout verify --generator`) the
  catch-rate would likely rise.
- The false-green rate's dependence on VISIBLE is reported as a curve above, not
  a single number — see "The false-green rate is a curve, not a number."
- This is the **mechanism gate** (Phase 1). The **SWE-bench adapter** (Phase 2)
  is deferred pending these results.

## Reproduce

```sh
cd benchmark
git clone --depth 1 https://github.com/jkoppel/QuixBugs.git quixbugs-data
cargo build              # from repo root, produces target/debug/holdout
python3 run_gate.py
```
