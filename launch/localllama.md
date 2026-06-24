# r/LocalLLaMA

Frame: a practical tool for "did the agent's fix actually work?", not self-promo.

## Title

```
I built a CLI that catches when a coding agent's "fix" is secretly wrong — it even caught a real SWE-bench false-green
```

## Body

```
If you run coding agents, you've seen this: the agent says "fixed ✅", the tests
it can see pass, and the change is still wrong. Agents also tend to game the
checks you give them — special-casing the examples, editing the test, hard-coding
outputs.

holdout is a small terminal tool that grades a candidate change against a trusted
reference on inputs the agent never saw — held out, or freshly generated each run.
The oracle is held out-of-band, so the agent can't read, edit, or memorize it. You
get pass/fail + the first divergence + a reward, as exit codes / JSON, so it drops
into an agent loop.

To check it does something real, I ran it against SWE-bench Verified: it flagged a
submitted patch (django__django-16485) that's officially "resolved" but fails a
held-out test — the kind of false-green the eval's own metric misses. One
confirmed case (not a rate; the full sweep needs more compute).

cargo install holdout — https://github.com/brevity1swos/holdout

Works as a refactor/regression gate today; greenfield via human-authored
properties. Curious whether people would want this as an MCP tool so the agent
calls it directly.
```

Engage replies practically; the MCP question invites useful feedback.
