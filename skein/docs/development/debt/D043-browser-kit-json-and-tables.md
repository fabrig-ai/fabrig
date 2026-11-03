# D043: The browser kit keeps its own JSON decoder and `Vec` tables

- **Kind:** primitives
- **Where:** `skein/testing/skein-browser/src/wire/decode.rs`, `skein/testing/skein-browser/src/browser.rs`, `skein/tests/browser`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (themes T5 and T6, the browser rows; A11) and `reviews/2026-10-31-skein-areas.md` (A11, F9)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** M
- **Status:** open
- **Paid when:** skein-browser decodes the DevTools protocol with skein-json and holds its tables in lib's containers, without module-wide expects; the fake CDP peer in `skein/tests/browser` decodes commands with skein-json, not with the kit's own decoder.

## What

- `wire/decode.rs` (527 lines) is a second, recursive JSON parser
  (`value`, `object`, `array`), beside skein-json.
- The kit keeps `Vec` tables (9 non-test `Vec<` uses) and module-wide
  `#![expect]`s in six of its files.
- The fake CDP peer (`skein/tests/browser/src`, its world and tests)
  parses the kit's commands with `skein_browser::wire::decode`, so both
  sides read the wire with one decoder
  (`skein/docs/foundation/testing-strategy.md`, 4).

## Why it matters

A third JSON reader to maintain, and a fake that cannot catch the
kit's misreadings because it shares them.

## What pays it back

skein-browser onto skein-json and lib's containers; the fake peer onto
skein-json. D001's driver makes the first easier.
