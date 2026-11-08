+++
id = "left-off"
part = "left-off"
kind = "feature"
needs = ["look"]
files = ["temper/crates/temper-client-tui/src/view/launchpad.rs", "temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/tests/launchpad.rs", "temper/crates/temper-client-tui/src/tests/golden/launchpad-*.txt"]
effort = "high"
minutes = 60
+++

## Goal

The user's request item 1, the view's half: "Where you left off" lists
up to three live chats, and while the launchpad's box is empty Up and
Down move a selection through them, Enter opens the selected one and
Esc clears it. The selection is the TUI's state (`Told::LeftOff`, task
`look`); this task draws it and makes the events. Task `arrows` routes
the keys in the host.

Acceptance (`temper/docs/design/client/README.md`, 6.14, *Where you
left off*):

- The launchpad reads, of Home's live chats in its project
  (`Chats::live`), those that have not ended, newest `last_activity`
  first, the lower task first between two of one time, at most three,
  each its task, title and `shared::chat_state`. They are drawn under
  the head "Where you left off" as one list under `look::LEFT_OFF`, an
  unnumbered item a chat (title, then " · " and its state in
  `look::HINT`), so that with none selected the frame's text is as
  today's for one chat: `   Away mode · waiting on you` under
  ` Where you left off`.
- The item whose task is `tui.left_off()` also carries `look::PICKED`:
  its row shows `❯` at the mark's column and its title in the accent,
  across the band where the theme has one. A selected task not among
  the lines draws no selection.
- `arrow(view, Arrow::Down)` and `arrow(view, Arrow::Up)`, on the tree
  last built, while the screen is the launchpad with at least one
  left-off chat, the box's value is empty and no list stands under the
  box: with none selected (or one not among the lines), either gives
  `Event::LeftOff { task: Some(NEWEST) }`; with the line at index `i`
  selected, Down gives the line at `i + 1`, Up the line at `i - 1`, each
  staying at the last and the first (the same task again). Otherwise
  `None`, so that the key reader has the key.
- While a chat is selected and the box is empty, the box's Enter
  (`event(view, node, Act::Submit)` on the box) gives `Event::Act { act:
  Act::Go { address: Address::Chat { task } } }`; otherwise `Send`, as
  today.
- `escape(view)` gives `Event::LeftOff { task: None }` while a chat is
  selected and no list stands; a list's own escape event while one
  stands, as today.
- Rows in `tests/launchpad.rs`, from a root at Home with four live
  chats (one ended, so three are drawn) built in that file: the three
  lines in order; Down, Down, Down, Up from none (each step given to the
  `Tui` with `Told::LeftOff` and the view rebuilt) giving the newest,
  the second, the third, the third, the second; Up from none giving the
  newest; `None` with words in Home's composer and with the commands'
  list standing; Enter's `Act::Go` and `Send`; Esc's `LeftOff { task:
  None }`; the selected row's `❯` cell and accent title paint, its band
  with `theme(Scheme::Dark, Some(bg))` and none with `None`.
- The launchpad goldens still pass, or are regenerated where the list's
  laying changes a cell; the text of one chat's line is unchanged.

## Inputs

- `temper/docs/design/client/README.md`, 6.14 (*Where you left off*,
  *The box keeps the keys*, *The box*'s *The focus* and *Enter*), 6.13
  (the keys' paragraph, the events table's `Act` and `LeftOff` rows) and
  6.15 (the tokens `tui-left-off` and `tui-picked`, *The band*).
- `skein/docs/design/ui-screen.md`, 3 (*a mark*, *a band*) and 5.3 (*A
  mark*, *A band*).
- `temper/docs/plan/tui-launchpad-2/notes/common.md` and
  `notes/contracts.md`, section 1.

## Owns and may change

- Owns the files in `files`. In `view.rs`: the `View`'s fields for what
  the arrows, Enter and Esc read, `arrow`'s body, the box's `Submit` in
  `event`, and the escape set after `foot::draw`; not `offered`.
- Task `chat-rule` changes `temper-client-tui` at the same time, in
  `view/chat.rs`, `view/foot.rs`, `look.rs`, `theme.rs`, `words.rs`,
  `tests/chat.rs`, `tests/foot.rs`, `tests/theme.rs` and the chat and
  foot goldens: none of those is yours. Read `foot::Foot`
  (`composer.value`, `list`) from `view.rs`; never change it. Build
  your test fixtures in `tests/launchpad.rs`; don't edit
  `tests/fixtures.rs` or `tests/goldens.rs`.
- Pre-approved: if `state.rs` lacks something the contract promises,
  that is a `Question:`; if a host test in `temper-client-shell-tui`
  fails because the box's Enter or Esc now makes another event with a
  chat selected, it is task `arrows`' (report it under `## Left`).

## Steps

1. `launchpad.rs`: up to three `LeftOff { task, title, state }`, newest
   first; the list under `LEFT_OFF`, the selected item also under
   `PICKED`.
2. `view.rs`: record at each build whether the arrows apply, the tasks
   drawn and the one selected; `arrow`; Enter's `Act::Go`; Esc.
3. Tests as above; break each rule once (the order's tie, the
   three-line bound, the edge's staying, Enter's gate on an empty box),
   see its row fail, restore.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-client-tui -p temper-client-shell-tui -p
   temper-client-journeys` (no journey's frame should change: a fresh
   box has no left-off chat).
5. Commit and push.

## Report

As the implementer's text says.
