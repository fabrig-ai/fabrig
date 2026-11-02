# me-retention: the plan

From `inputs/request.md` (QA finding 13, `v0.1.0-ladder/finding-13` in
`temper/docs/development/v0.1.0-ladder.md`): `temper ctl DIR me` prints
`retention: 86400000000000 ns`. One direct part, one task, as the request asks.

## Parts and tasks

- **me-retention** (`temper/docs/design/client/tools.md, 6.1`), slice 1.
  - `tasks/retention-seconds.md`, medium, ~25 minutes, needs nothing: `me_text` prints
    `retention: N seconds` (whole seconds, a remainder dropped); a regression test in
    `src/tests/jig/reads.rs` (a day prints `retention: 86400 seconds`, and so does a day
    plus 999,999,999 ns); `golden/me.txt` regenerated (`retention: 1000 seconds`), its JSON
    twin unchanged.

## Critical path

`retention-seconds` (~25 minutes, up to ~50 with the buffer), then fab's end check on the
trunk.

## Overlaps

`v0.1.0-ladder`'s open `finding-4` adds one line to `render/document.rs` in `task_text`
(~line 261), far from `me_text`'s retention lines (193-195): the same file, no shared hunk.
No other open ladder branch touches the task's files.

## Calls made

- **The form:** the request asks for the duration form ctl already uses, not a new one.
  ctl prints durations as whole seconds with ` seconds`: the help's bound (`help.rs`,
  `Bound::Seconds`: `at most 3600 seconds`) and `--timeout`'s description (`300 seconds
  when absent`); argv reads `seconds, or a number with s, m or h`. Nothing in the
  workspace prints `1 day` (skein-ui's spans print one integer unit, seconds in jig's
  English, and ctl's text uses none). So `retention: 86400 seconds`, with no plural
  special case, as the help's bound has none.
- **A remainder below a second is dropped,** as the help's bound and skein-ui's spans drop
  it; a shorter stated retention is the safe side for a client deciding whether to resend.
- **JSON unchanged:** `me --json` keeps `"retention"` in nanoseconds, as its other
  durations and walls are.
- **Help unchanged:** no help page or help golden names the retention's form (`rows.rs`
  says "the requests' retention"; `ctl.md` "the retention `me` prints"), so the task
  touches no `docs/help/` and needs no help tests.
- **Left outside the task** (the request's one area): a refused worker hello's stop bound
  prints `allowed grace N ns` (`render/document/llm.rs`), the same raw form; the task's
  report lists it. For the designer at retirement: `tools.md` 6.1 could name the
  `retention: N seconds` line, and the debt entry finding-13 closes.
