+++
id = "connect-close"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/tests.rs"]
effort = "medium"
minutes = 15
+++

# skein-forge-connection: `connect` keeps a close under way

## Goal

Finding 1 of `temper/docs/plan/debt-23/inputs/debt21-review.md`, checked against the code. `skein_link::Link::connected` answers `Start::Closing` only once `Link::close` has run, and `Connection::close` and `abort` leave the state `Closed { closing: true, .. }`. `Connection::connect`'s `Start::Closing` arm rebuilds it with `closing: false, ended: None`, so `step` skips the link pump and `component/github.rs`'s `c.closing()` doesn't break. `component/calls.rs`'s sweep closes it again in the same `drive`, so nothing outward has been seen to differ.

Acceptance:

- The arm reads `Start::Closing => state`, and its three other lines go.
- A test in `src/connection/tests.rs` pins it: a connection that heard `close` before its connected notice still answers `closing()` after `connect`, in `Phase::Closed`. Its shape, from `handshake_bytes_do_not_write_an_exchange`'s setup:

  ```rust
  let mut c = Connection::new(&forge, &env.limits, f::shared(), env.now);
  let mut io = Queue::uncapped();
  c.link.connecting(Token::new(100), &mut io);
  c.close(&env, false, &mut io);
  c.connect(&env);
  assert!(c.closing(), "the connected notice keeps the close");
  assert_eq!(c.phase(), Phase::Closed);
  ```

- **The pin goes first and fails before the fix:** run it on the old arm, see it fail on `closing()`, then fix. The report quotes that failure.
- `heavy cargo nextest run -p skein-forge-connection --lib`, `-p skein-forge-machine-world` and `-p skein-forge-protocol-world` pass with no assertion changed.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/inputs/debt21-review.md`, finding 1
- `skein/docs/design/forge-connection.md`, 8.1
- `skein/crates/skein-link/src/link.rs`, `connected`, `close`

## Owns and may change

- **Owns:** the files in the header.
- **`ended` now survives the notice,** as `closing` does: the old arm dropped it too. If a test goes red on that, stop and ask, with the test's name and what it saw; don't keep the drop.
- **The other findings in this file aren't yours:** `connection-deadline` and `connection-states` take them after you.

## Steps

1. The test; run it; see it fail: `heavy cargo nextest run -p skein-forge-connection --lib -E 'test(=NAME)'`, with the test's full name.
2. The fix; the test again.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the failure the pin showed before the fix.
