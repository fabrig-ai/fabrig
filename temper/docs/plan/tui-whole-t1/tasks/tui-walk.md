+++
id = "tui-walk"
part = "walk"
kind = "feature"
needs = ["tui-launchpad", "tui-surface", "view-needs"]
files = [
  "temper/crates/temper-client-tui/src/view/foot/lists.rs",
  "temper/crates/temper-client-tui/src/view/foot/notes.rs",
  "temper/crates/temper-client-tui/src/tests/foot.rs",
  "temper/crates/temper-client-tui/src/tests/commands.rs",
  "temper/crates/temper-client-tui/src/tests/golden/**",
  "temper/docs/design/client/tui/journeys/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 140
+++

# The walk's prompt, the TUI's own commands in the list and `/help`, and the walk's and opener's notes

## Goal

Draw what `n` and `/needs` walk, a prompt a need under the box; list the
TUI's own commands (`/change`, `/ask`, `/brief`, `/needs`, `/open`) among
the root's in the commands' list and `/help`'s note, with the key "n walks
what needs you"; and note the walk's end and a page the opener didn't open
(`temper/docs/design/client/README.md`, 6.14, *Walking what needs you*,
*The TUI's own commands*, `/help`'s note).

Acceptance (`notes/contracts.md`, sections 3 and 4):

1. **The walk's prompt** (`view/foot/lists.rs`): `walk(root, tui, words)
   -> Option<Options>` as the contract has it, while `tui.walk()` stands:
   the first of `shared::need_prompts(root, words.view())` the walk has
   not shown; by its `act`, every variant listed:
   - `Answer(question)`: as `question_list` lays a chat's question (its
     head, its choices numbered, the primary first and active), each
     choice `Event::Walk { object, step: WalkStep::Choose { words } }`;
     then "Tell it something else" (`Word::Else`, `Event::Aside { object }`);
     then `Word::Next` (`Event::Walk { object, step: Next }`);
   - `Review { words, address }`: the head the row's `line.words`; "1. "
     and `words`, `Event::Walk { object, step: Open { address } }`; then
     `Next`;
   - `Web`: the head as above; "1. " and `Word::OpenOnWeb`, `Event::Walk {
     object, step: Open { address } }`, the address `http://` +
     `tui.address()` + `address::print(&Address::Inbox { filter })`, the
     filter the need's project alone; then `Next`.

   Its escape `Event::EndWalk`. `read` takes it first, before the
   confirmation, the listings, the commands and the chat's question (but
   after 6.17's quit question in front). The walk with every need shown
   draws no prompt.
2. **The commands' list** (`commands`): the offered root commands and
   `tui::offered_own(root, tui)` merged in 6.14's order (`/change`,
   `/ask`, `/model`, `/effort`, `/new`, `/chats`, `/brief`, `/needs`,
   `/open`, `/theme`, `/help`, then `/log` in front, then `/exit`), each
   whose name begins with the box's words; an own command's item binds
   `Event::Own { command }` and its words are `Word::ChangeCommand`,
   `AskTemper`, `BriefCommand`, `NeedsCommand`, `OpenCommand`.
3. **`/help`'s note** (`view/foot/notes.rs`): the own commands among the
   others in the same order, the padding counting them; the keys with
   `Word::KeyWalk` after "Esc closes a list" on the launchpad while
   walkable but for the box.
4. **The notes:** `Notes` gains the walk's end, `Word::WalkDone`, while
   the walk stands with every need shown, and the opener's,
   `Word::OpenAt` with `tui.not_opened()`; each drawn after the root's
   notes in `look::NOTE`. `read_notes` and `draw_notes` keep their
   signatures.
5. **Tests,** a failing test first for each (`notes/common.md`):
   - `tests/foot.rs`' `question_states` (or `read_states`, where lists are
     read): the walk over a question (choices, "Tell it something else",
     "Next", escape `EndWalk`), a sign-off (Review's words and address), a
     held task (Open it on the web, `/?project=P`), the second need after
     the first is shown, every need shown (no list), the walk over the
     commands' list and a chat's question (it stands first).
   - `tests/foot.rs`' `note_states`: the walk's end, the opener's note,
     neither once cleared.
   - `tests/commands.rs`' `the_tui_offers_its_commands`: rows the
     launchpad with a brief and a need (the twelve in order), with
     neither, `/c` (`/change`, `/chats`), `/b` (`/brief`), a chat (`/open`
     after `/chats`), no project (none of them), in front (`/log` before
     `/exit`); `/help`'s note with the own commands and "n walks what
     needs you".
   - `the_foots_are_their_goldens`: rows `foot-walk-80x24-dark`,
     `foot-walk-80x24-light` and `foot-walk-110x32-dark` (new): Hello's
     launchpad as `tui-launchpad`'s golden has it, the walk's prompt
     over the question "csv-export: comma or semicolon?" with "Comma"
     recommended, written by hand from 6.14; `foot-commands-80x24-dark`
     regenerated (the list gains the own commands), read line by line.
6. **Journeys:** reprint (`heavy cargo run -p temper-client-journeys`, no
   argument) and read the moved pages (`launchpad-chat/02`, its commands);
   if the replay goes stale, record it again as `notes/common.md` says.
7. **Status:** delete the **Not yet** bullet naming `tui-walk` in
   `temper/docs/design/client/README.status.md`. No other design line.
8. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-tui`; `cargo nextest run -p temper-client-shell-tui`;
   `cargo nextest run -p temper-client-journeys --lib -E
   'test(=tests::the_committed_journeys_are_their_print)'`.

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`; `notes/contracts.md`,
  sections 3 and 4.
- `temper/docs/design/client/README.md`, 6.14 (*Walking what needs you*,
  *The chat's question*, *The TUI's own commands*, *The list*, `/help`'s
  note), 6.17 (the quit question's place).
- The code: `view/foot/lists.rs` (`read`, `commands`, `description`,
  `question_list`, `draw`), `view/foot/notes.rs` (`Notes`, `help`,
  `read_notes`, `draw_notes`), `view.rs` (`offered`, `offered_own`),
  `event.rs` (`Own`, `WalkStep`), `state.rs` (`Walk`); the view's
  `shared::need_prompts`, `NeedAct`; `temper-client-domain`'s
  `address.rs` (`print`, `Filter`); tests `tests/foot.rs`,
  `tests/commands.rs`.

## Owns and may change

Owns the files listed. Not `view.rs`, `words.rs`, `state.rs`, `event.rs`
(`tui-surface`'s), not `view/launchpad.rs` or `view/foot.rs`
(`tui-launchpad`'s, done); if `foot::read` must pass the walk's list, the
one call in `foot.rs` that reads `lists::read` may change its arguments,
and nothing else there. Not `temper-client-shell-tui`
(`shell-launchpad`'s).

Options, each decided:

- **The question's "Tell it something else" with no parsed choice:** as
  the chat's question, its line and "Tell it something else" alone, then
  "Next".
- **A need whose prompt the view can't build** (no `need_prompts` row for
  a shown object any more): it is passed; the next is drawn.

## Steps

1. The failing rows and goldens, seen failing; then the walk's reader, the
   commands' list, `/help`, the notes; the narrowest test after each.
2. The goldens: new ones by hand, the regenerated one read line by line;
   the journeys reprinted and read.
3. The status edit; the runs of acceptance 8; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
4. Commit (why: what needs the person is walked one prompt after another
   from the launchpad, and the TUI lists its own commands); push.

## Report

As the implementer's text says, with each row and the rule broken by hand
for it (for instance a shown need drawn again, or `/brief` offered with no
brief), and each golden and journey page moved.
