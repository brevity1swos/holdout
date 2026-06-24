# Twitter/X

## Single tweet (~270 chars)

```
AI agents reward-hack your tests. holdout is a verifier they can't game: it grades a change on held-out/fresh inputs the agent never saw.

It even caught a real SWE-bench patch the official metric marks "resolved."

cargo install holdout
github.com/brevity1swos/holdout
```

## Thread (preferred — lead with the result)

**1/**
```
Agentic coding made *generating* code cheap. Verifying it's correct is the
bottleneck — and agents reward-hack the tests you hand them.

holdout is a terminal verifier you cannot game.

cargo install holdout 🧵
```

**2/**
```
Give it a trusted reference; it grades the agent's version on inputs the agent
never saw — held-out or freshly generated. The oracle is held out-of-band, so it
can't be read, edited, or memorized against.

Pass/fail, the first divergence, a reward — exit codes + JSON, in-loop.
```

**3/**
```
Does it catch anything real? On SWE-bench Verified it flagged
django__django-16485 — a patch the official oracle marks "resolved" — via a
held-out test the metric never ran. On a real Docker eval.

(One confirmed false-green, not a measured rate. But it's real.)
```

**4/**
```
It's the 4th tool in stepwise — terminal-native tooling for the AI-agent loop:

ccr → agx → sift → holdout

The first three make what the agent did *visible*; holdout makes correctness
*checkable*.

github.com/brevity1swos/holdout
```

Attach `assets/demo.gif` to tweet 1 or 2 for the visual.
