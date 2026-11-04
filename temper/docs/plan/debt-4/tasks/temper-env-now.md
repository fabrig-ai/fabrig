+++
id = "temper-env-now"
part = "temper-forms"
kind = "migration"
needs = ["skein-now-with"]
files = [
  "temper/crates/temper-forge-setup-shell/src/fetch.rs",
  "temper/crates/temper-service/src/tests/stop/pre_settings.rs",
  "temper/crates/temper/tests/support/forge_parent.rs",
  "temper/tests/real-loop/src/github.rs",
]
effort = "medium"
minutes = 15
+++

# temper D004: the shells' `Env` comes from `Now::with`

## Goal

Thirteen of D004's rewraps read their clocks from a `skein_shell::Now`, which is no `Env`, so
`env.with` does not reach them. They call `Now::with`, which `skein-now-with` adds. The items stay
open: `temper-sweep-paid` deletes them.

Acceptance:

- With `OWN` the four files below, the third search of
  `temper/docs/plan/debt-4/notes/temper-forms.md` reads nothing.
- `heavy cargo nextest run -p temper-forge-setup-shell` passes, with its count above zero;
  `heavy cargo clippy -p temper-forge-setup-shell -p temper-service -p temper -p temper-real-loop-world
  --all-targets -- -D warnings` passes.
- `heavy cargo nextest run -p temper-service --lib -E 'test(pre_settings)'` passes, with its count
  above zero. `forge_parent.rs` is a support file of `temper`'s tests: `heavy cargo check -p temper
  --tests` stands for its run, and `heavy cargo check -p temper-real-loop-world --tests` for
  `github.rs`'s.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/temper-forms.md`,
  `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D004-env-with.md`, whole; `skein/docs/design/shell.md`, 5.
- At base: `fetch.rs` 7, `pre_settings.rs` 3, `forge_parent.rs` 2, `github.rs` 1.

## Owns and may change

- Owns: the four files in `files`, for this form only.
- Pre-approved: if `Now`'s value is used after the call, nothing changes, since `Now` is `Copy`.

## Steps

1. `Env { now: now.now, wall: now.wall, limits: l }` becomes `now.with(l)`, and `&Env { .. }`
   stays borrowed, `&now.with(l)`. In `github.rs`, the value is called `clock`:
   `clock.with(config)`. A `use skein_lib::Env` left unused goes.
2. Run the search and the checks above; the citations check. Commit and push.

## Report

The count before and after, and what ran.
