+++
id = "world-oauth-accounts"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "world-kit", "counts"]
files = ["skein/tests/oauth-accounts/**"]
effort = "high"
minutes = 60
+++

## Goal

The oauth-accounts world's sign-in and world hosts run their io stage through `skein_io::stage`, so every one checks the
kernel's submission room before taking work
(`skein/docs/design/draft/component-kit.md, 3.3`; testing-strategy.md, 6);
its `Process` dispatchers implement `skein_shell::Forward` instead of
forwarding `Host` method by method, and its referees and hosts use the
world surfaces' defaults (3.5).

Acceptance:

- Each io stage site below calls `skein_io::stage::up`/`down`, or keeps
  `io::resume` for a reason `notes/migrations.md` allows, named in the
  report. The `io_resume` count falls by the sites moved.
- Each dispatcher below is `impl Forward for Process`; the
  `forwarding_host` count falls by 2.
- `Host` impls whose `next_deadline` hand-merges a mechanism deadline with
  their policy deadline implement `next_mechanism_deadline` instead;
  trivial `Referee` overrides go.
- The package's tests pass.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and
  `notes/migrations.md`.
- `skein/crates/skein-io/src/stage.rs`; `skein/crates/skein-shell/src/drive.rs`
  (`Host`, `Forward`); `skein/testing/skein-world/src/referee.rs`; the
  model dispatcher `skein/tests/world/tests/suite/end_to_end.rs` (moved by
  task world-kit).
- The io stage sites (`rg -n 'io::resume'`):
  - `skein/tests/oauth-accounts/src/sign_in.rs`, `src/sign_in/browser.rs`, `src/world.rs`.
- The dispatchers:
  - `skein/tests/oauth-accounts/src/sign_in.rs` (`enum Process`, line 539) and `src/world.rs` (line 808).

## Owns and may change

- Owns `skein/tests/oauth-accounts/`.
- This world has two owner adapters, two `Process` enums and two restart wrappers (the review, theme 4): do not merge them here (a later run's); move each onto the kit as it is.
- `src/private_cuts.rs` and `src/sign_in/cuts.rs` implement `Host` for a `Process`: move them onto `Forward` only if they forward every method to one held host; otherwise leave them.

## Steps

1. Move each io stage site, the narrowest test after each.
2. Move each dispatcher onto `Forward`. A dispatcher that matched `self`
   in every method gets the two accessors. If a test's outcome changes
   because `exit` or `state_digest` now forwards, report it and keep
   the forwarding.
3. The deadlines and referee defaults (notes/migrations.md).
4. `cargo fmt`; `heavy cargo clippy -p skein-oauth-accounts-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-oauth-accounts-world`.
5. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each site moved or kept, and each dispatcher's
lines before and after.
