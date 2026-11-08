+++
id = "tui-colours"
part = "tui-colours"
kind = "feature"
needs = ["tui-local"]
files = ["temper/crates/temper-client-tui/src/theme.rs", "temper/crates/temper-client-tui/src/tests/theme.rs", "temper/crates/temper-client-tui/src/tests/goldens.rs", "temper/crates/temper-client-tui/src/tests/golden/**", "temper/tools/journeys/src/tui/print.rs", "temper/tools/journeys/src/tui/print/tests.rs", "temper/docs/design/client/tui/journeys/theme.css", "temper/docs/design/client/tui/journeys/sign-in-approved/*.html"]
effort = "medium"
minutes = 90
+++

# tui-link-colours: the TUI's themes on the terminal's own colours

## Goal

The TUI's base becomes the terminal's own default text and background, so that emacs
vterm's ground and body text show through and the TUI paints only its accents, as Claude
Code does; the palette keeps the accent, dim, faint and good colours and `bg` and `ink`
as the journeys' pages' stand-ins; the focus is the accent beside its mark, with no
background; and `Scheme::of_background` tells dark from light for the terminal's answer
(the run uses it, later).

Acceptance:

- `temper-client-tui`'s `theme.rs`, as `temper/docs/design/client/README.md`, 6.15:
  - `theme(scheme).base` is `Paint::NONE`; no region's or token's paint names a
    background;
  - `palette(scheme)` is six colours, `bg`, `ink`, `dim`, `faint`, `accent`, `good`, in
    that order, their values as now; `sel` goes;
  - `focus` is the accent as foreground, no background, no attribute;
  - `Scheme::of_background(rgb)`: `Light` where `2126 * R + 7152 * G + 722 * B` (8-bit
    components, in `u32`) is above `1_275_000`, else `Dark`; a `const fn`, exported with
    `Scheme`.
- Rows in `src/tests/theme.rs`, each a failing test first: the base names no colour in
  either scheme; the palette's six names in order and none is `sel`; the focus's paint;
  `of_background`'s table: black and white, solarized dark `#002b36` and light
  `#fdf6e3`, the theme's own `bg` of each scheme giving that scheme, and the two values
  either side of the threshold (`#808080` light, `#7f7f7f` dark: check them against the
  formula and keep whichever pair sits either side).
- The goldens (`src/tests/golden/*.txt`), kept by hand (`notes/common.md`): edited by
  hand from `the_tuis_screens_are_their_goldens`'s mismatch print, so that every legend
  line loses `back #191816` or `back #FFFEFD`, the focused field's cells take the accent
  as foreground and lose the `sel` background, and paints that become one share a
  letter; nothing else moves, and the report names any line that changed beyond those.
- The TUI's journey pages: `temper/tools/journeys/src/tui/print.rs` keeps the `pre`'s
  colours `ink` and `bg` (it names them; check it does not take them from the base), and
  its tests (`print/tests.rs`) index the palette by name or by the new order (`accent`
  is the fifth), their `b-sel` row a span of a palette foreground and a background that
  is no palette's (the `style` form) or the `f-accent` alone; the committed `theme.css`
  and `sign-in-approved/01.html` and `02.html` reprinted by `cargo run -p
  temper-client-journeys` (its texts and recording unchanged), as
  `temper/docs/design/client/tui/journeys.md`, 6.4, says.
- No status file changes: the task `again` deletes this task's **Not yet** entries
  once every task has merged, so that no two tasks running at once edit neighbouring
  lines of one status file.

## Inputs

- `temper/docs/design/client/README.md`, 6.15 (all of it).
- `temper/docs/design/client/tui/journeys.md`, 6.2, 6.4 (*A row's cells*, *`theme.css`*)
  and 7 (*The print*).
- `skein/docs/design/ui-screen.md`, 3 (*The terminal's default*) and 4 (*Its print*).
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 3.

## Owns and may change

- Owns the files in `files`. `temper-client-tui`'s `state.rs`, `view.rs` and `note.rs`
  are `tui-local`'s, landed: leave them.
- Readers of `palette` and `theme`: `temper/tools/journeys/src/tui/print.rs` and its
  tests (yours, above); `temper-client-shell-tui` reads `theme()` and `Scheme` only.
  Another reader that indexes the palette is a compiler-forced or oracle edit, allowed.
- The browser's journeys (`temper/docs/design/client/journeys/`) and its `theme.css`
  must not change: if the reprint touches them, it's a red; stop and say so.
- If the reprint finds `sign-in-approved` stale (it should not: no duty changes here),
  leave it for the task `again` and say so.

## Steps

1. The rows; `heavy cargo nextest run -p temper-client-tui --lib`; see them fail.
2. The theme; the goldens edited; the print's tests; the reprint
   (`heavy cargo run -p temper-client-journeys`).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-tui -p temper-client-journeys --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-tui -p temper-client-shell-tui -p temper-client-journeys`;
   `heavy cargo check -p temper -p temper-client-world --tests`.
4. Commit, push.

## Report

As the implementer's text says; the golden lines that moved, the threshold rows, and each
row's failure seen.
