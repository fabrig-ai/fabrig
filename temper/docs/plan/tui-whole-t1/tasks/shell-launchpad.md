+++
id = "shell-launchpad"
part = "host"
kind = "feature"
needs = ["tui-surface"]
files = [
  "temper/crates/temper-client-shell-tui/src/host.rs",
  "temper/crates/temper-client-shell-tui/src/run.rs",
  "temper/crates/temper-client-shell-tui/src/tests.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 150
+++

# The host carries the launchpad's own commands, `n` and the walk, `/chats N`, and pages opened on the web

## Goal

`temper-client-shell-tui`'s host takes the TUI's new events and words to
the root and its duties (`temper/docs/design/client/README.md`, 6.13, *The
TUI's events*; 6.14, *The TUI's own commands*, *Walking what needs you*):
`/change WORDS` starts a chat, `/ask WORDS` asks temper, `/chats N` opens
a chat, `/brief` and `/needs` tell the TUI's state, `n` walks, a walk's
choice answers or opens and marks its need shown, and `/open`, a goal's
Enter, a review or "Open it on the web" open a page's address through the
opener, noting it where no browser opened.

Acceptance (`notes/contracts.md`, section 5):

1. **`Own { command }`** (`host.rs`, `event`): `Change` and `Ask` write
   "/change " or "/ask " as the box's words (`Act::ChatWords`, so the
   write count moves); `Brief` tells `Told::Brief` and dismisses a
   standing list (`terminal::Act::Dismiss`); `Needs` tells `Told::Walk {
   by_key: false }`; `Open` sets `Duties.open` to `tui::focus(view)`,
   nothing where it is none.
2. **The `n` key:** read before the key reader, beside `tui::arrow`, by
   `tui::walk_key(view)`: where it is some, the host tells `Told::Walk {
   by_key: true }` and the key goes no further; `event_of` makes the same
   event, so a recording keeps it (`tui/journeys.md`, 3.3).
3. **`Open { address }`:** `Duties.open`; the host keeps whether the open
   it asked is a page's or the approval's tab, and an `Opened { opened:
   false }` for a page tells `Told::NotOpened { address: Some(address) }`
   (the approval's tab as today). The opener is given the address alone,
   as one argument, as `opener::spawn` does.
4. **`Walk { object, step }`:** `Choose { words }` makes
   `needs::Act::Choose { object, words }`; `Open { address }` opens as
   `Open` does; `Next` makes nothing; each then tells `Told::Walked {
   object }`. **`EndWalk`:** `Told::EndWalk`. `Aside { object }` while a
   walk stands sets it aside as on a chat.
5. **`Send` on the launchpad,** before the root's commands, in this
   order: `/change WORDS` (non-empty words after one space): the root
   `Event::Went { place: Address(NewChat { draft: false }) }`,
   `Act::ChatWords { words }`, `Act::StartChat`, the box cleared, the chat
   opened as the root opens a started chat; `/ask WORDS`:
   `Act::ChatWords { words }` and `Act::StartChat` on Home, as the box's
   words; `/chats N`: `tui::chat_number(view, N)`, `Act::Go` to its
   `Chat { task }`, and for no chat the TUI's unknown note (`Told::Unknown`)
   as for an unknown command; the walk's question set aside:
   `needs::Act::OpenReply`, `Act::CardWords`, `needs::Act::SendReply` for
   its object, then `Told::Walked { object }`, as a chat's question's
   words are answered. Words that are none of these go as today. An own
   command's name sent alone (`/brief`, `/needs`, `/open`, `/change`,
   `/ask`) is its `Own` event; a prefix of one is matched as root
   commands' prefixes are, in 6.14's order.
6. **Typing after `n`:** a `Typed { value }` while a walk stands that
   `by_key` opened and nothing `taken`: the box's words are "n" and the
   value (`Act::ChatWords`), then `Told::Typed` as today, which ends the
   walk.
7. **Tests,** a failing test first for each (`notes/common.md`), in
   `src/tests.rs`:
   - `each_event_becomes_its_acts`: rows `Own` × 5, `Open`, `Walk` × 3
     (`Choose`, `Open`, `Next`), `EndWalk`, each with the acts, duties and
     `Told` it makes.
   - a table `the_launchpad_takes_its_own_words` (new): `/change Add a
     CSV export` (Went `NewChat`, its words, `StartChat`), `/change` alone
     (`Own { Change }`), `/ask What landed?`, `/chats 1`, `/chats 2` of
     one chat (the unknown note), `/br` (`/brief`), the walk's aside
     answered in words (the three acts and `Walked`), ordinary words (as
     today).
   - `n` on the launchpad's empty box with a need (the walk, `by_key`),
     with words in the box (types), on a chat (types), and "o" typed after
     it (the box holds "no"); a page not opened (the note) and the
     approval's tab not opened (as today); the opener given the page's
     address as one argument (`the_opener_gets_one_argument_and_no_terminal_descriptors`
     gains a row).
8. **Status:** delete the **Not yet** bullet naming `shell-launchpad` in
   `temper/docs/design/client/README.status.md`. No other design line.
9. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-shell-tui`; `cargo check -p temper --tests`; `cargo
   check -p temper-client-world --tests`.

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`; `notes/contracts.md`,
  sections 3 and 5.
- `temper/docs/design/client/README.md`, 6.13 (*The TUI's events*, *The
  seam's shape*), 6.14 (*The TUI's own commands*, *Walking what needs
  you*, *The chat's question*), 7.1 (the approval's tab, opened).
- The code: `host.rs` (`event`, `send`, `arrow`, `event_of`, `approval`,
  `Duties`, its `Report::Key` path), `run.rs` (`open`, `Opened`),
  `opener.rs`; `temper-client-tui`'s `event.rs`, `state.rs`, `view.rs`
  (`walk_key`, `chat_number`, `focus`, `offered_own`, `arrow`);
  `temper-client-domain`'s `pages.rs` (`Act::ChatWords`, `StartChat`,
  `start_chat`), `pages/chats.rs` (`start`, `NewChat`), `pages/needs.rs`
  (`Act`); tests `src/tests.rs` (`each_event_becomes_its_acts`,
  `left_off_host`, `waiting_host`).

## Owns and may change

Owns the files listed. Not `temper-client-tui` (`tui-surface`'s,
`tui-launchpad`'s and `tui-walk`'s): a name it lacks is a question. Not
`temper-client-domain`.

Options, each decided:

- **`/change` refused:** where the root keeps the start's refusal on
  `NewChat`, the host goes back to `Home { project }` once the refusal is
  read and the TUI notes it in the view's words under the box, as a
  send's refusal is drawn; if the root drops the refusal when it leaves
  `NewChat`, stay on `NewChat`, which the TUI draws as the launchpad, and
  say so. Test the one taken.
- **`/change` on a project with no chat template:** the root's own
  refusal, as above.
- **The kind of the open asked:** a field of the host; if `run.rs` must
  carry it to tell the reports apart, change `run.rs` that far alone.

## Steps

1. The failing rows, seen failing; then the events, `n`, the sends, the
   opener's note; the narrowest test after each, e.g. `heavy cargo nextest
   run -p temper-client-shell-tui -E 'test(/each_event_becomes_its_acts/)'`.
2. The status edit; the runs of acceptance 9; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-shell-tui --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
3. Commit (why: the launchpad's own commands, its walk and pages opened on
   the web reach the root and the opener); push.

## Report

As the implementer's text says, with each row and the rule broken by hand
for it (for instance `n` typing on the launchpad's empty box, or a page
opened with more than its address), and the `/change` refusal option
taken.
