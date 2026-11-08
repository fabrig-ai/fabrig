# inproc-runner: the plan

The focused and fuzzy suites gain a second mode of the CPU collector: each test
binary's selected tests in one process, binaries in parallel on as many lanes as the
machine has CPUs, CPU measured per process from `wait4` and per test not at all
(`inputs/request.md`). Nextest's path, `cpu-wrapper.c` and every slow tier stay as they
are; the gate switches to the new mode afterwards, in a change of its own.

Contracts: `notes/contracts.md`; standing rules: `notes/common.md`.

## Parts and tasks

1. `starved` (skein): under `TIERS_ONE_PROCESS` a starved pass writes its line and
   unwinds, ending only its test (`skein/docs/design/testing.md`, 5). Needs nothing.
2. `runner` (tools/tiers): `one_process.py`'s `run` and its lanes, its fixtures and
   `test_one_process.py`, to the run directory's files (`tools/tiers/design.md`, 11).
   Needs nothing.
3. `summary` (tools/tiers): `summary`'s reading of a one-process directory,
   `execute_one_process` and its exit statuses, and `tools/tiers/one-process`
   (`tools/tiers/design.md`, 10.2 and 11.9), built to `notes/contracts.md`, section 2,
   beside `runner`. Needs nothing.

The repairs item 10 of the request finds and the plan's close follow in the next batch.

## Decisions

- **The design is written by this cut:** `tools/tiers/design.md`, 1, 10.1 to 10.3 and
  11; `skein/docs/design/testing.md`, 5; `skein/docs/foundation/testing-strategy.md`, 8
  (*A binary is a process*, the user's words, and *What a test costs the suite*) and
  8.1 (*The budget is CPU seconds*), with the reason in `skein/docs/foundation/notes.md`;
  `temper/docs/design/domain/testing.md`, 6.1.
- **A binary's command line adds three flags to the request's:** `--nocapture`, as
  nextest runs a test, so that a test's output is in order between libtest's
  announcements and the starved line is not captured; `--include-ignored`, so that a
  selected ignored test runs (the names hold the selection); `--color never`.
- **The starved test is read from the output.** It unwinds with `StarvedPass`, which
  libtest reports failed; the runner reads it starved when its output's last line is
  the `skein-world: starved:` line (`notes/contracts.md`, 1). A destructor or panic
  printing after the line leaves it failed: the error is always to red.
- **A helper process exits.** `signal_sender_fixture` inherits `TIERS_ONE_PROCESS`,
  but it is not a test: its starved end is `Starved::exit`, the line and exit 0, which
  its test forwards with `pass_starved`.
- **The overrides are read by the runner itself,** in a subset of nextest's filtersets
  (11.3), refusing the rest; where the caller gives no filter, the profile's
  `default-filter` read the same way must agree with nextest's `filter-match` on every
  test, which proves the reading on the real configuration at every run.
- **`process-of-its-own`,** a test group, is the nextest override the runner honours
  for a test that cannot yet leave its process clean (request, item 10). It is
  declared only when the repairs need it.
- **`NEXTEST_PROFILE` is given,** as nextest gives it (jig's support and skein's live
  test read it); `NEXTEST` and every other inherited `NEXTEST*` variable are not.
- **Kept times** live in `TARGET/tiers-one-process/times.json`, the only file the mode
  writes outside its run directory (11.8).

## Overlaps with other drafts

This cut's design edits touch `temper/docs/design/domain/testing.md` and its status
file, which `ui-ask-drop/close`'s glob (`temper/docs/design/**/*.status.md`) covers:
one added **Not yet** entry, in a place that draft's entries don't touch. No task of
this plan's first batch touches a file another draft changes.

## Critical path

`starved`, `runner` and `summary` start at once; the repairs need all three, and the
close needs the repairs: `runner` (the longest) → repairs → close.
