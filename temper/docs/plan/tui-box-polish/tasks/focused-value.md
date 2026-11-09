+++
id = "focused-value"
part = "box-ink"
kind = "interface"
needs = []
files = ["skein/crates/skein-ui-screen/src/theme.rs", "skein/crates/skein-ui-screen/src/render.rs", "skein/crates/skein-ui-screen/src/render/**", "skein/crates/skein-ui-screen/src/tests.rs", "skein/crates/skein-ui-screen/src/tests/**", "skein/docs/design/ui-screen.status.md"]
effort = "high"
minutes = 45
+++

## Goal

skein's full-screen renderer lays a focused text field's or text area's
value and caret in its token's focused value's paint, where the token's
look names one, and keeps the focus's paint on the focus mark (and a
label) alone. The TUI's box needs it so that the accent lies on "❯"
alone and the person's words are the terminal's own ink (plan
`tui-box-polish`, the user's request of 2026-11-09). This task builds the
generic mechanism only; `box-ink` turns it on for the TUI.

Acceptance:

- `StyleLook` has `pub focused_value: Option<Paint>`, after
  `active_name`, as `temper/docs/plan/tui-box-polish/notes/contracts.md`
  has it; `None` draws every frame exactly as today.
- With `Some(paint)` on a token a focused text field or text area
  carries: its value's cells and its caret's cell are `paint` over the
  node's own composed paint (its tokens' over its region's), with no
  focus paint; the placeholder is the placeholder's paint over that; the
  focus mark, and a label where the field lays one, keep the focus's
  paint. The last of the node's tokens naming one wins.
- Unfocused fields, options menus' active items (`active_name` and the
  band), and every other node are unchanged.
- A render test pins it, and bites (break the rule by hand once, see it
  fail, restore).
- `skein/docs/design/ui-screen.status.md`: the plan's **Not yet** entry
  gone, "No known implementation gap." back at the head of the next
  paragraph, and "focused-value paints" added to its list beside
  "opt-in active-name paints".

## Inputs

- `skein/docs/design/ui-screen.md`, section 3 (*A style token's look*,
  `focused_value`), 5.4 (*A text field*) and 9 (*Golden frames*).
- `temper/docs/plan/tui-box-polish/notes/contracts.md`, *skein-ui-screen*.
- `temper/docs/plan/tui-box-polish/notes/common.md`.

## Owns and may change

- Owns: `skein/crates/skein-ui-screen/src/theme.rs` (the field),
  `src/render.rs` and `src/render/**` (`control`, `Canvas::field` and
  `caret`, wherever the focused value's cells take their paint),
  `src/tests.rs`, `src/tests/**` (the fixtures' `StyleLook` literals take
  `focused_value: None`; the new test), and
  `skein/docs/design/ui-screen.status.md`.
- May change, the new field's literal alone (a required field, carried
  across trees): `temper/crates/temper-client-tui/src/theme.rs`'s
  `style()` takes `focused_value: None,`, and
  `temper/crates/temper-client-tui/src/tests/theme.rs`'s `want` literal
  (about line 223) takes `focused_value: None` in its field list. Nothing
  else in temper.
- If `Theme::valid` or a skein-ui-screen doc comment names every
  `StyleLook` field, add the new one there too.

## Steps

1. Add the field and its doc comment. Find where `control` passes the
   focus-composed `paint` to `Canvas::field` (and from there to
   `field_bytes` and `caret`): compute, for a text field or text area
   that is focused and whose look names a focused value's paint, the
   value's paint as that paint over the node's own paint
   (`info.paint`), and pass it for the value and the caret; keep the
   focus-composed paint for `out.control` (the mark) and for the label
   (`content`, `: `, `words::entry`). Read the token's look as
   `active_name_paint` reads `active_name` (`render.rs`, about line 840):
   the node's styles, the last naming one winning.
2. Test, in `src/tests/render.rs` beside
   `active_name_paint_replaces_tokens_only_for_the_active_name`:
   `focused_value_paint_leaves_the_focus_paint_on_the_mark_alone`, a
   table over `None`, `Some(Paint::NONE)` and a coloured paint, for a
   focused text field with a value and caret, an empty one with a
   placeholder, and a text area: the mark's cell, the value's cells,
   the caret's cell, the placeholder's first cell. Use the fixtures'
   theme with `theme.focus` set to an accent, as that test does.
3. Narrowest runs: `heavy cargo nextest run -p skein-ui-screen --lib -E
   'test(=tests::render::focused_value_paint_leaves_the_focus_paint_on_the_mark_alone)'`
   (adjust the path to the test's module), then the package's focused
   tests; `heavy cargo check -p temper-client-tui --tests`.
4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-ui-screen -p temper-client-tui --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-ui-screen -p temper-client-tui`
   (the TUI's goldens must be unchanged: `None` changes nothing).
5. Commit, push.

## Report

As the implementer's text asks; name the functions whose paint changed
and the bite: the rule broken and the failure seen.
