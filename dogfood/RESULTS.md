# Dogfood — holdout in a real agent loop (Phase 0)

**Date:** 2026-06-20 · the go/no-go gate from the weakness-remediation plan.

**Setup:** a subagent acting as the *coding agent* implements `candidate.py` from a
few visible examples, verifying ONLY through `./check candidate.py` (which runs
`holdout verify` against a hidden reference + held-out inputs the agent never
sees). The agent cannot read the oracle. Every `./check` run is logged to
`check.log` — **the agents' own summaries came back useless ("Done."), so the
instrumented log is the ground truth.** That instrumentation lesson is itself a
finding: don't trust an agent's self-report; capture the loop externally.

## Runs

| # | model | task | visible examples | check runs | outcome |
|---|-------|------|------------------|-----------|---------|
| 1 | sonnet | Excel column label (1→A,26→Z,27→AA) | 1,2,3,24,25,26 | 1 | one-shot correct |
| 2 | haiku | Excel column label | 1,2,3 | 1 | one-shot correct |
| 3 | haiku | ordinal suffix (1st…11th…) | 1–5 | 1 | one-shot correct (knew the 11/12/13 rule) |
| 4 | haiku | **novel** f(n)=n, but `OVER` for n>100 | 1,2,50 | **2** | **caught + converged** |

## The one that matters — run 4 (ground truth from `check.log`)

```
attempt 1:  print(n)
  >> verified 100/200   first divergence @ g100: input "101" reference "OVER" candidate "101"   (exit 1)
attempt 2:  if n > 100: print("OVER") else: print(n)
  >> verified 200/200   reward 1.00   (exit 0)
```

The agent shipped `print(n)` — which **passes all 3 visible examples** — but is
wrong above 100. holdout caught the false-green at the held-out input 101, the
agent read the single divergence line, inferred the hidden rule, and converged in
two attempts. This is the thesis working end-to-end in a live loop: *a candidate
that passes the examples but is wrong, caught on held-out input, with the
divergence driving the fix.*

## Honest verdict — conditional GREEN

**What's proven:**
- The agent-facing loop **works and is usable**. In 4/4 runs a cheap agent wrote a
  candidate, ran `holdout`, and acted on the verdict; in run 4 it used the
  first-divergence line to discover an unseen rule and converge. The exit-code +
  divergence surface is enough for an agent to drive.
- holdout **catches real weak-oracle false-greens** from natural agent output
  (run 4), not just from synthetic bugs.

**What's NOT proven — and the honest limit:**
- On **well-known small functions** (runs 1–3), capable agents **one-shot** the
  solution, so no false-green arises and holdout only *confirms* — its marginal
  value over "run the visible examples" is low there.
- The catch in run 4 required a **novel/under-specified** spec the agent couldn't
  memorize. holdout earns its keep exactly where the gap between *what the
  examples show* and *what the spec requires* is real — i.e., under-specified,
  novel, or edge-dense code.
- That regime — large, real, edge-heavy code where agents genuinely slip — is the
  **SWE-bench-scale validation still not done**. The dogfood supports the thesis
  in the regime that matters but does not yet measure it at realistic scale.

**Decision (per the plan's rule):** better than YELLOW (a real catch + convergence
in a live loop), short of unconditional GREEN (the catch needed a novel spec, not
a realistic slip on known code). → **conditional GREEN: enough to justify the
conditional phases (#6 hashed-expected, #5 properties), with eyes open that the
decisive validation is still the SWE-bench-scale regime.**

## Reproduce

The harness is task-swappable: edit `reference.py` (hidden oracle) + `gen.py`
(hidden inputs) + the visible examples in the agent prompt, then have an agent
iterate via `./check candidate.py`. `check.log` records every attempt.
