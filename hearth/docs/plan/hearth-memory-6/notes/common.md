# hearth-memory-6: rules every task names

The request is `hearth/docs/plan/hearth-memory-6/inputs/request.md`. The design is `hearth/docs/design/domain/render.md`, 8.4 and 8.5, which this cut already wrote. The list of checks is `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps", until `close` deletes it. The shapes that cross tasks are in `hearth/docs/plan/hearth-memory-6/notes/contracts.md`.

## What doesn't move

- **What hearth does for a user doesn't change:** same specs, same digests, same records, same refusals at the edges.
- **skein and jig don't change.** Something hearth seems to need from skein is a `Question:`, never an edit. Other sessions share them.
- **A branch stays in hearth,** but for `tools/` it only runs.
- **D006, D009 and D013 stay** as they are: they are another plan's.
- **The kernel's edge stays.** `completions` and `submissions` keep their capacity, and the room check on `submissions` at the end of `iterate` stays. So do the edges in the "machines and domains" table of D001: `tail.lines.room()`, `revision_stdout`, `Indices`' `intake.room()` and `Fields::value`.
- **The `work_pending` guard in `protocol_up` stays,** whatever any test shows.
- **`hearth-service-builder/src/lib.rs` has one writer at a time.** Each task that changes it needs the last one that did.

## A failing test is a finding first

- **Don't weaken a test to get past it.** Read the log, and find what changed. No assertion or case goes but those your task names.
- **A ceiling moves only for a real change:** if one in `hearth/tests/builder/tests/suite/memory.rs`, `hearth/tests/builder/tests/suite/fuzzy/memory.rs` or `hearth/tests/tree/tests/suite/memory.rs` is passed, measure the peak, set the ceiling a quarter above it, rounded up to two significant digits (`skein/docs/foundation/testing-strategy.md`, 6), and give the old peak, the new one and why in the commit and the report. A ceiling that isn't passed stays.
- **`fixtures::CUTS`** (`hearth/tests/builder/src/fixtures/worlds/simulation_world.rs`) counts the calm client's submissions. Routing and pacing change no operation, so a change to it is a finding, for the report, before it's a new figure.

## A test of an order

Each one holds an order in `skein/docs/foundation/programming-model.md`, 7.2's sense: what stays ahead of what, and what keeps it there.

- **Its name states the order.**
- **A queue the test owns is `Queue::uncapped()`,** so a broken order shows as itself and not as a panic on a full queue.
- **It is small:** fixed seeds, under 0.05 CPU s where it can be. hearth's focused suite is over its 30 CPU s, and that figure doesn't rise. A test that must cost more says so in the report, with the measurement.
- **It bites.** Your task names the rule to break. Break it by hand, see the test fail for that reason, and restore it.

## The end of every task

- From `hearth/`, `python3 -I -B scripts/old_forms.py`, with `--write` only to lower a count for what your task removed.
- From the root, `python3 -I -B tools/citations/citations.py --all` stays at 0.
- `tests/boundary/tests/outward.rs` (package `fabrig-boundary`) keeps passing: none of these tasks should add text that names anything outside the repository, so it runs in the trunk's end check, and in your task only if yours does.
- The report gives the CPU seconds of the tests you added or changed, from nextest's times.
