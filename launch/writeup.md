# I caught a real SWE-bench false-green with a 1,500-line Rust CLI

*Draft for dev.to / blog / the stepwise site. Publish with a stable URL; every
other channel links here.*

---

Generating code with an AI agent is cheap now. Knowing the change is **correct** is
the expensive part — and it's getting worse, because agents reward-hack the checks
you give them: they special-case the visible examples, edit the test, hard-code the
expected output, or quietly break something the test doesn't cover.

So I built **holdout**: a small terminal tool that grades a candidate change in a
way the candidate can't game.

```sh
cargo install holdout
```

## The idea: grade on inputs the candidate never saw

Give holdout a *trusted reference* (the old version, a reference impl, a recorded
behavior) and a *candidate* (the agent's edit). holdout runs both on inputs the
candidate never saw — either **held-out** cases sealed away, or inputs **freshly
generated each run** — and compares.

```sh
# Capture a reference's behavior, then check the agent's version preserves it,
# on inputs generated fresh at grade time (so they can't be memorized):
holdout verify --reference ./old --candidate ./new --generator ./gen --n 200
```

You get back: a pass rate, the **first divergence** (the exact input, expected, and
actual), and a scalar reward — over exit codes and JSON, so an agent reads it back
inside its own loop.

The anti-gaming property is structural, not vibes:

- The oracle is held **out-of-band** (`--seal` / `HOLDOUT_SEAL`) — a workspace-
  writable agent can't forge it.
- `record --hash-expected` stores only BLAKE3 hashes of the held-out answers, so
  reading the oracle file reveals nothing.
- `verify` stores nothing at all — the reference is live and the inputs are
  generated at grade time, so there's no fixed answer key to tune against.

I'm deliberate about the limits: this is robust against an agent that's merely
*sometimes wrong*, not a determined adversary with full filesystem access. Against
that, the guarantees reduce to whatever you keep out-of-band. (Full threat model is
in the README.)

## Does it catch anything *real*?

A toy demo proves nothing. SWE-bench Verified is the benchmark everyone uses to
score coding agents — so I pointed holdout at it.

The setup: SWE-bench scores a patch "resolved" if it passes a fixed set of tests
(`FAIL_TO_PASS` + `PASS_TO_PASS`). The PatchDiff and UTBoost papers showed those
test sets are *incomplete* — 7.8–29.6% of "resolved" patches are still
behaviorally wrong, exposed only by additional tests. UTBoost published those
augmented tests.

So I took a real submitted patch from a strong agent (SWE-agent + Claude 3.5
Sonnet) that SWE-bench officially marks **resolved**, and ran it against the
UTBoost-augmented tests in a real Docker eval, with holdout grading the results:
the **original** tests as the "visible" oracle, the **augmented** tests as
held-out.

Result, on `django__django-16485` (Django's `floatformat` template filter):

```
holdout grade  →  visible 100%   heldout 0%   gap 1.00   exit 1
first divergence: test_additional_floatformat_zero_cases  expected "PASS" got "FAIL"
```

The patch passes **all 10 original SWE-bench tests** — the official metric calls it
solved — but fails a held-out test about zero-case handling. holdout flags it. A
patch that is 100% green on SWE-bench's own oracle, caught via a test the metric
never ran.

## The part where I almost fooled myself

My first detector ("resolved on the original set XOR resolved on UTBoost") flagged a
`pylint` instance too. But on inspection, 136 of its 171 *original* tests were
failing — that wasn't a behavioral catch, it was my arm64 Mac failing to reproduce
the upstream environment (pylint's output-format tests are platform-sensitive), and
UTBoost hadn't even augmented that instance.

So I tightened the rule: **a clean false-green requires every original test to PASS
(the patch genuinely reproduces "resolved") AND an augmented test to FAIL.** Under
that rule, the pylint case is correctly rejected, several genuinely-correct patches
are correctly *not* flagged, and `django-16485` stands. That distinction — between a
real catch and an environment artifact — is the whole game in eval work, and it's
worth being loud about.

Honest scope: that's **one confirmed false-green, not a measured rate.** The full
sweep across all 26 augmented instances and many models needs x86 hardware to run
the heavy scientific repos reliably; I'm on an arm64 laptop. The existence proof on
real Docker is what's done.

## Where it fits

holdout is the verification layer of **stepwise** — terminal-native tooling for the
AI-agent loop: `ccr → agx → sift → holdout`. The first three make what the agent did
*visible*; holdout makes correctness *checkable*.

It works today as a refactor/regression gate (`record` → `grade`), a live
differential checker (`verify`), and a greenfield grader against human-authored
properties (`properties`). Rust, five dependencies, ~1,500 LOC, Unix-only.

```sh
cargo install holdout
```

→ https://github.com/brevity1swos/holdout

Feedback on the held-out-vs-fresh-input design, or whether you'd want this as an MCP
tool so your agent calls it directly, very welcome.
