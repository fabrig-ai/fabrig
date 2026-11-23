# The run: delivering a change

## 1. Delivering a change

### 1.1 Checks

- **A directory says what its checks are,** by convention: an
  executable at the path the conventions name, `.smith/check` unless the
  charter says otherwise (temper says `.temper/pre-pr`), looked up in each
  writable directory when the run starts. A directory without one has no
  checks, and nothing above the agent needs to know either way.
- **Checked is delivered,** but for a save, which runs no check (1.5).
  The checks run as contained processes with
  deadlines, and the delivery that follows is made, under the lease over
  the whole workspace (`run.md`, 5.3): nothing else holds write authority from the
  checks' start to the delivery's answer, so the host makes exactly the
  tree that was checked durable. That holds by construction: a write runs
  alone in its session, a sub-agent lives only as long as the call
  that asked for it, and no job runs through a check or a delivery.
- **Jobs are cancelled first.** A `finish` that delivers, a `deliver` and
  a `save` cancel every running job, and wait for each to settle, before
  the checks start, or before the delivery when no directory has checks
  or the call is a save
  (`run.md`, 5.3; tools.md, section 5). Only their own session's can be running:
  they run alone, and a child lives only as long as its call. A job so
  cancelled ends `cancelled`, which the next `job` call naming it
  answers.
- **A long operation is said.** The run tells its host when checks start,
  with their deadline, and when they end, as it does for each LLM
  completion, each command its sessions run, until it ends or is
  answered `running`, and each `job` call's wait, so a long test suite,
  a slow completion or a long build is not taken for a hang (host.md,
  section 4).
- **The checks' bound** is the charter's, where it names one
  (`../protocol/charter.md`, section 2), and the run's `check_timeout`
  otherwise: each check is given that much from its start, or what is
  left of the call's own deadline when that comes first, cut at the
  run's end. A host that knows a run's work is large (temper, for a
  project's goal-sized landings) gives its charter a larger bound; the
  reserve still holds one run at `check_timeout` alone (budget.md, 1),
  so a larger bound's check in the wind-down is cut at the run's end.
- **A wait for what a check needs** is not its time. A check queued for a
  place its environment limits (a build's, among other repositories'
  builds) says so on its stderr: the whole line `smith-check: waiting`
  stops its bound's clock, and `smith-check: admitted` starts it again,
  each newline-ended, those bytes exactly; a `waiting` while it waits,
  or an `admitted` while it does not, changes nothing. While it waits,
  only the call's own deadline and the run's end bound it; admitted, it
  has what was left of its bound, from then, cut as before. A check that
  says neither is bounded as before. The deadline told to the host at
  its start can so move later, never past the call's own deadline,
  within which the host's watchdog never stops a run (host.md, section 4).
  The time a check was given, in its feedback and its retry's, is its bound.
- **Out of time, once more.** A check stopped at its deadline (it timed
  out: it neither exited, nor failed to start, nor was cancelled) runs
  once more at once, on the same tree under the same lease, given twice
  the time its first run was given, cut at the call's deadline and at
  the run's ordinary deadline, so the retry never spends the finishing
  reserve. The host is told the second check's start with its deadline,
  as the first's. There is no retry while the run winds down, nor where
  the cut leaves no more than the first run was given; and never a
  third. Each directory's check retries on its own as it times out; a
  check that fails otherwise refuses at once, as before. A check that
  ran out of time a second time, or could not be retried, is feedback
  as below, `timed-out`, with the time its last run was given: a check
  that only ran out of time is told as that, never as a failing check's
  exit.
- **A failing check is feedback,** with its output's tail, like any
  other. It names its directory and the check it ran there (the
  convention's path), how it ended, and the time it was given, for
  every exit: the checks' bound, or what was left of the call's
  own deadline when that came first. A stopped check keeps its output's
  bounded tail through the last byte read before its pipes close,
  including output read during stopping, as a check that exits does.
  An empty tail with no discarded bytes records no output read before
  the pipes closed.
- **Not a security boundary.** An agent can change what a check runs;
  whatever guards the host's systems (temper's CI and its authority's
  requirements) guards them. The checks catch failures early, inside the
  run that can fix them.

### 1.2 Delivery

`deliver` is the run's request to its host: make the checked state
durable, with the result's fields. The host takes exactly that state, in
each writable directory that changed, and does whatever delivering means
to it: temper commits and pushes a branch; the local host commits a
repository in place and keeps a plain directory's files as they are;
another host might copy files out or attach them to a ticket.

The host answers in smith's terms:

- **delivered,** naming per directory what it made, as text the run
  passes on (a commit, a branch's head, files kept);
- **nothing,** when no directory changed, which is how a declared
  change is checked to exist: the LLM is told its delivery failed;
- **refused,** for something the LLM can fix, naming it: a conflicted
  file that still holds a marker (1.3);
- **failed,** with a reason (unreachable, refused by the target, timed
  out, broken, too large, missing, busy, unavailable, cancelled, unknown)
  and the tail of the host's diagnostic output with the number of bytes
  dropped before it, within a cap the vocabulary seals
  (`../protocol/limits.md`); with
  several directories, the first that failed in workspace order;
- **stale:** where the change goes has moved under the run, so every
  later delivery of this run would find the same. It ends the run as
  stale, and the host decides afresh.

The delivery value, `Delivered`, reports what it owns through
`owned_bytes()`; `owned_bound()` bounds its receipt cells and opaque
receipt bytes (skein/docs/foundation/programming-model.md, 6.3).

Every answer but delivered and stale goes back to the LLM. A delivery
that lands is the result, even if the run was ending meanwhile; a
delivery in flight is never abandoned (host.md, section 2).

### 1.3 Merges in progress

A repository may start from a merge in progress, the host having merged
and left the conflicts in the working tree; the start names the files in
conflict, and the run tells the LLM. It also tells it what a merge in
progress holds, before it looks: `HEAD` is its work so far, `MERGE_HEAD` the
tip the host merged in, and every other staged file is what that merge
brought, which it leaves in place. The LLM edits the files in conflict and
runs the checks; it has no git writes (tools.md, section 5). Delivered, the host
commits the tree as the merge, and refuses while a file the merge left
in conflict still holds a marker, naming it.

### 1.4 Delivering mid-run

A run whose charter grants `deliver` may deliver before it finishes, as
a chat does with a small fix before handing it on: checked, then
delivered, the same way, and the run goes on. A finish with a change
delivers what is there at finish. A mid-run delivery is not the run's
result: a run ended while one is in flight waits for its answer, then
answers as it was ending; the host has the delivery's own answer.

### 1.5 Saving unchecked

A run whose charter grants `save` may hand its tree to its host at any
time with no check run, so that work in progress is kept before the run
asks a question and waits (the owner, 2026-11-09). A host that keeps
work on a branch has the run's tree there to read while the question is
open.

- **Granted only with `deliver`,** so only in a Finite run: a charter
  that grants `save` without `deliver` is refused at the entrance
  (../protocol/charter.md, section 2). smith's local host grants none
  (../protocol/host.md, 5.5).
- **No input.** `save` takes an empty object: it carries none of the
  result's fields, since it is no result.
- **A delivery without its checks.** A save takes the lease over the
  whole workspace and cancels every running job, as a delivery does
  (1.1), and asks its host to make the tree durable at once, as a save:
  the host knows it from a delivery by its ask (../protocol/channel.md,
  section 4), and marks what it makes as unchecked, in its own terms (a
  commit's message, a receipt's words). Its answer is a delivery's
  (1.2), told the LLM the same way; stale ends the run, as a delivery's
  does.
- **Never a result.** A save does not finish the run and is not its
  change. A `finish` with a change, and a `deliver`, run the full checks
  whatever was saved before them, so only a tree that passed them is
  delivered as checked. A host that merges a run's work merges what a
  checked delivery made, never a save alone.
- **Main's alone, and not in wind-down.** Like `deliver`, a save is an
  exclusive write of main's; a child is never offered it, and a run
  winding down offers it no more: a save asked while main is closing or
  the run is winding down is answered cancelled.
