# Show HN

## Title

```
Show HN: holdout – a verifier you can't game (it caught a real SWE-bench false-green)
```

## URL

https://github.com/brevity1swos/holdout

## First comment (post immediately after submitting)

```
Generating code with an agent is cheap now; knowing the change is actually
correct is the bottleneck — and agents will reward-hack whatever tests you give
them (special-case the visible cases, edit the test, hard-code the output).

holdout is a small Rust CLI that grades a candidate against a trusted reference
on inputs the candidate never saw — held-out, or freshly generated each run. The
oracle is held out-of-band, so a workspace-writable agent can't read, edit, or
memorize against it. You get pass/fail, the first divergence (input/expected/got),
and a scalar reward, over exit codes + JSON, so an agent reads it back in-loop.

The part I didn't expect to work: on SWE-bench Verified, holdout flagged a real
submitted patch (django__django-16485) that the official metric marks *resolved* —
it passes all the original tests, but fails a held-out UTBoost test the metric
never runs. That's the PatchDiff/UTBoost "false-green" finding, reproduced as a
reusable CLI on a real Docker eval.

Honest about limits: that's ONE confirmed false-green, not a measured rate (the
full sweep needs x86; I'm on an arm64 Mac). The "ungameable" property is
conditional — it only holds if you keep the oracle out-of-band. Procedure checks
are trace-string matching, not syscall instrumentation. It's Unix-only.

cargo install holdout

It's the verification layer of a 4-tool set (stepwise: ccr → agx → sift →
holdout). Happy to go into the held-out-vs-fresh-input design or the SWE-bench
mapping.
```

## Tips
- Post 8–11am ET on a weekday for the best shot.
- Don't editorialize the title — the SWE-bench claim does the work.
- Engage every substantive comment fast and honestly; concede limits openly.
