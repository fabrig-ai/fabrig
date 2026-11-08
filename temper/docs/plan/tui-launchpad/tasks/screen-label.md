+++
id = "screen-label"
part = "screen-label"
kind = "feature"
needs = ["screen-lists"]
files = ["skein/crates/skein-ui-screen/src/**", "skein/docs/design/ui-screen.status.md"]
effort = "medium"
minutes = 60
+++

# tui-launchpad: skein's screen lays a field with no label where its token says so

## Goal

A text field or text area whose token's look says `label: false` is laid as its focus
columns, then its value or placeholder alone, with no `NAME: `, its name still the
tree's, so that a chat terminal's box reads `❯ Ask temper` (the focus mark, then the
placeholder) as the request's frames draw it.

Acceptance:

- `skein_ui_screen::StyleLook` gains `label: bool`, as
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1 (*Added after the first
  release*) spells it; the nearest token whose look names it decides, as for `align` and
  `band`.
- The renderer lays it as `skein/docs/design/ui-screen.md`, 3 (*a label*) and 5.4 (*A
  text field*) say: with `label: false`, a text field's row is its focus columns, then
  its value from the offset that keeps the caret in view, or its placeholder in the
  placeholder's paint; a text area's rows likewise, its value wrapped at the room the
  missing name leaves; the cursor at the caret. A secret is masked as before. With
  `label: true` everywhere, every existing golden is byte-for-byte as it is.
- Rows, each a failing test first, in `src/tests/render.rs` beside the field rows: a
  focused field with no label shows the focus mark then its value and the cursor two
  columns after the mark; empty, its placeholder in the placeholder's paint; a text area
  with no label wraps at the frame's width less its focus columns; a secret with no
  label masked; a field with `label: false` under a token with `label: true` nearer is
  labelled.
- `fixtures::looks()`'s bottom field (from `screen-looks`), if it has one, keeps its
  label: this task adds its own fixture field.
- `skein/docs/design/ui-screen.status.md`: delete the **Not yet** entry "3 and 5.4:
  `label` on a token's look, a text field or area laid with no name." and nothing else (a
  stated departure from the implementer's text).

## Inputs

- `skein/docs/design/ui-screen.md`: 3 and 5.4.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1.
- The frames, for the look only: `temper/docs/plan/tui-launchpad/inputs/launchpad-01.txt`
  (`❯ Ask temper`), `-02.txt` (`❯ /`), `-03.txt` (`❯ Reply`).

## Owns and may change

- Owns `skein-ui-screen` for this wave.
- **Readers outside skein:** `temper/crates/temper-client-tui/src/theme.rs` and
  `src/tests/theme.rs` construct `StyleLook`: add `label: true` to each, the line edited
  only for that, and run that crate's goldens unchanged. If the task `tui-surface` has
  landed and its theme already names `label` (it may, as `label: false` on its box's
  token, once this lands), leave its value as it is.
- The text shell names every field as it does: unchanged.

## Steps

1. The rows; `heavy cargo nextest run -p skein-ui-screen --lib`; see them fail.
2. `StyleLook::label`; the field's and area's layout in the renderer.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-ui-screen -p temper-client-tui --all-targets -- -D warnings`; `heavy cargo
   nextest run -p skein-ui-screen -p skein-ui-screen-terminal -p temper-client-tui`.
4. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen.
