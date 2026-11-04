+++
id = "browser-contains"
part = "browser-kit"
kind = "feature"
needs = []
files = ["skein/testing/skein-browser/src/browser.rs"]
effort = "medium"
minutes = 15
+++

# The browser kit's `contains` takes an empty needle

## Goal

`skein-browser`'s private `contains` (`src/browser.rs`) calls `haystack.windows(needle.len())`,
which panics on an empty needle. `find::words` (`src/find.rs`) is the same search with the guard.
Today's callers pass literals, so this is the one spot left to close. (Bug 19, its seen half.)

Acceptance:

- `contains(haystack, b"")` is true, as `find::words` says, and no panic is left in it.
- A unit test fails on the old code, by that panic.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/testing/skein-browser/src/find.rs`, `words`.

## Owns and may change

- Owns: `skein/testing/skein-browser/src/browser.rs`: `contains` and a test beside it.
- `src/find.rs` is read, not changed. `contains`' call sites stay as they are.

## Steps

1. Add `#[cfg(test)] mod` in `browser.rs` (the file's first; `find.rs`'s tests are the model) with
   `contains_takes_an_empty_needle`. Run it against the old code:
   `heavy cargo nextest run -p skein-browser --lib -E 'test(contains_takes_an_empty_needle)'`.
2. Make `contains`' body `crate::find::words(haystack, needle)`.
3. The narrow test again, then `heavy cargo nextest run -p skein-browser --lib`. `cargo fmt`,
   `heavy cargo clippy -p skein-browser --all-targets -- -D warnings`. The browser tier (Chromium)
   is not needed.
4. Commit, push.

## Report

As the implementer's text says, with the panic the test showed on the old code.
