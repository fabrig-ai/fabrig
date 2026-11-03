# D026: Three crates keep a `tests/` directory under `crates/`

- **Kind:** tests
- **Where:** `skein-forge/tests`, `skein-fake-forge-protocol/tests`, `skein-fail-stop/tests`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, layout) and `reviews/2026-10-31-skein-areas.md` (A0, A2)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 6 and decision D4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** no crate under `skein/crates/` has a `tests/` directory; what drove each crate from outside lives under `skein/tests/` or in its `src/tests/`, as `skein/docs/design/testing.md`, 6, puts it.
- **Measure:** `ls -d skein/crates/*/tests | wc -l`: 3 at base

## What

`skein-forge/tests` holds 6,781 lines; `skein-fake-forge-protocol/tests`
(a memory suite) 146 and `skein-fail-stop/tests` 70. `testing.status.md`
records the departure with no reason
(`skein/docs/design/testing.status.md`). skein-forge's integration tests
compile under step-crate lints, which the review tied to its 67
file-level expects.

## Why it matters

The layout rule is what lets a reader find a crate's tests, and the
step lints applied to test code cost exceptions that ordinary test
crates do not need.

## What pays it back

Move skein-forge's suite to a test crate under `skein/tests/forge/` (or
its `src/tests/` where it needs private items), and the two small ones
likewise; then drop the departure from `testing.status.md`.
