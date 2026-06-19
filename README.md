# holdout

A tamper-resistant, agent-facing verification oracle: grade a candidate command
against a **sealed, held-out** suite of input→expected cases it cannot read or
edit. The held-out score plus the validation−heldout gap expose memorization and
reward hacking that a naive "do the visible tests pass?" check misses.

## Usage

```sh
holdout seal  --oracle oracle.json                 # writes oracle.json.seal
holdout grade --oracle oracle.json --candidate 'CMD' [--json] [--gap-tolerance F] [--no-perturb]
```

`CMD` is run once per case: the case `input` is written to its stdin and its
trimmed stdout is compared to `expected`.

Exit codes: `0` = held-out passes and gap within tolerance, `1` = fails, `2` =
seal mismatch / usage / IO error.

## Status

Phase-1 MVP. Oracle kind: `HeldoutCases` only. Perturbation is reorder+rename
(does not mutate inputs). Unix-only candidate execution. Procedure-aware gating,
the `watch` digest, reference-impl/metamorphic perturbation, and SWE-bench
validation are not yet implemented.

## License

MIT — see `LICENSE`.
