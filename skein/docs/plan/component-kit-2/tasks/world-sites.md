+++
id = "world-sites"
part = "world-surfaces"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/tests/world/**", "skein/testing/skein-world/src/real.rs", "skein/testing/skein-world/src/end_to_end.rs", "skein/tests/protocol/src/world.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# The world harness's own tests on the shared stage and surfaces

## Goal

Draft, 3.3 and 3.5, for the world harness: its tests' three `Process`
dispatchers forward `Host` through `Delegate`, its referees take
`Referee`'s defaults, its hosts state their mechanism deadline, and its
terminal fixture's io stage calls `skein_io::stage`, by `notes/stage.md`.

Acceptance:

- Dispatchers: `impl Host for Process` in
  `skein/tests/world/tests/suite/end_to_end.rs` (48-79) and
  `writer_pipe.rs` (84-117), and `impl Host for Proc` in
  `real_conformance.rs` (190-224), each become `impl Delegate` with the
  two accessors (the first already has them). `host_dispatch` falls by 3.
- Stage: `skein/tests/world/src/bin/terminal_fixture.rs:81` moves, unless
  a default leaves it (named). `io_resume` falls by the sites moved.
- Referees: every root `Referee` impl in the header's files (about 20:
  `skein/tests/world/src/{hosted,lib}.rs`,
  `skein/tests/world/tests/suite/{end_to_end,hosted,interrupt,real,real_conformance,signals,stop_continue,world,writer_pipe}.rs`,
  `skein/testing/skein-world/src/real.rs` (two, in its tests),
  `skein/tests/protocol/src/world.rs`) drops the methods equal to the
  defaults.
- Hosts in these files whose `next_deadline` is a hand merge follow
  `stage.md` item 2; wrappers (`CuttingScript`, `Closing` and the like)
  stay.
- `heavy cargo nextest run -p skein-world -p skein-world-tests -p
  skein-protocol-world` passes, and `--profile fuzzy` for
  `skein-world-tests` (counts named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/design/world.md` for the words.
- `skein_shell::Delegate`, `skein_world::Referee` (from `kit-shell`,
  `skein/testing/skein-world/src/referee.rs`), `skein_io::stage` on your
  base.

## Owns and may change

- The files in the header. In skein-world, only `real.rs` and
  `end_to_end.rs` (`impl Host for Binary`, if
  item 2 applies); `referee.rs` and `lib.rs` are `kit-shell`'s, merged on
  your base.
- `skein/testing/skein-world/src/domain/referee.rs`'s `Referee` is a
  struct of the domain harness, not the root trait: not yours.
- The pop loop in `real_conformance.rs` uses what it pops: leave it.

## Steps

1. Dispatchers, item 3; the stage, item 1; referees, item 4; hosts,
   item 2.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`, with each dispatcher's lines before and after and the
number of referee methods removed.
