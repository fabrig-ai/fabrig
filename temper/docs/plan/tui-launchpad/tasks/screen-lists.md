+++
id = "screen-lists"
part = "screen-lists"
kind = "feature"
needs = ["screen-looks"]
files = ["skein/crates/skein-ui-screen/src/**", "skein/crates/skein-ui-screen-terminal/src/command.rs", "skein/crates/skein-ui-screen-terminal/src/tests/command.rs", "skein/docs/design/ui-screen.status.md", "skein/docs/design/STATUS.md"]
effort = "high"
minutes = 210
+++

# tui-launchpad: skein's screen lays an options menu from a field and follows a log's end

## Goal

skein's full-screen shell gains what a chat terminal's foot and conversation need, as
skein-ui already says them (`skein/docs/design/ui.md`, 3.2, *options* and *an end
request*): an options menu after a focused field laid one row an item, its names in a
column and its active item shown by the focus mark over a band, moved by Up and Down,
pressed by Enter and completed by Tab while the field keeps the focus; and a body that
follows a log's end as it grows until the person pages up, paging by the body's rows.

Acceptance:

- `skein_ui_screen::Theme` gains `active: Paint`; `skein_ui_screen::Focus` gains
  `active`, `follow`, `ended` and `rows`, exactly as
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1, spells them, `Focus`
  still deriving `Default` (none of them set).
- The renderer, as `skein/docs/design/ui-screen.md` says: 3 (*the active paint*), 5.2
  (*The body* says `Focus::rows`; *The body follows a log's end*), 5.3 (*an options
  menu*) and 5.4 (*The focus*, *The active item*).
- The key reader, `skein_ui_screen_terminal::command`, as 6.2 says: `PageUp` and
  `PageDown` by `Focus::rows` less one, at least one, `PageUp` clearing `follow`; `Up`,
  `Down`, `Enter` and `Tab` on a text field or area an options menu follows, while the
  menu has an enabled item; Escape there as anywhere.
- Goldens, new, in `skein-ui-screen`'s `src/tests/golden/`, written by hand from the
  design, each a row of `golden_frames_lay_bands_text_fields_scroll_and_roles`:
  - `options.txt`, 60 by 14: a form holding a text field with the value `/` and the focus,
    then a menu with `State::Options` of five bound items, each a name and a
    description, the third disabled, the second marked selected, so that it is active:
    the names' column, the band of `active` across the menu's inner width under the
    focus's paint and mark, the disabled item's `(disabled)` as 5.3 has it, placed in
    a bottom region, with a body above.
  - `follow.txt`, 40 by 12: a log of thirty one-line entries with an end request of 1,
    over a bottom region holding a focused field: the log's last rows shown.
- Rows, each a failing test first:
  - `skein-ui-screen`'s `src/tests/render.rs`, beside the scroll rows: a log whose end
    request is newer than `Focus::ended` sets `follow` and `ended` and shows the end;
    the log grown by five entries the next frame still shows its end; `follow` cleared
    and the log grown keeps the scroll; a log that continues (`State::Continues`) is not
    followed; a focused node in the body out of view wins over `follow`; a frame whose
    scroll is the body's last row of a body taller than its room sets `follow`;
    `Focus::rows` is the body's rows laid; the active item kept on an enabled item when
    the one it named leaves the tree or is disabled, and none once the field loses the
    focus.
  - `skein-ui-screen-terminal`'s `src/tests/command.rs`: `PageUp` and `PageDown` with
    `rows` 10 move by 9, with `rows` 0 and 1 by 1, `PageUp` clearing `follow`; on a
    field an options menu follows, `Down` and `Up` move `active` among the enabled
    items and stay at both ends, `Enter` is `Act::Press` on the active item, `Tab` an
    `Input` of the active item's name where the value begins it and a focus move
    otherwise; a menu with no enabled item leaves every key as without it.
- `skein/docs/design/ui-screen.status.md`: delete the remaining **Not yet** entry ("3,
  5.3 and 5.4: the theme's active paint; ...") so the file lists no gap of sections 3 to
  6, and `skein/docs/design/STATUS.md`'s summary sentence drops what this task builds
  (options menus from a field, a body that follows a log's end, paging by the body's
  rows). These two status edits are a stated departure from the implementer's text.

## Inputs

- `skein/docs/design/ui.md`: 3.2 (*options*, *continues*, *an end request*), 6.1
  (Escape).
- `skein/docs/design/ui-screen.md`: 3, 5.2, 5.3, 5.4, 6.2 and 9.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 1.
- The frames this serves, for the look only: `temper/docs/plan/tui-launchpad/inputs/launchpad-02.txt`
  (the commands under the box) and `-04.txt` (a numbered choice active).

## Owns and may change

- Owns `skein-ui-screen` (all of it, after `screen-looks`) and `skein-ui-screen-terminal`'s
  key reader and its test for this wave. The terminal (`terminal.rs`) and the drawing
  stay as they are.
- **Readers outside skein:** `temper/crates/temper-client-tui/src/theme.rs` constructs
  `Theme`: add `active: Paint::NONE` there, the line edited only for that, and run its
  goldens unchanged. `Focus` is built with `..Focus::default()` or `Focus::default()`
  outside skein (`rg 'Focus \{'`; `temper-client-shell-tui`'s host): if a literal names
  every field, add the four with their defaults and say so.
- The text shell (`skein-ui-text`) prints an options menu as any menu: unchanged.
- If an options menu's item has no description, its row is its name alone, unpadded.
- If an item's description is a property rather than a child node in skein-ui's tree,
  lay its first line in the theme's base under the item's tokens and say so; the TUI
  paints it through the item's tokens either way.
- The key reader learns the menu from the tree (the menu after the focused field in its
  innermost form, `State::Options`, as `skein-ui`'s outline checks it); if `skein-ui`
  has a helper for "the options menu of this field", use it, and do not add one to
  `skein-ui` without asking.

## Steps

1. The fixtures, the goldens by hand and the rows; `heavy cargo nextest run -p
   skein-ui-screen -p skein-ui-screen-terminal --lib`; see them fail for their reasons.
2. `Theme::active` and `Focus`'s fields; the renderer's body (`Focus::rows`, the log's
   end request, `follow`), the options menu's rows and the active item; the key reader.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-ui-screen -p skein-ui-screen-terminal -p temper-client-tui --all-targets -- -D
   warnings`; `heavy cargo nextest run -p skein-ui-screen -p skein-ui-screen-terminal -p
   skein-ui-screen-tests -p temper-client-tui -p temper-client-shell-tui`; `heavy cargo
   check -p temper-client-journeys -p temper-client-world --tests`.
4. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen, and the goldens checked
against the design.
