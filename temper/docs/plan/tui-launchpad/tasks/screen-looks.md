+++
id = "screen-looks"
part = "screen-looks"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui-screen/src/theme.rs", "skein/crates/skein-ui-screen/src/lib.rs", "skein/crates/skein-ui-screen/src/render.rs", "skein/crates/skein-ui-screen/src/render/**", "skein/crates/skein-ui-screen/src/tests.rs", "skein/crates/skein-ui-screen/src/tests/**", "skein/docs/design/ui-screen.status.md", "skein/docs/design/STATUS.md"]
effort = "high"
minutes = 150
+++

# tui-launchpad: skein's screen draws rules, a title's end, end-aligned lines and bands

## Goal

skein's full-screen renderer gains three looks the TUI's launchpad and chat need, all
generic and named by the theme: a region framed by two rules with no sides (the box
between `────` rows), a border title's part drawn at the border's end (the address in
`╭─ temper · Hello ──── 127.0.0.1:8367 ─╮`), and, on a style token's look, an alignment
to the end (`written by the status agent 07:42` flush right) and a band (a background
across the region's inner width, the person's words in a chat).

Acceptance:

- `skein_ui_screen::Border` gains `sides: bool`; `skein_ui_screen::Align { Start, End }`
  (exported from `lib.rs`); `StyleLook` gains `align: Align` and `band: bool`, exactly as
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1, spells them.
  `Theme::valid` checks a border's glyphs whatever `sides` says.
- The renderer lays them as `skein/docs/design/ui-screen.md` says: 3 (the theme), 5.2
  (*A framed region*: the title's end part, left out whole first where the border lacks
  room for both; *A region framed by rules*) and 5.3 (*Aligned to the end*, *A
  band*). With `sides: true`, `align: Start` and `band: false` everywhere, every
  existing golden is byte-for-byte as it is.
- Goldens, new, in `src/tests/golden/`, written by hand from the design and checked
  cell by cell (print, `--`, paint letters, cursor):
  - `looks.txt`, 80 by 24: a fixture tree whose top region is boxed with a title made of
    two runs in different paints (a level-1 heading with `Text` children carrying two
    tokens) and an end part (a child carrying a token whose look aligns `End`); a body
    paragraph aligned to its end; a banded paragraph that is indented under a list item
    and wraps to two rows; a bottom region framed by rules (`sides: false`) holding a
    focused text field with a placeholder, and under it a spread bottom region.
  - `looks-narrow.txt`, 30 by 12, the same tree: the end part left out, and the title
    cut at its end.
  Each is a row of `golden_frames_lay_bands_text_fields_scroll_and_roles`
  (`src/tests/render.rs`), its fixture `fixtures::looks()` beside the others in
  `src/tests/fixtures.rs`, its regions and tokens added to `fixtures::theme()` without
  touching what the existing fixtures use.
- Rows, each a failing test first, in `rendering_honours_focus_paints_scroll_and_frame_bounds`
  or a table beside it in `src/tests/render.rs`: an end-aligned line exactly as wide as
  its room is laid from its room's start; an end-aligned node nested under an aligned
  `Start` token's node is aligned (the nearest token decides); a band under a region's
  paint takes the band's paint on its padded rows' inner cells and not on the border; a
  border with room for the title and not for the end part shows the title alone, whole. `src/tests.rs`: `Theme::valid` refuses a control glyph in a
  border with `sides: false`.
- `skein/docs/design/ui-screen.status.md`: delete the first **Not yet** entry ("3 and
  5.2: a border with no sides, ... a line aligned to its end and a band.") and nothing
  else; `skein/docs/design/STATUS.md`'s summary sentence then drops "Regions framed by
  rules, a title's end, lines aligned to their end, bands," from what is not built.
  These two status edits are a stated departure from the implementer's text.

## Inputs

- `skein/docs/design/ui-screen.md`: 3, 5.2, 5.3 and 9 (the goldens' list).
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1.
- The frames this serves, for the look only: `temper/docs/plan/tui-launchpad/inputs/launchpad-01.txt`
  and `-03.txt` (the header's border, the rules around `❯ Ask temper`, the band of
  `❯ What landed overnight?`).

## Owns and may change

- Owns `skein-ui-screen`'s theme, renderer and its tests for this wave. `Focus` and the
  options menu are the task `screen-lists`'s next: leave `focus.rs` as it is.
- **Readers outside skein:** `temper/crates/temper-client-tui/src/theme.rs` and
  `src/tests/theme.rs` construct `Border` and `StyleLook`. Add the new fields there with
  the values that draw as before (`sides: true`, `align: Align::Start`, `band: false`),
  each line edited only for that (the branch's one cross-tree edit, as a required
  field's), and run `temper-client-tui`'s goldens unchanged. No other crate constructs
  them (`rg 'StyleLook \{|Border \{'`).
- `StyleLook` and `Border` keep their derives; `Align` derives `Clone, Copy, PartialEq,
  Eq, Debug`.
- If the title's runs in different paints already lay as the design says (the survey
  found the left title keeps its children's paints), keep that code; if a run's paint is
  lost, fix it here and say so.
- A paint on `Run::Strong` or `Run::Emphasis` is not this task's.

## Steps

1. The fixture, the goldens by hand and the rows; `heavy cargo nextest run -p
   skein-ui-screen --lib`; see the new ones fail for their reasons.
2. `theme.rs`, then the border (`render/canvas.rs`'s region entry, rows and title), the
   title's end part (`render.rs`'s title capture), the end alignment and the band (the
   canvas's line commit).
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-ui-screen -p skein-ui-screen-terminal -p temper-client-tui --all-targets -- -D
   warnings`; `heavy cargo nextest run -p skein-ui-screen -p skein-ui-screen-terminal -p
   skein-ui-screen-tests -p temper-client-tui`; `heavy cargo check -p
   temper-client-shell-tui -p temper-client-journeys --tests`.
4. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen, and the goldens' rows checked
against the design.
