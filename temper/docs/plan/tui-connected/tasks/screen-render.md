+++
id = "screen-render"
part = "screen"
kind = "feature"
needs = ["screen-surface"]
files = [
  "skein/crates/skein-ui-screen/**",
]
effort = "high"
minutes = 150
+++

# skein-ui-screen: the renderer

## Goal

`skein_ui_screen::render` lays skein-ui's tree out on a frame: the bands
the theme places, framed regions with their titles, a spread row, the body
scrolled to keep the focus in view, wrapping at each region's width, the
focus mark and the focus's paint, fields with their caret, masks and
placeholders, and the roles an application's first screens use. Every
other role is `screen-roles`'s; here it lays its children as a group does.

`skein/docs/design/ui-screen.status.md` is not yours: tasks beside this
one run in parallel, and `screen-status` closes its entries once all have
merged.

Acceptance:

1. `skein_ui_screen::render(tree, theme, size, focus, frame)` exists with
   the signature of `temper/docs/plan/tui-connected/notes/contracts.md`,
   section 2, exported from `lib.rs`, doing what
   `skein/docs/design/ui-screen.md`, sections 5.1 to 5.4, say: the text
   (width, control characters, wrapping), the bands (their order, their
   room, framed regions and their titles, the body and its scroll), the
   roles (5.3, for the roles of acceptance 2), the spread region, fields
   and the focus (requests honoured, a gone node dropped, the caret, the
   cursor).
2. **The roles laid as 5.3 says:** page; banner, navigation, main,
   complementary, content info, form, search, region (named: `NAME:` and
   its children indented; unnamed: its children); group and slot; heading;
   paragraph, text and time; status; alert; separator; list and item (with
   an order's numbers and positions); button and link (an external link as
   `TEXT <URL>`); text field and text area. Every other role lays its
   children as a group does, its arm naming each role (no `_`).
3. **Golden frames** in `src/tests/golden/`, each a `Frame::print`,
   written by hand from the design and compared byte for byte by a test in
   `src/tests/render.rs` (or `src/tests.rs`'s module of that name), all
   with one fixture theme of `src/tests/fixtures.rs` (a top region framed
   with a title, a bottom region spread, two style tokens with paints):
   - `bands.txt`: a top region with a level-1 heading and two paragraphs,
     a body of a paragraph, a list of three items and two buttons the
     second focused, a bottom region spread with a button and a status, at
     40 by 12;
   - `bands-small.txt`: the same tree at 20 by 5, cut as 5.2 says;
   - `text.txt`: a paragraph wrapping at a space and at the width, a wide
     glyph that would cross the row's end, a control character, a tab and
     bytes that are not UTF-8, at 24 by 8;
   - `fields.txt`: a text field focused with its caret mid-value, a secret
     with a value, a text field whose value passes its room scrolled to its
     caret, an empty field with a placeholder, a text area wrapping, at 32
     by 10;
   - `scroll.txt`: a body of twenty paragraphs and a button after them
     focused, at 30 by 8, the body scrolled the least that shows it;
   - `roles.txt`: one node of each role of acceptance 2, with states and
     a description, at 80 by 24.
4. **Rows** in a table test beside them: a focus request newer than the
   last honoured moves the focus to its node, caret at the value's end; an
   older one does nothing; a focus on a node gone is none after `render`;
   a token's paint lays over its region's, the nearest token last; the
   frame's scroll is `focus.scroll` as moved; `render` at a size larger
   than `frame.most()` lays `most` alone.
5. `render`'s working storage is bounded by the tree's nodes and the
   frame's cells: it makes no list whose capacity grows with the text.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, section 2: binds.
- `skein/docs/design/ui-screen.md`, sections 3, 4 and 5.
- `skein/docs/design/ui-text.md`, section 5: the roles' words, marks,
  states and properties this renderer lays in cells, and
  `skein/crates/skein-ui-text/src/render.rs`, whose walk (`render`, `node`,
  `content`, `safe`) is the one to follow: an ancestor stack of ends, a
  role either descending or consuming its subtree.
- `skein/docs/design/ui.md`, 3.1 and 3.2 (roles, states, properties), 3.2's
  focus request, and section 8 (regions and tokens).

## Owns and may change

**Owned:** the `files` above: `skein-ui-screen`'s `render` module (and its
private helpers), its exports, its tests and goldens.

**Readers and neighbours:** none outside the crate. The surface types of
`screen-surface` stay as they are; a helper they need (a `Frame` method to
fill a run of cells, say) may be added as `pub(crate)`, never public.

**Options, each with its default:**

- Two passes or one: measuring the body's rows up to the focused node
  before laying them is the default; a single pass that lays into the
  frame and shifts is yours to choose if it keeps the bound of
  acceptance 5.
- Words shared with the text shell (its states' words, its `NAME:`
  rules): copy the few that apply into this crate rather than depend on
  `skein-ui-text`, whose crate graph would then pull in io. If a whole
  table would be copied, keep it in one module with a comment citing
  ui-text.md, section 5.
- A frame of fewer columns than a framed region's border and padding
  need: the border alone, cut, and no content; say what you chose in a
  comment and the golden.

## Steps

1. The walk: bands first (find each placed region by its `Region` in the
   theme's looks, in preorder), then each band's lines into the frame.
2. The text: `width`, the replacements, wrapping, paints laid over in
   order.
3. The roles of acceptance 2, the spread region, fields, the focus and
   its requests, the cursor, the scroll.
4. The fixtures, goldens and rows (acceptance 3 and 4). Bite: lay the
   bottom band after the top once, and break the scroll's least move once;
   see `bands-small.txt` and `scroll.txt` fail, and restore.
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-ui-screen --all-targets -- -D warnings`; `heavy cargo
   nextest run -p skein-ui-screen --lib`. Commit ("skein-ui-screen: the
   renderer, regions placed by name and the body kept on the focus"),
   push.

## Report

As the implementer's text says, and: one pass or two, what a too-narrow
framed region draws, and the goldens' time.
