+++
id = "screen-entries"
part = "screen-entries"
kind = "feature"
needs = ["screen-label"]
files = ["skein/crates/skein-ui-screen/src/**", "skein/docs/design/ui-screen.status.md"]
effort = "high"
minutes = 120
+++

# tui-launchpad: skein's screen lays an item's mark, a description's own paint and spaced children

## Goal

Three looks a style token gives, so that a chat terminal's conversation reads as the
request's frames draw it: an unnumbered item laid with its token's mark glyph in its own
paint in place of `- ` (`✦ Three changes landed…`, `❯ What landed overnight?`, `▸ Listed
changes…`), its later lines and blocks hanging under it; a node's description laid in a
paint of its own (an options menu's `/model` bold, `Change the model, from the next
turn` dim); and one blank row between each two children of a node whose token spaces
them (the conversation's entries, the launchpad's sections, an answer's blocks).

Acceptance:

- `skein_ui_screen::StyleLook` gains `mark`, `described` and `spaced`, and
  `skein_ui_screen::Mark` exists, exactly as
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1 (*Added with the third
  batch*) spells them; `Theme::valid` refuses a mark's glyph that is a control character
  or not one cell wide.
- The renderer lays them as `skein/docs/design/ui-screen.md`, 3 and 5.3 (*A mark*, *A
  description's paint*, *Spaced*) say. With `mark: None`, `described: None` and `spaced:
  false` everywhere, every existing golden is byte-for-byte as it is.
- A golden, new, `entries.txt`, 60 by 20, written by hand from the design and checked
  cell by cell, a row of `golden_frames_lay_bands_text_fields_scroll_and_roles`, from a
  fixture `fixtures::entries()`: a log whose token is `spaced`, holding an unordered list
  of three items, each under a token with its own mark (one glyph in an accent paint
  over plain words; one dim glyph on a banded item; one dim glyph on a dim item), the
  first item holding a paragraph, a numbered list of two items that wraps and a second
  paragraph, so that its later blocks hang two columns in and the numbered items keep
  their numbers; and beneath, an options menu after a focused field whose items' token
  is bold and gives a dim `described` paint, so that each name is bold and its words
  dim.
- Rows, each a failing test first, in `src/tests/render.rs`: a spaced node's first and
  last child have no gap before or after them; a spaced node's grandchildren are not
  spaced; a child laid as no row takes no gap; a mark on a numbered item is ignored; the
  nearest token's mark wins; a description under a `described` paint takes it over the
  region's and none of its tokens' attributes. `src/tests.rs`: `Theme::valid` refuses a
  control character as a mark's glyph and a glyph two cells wide.
- `skein/docs/design/ui-screen.status.md`: delete the **Not yet** entry "3 and 5.3:
  `mark`, `described` and `spaced` on a token's look: ..." and nothing else (a stated
  departure from the implementer's text).

## Inputs

- `skein/docs/design/ui-screen.md`: 3 and 5.3.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1.
- The frames, for the look only: `temper/docs/plan/tui-launchpad/inputs/launchpad-01.txt`
  (the sections a blank row apart), `-02.txt` (the commands' names and words), `-03.txt`
  and `-04.txt` (the conversation's marks and hanging blocks); their pages' paints by
  `git show 9b3942a123:temper/docs/design/client/tui/journeys/launchpad/03.html`.

## Owns and may change

- Owns `skein-ui-screen` for this wave.
- **Readers outside skein:** `temper/crates/temper-client-tui/src/theme.rs` and
  `src/tests/theme.rs` construct `StyleLook`: add `mark: None, described: None, spaced:
  false` to each literal, each line edited only for that, and run that crate's goldens
  unchanged. If the task `tui-surface` has landed first and its theme already names a
  field, leave its value as it is.
- If a description is laid by `descriptions` for roles beyond an options menu's item,
  `described` applies to all of them, as the design says: it is a token's look.

## Steps

1. The fixture, the golden by hand and the rows; `heavy cargo nextest run -p
   skein-ui-screen --lib`; see them fail for their reasons.
2. `Mark` and the three fields; the item's mark where the renderer writes `- `; the
   description's paint in `descriptions`; the gap between a spaced node's children.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-ui-screen -p temper-client-tui --all-targets -- -D warnings`; `heavy cargo
   nextest run -p skein-ui-screen -p skein-ui-screen-terminal -p temper-client-tui`.
4. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen, and the golden checked against
the design.
