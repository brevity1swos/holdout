# Terminal Trove submission

Submit at: https://terminaltrove.com/submit/ (or their current form).
Fields below map 1:1 to the submission form. Character counts respect the form's
limits (300 / 300 / 300 / 250).

## Basic Info

- **Tool name:** `holdout`
- **URL:** `https://github.com/brevity1swos/holdout`
- **Tagline:** `A tamper-resistant verification oracle for AI coding agents.`

## Description

**Box 1** (~285/300):
> holdout grades a code change — a refactor, an optimization, an AI agent's edit —
> against a trusted reference on held-out or freshly-generated inputs it can't read,
> edit, or memorize against. It returns a held-out score, the first divergence, and
> a reward over exit codes + JSON, for in-loop use by an agent.

**Box 2 — core features** (~290/300):
> Core modes: record→grade (sealed reference-capture refactor loop), verify (live
> differential over freshly-generated inputs), properties (greenfield invariants, no
> reference needed), and watch (a drift digest you can interrupt mid-loop). Plus
> procedure-aware gating and a wall-clock budget so a stuck candidate can't hang the
> grader.

**Box 3 — other features** (~290/300):
> The oracle is held out-of-band (--seal / HOLDOUT_SEAL) so a workspace-writable
> agent can't forge it; record --hash-expected stores only BLAKE3 hashes, so reading
> the oracle reveals nothing. Validated on real bugs — 97% on the QuixBugs corpus,
> and it flagged a real SWE-bench Verified false-green a metric marks "resolved."

**Box 4 — audience** (~210/250):
> For developers and teams running AI coding agents who need hard-to-overfit feedback
> on whether a generated change is actually correct. Rust, five dependencies,
> Unix-only. It's the verification layer of the stepwise terminal toolset.

## Technical Details — Image Preview

- **PNG:** `https://raw.githubusercontent.com/brevity1swos/holdout/main/assets/demo.png`
- **GIF:** `https://raw.githubusercontent.com/brevity1swos/holdout/main/assets/demo.gif`

(Both are real terminal output: a faithful refactor passing 50/50, then a
plausible-but-wrong candidate caught at 1/50 with the first divergence localized.)

## Categories — select all that apply

There is no "Testing / Dev Tools" category, so the closest honest fits are:
- **DevOps & Infrastructure** (a verification / CI-style gate)
- **General**

(Skip OS / Databases / Networking / UI & Display / Data & Text — not a fit.)

## Install Instructions

On crates.io only (verified) — not brew/apt/etc.

| Platform | Manager | Command |
|---|---|---|
| Linux / macOS | cargo (crates.io) | `cargo install holdout` |

Auto-fill tool name: `holdout`. Rust 1.74+. Unix-only candidate execution.

## Author & Confirmation

- **Are you the author?** → **Yes** (submit under the `brevity1swos` identity; no
  personal name).

## Note on fit

Terminal Trove leans toward end-user terminal *apps*; holdout is a dev/CI
verification tool, so it's a slightly atypical (but legitimate) entry. The
SWE-bench-false-green line in Box 3 is the hook that will land with reviewers.
