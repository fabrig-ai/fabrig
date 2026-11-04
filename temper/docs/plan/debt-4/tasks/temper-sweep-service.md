+++
id = "temper-sweep-service"
part = "temper-forms"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-service/**",
  "temper/crates/temper/**",
  "temper/crates/temper-engine-smith/**",
  "temper/crates/temper-protocol-store/**",
]
effort = "medium"
minutes = 55
+++

# temper D003, D004: `temper-service`, `temper` and two more use `len_u64`, `usize_of` and `env.with`

## Goal

`temper-service`, `temper`, `temper-engine-smith` and `temper-protocol-store` spell their widenings with skein-lib's helpers and hand their child steps their limits with `env.with`. The `Env` rewraps on a `skein_shell::Now` are `temper-env-now`'s and stay. Both items stay open: `temper-sweep-paid` deletes them.

Acceptance:

- With `OWN="temper/crates/temper-service temper/crates/temper temper/crates/temper-engine-smith temper/crates/temper-protocol-store"`, the three searches of `temper/docs/plan/debt-4/notes/temper-forms.md` read nothing, but for the third's two files below.
- The third search lists exactly `temper/crates/temper-service/src/tests/stop/pre_settings.rs` (3) and `temper/crates/temper/tests/support/forge_parent.rs` (2), both on a `Now`, which `temper-env-now` owns; those two files stay untouched here. `temper-service/src/tests/setup.rs`'s `u64::try_from(lifecycle::stopped_text(*why).len())` is `len_u64` too.
- `heavy cargo nextest run -p temper-service -p temper -p temper-engine-smith -p temper-protocol-store` passes, every count above zero; no assertion of an existing test changes.
- `heavy cargo clippy -p temper-service -p temper -p temper-engine-smith -p temper-protocol-store --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/temper-forms.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D003-widening-try-from.md` and `temper/docs/development/debt/D004-env-with.md`, whole.
- `skein/docs/design/lib.md`, 6 (the widenings) and 9 (`env.with`); `skein-lib/src/length.rs`, `skein-lib/src/env.rs`.

At base, `$OWN`'s sites by the note's searches:

| Package | `u64::try_from(..len())` | `"u32 fits usize"` | `Env` rewraps | Files |
|---|---|---|---|---|
| `temper-service` | 23 | 2 | 12 | 18 |
| `temper` | 4 | 2 | 2 | 6 |
| `temper-engine-smith` | 3 | 1 | 0 | 3 |
| `temper-protocol-store` | 3 | 1 | 0 | 3 |

## Owns and may change

- Owns: `$OWN`'s Rust sources, for the forms the note names and nothing else.
- Not these two: `temper-service/src/tests/stop/pre_settings.rs`, `temper/tests/support/forge_parent.rs`.
- Overlap: `temper-protocol-store/src/tests/records.rs` has one site and `fabrig-scale` adds to that file; `temper-service`'s `stock.rs` and `startup*`, which `fabrig-scale` is changing, hold no site.
- Pre-approved: the note's `Pre-approved fallout`, each with its default.

## Steps

1. Sweep `$OWN` by the note's table, package by package, and read what the first two rows did not take.
2. Run the three searches; then what the note's `What runs` names.
3. Commit (`D003, D004: <packages> use skein-lib's helpers`, saying why), and push.

## Report

The counts before and after; each site the note's fallout touched; each `#[expect]` you added;
any site you left and why.
