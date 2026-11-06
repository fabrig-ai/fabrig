+++
id = "connect-cells"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/tests.rs"]
effort = "high"
minutes = 30
+++

# skein-forge-connection: a repeat connect can't silently drop an exchange

## Goal

Findings 1 and 7's first pin in `temper/docs/plan/debt-25/inputs/debt23-review.md`, both checked against the code. `Connection::connect` matches only what the link answers: `Start::Ready => state.idle()` on `Sending` or `Reading` discards the exchange, and the next HTTP event panics far from the cause; `Start::Handshaking => Handshaking { until }` does the same on a TLS connection. io's stream table says a connect completes once, so these cells can't happen: they say so where they would.

Acceptance:

- `connect` panics with `unreachable!("a connect completes once")` from `Handshaking`, `Sending`, `Reading` and `Idle`, whatever the link answers. `Connecting` and `Closed` keep the rows they have, unchanged, `Closed` included: this task changes no reachable cell.
- `a_connected_notice_keeps_the_close_under_way` also calls `c.failed(Failure::Unseen)` before `close` and asserts `take_ended()` gives that failure after `connect`. Breaking `Start::Closing => state` (build a fresh `Closed { closing: true, ended: None, draining: false }`) turns it red.
- One `#[should_panic(expected = "a connect completes once")]` test: a second `connect` on a `Sending` connection, through `f::ConnectionHarness`. Restoring the old arm turns it red. One test, not a row per state.
- Every test named under Steps passes with no assertion changed.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, findings 1 and 7
- `skein/docs/design/forge-connection.md`, 8.1
- `skein/docs/design/io.md`, the stream table's `Connecting` row

## Owns and may change

- **Owns:** the files in the header.
- **If a test now reaches the panic,** read why. A harness that repeats `Connected`: make it stop, as `skein/tests/forge/machine/src/hand.rs`'s `connected` does (`skein/crates/skein-forge-connection/src/component/tests.rs`'s `Harness::connected` and the test kits under `skein/testing/skein-forge-kit` are allowed for that, and only that), and name it in your report. Production sending a second `Connected`: stop and ask.
- **Stay off:** the `rest` calls and `until` fields: a later task of this plan renames them.

## Steps

1. Take the state first, so the impossible cells are written once, and answer the link only from what's left:

   ```rust
   pub fn connect(&mut self, env: &Env<Limits>) {
       let until = self.deadline();
       let state = match self.state.take() {
           Handshaking { .. } | Sending { .. } | Reading { .. } | Idle { .. } => unreachable!("a connect completes once"),
           state @ (Connecting { .. } | Closed { .. }) => state,
       };
       self.state = match self.link.connected(&self.link_env(env)) {
           Start::Handshaking => Handshaking { until },
           Start::Ready => state.idle(),
           Start::Closing => state,
       };
   }
   ```

2. The two tests in `connection/tests.rs`. Break each rule by hand as above, see the failure, and restore.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`. This changes what a repeat means, so run its readers: `heavy cargo nextest run -p skein-forge-connection --lib`, then `-p skein-forge-machine-world`, `-p skein-forge-protocol-world`, `-p skein-fake-peers-tests` and `-p temper-engine-protocol-forge-world`. Commit and push.

## Report

The common note's items; what you broke and what failed, for both tests; whether any harness needed changing, and why.
