+++
id = "stage-io"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/tests/io/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# tests/io on the shared stage and surfaces

## Goal

Draft, 3.3 and 3.5, for io's own world (`skein-io-world`): its hosts'
hand-written stages move onto `skein_io::stage`, its hosts onto the
provided deadline, and its referees onto `Referee`'s defaults, by
`notes/stage.md`.

Acceptance:

- Moved: the stages at `src/append.rs:136`, `src/usage.rs:151`,
  `tests/suite/memory.rs:169`, `tests/suite/process_memory.rs:159` and
  `:389`, unless a default of `stage.md` leaves one (named).
- Left, by `stage.md`'s "a site that measures each step" default:
  `src/output.rs:285`, `src/processes.rs:~218`, `src/world.rs:78`, which
  check each step's submissions against its `MAX_OUT` (`mark`/`within`).
  They are io's bound tests and the count's target keeps them.
- `io_resume` falls by the sites moved (5 expected).
- Referees in `src/{append,cuts,files,locks,pages,private,processes,renames,usage}.rs`
  drop the methods equal to the defaults; hosts (`Owner`, `Writer`,
  `Process`) whose `next_deadline` merges io's with the policy deadline
  state `next_mechanism_deadline` instead.
- `heavy cargo nextest run -p skein-io-world` and `heavy cargo nextest
  run -p skein-io-world --profile fuzzy` pass (counts named); clippy
  clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/design/io.md`, 8 (testing).
- `skein_io::stage`, `skein_shell::Host`, `skein_world::Referee` on your
  base: their contracts.

## Owns and may change

- `skein/tests/io/**`, `skein/scripts/old-forms.txt`.
- `tests/suite/stage.rs` is `kit-io`'s ordering test: leave it.
- The `Lifecycle` in `src/processes.rs` is not a copy (a child's run);
  leave it.

## Steps

1. The stage sites, by `stage.md` item 1 and its defaults. In
   `process_memory.rs` and `memory.rs` the stage may sit inside a
   counting test harness: keep what it counts.
2. The deadlines and referees, items 2 and 4.
3. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`; for each site left, the default and the test that pins
it.
