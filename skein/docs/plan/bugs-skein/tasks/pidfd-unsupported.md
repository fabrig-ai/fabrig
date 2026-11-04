+++
id = "pidfd-unsupported"
part = "shell"
kind = "feature"
needs = []
files = ["skein/crates/skein-shell/src/process.rs"]
effort = "medium"
minutes = 15
+++

# `open_child_pidfd` off Linux names `ENOTSUP`

## Goal

`skein-shell`'s `open_child_pidfd` answers `Error::Other(45)` off Linux, Darwin's `ENOTSUP` as a bare number.
(Bug 16, its seen half. Its other half, the readiness table's wrapping generation, is dropped: `plan.md`.)

Acceptance:

- A private `fn unsupported() -> Error`, `#[cfg(any(test, not(target_os = "linux")))]`, answers
  `Error::Other(libc::ENOTSUP)`, and the off-Linux arm calls it. `process.rs` is `cfg(unix)`, so `libc` is there.
- The test below.

**What stands in for a failure on the old code** (`notes/common.md`): the gate builds no target that compiles
this arm. The test runs on Linux, where `ENOTSUP` is 95. So first write `unsupported` with the old `45`, run the
test and quote the failure, then name it. Say in the report that the off-Linux arm itself was not compiled here.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/shell.md`, 7.5.

## Owns and may change

- Owns: `src/process.rs` (`open_child_pidfd`, `unsupported`, its test, unfenced by platform).
- No reader changes: callers test the error's value nowhere off Linux, and on Linux nothing changes.

## Steps

1. `unsupported_answers_the_platforms_own_enotsup`: `assert_eq!(unsupported(), Error::Other(libc::ENOTSUP))`.
2. `heavy cargo nextest run -p skein-shell --lib -E 'test(unsupported_answers)'`, on `45`, then on the name.
3. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-shell --all-targets --
   -D warnings`.
4. Commit, push.

## Report

As the implementer's text says.
