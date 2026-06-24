# Terminal Trove submission

Submit at: https://terminaltrove.com/submit/ (or their current form)

- **Name:** holdout
- **Repo:** https://github.com/brevity1swos/holdout
- **Language:** Rust
- **Category:** testing / developer tools / AI

## One-liner

```
A tamper-resistant, agent-facing verification oracle — a verifier you cannot game.
```

## Short description

```
holdout grades a code candidate (a refactor, an optimization, an agent's edit)
against a trusted reference on held-out or freshly-generated inputs it can't read,
edit, or memorize against. Reports the held-out score, the first divergence, and a
scalar reward over exit codes + JSON, so an LLM agent reads it back in-loop. It
reproduced a real SWE-bench Verified false-green — a patch the official metric
marks resolved — on a Docker eval. Install: cargo install holdout.
```

Attach `assets/demo.gif`.
