+++
id = "tui-chat"
part = "tui-chat"
kind = "feature"
needs = ["tui-foot", "view-lists"]
files = ["temper/crates/temper-client-tui/src/view/chat.rs", "temper/crates/temper-client-tui/src/view/chat/**", "temper/crates/temper-client-tui/src/tests/chat.rs", "temper/crates/temper-client-tui/src/tests/golden/chat-*.txt", "temper/crates/temper-client-view/src/shared.rs", "temper/crates/temper-client-view/src/shared/**", "temper/crates/temper-client-view/src/tests/shared.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 240
+++

# tui-launchpad: a chat, read as the browser's chat terminal reads it

## Goal

On `Chat { task }` the TUI draws the chat: its top line, "✦ PROJECT / TITLE TN" and the
address; its conversation in the body, a log that follows its end: the person's words on a
band after "❯", a run of calls as one line in words with its time under "▸", counting what
ended and naming the call under way while it runs, the agent's answer after "✦" as
Markdown with numbered lists numbered, the words being written growing whole with a small
block at their end, the three dots before the first call or word, a card as one line; then
the notes; under it the foot `tui-foot` drew, the running line, the box "Reply", and the
chat's question when it waits on the person.

Acceptance, as `temper/docs/design/client/README.md`, 6.14, *A chat* says, every bullet:

- `temper_client_view::shared::working(root, w)` (contracts.md, section 2, *Added for
  `tui-chat`*), over the browser's `chats::working`, with a row in `src/tests/shared.rs`'s
  `shared_words_are_the_browsers`: true while the browser's run line is under way, false
  once the words being written begin.
- `src/view/chat.rs`: `Chat`, `read`, `draw`:
  - *The top line,* region `tui-chat-top`, spread: a paragraph of `Role::Text` children,
    `shared::say(Star)` and a space (`tui-star`), the project's name (`tui-name`), `Between`
    (`tui-faint`), `shared::heading(title)` (`tui-name`), a space and the chat's number as
    the view's `Word::Chat` says it ("T2", `tui-hint`); then the address (`tui-address`),
    left out where `Tui::columns()` is above 0 and the two children's widths, two spaces
    and two columns of padding pass it.
  - *The conversation:* a `Role::Log` with the end request `Chat::to_end()`, holding one
    `Role::List` in `tui-spaced`, an item an entry, in the thread's order
    (`pages().chat()`'s `thread.lines()`), walked as the browser's terminal walks it
    (`temper-client-view`'s `pages/chat.rs`, its conversation's lines; README.md, 3.5, *A
    chat's conversation*): the request first where the window's head is the chat's start
    (`Chat::request`), each words line a person sent (`tui-said`, a paragraph a paragraph
    of words), a task's words (`tui-agent`, "TN · " and its words), each committed turn's
    texts (`tui-agent`, `skein_ui::markdown::read`) and calls (into a
    `shared::Run`: `call`, `result`), a run flushed by a text, words, a card or the
    window's end into an item `tui-run` of its `RunLine`'s name, two spaces and its
    description (`tui-run-failed` where `failed` is above 0); the words being written
    (`Line::Provisional`, nonempty) read by `markdown::read_growing` whose end puts a
    `Role::Text` of one space in `tui-end` after the last word; the last run's `line`
    given `Chat::current_call()` and `running` `shared::working(root, w)`; the three dots
    (`tui-dots`, `Dots`) while `working` and nothing is being written and no call is under
    way. What the browser's terminal hides stays hidden; a document's write and "Being
    sent" / "Sent" are not drawn.
  - *The cards,* after the conversation's lines, items in `tui-agent`: each of
    `Chat::delegations` whose task `Chat::delegation(client, task)` reads, its
    `shared::delegation_line` (kind, title, state, joined by `Separator`); each card of
    `Chat::cards` with a `shared::proposal_line`, `Proposal` (the view's, `tui-signal`) and
    the title. No card is bound.
  - *The notes,* `foot::draw_notes`, after the log.
- Rows, each a failing test first, in `src/tests/chat.rs`: a table test
  `the_chat_draws_each_entry`, a row a case: the person's request and a later words line
  on the band; a task's words; a committed turn of two calls then a text (the run's line
  before the text, its time); `see changes` and three `see change N` read "Listed changes,
  read 3 changes"; a failed call's description in `tui-run-failed`; a run under way with one
  call ended naming the call under way with "…" and no time; the three dots; the words
  being written, their end's block after the last word, and the same blocks committed
  without it; a numbered list numbered; a delegation's line and a proposal's; the top
  line's address left out when narrow; nothing bound in the body. `read` rows where the
  root's fixtures reach a chat (its task, activity and run documents answered by
  `f::client` with the wire's documents), `draw` rows from data where they don't, the
  report naming which.
- **Goldens,** written by hand from the design and the request's 03 and 04, checked cell by
  cell, rows of `goldens::check` in `src/tests/chat.rs`: `chat-80x24-dark.txt` (03: the
  top line, the person's words, the run's line with its time, the answer being written
  with its numbered list and its end's block, the running line "✦ working", the box
  "Reply", the quiet line); `chat-question-80x24-dark.txt` (04: the answer committed in
  place, the person's reply, the box empty, the question's list as `tui-foot` draws it);
  `chat-40x12-dark.txt` (03's chat narrow: no address, no closed line).
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "6.14, *A
  chat* and *The chat's question*: no chat is drawn." (its bullet's lines alone), a stated
  departure from the implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 3.5, *A chat's conversation* (*the words being
  written*, *its calls*, *the person's words*, *a delegate's words*, *not drawn*), *A
  chat's terminal* (*The top line*, *The conversation*, *A proposal or an Accept card*);
  6.14, *A chat*; 6.15.
- `temper/docs/design/draft/chat-reading.md`, 2, 3 and 6; its status.
- `skein/docs/design/ui.md`, section 10 (the reader, `read_growing`);
  `skein/docs/design/ui-screen.md`, 3 (`mark`, `band`, `spaced`), 5.2 (*The body follows a
  log's end*), 5.3.
- The request's frames, for the look only: `temper/docs/plan/tui-launchpad/inputs/launchpad-03.txt`
  and `-04.txt`; their pages `git show
  9b3942a123:temper/docs/design/client/tui/journeys/launchpad/0N.html`.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0 to 3.

## Owns and may change

- Owns `src/view/chat.rs` (and `chat/` if it splits), `src/tests/chat.rs`, the goldens
  `chat-*`, and in `temper-client-view` `shared::working` and its row alone (no other task
  writes that crate in this wave; keep off its `frame.rs`, `look.rs`, `words.rs` and
  `tests/rail.rs`). `tui-launchpad` runs beside this task in the same crate, in
  `view/launchpad.rs`, `tests/launchpad.rs` and its own goldens: touch none of them, nor
  `foot.rs`, `view.rs`, `look.rs`, `theme.rs`, `event.rs`, `state.rs`, `words.rs` or
  `goldens.rs`.
- The frames differ where the design does: the top line is padded one column at each end
  (03 starts at the row's first column); no blank row between the top line and the first
  entry nor after the last (skein's `spaced` lays none there); the end's block is a
  reversed space. Name each golden line that differs from 03 or 04 in the report, and why.
- The walk follows the browser's rules for what ends a run and what is hidden; where the
  browser's walk reads a fact the TUI's data doesn't hold (a card's place at its call),
  the TUI's design puts the card after the conversation: keep to it.
- If the TUI's tree bound (`limits`) refuses a long answer's tree, the bound is
  `tui-surface`'s and at least the view's: say so with the numbers rather than raise it.

## Steps

1. The rows and the goldens by hand; `TEMPER_BUNDLE=none heavy cargo nextest run -p
   temper-client-tui --lib -E 'test(/chat::/)'` and `-p temper-client-view --lib -E
   'test(/shared::/)'`; see them fail.
2. `shared::working`; `chat.rs`'s `read`, then `draw`.
3. The status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-tui -p temper-client-view --all-targets -- -D warnings`;
   `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-tui -p temper-client-view
   -p temper-client-shell-tui`; `TEMPER_BUNDLE=none heavy cargo check -p temper -p
   temper-client-world -p temper-client-journeys --tests`.
5. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen; the golden lines that differ
from 03 and 04, and why.
