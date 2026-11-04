# bugs-temper: the plan

From `inputs/bugs-temper.md`: three temper bugs the debt backfill routed. Each is small, sits in
its own crate and needs nothing of the others, so each is its own part and slice, released at
once. `notes/common.md` holds the standing rules every task names.

## Parts and tasks

1. **github-base** (bug 1; `temper/docs/design/protocol/configuration.md, 3`), slice 1.
   - `tasks/github-base.md`, medium, ~20 minutes, needs nothing: the base loses its final `/`,
     and `machine_remote` measures the same value. Closes the **Departs** entry.
2. **peer-ports** (bug 3; `temper/docs/design/protocol/setup.md, 7.5`), slice 2.
   - `tasks/peer-ports.md`, medium, ~30 minutes, needs nothing: `prepare` and `checked_peers` take
     their ports from one function.
3. **start-story** (bug 2; `jig/docs/design/domain/engine.md, 6`), slice 3.
   - `tasks/people-start-story.md`, medium, ~30 minutes, needs nothing: the ignored story runs on
     a row a start refuses, a second test holds the panic. Closes the **Not yet** entry, with no
     product code changed.

Critical path: none. The three run side by side; the longest is about 30 minutes.

## What was decided

- **Bug 2 stays a panic by default.** The request calls it a bug, but
  `jig/docs/design/domain/engine.md`, 6 (the user, 10-28) has a start judge no record and panic at
  a link it cannot make. The assertion followed that rule on 10-30; the ignored test (10-31) and
  the status line (11-02) came after. Refusing would reverse a user's decision and reach into two
  jig crates and jig's design, against "keep to the bugs" and "no reshaping". So
  `people-start-story` builds the design's reading, and the plan's question `duplicate-identity`
  asks the user. Nothing waits on it. The rule that a fix's test fails on the old code has nothing
  to apply to here, and the spec says so.
- **Bug 3 has a cause, found by reading and not reproduced.** The survey's finding was checked
  against the source: `prepare`'s loop over `deployment.checked.setup.entries` never runs, because
  `setup.rs` leaves `entries` empty, so the owned and fake listeners are not excluded, while
  `checked_peers` counts them. In a preparation importing a source nothing holds those ports, so
  `bind(0)` can return one. The check also runs on the fresh path, so the failure most likely came
  from the test's first `start`; the survey puts the odds near 1 in 3,500 preparations (an
  estimate). About 30 tests take the same path, so the test's name is not itself evidence. The
  original report is outside this repository and was not read.
- **Bug 1 leaves stored forges alone.** A kept forge keeps the place its record holds
  (`configuration.md`, 8), so a GitHub forge recorded before the fix keeps its double `/` until it
  is removed and connected again. No patch or trimming: "no reshaping".

## Overlaps not avoided

- `github-base` and fabrig-scale (integration) share `temper-service/src/system.rs`,
  `startup/checks.rs` and `tests/welcome_policy.rs`. Their changes are in other hunks.
- `github-base` and fabrig-scale both edit the single long **Departs** line of
  `temper/docs/design/protocol/configuration.status.md`. Whichever lands second meets a textual
  conflict there; both edits stand.
- debt-5's areas are untouched.
