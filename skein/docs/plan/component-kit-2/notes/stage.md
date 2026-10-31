# The stage family: hosts onto `skein_io::stage` and the shell surfaces

For the echo-first, stage-sites and world-sites tasks. Read `common.md`
first.

## The shared items

Landed by `kit-io` (`skein/crates/skein-io/src/stage.rs`) and `kit-shell`
(`skein/crates/skein-shell/src/drive.rs`, `skein/testing/skein-world/src/referee.rs`);
read their `///` contracts on your base.

- `skein_io::stage::up(io, env, completions, events, submissions, now)`:
  io's ready list, then, once nothing is ready, the completions, then,
  once none is left, the due timers; each step only while the kernel's
  submission queue has room for it (`MAX_OUT_RESUME`, `MAX_OUT_UP`,
  `MAX_OUT_FIRE`).
- `skein_io::stage::down(io, env, requests, submissions)`: the requests,
  while `io.takes()` and the queue has room for `MAX_OUT_DOWN`.
- `skein_shell::Host::next_mechanism_deadline` (provided, `None`) and
  `Host::next_deadline`, now provided: the earlier of the mechanism and
  policy deadlines.
- `skein_shell::Delegate { host, host_mut }`: a type that states the two
  accessors is a `Host`, every method forwarded. skein-world re-exports it
  beside `Host`.
- `skein_world::Referee::{act, next_deadline, overdue}` have defaults: do
  nothing, `None`, `None`.

## What every task does in the files it owns

1. **The stage.** Each hand-written io stage (an `io::resume` loop, the
   `io::up` loop over completions, the `io::fire` loop) becomes one
   `stage::up` call, and its `io::down` loop over requests one
   `stage::down` call. The private copies go with them (`room` helpers,
   loop bounds). After the move every site checks the kernel's room
   before taking work (testing-strategy.md, 6; io.md, 8): that is the one
   change of behaviour these tasks make.
2. **The deadline.** A host whose `next_deadline` is a hand merge of io's
   deadline (and any other mechanism layer's) with `next_policy_deadline`
   states `next_mechanism_deadline` instead and drops `next_deadline`. A
   host whose `next_deadline` does anything else keeps it.
3. **The dispatchers.** An `enum` (or struct) whose `impl Host` forwards
   each method to the host it holds becomes `impl Delegate` with the two
   accessors, and its `impl Host` goes. Accessors it already has
   (`fn host`, `fn host_mut`) move into the `impl Delegate`.
4. **The referees.** A `Referee` impl whose `act` does nothing, or whose
   `next_deadline` or `overdue` returns `None` unconditionally, drops
   that method.
5. **The counts.** Lower `io_resume` and `host_dispatch` with `--write`.

## Pre-approved defaults

- **A site with no `io::down` loop** (it never asks io for work): call
  `stage::up` only. A site with only a down loop: `stage::down` only.
- **A site that interleaves its own work** between resume, up and fire
  (it reads an event or handles its own completion between two calls on
  purpose): move it if the order its tests pin is kept, for instance by
  routing its own completions before `stage::up`; otherwise leave it and
  name it, with the test that pins the order.
- **A site that handles io's events after each single step** (a loop
  of one resume, up or fire, then its events): move it only if its event
  queue holds what a whole `stage::up` pass tells (sized from io's limits,
  or uncapped); otherwise leave it and name it. Never enlarge a queue a
  limit or a design sizes.
- **A site that measures each step** against its `MAX_OUT` (an io bound
  test): leave it and name it.
- **A site that drains completions from somewhere other than a
  `Queue<Complete>`:** collect them into a queue first if that is a few
  lines; otherwise leave it and name it.
- **A world that fills the kernel's queue on purpose** and now sees a
  stage stop at the room check: that is the rule being kept. Adjust the
  test's expectation, never the stage, and say so.
- **A test that now fails because `Delegate` forwards `exit`,
  `state_digest` or `drain`**, which the old dispatcher did not: the
  forwarded value is the host's own and is right; fix the test's
  expectation and name it. If the failure is a host's own bug that the
  forwarding exposes, stop with a `Question:`.
- **A host that holds io but stated no mechanism deadline** in its old
  `next_deadline` (it merged nothing): keep its `next_deadline`.
- **A dispatcher variant that holds no `Host`** (a step machine its
  dispatcher drove through free functions and a `Limits`): a small host
  type in the same file holds the machine and what the dispatcher used
  (its limits), implements `Host` with the calls the dispatcher made, and
  the variant holds it. Typed accessors (`as_client` and the like) keep
  their return types. Nothing moves into the machine's own crate.
- **A dispatcher whose `impl Host` does more than forward** (it adds work
  in `iterate`, or a deadline of its own): keep it; it is a wrapper, not
  a dispatcher, and is not counted.
