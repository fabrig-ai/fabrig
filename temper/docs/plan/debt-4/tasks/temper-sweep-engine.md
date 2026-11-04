+++
id = "temper-sweep-engine"
part = "temper-forms"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-engine-domain/**",
]
effort = "medium"
minutes = 75
+++

# temper D003, D004: `temper-engine-domain` uses `len_u64`, `usize_of` and `env.with`

## Goal

`temper-engine-domain`, the sweep's largest crate, spells its widenings with skein-lib's helpers and hands its child steps their limits with `env.with`. Both items stay open: `temper-sweep-paid` deletes them.

Acceptance:

- With `OWN="temper/crates/temper-engine-domain"`, the three searches of `temper/docs/plan/debt-4/notes/temper-forms.md` read nothing.
- `temper/crates/temper-engine-domain/src/store.rs`'s two `u64::try_from(uses.as_deref().unwrap_or_default().len())` are `len_u64` too.
- `heavy cargo nextest run -p temper-engine-domain` passes, every count above zero; no assertion of an existing test changes.
- `heavy cargo clippy -p temper-engine-domain --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/temper-forms.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D003-widening-try-from.md` and `temper/docs/development/debt/D004-env-with.md`, whole.
- `skein/docs/design/lib.md`, 6 (the widenings) and 9 (`env.with`); `skein-lib/src/length.rs`, `skein-lib/src/env.rs`.

At base, `$OWN`'s sites by the note's searches:

| Package | `u64::try_from(..len())` | `"u32 fits usize"` | `Env` rewraps | Files |
|---|---|---|---|---|
| `temper-engine-domain` | 108 | 34 | 35 | 27 |

## Owns and may change

- Owns: `$OWN`'s Rust sources, for the forms the note names and nothing else.
- Overlap: `src/limits.rs` holds 16 sites and `fabrig-scale` changes three of its lines; sweep it all the same, a one-line merge is fab's.
- Pre-approved: the note's `Pre-approved fallout`, each with its default.

## Steps

1. Sweep `$OWN` by the note's table, package by package, and read what the first two rows did not take.
2. Run the three searches; then what the note's `What runs` names.
3. Commit (`D003, D004: <packages> use skein-lib's helpers`, saying why), and push.

## Report

The counts before and after; each site the note's fallout touched; each `#[expect]` you added;
any site you left and why.
