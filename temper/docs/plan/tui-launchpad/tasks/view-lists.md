+++
id = "view-lists"
part = "view-lists"
kind = "interface"
needs = ["view-shared"]
files = ["temper/crates/temper-client-view/src/shared.rs", "temper/crates/temper-client-view/src/shared/**", "temper/crates/temper-client-view/src/pages/terminal/commands.rs", "temper/crates/temper-client-view/src/pages/terminal/question.rs", "temper/crates/temper-client-view/src/pages/ask.rs", "temper/crates/temper-client-view/src/choice/picker.rs", "temper/crates/temper-client-view/src/cards/delegation.rs", "temper/crates/temper-client-view/src/cards/proposal.rs", "temper/crates/temper-client-view/src/confirm.rs", "temper/crates/temper-client-view/src/pages/chat.rs", "temper/crates/temper-client-view/src/pages/session.rs", "temper/crates/temper-client-view/src/pages/configure.rs", "temper/crates/temper-client-view/src/tests/shared.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 180
+++

# tui-launchpad: the view's shared lists, the chat's question and a card's line

## Goal

`temper_client_view::shared` gains, as data, what the browser's chat terminal lists under
its box and draws as a card's one line: the commands' lists (`/model`'s models,
`/effort`'s efforts, `/chats`' chats) and their heads, `/close`'s and `/start-over`'s
confirmation, the chat's question with its parsed choices, and a delegation's and a
proposal's one line. The browser's builders build from them, so its trees stay byte for
byte; the TUI's lists take them next.

Acceptance:

- `temper_client_view::shared` holds every item of
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 2 marked *view-lists*,
  spelled as given there (`Choice`, `list_head`, `models`, `efforts`, `chats`,
  `Confirmation`, `confirmation`, `QuestionChoice`, `Question`, `question`,
  `DelegationLine`, `delegation_line`, `proposal_line`, `refusal`), each documented with
  its design section.
- Each says what the browser's terminal draws, and that builder builds from it:
  - `models` and `efforts`: `choice::picker::terminal_models` and `terminal_efforts`; a
    `Choice` an option, its `name` the option's name, `words` its description ("your
    subscription", "default" after it on the default's, as the picker joins them), its
    `selected` the option's `State::Selected`, its `act` the act the option binds
    (`Act::Choice(choice::Act::Model { .. })`, `Effort { .. }`); the picker's "Connect a
    plan or key ›" link rows are not `Choice`s (a link, not an act): `models` leaves them
    out, and the browser's builder keeps drawing them itself.
  - `chats`: `commands::chat_options`; `name` "TN", `words` the title, `act` `Act::Go {
    address: Chat { task } }`, none selected.
  - `list_head`: `commands::under`'s three heads. `confirmation`: its confirms branch,
    `head`, `take` ("Close", "Start over") and `keep` ("Keep it open", "Keep this one"),
    none once the confirmation is sent.
  - `question`: `terminal::question::under`'s choice of card (the first open question of
    the page's inbox the person may answer, `Admission::Allowed` or `Bounded`, with a
    parsed question), whatever the look; `head` its parsed question, `choices` its parsed
    choices in order with `recommended`, `primary` `Parsed::primary()`; the browser's
    builder then draws from it, adding "Later" and "Answer in Needs you ›" itself.
  - `delegation_line`: `cards::delegation::terminal_card`'s kind ("T7 · Change"), title
    and state words; `proposal_line`: the title the terminal's proposal card line draws
    after "Proposal", none for a card that is no proposal.
  - `refusal`: the words the browser draws under its composer for the page's last send,
    jig's words of a refused or failed start (the chats' page's `refusal` and `outcome`,
    on `Ask` and `Home`) or message (the chat's `outcome`), and of the waiting question's
    answer refused (`needs().refused(object)`, as `terminal::question::under` draws it
    through `confirm::outcome_words`); each of those builders over it.
- Every existing test of `temper-client-view` passes unchanged.
- Rows, each a failing test first, in `src/tests/shared.rs`'s
  `shared_words_are_the_browsers` (the table `view-shared` made), with the view's existing
  roots (`src/tests/terminal.rs`'s `command_root`, `terminal_question_root`, the picker's
  in `src/tests/choice.rs`, the cards' in `src/tests/delegation.rs`) where they serve: each
  function's data against the browser's tree: `/model`'s list's option names, words and
  selected one, with a provider with no model left out of `models`; `/effort`'s; `/chats`'
  with the open chat left out, newest first; `/close`'s and `/start-over`'s heads and
  acts, and none once sent; `question` with choices (the recommended one's `primary`),
  with none (no choices), and none on a chat with no open question; a delegation's line;
  a proposal's title, and none for a question's card; `refusal` for a start refused on
  `Ask`, a message refused on a chat, and none after a send accepted.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "6.14, *The
  commands* and *The chat's question*, *Its words*: the shared surface's lists, ..." (its
  bullet's lines alone, not the blank lines around it), a stated departure from the
  implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 3.5, *A chat's terminal* (*A command's list*, *A
  goal's question*, *A proposal or an Accept card*), *The work's account and model* (the
  picker); 6.14, *The commands* and *The chat's question*.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0 and 2.

## Owns and may change

- Owns the files in `files`; `src/shared.rs` is `view-shared`'s, landed: add to it, and
  split it into `src/shared/*.rs` if it grows past a screen of items. Keep off
  `src/frame.rs`, `src/look.rs`, `src/words.rs` and `src/tests/rail.rs` (another draft's);
  if a function needs a line there, stop and say so.
- Readers to keep passing: the picker's builders (`choice.rs`, `pages/terminal/commands.rs`,
  `src/tests/choice.rs`), `ask::read` (`pages/needs.rs`, `pages/home.rs`, `pages/goal.rs`,
  `pages/terminal/question.rs`, `src/tests/needs.rs`), `cards::delegation::terminal_card`
  (`pages/chat.rs`), the proposal card's line (`pages/chat.rs`). Signatures may change where
  the share needs it, each reader edited in the same commit.
- `Choice::act` is `temper_client_domain::pages::Act`, the same value the browser's builder
  inserts into its bindings; the bindings stay the builder's.
- If the proposal card's terminal line has no title of its own (its words are its
  kicker and acts alone), `proposal_line` gives the title the browser's card heading
  draws, and you say so.

## Steps

1. The rows; `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-view --lib -E
   'test(/shared::/)'`; see them fail.
2. Each data function, then its builder over it.
3. The status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-view --all-targets -- -D warnings`; `TEMPER_BUNDLE=none heavy cargo
   nextest run -p temper-client-view -p temper-client-tui -p temper-client-shell-terminal`;
   `TEMPER_BUNDLE=none heavy cargo check -p temper-client-shell-web -p
   temper-client-gallery -p temper-client-world --tests`.
5. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen; each builder now over its data.
