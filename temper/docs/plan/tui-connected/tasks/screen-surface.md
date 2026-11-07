+++
id = "screen-surface"
part = "screen"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-ui-screen/**",
  "Cargo.toml",
  "Cargo.lock",
  "skein/docs/design/ui-screen.status.md",
]
effort = "high"
minutes = 70
+++

# skein-ui-screen: the theme, the frame and the focus

## Goal

The full-screen shell's renderer crate, with the surface every other
task of this plan builds on: the theme, the frame and its print, the focus,
and a character's width. The renderer itself is `screen-render`'s; this
task is small on purpose, so the renderer, the key reader and temper's
theme start from it at once.

This task departs from the implementer's text in one way: a status entry
is yours to change (acceptance 5).

Acceptance:

1. `skein/crates/skein-ui-screen` exists as a member of the root
   `Cargo.toml` and an entry of its `[workspace.dependencies]`, `no_std`
   with `alloc`, `#![forbid(unsafe_code)]`, its manifest's lint block
   `[lints] workspace = true` as `skein-ui-text`'s, depending on
   `skein-ui` and `skein-lib` alone.
2. It exports, with the names, fields and signatures of
   `temper/docs/plan/tui-connected/notes/contracts.md`, section 2: `Rgb`,
   `Paint` (with `NONE` and `over`), `Place`, `Arrange`, `Border`,
   `RegionLook`, `StyleLook`, `Glyphs`, `Theme` (with `valid`), `Size`,
   `Position`, `Cell` (with `BLANK`), `Frame` (every method of section 2),
   `Focus` and `width`. It does not yet export `render`.
3. `Frame::text` and `Frame::print` write exactly the forms of
   `skein/docs/design/ui-screen.md`, section 4; `Theme::valid` refuses what
   section 3 says; `width` answers as 5.1 says, from a table of ranges
   (East Asian Wide and Fullwidth: two; combining marks, U+200B to U+200F,
   U+2060 to U+2064, U+FE00 to U+FE0F and U+FEFF: none; every other: one),
   searched without indexing.
4. Step tests in `src/tests.rs` (tables, rows as the strict subset's
   tests are written beside skein-ui-text's), each named in the report:
   `Paint::over` (a colour over none, none over a colour, attributes
   or'ed); `Theme::valid` (a valid theme, two looks for one region, two
   for one style, a wide focus glyph, a control mask glyph); `width` (ASCII,
   `é`, U+0301, `中`, `가`, U+FF01, `●`, `─`, U+1F600); the frame's bounds (a new
   frame 0 by 0; `clear` cut to `most` on each side; `row` and `put`
   refused past the size; the cursor none after `clear`); `Frame::print` of a 3 by 6
   frame with two paints, a wide glyph and the cursor, compared with a
   golden kept in `src/tests/golden/print.txt`, written by hand from
   section 4.
5. `skein/docs/design/ui-screen.status.md`'s **Not yet** loses "3, 4: the
   theme, the frame and its print." and its 2 reads "2: `skein-ui-screen-terminal`
   and `tests/ui-screen`."; the rest is unchanged.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, sections 1 and 2:
  they bind.
- `skein/docs/design/ui-screen.md`, sections 2, 3, 4 and 5.1 (the width);
  5.4's `Focus`.
- `skein/docs/design/ui-text.md`, section 2, for the crate's shape;
  `skein/crates/skein-ui-text` as the sibling to copy its manifest, its
  `lib.rs` head and its tests' layout from.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** `Cargo.lock` gains the crate's entry; nothing
else names the crate yet.

**Options, each with its default:**

- The frame's cells: a `skein_lib::List<Cell>` with capacity for
  `most`'s cells, rows laid one after another; `row` gives a slice of it.
  If `List` gives no slice of a range, keep `Box<[Cell]>` of `most`'s
  cells made at `new`, and say so.
- `Frame::print`'s letters: in the order each paint first appears reading
  rows top to bottom, each left to right; the base is the paint of the
  cell the frame was last cleared with (`clear`'s `fill`), and prints as
  `.`. Keep that paint in the frame for this.
- `Size` and `Position` are this crate's own, not skein-io's: the renderer
  is free of io. The terminal converts.

## Steps

1. The crate: manifest, `lib.rs` citing `ui-screen.md` at module level,
   the modules `theme`, `frame`, `focus` and `width` (the width table may
   sit in `frame` or a module of its own: your choice).
2. The types and their methods, as section 2 of the contracts.
3. The tests (acceptance 4). Bite: make `over` drop an attribute once,
   and make `print` letter the base, and see their rows fail.
4. The status entry (acceptance 5).
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-ui-screen --all-targets -- -D warnings`; `heavy cargo
   nextest run -p skein-ui-screen --lib`; `heavy cargo check -p
   skein-ui-screen --target wasm32-unknown-unknown` if skein-ui's gate
   builds it for wasm32 (ui.md, section 2), else skip and say so. Commit
   ("skein-ui-screen: the full-screen shell's theme, frame and focus"),
   push.

## Report

As the implementer's text says, and: the cells' storage, how `width`'s
table is searched and its size in ranges.

## Answers

2026-11-07. *Two u16 sides never make a cell count past a u32, so
`Frame::new`'s refusal and its test cannot be: keep u16, or widen?* Keep
`Size { rows: u16, columns: u16 }`. `Frame::new(most: Size) -> Frame`: it
never refuses (contracts.md, section 2, now says so; an always-`Some`
`Option` would trip pedantic `unnecessary_wraps`). The refusal's test is
gone from acceptance 4; in its place a table of the frame's bounds: a new
frame 0 by 0, `clear` cut to `most` on each side, `row` and `put` refused
past the size, the cursor none after `clear`. Keep it small: no frame of
65,535 by 65,535 is made in a test.
