+++
id = "world-terminal"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "world-kit", "counts"]
files = ["skein/tests/world/src/bin/terminal_fixture.rs"]
effort = "medium"
minutes = 20
+++

## Goal

The world harness's terminal fixture runs its io stage through
`skein_io::stage`, checking the kernel's submission room before taking
work (`skein/docs/design/draft/component-kit.md, 3.3`; testing-strategy.md,
6).

Acceptance:

- `tests/world/src/bin/terminal_fixture.rs` calls `skein_io::stage`
  instead of `io::resume` and its own loops; the `io_resume` count falls
  by 1.
- `skein-world-tests` passes its tests (the fixture is run by its
  terminal tests: find them with `rg world_terminal_fixture skein/tests/world`).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and `notes/migrations.md`
  (the io stage).
- `skein/crates/skein-io/src/stage.rs`.

## Owns and may change

- Owns `skein/tests/world/src/bin/terminal_fixture.rs` only.

## Steps

1. Move the site.
2. `cargo fmt`; `heavy cargo clippy -p skein-world-tests --all-targets --
   -D warnings`; `heavy cargo nextest run -p skein-world-tests`.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says.
