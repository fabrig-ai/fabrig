+++
id = "terminal-commands"
part = "terminal-commands"
kind = "feature"
needs = ["terminal-frame", "field-options"]
files = [
  "temper/crates/temper-client-domain/src/pages/terminal.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/terminal.rs",
  "temper/crates/temper-client-view/src/pages/terminal.rs",
  "temper/crates/temper-client-view/src/pages/terminal/**",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/terminal.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 180
+++

# A chat's terminal: its commands, their lists and its notes

## Goal

Land the commands of the look request `chat-terminal`: "/" in the box opens the commands the page offers, filtered as the person types, moved through with the arrows, completed with Tab, taken with Enter or a click, closed with Esc; each command does the page's own act, and `/model`, `/effort`, `/chats`, `/close` and `/start-over` open their lists under the box. A closed chat keeps a box for commands. The shapes are `notes/contracts.md`, 6, binding; the list is skein's options menu (contract 3).

Acceptance, by the tests below. No browser runs, here or in any step:

1. `pages::terminal` gains `Command`, `Listing`, `Note`, `Act::Run` and `Act::Dismiss`, and `Pages` `listing` and `notes`, as contract 6 has them, each `Run` doing the act it names there.
2. In the terminal, words in the box that begin "/" and hold no white space draw, under the box in the quiet line's place, a menu named "Commands" with `State::Options`: each command the page offers whose name begins with the words, in the table's order (`temper/docs/design/client/README.md`, 3.5, *Commands*), named by its name, described by its words, bound to `Run`. Words naming none draw no list. The words are the design's table's, exactly.
3. A command is offered where the design's table says: `/pause` where the bar would draw Pause, `/close` where it would draw Close, `/start-over` where Start over, `/goal` where "Start goal…", `/draft` where "Draft" or "+ Draft", `/model` where the picker is, `/effort` where the chosen model has efforts; the rest always.
4. Sending words that begin "/" and hold no white space sends nothing: a command's name runs it, any other words note "Unknown command /WORDS. /help lists them.".
5. `/model`, `/effort` and `/chats` draw their lists as *A command's list* says, head and options, the current one selected, each choice bound to the page's own act (`choice::Act::Model`, `choice::Act::Effort`, `Act::Go`); `/close` and `/start-over` draw the confirmation as a list of its two acts (`Act::Confirm`, `Act::Cancel`), and no sheet; a confirmation that asks delegates' fates draws its sheet as today.
6. While any list is open, the form holds a button bound to `Act::Terminal(Dismiss)` with `Shortcut::ESCAPE`, so Esc closes it (skein's `ui-web.md`, 7); it draws no words a person sees.
7. The quiet line's model is a button bound to `Run { command: Model }`, in place of the picker's disclosure.
8. Notes: `/help`'s table of the commands and keys, an unknown command's, `/font`'s "Font: DejaVu Sans Mono." or "Font: IBM Plex Mono.", a closed chat's "This chat is closed. /new starts another.", after the conversation, in `TERMINAL_NOTE`, at most 8.
9. A chat that has ended draws, in the terminal, its box for commands, with the placeholder "/new starts another chat", and its quiet line.
10. The list's rules (`terminal.css`'s `.tt-list`, `.tt-head` and `[role="option"]` rules) are in `temper.css`, under the chats marker, after `terminal-frame`'s terminal rules; the gallery holds `chat-terminal-draft`, `chat-terminal-commands`, `chat-terminal-model`, `chat-terminal-light` and `chat-terminal-closed` (`browser.md`, 7.2).

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules, the requests and how to read them.
- `temper/docs/plan/look-round-16/notes/contracts.md`, 3 to 6: binding.
- `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*: *The box* to *Keys*; *A chat's bar* and *A person's pause*, for what each command does; *The work's account and model*, *The menu*, for `/model`'s and `/effort`'s choices.
- `skein/docs/design/ui.md`, 3.2, *options*; `skein/docs/design/ui-web.md`, 7, *A field's options* and *Escape*.
- `temper/docs/design/client/browser.md`, 2.5, *A chat's terminal*; 7.2's `chat-terminal-*` rows.
- The request `chat-terminal`: its message, `terminal.js`'s `COMMANDS` and lists, pages 04 and 05.

## Owns and may change

Owns the header's files; in `pages/chat.rs`, `new_chat.rs` and `configure.rs`, only the terminal's branch that `terminal-frame` made, and the confirmation's skip while it draws as a list.

Reader edits allowed:

- An exhaustive `match` on `pages::Act` or `terminal::Act` elsewhere in temper: the arms for `Run` and `Dismiss`.
- `temper-client-domain/src/pages/chat.rs` or `chats.rs`: a `pub(crate)` accessor for what `Run` needs (the composer's words cleared, the chats read asked), one each, if the act's dispatch can't reach it from `pages.rs`.
- A view test of the picker (`tests/choice.rs`) if the picker gains a `pub(crate)` function the list shares: its rows unchanged.

Options, each decided:

- **The models' choices:** the rows the picker's menu draws (`choice/picker.rs`, `menu`), shared by a `pub(crate)` function there that both draw from, not a copy; "All models" and "Find a model" are left out of the terminal's list.
- **The chats' read:** the read `NewChat` makes of the project's chats (`Pages.chats`, as `Chats` reads them), asked by `Run { Chats }` where the page has none; its Live rows, newest activity first, the chat itself left out.
- **Offered:** the view decides, with the predicates the bar uses; the domain runs what it is given, the page's own act refusing what it would refuse.
- **The help's table:** each command's name padded to the longest plus two, then its words; a blank line; "Enter sends", "Shift+Enter makes a new line", "Ctrl+\ shows or hides the draft", "PgUp and PgDn read back", "Esc closes a list", as `terminal.js`'s `help` has them, in a `Code` node so its columns hold.
- **The Escape button's words:** "Close the list", hidden by the terminal's rules; the tree names it.

Leave alone: full screen, the kept record and the shell's host (`terminal-browser`'s), the pane's look and `pages/document.rs` (`terminal-draft`'s), the question and the cards (`terminal-cards`'), `words.rs`, `temper-client-shell-terminal`. A draft beside this one (`ui-forge-identity/approval-card`) changes `look.rs`, `temper.css` and the gallery: keep to your lines.

## Steps

### 1. The domain

`pages/terminal.rs` gains contract 6's types and acts; `pages.rs` the listing and the notes on `Pages`, `Run`'s and `Dismiss`'s dispatch, the listing's closing on any other act, the notes' clearing on an address change, and the send's interception while the terminal draws.

### 2. The view

`pages/terminal.rs`'s `under` draws the open list (commands, a command's list, the confirmation as a list) in the quiet line's place, or the quiet line with its model button; `notes` draws the notes; the composer of an ended chat draws in the terminal with its placeholder. New tokens, if any, after `terminal-frame`'s.

### 3. The tests

- **Rows of what exists:** `the_look_moves_by_its_acts` (`tests/pages/terminal.rs`) gains `Theme` and `Font` noting rows.
- **New, domain:** `a_command_runs_the_pages_own_act`, in `tests/pages/terminal.rs`: a row per command from a chat at rest, with the act or listing or note it gives and the box's words cleared; rows for a send of `/model`, of `/nope`, of `/model now` (sent as words, white space in it), and of words on an ended chat; a listing closed by a later act and by `Dismiss`; `Dismiss` clearing "/mo" and leaving "hello".
- **New, view:** `the_commands_list_what_the_page_offers`, in `tests/terminal.rs`: rows of (box's words, the chat's state: at rest, paused, configuring, with a document, ended) and the commands' names drawn, in order, with `State::Options` on the menu, each item's description its words and its binding `Run`; `/model`'s, `/effort`'s and `/chats`' lists' heads and selected rows; `/close`'s confirmation as a list and no dialog; the Escape button while a list is open and not otherwise; the help note's lines.
- **The gallery:** `chat-terminal-draft` (`chat-draft-split`'s world, ">_" pressed), `chat-terminal-commands` (">_", then "/" typed), `chat-terminal-model` (">_", then `/model`), `chat-terminal-light` (">_", `/theme`, `/font`, in Split), `chat-terminal-closed` (a closed chat, ">_"); `names.txt` sorted; a `HOLDS` row each.

Break each rule once by hand (the prefix filter, an offered predicate, the send's interception, the listing's closing, `Dismiss`'s "/" test), run the test, see it fail for that reason, restore, and name each in the report.

### 4. The checks

One command at a time:

- `heavy cargo nextest run -p temper-client-domain --lib -E 'test(=NAME)'` and `heavy cargo nextest run -p temper-client-view --lib -E 'test(=NAME)'` for each test you added or changed.
- `cargo fmt`, then `python3 -I -B temper/scripts/old_forms.py`.
- `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-gallery --all-targets -- -D warnings`, and `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-view -- -D warnings`.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-gallery -p temper-client-journeys -p temper-client-world`. No recorded journey's page should change: the commands draw only in the terminal.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 5. Commits

- **Trailer:** `Look-request: chat-terminal` on every commit, one line above the other trailers; `chat-terminal-light`'s `/font` part, if a commit of its own, `Look-request: chat-terminal-font`.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and, for the account `terminal-close` writes:

- each command and list landed, with its words, or how it differs from `terminal.js`;
- the list's rules landed, with their selectors there and here;
- each rule broken and the failure seen;
- each reader file you edited beyond the header's.
