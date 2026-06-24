# Lobste.rs

Tags: `rust`, `ai` (or `ml`), `testing`
Link to: the writeup (stable URL), not the bare repo.

## Title

```
holdout: a verifier you can't game — and it caught a real SWE-bench false-green
```

## Authored-post note (if posting as "authored by you")

```
holdout grades an agent's code change against a trusted reference on inputs the
candidate never saw (held-out, or freshly generated), with the oracle held
out-of-band so it can't be read, edited, or memorized against. Exit codes + JSON
for in-loop use.

The writeup walks through reproducing the PatchDiff/UTBoost "false-green" finding
as a CLI: a real SWE-bench Verified patch (django__django-16485) that's officially
*resolved* but fails a held-out test, caught on a Docker eval. One confirmed case,
not a rate — limits are stated. Rust, 5 deps, ~1.5k LOC. Feedback on the
held-out-vs-fresh-input design welcome.
```

Lobste.rs dislikes pure self-promo; submit the **writeup** and discuss the design,
not "check out my tool."
