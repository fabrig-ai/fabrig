# readiness-eintr: the plan

From `inputs/request.md`: status-gaps' question `readiness-eintr`, answered (a) on 2026-11-03 at
the director's word. Under the readiness backend an interrupted `Socket`, `Accept`, `Connect` or
`Open` completes `Other(EINTR)`, as the ring's does, so one error number means one thing on both
backends. One part, one task, in skein alone; every task names `notes/common.md`.

## The design, written by this cut

- `skein/docs/design/shell.md`, 7.4: a bullet states the rule for the four (made once, by
  neither the shared part nor the system's call; a `Connect` never calls `connect` twice), and
  the open-rule sentence goes. Section 10: the question for the four goes; in its place, the
  one this survey found still open (below). The header's revision date is today's.
- `skein/docs/design/shell.status.md`: the **Departs** 7.4 entry becomes a **Not yet** 7.4 item
  (the four still made again, and where), which the task removes. `STATUS.md`'s summary says so
  until then.

## Parts and tasks

1. **interrupted-calls** (skein, `skein/docs/design/shell.md, 7.4`), slice 1.
   - `tasks/interrupted-once.md`, high, ~50 minutes, needs nothing: a seam on `attempt`
     (`attempt_with`, the call as a parameter) and its tests in `readiness/interrupted.rs`; the
     rule in `attempt`; the four calls made once on Linux and Darwin (`socket`, `accept`,
     `connect`, `files::open`), `open_listener` and Darwin's `open_flags` keeping their retry;
     the status entries gone.

Critical path: `interrupted-once` alone.

## What was decided, and why

- **The retry is in two places, and both go.** The request says to drop the three from
  `attempt`'s retry. But each system's call wraps its syscall in `retry` (`linux.rs`,
  `linux/files.rs`, `darwin.rs`, `darwin/files.rs`), so `attempt` never sees their `EINTR`:
  dropping only `attempt`'s would change nothing, and `connect` would still be called twice.
  The task removes the retry from both, for the four alone; the callers outside the backend
  that share those calls keep their behaviour (`open_listener` wraps its `socket` in `retry`;
  Darwin's `open_flags` keeps `retry` for its other two readers).
- **Only the four.** The readiness backend also makes again, on `EINTR`, sixteen other
  operations the answer did not name (`Bind`, `Listen`, `Shutdown`, the file operations but
  `Open`, `List`, `Lock`, `Wait`, `Spawn`, `Signal`, `Usage`, `Window`), where the ring completes
  its own entries `Other(EINTR)`. Bringing them under the rule takes the same change in about
  sixteen calls on each system, beyond "one small skein task". It is the user's: question
  `other-interrupted` in `plan.toml`, recommending the same rule, and `shell.md` 10 holds it
  open meanwhile. No task waits for it.
- **The test seam is new.** The request names "the backend's fake syscall seam, as its other
  errno tests do"; the readiness backend has none (its errno tests provoke real errors), and a
  real `EINTR` on a non-blocking socket cannot be provoked in a one-thread test. The task adds
  one, `attempt_with`, shaped as the ring's seam tests are (`linux/ring/tests/write_retries.rs`),
  and its tests also pin that the eight transfers are still reissued.

## Overlaps with other drafts

None found: no branch of `reference-m1` or `configuring-p1` touches `skein/`, and the queued
drafts (`v0.1.0-ladder`, `sign-in-wait`, `reference-walks`) name no files yet. `STATUS.md`'s
summary paragraph is shared by every skein draft; this task edits one clause of it.
