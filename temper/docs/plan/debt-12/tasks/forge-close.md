+++
id = "forge-close"
part = "forge"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/calls.rs", "skein/crates/skein-forge-connection/src/component/tests.rs", "skein/crates/skein-forge-connection/src/tests/connection.rs", "skein/tests/forge/machine/src/peer.rs", "skein/tests/forge/machine/src/machine.rs", "skein/tests/forge/machine/tests/suite/machine.rs"]
effort = "high"
minutes = 75
+++

# The forge connection's close: its bound pinned, no spin, its guard first

## Goal

Four changes to how a forge connection closes, none of which needs anything new of the link. Deleting
`link.fire` today leaves everything green while a stalled peer holds `Lifecycle::Closing` open forever: pin it.
A closing connection with an `heard` nobody takes asks for a turn at once until its socket closes: stop that.
`pump` runs link events while an answer is pending or the connection is closed: guard them. And two dead lines
go, which frees `Link::socket`.

Acceptance:

- A machine test fails with `link.fire` deleted from `Component::fire`, and fails with `c.link.next_deadline()`
  deleted from `Component::next_deadline`.
- A test fails with `c.link.has_work()` put back to `c.ready()` in either `if` of `drive`'s last loop.
- A test fails with `link.pump` run above the guard unconditionally.
- `rg -n 'link\.socket\(\)|link\.closed\(\)' skein/crates/skein-forge-connection/src` finds nothing.
- `heavy cargo nextest run` passes for `-p skein-forge-connection -p skein-forge-machine-world -p
  skein-forge-protocol-world -p skein-fake-peers-tests -p temper-engine-protocol-forge-world`, with no assertion
  and no timing of an existing test changed: step 3 changes an order those read.
- `heavy cargo clippy -p skein-forge-connection -p skein-forge-machine-world --all-targets -- -D warnings` passes.
- Production stays within a few lines of its base; the tests about 60 lines, the hold included.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/notes/contracts.md`, 3
- `temper/docs/plan/debt-12/inputs/link-review.md`, findings 1, 3, 8 and 6's forge half
- `skein/docs/design/link.md`, 3.3
- `skein/docs/design/forge-connection.md`, 8.1
- `skein/tests/forge/setup/src/tls.rs`, for `hold_room`

## Owns and may change

- **Owns:** the files in this spec's `files`.
- **Not yours:** `skein-link`; `forge-tidy` does `link_env`, `Upload::of` and `upload()` in this crate, after you.
- **Options:**
  - If `world.close()` can't run to its tick bound with the room held, drive `Request::Close` by a small
    `Machine` method beside `close`, and say so.
  - `Machine` has no `next_deadline` or `now`: add one-line readers.
  - If step 3 turns a test red, stop and ask, with the test and what moved: don't change the test.

## Steps

One commit each, in this order: they're in order of value.

1. **The bound.** `Peer` gets `hold_room`, as setup's world has: while it's set the peer answers no demand for room.
   `Machine::hold_room()` sets it on every peer. Beside
   `a_secure_graceful_close_reaches_the_server_before_socket_settlement` in `suite/machine.rs`:

   ```rust
   #[test]
   fn a_stalled_secure_close_is_aborted_at_the_close_timeout() {
       // transport_machine(true, the same two answers), run_for(SECOND), script_done
       world.hold_room();
       assert!(!world.close(), "a stalled peer holds the close");
       let until = /* now */ + fixtures::shared().io.close_timeout;
       assert_eq!(world.next_deadline(), Some(until));
       assert_eq!(world.tls_closed(), [false], "the peer never got close_notify");
       assert_eq!(world.aborts(), 0);
       // advance to a nanosecond before: still none; then to until:
       assert_eq!(world.aborts(), 1);
       assert_eq!(world.closed_events(), 1);
       assert_eq!(world.event_order().last(), Some(&"closed"));
   }
   ```

   Break both: delete `link.fire` from `Component::fire`, then the `next_deadline` line.
2. **No spin.** `skein-forge-connection/src/component/calls.rs`, `drive`'s last loop: both `if c.ready()` become
   `if c.link.has_work()`. After `c.close` the connection is closing, so `pump` moves the link alone, and only a
   holder takes `heard`, in its own loop with its own wake.

   A test in `component/tests.rs`, on `fixtures::Harness`: an idle call connection, a call done and no holder,
   takes `skein_io::Event::Failed`. That leaves an `heard` nobody takes. Before its socket's `Closed`,
   `next_deadline()` isn't `now`. Break it by restoring each `if`.
3. **The guard.** `Connection::pump` in `connection.rs`:

   ```rust
   for _ in 0_u32..64 {
       let held = self.answer_pending() || self.phase == Phase::Closed;
       if !held || self.closing {
           match self.link.pump(io) { /* as it is */ }
       }
       if held {
           break;
       }
       // the rest, as it is
   }
   ```

   `closing` always has `Phase::Closed`, so `held` covers it; if you'd rather say that in the guard, say why in the
   report. A test in `tests/connection.rs`, on `fixtures::ConnectionHarness`: a started connection takes
   `failed`, then an `Up` event; after `h.pump()` the link still `has_work`, and the phase is `Closed`. After
   `close`, a pump drains it. Break it by moving `link.pump` above the guard.
4. **Dead lines.** `Connection::stream`'s `if self.phase == Phase::Closed && self.link.socket().is_none()` goes:
   `up` drops an event once the link is closed, and no `Stream` comes before `Connecting`. `Component::up`'s
   `c.link.closed()` at `Event::Closed` goes: the connection is retired a few lines on and nothing reads its link.
   Read `self.finish` first; if it reaches that link, keep the line and say so.
5. Fmt, clippy, tests, citations. `python3 -I -B scripts/old_forms.py` in `skein/`. Commit and push.

## Report

The report of your implementer text, and: each thing you broke and what failed; whether any test's trace moved
at step 3; net lines, production and tests apart.
