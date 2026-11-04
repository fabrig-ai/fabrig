+++
id = "temper-sweep-forge"
part = "temper-forms"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/**",
  "temper/crates/temper-engine-domain-forge-change/**",
  "temper/crates/temper-engine-domain-forge-issues/**",
  "temper/crates/temper-engine-domain-forge-landing/**",
  "temper/crates/temper-engine-domain-forge-tiers/**",
]
effort = "medium"
minutes = 60
+++

# temper D003, D004: the forge domain crates use `len_u64`, `usize_of` and `env.with`

## Goal

The five forge domain crates spell their widenings with skein-lib's helpers and hand their child steps their limits with `env.with`. Both items stay open: `temper-sweep-paid` deletes them.

Acceptance:

- With `OWN="temper/crates/temper-engine-domain-forge temper/crates/temper-engine-domain-forge-change temper/crates/temper-engine-domain-forge-issues temper/crates/temper-engine-domain-forge-landing temper/crates/temper-engine-domain-forge-tiers"`, the three searches of `temper/docs/plan/debt-4/notes/temper-forms.md` read nothing.
- `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain-forge-change -p temper-engine-domain-forge-issues -p temper-engine-domain-forge-landing -p temper-engine-domain-forge-tiers` passes, every count above zero; no assertion of an existing test changes.
- `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain-forge-change -p temper-engine-domain-forge-issues -p temper-engine-domain-forge-landing -p temper-engine-domain-forge-tiers --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/temper-forms.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D003-widening-try-from.md` and `temper/docs/development/debt/D004-env-with.md`, whole.
- `skein/docs/design/lib.md`, 6 (the widenings) and 9 (`env.with`); `skein-lib/src/length.rs`, `skein-lib/src/env.rs`.

At base, `$OWN`'s sites by the note's searches:

| Package | `u64::try_from(..len())` | `"u32 fits usize"` | `Env` rewraps | Files |
|---|---|---|---|---|
| `temper-engine-domain-forge` | 55 | 34 | 1 | 19 |
| `temper-engine-domain-forge-issues` | 0 | 7 | 0 | 1 |
| `temper-engine-domain-forge-change` | 0 | 5 | 0 | 1 |
| `temper-engine-domain-forge-landing` | 1 | 2 | 0 | 2 |
| `temper-engine-domain-forge-tiers` | 0 | 2 | 0 | 2 |

## Owns and may change

- Owns: `$OWN`'s Rust sources, for the forms the note names and nothing else.
- Pre-approved: the note's `Pre-approved fallout`, each with its default.

## Steps

1. Sweep `$OWN` by the note's table, package by package, and read what the first two rows did not take.
2. Run the three searches; then what the note's `What runs` names.
3. Commit (`D003, D004: <packages> use skein-lib's helpers`, saying why), and push.

## Report

The counts before and after; each site the note's fallout touched; each `#[expect]` you added;
any site you left and why.
