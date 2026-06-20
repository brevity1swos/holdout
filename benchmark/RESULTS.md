# QuixBugs mechanism gate — results

**Date:** 2026-06-20 · **holdout:** local `target/debug` · **VISIBLE=2**

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
| **Runnable via the JSON command interface** | **31** |
| **Caught by holdout** | **30 / 31 = 97%** |
| **Weak-oracle false-greens** (visible passes, held-out catches) | **6 / 31 = 19%** |
| Missed (no provided test triggers the defect) | 1 (`quicksort`) |

## Why 19% matters

The 6 **false-greens** are the thesis made concrete: the buggy version matches
the correct one on the first 2 (visible) cases — an agent checking only those
examples would ship the bug — but diverges on held-out inputs, which holdout
flags via `heldout_score < 1.0` and `Δ_gap > 0`:

| program | visible | heldout | gap |
|---|---|---|---|
| is_valid_parenthesization | 1.00 | 0.00 | 1.00 |
| shunting_yard | 1.00 | 0.00 | 1.00 |
| to_base | 1.00 | 0.12 | 0.88 |
| longest_common_subsequence | 1.00 | 0.50 | 0.50 |
| lis | 1.00 | 0.60 | 0.40 |
| next_palindrome | 1.00 | 0.67 | 0.33 |

**19% lands squarely inside the literature's 7.8–29.6% band** for
behaviorally-wrong patches that pass weak SWE-bench oracles (PatchDiff/UTBoost) —
independent corroboration of the verification-bottleneck thesis on a different
real-bug corpus.

## A real holdout finding the benchmark surfaced

The buggy `bitcount` (`n ^= n - 1` vs `n &= n - 1`) **infinite-loops**, and
`holdout grade`/`verify` have **no wall-clock budget on candidate execution** —
a non-terminating candidate hangs the grader forever (the first run timed out at
120 s). The spec quarantined execution behind a `Budget { max_wall, max_steps }`,
but the MVP grade/verify path never enforced one. **This is the top hardening
item for holdout** (Plan 5): an ungameable grader for untrusted candidates MUST
bound execution. The benchmark works around it with a 3 s SIGALRM in `runner.py`;
holdout itself should own that timeout.

## Honest caveats

- **31 of 40 programs** ran; 9 graph/linked-list/object-based programs are
  excluded because they take `Node`/`WeightedEdge` objects, not JSON — a limit
  of this harness's stdin interface, not of holdout.
- `quicksort` is "missed" because **QuixBugs' own provided testcases don't
  trigger its defect** — a known limitation of the QuixBugs test sets, not a
  holdout miss. With fresh generated inputs (`holdout verify --generator`) the
  catch-rate would likely rise.
- `VISIBLE=2` is a choice; the false-green count is a function of how many cases
  an agent is assumed to see. More visible cases → fewer false-greens (the bug
  more likely shows in the visible set); the 97% catch-rate is independent of it.
- This is the **mechanism gate** (Phase 1). The **SWE-bench adapter** (Phase 2)
  is deferred pending these results.

## Reproduce

```sh
cd benchmark
git clone --depth 1 https://github.com/jkoppel/QuixBugs.git quixbugs-data
cargo build              # from repo root, produces target/debug/holdout
python3 run_gate.py
```
