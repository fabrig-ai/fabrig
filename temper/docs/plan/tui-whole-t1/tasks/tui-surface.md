+++
id = "tui-surface"
part = "surface"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-tui/src/event.rs",
  "temper/crates/temper-client-tui/src/state.rs",
  "temper/crates/temper-client-tui/src/words.rs",
  "temper/crates/temper-client-tui/src/limits.rs",
  "temper/crates/temper-client-tui/src/lib.rs",
  "temper/crates/temper-client-tui/src/view.rs",
  "temper/crates/temper-client-tui/src/view/launchpad.rs",
  "temper/crates/temper-client-tui/src/tests/event.rs",
  "temper/crates/temper-client-tui/src/tests/state.rs",
  "temper/crates/temper-client-tui/src/tests/launchpad.rs",
  "temper/crates/temper-client-tui/src/tests/limits.rs",
  "temper/crates/temper-client-shell-tui/src/host.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 130
+++

# The TUI's names for the launchpad as drawn: its events, state, words, rows and keys

## Goal

Lay every name the launchpad's tasks build on, so that they run beside one
another with no question: the events, the walk's state and the notes'
state, all the plan's words, the left-off rows' shape and the view's keys
and focus, exactly as `notes/contracts.md`, section 3, has them. Nothing
drawn changes: every golden stays as it is.

Acceptance:

1. **Events** (`src/event.rs`, exported from `lib.rs`): `Event` gains
   `Own { command }`, `Open { address }`, `Walk { object, step }` and
   `EndWalk`; `Own` with `ALL` and `name`, and `WalkStep`, as the
   contract has them. `LOG_COMMAND` stays.
2. **State** (`src/state.rs`): `Told` gains `Walk { by_key }`, `Walked {
   object }`, `EndWalk`, `Brief` and `NotOpened { address }`; `Tui` gains
   `walk()`, `brief_shown()` and `not_opened()`, and `Walk` its
   `shown`, `by_key` and `taken`, with the rules the contract gives
   (`Went` and `Cleared` end the walk and clear both notes; `Typed` with
   a value ends the walk). `limits::WALK = 64` bounds the shown objects;
   a walk past it shows the rest as not shown (no panic).
3. **Words** (`src/words.rs`): the twenty words of the contract's table,
   numbered 90 to 109 in its order, their English exactly as there, in
   `ALL`, with their holes (`ChatNumber` a number, `ChangeStart` and
   `OpenAt` a name). `limits()` keeps holding every word
   (`tests/limits.rs`' rows grow by the new words where they list words).
4. **The left-off rows** (`src/view/launchpad.rs`): `LeftOff` and
   `LeftKind` as the contract has them. `left_off` fills them for today's
   chats, each `Chat { number }` by its place (1 the newest), `project`
   the launchpad's project's name, `at` the view's clock of its
   `last_activity` (`TimeForm::Clock`), `state` as today; `draw` draws
   them exactly as today (title, " · ", state), so no golden moves. No
   goal is read here (`tui-launchpad` reads it).
5. **The view's keys and focus** (`src/view.rs`, exported from `lib.rs`):
   `walk_key`, `chat_number`, `focus` and `offered_own` as the contract
   has them; `View` keeps what they need, filled where the view has the
   root and the TUI's state (`build`) or from the launchpad's part
   (`signed`); `event()`'s Enter on the empty box with a goal selected
   makes `Event::Open` with the goal's page (`Address::Task { task }`),
   with a chat as today.
6. **The host compiles:** `temper-client-shell-tui`'s `host.rs` matches
   the new events with no behaviour yet (each arm does nothing and says
   so in one comment naming `shell-launchpad`); nothing else there.
7. **Tests,** a failing test first for each (`notes/common.md`):
   - `tests/event.rs`: a row each for `Own::name` over `Own::ALL`.
   - `tests/state.rs`' table: rows walk started by `n` and by `/needs`,
     a need walked (`shown`, `taken`), the walk ended by `EndWalk`, by
     `Went`, by `Cleared` and by typing, `/brief`'s note shown and cleared
     by `Went`, the opener's note with and without an address and
     cleared; a walk past `WALK` objects.
   - `tests/launchpad.rs`: rows for `walk_key` (the launchpad with a need
     and an empty box: `Own { Needs }`; with words, with a list, with no
     need, on a chat: none), `chat_number` (1 and 2 of two chats, 3 none),
     `focus` (a selected chat `http://127.0.0.1:8367/chats/T`, a selected
     goal `/tasks/T`, none selected `/projects/P`, a chat's page, signed
     out none), Enter on a selected goal (`Open`), and `offered_own` (the
     launchpad with a brief and a need: all five; with neither: `Change`,
     `Ask`, `Open`; a chat: `Open`; no project: none). Build a goal row
     as data (a `Launchpad` part with a `LeftKind::Goal` row) where the
     reader doesn't read goals yet.
8. **Status:** delete the **Not yet** bullet naming `tui-surface` in
   `temper/docs/design/client/README.status.md`. No other design line.
9. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-tui`; `cargo nextest run -p temper-client-shell-tui`.
   Every golden of both crates stays as it is.

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`; `notes/contracts.md`,
  section 3 (and sections 4 and 5, for what the later tasks build on these
  names).
- `temper/docs/design/client/README.md`, 6.13, *The TUI's events*; 6.14,
  *The launchpad* (*Where you left off*, *Walking what needs you*, *Start*,
  `/brief`'s note), *The TUI's own commands*, `/help`'s note.
- The code: `src/event.rs` (`Event`, `LOG_COMMAND`); `src/state.rs`
  (`Told`, `Tui`, `step`); `src/words.rs` (`Word`, `ALL`, `number`,
  `english`, `capacity`); `src/limits.rs`; `src/view.rs` (`View`, `build`,
  `signed`, `event`, `escape`, `arrow`, `offered`); `src/view/launchpad.rs`
  (`LeftOff`, `left_off`, `sections`); `temper-client-domain`'s
  `address.rs` (`print`, `Address::Task`), `pages/home.rs` (`facts`,
  `Facts::brief`), `pages/needs.rs` (`needs`); `temper-client-shell-tui`'s
  `host.rs` (`event`, its match on `tui::Event`).

## Owns and may change

Owns the files listed. In the same weeks `tui-views` changes
`view/chat.rs`, `view/chat/read.rs`, `tests/chat.rs` and adds
`view/printed.rs` and a `mod printed;` line in `view.rs`: keep this
task's `view.rs` hunks away from the `mod` lines. Not `view/foot*`
(`tui-launchpad`'s and `tui-walk`'s), not `look.rs` or `theme.rs` (no
token is added). `host.rs`: the new arms alone (`shell-launchpad` fills
them).

Options, each decided:

- **Where the focus is computed:** in `build`, where root and state are,
  kept in `View`; if `signed`'s data is the better place (a field of the
  launchpad's part), put it there and say so.
- **The walk's objects:** a bounded list of `ObjectKey`; if `ObjectKey`
  lacks an equality the list needs, compare by its fields and say so.

## Steps

1. The failing rows, seen failing; then events, state, words, the rows'
   shape, the view's keys; the narrowest test after each, e.g. `heavy
   cargo nextest run -p temper-client-tui -E 'test(/state::/)'`.
2. The host's arms; the status edit; the runs of acceptance 9; `cargo
   fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-client-tui -p temper-client-shell-tui --all-targets -- -D
   warnings`; `python3 -I -B tools/citations/citations.py --base
   origin/main`.
3. Commit (why: the names the launchpad as drawn is built on, laid first
   so its parts are built side by side); push.

## Report

As the implementer's text says, with each row and the rule broken by hand
for it, where the focus is computed, and any name that departs from the
contract (none should).
