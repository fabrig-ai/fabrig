# resume-fits-room: rules every task keeps

Every task of this plan reads this note first.

## What the plan is for

A run resumed after a loss carries every call committed since its last
turn, each with its answer (`jig/docs/design/domain/engine.md`, 7.2). A
producer whose program works an hour between two turns commits many;
on both stock profiles 32 committed calls at the largest answer pass
the link's room, io's `output` (1 MiB), alone. Its start can then never
cross the link, so it can never resume
(`temper/docs/plan/resume-fits-room/inputs/request.md`). The evidence is
outside the repository: the request's words are all there is.

The fix, as the design has it: before an assignment is measured against
the engine's room, its committed calls are fitted. Every call stays, by
its name and tool; from the newest back, host answers stay whole while
the frame fits, and the rest are cut to fixed words saying the call ran
and how many bytes its answer had; a delivery's answer is never cut.
Limits under which even the smallest resumed start cannot fit the room
are not made, and temper names the bound at start.

## This plan's base

The plan builds on `worker-link-quiet` (`temper/docs/plan/worker-link-quiet/`),
whose tasks give the engine's link its room (`Limits.room`), measure a
start against it and refuse one past it (`engine-room`), cap the joined
world's pipe as io does, and write temper's stock link story
(`temper-room`'s `temper/crates/temper-service/src/tests/link_room.rs`).
This trunk holds that plan's design but its code only once it has landed
on main and this trunk caught up. A task whose spec names a premise on
that code checks it first, as its spec says.

## Reading

- fabrig's `AGENTS.md` and the tree's `AGENTS.md`.
- This plan's design, never a whole document:
  `jig/docs/design/protocol/channel.md`, 9 (*A resumed start fits its
  room first*, and *Every frame fits the stream below* and *A start past
  the room fails its run, never the link* before it), 11 (the sizes'
  bullet and *The room is the stream's, not the domains'*), 12 (*A
  resumed start fitted to its room*); `jig/docs/design/domain/engine.md`,
  7.2 (*Resuming*); `temper/docs/design/domain/worker.md`, 4.1
  (*Transcript*) and 4.3 (*A start the link cannot carry*);
  `temper/docs/design/protocol/configuration.md`, 7 (the channel's
  checks).
- `temper/docs/plan/resume-fits-room/notes/contracts.md`: the shapes
  that cross tasks. They bind.

## Rules

- **Failing test first.** A task that fixes a fault commits the rows
  that show it, failing on its base for the fault's reason, alone,
  before the commit that fixes it. The report quotes the failure seen.
- **A new case of one behaviour is a row** of an existing table test, or
  of the one table your spec names.
- **Tests stay cheap:** step tests and machine worlds on a simulated
  clock; no test reaches a real host or network; no sleep. A row that
  needs a frame past the room builds the answers it needs and no more:
  as many as the room holds whole at that size, plus two.
- **Only the focused and fuzzy suites run,** through `heavy`, one
  command at a time, narrowest first (`AGENTS.md`, 5). Never
  `tools/tiers/run`, a `<tier>-<tree>` profile, `scripts/browser.sh`, a
  walk, a render check or a throwaway driver.
- **No figure.** No limit, bound, deadline or default changes in
  `jig/crates/jig-protocol-config/src/profile/*.rs`, temper's profiles
  (`temper/crates/temper-service/src/limits.rs`), the stock settings or
  `temper/docs/examples/`. A test may give its own limits a smaller room
  for a row; that is the row's input, not a figure.
- **No stored format or protocol change:** no wire or store schema,
  golden or codec change. A cut answer is a host answer with other
  words, in the vocabulary that exists.
- **Trees:** skein names none of the others; jig never names temper. No
  skein change is planned; needing one is a `Question:`.
- **Status lines:** the design is written. A task removes only the
  **Not yet** entries its spec names, their lines alone. That departs
  from the implementer's text, which closes `docs/design/` to you: the
  entries are your task's.
- **Not yours:** any other `docs/`, drafts, `docs/help/` and this plan's
  files.
- **Old forms:** `python3 -I -B <tree>/scripts/old_forms.py` for each
  tree you touch, before the push; `--write` only to lower a count.
- **Citations:** after committing, from the repository root,
  `python3 -I -B tools/citations/citations.py --base origin/main`. It
  reads HEAD, not the working tree.
- **A run's failure your change can't reach** runs once alone on the
  pristine base before you call it outside your change.

## Packages

| Package | Path | Its tests here |
|---|---|---|
| `jig-protocol-worker-wire` | `jig/crates/jig-protocol-worker-wire` | `--lib` |
| `jig-protocol-engine-workers` | `jig/crates/jig-protocol-engine-workers` | `--lib` |
| `jig-engine-workers-world` | `jig/tests/engine-workers` | `--test suite` |
| `jig-worker-link-world` | `jig/tests/worker-link` | `--test suite` |
| `jig-test-service` | `jig/testing/jig-test-service` | `--lib` |
| `temper-service` | `temper/crates/temper-service` | `--lib` |

Check each name with `cargo metadata --no-deps --format-version 1`
before you run it; a package named otherwise is the one at that path.

## Who owns what

| Task | Owns |
|---|---|
| `fit` | `jig/crates/jig-protocol-worker-wire/src/fit.rs`, its tests `src/tests/fit.rs`, `smallest_resumed` in `src/sizes.rs` |
| `named-floor` | `resumed_start` in `temper/crates/temper-service/src/startup/checks.rs`, its rows in `src/tests/checks.rs` |
| `fit-room` | the fit's call and `valid`'s rule in `jig/crates/jig-protocol-engine-workers`, their rows in `jig/tests/engine-workers` and the joined world `jig/tests/worker-link` |
| `temper-fits` | the fitted rows of `temper/crates/temper-service/src/tests/link_room.rs` |

A file another task owns is changed only as your spec's *Owns and may
change* allows, and the report names each such edit.
