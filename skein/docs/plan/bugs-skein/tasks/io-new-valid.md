+++
id = "io-new-valid"
part = "io"
kind = "feature"
needs = []
files = ["skein/crates/skein-io/src/layer.rs", "skein/crates/skein-io/src/tests/layer.rs", "skein/examples/echo/service/src/lib.rs", "skein/examples/echo/service/src/tests.rs"]
effort = "medium"
minutes = 25
+++

# `Io::new` asks for valid limits, and echo refuses others at startup

## Goal

`Io::new` asserts `is_usable`, then an `.expect` panics when `limits::flights` overflows, at `sockets >=
2^29`. `is_usable` has no capacity check; `skein_io::valid` does. `skein/examples/echo/service`'s `check`
asks only `is_usable`, so it passes such limits and `Io::new` panics after it. (Bug 15's second half,
re-checked: holds on that path; smith's and temper's paths ask `valid` first.)

Acceptance:

- `Io::new` asserts `crate::valid(limits)`, its message keeping "usable limits", its doc saying "valid".
  Its signature does not change.
- Echo's `check` asks `skein_io::valid(&self.io)`.
- The tests below fail on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/io.md`, 2 (`skein_io::valid`).
- `skein/docs/foundation/programming-model.md`, 6.3.

## Owns and may change

- Owns: `src/layer.rs` (`Io::new`), `src/tests/layer.rs`; `skein/examples/echo/service/src/lib.rs`
  (`check`), `skein/examples/echo/service/src/tests.rs`.
- Not here: forge-setup's like line (`host/limits.rs`) is `setup-unusable`'s.
- `is_usable` stays public with its readers: making it private is reshaping.
- About 70 callers of `Io::new` use small limits; tests with huge sockets stop at admission and name
  no panic text (surveyed).

## Steps

1. In `src/tests/layer.rs`, beside `io_does_not_run_under_limits_it_cannot_use`, add a
   `#[should_panic(expected = "usable limits")]` at `sockets: 1 << 29`. In echo's
   `limits_are_checked_layer_by_layer_and_against_each_other`, add `sockets: 1 << 29` expecting
   `Err(Unusable::Io)`. Run on the old code and quote the failures.
2. The two changes.
3. `heavy cargo nextest run -p skein-io --lib`, `heavy cargo nextest run -p skein-echo-service`. `cargo
   fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-io -p skein-echo-service
   --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the tests showed on the old code.
