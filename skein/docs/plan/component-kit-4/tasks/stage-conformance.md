+++
id = "stage-conformance"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "world-kit", "counts"]
files = ["skein/tests/conformance/real/**", "skein/crates/skein-sim/src/tests/given_listener.rs"]
effort = "high"
minutes = 50
+++

## Goal

The real-kernel conformance tests and the simulator's given-listener test, which drive io by hand, run their io stage through `skein_io::stage`, so every one of them
checks the kernel's submission room before taking work
(`skein/docs/design/draft/component-kit.md, 3.3`; testing-strategy.md, 6),
and their worlds use the world surfaces where they forward or merge by hand
(3.5).

Acceptance:

- Each site below calls `skein_io::stage::up`/`down` instead of its own
  loops, or keeps `io::resume` for a reason `notes/migrations.md`
  allows, named in the report. The `io_resume` count falls by the sites
  moved.
- In the packages you own: `Host` impls whose `next_deadline` hand-merges
  a mechanism deadline with their policy deadline implement
  `next_mechanism_deadline` instead; trivial `Referee` overrides go.
- The tests of every package you own pass, and those of their direct
  dependents within skein check (`heavy cargo check --tests -p ...`).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and
  `notes/migrations.md`.
- `skein/crates/skein-io/src/stage.rs` (the contract);
  `skein/crates/skein-shell/src/drive.rs` (`Host`, `Forward`);
  `skein/testing/skein-world/src/referee.rs`.
- The sites (`rg -n 'io::resume'` in each path):
  - `skein/tests/conformance/real/tests/suite/adopted_listener.rs` (two), `tests/suite/given_listener.rs`, `tests/support/deadline.rs`, `tests/support/given_listener.rs`, `src/bin/signal_sender_fixture.rs`;
  - `skein/crates/skein-sim/src/tests/given_listener.rs` (a test in the simulator crate).

## Owns and may change

- Owns `tests/conformance/real/` and, in skein-sim, only `src/tests/given_listener.rs`.
- These tests hold the simulator to the real kernel: a move that changes what a scenario observes is not allowed; keep that site and report it.

## Steps

1. Move each site (notes/migrations.md, the io stage), the narrowest test
   of its package after each.
2. The world surfaces in the packages you own (notes/migrations.md).
   The conformance tests run against the real kernel on the builder:
   run them as the default profile does, and name any that the profile
   skips.
3. `cargo fmt`; `heavy cargo clippy -p skein-conformance-real -p skein-sim --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-conformance-real -p skein-sim`; the dependents' checks
   (`cargo tree -i <package> --workspace -e normal,dev --depth 1` finds
   them).
4. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each site: moved, or kept and why.

## Answers

2026-10-31. Which Git author identity should the commit use? The one every
implementer commit on this run's trunk already carries: `Codex
<noreply@openai.com>`. Set it for the commit only, never in a global
config: `git -c user.name=Codex -c user.email=noreply@openai.com commit`
(or the same two keys with `git config --local` in your own slot). Commit
the staged change as it stands, then push the task branch
`component-kit-4/stage-conformance`: it sits at the unchanged base, so the
push is a fast-forward. Chain the push after the commit with `&&`, so a
failed commit pushes nothing.
