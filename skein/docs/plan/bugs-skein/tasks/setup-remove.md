+++
id = "setup-remove"
part = "forge-setup"
kind = "feature"
needs = ["setup-unusable"]
files = ["skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/host/tests.rs", "skein/docs/design/forge-setup.status.md"]
effort = "medium"
minutes = 30
+++

# `Remove` keeps the end a run held

## Goal

`skein-forge-setup`'s `Remove` (`src/host.rs`) cuts each run and makes up a `Failure` at `Stage::Admin`
naming `remove` for anything but a held `Stop`. A run that has finished, its `Done` still held past
`drive`'s pass cap, so has its `Done` replaced by a false `Stop`. `Abort`, just above it, guards this.
(Bug 8's second half, re-checked: holds in this case; a live run always has a real `Stop` after its
halt.)

Acceptance:

- `Remove` does as `Abort` does: no `cut` of a run holding `Done` or `Stop`, then it pushes the end the
  run holds, `Event::Done` or `Event::Stop`; anything else is `unreachable!("a halted run holds its
  stop")`. The made-up `Failure` is gone.
- The test below fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/forge-setup.md`, 6.3 ("each of its live runs ends at once").

## Owns and may change

- Owns: `src/host.rs` (the `Remove` arm; if `Abort` and `Remove` then share a few lines, a small private
  fn), `src/host/tests.rs`.
- `skein/docs/design/forge-setup.status.md`: remove this plan's **Not yet** entry (`notes/common.md`).
- temper's `lifecycle::done` meets a `Done` for a removed forge: it refuses a forge no longer configured
  or ended (`temper-service/src/setup/lifecycle.rs`, `done`, its first lines). No edit is expected; if a
  `temper-service` test shows otherwise, stop and ask.

## Steps

1. In `src/host/tests.rs`, write `a_removed_forge_keeps_a_held_done`, modelled on
   `remove_ends_its_runs_with_their_stop` (`src/host/runtime_tests.rs`, a child of `crate::tests::host`
   that cannot reach `Host::runs`, and whose `World` is private to it). A departure allowed here: reaching
   a held `Done` through the public entrance takes exhausting `drive`'s 256 passes, so the test, a child
   module of `host`, builds a `Host` from `crate::tests::host`'s `limits()`, `settings(&[1])` and
   `crypto()`, starts a run with `Request::Run`, sets the run's `pending` to a `Done` directly, then sends
   `Request::Remove`, and asserts `Event::Done`, one end.
   Run it on the old code and quote the failure (`Stop`, `Admin`, `remove`).
2. The fix.
3. `heavy cargo nextest run -p skein-forge-setup`, `-p skein-forge-setup-world`, and `heavy cargo
   nextest run -p temper-service --lib -E 'test(/setup/)'`. `cargo fmt`, `python3 -I -B
   skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.

## Answers

2026-11-04. Where does `a_removed_forge_keeps_a_held_done` go, `src/host/runtime_tests.rs` being a child
of `crate::tests::host` that cannot reach the private `Host::runs`? In `src/host/tests.rs`, the child test
module of `host`: this spec's `files`, Owns and step 1 now say so, and `runtime_tests.rs` is no longer
owned. Nothing moves: no relocation of `runtime_tests`, no visibility change, no edit to
`src/tests/host.rs`. `limits`, `settings` and `crypto` there are `pub(crate)`; `Run::pending` is visible
from `host`'s children. `World` is private, so the test calls `Host::down` itself, as this file's tests
hold their own `Env`.
