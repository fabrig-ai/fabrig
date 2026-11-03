# D034: tests/codec generates its fixtures twice, and skein-codec compiles one of them

- **Kind:** tests
- **Where:** `skein/tests/codec/build.rs`, `skein/tests/codec/tests/suite/names.rs`, `skein-codec/src/lib.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (phase 1 item 5) and `reviews/2026-10-31-skein-areas.md` (A8, F1 and F2)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** `skein/tests/codec` has no build script that regenerates its schemas, and `skein-codec`'s source has no `#[path]` into `skein/tests/`.

## What

- `skein/tests/codec/build.rs` regenerates every fixture schema into
  `OUT_DIR` with `skein_codegen::generate`, for `tests/suite/names.rs`
  to compare with the committed modules; the suite's `drift.rs` already
  requires the committed text to equal the generator's output.
- `skein-codec/src/lib.rs` compiles
  `../../../tests/codec/src/generated/v1.rs` as `generated_tests` under
  `#[cfg(test)]`, with `extern crate self as skein_codec`, so a step
  crate's unit tests reach into a test crate.

## Why it matters

The second generation compares a copy with itself and costs a build
script that compiles the generator; the `#[path]` points the dependency
the wrong way (D028).

## What pays it back

Delete `build.rs`, `names.rs`, the build-dependency and the `#[path]`
module. Under an hour, by the review's estimate.
