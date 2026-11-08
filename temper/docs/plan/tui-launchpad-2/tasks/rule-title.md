+++
id = "rule-title"
part = "rule-title"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui-screen/src/render/canvas.rs", "skein/crates/skein-ui-screen/src/render.rs", "skein/crates/skein-ui-screen/src/tests/render.rs", "skein/crates/skein-ui-screen/src/tests/fixtures.rs"]
effort = "high"
minutes = 40
+++

## Goal

skein's full-screen renderer lays a border's title the way a rule above
a TUI's box needs it, so that temper can put a chat's title in that rule,
aligned right (the look request `tui-chat-title-below`): a title whose
heading's children all align `End` lays nothing at the border's start
but its glyphs across, and in a rule (a border with no sides) the end's
part stands before two glyphs across, where a box has one and its last
corner.

Acceptance:

- A region framed by rules whose border draws its title, with a level-1
  heading whose only children align `End` ("Hello / Ask T2 on box"), at
  40 columns, lays its top rule as `─────────────── Hello / Ask T2 on box ──` (40 cells):
  the text's last cell four columns before the rule's end, then one
  space and two `─` (`skein/docs/design/ui-screen.md`, 5.2).
- The same heading in a box with sides, 40 columns, lays
  `╭────────────── Hello / Ask T2 on box ─╮`: its end part as today's, its start with no
  spaces (`╭───…`), not `╭─  ───…`.
- A heading with a start part and an end part in a rule: the start after
  one glyph across and a space, as today; the end before a space and two
  glyphs across.
- Too narrow: the end part's room is its width and six columns within
  the border's inner span, as today's rule for a start and an end; where
  it lacks it, the end part is left out whole and the rule (or the box's
  top border) is glyphs across alone, with no stray spaces.
- Every existing row of `rendering_lays_end_alignment_bands_and_title_parts`
  and `border_titles_keep_their_heading_paint` passes unchanged: a box's
  titles, and a rule's start-only title, are laid as before.
- Commits carry the trailer `Look-request: tui-chat-title-below`
  (`temper/docs/plan/tui-launchpad-2/notes/common.md`).

## Inputs

- `skein/docs/design/ui-screen.md`, 5.2 (*A framed region*, *A region
  framed by rules*, *A title with no start*) and 3 (*its border*,
  *its alignment*).
- `skein/docs/design/ui-screen.status.md`: the **Not yet** entry this
  task closes (task `journeys` removes it; you don't edit design).
- `temper/docs/plan/tui-launchpad-2/notes/common.md` and
  `notes/contracts.md`, section 4.
- Today's code: `Canvas::title` in
  `skein/crates/skein-ui-screen/src/render/canvas.rs` (the start part's
  two spaces are laid even when the start is empty; the end's `limit` is
  the region's right edge less one side, so a rule keeps one glyph across
  after its end part).

## Owns and may change

- Owns the files in `files`. Changes stay inside `Canvas::title` and
  whatever `render.rs` passes it; no public type or signature changes.
- `skein/crates/skein-ui-screen/src/tests/fixtures.rs`: add a fixture (a
  titled rule region in the fixture theme, or a function building the
  trees below) beside `end_title()`; existing fixtures keep their output.
- If another skein crate's test (`skein-ui-screen-terminal`) prints a
  titled rule and changes, update its expected text and say so in the
  report; a temper test that changes is not yours: none should, since
  temper's only rule (`tui-box`) draws no title until task `chat-rule`.

## Steps

1. Read `Canvas::title` and how `render.rs` gathers the heading's start
   and end canvases for a titled region.
2. Lay nothing at the start when the start part is empty (no leading
   spaces); keep the end part's right edge at the region's right edge
   less one column in a rule as in a box, so that two glyphs across
   follow it in a rule, and a glyph across and the corner in a box.
3. Add rows to `rendering_lays_end_alignment_bands_and_title_parts` (or
   a sibling table if that one's row shape doesn't fit), each a tree, a
   size, the expected text and a cell or two:
   - "rule title at the end": the 40-column rule above;
   - "box title at the end only": the box above;
   - "rule title start and end": both parts in a rule;
   - "rule title too narrow": the end part left out, the rule bare.
   Break the rule once by hand for the first two (restore the spaces,
   restore the old limit), see each row fail, restore.
4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-ui-screen --all-targets -- -D warnings`; `heavy cargo
   nextest run -p skein-ui-screen`; then its direct dependents' focused
   tests: `heavy cargo nextest run -p skein-ui-screen-terminal -p
   temper-client-tui`.
5. Commit (with the trailer) and push.

## Report

As the implementer's text says; name each row added and the break you
made for it.
