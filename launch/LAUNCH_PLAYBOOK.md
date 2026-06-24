# holdout — launch playbook

Outreach materials for `holdout` v0.1.0. The hook is a **result, not a tool**:
holdout caught a real SWE-bench Verified patch the official metric marks
*resolved*. Lead with that everywhere.

## The hook (use verbatim)

> **holdout — a verifier you cannot game.** It grades an agent's change on
> held-out / freshly-generated inputs it can't read, edit, or memorize against —
> and it caught a real SWE-bench Verified patch the official oracle marks resolved.

`cargo install holdout` · https://github.com/brevity1swos/holdout

## Honesty guardrails (do not break these)

- **"One confirmed false-green, not a measured rate."** Never imply a percentage on
  real SWE-bench. The QuixBugs 97% is a separate, smaller-scale corpus.
- The **"ungameable" claim is conditional** — the oracle must be held out-of-band.
  Keep the caveat; the README's threat-model section is the source of truth.
- No personal identity in any post — publish from the project account.

## Lead artifact

`launch/writeup.md` — the narrative "I caught a SWE-bench false-green" post.
Publish it somewhere with a stable URL (dev.to / blog / the stepwise site) first;
every channel below links to it. The raw run log is `benchmark/swebench/FALSE_GREEN.md`.

## Channels

Audience first: holdout's crowd is **AI-agent / eval people**, not just Rust-CLI.

| Channel | Draft | Notes |
|---|---|---|
| Twitter/X | `launch/twitter.md` | thread; lead with the SWE-bench result |
| Show HN | `launch/show_hn.md` | result-forward title; one shot |
| Lobste.rs | `launch/lobsters.md` | `rust` + `ai`/`ml` tags; link the writeup |
| r/LocalLLaMA | `launch/localllama.md` | practical framing, not self-promo |
| Terminal Trove | `launch/terminal_trove.md` | low-effort submission |

**Do NOT post to r/rust or r/commandline** — both restrict AI-assisted project
posts; not worth the removal/ban risk.

`awesome-rust` and similar lists have an inclusion bar (≈50★ or ≈2k downloads);
holdout doesn't clear it yet — revisit later, not at launch. (No `awesome-ratatui`
— holdout has no TUI.)

## Sequence

1. Publish `writeup.md` (stable URL).
2. Twitter/X thread → link writeup + repo.
3. Show HN (same day or +1).
4. Lobste.rs + r/LocalLLaMA (spaced out, not simultaneous).
5. Terminal Trove follow-up.
6. Review after ~2 weeks; if flat, move to maintenance — no repeat-posting hunts.

## Positioning vs the stack

holdout is the 4th tool in [stepwise](https://github.com/brevity1swos/stepwise):
`ccr → agx → sift → holdout`. The first three make what the agent did *visible*;
holdout makes correctness *checkable*. Mention the stack as context, but the
SWE-bench result carries the launch on its own.
