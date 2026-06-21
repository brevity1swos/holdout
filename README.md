# holdout

An **agent-facing differential grader** for the autonomous-coding loop. Given a
trusted reference and a candidate (a refactor, an optimization, an agent's edit),
`holdout` checks whether the candidate preserves the reference's behavior — on
**held-out** or **freshly generated** inputs the candidate never saw, against an
oracle it cannot author, and (when the seal is held out-of-band) cannot forge.
It reports a held-out score, the first divergence, a procedure check, and a
scalar reward, via exit codes + JSON so an agent can read it back in-loop.

Under the hood this is **differential + held-out + metamorphic-style testing**
(established techniques) packaged for one job: giving an LLM agent precise,
hard-to-overfit behavioral feedback during a refactor/regression loop.

![holdout demo](https://raw.githubusercontent.com/brevity1swos/holdout/main/assets/demo.gif)

> **Validated on real bugs.** On the [QuixBugs](https://github.com/jkoppel/QuixBugs)
> corpus, holdout catches **28/29 = 97%** of real one-line bugs. And on a real
> SWE-bench Verified instance (`django__django-16485`), it flags a patch that
> SWE-bench's official oracle marks **resolved** — caught via a held-out
> [UTBoost](https://github.com/CUHK-Shenzhen-SE/UTBoost) test the metric never ran.
> See [`benchmark/`](benchmark/).

## What it does well

- **Refactor / regression verification.** Capture a reference's behavior, then
  catch any candidate that changes it — including on inputs the agent never saw,
  so passing the visible examples isn't enough.
- **Surfaces weak-oracle false-greens.** The held-out score and the
  visible−heldout gap expose candidates that pass the examples but are wrong.
- **Bounded + interruptible.** A wall-clock budget kills a non-terminating
  candidate; `watch` renders a drift digest a human can act on mid-loop.

## Usage

```sh
# Refactor loop: capture reference behavior → seal → grade the candidate.
holdout record --reference './old' --inputs cases.txt --visible 3 --out oracle.json
holdout seal   --oracle oracle.json                  # prints the seal; also writes a sidecar
holdout grade  --oracle oracle.json --candidate './new' --seal "$SEAL" [--json]

# Live differential over fresh generated inputs (strongest anti-overfit).
holdout verify --reference './old' --candidate './new' --generator './gen' --n 200 [--policy p.json] [--log run.jsonl]

# Render a run-log digest with a drift flag (mid-loop human signal).
holdout watch  --log run.jsonl

# Greenfield (no reference): grade against human-authored property predicates.
holdout properties --candidate './impl' --generator './gen' --properties props.json
```

`props.json` is `{ "properties": [ {"name": "...", "command": "..."} ] }`. Each
property command receives `{"input":..,"output":..}` JSON on stdin and exits 0 if
the invariant holds, non-zero if violated (e.g. "output is sorted", "round-trips",
"len(out)==len(in)"). No reference output is needed — you state what must be true.

Exit codes throughout: `0` = passed, `1` = divergence / failure, `2` = seal
mismatch / usage / IO error. A timed-out candidate is recorded as a divergence
(`--timeout-ms`, default 5000).

## Threat model — read this before trusting the "tamper-resistant" claim

The anti-gaming guarantees are **conditional**, and the honest conditions are:

- **The seal only helps if it is held out-of-band.** Pass it via `--seal` or
  `HOLDOUT_SEAL` from a trusted orchestrator. The on-disk `.seal` sidecar is a
  convenience for local use — an agent that can write the workspace can forge it.
- **The held-out *answers* live in the oracle file by default** — but
  `record --hash-expected` stores only BLAKE3 hashes of the held-out answers, so
  reading the oracle reveals nothing (grading still works by hashing the
  candidate's output; divergences show the expected value as `<hashed>`). For a
  check that stores nothing at all, use `verify` (live reference + inputs
  generated at grade time).
- **Procedure gating is trace-string checking, not instrumentation.** `--policy`
  matches `forbidden` / `required` / `require_order` substrings against the
  candidate's stderr. It catches a corrupt success only if the candidate's trace
  records the step; it does not intercept syscalls, tool calls, or auth checks.
- **holdout grades against a *given* oracle; it does not invent one.** For
  refactor/regression you have a reference (`record`/`verify`). For greenfield,
  `holdout properties` checks human-authored invariants with no reference needed —
  but *you* still author the properties; that spec authoring is not automated away.

In short: against a non-adversarial agent that is merely *sometimes wrong*,
holdout is a robust, ungameable-by-construction grader. Against a determined
adversary with workspace access, its guarantees reduce to whatever you keep
out-of-band.

## Status

Working core, validated on real bugs and dogfooded once in a live agent loop
(`dogfood/RESULTS.md` — conditional positive: the loop works and catches real
false-greens, but adds little on tasks an agent already one-shots). Subcommands
`seal`/`record`/`grade`/`verify`/`watch`/`properties` all implemented; wall-clock
budget, procedure gating, hashed-held-out (`record --hash-expected`), and
greenfield property grading shipped. The QuixBugs mechanism gate (`benchmark/`)
catches **28/29 = 97%** of real one-line bugs; the weak-oracle false-green rate is
a curve in how many examples the agent sees (62%→10% at VISIBLE 1→5). Not on
crates.io; Unix-only candidate execution. The **SWE-bench repo-patch regime is the
decisive unmeasured validation**; live `watch --follow` and a `grade` run-log are
not yet implemented. See `benchmark/RESULTS.md` and `dogfood/RESULTS.md`.

## License

MIT — see `LICENSE`.
