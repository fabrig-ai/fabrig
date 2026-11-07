+++
id = "terminal-cards"
part = "terminal-cards"
kind = "feature"
needs = ["terminal-commands"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/terminal.rs",
  "temper/crates/temper-client-view/src/pages/terminal/**",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chat_line.rs",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/cards/delegation.rs",
  "temper/crates/temper-client-view/src/confirm.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/terminal.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 150
+++

# A chat's terminal: a goal's question, its cards on one line, a failed call's error

## Goal

Land what the look request `chat-terminal` names as not drawn, which a real chat has, in the terminal's language: a goal's question as a list under the box, a proposal's and an Accept card in the conversation and a delegation's card each one line where a line will do, and a failed call's error on its folded row. The design is `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*, *A goal's question* and *A proposal or an Accept card*, and *The conversation*'s failed fold.

Acceptance, by the tests below. No browser runs, here or in any step:

1. While the terminal draws a chat whose wait is on the person, the page reads the inbox of the chat's subtree, its questions (`Read::Inbox` with the chat as its root, `goal: Some(chat)`, kind questions), into the page's inbox (`Pages.inbox`), as a tracked task's page reads its own; the browser's look of a chat reads none, and a chat leaving the terminal or its wait drops it.
2. The terminal draws the first loaded question under the box, where no other list is open, as a list: its question line as the head (`TERMINAL_HEAD`), its parsed choices as the options menu's items (contract 3), the recommended one selected, bound to `needs::Act::Choose`, then "Later", bound to `needs::Act::Later`. A question with no choices draws its line and "Answer in Needs you ›", a link to its card. Answering is the shared one (`Pages.needs`): its hold, its undo and its words as Needs you's.
3. A proposal's card in the conversation, in the terminal, is one line: its kicker "Proposal" in the accent, its title, then its acts ("Accept", "Reject…" as today's), its facts folded under it in a disclosure as a call's are. Accept's confirmation, where it is two acts, draws as a list under the box as `/close`'s does; otherwise its sheet as today.
4. A delegation's card in the conversation, in the terminal, is one line: "TN · KIND", its title, its state and "Open"; its details and steps are not drawn there.
5. A fold whose call failed shows, in the terminal, its first failure's line on its folded row, "failed: WORDS", in place of "1 failed"; the browser's look keeps "1 failed".
6. The gallery holds `chat-terminal-question` and `chat-terminal-proposal` (`browser.md`, 7.2), each with its `HOLDS` row.
7. The cards' rules are in `temper.css` under the chats marker, after `terminal-commands`' list rules, in the terminal's type and ground.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules, the requests and how to read them.
- `temper/docs/plan/look-round-16/notes/contracts.md`, 3 to 6.
- `temper/docs/design/client/README.md`, 3.5, *A chat's terminal* (all of it), *Needs you* and *Whole* (a question's choices and answering), *A card in the conversation*; 5.7 (a delegation's card).
- `temper/docs/design/client/browser.md`, 2.5, *A chat's terminal*; 7.2's `chat-terminal-question` and `chat-terminal-proposal`.
- The request `chat-terminal`: its message, *Not drawn*.

## Owns and may change

Owns the header's files: in `pages.rs` (domain) only the chat's inbox read and its start and stop; in the view's `pages/chat.rs`, `cards/*` and `confirm.rs` only the terminal's branch of what they draw.

Reader edits allowed:

- `temper-client-domain/src/pages/needs.rs`: none by default; `answerable` reads `Pages.inbox`, which the chat now fills. If it needs a line to take the chat page's cards, that line, named in the report.
- `temper-client-view/src/pages/ask.rs`: a `pub(crate)` on what the list shares with Needs you's card (`Asked`, `read`), no change of what it does.
- `temper-client-view/src/pages/chat_line.rs`'s `Run::flush`: a parameter saying the terminal draws, its callers given it.
- A test of `temper-client-view` that lists a card's or a fold's nodes in the browser (`tests/cards.rs`, `tests/chat_line.rs`, `tests/delegation.rs`): nothing should change; if one does, it is a red of yours.
- Another draft's task (`terminal-browser`) changes the domain's `pages.rs` beside you: keep to your lines.

Options, each decided:

- **Which question:** the inbox's first question card of the chat's subtree that is answerable, as Needs you orders them; one at a time.
- **A reply:** not in the terminal's list; "Answer in Needs you ›" stands for a question with no choices, and Later for one with.
- **When the read starts:** when the page's state says the chat waits on the person (its status "Waiting for you", jig's wait on the person) while the terminal draws; again on the inbox's changes as the tracked task's page watches them.
- **The proposal's facts:** a `Disclosure` named "Details", folded, holding today's facts' nodes.
- **Settings proposals** (`cards/proposal.rs`'s `settings`): the same one line, its Accept as its own today.

Leave alone: full screen and the kept record (`terminal-browser`'s), the pane (`terminal-draft`'s), the commands (`terminal-commands`'), `words.rs` (the terminal's words are in `pages/terminal/words.rs`), the browser's look of every card.

## Steps

### 1. The question's read

In the domain, the chat page's inbox read and its lifetime, as acceptance 1.

### 2. The view

`pages/terminal.rs`'s `under` draws the question's list; `pages/chat.rs`'s proposal card, `cards/delegation.rs`'s card and `chat_line.rs`'s fold take their terminal branch; new tokens after the last.

### 3. The tests

- **New, domain:** in `temper-client-domain/src/tests/pages/chat.rs`, `a_terminal_chat_reads_the_question_it_waits_on`: rows of (the look: terminal or not; the chat's wait: on the person, on its delegates, none) and whether the inbox read is asked with the chat as root; a row where the look leaves the terminal and the inbox goes.
- **New, view:** in `temper-client-view/src/tests/terminal.rs`, `the_terminal_draws_what_a_real_chat_has_on_a_line`: rows of a question with two choices and a recommendation, a question with none, a proposal with its facts, a settings proposal, a delegation running and landed, a fold with one failed call; each checks the nodes drawn and their bindings, and the browser's look for the same state unchanged.
- **The gallery:** `chat-terminal-question` (a chat whose goal asked the person, with two choices; ">_") and `chat-terminal-proposal` (a configuring chat with a proposal waiting; ">_"); `names.txt` sorted; a `HOLDS` row each.

Break each rule once by hand (the read's condition, the recommended selection, the one line's folded facts, the failed row's words), run the test, see it fail for that reason, restore, and name each in the report.

### 4. The checks

One command at a time:

- `heavy cargo nextest run -p temper-client-domain --lib -E 'test(=NAME)'` and `heavy cargo nextest run -p temper-client-view --lib -E 'test(=NAME)'` for each test you added or changed.
- `cargo fmt`, then `python3 -I -B temper/scripts/old_forms.py`.
- `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-gallery --all-targets -- -D warnings`, and `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-view -- -D warnings`.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-gallery -p temper-client-journeys -p temper-client-world`. No recorded journey reads the new inbox: none draws the terminal.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 5. Commits

- **Trailer:** `Look-request: chat-terminal` on every commit, one line above the other trailers.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and, for the account `terminal-close` writes:

- each state the request's message names as not drawn, landed or how it differs;
- each rule broken and the failure seen;
- each reader file you edited beyond the header's.

## Rebase (2026-11-07, step merge, foreman look-round-16/terminal-cards/foreman-1)

resolve the words.rs conflict by combining vocabulary counts
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`, in its three counted vocabulary declarations.
Integration commit `07dd6ddd61` retained main’s approval vocabulary and set counts to 2060 phrases, 3631 segments and 50547 bytes.
Task commit `597ba2b66a` adds three terminal phrases, three segments and 38 bytes.
Preserve integration’s vocabulary and the task’s additions in `pages/terminal/words.rs`; use combined counts 2063, 3634 and 50585.
Keep this shared-file adjustment limited to the counted declarations; it is required baseline maintenance.
Verify with `heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'`.
Run the task’s two new focused tests after rebasing to check the automatically merged chat and terminal changes.
