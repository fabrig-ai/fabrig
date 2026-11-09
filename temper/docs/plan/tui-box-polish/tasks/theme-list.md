+++
id = "theme-list"
part = "theme"
kind = "feature"
needs = ["theme-start", "notice", "quit"]
files = ["temper/crates/temper-client-tui/src/event.rs", "temper/crates/temper-client-tui/src/state.rs", "temper/crates/temper-client-tui/src/words.rs", "temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/view/foot/lists.rs", "temper/crates/temper-client-tui/src/view/foot.rs", "temper/crates/temper-client-tui/src/tests/state.rs", "temper/crates/temper-client-tui/src/tests/commands.rs", "temper/crates/temper-client-tui/src/tests/foot.rs", "temper/crates/temper-client-tui/src/tests/limits.rs", "temper/crates/temper-client-tui/src/tests/golden/foot-commands-80x24-dark.txt", "temper/crates/temper-client-tui/src/tests/golden/foot-theme-*.txt", "temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/docs/design/client/tui/journeys/**", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 90
+++

## Goal

`/theme` in the TUI opens a command's list, as `/model`'s does: its head
"Theme. Auto follows your terminal.", then "Auto" with what it reads
now, "Light" and "Dark", the current choice active. Taking one draws in
it at once, keeps it in the small store for the next run, and says so in
the notice: "Theme: auto, light as your terminal says." `/theme`
becomes the TUI's own command, its words "Auto, light or dark"; the
browser's terminal's `/theme` stays the root's flip (the user's request
of 2026-11-09: a flip on the terminal's own ground showed as almost
nothing).

Acceptance:

- `Own::Theme`, `Event::Theme { choice }`, `Notice::Theme { choice,
  terminal }`, `Told::Theme`, `Told::ThemeList`, `Tui::theme_choice`,
  `Tui::terminal_scheme` and `Tui::theme_list` as
  `temper/docs/plan/tui-box-polish/notes/contracts.md` has them.
- The TUI offers no root `Command::Theme`; `/theme` (`Own::Theme`) is
  offered wherever the root's was, signed out and on the welcome
  included, in its place: after `/open` (or `/chats`, or first with no
  project) and before `/help`, so the commands' order and `/help`'s note
  are as before but for its words, "Auto, light or dark".
- Pressed, or sent whole or begun ("/th"), it clears the box
  (`terminal::Act::Dismiss`) and opens the theme's list under the box,
  before any list but the front's quit question (`lists::read`): head
  (`look::LIST_HEAD`) "Theme. Auto follows your terminal."; "Auto", its
  words "light, as your terminal says", "dark, as your terminal says"
  or "dark: your terminal doesn't say" (the terminal's scheme told by
  the host, none where it says none); "Light"; "Dark" (`look::OPTION`);
  the current choice selected. Up and Down move, Enter takes, Esc
  (`Event::Theme { choice: None }`) lowers it, words typed lower it, and
  another screen (`Told::Went`) lowers it.
- Taking a choice: the host's `keep_theme` (from `theme-start`), the
  list lowered, the box cleared, and the notice: "Theme: auto, light as
  your terminal says.", "Theme: auto, dark as your terminal says.",
  "Theme: auto, dark: your terminal doesn't say.", "Theme: light." or
  "Theme: dark.".
- The host tells the TUI's state the choice and the terminal's scheme
  at `Start` and at each choice (`Told::Theme`); `theme-start`'s interim
  keep of the root's flip in `report` is gone.
- Goldens, host test, state rows and table rows below, biting; the
  README.status.md entry for 6.13's `Own` and `Theme`, 6.14, *`/theme`*,
  and 6.15, *The theme chosen*, gone.

## Inputs

- `temper/docs/design/client/README.md`, 6.13 (events table: `Own`,
  `Theme`, `Typed`; *The look*), 6.14 (*The commands*: *Offered*, *The
  TUI's own commands*, *`/theme`*; *The notice*) and 6.15 (*The scheme*,
  *The theme chosen*).
- `temper/docs/plan/tui-box-polish/notes/contracts.md`.
- `temper/docs/plan/tui-box-polish/notes/common.md`.

## Owns and may change

- Owns: `temper-client-tui`'s `src/event.rs` (`Own::Theme` last in the
  enum and `ALL`, named "/theme"; `Event::Theme`), `src/state.rs`
  (`Notice::Theme`; the three fields after the last field; `Told::Theme`
  and `Told::ThemeList` at the end of `Told` and of `step`; the
  accessors; `Told::Went` lowers the list), `src/words.rs` (the list's
  and the notice's words at the end), `src/view.rs` (`offered` and
  `offered_own`; leave `sign_in_drawn` and `landing`, `typing`'s, alone),
  `src/view/foot/lists.rs` (`read`'s order, a `theme` list function,
  `command_items` and `own_item`; leave `commands`' top, `typing`'s,
  alone), the notice's words where `view/foot.rs` says a `Notice` (one
  arm), and the tests and goldens in `files`;
  `temper-client-shell-tui`'s `src/host.rs` (`own`'s `Theme` arm;
  `event`'s `Theme` arm and its `Typed` arm lowering the list before
  `self.typed(value)`; `start` and `keep_theme` telling `Told::Theme`;
  the interim keep removed) and `src/tests.rs`.
- Do not edit `Host::typed`, `send`'s first lines or `launchpad`
  (`typing`'s), nor `route` and `heard` (`theme-start`'s, which stay).
- Test edits: "Dark or light" becomes "Auto, light or dark" in
  `tests/foot.rs`'s `note_states`, in `tests/commands.rs` where a
  description is read, and in `foot-commands-80x24-dark.txt` (its text
  line and its paint row's description run); host rows that pressed the
  root's `/theme` and read its flip (grep `Command::Theme` and
  `Theme::Light` in `src/tests.rs`) now take the list's choice.
- If `temper/tests/client/tests/suite/tui.rs` reads `/theme`'s flip,
  it takes the list too (an allowed reader edit; say so).

## Steps

1. Event, state and words. State rows in a new test function at the
   end of `tests/state.rs`, `the_theme_list_steps`: told, opened,
   lowered by `Theme`'s close, `Cleared`, `Went`, each moving the
   revision only on a change.
2. The view: `offered`, `offered_own`, `command_items`, the list in
   `lists::read`, the notice's words. Table rows in
   `tests/commands.rs` for `/theme` offered as own, its place and words,
   with no project and signed out included; in `tests/foot.rs`, the
   list's three items and their words for a terminal saying light, dark
   and none, the current one selected.
3. Two goldens, rows of `the_foots_are_their_goldens`:
   `foot-theme-80x24-dark.txt`, no word from the terminal, Auto chosen
   ("Auto" with "dark: your terminal doesn't say", active); and
   `foot-theme-answered-80x24-dark.txt`, the terminal answered a dark
   background (the band shows), Light chosen and active, Auto's words
   "dark, as your terminal says".
4. The host. Test, placed right after `theme-start`'s
   `a_theme_write_answer_never_reaches_the_root` in `src/tests.rs`:
   `the_theme_list_takes_keeps_and_notices_a_choice`: `/theme` opens it,
   Auto active; Down then Enter takes Light: the root's look is light,
   the frame draws light, `duties.asked` holds the `Save` of `theme` =
   `light`, the notice says "Theme: light.", the list is gone; Esc and
   typed words lower it; Auto with no terminal word notices "Theme:
   auto, dark: your terminal doesn't say."; a host started with the
   kept record opens the list with that choice active.
5. Journeys: the print test (`notes/common.md`, *Journeys*);
   `launchpad-chat`'s frame 02 lists `/theme`'s words, so reprint, and
   check that only those words changed.
6. Runs: the narrowest first; then `heavy cargo nextest run -p
   temper-client-tui -p temper-client-shell-tui -p temper-client-world -p
   temper-client-journeys`.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui -p temper-client-shell-tui --all-targets
   -- -D warnings`; the status entry; commit, push.

## Report

As the implementer's text asks; the goldens and journeys changed, and
the bites.
