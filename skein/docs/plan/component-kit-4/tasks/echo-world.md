+++
id = "echo-world"
part = "world-surfaces"
kind = "migration"
needs = ["echo", "world-kit", "counts"]
files = ["skein/tests/echo/**"]
effort = "medium"
minutes = 30
+++

## Goal

tests/echo's two `Process` dispatchers implement `skein_shell::Forward`
instead of forwarding `Host` method by method, and its referees drop the
overrides the trait's defaults now give
(`skein/docs/design/draft/component-kit.md, 3.5`).

Acceptance:

- `tests/echo/src/proc.rs` and `tests/echo/tests/end_to_end.rs`: `impl
  Forward for Process` with `host` and `host_mut`; no `impl Host for
  Process` left in tests/echo. The `forwarding_host` count falls by 2.
- Trivial `act`, `next_deadline` and `overdue` overrides in tests/echo's
  referees are gone.
- `skein-echo-world`'s tests pass.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and `notes/migrations.md`
  (the world surfaces).
- `skein/crates/skein-shell/src/drive.rs` (`Forward`, `Host`);
  `skein/testing/skein-world/src/referee.rs` (the defaults).
- tests/world's dispatchers, already moved by task world-kit, as the model
  (`skein/tests/world/tests/suite/end_to_end.rs`).

## Owns and may change

- Owns `skein/tests/echo/`.

## Steps

1. Move both dispatchers onto `Forward` (proc.rs's matches `self`: give it
   the two accessors).
2. Drop the trivial referee overrides.
3. `cargo fmt`; `heavy cargo clippy -p skein-echo-world --all-targets --
   -D warnings`; `heavy cargo nextest run -p skein-echo-world`.
4. `old_forms.py --write`; commit and push.

## Report

As common.md says, with the lines each dispatcher had before and after.
