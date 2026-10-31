# The stage-site family: moving a host onto `skein_io::stage`

For the echo-first and stage-sites tasks. Read `common.md` first.

## The shared items (landed by kit-io and shell-surfaces)

- `skein_io::stage::up(io, env, completions, events, submissions, now)`:
  io's ready list, then the completions once the ready list is empty,
  then the due timers, each step only while the kernel's submission
  queue has room for that step (`MAX_OUT_RESUME`, `MAX_OUT_UP`,
  `MAX_OUT_FIRE`).
- `skein_io::stage::down(io, env, requests, submissions)`: the requests,
  while io takes work and the queue has room for `MAX_OUT_DOWN`.
- `skein_shell::Host::next_mechanism_deadline` (defaults to `None`) and
  `Host::next_deadline`, now provided: the earlier of the mechanism and
  the policy deadlines.

## What every site task does the same way

- Each hand-written io stage in its files (the `io::resume` loop, its
  `io::up` loop over completions, its `io::fire` loop, and the matching
  `io::down` loop over requests) becomes one call to `stage::up` and one
  to `stage::down`. The private copies (`room` helpers, loop bounds) go.
- Every site then checks the kernel's room before taking work, as
  `testing-strategy.md` 6 requires: that is the change of behaviour these
  tasks make, and the only one.
- A host in your files whose `next_deadline` merges io's deadline with
  `next_policy_deadline` by hand states `next_mechanism_deadline` (io's,
  and any other mechanism layer's it merged) and drops its
  `next_deadline`. A host whose `next_deadline` does more keeps it.
- Lower the `io_resume` count with `--write`.

## Pre-approved defaults

- **A site with no `io::down` loop** (it never asks io for work):
  call `stage::up` only.
- **A site whose stage interleaves its own work** between resume, up and
  fire (it reads an event between two calls on purpose): move it only if
  the order its tests pin is kept; otherwise leave it, and name it with
  the test that pins it in your report.
- **A test that drives one `io::resume` by hand to observe one step**
  (a conformance or io-world test of io itself): leave it, and name it.
  The count's target allows only such named sites.
- **A site that drains completions it got from somewhere other than a
  `Queue<Complete>`:** collect them into the host's queue first if that
  is a few lines; otherwise leave the site and name it.
- **A world that fills the kernel's queue on purpose** and now sees a
  stage stop at the room check: that is the rule being kept; adjust the
  test's expectation and say so.
