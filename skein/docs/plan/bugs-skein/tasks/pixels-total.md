+++
id = "pixels-total"
part = "ui-web"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui-web/src/page.rs", "skein/crates/skein-ui-web/src/page/web.rs", "skein/crates/skein-ui-web/src/page/measure.rs", "skein/crates/skein-ui-web/src/tests/measure.rs"]
effort = "medium"
minutes = 25
+++

# `pixels` answers for any size

## Goal

`skein-ui-web`'s `pixels` (`src/page/web.rs`) prints a clamped floor and parses it back, with an `.expect`. NaN
prints `NaN` and `-0.0` prints `-0`, and neither parses, so it panics. A conforming browser gives neither on
today's paths, but a size from outside is no place for a panic. (Bug 19, its ui-web half, re-checked: holds, and
`-0.0` too.)

Acceptance:

- `pixels` is in a new `src/page/measure.rs`, `#[cfg(any(test, target_arch = "wasm32"))] mod measure;` as `fold`
  is, so a native test reaches it; `web.rs`'s three calls use it.
- Below one pixel, NaN and `-0.0` among it, it answers 0: `if value.is_nan() || value < 1.0 { return 0; }`, ahead
  of the line it has, whose floor and clamp then hold a whole number from 1 to 65535. No `as`.
- Its test table: NaN, `-0.0`, `-3.0` and `0.9` give 0; `100.9` gives 100; `70000.0` and infinity give 65535.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/ui-web.md`, 8 ("whole CSS pixels").
- `src/page/fold.rs`, and how it loads `src/tests/fold.rs` by `#[path]`.

## Owns and may change

- Owns: `src/page.rs` (the `mod` line), `src/page/measure.rs` (new), `src/page/web.rs` (`pixels` moves out, one
  `use`), `src/tests/measure.rs` (new).
- `Page::resized` and `Event::Drawing` do not change. `src/platform/worker*` is another plan's: leave it.

## Steps

1. Move `pixels` unchanged to `measure.rs` and write `src/tests/measure.rs`; run it and quote the panic on NaN:
   `heavy cargo nextest run -p skein-ui-web --lib -E 'test(/measure/)'`.
2. The guard.
3. `heavy cargo nextest run -p skein-ui-web --lib`. `.config/gate/wasm32.txt` lists the crate, so also `heavy cargo
   clippy -p skein-ui-web --target wasm32-unknown-unknown -- -D warnings`, and `heavy cargo clippy -p skein-ui-web
   --all-targets -- -D warnings`. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the moved code.
