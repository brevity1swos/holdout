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

### The refactor loop

```sh
# 1. Capture a trusted reference's behavior into a sealed oracle.
holdout record --reference './old_version' --inputs cases.txt --visible 3 --out oracle.json

# 2. The agent refactors/optimizes. Then verify it preserved behavior —
#    on held-out inputs it never saw, against an oracle it cannot edit.
holdout grade --oracle oracle.json --candidate './new_version'
```

`record` runs `--reference` once per input line, captures stdout, and writes a
sealed `GoldenTrace` oracle (`--visible` cases are shown to the agent; the rest
are held out). Because the expected outputs come from captured real behavior,
there is nothing to overfit and no examiner to collude with — memorization is
defeated by the held-out inputs the agent never sees.

### Live differential verification (fresh inputs)

```sh
holdout verify \
  --reference './old_version' \
  --candidate './new_version' \
  --generator './gen_inputs' \
  --n 200
```

`verify` runs `--generator` to produce fresh inputs, then runs `--reference`
and `--candidate` on each and compares. There is no stored oracle to seal or
edit: the reference is the live oracle, and the inputs are generated at verify
time, so a candidate cannot be tuned to pass a known set. Exit `0` = every
generated input matched, `1` = a divergence (with the first one reported),
`2` = the generator produced no inputs or a run failed. The generator owns all
input variation — `holdout` contains no randomness.

## Status

Phase-1 MVP. Oracle kind: `HeldoutCases` only. Perturbation is reorder+rename
(does not mutate inputs). Unix-only candidate execution. Procedure-aware gating,
the `watch` digest, reference-impl/metamorphic perturbation, and SWE-bench
validation are not yet implemented.

## License

MIT — see `LICENSE`.
